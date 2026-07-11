use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::network::connectivity::{ConnectivityMode, ConnectivityPolicy, ConnectivityPolicyError};

/// Empty bounded request for the current connectivity mode.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GetConnectivityModeRequest {}

/// Closed connectivity mode requested for the current session.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetConnectivityModeRequest {
    mode: ConnectivityModeDto,
}

/// Effective Rust-owned connectivity mode.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConnectivityModeResponse {
    mode: ConnectivityModeDto,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConnectivityModeDto {
    Online,
    Offline,
}

/// Stable failure exposed when session connectivity state cannot be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ConnectivityModeCommandError {
    ConnectivityUnavailable,
}

/// Returns the effective Rust-owned connectivity mode.
#[tauri::command]
pub(crate) fn get_connectivity_mode(
    connectivity: State<'_, Arc<ConnectivityPolicy>>,
    _request: GetConnectivityModeRequest,
) -> Result<ConnectivityModeResponse, ConnectivityModeCommandError> {
    connectivity
        .mode()
        .map(ConnectivityModeResponse::from)
        .map_err(Into::into)
}

/// Replaces the effective connectivity mode for the current session.
#[tauri::command]
pub(crate) fn set_connectivity_mode(
    connectivity: State<'_, Arc<ConnectivityPolicy>>,
    request: SetConnectivityModeRequest,
) -> Result<ConnectivityModeResponse, ConnectivityModeCommandError> {
    connectivity
        .set_mode(request.mode.into())
        .map(ConnectivityModeResponse::from)
        .map_err(Into::into)
}

impl From<ConnectivityMode> for ConnectivityModeResponse {
    fn from(mode: ConnectivityMode) -> Self {
        Self { mode: mode.into() }
    }
}

impl From<ConnectivityMode> for ConnectivityModeDto {
    fn from(mode: ConnectivityMode) -> Self {
        match mode {
            ConnectivityMode::Online => Self::Online,
            ConnectivityMode::Offline => Self::Offline,
        }
    }
}

impl From<ConnectivityModeDto> for ConnectivityMode {
    fn from(mode: ConnectivityModeDto) -> Self {
        match mode {
            ConnectivityModeDto::Online => Self::Online,
            ConnectivityModeDto::Offline => Self::Offline,
        }
    }
}

impl From<ConnectivityPolicyError> for ConnectivityModeCommandError {
    fn from(_error: ConnectivityPolicyError) -> Self {
        Self::ConnectivityUnavailable
    }
}

#[cfg(test)]
mod get_tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: for<'a> fn(
        State<'a, Arc<ConnectivityPolicy>>,
        GetConnectivityModeRequest,
    ) -> Result<
        ConnectivityModeResponse,
        ConnectivityModeCommandError,
    > = get_connectivity_mode;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(serde_json::from_value::<GetConnectivityModeRequest>(json!({})).is_ok());
        assert!(
            serde_json::from_value::<GetConnectivityModeRequest>(json!({ "mode": "online" }))
                .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ConnectivityModeResponse::from(ConnectivityMode::Online)).unwrap(),
            json!({ "mode": "online" })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ConnectivityModeCommandError::ConnectivityUnavailable).unwrap(),
            json!({ "code": "connectivity_unavailable" })
        );
    }
}

#[cfg(test)]
mod set_tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: for<'a> fn(
        State<'a, Arc<ConnectivityPolicy>>,
        SetConnectivityModeRequest,
    ) -> Result<
        ConnectivityModeResponse,
        ConnectivityModeCommandError,
    > = set_connectivity_mode;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(
            serde_json::from_value::<SetConnectivityModeRequest>(json!({ "mode": "offline" }))
                .is_ok()
        );
        assert!(
            serde_json::from_value::<SetConnectivityModeRequest>(json!({ "mode": "automatic" }))
                .is_err()
        );
        assert!(
            serde_json::from_value::<SetConnectivityModeRequest>(
                json!({ "mode": "online", "persist": true })
            )
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ConnectivityModeResponse::from(ConnectivityMode::Offline))
                .unwrap(),
            json!({ "mode": "offline" })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ConnectivityModeCommandError::ConnectivityUnavailable).unwrap(),
            json!({ "code": "connectivity_unavailable" })
        );
    }
}
