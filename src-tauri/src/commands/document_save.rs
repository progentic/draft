use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::{
    documents::{
        atomic_write::AtomicDocumentWriteError,
        dialog::{select_save_as_output, select_save_document},
        envelope::{DocumentEnvelope, DocumentEnvelopeError, DocumentId},
        persistence::{
            SaveDocumentError, SaveDocumentOutcome, save_document as save_snapshot,
            save_document_as as save_snapshot_as, save_requires_target,
        },
        registry::{DocumentRegistry, DocumentRegistryError},
        save_as::{SaveAsFormat, SaveAsTargetError, normalize_save_as_target},
    },
    exports::{
        docx::{DocxExportError, compile_docx, write_docx_artifact},
        plain_text::{PlainTextExportError, compile_plain_text, write_plain_text_artifact},
    },
};

const DEFAULT_NEW_DOCUMENT_STEM: &str = "Untitled";

/// Immutable frontend snapshot submitted for one Rust-owned save operation.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveDocumentRequest {
    display_name: String,
    snapshot: Value,
    mode: SaveDocumentMode,
    origin: SaveDocumentOrigin,
    #[serde(default)]
    format: Option<String>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SaveDocumentResponse {
    DraftSaved {
        document_id: DocumentId,
        display_name: String,
        was_save_as: bool,
        authoritative_identity_changed: bool,
        dirty_state_cleared: bool,
    },
    ConvertedOutput {
        display_name: String,
        output_format: SaveAsFormat,
        bytes_written: usize,
        authoritative_identity_changed: bool,
        dirty_state_changed: bool,
    },
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum SaveDocumentCommandError {
    UnsupportedFileLocation,
    InvalidOperation,
    UnsupportedFormat,
    InvalidTarget,
    SaveAsTarget { cause: SaveAsTargetError },
    InvalidEnvelope { cause: DocumentEnvelopeError },
    SerializationFailed,
    Registry { cause: DocumentRegistryError },
    WriteFailed { cause: AtomicDocumentWriteError },
    DurabilityUncertain,
    DocxConversion { cause: DocxExportError },
    PlainTextConversion { cause: PlainTextExportError },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SaveOperation {
    Save,
    SaveAs(SaveAsFormat),
}

/// Saves one explicit validated snapshot through the selected Rust-owned policy.
#[tauri::command]
pub(crate) async fn save_document(
    app_handle: AppHandle,
    registry: State<'_, DocumentRegistry>,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let operation = parse_operation(&request)?;
    validate_snapshot(&request.snapshot)?;
    match operation {
        SaveOperation::Save => save_current_document(&app_handle, &registry, request).await,
        SaveOperation::SaveAs(format) => {
            save_as_output(&app_handle, &registry, request, format).await
        }
    }
}

async fn save_current_document(
    app_handle: &AppHandle,
    registry: &DocumentRegistry,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let suggested =
        suggested_file_name(&request.display_name, request.origin, SaveAsFormat::Draft)?;
    let selected = if save_requires_target(registry, &request.snapshot).map_err(map_save_error)? {
        select_save_document(app_handle, &suggested)
            .await
            .map_err(|_| SaveDocumentCommandError::UnsupportedFileLocation)?
    } else {
        return save_snapshot(registry, request.snapshot, unreachable_selection)
            .map(response_from_draft_outcome)
            .map_err(map_save_error);
    };
    save_snapshot(registry, request.snapshot, || Ok(selected))
        .map(response_from_draft_outcome)
        .map_err(map_save_error)
}

async fn save_as_output(
    app_handle: &AppHandle,
    registry: &DocumentRegistry,
    request: SaveDocumentRequest,
    format: SaveAsFormat,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let suggested = suggested_file_name(&request.display_name, request.origin, format)?;
    let selected = select_save_as_output(app_handle, format, &suggested)
        .await
        .map_err(|_| SaveDocumentCommandError::UnsupportedFileLocation)?;
    save_as_selected(registry, request.snapshot, format, selected)
}

fn save_as_selected(
    registry: &DocumentRegistry,
    snapshot: Value,
    format: SaveAsFormat,
    selected: Option<PathBuf>,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let Some(selected) = selected else {
        return Ok(SaveDocumentResponse::Cancelled);
    };
    let target = normalize_save_as_target(selected, format)
        .map_err(|cause| SaveDocumentCommandError::SaveAsTarget { cause })?;
    match format {
        SaveAsFormat::Draft => save_as_draft(registry, snapshot, target),
        SaveAsFormat::Docx => save_as_docx(snapshot, target),
        SaveAsFormat::Txt => save_as_text(snapshot, target),
    }
}

fn save_as_draft(
    registry: &DocumentRegistry,
    snapshot: Value,
    target: PathBuf,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    save_snapshot_as(registry, snapshot, || Ok(Some(target)))
        .map(response_from_draft_outcome)
        .map_err(map_save_error)
}

fn save_as_docx(
    snapshot: Value,
    target: PathBuf,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let document = validate_snapshot(&snapshot)?;
    let artifact = compile_docx(&document)
        .map_err(|cause| SaveDocumentCommandError::DocxConversion { cause })?;
    let outcome = write_docx_artifact(&artifact, &target)
        .map_err(|cause| SaveDocumentCommandError::DocxConversion { cause })?;
    converted_response(&target, SaveAsFormat::Docx, outcome.bytes_written())
}

fn save_as_text(
    snapshot: Value,
    target: PathBuf,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    let document = validate_snapshot(&snapshot)?;
    let artifact = compile_plain_text(&document)
        .map_err(|cause| SaveDocumentCommandError::PlainTextConversion { cause })?;
    let bytes_written = write_plain_text_artifact(&artifact, &target)
        .map_err(|cause| SaveDocumentCommandError::PlainTextConversion { cause })?;
    converted_response(&target, SaveAsFormat::Txt, bytes_written)
}

fn converted_response(
    target: &Path,
    format: SaveAsFormat,
    bytes_written: usize,
) -> Result<SaveDocumentResponse, SaveDocumentCommandError> {
    Ok(SaveDocumentResponse::ConvertedOutput {
        display_name: display_name(target)?,
        output_format: format,
        bytes_written,
        authoritative_identity_changed: false,
        dirty_state_changed: false,
    })
}

fn parse_operation(
    request: &SaveDocumentRequest,
) -> Result<SaveOperation, SaveDocumentCommandError> {
    match (request.mode, request.format.as_deref()) {
        (SaveDocumentMode::Save, None) => Ok(SaveOperation::Save),
        (SaveDocumentMode::SaveAs, Some(code)) => SaveAsFormat::from_code(code)
            .map(SaveOperation::SaveAs)
            .ok_or(SaveDocumentCommandError::UnsupportedFormat),
        (SaveDocumentMode::SaveAs, None) | (SaveDocumentMode::Save, Some(_)) => {
            Err(SaveDocumentCommandError::InvalidOperation)
        }
    }
}

fn suggested_file_name(
    display_name: &str,
    origin: SaveDocumentOrigin,
    format: SaveAsFormat,
) -> Result<String, SaveDocumentCommandError> {
    require_basename(display_name)?;
    let stem = if origin == SaveDocumentOrigin::New {
        DEFAULT_NEW_DOCUMENT_STEM
    } else {
        file_stem(display_name)?
    };
    Ok(format!("{stem}.{}", format.extension()))
}

fn file_stem(display_name: &str) -> Result<&str, SaveDocumentCommandError> {
    Path::new(display_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::trim)
        .filter(|stem| !stem.is_empty() && stem.len() <= 240)
        .ok_or(SaveDocumentCommandError::InvalidTarget)
}

fn require_basename(display_name: &str) -> Result<(), SaveDocumentCommandError> {
    let valid = !display_name.trim().is_empty()
        && !display_name.contains('/')
        && !display_name.contains('\\')
        && !display_name.chars().any(char::is_control);
    valid
        .then_some(())
        .ok_or(SaveDocumentCommandError::InvalidTarget)
}

fn validate_snapshot(snapshot: &Value) -> Result<DocumentEnvelope, SaveDocumentCommandError> {
    DocumentEnvelope::from_json_value(snapshot.clone())
        .map_err(|cause| SaveDocumentCommandError::InvalidEnvelope { cause })
}

fn display_name(target: &Path) -> Result<String, SaveDocumentCommandError> {
    target
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or(SaveDocumentCommandError::InvalidTarget)
}

fn unreachable_selection() -> Result<Option<PathBuf>, SaveDocumentError> {
    Err(SaveDocumentError::Registry {
        cause: DocumentRegistryError::RegistryUnavailable,
    })
}

fn response_from_draft_outcome(outcome: SaveDocumentOutcome) -> SaveDocumentResponse {
    match outcome {
        SaveDocumentOutcome::Saved {
            document_id,
            display_name,
            was_save_as,
        } => SaveDocumentResponse::DraftSaved {
            document_id,
            display_name,
            was_save_as,
            authoritative_identity_changed: was_save_as,
            dirty_state_cleared: true,
        },
        SaveDocumentOutcome::Cancelled => SaveDocumentResponse::Cancelled,
    }
}

fn map_save_error(error: SaveDocumentError) -> SaveDocumentCommandError {
    match error {
        SaveDocumentError::InvalidTarget => SaveDocumentCommandError::InvalidTarget,
        SaveDocumentError::InvalidEnvelope { cause } => {
            SaveDocumentCommandError::InvalidEnvelope { cause }
        }
        SaveDocumentError::SerializationFailed => SaveDocumentCommandError::SerializationFailed,
        SaveDocumentError::Registry { cause } => SaveDocumentCommandError::Registry { cause },
        SaveDocumentError::WriteFailed { cause } => SaveDocumentCommandError::WriteFailed { cause },
        SaveDocumentError::DurabilityUncertain => SaveDocumentCommandError::DurabilityUncertain,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, future::Future};

    use serde_json::json;

    use super::*;
    use crate::{documents::test_support::TestDocumentPath, interoperability::import_docx_source};

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";

    fn typed_command<'a>(
        app_handle: AppHandle,
        registry: State<'a, DocumentRegistry>,
        request: SaveDocumentRequest,
    ) -> impl Future<Output = Result<SaveDocumentResponse, SaveDocumentCommandError>> + 'a {
        save_document(app_handle, registry, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn operation_requires_one_closed_save_as_format() {
        let save = request("save", None);
        let save_as = request("save_as", Some("docx"));
        let missing = request("save_as", None);
        let unexpected = request("save", Some("txt"));
        let unsupported = request("save_as", Some("pdf"));

        assert!(matches!(parse_operation(&save), Ok(SaveOperation::Save)));
        assert!(matches!(
            parse_operation(&save_as),
            Ok(SaveOperation::SaveAs(SaveAsFormat::Docx))
        ));
        assert_eq!(
            parse_operation(&missing),
            Err(SaveDocumentCommandError::InvalidOperation)
        );
        assert_eq!(
            parse_operation(&unexpected),
            Err(SaveDocumentCommandError::InvalidOperation)
        );
        assert_eq!(
            parse_operation(&unsupported),
            Err(SaveDocumentCommandError::UnsupportedFormat)
        );
    }

    #[test]
    fn save_dialog_suggestions_preserve_basename_identity() {
        let cases = [
            (
                "Research notes.draft",
                SaveDocumentOrigin::OpenedDraft,
                SaveAsFormat::Draft,
                "Research notes.draft",
            ),
            (
                "Imported paper.docx",
                SaveDocumentOrigin::ImportedExternal,
                SaveAsFormat::Docx,
                "Imported paper.docx",
            ),
            (
                "Imported paper.docx",
                SaveDocumentOrigin::ImportedExternal,
                SaveAsFormat::Txt,
                "Imported paper.txt",
            ),
            (
                "notes.md",
                SaveDocumentOrigin::ImportedText,
                SaveAsFormat::Draft,
                "notes.draft",
            ),
            (
                "Untitled document",
                SaveDocumentOrigin::New,
                SaveAsFormat::Txt,
                "Untitled.txt",
            ),
        ];
        for (display_name, origin, format, expected) in cases {
            assert_eq!(
                suggested_file_name(display_name, origin, format),
                Ok(expected.to_owned())
            );
        }
    }

    #[test]
    fn converted_outputs_do_not_rebind_registered_draft() {
        let registry = DocumentRegistry::new();
        let source = TestDocumentPath::new("save-as-source");
        let docx = TestDocumentPath::with_extension("save-as-copy", "docx");
        let txt = TestDocumentPath::with_extension("save-as-copy", "txt");
        let snapshot = envelope_value();
        save_snapshot(&registry, snapshot.clone(), || {
            Ok(Some(source.path().to_owned()))
        })
        .unwrap();

        let docx_result = save_as_selected(
            &registry,
            snapshot.clone(),
            SaveAsFormat::Docx,
            Some(docx.path().to_owned()),
        )
        .unwrap();
        let txt_result = save_as_selected(
            &registry,
            snapshot,
            SaveAsFormat::Txt,
            Some(txt.path().to_owned()),
        )
        .unwrap();

        assert!(matches!(
            docx_result,
            SaveDocumentResponse::ConvertedOutput {
                output_format: SaveAsFormat::Docx,
                authoritative_identity_changed: false,
                dirty_state_changed: false,
                ..
            }
        ));
        assert!(matches!(
            txt_result,
            SaveDocumentResponse::ConvertedOutput {
                output_format: SaveAsFormat::Txt,
                authoritative_identity_changed: false,
                dirty_state_changed: false,
                ..
            }
        ));
        assert_eq!(
            registry.source_path(envelope().document_id()).unwrap(),
            Some(source.path().to_owned())
        );
        assert!(import_docx_source(docx.path()).is_ok());
        assert_eq!(fs::read_to_string(txt.path()).unwrap(), "");
    }

    #[test]
    fn cancelled_save_as_preserves_registry_and_outputs() {
        let registry = DocumentRegistry::new();
        let source = TestDocumentPath::new("cancelled-save-as-source");
        save_snapshot(&registry, envelope_value(), || {
            Ok(Some(source.path().to_owned()))
        })
        .unwrap();
        let before = fs::read(source.path()).unwrap();

        for format in [SaveAsFormat::Draft, SaveAsFormat::Docx, SaveAsFormat::Txt] {
            let result = save_as_selected(&registry, envelope_value(), format, None).unwrap();
            assert_eq!(result, SaveDocumentResponse::Cancelled);
        }
        assert_eq!(fs::read(source.path()).unwrap(), before);
        assert_eq!(
            registry.source_path(envelope().document_id()).unwrap(),
            Some(source.path().to_owned())
        );
    }

    #[test]
    fn draft_save_as_rebinds_only_after_atomic_persistence() {
        let registry = DocumentRegistry::new();
        let source = TestDocumentPath::new("draft-save-as-source");
        let replacement = TestDocumentPath::new("draft-save-as-replacement");
        let snapshot = envelope_value();
        save_snapshot(&registry, snapshot.clone(), || {
            Ok(Some(source.path().to_owned()))
        })
        .unwrap();
        let source_bytes = fs::read(source.path()).unwrap();

        let result = save_as_selected(
            &registry,
            snapshot,
            SaveAsFormat::Draft,
            Some(replacement.path().to_owned()),
        )
        .unwrap();

        assert!(matches!(
            result,
            SaveDocumentResponse::DraftSaved {
                authoritative_identity_changed: true,
                dirty_state_cleared: true,
                ..
            }
        ));
        assert_eq!(fs::read(source.path()).unwrap(), source_bytes);
        assert_eq!(
            registry.source_path(envelope().document_id()).unwrap(),
            Some(replacement.path().to_owned())
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let responses = [
            response_from_draft_outcome(SaveDocumentOutcome::Saved {
                document_id: envelope().document_id(),
                display_name: "Research notes.draft".to_owned(),
                was_save_as: true,
            }),
            SaveDocumentResponse::ConvertedOutput {
                display_name: "Research notes.docx".to_owned(),
                output_format: SaveAsFormat::Docx,
                bytes_written: 42,
                authoritative_identity_changed: false,
                dirty_state_changed: false,
            },
            SaveDocumentResponse::Cancelled,
        ];

        assert_eq!(
            serde_json::to_value(responses).unwrap(),
            json!([
                { "status": "draft_saved", "documentId": DOCUMENT_ID, "displayName": "Research notes.draft", "wasSaveAs": true, "authoritativeIdentityChanged": true, "dirtyStateCleared": true },
                { "status": "converted_output", "displayName": "Research notes.docx", "outputFormat": "docx", "bytesWritten": 42, "authoritativeIdentityChanged": false, "dirtyStateChanged": false },
                { "status": "cancelled" }
            ])
        );
    }

    #[test]
    fn request_deserialization_is_stable() {
        let value = json!({
            "displayName": "Research notes.draft",
            "snapshot": envelope_value(),
            "mode": "save_as",
            "origin": "opened_draft",
            "format": "draft",
            "path": "/tmp/document.draft"
        });
        assert!(serde_json::from_value::<SaveDocumentRequest>(value).is_err());
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(SaveDocumentCommandError::UnsupportedFormat).unwrap(),
            json!({ "code": "unsupported_format" })
        );
        assert_eq!(
            serde_json::to_value(SaveDocumentCommandError::SaveAsTarget {
                cause: SaveAsTargetError::ConflictingExtension,
            })
            .unwrap(),
            json!({
                "code": "save_as_target",
                "cause": { "code": "conflicting_extension" }
            })
        );
    }

    fn request(mode: &str, format: Option<&str>) -> SaveDocumentRequest {
        let mut value = json!({
            "displayName": "Research notes.draft",
            "snapshot": envelope_value(),
            "mode": mode,
            "origin": "opened_draft"
        });
        if let Some(format) = format {
            value["format"] = json!(format);
        }
        serde_json::from_value(value).unwrap()
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
