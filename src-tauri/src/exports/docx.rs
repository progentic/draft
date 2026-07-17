use std::{error::Error, fmt, path::Path};

use serde::Serialize;

use crate::documents::{
    atomic_write::{AtomicDocumentWriteError, write_document_atomically},
    envelope::DocumentEnvelope,
};
use crate::docx_trace;

use super::{docx_model::parse_docx_document, docx_package::build_docx_package};

/// Maximum serialized Tiptap document bytes accepted for DOCX compilation.
pub const MAX_DOCX_SOURCE_BYTES: usize = 8 * 1024 * 1024;
/// Maximum Tiptap nodes traversed during DOCX compilation.
pub const MAX_DOCX_NODES: usize = 100_000;
/// Maximum JSON nesting depth traversed during DOCX compilation.
pub const MAX_DOCX_NESTING_DEPTH: usize = 16;
/// Maximum complete DOCX artifact size retained or written by the exporter.
pub const MAX_DOCX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;

/// One compiled in-memory DOCX artifact with no source-document authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocxArtifact {
    bytes: Vec<u8>,
}

/// Bounded structural location containing indexes only, never source content.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocxContentPath {
    indexes: Vec<usize>,
}

/// Atomic write stage that prevented a DOCX target replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocxWriteStage {
    OpenTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    ReplaceTarget,
    CleanupTemporaryFile,
}

/// Bounded failures from strict DOCX compilation and atomic export.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DocxExportError {
    InvalidTarget,
    SourceTooLarge,
    TooManyNodes,
    NestingTooDeep,
    InvalidDocumentStructure { path: DocxContentPath },
    UnsupportedDocumentContent { path: DocxContentPath },
    UnsupportedCitation { path: DocxContentPath },
    PackageConstructionFailed,
    ArtifactTooLarge,
    WriteFailed { stage: DocxWriteStage },
    DurabilityUncertain,
}

/// Successful atomic export metadata without exposing a filesystem path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocxExportOutcome {
    bytes_written: usize,
}

/// Compiles one validated immutable document into deterministic DOCX bytes.
pub fn compile_docx(document: &DocumentEnvelope) -> Result<DocxArtifact, DocxExportError> {
    compile_docx_with_limit(document, MAX_DOCX_ARTIFACT_BYTES)
}

/// Compiles and atomically replaces one Rust-owned `.docx` target.
pub fn export_docx(
    document: &DocumentEnvelope,
    target_path: &Path,
) -> Result<DocxExportOutcome, DocxExportError> {
    export_docx_with_writer(document, target_path, write_document_atomically)
}

impl DocxArtifact {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl DocxContentPath {
    pub fn indexes(&self) -> &[usize] {
        &self.indexes
    }

    pub(crate) fn root() -> Self {
        Self { indexes: vec![] }
    }

    pub(crate) fn child(&self, index: usize) -> Self {
        let mut indexes = self.indexes.clone();
        indexes.push(index);
        Self { indexes }
    }
}

impl DocxExportOutcome {
    pub fn bytes_written(self) -> usize {
        self.bytes_written
    }
}

impl fmt::Display for DocxExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for DocxExportError {}

impl DocxExportError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidTarget => "DOCX export target is invalid",
            Self::SourceTooLarge => "document is too large for DOCX export",
            Self::TooManyNodes => "document contains too many nodes for DOCX export",
            Self::NestingTooDeep => "document is nested too deeply for DOCX export",
            Self::InvalidDocumentStructure { .. } => {
                "document structure is invalid for DOCX export"
            }
            Self::UnsupportedDocumentContent { .. } => {
                "document contains content not supported by DOCX export"
            }
            Self::UnsupportedCitation { .. } => {
                "citation rendering is not supported by DOCX export"
            }
            Self::PackageConstructionFailed => "DOCX package construction failed",
            Self::ArtifactTooLarge => "compiled DOCX artifact is too large",
            Self::WriteFailed { .. } => "DOCX target could not be replaced",
            Self::DurabilityUncertain => "DOCX was replaced but durability is uncertain",
        }
    }
}

fn compile_docx_with_limit(
    document: &DocumentEnvelope,
    artifact_limit: usize,
) -> Result<DocxArtifact, DocxExportError> {
    let model = parse_docx_document(document.document())?;
    docx_trace::emit("export_model_built", format_args!("status=accepted"));
    let bytes = build_docx_package(&model, artifact_limit)?;
    docx_trace::emit(
        "export_package_built",
        format_args!("bytes={}", bytes.len()),
    );
    Ok(DocxArtifact { bytes })
}

fn export_docx_with_writer<WriteArtifact>(
    document: &DocumentEnvelope,
    target_path: &Path,
    write_artifact: WriteArtifact,
) -> Result<DocxExportOutcome, DocxExportError>
where
    WriteArtifact: FnOnce(&Path, &[u8]) -> Result<(), AtomicDocumentWriteError>,
{
    require_docx_target(target_path)?;
    let artifact = compile_docx(document)?;
    write_artifact(target_path, artifact.as_bytes()).map_err(map_write_error)?;
    docx_trace::emit(
        "export_package_written",
        format_args!("bytes={}", artifact.len()),
    );
    docx_trace::emit("export_target_promoted", format_args!("status=completed"));
    Ok(DocxExportOutcome {
        bytes_written: artifact.len(),
    })
}

fn require_docx_target(target_path: &Path) -> Result<(), DocxExportError> {
    let is_docx = target_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("docx"));
    if is_docx {
        Ok(())
    } else {
        Err(DocxExportError::InvalidTarget)
    }
}

fn map_write_error(error: AtomicDocumentWriteError) -> DocxExportError {
    match error {
        AtomicDocumentWriteError::OpenTemporaryFile => {
            write_failure(DocxWriteStage::OpenTemporaryFile)
        }
        AtomicDocumentWriteError::WriteTemporaryFile => {
            write_failure(DocxWriteStage::WriteTemporaryFile)
        }
        AtomicDocumentWriteError::SyncTemporaryFile => {
            write_failure(DocxWriteStage::SyncTemporaryFile)
        }
        AtomicDocumentWriteError::ReplaceTarget => write_failure(DocxWriteStage::ReplaceTarget),
        AtomicDocumentWriteError::CleanupTemporaryFile => {
            write_failure(DocxWriteStage::CleanupTemporaryFile)
        }
        AtomicDocumentWriteError::SyncParentDirectory => DocxExportError::DurabilityUncertain,
    }
}

fn write_failure(stage: DocxWriteStage) -> DocxExportError {
    DocxExportError::WriteFailed { stage }
}

#[cfg(test)]
#[path = "docx_tests.rs"]
mod tests;
