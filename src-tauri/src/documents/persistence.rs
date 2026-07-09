use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;

use crate::documents::{
    atomic_write::write_document_atomically,
    envelope::{DocumentEnvelope, DocumentEnvelopeError, DocumentId},
    registry::{DocumentRegistry, DocumentRegistryError},
};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum OpenDocumentError {
    UnsupportedFileLocation,
    FileNotFound,
    ReadFailed,
    MalformedJson,
    InvalidEnvelope { cause: DocumentEnvelopeError },
    Registry { cause: DocumentRegistryError },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum SaveDocumentError {
    UnsupportedFileLocation,
    InvalidEnvelope { cause: DocumentEnvelopeError },
    SerializationFailed,
    Registry { cause: DocumentRegistryError },
    WriteFailed,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum OpenDocumentOutcome {
    Opened { envelope: DocumentEnvelope },
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SaveDocumentOutcome {
    Saved { document_id: DocumentId },
    Cancelled,
}

enum SavePlan {
    Existing(PathBuf),
    Attach(PathBuf),
    Register(PathBuf),
    Cancelled,
}

pub(crate) fn open_document(
    registry: &DocumentRegistry,
    selected_path: Option<PathBuf>,
) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    let Some(source_path) = selected_path else {
        return Ok(OpenDocumentOutcome::Cancelled);
    };
    let envelope = load_envelope(&source_path)?;
    registry
        .open_from_path(envelope.clone(), source_path)
        .map_err(|cause| OpenDocumentError::Registry { cause })?;
    Ok(OpenDocumentOutcome::Opened { envelope })
}

pub(crate) fn save_document<SelectPath>(
    registry: &DocumentRegistry,
    snapshot: Value,
    select_path: SelectPath,
) -> Result<SaveDocumentOutcome, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
{
    let envelope = validate_snapshot(snapshot)?;
    let contents = serialize_envelope(&envelope)?;
    let plan = plan_save(registry, envelope.document_id(), select_path)?;
    apply_save_plan(registry, envelope, contents, plan)
}

fn load_envelope(source_path: &Path) -> Result<DocumentEnvelope, OpenDocumentError> {
    let contents = read_document(source_path)?;
    deserialize_envelope(&contents)
}

fn read_document(source_path: &Path) -> Result<Vec<u8>, OpenDocumentError> {
    fs::read(source_path).map_err(map_read_error)
}

fn map_read_error(error: io::Error) -> OpenDocumentError {
    match error.kind() {
        io::ErrorKind::NotFound => OpenDocumentError::FileNotFound,
        _ => OpenDocumentError::ReadFailed,
    }
}

fn deserialize_envelope(contents: &[u8]) -> Result<DocumentEnvelope, OpenDocumentError> {
    let value = serde_json::from_slice(contents).map_err(|_| OpenDocumentError::MalformedJson)?;
    DocumentEnvelope::from_json_value(value)
        .map_err(|cause| OpenDocumentError::InvalidEnvelope { cause })
}

fn validate_snapshot(snapshot: Value) -> Result<DocumentEnvelope, SaveDocumentError> {
    DocumentEnvelope::from_json_value(snapshot)
        .map_err(|cause| SaveDocumentError::InvalidEnvelope { cause })
}

fn serialize_envelope(envelope: &DocumentEnvelope) -> Result<Vec<u8>, SaveDocumentError> {
    serde_json::to_vec(envelope).map_err(|_| SaveDocumentError::SerializationFailed)
}

fn plan_save<SelectPath>(
    registry: &DocumentRegistry,
    document_id: DocumentId,
    select_path: SelectPath,
) -> Result<SavePlan, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
{
    let plan = match registry.source_path(document_id) {
        Ok(Some(path)) => SavePlan::Existing(path),
        Ok(None) => selected_save_plan(select_path, SavePlan::Attach)?,
        Err(DocumentRegistryError::NotOpen) => selected_save_plan(select_path, SavePlan::Register)?,
        Err(cause) => return Err(SaveDocumentError::Registry { cause }),
    };
    validate_target_ownership(registry, document_id, &plan)?;
    Ok(plan)
}

fn selected_save_plan<SelectPath, BuildPlan>(
    select_path: SelectPath,
    build_plan: BuildPlan,
) -> Result<SavePlan, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
    BuildPlan: FnOnce(PathBuf) -> SavePlan,
{
    Ok(match select_path()? {
        Some(path) => build_plan(path),
        None => SavePlan::Cancelled,
    })
}

fn apply_save_plan(
    registry: &DocumentRegistry,
    envelope: DocumentEnvelope,
    contents: Vec<u8>,
    plan: SavePlan,
) -> Result<SaveDocumentOutcome, SaveDocumentError> {
    let document_id = envelope.document_id();
    let Some(target_path) = save_target_path(&plan) else {
        return Ok(SaveDocumentOutcome::Cancelled);
    };
    write_document_atomically(target_path, &contents)
        .map_err(|_| SaveDocumentError::WriteFailed)?;
    commit_registry_update(registry, envelope, plan)?;
    Ok(SaveDocumentOutcome::Saved { document_id })
}

fn validate_target_ownership(
    registry: &DocumentRegistry,
    document_id: DocumentId,
    plan: &SavePlan,
) -> Result<(), SaveDocumentError> {
    let Some(target_path) = save_target_path(plan) else {
        return Ok(());
    };
    registry
        .ensure_source_path_available(document_id, target_path)
        .map_err(|cause| SaveDocumentError::Registry { cause })
}

fn save_target_path(plan: &SavePlan) -> Option<&Path> {
    match plan {
        SavePlan::Existing(path) | SavePlan::Attach(path) | SavePlan::Register(path) => Some(path),
        SavePlan::Cancelled => None,
    }
}

fn commit_registry_update(
    registry: &DocumentRegistry,
    envelope: DocumentEnvelope,
    plan: SavePlan,
) -> Result<(), SaveDocumentError> {
    let result = match plan {
        SavePlan::Existing(_) => registry.update(envelope),
        SavePlan::Attach(path) => registry.update_source(envelope, path),
        SavePlan::Register(path) => registry.open_from_path(envelope, path),
        SavePlan::Cancelled => return Ok(()),
    };
    result.map_err(|cause| SaveDocumentError::Registry { cause })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;
    use crate::documents::test_support::TestDocumentPath;
    use serde_json::json;

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
    const SECOND_DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000002";

    #[test]
    fn document_round_trip_preserves_updated_snapshot() {
        let target = TestDocumentPath::new("round-trip");
        target.write(&serialized_envelope("Original"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");

        let updated = envelope_value("Updated");
        let outcome = save_document(&registry, updated.clone(), no_path_selection)
            .expect("document should save");
        registry
            .close(document_id(&updated))
            .expect("document should close");
        let reopened = open_document(&registry, Some(target.path().to_owned()))
            .expect("document should reopen");

        assert_eq!(
            outcome,
            SaveDocumentOutcome::Saved {
                document_id: document_id(&updated),
            },
        );
        assert_eq!(
            reopened,
            OpenDocumentOutcome::Opened {
                envelope: validated_envelope(updated),
            },
        );
    }

    #[test]
    fn malformed_json_fails_before_registry_entry() {
        let target = TestDocumentPath::new("malformed");
        target.write(br#"{ "schema_version": 1,"#);
        let registry = DocumentRegistry::new();

        assert_eq!(
            open_document(&registry, Some(target.path().to_owned())),
            Err(OpenDocumentError::MalformedJson),
        );
        assert_eq!(
            registry.source_path(document_id(&envelope_value("Missing"))),
            Err(DocumentRegistryError::NotOpen),
        );
    }

    #[test]
    fn unsupported_schema_version_fails_explicitly() {
        let target = TestDocumentPath::new("unsupported-version");
        let mut unsupported = envelope_value("Unsupported");
        unsupported["schema_version"] = json!(2);
        target.write(&serde_json::to_vec(&unsupported).expect("fixture should serialize"));

        assert_eq!(
            open_document(&DocumentRegistry::new(), Some(target.path().to_owned())),
            Err(OpenDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::UnsupportedSchemaVersion { found: 2 },
            }),
        );
    }

    #[test]
    fn duplicate_load_returns_already_open() {
        let target = TestDocumentPath::new("duplicate-load");
        target.write(&serialized_envelope("Duplicate"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("first load should open");

        assert_eq!(
            open_document(&registry, Some(target.path().to_owned())),
            Err(OpenDocumentError::Registry {
                cause: DocumentRegistryError::AlreadyOpen,
            }),
        );
    }

    #[test]
    fn save_uses_explicit_snapshot_and_retained_path() {
        let target = TestDocumentPath::new("explicit-snapshot");
        target.write(&serialized_envelope("Before"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");

        let updated = envelope_value("After");
        save_document(&registry, updated.clone(), no_path_selection).expect("document should save");

        assert_eq!(read_json(target.path()), updated);
    }

    #[test]
    fn save_new_snapshot_uses_rust_selected_path() {
        let target = TestDocumentPath::new("first-save");
        let registry = DocumentRegistry::new();
        let snapshot = envelope_value("First save");

        let outcome = save_document(&registry, snapshot.clone(), || {
            Ok(Some(target.path().to_owned()))
        })
        .expect("new document should save");

        assert_eq!(
            outcome,
            SaveDocumentOutcome::Saved {
                document_id: document_id(&snapshot),
            },
        );
        assert_eq!(read_json(target.path()), snapshot);
    }

    #[test]
    fn cancelled_first_save_does_not_register_document() {
        let registry = DocumentRegistry::new();
        let snapshot = envelope_value("Cancelled");
        let document_id = document_id(&snapshot);

        assert_eq!(
            save_document(&registry, snapshot, || Ok(None)),
            Ok(SaveDocumentOutcome::Cancelled),
        );
        assert_eq!(
            registry.source_path(document_id),
            Err(DocumentRegistryError::NotOpen),
        );
    }

    #[test]
    fn failed_first_save_does_not_register_document() {
        let target = TestDocumentPath::under_missing_parent("failed-first-save");
        let registry = DocumentRegistry::new();
        let snapshot = envelope_value("Failed first save");
        let document_id = document_id(&snapshot);

        assert_eq!(
            save_document(&registry, snapshot, || Ok(Some(target.path().to_owned()))),
            Err(SaveDocumentError::WriteFailed),
        );
        assert_eq!(
            registry.source_path(document_id),
            Err(DocumentRegistryError::NotOpen),
        );
    }

    #[test]
    fn failed_attach_preserves_registry_state() {
        let target = TestDocumentPath::under_missing_parent("failed-attach");
        let registry = DocumentRegistry::new();
        let original = validated_envelope(envelope_value("Original"));
        let document_id = original.document_id();
        registry
            .open(original.clone())
            .expect("document should open");

        let result = save_document(&registry, envelope_value("Updated"), || {
            Ok(Some(target.path().to_owned()))
        });

        assert_eq!(result, Err(SaveDocumentError::WriteFailed));
        assert_eq!(registry.source_path(document_id), Ok(None));
        assert_eq!(registry.close(document_id), Ok(original));
    }

    #[test]
    fn failed_existing_save_preserves_registry_snapshot() {
        let target = TestDocumentPath::under_missing_parent("failed-existing");
        let registry = DocumentRegistry::new();
        let original = validated_envelope(envelope_value("Original"));
        let document_id = original.document_id();
        registry
            .open_from_path(original.clone(), target.path().to_owned())
            .expect("document should open");

        assert_eq!(
            save_document(&registry, envelope_value("Updated"), no_path_selection),
            Err(SaveDocumentError::WriteFailed),
        );
        assert_eq!(registry.close(document_id), Ok(original));
    }

    #[test]
    fn save_rejects_source_path_owned_by_another_document() {
        let target = TestDocumentPath::new("path-conflict");
        let original_bytes = serialized_envelope("First");
        target.write(&original_bytes);
        let registry = DocumentRegistry::new();
        registry
            .open_from_path(
                validated_envelope(envelope_value("First")),
                target.path().to_owned(),
            )
            .expect("first document should open");
        let second = envelope_value_for(SECOND_DOCUMENT_ID, "Second");

        assert_eq!(
            save_document(&registry, second, || Ok(Some(target.path().to_owned()))),
            Err(SaveDocumentError::Registry {
                cause: DocumentRegistryError::SourcePathInUse,
            }),
        );
        assert_eq!(
            fs::read(target.path()).expect("file should remain"),
            original_bytes
        );
    }

    #[test]
    fn invalid_snapshot_fails_before_path_selection() {
        let registry = DocumentRegistry::new();
        let mut snapshot = envelope_value("Invalid");
        snapshot["references"] = json!([]);

        assert_eq!(
            save_document(&registry, snapshot, || panic!(
                "path selection must not run"
            )),
            Err(SaveDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::UnknownEnvelopeField {
                    field: "references".to_owned(),
                },
            }),
        );
    }

    #[test]
    fn save_does_not_reopen_dialog_for_loaded_document() {
        let target = TestDocumentPath::new("retained-path");
        target.write(&serialized_envelope("Loaded"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");

        save_document(&registry, envelope_value("Saved"), || {
            panic!("loaded document must retain its Rust-owned path")
        })
        .expect("document should save");
    }

    fn no_path_selection() -> Result<Option<PathBuf>, SaveDocumentError> {
        panic!("existing document must not request a path")
    }

    fn serialized_envelope(title: &str) -> Vec<u8> {
        serde_json::to_vec(&validated_envelope(envelope_value(title)))
            .expect("envelope should serialize")
    }

    fn validated_envelope(value: Value) -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(value).expect("envelope should validate")
    }

    fn document_id(value: &Value) -> DocumentId {
        validated_envelope(value.clone()).document_id()
    }

    fn envelope_value(title: &str) -> Value {
        envelope_value_for(DOCUMENT_ID, title)
    }

    fn envelope_value_for(document_id: &str, title: &str) -> Value {
        json!({
            "schema_version": 1,
            "document_id": document_id,
            "title": title,
            "document": {
                "type": "doc",
                "content": [{ "type": "paragraph", "content": [] }]
            }
        })
    }

    fn read_json(path: &Path) -> Value {
        let bytes = fs::read(path).expect("saved document should read");
        serde_json::from_slice(&bytes).expect("saved document should be JSON")
    }
}
