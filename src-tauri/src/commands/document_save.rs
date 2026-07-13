use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::documents::{
    dialog::select_save_document,
    persistence::{
        SaveDocumentError, SaveDocumentOutcome, save_document as save_snapshot,
        save_requires_target,
    },
    registry::DocumentRegistry,
};

/// Immutable frontend snapshot submitted for a Rust-owned document save.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SaveDocumentRequest {
    snapshot: Value,
}

/// Saves one explicit validated snapshot through the atomic writer.
#[tauri::command]
pub(crate) async fn save_document(
    app_handle: AppHandle,
    registry: State<'_, DocumentRegistry>,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentOutcome, SaveDocumentError> {
    let selected = if save_requires_target(&registry, &request.snapshot)? {
        Some(
            select_save_document(&app_handle)
                .await
                .map_err(|_| SaveDocumentError::UnsupportedFileLocation)?,
        )
    } else {
        None
    };
    save_snapshot(&registry, request.snapshot, || {
        selected_save_target(selected)
    })
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
            "snapshot": envelope_value()
        }))
        .expect("request should deserialize");
        let unknown = serde_json::from_value::<SaveDocumentRequest>(json!({
            "snapshot": envelope_value(),
            "path": "/tmp/document.draft"
        }));

        assert_eq!(request.snapshot, envelope_value());
        assert!(unknown.is_err());
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
            "schema_version": 1,
            "document_id": DOCUMENT_ID,
            "title": "Saved document",
            "document": { "type": "doc", "content": [] }
        })
    }
}
