use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    documents::{
        atomic_write::{AtomicDocumentWriteError, write_document_atomically},
        envelope::{DocumentEnvelope, DocumentEnvelopeError, DocumentId},
        registry::{DocumentRegistry, DocumentRegistryError},
    },
    exports::docx::{DocxExportError, compile_docx},
    interoperability::{
        docx_import::MAX_DOCX_IMPORT_PACKAGE_BYTES,
        provenance::{CurrentSource, ExternalSourceProvenance, SameFormatSaveDisposition},
    },
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalSaveDecision {
    SaveExact,
    AcceptNormalization,
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SaveExternalDocumentOutcome {
    Saved {
        document_id: DocumentId,
        display_name: String,
        bytes_written: usize,
        disposition: SameFormatSaveDisposition,
    },
    Unchanged {
        document_id: DocumentId,
        display_name: String,
    },
    ConfirmationRequired {
        document_id: DocumentId,
        disposition: SameFormatSaveDisposition,
    },
    Denied {
        document_id: DocumentId,
        disposition: SameFormatSaveDisposition,
    },
    Cancelled {
        document_id: DocumentId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ExternalSourceReadError {
    ReadFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ExternalSaveCommitFailure {
    DurabilityUncertain,
    Registry { cause: DocumentRegistryError },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum SaveExternalDocumentError {
    InvalidEnvelope {
        cause: DocumentEnvelopeError,
    },
    Registry {
        cause: DocumentRegistryError,
    },
    SourceRead {
        cause: ExternalSourceReadError,
    },
    Compilation {
        cause: DocxExportError,
    },
    WriteFailed {
        cause: AtomicDocumentWriteError,
    },
    ReplacementRolledBack {
        cause: ExternalSaveCommitFailure,
    },
    RollbackFailed {
        cause: ExternalSaveCommitFailure,
        rollback: AtomicDocumentWriteError,
    },
}

enum ExternalSavePlan {
    Complete(SaveExternalDocumentOutcome),
    Write(Box<PreparedExternalWrite>),
}

struct PreparedExternalWrite {
    envelope: DocumentEnvelope,
    provenance: ExternalSourceProvenance,
    original_source: Vec<u8>,
    replacement: Vec<u8>,
    disposition: SameFormatSaveDisposition,
}

enum SourceState {
    Bytes(Vec<u8>),
    Missing,
    Changed,
}

pub(crate) fn save_external_document(
    registry: &DocumentRegistry,
    snapshot: Value,
    decision: ExternalSaveDecision,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError> {
    save_external_document_with_writer(registry, snapshot, decision, write_document_atomically)
}

fn save_external_document_with_writer<WriteDocument>(
    registry: &DocumentRegistry,
    snapshot: Value,
    decision: ExternalSaveDecision,
    writer: WriteDocument,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    let envelope = validate_snapshot(snapshot)?;
    if decision == ExternalSaveDecision::Cancel {
        return Ok(cancelled_outcome(&envelope));
    }
    let _file_operation = lock_file_operation(registry)?;
    save_locked(registry, envelope, decision, writer)
}

fn save_locked<WriteDocument>(
    registry: &DocumentRegistry,
    envelope: DocumentEnvelope,
    decision: ExternalSaveDecision,
    writer: WriteDocument,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    let provenance = match registry.external_source(envelope.document_id()) {
        Ok(provenance) => provenance,
        Err(DocumentRegistryError::ExternalSourceUnavailable) => {
            return Ok(denied_missing_provenance(&envelope));
        }
        Err(cause) => return Err(SaveExternalDocumentError::Registry { cause }),
    };
    execute_plan(registry, plan_save(envelope, provenance, decision)?, writer)
}

fn validate_snapshot(snapshot: Value) -> Result<DocumentEnvelope, SaveExternalDocumentError> {
    DocumentEnvelope::from_json_value(snapshot)
        .map_err(|cause| SaveExternalDocumentError::InvalidEnvelope { cause })
}

fn lock_file_operation(
    registry: &DocumentRegistry,
) -> Result<std::sync::MutexGuard<'_, ()>, SaveExternalDocumentError> {
    registry
        .lock_file_operations()
        .map_err(|cause| SaveExternalDocumentError::Registry { cause })
}

fn plan_save(
    envelope: DocumentEnvelope,
    provenance: ExternalSourceProvenance,
    decision: ExternalSaveDecision,
) -> Result<ExternalSavePlan, SaveExternalDocumentError> {
    let source_state = read_owned_source(&provenance)?;
    let disposition = disposition_for(&provenance, &envelope, &source_state);
    choose_plan(envelope, provenance, source_state, disposition, decision)
}

fn disposition_for(
    provenance: &ExternalSourceProvenance,
    envelope: &DocumentEnvelope,
    source_state: &SourceState,
) -> SameFormatSaveDisposition {
    match source_state {
        SourceState::Bytes(bytes) => {
            provenance.save_disposition(envelope, CurrentSource::Bytes(bytes))
        }
        SourceState::Missing => SameFormatSaveDisposition::DeniedSourceMissing,
        SourceState::Changed => SameFormatSaveDisposition::DeniedSourceChanged,
    }
}

fn choose_plan(
    envelope: DocumentEnvelope,
    provenance: ExternalSourceProvenance,
    source_state: SourceState,
    disposition: SameFormatSaveDisposition,
    decision: ExternalSaveDecision,
) -> Result<ExternalSavePlan, SaveExternalDocumentError> {
    match disposition {
        SameFormatSaveDisposition::NoChanges => Ok(unchanged_plan(&envelope, &provenance)),
        SameFormatSaveDisposition::AllowedExact => {
            prepare_write(envelope, provenance, source_state, disposition)
        }
        SameFormatSaveDisposition::AllowedAfterAcceptedNormalization
            if decision == ExternalSaveDecision::AcceptNormalization =>
        {
            prepare_write(envelope, provenance, source_state, disposition)
        }
        SameFormatSaveDisposition::AllowedAfterAcceptedNormalization => {
            Ok(confirmation_plan(&envelope, disposition))
        }
        _ => Ok(denied_plan(&envelope, disposition)),
    }
}

fn prepare_write(
    envelope: DocumentEnvelope,
    provenance: ExternalSourceProvenance,
    source_state: SourceState,
    disposition: SameFormatSaveDisposition,
) -> Result<ExternalSavePlan, SaveExternalDocumentError> {
    let SourceState::Bytes(original_source) = source_state else {
        return Ok(denied_plan(&envelope, disposition));
    };
    let artifact = compile_docx(&envelope)
        .map_err(|cause| SaveExternalDocumentError::Compilation { cause })?;
    Ok(ExternalSavePlan::Write(Box::new(PreparedExternalWrite {
        envelope,
        provenance,
        original_source,
        replacement: artifact.as_bytes().to_vec(),
        disposition,
    })))
}

fn execute_plan<WriteDocument>(
    registry: &DocumentRegistry,
    plan: ExternalSavePlan,
    writer: WriteDocument,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    match plan {
        ExternalSavePlan::Complete(outcome) => Ok(outcome),
        ExternalSavePlan::Write(prepared) => {
            apply_prepared_write(*prepared, writer, |envelope, bytes| {
                registry.commit_external_write(envelope.clone(), bytes)
            })
        }
    }
}

fn apply_prepared_write<WriteDocument, CommitWrite>(
    prepared: PreparedExternalWrite,
    mut writer: WriteDocument,
    commit: CommitWrite,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
    CommitWrite: FnOnce(&DocumentEnvelope, &[u8]) -> Result<(), DocumentRegistryError>,
{
    if !source_still_matches(&prepared)? {
        return Ok(source_changed_outcome(&prepared.envelope));
    }
    replace_and_commit(prepared, &mut writer, commit)
}

fn source_still_matches(
    prepared: &PreparedExternalWrite,
) -> Result<bool, SaveExternalDocumentError> {
    match read_owned_source(&prepared.provenance)? {
        SourceState::Bytes(bytes) => Ok(bytes == prepared.original_source),
        SourceState::Missing | SourceState::Changed => Ok(false),
    }
}

fn replace_and_commit<WriteDocument, CommitWrite>(
    prepared: PreparedExternalWrite,
    writer: &mut WriteDocument,
    commit: CommitWrite,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
    CommitWrite: FnOnce(&DocumentEnvelope, &[u8]) -> Result<(), DocumentRegistryError>,
{
    let target = prepared.provenance.source_path();
    match writer(target, &prepared.replacement) {
        Ok(()) => commit_replacement(prepared, writer, commit),
        Err(cause) if cause.target_was_replaced() => rollback_after(
            prepared,
            writer,
            ExternalSaveCommitFailure::DurabilityUncertain,
        ),
        Err(cause) => Err(SaveExternalDocumentError::WriteFailed { cause }),
    }
}

fn commit_replacement<WriteDocument, CommitWrite>(
    prepared: PreparedExternalWrite,
    writer: &mut WriteDocument,
    commit: CommitWrite,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
    CommitWrite: FnOnce(&DocumentEnvelope, &[u8]) -> Result<(), DocumentRegistryError>,
{
    match commit(&prepared.envelope, &prepared.replacement) {
        Ok(()) => Ok(saved_outcome(&prepared)),
        Err(cause) => rollback_after(
            prepared,
            writer,
            ExternalSaveCommitFailure::Registry { cause },
        ),
    }
}

fn rollback_after<WriteDocument>(
    prepared: PreparedExternalWrite,
    writer: &mut WriteDocument,
    cause: ExternalSaveCommitFailure,
) -> Result<SaveExternalDocumentOutcome, SaveExternalDocumentError>
where
    WriteDocument: FnMut(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    match writer(prepared.provenance.source_path(), &prepared.original_source) {
        Ok(()) => Err(SaveExternalDocumentError::ReplacementRolledBack { cause }),
        Err(rollback) => Err(SaveExternalDocumentError::RollbackFailed { cause, rollback }),
    }
}

fn read_owned_source(
    provenance: &ExternalSourceProvenance,
) -> Result<SourceState, SaveExternalDocumentError> {
    let path = provenance.source_path();
    let canonical = match fs::canonicalize(path) {
        Ok(canonical) => canonical,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(SourceState::Missing),
        Err(_) => return Err(source_read_error()),
    };
    if canonical != path {
        return Ok(SourceState::Changed);
    }
    read_bounded_source(path)
}

fn read_bounded_source(path: &Path) -> Result<SourceState, SaveExternalDocumentError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(SourceState::Missing),
        Err(_) => return Err(source_read_error()),
    };
    if source_is_too_large(&file)? {
        return Ok(SourceState::Changed);
    }
    read_source_bytes(file)
}

fn source_is_too_large(file: &File) -> Result<bool, SaveExternalDocumentError> {
    file.metadata()
        .map(|metadata| metadata.len() > MAX_DOCX_IMPORT_PACKAGE_BYTES as u64)
        .map_err(|_| source_read_error())
}

fn read_source_bytes(file: File) -> Result<SourceState, SaveExternalDocumentError> {
    let mut bytes = Vec::new();
    file.take((MAX_DOCX_IMPORT_PACKAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| source_read_error())?;
    if bytes.len() > MAX_DOCX_IMPORT_PACKAGE_BYTES {
        Ok(SourceState::Changed)
    } else {
        Ok(SourceState::Bytes(bytes))
    }
}

fn source_read_error() -> SaveExternalDocumentError {
    SaveExternalDocumentError::SourceRead {
        cause: ExternalSourceReadError::ReadFailed,
    }
}

fn cancelled_outcome(envelope: &DocumentEnvelope) -> SaveExternalDocumentOutcome {
    SaveExternalDocumentOutcome::Cancelled {
        document_id: envelope.document_id(),
    }
}

fn denied_missing_provenance(envelope: &DocumentEnvelope) -> SaveExternalDocumentOutcome {
    denied_outcome(envelope, SameFormatSaveDisposition::DeniedMissingProvenance)
}

fn source_changed_outcome(envelope: &DocumentEnvelope) -> SaveExternalDocumentOutcome {
    denied_outcome(envelope, SameFormatSaveDisposition::DeniedSourceChanged)
}

fn unchanged_plan(
    envelope: &DocumentEnvelope,
    provenance: &ExternalSourceProvenance,
) -> ExternalSavePlan {
    ExternalSavePlan::Complete(SaveExternalDocumentOutcome::Unchanged {
        document_id: envelope.document_id(),
        display_name: provenance.display_name().to_owned(),
    })
}

fn confirmation_plan(
    envelope: &DocumentEnvelope,
    disposition: SameFormatSaveDisposition,
) -> ExternalSavePlan {
    ExternalSavePlan::Complete(SaveExternalDocumentOutcome::ConfirmationRequired {
        document_id: envelope.document_id(),
        disposition,
    })
}

fn denied_plan(
    envelope: &DocumentEnvelope,
    disposition: SameFormatSaveDisposition,
) -> ExternalSavePlan {
    ExternalSavePlan::Complete(denied_outcome(envelope, disposition))
}

fn denied_outcome(
    envelope: &DocumentEnvelope,
    disposition: SameFormatSaveDisposition,
) -> SaveExternalDocumentOutcome {
    SaveExternalDocumentOutcome::Denied {
        document_id: envelope.document_id(),
        disposition,
    }
}

fn saved_outcome(prepared: &PreparedExternalWrite) -> SaveExternalDocumentOutcome {
    SaveExternalDocumentOutcome::Saved {
        document_id: prepared.envelope.document_id(),
        display_name: prepared.provenance.display_name().to_owned(),
        bytes_written: prepared.replacement.len(),
        disposition: prepared.disposition,
    }
}

#[cfg(test)]
#[path = "external_save_tests.rs"]
mod tests;
