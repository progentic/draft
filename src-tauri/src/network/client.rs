use std::{error::Error, fmt, time::Duration};

const PRODUCT_NAME: &str = "DRAFT";
const PRODUCT_URL: &str = "https://github.com/progentic/draft";

/// Maximum time allowed to establish one outbound connection.
pub const NETWORK_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum total time allowed for one bounded outbound request.
pub const NETWORK_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Shared configured HTTP transport owned by the Rust core.
pub struct NetworkClient {
    _http: reqwest::Client,
}

/// Bounded failures produced while constructing the shared network client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkClientError {
    InvalidApplicationVersion,
    ClientBuildFailed,
}

#[derive(Debug, Eq, PartialEq)]
struct NetworkClientPolicy {
    user_agent: String,
    connect_timeout: Duration,
    request_timeout: Duration,
}

impl NetworkClient {
    /// Builds the production client without issuing an external request.
    pub fn new() -> Result<Self, NetworkClientError> {
        build_network_client(env!("CARGO_PKG_VERSION"))
    }
}

impl fmt::Display for NetworkClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for NetworkClientError {}

impl NetworkClientError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidApplicationVersion => "application version is invalid",
            Self::ClientBuildFailed => "network client could not be constructed",
        }
    }
}

fn build_network_client(version: &str) -> Result<NetworkClient, NetworkClientError> {
    let policy = network_client_policy(version)?;
    let http = reqwest::Client::builder()
        .user_agent(&policy.user_agent)
        .connect_timeout(policy.connect_timeout)
        .timeout(policy.request_timeout)
        .https_only(true)
        .build()
        .map_err(|_| NetworkClientError::ClientBuildFailed)?;
    Ok(NetworkClient { _http: http })
}

fn network_client_policy(version: &str) -> Result<NetworkClientPolicy, NetworkClientError> {
    let version = validated_application_version(version)?;
    Ok(NetworkClientPolicy {
        user_agent: format!("{PRODUCT_NAME}/{version} (+{PRODUCT_URL})"),
        connect_timeout: NETWORK_CONNECT_TIMEOUT,
        request_timeout: NETWORK_REQUEST_TIMEOUT,
    })
}

fn validated_application_version(version: &str) -> Result<&str, NetworkClientError> {
    let version = version.trim();
    if version.is_empty() || !version.chars().all(is_version_character) {
        return Err(NetworkClientError::InvalidApplicationVersion);
    }
    Ok(version)
}

fn is_version_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '+')
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
