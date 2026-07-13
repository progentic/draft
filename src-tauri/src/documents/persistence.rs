use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;

use crate::documents::{
    atomic_write::{AtomicDocumentWriteError, write_document_atomically},
    envelope::{DocumentEnvelope, DocumentEnvelopeError, DocumentId},
    registry::{DocumentRegistry, DocumentRegistryError},
    text_import::{TextImportError, import_text_document},
};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum OpenDocumentError {
    UnsupportedFileLocation,
    UnsupportedFileType,
    FileNotFound,
    ReadFailed,
    MalformedJson,
    InvalidTextEncoding,
    TextTooLarge,
    InvalidEnvelope { cause: DocumentEnvelopeError },
    Registry { cause: DocumentRegistryError },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum SaveDocumentError {
    UnsupportedFileLocation,
    InvalidTarget,
    InvalidEnvelope { cause: DocumentEnvelopeError },
    SerializationFailed,
    Registry { cause: DocumentRegistryError },
    WriteFailed { cause: AtomicDocumentWriteError },
    DurabilityUncertain,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum OpenDocumentOutcome {
    OpenedDraft { envelope: DocumentEnvelope },
    ImportedText { envelope: DocumentEnvelope },
    Cancelled,
}

#[derive(Clone, Copy)]
enum OpenSourceKind {
    Draft,
    Text,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SaveDocumentOutcome {
    Saved {
        document_id: DocumentId,
        display_name: String,
        was_save_as: bool,
    },
    Cancelled,
}

enum SavePlan {
    Existing(PathBuf),
    Attach(PathBuf),
    Register(PathBuf),
    ReplaceSource(PathBuf),
    Cancelled,
}

pub(crate) fn open_document(
    registry: &DocumentRegistry,
    selected_path: Option<PathBuf>,
) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    let Some(source_path) = selected_path else {
        return Ok(OpenDocumentOutcome::Cancelled);
    };
    let source_kind = classify_open_source(&source_path)?;
    let _file_operation = lock_file_operation(registry).map_err(map_open_registry_error)?;
    open_source(registry, source_path, source_kind)
}

fn open_source(
    registry: &DocumentRegistry,
    source_path: PathBuf,
    source_kind: OpenSourceKind,
) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    match source_kind {
        OpenSourceKind::Draft => open_draft_document(registry, source_path),
        OpenSourceKind::Text => import_text_source(&source_path),
    }
}

fn open_draft_document(
    registry: &DocumentRegistry,
    source_path: PathBuf,
) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    let envelope = load_envelope(&source_path)?;
    registry
        .open_from_path(envelope.clone(), source_path)
        .map_err(|cause| OpenDocumentError::Registry { cause })?;
    Ok(OpenDocumentOutcome::OpenedDraft { envelope })
}

fn import_text_source(source_path: &Path) -> Result<OpenDocumentOutcome, OpenDocumentError> {
    import_text_document(source_path)
        .map(|envelope| OpenDocumentOutcome::ImportedText { envelope })
        .map_err(map_text_import_error)
}

fn classify_open_source(path: &Path) -> Result<OpenSourceKind, OpenDocumentError> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("draft") => Ok(OpenSourceKind::Draft),
        Some(extension) if extension.eq_ignore_ascii_case("json") => Ok(OpenSourceKind::Draft),
        Some(extension) if extension.eq_ignore_ascii_case("txt") => Ok(OpenSourceKind::Text),
        Some(extension) if extension.eq_ignore_ascii_case("md") => Ok(OpenSourceKind::Text),
        _ => Err(OpenDocumentError::UnsupportedFileType),
    }
}

fn map_text_import_error(error: TextImportError) -> OpenDocumentError {
    match error {
        TextImportError::FileNotFound => OpenDocumentError::FileNotFound,
        TextImportError::ReadFailed => OpenDocumentError::ReadFailed,
        TextImportError::TooLarge => OpenDocumentError::TextTooLarge,
        TextImportError::InvalidUtf8 => OpenDocumentError::InvalidTextEncoding,
        TextImportError::InvalidEnvelope(cause) => OpenDocumentError::InvalidEnvelope { cause },
    }
}

pub(crate) fn save_document<SelectPath>(
    registry: &DocumentRegistry,
    snapshot: Value,
    select_path: SelectPath,
) -> Result<SaveDocumentOutcome, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
{
    save_document_with_writer(
        registry,
        snapshot,
        select_path,
        false,
        write_document_atomically,
    )
}

pub(crate) fn save_document_as<SelectPath>(
    registry: &DocumentRegistry,
    snapshot: Value,
    select_path: SelectPath,
) -> Result<SaveDocumentOutcome, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
{
    save_document_with_writer(
        registry,
        snapshot,
        select_path,
        true,
        write_document_atomically,
    )
}

pub(crate) fn save_requires_target(
    registry: &DocumentRegistry,
    snapshot: &Value,
) -> Result<bool, SaveDocumentError> {
    let envelope = validate_snapshot(snapshot.clone())?;
    match registry.source_path(envelope.document_id()) {
        Ok(Some(_)) => Ok(false),
        Ok(None) | Err(DocumentRegistryError::NotOpen) => Ok(true),
        Err(cause) => Err(SaveDocumentError::Registry { cause }),
    }
}

fn save_document_with_writer<SelectPath, WriteDocument>(
    registry: &DocumentRegistry,
    snapshot: Value,
    select_path: SelectPath,
    force_new_target: bool,
    write_document: WriteDocument,
) -> Result<SaveDocumentOutcome, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
    WriteDocument: FnOnce(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    let envelope = validate_snapshot(snapshot)?;
    let contents = serialize_envelope(&envelope)?;
    let _file_operation = lock_file_operation(registry).map_err(map_save_registry_error)?;
    let plan = plan_save(
        registry,
        envelope.document_id(),
        select_path,
        force_new_target,
    )?;
    apply_save_plan(registry, envelope, contents, plan, write_document)
}

fn lock_file_operation(
    registry: &DocumentRegistry,
) -> Result<std::sync::MutexGuard<'_, ()>, DocumentRegistryError> {
    registry.lock_file_operations()
}

fn map_open_registry_error(cause: DocumentRegistryError) -> OpenDocumentError {
    OpenDocumentError::Registry { cause }
}

fn map_save_registry_error(cause: DocumentRegistryError) -> SaveDocumentError {
    SaveDocumentError::Registry { cause }
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
    force_new_target: bool,
) -> Result<SavePlan, SaveDocumentError>
where
    SelectPath: FnOnce() -> Result<Option<PathBuf>, SaveDocumentError>,
{
    let plan = match (force_new_target, registry.source_path(document_id)) {
        (true, Ok(Some(_))) => selected_save_plan(select_path, SavePlan::ReplaceSource)?,
        (true, Ok(None)) => selected_save_plan(select_path, SavePlan::Attach)?,
        (true, Err(DocumentRegistryError::NotOpen)) => {
            selected_save_plan(select_path, SavePlan::Register)?
        }
        (true, Err(cause)) => return Err(SaveDocumentError::Registry { cause }),
        (false, Ok(Some(path))) => SavePlan::Existing(path),
        (false, Ok(None)) => selected_save_plan(select_path, SavePlan::Attach)?,
        (false, Err(DocumentRegistryError::NotOpen)) => {
            selected_save_plan(select_path, SavePlan::Register)?
        }
        (false, Err(cause)) => return Err(SaveDocumentError::Registry { cause }),
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
        Some(path) => {
            validate_new_save_target(&path)?;
            build_plan(path)
        }
        None => SavePlan::Cancelled,
    })
}

fn validate_new_save_target(path: &Path) -> Result<(), SaveDocumentError> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("draft") => Ok(()),
        _ => Err(SaveDocumentError::InvalidTarget),
    }
}

fn apply_save_plan<WriteDocument>(
    registry: &DocumentRegistry,
    envelope: DocumentEnvelope,
    contents: Vec<u8>,
    plan: SavePlan,
    write_document: WriteDocument,
) -> Result<SaveDocumentOutcome, SaveDocumentError>
where
    WriteDocument: FnOnce(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    let document_id = envelope.document_id();
    let Some(target_path) = save_target_path(&plan) else {
        return Ok(SaveDocumentOutcome::Cancelled);
    };
    let display_name = save_display_name(target_path)?;
    let was_save_as = save_plan_selects_target(&plan);
    if let Err(cause) = write_document(target_path, &contents) {
        return handle_write_failure(registry, envelope, plan, cause);
    }
    commit_registry_update(registry, envelope, plan)?;
    Ok(SaveDocumentOutcome::Saved {
        document_id,
        display_name,
        was_save_as,
    })
}

fn save_display_name(target_path: &Path) -> Result<String, SaveDocumentError> {
    target_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or(SaveDocumentError::InvalidTarget)
}

fn save_plan_selects_target(plan: &SavePlan) -> bool {
    matches!(
        plan,
        SavePlan::Attach(_) | SavePlan::Register(_) | SavePlan::ReplaceSource(_)
    )
}

fn handle_write_failure(
    registry: &DocumentRegistry,
    envelope: DocumentEnvelope,
    plan: SavePlan,
    cause: AtomicDocumentWriteError,
) -> Result<SaveDocumentOutcome, SaveDocumentError> {
    if cause.target_was_replaced() {
        commit_registry_update(registry, envelope, plan)?;
        return Err(SaveDocumentError::DurabilityUncertain);
    }
    Err(SaveDocumentError::WriteFailed { cause })
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
        SavePlan::Existing(path)
        | SavePlan::Attach(path)
        | SavePlan::Register(path)
        | SavePlan::ReplaceSource(path) => Some(path),
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
        SavePlan::ReplaceSource(path) => registry.update_source(envelope, path),
        SavePlan::Cancelled => return Ok(()),
    };
    result.map_err(|cause| SaveDocumentError::Registry { cause })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        sync::{Arc, mpsc},
        thread,
        time::Duration,
    };

    use super::*;
    use crate::citations::node::CitationNodeError;
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
                display_name: target
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                was_save_as: false,
            },
        );
        assert_eq!(
            reopened,
            OpenDocumentOutcome::OpenedDraft {
                envelope: validated_envelope(updated.clone()),
            },
        );
        assert_eq!(
            open_document(&registry, Some(target.path().to_owned())),
            Err(OpenDocumentError::Registry {
                cause: DocumentRegistryError::AlreadyOpen,
            }),
        );
        assert_eq!(
            registry.close(document_id(&updated)),
            Ok(validated_envelope(updated.clone())),
        );
        assert_eq!(
            registry.close(document_id(&updated)),
            Err(DocumentRegistryError::NotOpen),
        );
    }

    #[test]
    fn font_formatting_persists_through_save_close_and_reopen() {
        let target = TestDocumentPath::new("font-format-round-trip");
        let registry = DocumentRegistry::new();
        let snapshot = envelope_with_font_formatting();
        let expected = validated_envelope(snapshot.clone());
        let document_id = expected.document_id();

        save_document(&registry, snapshot, || Ok(Some(target.path().to_owned()))).unwrap();
        assert_eq!(registry.close(document_id), Ok(expected.clone()));
        assert_eq!(
            open_document(&registry, Some(target.path().to_owned())),
            Ok(OpenDocumentOutcome::OpenedDraft { envelope: expected })
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
    fn current_document_version_loads_without_mutation() {
        let target = TestDocumentPath::new("current-version");
        let value = envelope_value("Current");
        let expected = validated_envelope(value.clone());
        let source = serde_json::to_vec_pretty(&value).unwrap();
        target.write(&source);
        let source_directory = directory_entries(target.path());

        assert_eq!(
            open_document(&DocumentRegistry::new(), Some(target.path().to_owned())),
            Ok(OpenDocumentOutcome::OpenedDraft { envelope: expected }),
        );
        assert_eq!(fs::read(target.path()).unwrap(), source);
        assert_eq!(directory_entries(target.path()), source_directory);
    }

    #[test]
    fn text_import_is_unsaved_and_first_save_preserves_source() {
        let source = TestDocumentPath::with_extension("notes", "txt");
        let target = TestDocumentPath::new("imported-notes");
        let source_bytes = b"First line\nSecond line\n";
        source.write(source_bytes);
        let registry = DocumentRegistry::new();
        let envelope =
            imported_envelope(open_document(&registry, Some(source.path().to_owned())).unwrap());
        let document_id = envelope.document_id();
        let snapshot = serde_json::to_value(&envelope).unwrap();

        assert_eq!(
            registry.source_path(document_id),
            Err(DocumentRegistryError::NotOpen)
        );
        assert_eq!(save_requires_target(&registry, &snapshot), Ok(true));
        save_document(&registry, snapshot, || Ok(Some(target.path().to_owned()))).unwrap();

        assert_eq!(fs::read(source.path()).unwrap(), source_bytes);
        assert_eq!(
            registry.source_path(document_id),
            Ok(Some(target.path().to_owned()))
        );
    }

    #[test]
    fn later_import_save_reuses_only_the_chosen_draft_target() {
        let source = TestDocumentPath::with_extension("source", "txt");
        let target = TestDocumentPath::new("chosen-target");
        let source_bytes = b"Imported source";
        source.write(source_bytes);
        let registry = DocumentRegistry::new();
        let envelope =
            imported_envelope(open_document(&registry, Some(source.path().to_owned())).unwrap());
        let snapshot = serde_json::to_value(&envelope).unwrap();
        save_document(&registry, snapshot.clone(), || {
            Ok(Some(target.path().to_owned()))
        })
        .unwrap();
        let mut updated = snapshot;
        updated["title"] = json!("Updated import");

        save_document(&registry, updated.clone(), || {
            panic!("later save must reuse the DRAFT target")
        })
        .unwrap();

        assert_eq!(fs::read(source.path()).unwrap(), source_bytes);
        assert_eq!(read_json(target.path()), updated);
    }

    #[test]
    fn markdown_import_uses_basename_and_remains_literal_editable_source() {
        let source = TestDocumentPath::with_extension("research-notes", "md");
        let source_bytes = b"# Heading\n\n**literal emphasis**\n- source item";
        source.write(source_bytes);
        let registry = DocumentRegistry::new();
        let envelope =
            imported_envelope(open_document(&registry, Some(source.path().to_owned())).unwrap());

        assert_eq!(envelope.title(), "research-notes.md");
        assert_eq!(
            plain_text(envelope.document()),
            String::from_utf8(source_bytes.to_vec()).unwrap()
        );
        assert!(
            document_nodes(envelope.document())
                .iter()
                .all(|node| node["type"] == "paragraph")
        );
        assert_eq!(
            registry.source_path(envelope.document_id()),
            Err(DocumentRegistryError::NotOpen)
        );
        assert_eq!(fs::read(source.path()).unwrap(), source_bytes);
    }

    #[test]
    fn malformed_and_oversized_text_imports_fail_without_mutation() {
        let malformed = TestDocumentPath::with_extension("malformed", "txt");
        let oversized = TestDocumentPath::with_extension("oversized", "md");
        let malformed_bytes = [0xff, 0xfe, 0xfd];
        let oversized_bytes = vec![b'a'; crate::documents::text_import::MAX_TEXT_IMPORT_BYTES + 1];
        malformed.write(&malformed_bytes);
        oversized.write(&oversized_bytes);
        let registry = DocumentRegistry::new();

        assert_eq!(
            open_document(&registry, Some(malformed.path().to_owned())),
            Err(OpenDocumentError::InvalidTextEncoding),
        );
        assert_eq!(
            open_document(&registry, Some(oversized.path().to_owned())),
            Err(OpenDocumentError::TextTooLarge),
        );
        assert_eq!(fs::read(malformed.path()).unwrap(), malformed_bytes);
        assert_eq!(fs::read(oversized.path()).unwrap(), oversized_bytes);
    }

    #[test]
    fn unsupported_open_extension_fails_before_read_or_registration() {
        let source = TestDocumentPath::with_extension("unsupported", "rtf");
        source.write(b"{\\rtf1 unsupported}");
        let registry = DocumentRegistry::new();

        assert_eq!(
            open_document(&registry, Some(source.path().to_owned())),
            Err(OpenDocumentError::UnsupportedFileType),
        );
    }

    #[test]
    fn unsupported_document_versions_fail_without_mutation() {
        for version in [0, 2] {
            let target = TestDocumentPath::new(&format!("unsupported-version-{version}"));
            let existing_target = TestDocumentPath::new(&format!("existing-version-{version}"));
            let existing = envelope_value_for(SECOND_DOCUMENT_ID, "Existing");
            let existing_envelope = validated_envelope(existing.clone());
            let existing_id = existing_envelope.document_id();
            existing_target.write(&serialized_envelope_value(existing));
            let mut unsupported = envelope_value("Unsupported");
            let document_id = validated_envelope(unsupported.clone()).document_id();
            unsupported["schema_version"] = json!(version);
            let source = serde_json::to_vec(&unsupported).expect("fixture should serialize");
            target.write(&source);
            let registry = DocumentRegistry::new();
            open_document(&registry, Some(existing_target.path().to_owned())).unwrap();
            let existing_source = registry.source_path(existing_id).unwrap();
            let source_directory = directory_entries(target.path());

            assert_eq!(
                open_document(&registry, Some(target.path().to_owned())),
                Err(OpenDocumentError::InvalidEnvelope {
                    cause: DocumentEnvelopeError::UnsupportedSchemaVersion { found: version },
                }),
            );
            assert_eq!(fs::read(target.path()).unwrap(), source);
            assert_eq!(directory_entries(target.path()), source_directory);
            assert_eq!(
                registry.source_path(document_id),
                Err(DocumentRegistryError::NotOpen),
            );
            assert_eq!(registry.source_path(existing_id), Ok(existing_source));
            assert_eq!(registry.close(existing_id), Ok(existing_envelope));
        }
    }

    #[test]
    fn invalid_citation_open_fails_before_registry_entry() {
        let target = TestDocumentPath::new("invalid-citation-open");
        let snapshot = envelope_with_invalid_citation("Invalid citation open");
        target.write(&serde_json::to_vec(&snapshot).unwrap());
        let registry = DocumentRegistry::new();

        assert_eq!(
            open_document(&registry, Some(target.path().to_owned())),
            Err(OpenDocumentError::InvalidEnvelope {
                cause: invalid_citation_error(),
            }),
        );
        assert_eq!(
            registry.source_path(document_id(&envelope_value("Known identity"))),
            Err(DocumentRegistryError::NotOpen),
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
                display_name: target
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                was_save_as: true,
            },
        );
        assert_eq!(read_json(target.path()), snapshot);
    }

    #[test]
    fn save_target_preflight_matches_registry_state() {
        let target = TestDocumentPath::new("preflight");
        target.write(&serialized_envelope("Existing"));
        let registry = DocumentRegistry::new();
        let snapshot = envelope_value("Existing");

        assert_eq!(save_requires_target(&registry, &snapshot), Ok(true));
        open_document(&registry, Some(target.path().to_owned())).unwrap();
        assert_eq!(save_requires_target(&registry, &snapshot), Ok(false));
    }

    #[test]
    fn save_target_preflight_rejects_invalid_snapshot() {
        let registry = DocumentRegistry::new();
        let mut snapshot = envelope_value("Invalid");
        snapshot["schema_version"] = json!(2);

        assert_eq!(
            save_requires_target(&registry, &snapshot),
            Err(SaveDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::UnsupportedSchemaVersion { found: 2 },
            }),
        );
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
    fn first_save_rejects_non_draft_target_before_write() {
        let target = TestDocumentPath::with_extension("invalid-target", "txt");
        let snapshot = envelope_value("Unsaved");
        let document_id = validated_envelope(snapshot.clone()).document_id();
        let registry = DocumentRegistry::new();

        assert_eq!(
            save_document(&registry, snapshot, || Ok(Some(target.path().to_owned()))),
            Err(SaveDocumentError::InvalidTarget),
        );
        assert_eq!(
            registry.source_path(document_id),
            Err(DocumentRegistryError::NotOpen)
        );
        assert!(!target.path().exists());
    }

    #[test]
    fn failed_first_save_does_not_register_document() {
        let target = TestDocumentPath::under_missing_parent("failed-first-save");
        let registry = DocumentRegistry::new();
        let snapshot = envelope_value("Failed first save");
        let document_id = document_id(&snapshot);

        assert_eq!(
            save_document(&registry, snapshot, || Ok(Some(target.path().to_owned()))),
            Err(write_failure(AtomicDocumentWriteError::OpenTemporaryFile)),
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

        assert_eq!(
            result,
            Err(write_failure(AtomicDocumentWriteError::OpenTemporaryFile))
        );
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
            Err(write_failure(AtomicDocumentWriteError::OpenTemporaryFile)),
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
    fn invalid_citation_save_fails_before_path_selection() {
        let registry = DocumentRegistry::new();

        assert_eq!(
            save_document(
                &registry,
                envelope_with_invalid_citation("Invalid citation save"),
                || panic!("path selection must not run"),
            ),
            Err(SaveDocumentError::InvalidEnvelope {
                cause: invalid_citation_error(),
            }),
        );
    }

    #[test]
    fn invalid_font_format_fails_before_path_selection() {
        let registry = DocumentRegistry::new();
        let mut snapshot = envelope_with_font_formatting();
        snapshot["document"]["content"][0]["content"][0]["marks"][1]["attrs"] =
            json!({ "points": 8.5 });

        assert!(matches!(
            save_document(&registry, snapshot, || panic!(
                "path selection must not run"
            )),
            Err(SaveDocumentError::InvalidEnvelope {
                cause: DocumentEnvelopeError::InvalidTextFormat {
                    cause: crate::documents::text_format::TextFormatError::InvalidFontSize,
                    ..
                },
            })
        ));
    }

    #[test]
    fn save_does_not_reopen_dialog_for_loaded_document() {
        let target = TestDocumentPath::new("retained-path");
        target.write(&serialized_envelope("Loaded"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");

        let outcome = save_document(&registry, envelope_value("Saved"), || {
            panic!("loaded document must retain its Rust-owned path")
        })
        .expect("document should save");

        assert_eq!(
            outcome,
            SaveDocumentOutcome::Saved {
                document_id: document_id(&envelope_value("Saved")),
                display_name: target
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                was_save_as: false,
            }
        );
    }

    #[test]
    fn save_as_preserves_old_source_and_rebinds_future_saves() {
        let source = TestDocumentPath::new("save-as-source");
        let target = TestDocumentPath::new("save-as-target");
        let original = serialized_envelope("Original");
        source.write(&original);
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(source.path().to_owned())).unwrap();
        let snapshot = envelope_value("Saved as copy");

        let outcome = save_document_as(&registry, snapshot.clone(), || {
            Ok(Some(target.path().to_owned()))
        })
        .unwrap();

        assert_eq!(
            outcome,
            SaveDocumentOutcome::Saved {
                document_id: document_id(&snapshot),
                display_name: target
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                was_save_as: true,
            }
        );
        assert_eq!(fs::read(source.path()).unwrap(), original);
        assert_eq!(read_json(target.path()), snapshot);
        assert_eq!(
            registry.source_path(document_id(&snapshot)),
            Ok(Some(target.path().to_owned()))
        );

        let later = envelope_value("Later save");
        save_document(&registry, later.clone(), || {
            panic!("save after Save As must reuse the replacement target")
        })
        .unwrap();
        assert_eq!(read_json(target.path()), later);
        assert_eq!(fs::read(source.path()).unwrap(), original);
    }

    #[test]
    fn cancelled_save_as_preserves_current_source_authority() {
        let source = TestDocumentPath::new("cancelled-save-as");
        let original = serialized_envelope("Original");
        source.write(&original);
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(source.path().to_owned())).unwrap();
        let snapshot = envelope_value("Not saved");

        assert_eq!(
            save_document_as(&registry, snapshot.clone(), || Ok(None)),
            Ok(SaveDocumentOutcome::Cancelled)
        );
        assert_eq!(fs::read(source.path()).unwrap(), original);
        assert_eq!(
            registry.source_path(document_id(&snapshot)),
            Ok(Some(source.path().to_owned()))
        );
    }

    #[test]
    fn durability_failure_advances_registry_to_complete_source() {
        let target = TestDocumentPath::new("durability-state");
        target.write(&serialized_envelope("Original"));
        let registry = DocumentRegistry::new();
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");
        let updated = envelope_value("Updated");

        let result = save_document_with_writer(
            &registry,
            updated.clone(),
            no_path_selection,
            false,
            write_then_report_uncertain,
        );

        assert_eq!(result, Err(SaveDocumentError::DurabilityUncertain));
        assert_eq!(read_json(target.path()), updated.clone());
        assert_eq!(
            registry.close(document_id(&updated)),
            Ok(validated_envelope(updated))
        );
    }

    #[test]
    fn concurrent_saves_keep_disk_and_registry_consistent() {
        let target = TestDocumentPath::new("concurrent-save");
        target.write(&serialized_envelope("Original"));
        let registry = Arc::new(DocumentRegistry::new());
        open_document(&registry, Some(target.path().to_owned())).expect("document should open");
        let first = envelope_value("First");
        let second = envelope_value("Second");

        run_ordered_concurrent_saves(&registry, first, second.clone());

        assert_eq!(read_json(target.path()), second.clone());
        assert_eq!(
            registry.close(document_id(&second)),
            Ok(validated_envelope(second))
        );
    }

    fn run_ordered_concurrent_saves(registry: &Arc<DocumentRegistry>, first: Value, second: Value) {
        let (first_entered_tx, first_entered_rx) = mpsc::channel();
        let (release_first_tx, release_first_rx) = mpsc::channel();
        let (second_started_tx, second_started_rx) = mpsc::channel();
        let (second_entered_tx, second_entered_rx) = mpsc::channel();

        thread::scope(|scope| {
            let first_registry = Arc::clone(registry);
            let first_save = scope.spawn(move || {
                save_document_with_writer(
                    &first_registry,
                    first,
                    no_path_selection,
                    false,
                    |path, contents| {
                        write_document_atomically(path, contents)?;
                        first_entered_tx.send(()).unwrap();
                        release_first_rx.recv().unwrap();
                        Ok(())
                    },
                )
            });
            first_entered_rx.recv().unwrap();
            let second_registry = Arc::clone(registry);
            let second_save = scope.spawn(move || {
                second_started_tx.send(()).unwrap();
                save_document_with_writer(
                    &second_registry,
                    second,
                    no_path_selection,
                    false,
                    |path, contents| {
                        second_entered_tx.send(()).unwrap();
                        write_document_atomically(path, contents)
                    },
                )
            });
            second_started_rx.recv().unwrap();
            assert!(
                second_entered_rx
                    .recv_timeout(Duration::from_millis(100))
                    .is_err()
            );
            release_first_tx.send(()).unwrap();
            first_save
                .join()
                .unwrap()
                .expect("first save should finish");
            second_save
                .join()
                .unwrap()
                .expect("second save should finish");
        });
    }

    fn write_then_report_uncertain(
        path: &Path,
        contents: &[u8],
    ) -> Result<(), AtomicDocumentWriteError> {
        write_document_atomically(path, contents)?;
        Err(AtomicDocumentWriteError::SyncParentDirectory)
    }

    fn write_failure(cause: AtomicDocumentWriteError) -> SaveDocumentError {
        SaveDocumentError::WriteFailed { cause }
    }

    fn no_path_selection() -> Result<Option<PathBuf>, SaveDocumentError> {
        panic!("existing document must not request a path")
    }

    fn serialized_envelope(title: &str) -> Vec<u8> {
        serde_json::to_vec(&validated_envelope(envelope_value(title)))
            .expect("envelope should serialize")
    }

    fn serialized_envelope_value(value: Value) -> Vec<u8> {
        serde_json::to_vec(&validated_envelope(value)).expect("envelope should serialize")
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

    fn envelope_with_font_formatting() -> Value {
        let mut value = envelope_value("Formatted");
        value["document"] = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "text",
                    "text": "Mixed formatting",
                    "marks": [
                        { "type": "fontFamily", "attrs": { "family": "courier_new" } },
                        { "type": "fontSize", "attrs": { "points": 21 } }
                    ]
                }]
            }]
        });
        value
    }

    fn envelope_with_invalid_citation(title: &str) -> Value {
        let mut envelope = envelope_value(title);
        envelope["document"]["content"] = json!([{
            "type": "paragraph",
            "content": [{
                "type": "citation",
                "attrs": { "citekey": "smith2025", "render_style": "apa7" }
            }]
        }]);
        envelope
    }

    fn imported_envelope(outcome: OpenDocumentOutcome) -> DocumentEnvelope {
        match outcome {
            OpenDocumentOutcome::ImportedText { envelope } => envelope,
            OpenDocumentOutcome::OpenedDraft { .. } => panic!("text source must import"),
            OpenDocumentOutcome::Cancelled => panic!("explicit text source must not cancel"),
        }
    }

    fn document_nodes(document: &Value) -> &[Value] {
        document["content"].as_array().expect("document content")
    }

    fn plain_text(document: &Value) -> String {
        document_nodes(document)
            .iter()
            .map(|node| {
                node.get("content")
                    .and_then(Value::as_array)
                    .and_then(|content| content.first())
                    .and_then(|text| text.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn invalid_citation_error() -> DocumentEnvelopeError {
        DocumentEnvelopeError::InvalidCitationNode {
            path: "document.content[0].content[0]".to_owned(),
            cause: CitationNodeError::MissingSchemaVersion,
        }
    }

    fn read_json(path: &Path) -> Value {
        let bytes = fs::read(path).expect("saved document should read");
        serde_json::from_slice(&bytes).expect("saved document should be JSON")
    }

    fn directory_entries(path: &Path) -> Vec<PathBuf> {
        let mut entries = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }
}
