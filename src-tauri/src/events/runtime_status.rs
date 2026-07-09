use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Stable Tauri event name for Rust-owned runtime status updates.
pub(crate) const RUNTIME_STATUS_EVENT_NAME: &str = "draft://runtime-status";

/// Typed runtime status update emitted by the trusted Rust process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum RuntimeStatusEvent {
    Ready { version: String },
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
    pub(crate) fn ready(version: String) -> Self {
        Self::Ready { version }
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
        let event = RuntimeStatusEvent::ready("0.1.0".to_owned());

        assert_eq!(
            serde_json::to_value(event).expect("event should serialize"),
            json!({ "type": "ready", "version": "0.1.0" }),
        );
    }
}
