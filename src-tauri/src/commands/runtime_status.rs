use serde::{Deserialize, Serialize};

use crate::application::runtime_status::{
    RuntimeStatus, RuntimeStatusError, current_runtime_status,
};
use crate::events::runtime_status::{
    EmitRuntimeStatusError, RuntimeStatusEvent, emit_runtime_status,
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
    EventDeliveryFailed,
}

/// Reports whether the trusted Rust runtime has valid compiled application metadata.
#[tauri::command]
pub(crate) fn get_runtime_status(
    app_handle: tauri::AppHandle,
    request: GetRuntimeStatusRequest,
) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError> {
    let response = runtime_status_response(request)?;
    emit_runtime_status(&app_handle, response.runtime_event())?;
    Ok(response)
}

fn runtime_status_response(
    request: GetRuntimeStatusRequest,
) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError> {
    let GetRuntimeStatusRequest {} = request;
    let status = current_runtime_status().map_err(GetRuntimeStatusError::from)?;
    Ok(GetRuntimeStatusResponse::from(status))
}

impl GetRuntimeStatusResponse {
    fn runtime_event(&self) -> RuntimeStatusEvent {
        RuntimeStatusEvent::ready(self.version.clone())
    }
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

impl From<EmitRuntimeStatusError> for GetRuntimeStatusError {
    fn from(error: EmitRuntimeStatusError) -> Self {
        match error {
            EmitRuntimeStatusError::DeliveryFailed => Self::EventDeliveryFailed,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        tauri::AppHandle,
        GetRuntimeStatusRequest,
    ) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError> = get_runtime_status;

    #[test]
    fn command_signature_is_typed() {
        assert_eq!(
            std::any::type_name_of_val(&TYPED_COMMAND),
            std::any::type_name::<
                fn(
                    tauri::AppHandle,
                    GetRuntimeStatusRequest,
                ) -> Result<GetRuntimeStatusResponse, GetRuntimeStatusError>,
            >(),
        );
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
        let response = runtime_status_response(GetRuntimeStatusRequest {})
            .expect("runtime status should succeed");

        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            json!({ "version": env!("CARGO_PKG_VERSION") }),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            GetRuntimeStatusError::from(RuntimeStatusError::MissingVersion),
            GetRuntimeStatusError::from(EmitRuntimeStatusError::DeliveryFailed),
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "invalid_application_version" },
                { "code": "event_delivery_failed" }
            ]),
        );
    }
}
