use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Stable Tauri event name for Rust-owned runtime status updates.
pub(crate) const RUNTIME_STATUS_EVENT_NAME: &str = "draft://runtime-status";

/// Typed runtime status update emitted by the trusted Rust process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum RuntimeStatusEvent {
    Ready {
        build_commit: String,
        build_profile: String,
        version: String,
    },
}

/// Failure to deliver a runtime status event to the frontend WebView.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum EmitRuntimeStatusError {
    DeliveryFailed,
}

/// Emits one bounded runtime status update to all current frontend listeners.
pub(crate) fn emit_runtime_status(
    app_handle: &AppHandle,
    event: RuntimeStatusEvent,
) -> Result<(), EmitRuntimeStatusError> {
    app_handle
        .emit(RUNTIME_STATUS_EVENT_NAME, event)
        .map_err(|_| EmitRuntimeStatusError::DeliveryFailed)
}

impl RuntimeStatusEvent {
    pub(crate) fn ready(version: String, build_commit: String, build_profile: String) -> Self {
        Self::Ready {
            build_commit,
            build_profile,
            version,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn event_name_is_stable() {
        assert_eq!(RUNTIME_STATUS_EVENT_NAME, "draft://runtime-status");
    }

    #[test]
    fn event_payload_serialization_is_stable() {
        let event = RuntimeStatusEvent::ready(
            "0.1.0".to_owned(),
            "0123456789abcdef0123456789abcdef01234567".to_owned(),
            "release".to_owned(),
        );

        assert_eq!(
            serde_json::to_value(event).expect("event should serialize"),
            json!({
                "type": "ready",
                "buildCommit": "0123456789abcdef0123456789abcdef01234567",
                "buildProfile": "release",
                "version": "0.1.0"
            }),
        );
    }
}
