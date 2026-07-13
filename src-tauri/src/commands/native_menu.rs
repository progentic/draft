use serde::{Deserialize, Serialize};
use tauri::State;

use crate::desktop_menu::{NativeMenuAvailability, NativeMenuItems, NativeMenuUpdateError};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NativeMenuStateRequest {
    can_new: bool,
    can_open: bool,
    can_close: bool,
    can_save: bool,
    can_save_as: bool,
    can_export: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NativeMenuStateResponse {
    applied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum NativeMenuCommandError {
    MenuUpdateFailed,
}

#[tauri::command]
pub(crate) fn set_native_menu_state(
    menu: State<'_, NativeMenuItems>,
    request: NativeMenuStateRequest,
) -> Result<NativeMenuStateResponse, NativeMenuCommandError> {
    menu.apply(request.into())?;
    Ok(NativeMenuStateResponse { applied: true })
}

impl From<NativeMenuStateRequest> for NativeMenuAvailability {
    fn from(request: NativeMenuStateRequest) -> Self {
        Self {
            can_new: request.can_new,
            can_open: request.can_open,
            can_close: request.can_close,
            can_save: request.can_save,
            can_save_as: request.can_save_as,
            can_export: request.can_export,
        }
    }
}

impl From<NativeMenuUpdateError> for NativeMenuCommandError {
    fn from(_error: NativeMenuUpdateError) -> Self {
        Self::MenuUpdateFailed
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: for<'a> fn(
        State<'a, NativeMenuItems>,
        NativeMenuStateRequest,
    )
        -> Result<NativeMenuStateResponse, NativeMenuCommandError> = set_native_menu_state;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<NativeMenuStateRequest>(json!({
            "canNew": true,
            "canOpen": true,
            "canClose": false,
            "canSave": false,
            "canSaveAs": false,
            "canExport": false
        }))
        .unwrap();

        assert!(NativeMenuAvailability::from(request).can_new);
        assert!(
            serde_json::from_value::<NativeMenuStateRequest>(json!({
                "canNew": true,
                "canOpen": true,
                "canClose": false,
                "canSave": false,
                "canSaveAs": false,
                "canExport": false,
                "path": "/private/document.draft"
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(NativeMenuStateResponse { applied: true }).unwrap(),
            json!({ "applied": true })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(NativeMenuCommandError::MenuUpdateFailed).unwrap(),
            json!({ "code": "menu_update_failed" })
        );
    }
}
