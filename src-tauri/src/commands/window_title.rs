use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

const MAX_DISPLAY_NAME_BYTES: usize = 255;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WindowTitleRequest {
    display_name: Option<String>,
    unsaved: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct WindowTitleResponse {
    applied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum WindowTitleError {
    InvalidTitle,
    WindowUpdateFailed,
}

#[tauri::command]
pub(crate) fn set_window_title(
    window: WebviewWindow,
    request: WindowTitleRequest,
) -> Result<WindowTitleResponse, WindowTitleError> {
    let title = validated_window_title(request)?;
    window
        .set_title(&title)
        .map_err(|_| WindowTitleError::WindowUpdateFailed)?;
    Ok(WindowTitleResponse { applied: true })
}

fn validated_window_title(request: WindowTitleRequest) -> Result<String, WindowTitleError> {
    let Some(display_name) = request.display_name else {
        return (!request.unsaved)
            .then(|| "DRAFT".to_owned())
            .ok_or(WindowTitleError::InvalidTitle);
    };
    require_display_name(&display_name)?;
    Ok(if request.unsaved {
        format!("{display_name} — Unsaved — DRAFT")
    } else {
        format!("{display_name} — DRAFT")
    })
}

fn require_display_name(display_name: &str) -> Result<(), WindowTitleError> {
    let valid = !display_name.trim().is_empty()
        && display_name.len() <= MAX_DISPLAY_NAME_BYTES
        && !display_name.contains('/')
        && !display_name.contains('\\')
        && !display_name.chars().any(char::is_control);
    valid.then_some(()).ok_or(WindowTitleError::InvalidTitle)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn typed_command(
        window: WebviewWindow,
        request: WindowTitleRequest,
    ) -> Result<WindowTitleResponse, WindowTitleError> {
        set_window_title(window, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<WindowTitleRequest>(json!({
            "displayName": "Paper.draft",
            "unsaved": true
        }))
        .unwrap();
        assert_eq!(
            validated_window_title(request),
            Ok("Paper.draft — Unsaved — DRAFT".to_owned())
        );
        assert!(
            serde_json::from_value::<WindowTitleRequest>(json!({
                "displayName": "Paper.draft",
                "unsaved": true,
                "path": "/private/Paper.draft"
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(WindowTitleResponse { applied: true }).unwrap(),
            json!({ "applied": true })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(WindowTitleError::InvalidTitle).unwrap(),
            json!({ "code": "invalid_title" })
        );
    }

    #[test]
    fn title_contract_is_bounded_and_path_free() {
        for display_name in ["", " ", "../Paper.draft", "folder/Paper.draft", "bad\nname"] {
            assert_eq!(
                validated_window_title(WindowTitleRequest {
                    display_name: Some(display_name.to_owned()),
                    unsaved: false,
                }),
                Err(WindowTitleError::InvalidTitle)
            );
        }
        assert_eq!(
            validated_window_title(WindowTitleRequest {
                display_name: None,
                unsaved: false,
            }),
            Ok("DRAFT".to_owned())
        );
        assert_eq!(
            validated_window_title(WindowTitleRequest {
                display_name: None,
                unsaved: true,
            }),
            Err(WindowTitleError::InvalidTitle)
        );
    }

    #[test]
    fn native_document_title_formats_are_stable() {
        let cases = [
            ("Paper.draft", false, "Paper.draft — DRAFT"),
            ("Paper.draft", true, "Paper.draft — Unsaved — DRAFT"),
            ("source.docx", true, "source.docx — Unsaved — DRAFT"),
            (
                "Untitled document",
                true,
                "Untitled document — Unsaved — DRAFT",
            ),
        ];
        for (display_name, unsaved, expected) in cases {
            assert_eq!(
                validated_window_title(WindowTitleRequest {
                    display_name: Some(display_name.to_owned()),
                    unsaved,
                }),
                Ok(expected.to_owned())
            );
        }
    }
}
