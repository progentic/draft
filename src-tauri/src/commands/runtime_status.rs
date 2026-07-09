use serde::{Deserialize, Serialize};

use crate::application::runtime_status::{
    RuntimeStatus, RuntimeStatusError, current_runtime_status,
};

/// Bounded request envelope for `get_runtime_status`.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GetRuntimeStatusRequest {}

/// Version information returned by the Rust runtime status command.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetRuntimeStatusResponse {
    version: String,
}

/// Stable error codes exposed by `get_runtime_status` over Tauri IPC.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum GetRuntimeStatusError {
    InvalidApplicationVersion,
}

/// Reports whether the trusted Rust runtime has valid compiled application metadata.
#[tauri::command]
pub(crate) fn get_runtime_status(
    request: GetRuntimeStatusRequest,
) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError> {
    let GetRuntimeStatusRequest {} = request;
    let status = current_runtime_status().map_err(GetRuntimeStatusError::from)?;
    Ok(GetRuntimeStatusResponse::from(status))
}

impl From<RuntimeStatus> for GetRuntimeStatusResponse {
    fn from(status: RuntimeStatus) -> Self {
        Self {
            version: status.into_version(),
        }
    }
}

impl From<RuntimeStatusError> for GetRuntimeStatusError {
    fn from(error: RuntimeStatusError) -> Self {
        match error {
            RuntimeStatusError::MissingVersion => Self::InvalidApplicationVersion,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        GetRuntimeStatusRequest,
    ) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError> = get_runtime_status;

    #[test]
    fn command_signature_is_typed() {
        let response = TYPED_COMMAND(GetRuntimeStatusRequest {})
            .expect("compiled version should produce runtime status");

        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<GetRuntimeStatusRequest>(json!({}));
        let unknown_field =
            serde_json::from_value::<GetRuntimeStatusRequest>(json!({ "extra": true }));

        assert_eq!(
            request.expect("empty request should deserialize"),
            GetRuntimeStatusRequest {},
        );
        assert!(unknown_field.is_err());
    }

    #[test]
    fn response_serialization_is_stable() {
        let response =
            get_runtime_status(GetRuntimeStatusRequest {}).expect("runtime status should succeed");

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            json!({ "version": env!("CARGO_PKG_VERSION") }),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let error = GetRuntimeStatusError::from(RuntimeStatusError::MissingVersion);

        assert_eq!(
            serde_json::to_value(error).expect("error should serialize"),
            json!({ "code": "invalid_application_version" }),
        );
    }
}
