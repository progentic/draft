use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub(crate) const NATIVE_MENU_EVENT_NAME: &str = "draft://native-menu-action";
const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum NativeMenuEvent {
    NewDocument,
    OpenDocument,
    CloseDocument,
    SaveDocument,
    SaveDocumentAs,
    ExportDocx,
}

pub(crate) fn emit_native_menu_action(app: &AppHandle, event: NativeMenuEvent) {
    let _ = app.emit_to(MAIN_WINDOW_LABEL, NATIVE_MENU_EVENT_NAME, event);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn event_name_is_stable() {
        assert_eq!(NATIVE_MENU_EVENT_NAME, "draft://native-menu-action");
    }

    #[test]
    fn event_payload_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(NativeMenuEvent::SaveDocumentAs).unwrap(),
            json!({ "action": "save_document_as" })
        );
    }
}
