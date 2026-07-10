use super::*;

#[test]
fn current_manifest_builds_network_client() {
    NetworkClient::new().expect("manifest metadata should build the client");
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
