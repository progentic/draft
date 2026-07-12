use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use crate::{
    documents::{
        dialog::select_export_docx,
        envelope::{DocumentEnvelope, DocumentEnvelopeError},
    },
    exports::docx::{DocxExportError, export_docx},
};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportDocumentRequest {
    snapshot: Value,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum ExportDocumentResponse {
    Exported { bytes_written: usize },
    Cancelled,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ExportDocumentError {
    UnsupportedFileLocation,
    InvalidEnvelope { cause: DocumentEnvelopeError },
    Export { cause: DocxExportError },
}

#[tauri::command]
pub(crate) async fn export_document(
    app_handle: AppHandle,
    request: ExportDocumentRequest,
) -> Result<ExportDocumentResponse, ExportDocumentError> {
    let document = validated_document(request.snapshot)?;
    let selected = select_export_docx(&app_handle)
        .await
        .map_err(|_| ExportDocumentError::UnsupportedFileLocation)?;
    export_selected(&document, selected)
}

fn validated_document(snapshot: Value) -> Result<DocumentEnvelope, ExportDocumentError> {
    DocumentEnvelope::from_json_value(snapshot)
        .map_err(|cause| ExportDocumentError::InvalidEnvelope { cause })
}

fn export_selected(
    document: &DocumentEnvelope,
    selected: Option<std::path::PathBuf>,
) -> Result<ExportDocumentResponse, ExportDocumentError> {
    let Some(target) = selected else {
        return Ok(ExportDocumentResponse::Cancelled);
    };
    let outcome =
        export_docx(document, &target).map_err(|cause| ExportDocumentError::Export { cause })?;
    Ok(ExportDocumentResponse::Exported {
        bytes_written: outcome.bytes_written(),
    })
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use serde_json::json;

    use super::*;
    use crate::documents::text_format::TextFormatError;

    fn typed_command(
        app_handle: AppHandle,
        request: ExportDocumentRequest,
    ) -> impl Future<Output = Result<ExportDocumentResponse, ExportDocumentError>> {
        export_document(app_handle, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn response_serialization_is_stable() {
        let request: ExportDocumentRequest = serde_json::from_value(json!({
            "snapshot": envelope_value()
        }))
        .unwrap();
        let document = validated_document(request.snapshot).unwrap();

        assert_eq!(
            export_selected(&document, None),
            Ok(ExportDocumentResponse::Cancelled)
        );
        assert_eq!(
            serde_json::to_value(ExportDocumentResponse::Cancelled).unwrap(),
            json!({
                "status": "cancelled"
            })
        );
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(
            serde_json::from_value::<ExportDocumentRequest>(json!({
                "snapshot": envelope_value(), "path": "/tmp/export.docx"
            }))
            .is_err()
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(ExportDocumentError::UnsupportedFileLocation).unwrap(),
            json!({ "code": "unsupported_file_location" })
        );
    }

    #[test]
    fn invalid_font_format_fails_before_dialog_or_export() {
        let mut snapshot = envelope_value();
        snapshot["document"] = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "text",
                    "text": "Invalid",
                    "marks": [{
                        "type": "fontSize",
                        "attrs": { "points": 7 }
                    }]
                }]
            }]
        });

        assert_eq!(
            validated_document(snapshot),
            Err(ExportDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::InvalidTextFormat {
                    path: "document.content[0].content[0].marks[0]".to_owned(),
                    cause: TextFormatError::InvalidFontSize,
                }
            })
        );
    }

    fn envelope_value() -> Value {
        json!({
            "schema_version": 1,
            "document_id": "00000000-0000-4000-8000-000000000001",
            "title": "Exported document",
            "document": { "type": "doc", "content": [] }
        })
    }
}
