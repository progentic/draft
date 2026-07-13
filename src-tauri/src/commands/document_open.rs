use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::documents::{
    dialog::select_open_document,
    persistence::{
        OpenDocumentError, OpenDocumentOutcome, open_document as open_selected_document,
    },
    registry::DocumentRegistry,
};

/// Bounded request for selecting and opening one DRAFT document.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenDocumentRequest {}

/// Opens a Rust-selected document path after envelope validation.
#[tauri::command]
pub(crate) async fn open_document(
    app_handle: AppHandle,
    registry: State<'_, DocumentRegistry>,
    request: OpenDocumentRequest,
) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    let OpenDocumentRequest {} = request;
    let selected_path = select_open_document(&app_handle)
        .await
        .map_err(|_| OpenDocumentError::UnsupportedFileLocation)?;
    open_selected_document(&registry, selected_path)
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use serde_json::json;

    use super::*;
    use crate::documents::{
        envelope::{DocumentEnvelope, DocumentEnvelopeError},
        registry::DocumentRegistryError,
    };
    use crate::interoperability::{
        ExternalDocumentImportError, fidelity::ExternalFidelity,
        provenance::ExternalDocumentSummary,
    };

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
    fn typed_command<'a>(
        app_handle: AppHandle,
        registry: State<'a, DocumentRegistry>,
        request: OpenDocumentRequest,
    ) -> impl Future<Output = Result<OpenDocumentOutcome, OpenDocumentError>> + 'a {
        open_document(app_handle, registry, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<OpenDocumentRequest>(json!({}));
        let unknown = serde_json::from_value::<OpenDocumentRequest>(json!({ "path": "/tmp" }));

        assert_eq!(
            request.expect("request should deserialize"),
            OpenDocumentRequest {}
        );
        assert!(unknown.is_err());
    }

    #[test]
    fn response_serialization_is_stable() {
        let responses = [
            OpenDocumentOutcome::OpenedDraft {
                envelope: envelope(),
            },
            OpenDocumentOutcome::ImportedText {
                envelope: envelope(),
                format: crate::documents::persistence::TextImportFormat::Markdown,
            },
            OpenDocumentOutcome::ImportedExternal {
                envelope: envelope(),
                external: ExternalDocumentSummary::imported_docx(
                    &test_provenance(),
                    &envelope(),
                    b"source",
                ),
            },
            OpenDocumentOutcome::Cancelled,
        ];

        assert_eq!(
            serde_json::to_value(responses).expect("responses should serialize"),
            json!([
                { "status": "opened_draft", "envelope": envelope_value() },
                {
                    "status": "imported_text",
                    "envelope": envelope_value(),
                    "format": "markdown"
                },
                {
                    "status": "imported_external",
                    "envelope": envelope_value(),
                    "external": {
                        "format": "docx",
                        "displayName": "paper.docx",
                        "fidelity": { "classification": "exact" },
                        "sameFormatSave": "no_changes"
                    }
                },
                { "status": "cancelled" }
            ]),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            OpenDocumentError::UnsupportedFileLocation,
            OpenDocumentError::UnsupportedFileType,
            OpenDocumentError::FileNotFound,
            OpenDocumentError::ReadFailed,
            OpenDocumentError::MalformedJson,
            OpenDocumentError::InvalidTextEncoding,
            OpenDocumentError::TextTooLarge,
            OpenDocumentError::ExternalImport {
                cause: ExternalDocumentImportError::PackageTooLarge,
            },
            OpenDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::UnsupportedSchemaVersion { found: 3 },
            },
            OpenDocumentError::Registry {
                cause: DocumentRegistryError::AlreadyOpen,
            },
            OpenDocumentError::Registry {
                cause: DocumentRegistryError::SourcePathInUse,
            },
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "unsupported_file_location" },
                { "code": "unsupported_file_type" },
                { "code": "file_not_found" },
                { "code": "read_failed" },
                { "code": "malformed_json" },
                { "code": "invalid_text_encoding" },
                { "code": "text_too_large" },
                {
                    "code": "external_import",
                    "cause": { "code": "package_too_large" }
                },
                {
                    "code": "invalid_envelope",
                    "cause": { "code": "unsupported_schema_version", "found": 3 }
                },
                {
                    "code": "registry",
                    "cause": { "code": "already_open" }
                },
                {
                    "code": "registry",
                    "cause": { "code": "source_path_in_use" }
                }
            ]),
        );
    }

    fn envelope() -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(envelope_value()).expect("envelope should validate")
    }

    fn test_provenance() -> crate::interoperability::provenance::ExternalSourceProvenance {
        crate::interoperability::provenance::ExternalSourceProvenance::imported_docx(
            std::path::PathBuf::from("paper.docx"),
            "paper.docx".to_owned(),
            b"source",
            &envelope(),
            ExternalFidelity::Exact,
        )
    }

    fn envelope_value() -> serde_json::Value {
        json!({
            "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Opened document",
            "document": { "type": "doc", "content": [] }
        })
    }
}
