use std::{
    collections::HashMap,
    error::Error,
    fmt,
    sync::Mutex,
    time::{Duration, Instant},
};

const PRODUCT_NAME: &str = "DRAFT";
const PRODUCT_URL: &str = "https://github.com/progentic/draft";

/// Maximum time allowed to establish one outbound connection.
pub const NETWORK_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum total time allowed for one bounded outbound request.
pub const NETWORK_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Minimum interval between requests sent to one metadata provider.
pub const PROVIDER_REQUEST_INTERVAL: Duration = Duration::from_secs(1);

/// Largest response body retained for one metadata request.
pub const MAX_METADATA_RESPONSE_BYTES: usize = 1024 * 1024;

/// Maximum process-local backoff after repeated remote rate limits.
pub const MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(60);

/// Shared configured HTTP transport owned by the Rust core.
pub struct NetworkClient {
    http: reqwest::Client,
    request_gate: Mutex<RequestGate>,
}

/// Bounded failures produced while constructing the shared network client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkClientError {
    InvalidApplicationVersion,
    ClientBuildFailed,
}

/// Metadata services with independent request-rate state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NetworkService {
    Crossref,
    SemanticScholar,
    Unpaywall,
}

/// Bounded failures from one centralized metadata request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkRequestError {
    InvalidUrl,
    RateLimited { retry_after_millis: u64 },
    Timeout,
    Offline,
    NotFound,
    AccessDenied,
    ServiceUnavailable,
    RequestRejected { status: u16 },
    ResponseTooLarge,
    ReadFailed,
    ClientUnavailable,
}

#[derive(Debug, Eq, PartialEq)]
struct NetworkClientPolicy {
    user_agent: String,
    connect_timeout: Duration,
    request_timeout: Duration,
}

#[derive(Default)]
struct RequestGate {
    services: HashMap<NetworkService, ServiceRateState>,
}

#[derive(Clone, Copy)]
struct ServiceRateState {
    next_allowed: Instant,
    rate_limit_attempts: u8,
}

impl NetworkClient {
    /// Builds the production client without issuing an external request.
    pub fn new() -> Result<Self, NetworkClientError> {
        build_network_client(env!("CARGO_PKG_VERSION"))
    }

    /// Executes one bounded metadata GET through the centralized transport.
    pub async fn get_metadata(
        &self,
        service: NetworkService,
        url: &str,
    ) -> Result<Vec<u8>, NetworkRequestError> {
        self.reserve_request(service)?;
        let response = self.send_get(url).await?;
        self.handle_response(service, response).await
    }

    fn reserve_request(&self, service: NetworkService) -> Result<(), NetworkRequestError> {
        let mut gate = self.lock_request_gate()?;
        gate.reserve_at(service, Instant::now())
            .map_err(|retry_after_millis| NetworkRequestError::RateLimited { retry_after_millis })
    }

    async fn send_get(&self, url: &str) -> Result<reqwest::Response, NetworkRequestError> {
        let url = validated_https_url(url)?;
        self.http.get(url).send().await.map_err(map_reqwest_error)
    }

    async fn handle_response(
        &self,
        service: NetworkService,
        response: reqwest::Response,
    ) -> Result<Vec<u8>, NetworkRequestError> {
        if response.status().is_success() {
            self.record_success(service)?;
            return read_bounded_response(response).await;
        }
        self.handle_failure_status(service, response)
    }

    fn handle_failure_status(
        &self,
        service: NetworkService,
        response: reqwest::Response,
    ) -> Result<Vec<u8>, NetworkRequestError> {
        let status = response.status().as_u16();
        if status == 429 {
            return Err(self.record_rate_limit(service, retry_after(&response))?);
        }
        self.record_success(service)?;
        Err(classify_response_status(status))
    }

    fn record_success(&self, service: NetworkService) -> Result<(), NetworkRequestError> {
        self.lock_request_gate()?.record_success(service);
        Ok(())
    }

    fn record_rate_limit(
        &self,
        service: NetworkService,
        retry_after: Option<Duration>,
    ) -> Result<NetworkRequestError, NetworkRequestError> {
        let retry_after_millis =
            self.lock_request_gate()?
                .record_rate_limit_at(service, Instant::now(), retry_after);
        Ok(NetworkRequestError::RateLimited { retry_after_millis })
    }

    fn lock_request_gate(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, RequestGate>, NetworkRequestError> {
        self.request_gate
            .lock()
            .map_err(|_| NetworkRequestError::ClientUnavailable)
    }
}

impl fmt::Display for NetworkClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for NetworkClientError {}

impl fmt::Display for NetworkRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for NetworkRequestError {}

impl NetworkClientError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidApplicationVersion => "application version is invalid",
            Self::ClientBuildFailed => "network client could not be constructed",
        }
    }
}

impl NetworkRequestError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidUrl => "metadata request URL is invalid",
            Self::RateLimited { .. } => "metadata service is rate limited",
            Self::Timeout => "metadata request timed out",
            Self::Offline => "metadata service is unreachable",
            Self::NotFound => "metadata record was not found",
            Self::AccessDenied => "metadata service denied access",
            Self::ServiceUnavailable => "metadata service is unavailable",
            Self::RequestRejected { .. } => "metadata request was rejected",
            Self::ResponseTooLarge => "metadata response is too large",
            Self::ReadFailed => "metadata response could not be read",
            Self::ClientUnavailable => "network client state is unavailable",
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
    Ok(NetworkClient {
        http,
        request_gate: Mutex::new(RequestGate::default()),
    })
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

impl RequestGate {
    fn reserve_at(&mut self, service: NetworkService, now: Instant) -> Result<(), u64> {
        let state = self.service_state(service, now);
        if now < state.next_allowed {
            return Err(duration_millis(state.next_allowed.duration_since(now)));
        }
        state.next_allowed = now + PROVIDER_REQUEST_INTERVAL;
        Ok(())
    }

    fn record_success(&mut self, service: NetworkService) {
        if let Some(state) = self.services.get_mut(&service) {
            state.rate_limit_attempts = 0;
        }
    }

    fn record_rate_limit_at(
        &mut self,
        service: NetworkService,
        now: Instant,
        retry_after: Option<Duration>,
    ) -> u64 {
        let state = self.service_state(service, now);
        state.rate_limit_attempts = state.rate_limit_attempts.saturating_add(1);
        let delay = rate_limit_delay(state.rate_limit_attempts, retry_after);
        state.next_allowed = now + delay;
        duration_millis(delay)
    }

    fn service_state(&mut self, service: NetworkService, now: Instant) -> &mut ServiceRateState {
        self.services.entry(service).or_insert(ServiceRateState {
            next_allowed: now,
            rate_limit_attempts: 0,
        })
    }
}

fn rate_limit_delay(attempts: u8, retry_after: Option<Duration>) -> Duration {
    let exponent = u32::from(attempts.saturating_sub(1).min(6));
    let exponential = Duration::from_secs(1_u64 << exponent);
    retry_after
        .unwrap_or_default()
        .max(exponential)
        .min(MAX_RATE_LIMIT_BACKOFF)
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let value = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?;
    parse_retry_after_seconds(Some(value))
}

fn parse_retry_after_seconds(value: Option<&str>) -> Option<Duration> {
    let seconds = value?.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds).min(MAX_RATE_LIMIT_BACKOFF))
}

fn validated_https_url(url: &str) -> Result<reqwest::Url, NetworkRequestError> {
    let url = reqwest::Url::parse(url).map_err(|_| NetworkRequestError::InvalidUrl)?;
    if url.scheme() != "https" {
        return Err(NetworkRequestError::InvalidUrl);
    }
    Ok(url)
}

fn map_reqwest_error(error: reqwest::Error) -> NetworkRequestError {
    classify_transport_failure(error.is_timeout(), error.is_connect())
}

fn classify_transport_failure(is_timeout: bool, is_connect: bool) -> NetworkRequestError {
    if is_timeout {
        NetworkRequestError::Timeout
    } else if is_connect {
        NetworkRequestError::Offline
    } else {
        NetworkRequestError::ReadFailed
    }
}

fn classify_response_status(status: u16) -> NetworkRequestError {
    match status {
        401 | 403 => NetworkRequestError::AccessDenied,
        404 => NetworkRequestError::NotFound,
        500..=599 => NetworkRequestError::ServiceUnavailable,
        _ => NetworkRequestError::RequestRejected { status },
    }
}

async fn read_bounded_response(
    mut response: reqwest::Response,
) -> Result<Vec<u8>, NetworkRequestError> {
    if response.content_length().is_some_and(response_too_large) {
        return Err(NetworkRequestError::ResponseTooLarge);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(map_reqwest_error)? {
        append_response_chunk(&mut body, &chunk)?;
    }
    Ok(body)
}

fn append_response_chunk(body: &mut Vec<u8>, chunk: &[u8]) -> Result<(), NetworkRequestError> {
    let new_length = body
        .len()
        .checked_add(chunk.len())
        .ok_or(NetworkRequestError::ResponseTooLarge)?;
    if new_length > MAX_METADATA_RESPONSE_BYTES {
        return Err(NetworkRequestError::ResponseTooLarge);
    }
    body.extend_from_slice(chunk);
    Ok(())
}

fn response_too_large(length: u64) -> bool {
    length > MAX_METADATA_RESPONSE_BYTES as u64
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().clamp(1, u128::from(u64::MAX)) as u64
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
