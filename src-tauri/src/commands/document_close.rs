use serde::{Deserialize, Serialize};
use tauri::State;

use crate::documents::{
    envelope::DocumentId,
    registry::{DocumentRegistry, DocumentRegistryError},
};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct CloseDocumentRequest {
    document_id: DocumentId,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum CloseDocumentResponse {
    Closed { document_id: DocumentId },
}

#[tauri::command]
pub(crate) fn close_document(
    registry: State<'_, DocumentRegistry>,
    request: CloseDocumentRequest,
) -> Result<CloseDocumentResponse, DocumentRegistryError> {
    close_with_registry(&registry, request)
}

fn close_with_registry(
    registry: &DocumentRegistry,
    request: CloseDocumentRequest,
) -> Result<CloseDocumentResponse, DocumentRegistryError> {
    registry.close(request.document_id)?;
    Ok(CloseDocumentResponse::Closed {
        document_id: request.document_id,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::documents::envelope::DocumentEnvelope;

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
    const TYPED_COMMAND: for<'a> fn(
        State<'a, DocumentRegistry>,
        CloseDocumentRequest,
    ) -> Result<CloseDocumentResponse, DocumentRegistryError> = close_document;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn response_serialization_is_stable() {
        let request = request();
        let registry = DocumentRegistry::new();
        registry.open(envelope()).unwrap();

        let response = close_with_registry(&registry, request).unwrap();

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "status": "closed", "documentId": DOCUMENT_ID
            })
        );
        assert_eq!(
            registry.close(document_id()),
            Err(DocumentRegistryError::NotOpen)
        );
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(
            serde_json::from_value::<CloseDocumentRequest>(json!({
                "documentId": DOCUMENT_ID, "path": "/tmp/file"
            }))
            .is_err()
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(DocumentRegistryError::NotOpen).unwrap(),
            json!({ "code": "not_open" })
        );
    }

    fn request() -> CloseDocumentRequest {
        serde_json::from_value(json!({ "documentId": DOCUMENT_ID })).unwrap()
    }

    fn envelope() -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(json!({
            "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Document",
            "document": { "type": "doc", "content": [] }
        }))
        .unwrap()
    }

    fn document_id() -> DocumentId {
        envelope().document_id()
    }
}
