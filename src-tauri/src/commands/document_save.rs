use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use tauri::{AppHandle, State};

use crate::documents::{
    dialog::select_save_document,
    persistence::{
        SaveDocumentError, SaveDocumentOutcome, save_document as save_snapshot,
        save_document_as as save_snapshot_as, save_requires_target,
    },
    registry::DocumentRegistry,
};

const DEFAULT_NEW_DOCUMENT_FILE_NAME: &str = "Untitled.draft";

/// Immutable frontend snapshot submitted for a Rust-owned document save.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveDocumentRequest {
    display_name: String,
    snapshot: Value,
    mode: SaveDocumentMode,
    origin: SaveDocumentOrigin,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SaveDocumentMode {
    Save,
    SaveAs,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SaveDocumentOrigin {
    ImportedExternal,
    ImportedText,
    New,
    OpenedDraft,
}

/// Saves one explicit validated snapshot through the atomic writer.
#[tauri::command]
pub(crate) async fn save_document(
    app_handle: AppHandle,
    registry: State<'_, DocumentRegistry>,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentOutcome, SaveDocumentError> {
    let suggested_file_name = suggested_draft_file_name(&request.display_name, request.origin)?;
    let selected = if request.mode == SaveDocumentMode::SaveAs
        || save_requires_target(&registry, &request.snapshot)?
    {
        Some(
            select_save_document(&app_handle, &suggested_file_name)
                .await
                .map_err(|_| SaveDocumentError::UnsupportedFileLocation)?,
        )
    } else {
        None
    };
    match request.mode {
        SaveDocumentMode::Save => save_snapshot(&registry, request.snapshot, || {
            selected_save_target(selected)
        }),
        SaveDocumentMode::SaveAs => save_snapshot_as(&registry, request.snapshot, || {
            selected_save_target(selected)
        }),
    }
}

fn suggested_draft_file_name(
    display_name: &str,
    origin: SaveDocumentOrigin,
) -> Result<String, SaveDocumentError> {
    require_basename(display_name)?;
    if origin == SaveDocumentOrigin::New {
        return Ok(DEFAULT_NEW_DOCUMENT_FILE_NAME.to_owned());
    }
    let stem = Path::new(display_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::trim)
        .filter(|stem| !stem.is_empty())
        .ok_or(SaveDocumentError::InvalidTarget)?;
    if stem.len() > 240 {
        return Err(SaveDocumentError::InvalidTarget);
    }
    Ok(format!("{stem}.draft"))
}

fn require_basename(display_name: &str) -> Result<(), SaveDocumentError> {
    let valid = !display_name.trim().is_empty()
        && !display_name.contains('/')
        && !display_name.contains('\\')
        && !display_name.chars().any(char::is_control);
    valid.then_some(()).ok_or(SaveDocumentError::InvalidTarget)
}

fn selected_save_target(
    selected: Option<Option<std::path::PathBuf>>,
) -> Result<Option<std::path::PathBuf>, SaveDocumentError> {
    selected.ok_or(SaveDocumentError::Registry {
        cause: crate::documents::registry::DocumentRegistryError::RegistryUnavailable,
    })
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use serde_json::json;

    use super::*;
    use crate::documents::{
        atomic_write::AtomicDocumentWriteError,
        envelope::{DocumentEnvelope, DocumentEnvelopeError},
        registry::DocumentRegistryError,
    };

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
    fn typed_command<'a>(
        app_handle: AppHandle,
        registry: State<'a, DocumentRegistry>,
        request: SaveDocumentRequest,
    ) -> impl Future<Output = Result<SaveDocumentOutcome, SaveDocumentError>> + 'a {
        save_document(app_handle, registry, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<SaveDocumentRequest>(json!({
            "displayName": "Research notes.draft",
            "snapshot": envelope_value(),
            "mode": "save",
            "origin": "opened_draft"
        }))
        .expect("request should deserialize");
        let unknown = serde_json::from_value::<SaveDocumentRequest>(json!({
            "displayName": "Research notes.draft",
            "snapshot": envelope_value(),
            "mode": "save",
            "origin": "opened_draft",
            "path": "/tmp/document.draft"
        }));

        assert_eq!(request.display_name, "Research notes.draft");
        assert_eq!(request.snapshot, envelope_value());
        assert_eq!(request.mode, SaveDocumentMode::Save);
        assert!(unknown.is_err());
        assert!(
            serde_json::from_value::<SaveDocumentRequest>(json!({
                "displayName": "Research notes.draft",
                "snapshot": envelope_value(),
                "mode": "replace",
                "origin": "opened_draft"
            }))
            .is_err()
        );
    }

    #[test]
    fn save_dialog_suggestions_preserve_basename_identity() {
        assert_eq!(
            suggested_draft_file_name("Research notes.draft", SaveDocumentOrigin::OpenedDraft),
            Ok("Research notes.draft".to_owned())
        );
        assert_eq!(
            suggested_draft_file_name("Imported paper.docx", SaveDocumentOrigin::ImportedExternal),
            Ok("Imported paper.draft".to_owned())
        );
        assert_eq!(
            suggested_draft_file_name("notes.md", SaveDocumentOrigin::ImportedText),
            Ok("notes.draft".to_owned())
        );
        assert_eq!(
            suggested_draft_file_name("Untitled document", SaveDocumentOrigin::New),
            Ok("Untitled.draft".to_owned())
        );
        for invalid in ["", "../private.draft", "folder/file.draft", "bad\nname"] {
            assert_eq!(
                suggested_draft_file_name(invalid, SaveDocumentOrigin::OpenedDraft),
                Err(SaveDocumentError::InvalidTarget)
            );
        }
    }

    #[test]
    fn response_serialization_is_stable() {
        let document_id = envelope().document_id();
        let responses = [
            SaveDocumentOutcome::Saved {
                document_id,
                display_name: "Research notes.draft".to_owned(),
                was_save_as: true,
            },
            SaveDocumentOutcome::Cancelled,
        ];

        assert_eq!(
            serde_json::to_value(responses).expect("responses should serialize"),
            json!([
                {
                    "status": "saved",
                    "documentId": DOCUMENT_ID,
                    "displayName": "Research notes.draft",
                    "wasSaveAs": true
                },
                { "status": "cancelled" }
            ]),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            SaveDocumentError::UnsupportedFileLocation,
            SaveDocumentError::InvalidTarget,
            SaveDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::InvalidDocumentRoot,
            },
            SaveDocumentError::SerializationFailed,
            SaveDocumentError::Registry {
                cause: DocumentRegistryError::RegistryUnavailable,
            },
            SaveDocumentError::WriteFailed {
                cause: AtomicDocumentWriteError::ReplaceTarget,
            },
            SaveDocumentError::DurabilityUncertain,
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "unsupported_file_location" },
                { "code": "invalid_target" },
                {
                    "code": "invalid_envelope",
                    "cause": { "code": "invalid_document_root" }
                },
                { "code": "serialization_failed" },
                {
                    "code": "registry",
                    "cause": { "code": "registry_unavailable" }
                },
                {
                    "code": "write_failed",
                    "cause": { "code": "replace_target" }
                },
                { "code": "durability_uncertain" }
            ]),
        );
    }

    fn envelope() -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(envelope_value()).expect("envelope should validate")
    }

    fn envelope_value() -> Value {
        json!({
            "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Saved document",
            "document": { "type": "doc", "content": [] }
        })
    }
}
