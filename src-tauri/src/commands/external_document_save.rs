use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use crate::documents::{
    external_save::{
        ExternalSaveDecision, SaveExternalDocumentError, SaveExternalDocumentOutcome,
        save_external_document as save_external_snapshot,
    },
    registry::DocumentRegistry,
};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SaveExternalDocumentRequest {
    snapshot: Value,
    decision: ExternalSaveDecision,
}

/// Replaces one imported external source through its Rust-owned provenance.
#[tauri::command]
pub(crate) async fn save_external_document(
    registry: State<'_, DocumentRegistry>,
    request: SaveExternalDocumentRequest,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError> {
    save_external_snapshot(&registry, request.snapshot, request.decision)
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use serde_json::json;

    use super::*;
    use crate::documents::{
        atomic_write::AtomicDocumentWriteError,
        envelope::{DocumentEnvelope, DocumentEnvelopeError, DocumentId},
        external_save::{
            ExternalSaveCommitFailure, ExternalSourceReadError, SaveExternalDocumentError,
            SaveExternalDocumentOutcome,
        },
        registry::DocumentRegistryError,
    };
    use crate::exports::docx::DocxExportError;
    use crate::interoperability::provenance::SameFormatSaveDisposition;

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";

    fn typed_command<'a>(
        registry: State<'a, DocumentRegistry>,
        request: SaveExternalDocumentRequest,
    ) -> impl Future<Output = Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>> + 'a
    {
        save_external_document(registry, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn request_deserialization_is_stable() {
        for decision in ["inspect", "save_exact", "accept_normalization", "cancel"] {
            assert!(
                serde_json::from_value::<SaveExternalDocumentRequest>(json!({
                    "snapshot": envelope_value(),
                    "decision": decision
                }))
                .is_ok()
            );
        }
        assert!(
            serde_json::from_value::<SaveExternalDocumentRequest>(json!({
                "snapshot": envelope_value(),
                "decision": "overwrite",
                "path": "/private/source.docx"
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let document_id = document_id();
        let responses = [
            SaveExternalDocumentOutcome::Eligibility {
                document_id,
                display_name: "paper.docx".to_owned(),
                disposition: SameFormatSaveDisposition::AllowedExact,
                normalizations: vec![],
            },
            SaveExternalDocumentOutcome::Saved {
                document_id,
                display_name: "paper.docx".to_owned(),
                bytes_written: 1024,
                disposition: SameFormatSaveDisposition::AllowedExact,
            },
            SaveExternalDocumentOutcome::Unchanged {
                document_id,
                display_name: "paper.docx".to_owned(),
            },
            SaveExternalDocumentOutcome::ConfirmationRequired {
                document_id,
                disposition: SameFormatSaveDisposition::AllowedAfterAcceptedNormalization,
            },
            SaveExternalDocumentOutcome::Denied {
                document_id,
                disposition: SameFormatSaveDisposition::DeniedSourceChanged,
            },
            SaveExternalDocumentOutcome::Cancelled { document_id },
        ];
        let value = serde_json::to_value(responses).unwrap();

        assert_eq!(value[0]["displayName"], "paper.docx");
        assert_eq!(value[0]["normalizations"], json!([]));
        assert!(value.to_string().find("/private/").is_none());
        assert!(value.to_string().find("fingerprint").is_none());
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            SaveExternalDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::InvalidDocumentRoot,
            },
            SaveExternalDocumentError::Registry {
                cause: DocumentRegistryError::ExternalSourceUnavailable,
            },
            SaveExternalDocumentError::SourceRead {
                cause: ExternalSourceReadError::ReadFailed,
            },
            SaveExternalDocumentError::Compilation {
                cause: DocxExportError::PackageConstructionFailed,
            },
            SaveExternalDocumentError::WriteFailed {
                cause: AtomicDocumentWriteError::ReplaceTarget,
            },
            SaveExternalDocumentError::ReplacementRolledBack {
                cause: ExternalSaveCommitFailure::DurabilityUncertain,
            },
            SaveExternalDocumentError::RollbackFailed {
                cause: ExternalSaveCommitFailure::Registry {
                    cause: DocumentRegistryError::RegistryUnavailable,
                },
                rollback: AtomicDocumentWriteError::ReplaceTarget,
            },
        ];
        let value = serde_json::to_value(errors).unwrap();
        let serialized = value.to_string();

        assert!(!serialized.contains("paper"));
        assert!(!serialized.contains("/private/"));
        assert!(!serialized.contains("fingerprint"));
    }

    fn document_id() -> DocumentId {
        DocumentEnvelope::from_json_value(envelope_value())
            .unwrap()
            .document_id()
    }

    fn envelope_value() -> Value {
        json!({
            "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Imported document",
            "document": { "type": "doc", "content": [] }
        })
    }
}
