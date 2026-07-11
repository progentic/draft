use super::*;
use crate::network::connectivity::{ConnectivityMode, ConnectivityPolicy};

fn network_client() -> NetworkClient {
    NetworkClient::new(Arc::new(ConnectivityPolicy::default()))
        .expect("manifest metadata should build the client")
}

#[test]
fn current_manifest_builds_network_client() {
    network_client();
}

#[tokio::test]
async fn offline_policy_denies_before_url_or_transport_work() {
    let connectivity = Arc::new(ConnectivityPolicy::default());
    connectivity.set_mode(ConnectivityMode::Offline).unwrap();
    let client = NetworkClient::new(connectivity).unwrap();

    assert_eq!(
        client
            .get_metadata(NetworkService::Crossref, "not a URL")
            .await,
        Err(NetworkRequestError::Offline)
    );
}

#[tokio::test]
async fn online_policy_preserves_url_validation() {
    let client = network_client();

    assert_eq!(
        client
            .get_metadata(NetworkService::Crossref, "not a URL")
            .await,
        Err(NetworkRequestError::InvalidUrl)
    );
}

#[test]
fn user_agent_policy_is_deterministic() {
    let policy = network_client_policy("1.2.3-alpha+5").unwrap();

    assert_eq!(
        policy.user_agent,
        "DRAFT/1.2.3-alpha+5 (+https://github.com/progentic/draft)"
    );
}

#[test]
fn request_and_connect_timeouts_are_explicit() {
    let policy = network_client_policy("1.2.3").unwrap();

    assert_eq!(policy.connect_timeout, Duration::from_secs(10));
    assert_eq!(policy.request_timeout, Duration::from_secs(30));
}

#[test]
fn invalid_application_versions_fail() {
    for version in ["", "   ", "1/2", "1\n2", "versión"] {
        assert_eq!(
            network_client_policy(version),
            Err(NetworkClientError::InvalidApplicationVersion)
        );
    }
}

#[test]
fn network_client_failure_shape_is_bounded() {
    assert_eq!(
        NetworkClientError::InvalidApplicationVersion.to_string(),
        "application version is invalid"
    );
    assert_eq!(
        NetworkClientError::ClientBuildFailed.to_string(),
        "network client could not be constructed"
    );
}

#[test]
fn request_gate_enforces_per_service_interval() {
    let now = Instant::now();
    let mut gate = RequestGate::default();

    assert_eq!(gate.reserve_at(NetworkService::Crossref, now), Ok(()));
    assert_eq!(
        gate.reserve_at(NetworkService::Crossref, now),
        Err(PROVIDER_REQUEST_INTERVAL.as_millis() as u64)
    );
    assert_eq!(
        gate.reserve_at(NetworkService::Crossref, now + PROVIDER_REQUEST_INTERVAL),
        Ok(())
    );
}

#[test]
fn request_gate_keeps_services_independent() {
    let now = Instant::now();
    let mut gate = RequestGate::default();

    assert_eq!(gate.reserve_at(NetworkService::Crossref, now), Ok(()));
    assert_eq!(
        gate.reserve_at(NetworkService::SemanticScholar, now),
        Ok(())
    );
    assert_eq!(gate.reserve_at(NetworkService::Unpaywall, now), Ok(()));
}

#[test]
fn server_rate_limits_apply_exponential_backoff() {
    let now = Instant::now();
    let mut gate = RequestGate::default();

    assert_eq!(
        gate.record_rate_limit_at(NetworkService::Crossref, now, None),
        1_000
    );
    assert_eq!(
        gate.record_rate_limit_at(NetworkService::Crossref, now, None),
        2_000
    );
    assert_eq!(
        gate.record_rate_limit_at(NetworkService::Crossref, now, Some(Duration::from_secs(10))),
        10_000
    );
    for _ in 0..10 {
        gate.record_rate_limit_at(NetworkService::Crossref, now, None);
    }
    assert_eq!(
        gate.reserve_at(NetworkService::Crossref, now),
        Err(MAX_RATE_LIMIT_BACKOFF.as_millis() as u64)
    );
}

#[test]
fn retry_after_seconds_are_bounded() {
    assert_eq!(
        parse_retry_after_seconds(Some(" 12 ")),
        Some(Duration::from_secs(12))
    );
    assert_eq!(
        parse_retry_after_seconds(Some("120")),
        Some(MAX_RATE_LIMIT_BACKOFF)
    );
    assert_eq!(parse_retry_after_seconds(Some("date")), None);
    assert_eq!(parse_retry_after_seconds(None), None);
}

#[test]
fn transport_failures_are_typed() {
    assert_eq!(
        classify_transport_failure(true, true),
        NetworkRequestError::Timeout
    );
    assert_eq!(
        classify_transport_failure(false, true),
        NetworkRequestError::Offline
    );
    assert_eq!(
        classify_transport_failure(false, false),
        NetworkRequestError::ReadFailed
    );
}

#[test]
fn response_statuses_are_typed() {
    assert_eq!(
        classify_response_status(401),
        NetworkRequestError::AccessDenied
    );
    assert_eq!(classify_response_status(404), NetworkRequestError::NotFound);
    assert_eq!(
        classify_response_status(503),
        NetworkRequestError::ServiceUnavailable
    );
    assert_eq!(
        classify_response_status(418),
        NetworkRequestError::RequestRejected { status: 418 }
    );
}

#[test]
fn response_limit_rejects_oversized_body() {
    let mut body = vec![0; MAX_METADATA_RESPONSE_BYTES];

    assert_eq!(
        append_response_chunk(&mut body, &[1]),
        Err(NetworkRequestError::ResponseTooLarge)
    );
}
