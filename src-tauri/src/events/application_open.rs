use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub(crate) const APPLICATION_OPEN_EVENT_NAME: &str = "draft://application-open";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ApplicationOpenEvent {
    Available,
    QueueUnavailable,
}

pub(crate) fn emit_application_open_event(app: &AppHandle, event: ApplicationOpenEvent) {
    let _ = app.emit(APPLICATION_OPEN_EVENT_NAME, event);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn event_name_is_stable() {
        assert_eq!(APPLICATION_OPEN_EVENT_NAME, "draft://application-open");
    }

    #[test]
    fn event_payload_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ApplicationOpenEvent::Available).unwrap(),
            json!({ "type": "available" })
        );
        assert_eq!(
            serde_json::to_value(ApplicationOpenEvent::QueueUnavailable).unwrap(),
            json!({ "type": "queue_unavailable" })
        );
    }
}
