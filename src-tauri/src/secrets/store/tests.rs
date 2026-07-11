use std::{collections::HashMap, io, sync::Mutex};

use super::*;

#[test]
fn identifiers_accept_only_bounded_normalized_service_names() {
    for valid in ["model-provider", "metadata2", "a"] {
        assert!(SecretId::service_api_key(valid).is_ok());
    }
    for invalid in [
        "",
        "ModelProvider",
        "model_provider",
        "-model",
        "model-",
        "model/key",
    ] {
        assert_eq!(
            SecretId::service_api_key(invalid),
            Err(SecretStoreError::InvalidIdentifier)
        );
    }
    assert_eq!(
        SecretId::service_api_key(&"a".repeat(MAX_INTEGRATION_NAME_BYTES + 1)),
        Err(SecretStoreError::InvalidIdentifier)
    );
}

#[test]
fn secret_values_are_nonempty_bounded_and_not_in_errors() {
    assert_eq!(
        SecretValue::new(Vec::new()).err(),
        Some(SecretStoreError::EmptySecret)
    );
    assert_eq!(
        SecretValue::new(vec![b'x'; MAX_SECRET_BYTES + 1]).err(),
        Some(SecretStoreError::SecretTooLong)
    );
    let secret = SecretValue::new(b"private-test-value".to_vec()).unwrap();
    assert_eq!(secret.expose_secret(), b"private-test-value");
    assert!(
        !SecretStoreError::StoreUnavailable
            .to_string()
            .contains("private-test-value")
    );
}

#[test]
fn store_load_replace_and_delete_are_deterministic() {
    let backend = Arc::new(MemorySecretBackend::default());
    let store = SecretStore::with_backend(backend.clone());
    let id = SecretId::service_api_key("model-provider").unwrap();

    assert!(store.load(&id).unwrap().is_none());
    store
        .store(&id, SecretValue::new(b"first".to_vec()).unwrap())
        .unwrap();
    assert_eq!(store.load(&id).unwrap().unwrap().expose_secret(), b"first");
    store
        .store(&id, SecretValue::new(b"second".to_vec()).unwrap())
        .unwrap();
    assert_eq!(store.load(&id).unwrap().unwrap().expose_secret(), b"second");
    assert_eq!(store.delete(&id), Ok(SecretDeleteOutcome::Deleted));
    assert_eq!(store.delete(&id), Ok(SecretDeleteOutcome::NotFound));
}

#[test]
fn malformed_backend_values_fail_as_invalid_stored_secrets() {
    let backend = Arc::new(MemorySecretBackend::default());
    let store = SecretStore::with_backend(backend.clone());
    let id = SecretId::service_api_key("model-provider").unwrap();

    backend.insert_raw(&id, Vec::new());
    assert_eq!(
        store.load(&id).err(),
        Some(SecretStoreError::InvalidStoredSecret)
    );
    backend.insert_raw(&id, vec![b'x'; MAX_SECRET_BYTES + 1]);
    assert_eq!(
        store.load(&id).err(),
        Some(SecretStoreError::InvalidStoredSecret)
    );
}

#[test]
fn backend_failures_map_to_closed_store_errors() {
    let mappings = [
        (
            SecretBackendError::AccessDenied,
            SecretStoreError::AccessDenied,
        ),
        (
            SecretBackendError::Unavailable,
            SecretStoreError::StoreUnavailable,
        ),
        (
            SecretBackendError::Ambiguous,
            SecretStoreError::AmbiguousEntry,
        ),
        (
            SecretBackendError::InvalidData,
            SecretStoreError::InvalidStoredSecret,
        ),
        (
            SecretBackendError::InvalidInput,
            SecretStoreError::InvalidIdentifier,
        ),
        (
            SecretBackendError::Unsupported,
            SecretStoreError::Unsupported,
        ),
    ];

    for (backend, expected) in mappings {
        assert_eq!(map_backend_error(backend), expected);
    }
}

#[test]
fn keyring_failures_drop_raw_details_during_mapping() {
    let platform = || io::Error::other("private native detail");
    let mappings = [
        (keyring::Error::NoEntry, SecretBackendError::NotFound),
        (
            keyring::Error::NoStorageAccess(Box::new(platform())),
            SecretBackendError::AccessDenied,
        ),
        (
            keyring::Error::PlatformFailure(Box::new(platform())),
            SecretBackendError::Unavailable,
        ),
        (
            keyring::Error::Ambiguous(Vec::new()),
            SecretBackendError::Ambiguous,
        ),
        (
            keyring::Error::BadEncoding(vec![0xff]),
            SecretBackendError::InvalidData,
        ),
        (
            keyring::Error::BadDataFormat(vec![0xff], Box::new(platform())),
            SecretBackendError::InvalidData,
        ),
        (
            keyring::Error::BadStoreFormat("private native detail".to_owned()),
            SecretBackendError::InvalidData,
        ),
        (
            keyring::Error::TooLong("account".to_owned(), 1),
            SecretBackendError::InvalidInput,
        ),
        (
            keyring::Error::Invalid("account".to_owned(), "detail".to_owned()),
            SecretBackendError::InvalidInput,
        ),
        (
            keyring::Error::NoDefaultStore,
            SecretBackendError::Unavailable,
        ),
        (
            keyring::Error::NotSupportedByStore("detail".to_owned()),
            SecretBackendError::Unsupported,
        ),
    ];

    for (error, expected) in mappings {
        assert_eq!(map_keyring_error(error), expected);
    }
}

#[test]
fn native_store_is_safe_to_manage_without_accessing_a_credential() {
    fn require_send_sync<T: Send + Sync>() {}

    require_send_sync::<SecretStore>();
    let _store = SecretStore::native();
}

#[derive(Default)]
struct MemorySecretBackend {
    values: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemorySecretBackend {
    fn insert_raw(&self, id: &SecretId, value: Vec<u8>) {
        self.values
            .lock()
            .unwrap()
            .insert(id.account().to_owned(), value);
    }
}

impl SecretBackend for MemorySecretBackend {
    fn store(&self, id: &SecretId, secret: &SecretValue) -> Result<(), SecretBackendError> {
        self.values
            .lock()
            .unwrap()
            .insert(id.account().to_owned(), secret.expose_secret().to_vec());
        Ok(())
    }

    fn load(&self, id: &SecretId) -> Result<Zeroizing<Vec<u8>>, SecretBackendError> {
        self.values
            .lock()
            .unwrap()
            .get(id.account())
            .cloned()
            .map(Zeroizing::new)
            .ok_or(SecretBackendError::NotFound)
    }

    fn delete(&self, id: &SecretId) -> Result<(), SecretBackendError> {
        self.values
            .lock()
            .unwrap()
            .remove(id.account())
            .map(|_| ())
            .ok_or(SecretBackendError::NotFound)
    }
}
