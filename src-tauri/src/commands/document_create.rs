use serde::{Deserialize, Serialize};

use crate::documents::envelope::DocumentEnvelope;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateDocumentRequest {}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum CreateDocumentResponse {
    Created { envelope: DocumentEnvelope },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum CreateDocumentError {
    TemplateInvalid,
}

#[tauri::command]
pub(crate) fn create_document(
    request: CreateDocumentRequest,
) -> Result<CreateDocumentResponse, CreateDocumentError> {
    let CreateDocumentRequest {} = request;
    DocumentEnvelope::create_initial()
        .map(|envelope| CreateDocumentResponse::Created { envelope })
        .map_err(|_| CreateDocumentError::TemplateInvalid)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        CreateDocumentRequest,
    ) -> Result<CreateDocumentResponse, CreateDocumentError> = create_document;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        assert!(serde_json::from_value::<CreateDocumentRequest>(json!({})).is_ok());
        assert!(
            serde_json::from_value::<CreateDocumentRequest>(json!({ "documentId": "frontend" }))
                .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let response = create_document(CreateDocumentRequest {}).unwrap();
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["status"], "created");
        assert_eq!(value["envelope"]["schema_version"], 1);
        assert_eq!(value["envelope"]["title"], "Untitled document");
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(CreateDocumentError::TemplateInvalid).unwrap(),
            json!({ "code": "template_invalid" })
        );
    }
}
