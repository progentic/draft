use std::{error::Error, fmt, sync::Arc};

use zeroize::{Zeroize, Zeroizing};

const NATIVE_SERVICE_NAME: &str = "com.progentic.draft";
const API_KEY_ACCOUNT_PREFIX: &str = "service-api-key/";
pub const MAX_INTEGRATION_NAME_BYTES: usize = 64;
pub const MAX_SECRET_BYTES: usize = 4_096;

/// One normalized OS credential-manager slot for a service API key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretId {
    account: String,
}

/// Secret bytes that are cleared when Rust releases the value.
pub struct SecretValue {
    bytes: Zeroizing<Vec<u8>>,
}

/// Result of an idempotent secret deletion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretDeleteOutcome {
    Deleted,
    NotFound,
}

/// Closed failures from the Rust-owned secret boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretStoreError {
    InvalidIdentifier,
    EmptySecret,
    SecretTooLong,
    AccessDenied,
    StoreUnavailable,
    AmbiguousEntry,
    InvalidStoredSecret,
    Unsupported,
}

/// Rust-owned access to the platform-native credential manager.
pub struct SecretStore {
    backend: Arc<dyn SecretBackend>,
}

impl SecretId {
    /// Creates one internal service API-key slot.
    pub fn service_api_key(integration: &str) -> Result<Self, SecretStoreError> {
        validate_integration_name(integration)?;
        Ok(Self {
            account: format!("{API_KEY_ACCOUNT_PREFIX}{integration}"),
        })
    }

    fn account(&self) -> &str {
        &self.account
    }
}

impl SecretValue {
    /// Takes ownership of validated secret bytes.
    pub fn new(bytes: Vec<u8>) -> Result<Self, SecretStoreError> {
        let bytes = Zeroizing::new(bytes);
        validate_secret_bytes(&bytes)?;
        Ok(Self { bytes })
    }

    /// Exposes the secret only inside Rust-owned trusted work.
    pub fn expose_secret(&self) -> &[u8] {
        &self.bytes
    }
}

impl SecretStore {
    /// Creates a lazy adapter for the operating-system credential manager.
    pub fn native() -> Self {
        Self::with_backend(Arc::new(NativeSecretBackend))
    }

    /// Stores or replaces one service API key.
    pub fn store(&self, id: &SecretId, secret: SecretValue) -> Result<(), SecretStoreError> {
        self.backend.store(id, &secret).map_err(map_backend_error)
    }

    /// Loads one service API key without exposing it outside Rust.
    pub fn load(&self, id: &SecretId) -> Result<Option<SecretValue>, SecretStoreError> {
        match self.backend.load(id) {
            Ok(bytes) => SecretValue::from_backend(bytes)
                .map(Some)
                .map_err(|_| SecretStoreError::InvalidStoredSecret),
            Err(SecretBackendError::NotFound) => Ok(None),
            Err(error) => Err(map_backend_error(error)),
        }
    }

    /// Deletes one service API key idempotently.
    pub fn delete(&self, id: &SecretId) -> Result<SecretDeleteOutcome, SecretStoreError> {
        match self.backend.delete(id) {
            Ok(()) => Ok(SecretDeleteOutcome::Deleted),
            Err(SecretBackendError::NotFound) => Ok(SecretDeleteOutcome::NotFound),
            Err(error) => Err(map_backend_error(error)),
        }
    }

    fn with_backend(backend: Arc<dyn SecretBackend>) -> Self {
        Self { backend }
    }
}

impl SecretValue {
    fn from_backend(bytes: Zeroizing<Vec<u8>>) -> Result<Self, SecretStoreError> {
        validate_secret_bytes(&bytes).map(|()| Self { bytes })
    }
}

impl fmt::Display for SecretStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentifier => "secret identifier is invalid",
            Self::EmptySecret => "secret value is empty",
            Self::SecretTooLong => "secret value exceeds the supported limit",
            Self::AccessDenied => "credential manager access was denied",
            Self::StoreUnavailable => "credential manager is unavailable",
            Self::AmbiguousEntry => "credential manager entry is ambiguous",
            Self::InvalidStoredSecret => "stored secret is invalid",
            Self::Unsupported => "credential manager operation is unsupported",
        })
    }
}

impl Error for SecretStoreError {}

trait SecretBackend: Send + Sync {
    fn store(&self, id: &SecretId, secret: &SecretValue) -> Result<(), SecretBackendError>;
    fn load(&self, id: &SecretId) -> Result<Zeroizing<Vec<u8>>, SecretBackendError>;
    fn delete(&self, id: &SecretId) -> Result<(), SecretBackendError>;
}

struct NativeSecretBackend;

impl SecretBackend for NativeSecretBackend {
    fn store(&self, id: &SecretId, secret: &SecretValue) -> Result<(), SecretBackendError> {
        native_entry(id)?
            .set_secret(secret.expose_secret())
            .map_err(map_keyring_error)
    }

    fn load(&self, id: &SecretId) -> Result<Zeroizing<Vec<u8>>, SecretBackendError> {
        native_entry(id)?
            .get_secret()
            .map(Zeroizing::new)
            .map_err(map_keyring_error)
    }

    fn delete(&self, id: &SecretId) -> Result<(), SecretBackendError> {
        native_entry(id)?
            .delete_credential()
            .map_err(map_keyring_error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SecretBackendError {
    NotFound,
    AccessDenied,
    Unavailable,
    Ambiguous,
    InvalidData,
    InvalidInput,
    Unsupported,
}

fn validate_integration_name(integration: &str) -> Result<(), SecretStoreError> {
    let valid_length = !integration.is_empty() && integration.len() <= MAX_INTEGRATION_NAME_BYTES;
    let valid_edges = integration
        .as_bytes()
        .first()
        .zip(integration.as_bytes().last())
        .is_some_and(|(first, last)| first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric());
    let valid_characters = integration
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if valid_length && valid_edges && valid_characters {
        Ok(())
    } else {
        Err(SecretStoreError::InvalidIdentifier)
    }
}

fn validate_secret_bytes(bytes: &[u8]) -> Result<(), SecretStoreError> {
    match bytes.len() {
        0 => Err(SecretStoreError::EmptySecret),
        length if length > MAX_SECRET_BYTES => Err(SecretStoreError::SecretTooLong),
        _ => Ok(()),
    }
}

fn map_backend_error(error: SecretBackendError) -> SecretStoreError {
    match error {
        SecretBackendError::NotFound | SecretBackendError::Unavailable => {
            SecretStoreError::StoreUnavailable
        }
        SecretBackendError::AccessDenied => SecretStoreError::AccessDenied,
        SecretBackendError::Ambiguous => SecretStoreError::AmbiguousEntry,
        SecretBackendError::InvalidData => SecretStoreError::InvalidStoredSecret,
        SecretBackendError::InvalidInput => SecretStoreError::InvalidIdentifier,
        SecretBackendError::Unsupported => SecretStoreError::Unsupported,
    }
}

fn native_entry(id: &SecretId) -> Result<keyring::Entry, SecretBackendError> {
    keyring::Entry::new(NATIVE_SERVICE_NAME, id.account()).map_err(map_keyring_error)
}

fn map_keyring_error(error: keyring::Error) -> SecretBackendError {
    match error {
        keyring::Error::NoEntry => SecretBackendError::NotFound,
        keyring::Error::NoStorageAccess(_) => SecretBackendError::AccessDenied,
        keyring::Error::Ambiguous(_) => SecretBackendError::Ambiguous,
        keyring::Error::BadEncoding(bytes) => zeroize_bytes(bytes),
        keyring::Error::BadDataFormat(bytes, _) => zeroize_bytes(bytes),
        keyring::Error::BadStoreFormat(detail) => {
            zeroize_text(detail, SecretBackendError::InvalidData)
        }
        keyring::Error::TooLong(attribute, _) => {
            zeroize_text(attribute, SecretBackendError::InvalidInput)
        }
        keyring::Error::Invalid(attribute, detail) => zeroize_invalid_input(attribute, detail),
        keyring::Error::NotSupportedByStore(detail) => {
            zeroize_text(detail, SecretBackendError::Unsupported)
        }
        keyring::Error::PlatformFailure(_) | keyring::Error::NoDefaultStore => {
            SecretBackendError::Unavailable
        }
        _ => SecretBackendError::Unavailable,
    }
}

fn zeroize_bytes(mut bytes: Vec<u8>) -> SecretBackendError {
    bytes.zeroize();
    SecretBackendError::InvalidData
}

fn zeroize_text(mut text: String, result: SecretBackendError) -> SecretBackendError {
    text.zeroize();
    result
}

fn zeroize_invalid_input(mut attribute: String, mut detail: String) -> SecretBackendError {
    attribute.zeroize();
    detail.zeroize();
    SecretBackendError::InvalidInput
}

#[cfg(test)]
mod tests;
