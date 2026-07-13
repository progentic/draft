//! Rust-owned external-document import and fidelity policy.

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::documents::envelope::{DocumentEnvelope, DocumentEnvelopeError};

pub(crate) mod docx_import;
pub mod fidelity;
pub(crate) mod provenance;

use docx_import::{DocxImportError, MAX_DOCX_IMPORT_PACKAGE_BYTES, parse_docx_package};
use provenance::{ExternalDocumentSummary, ExternalSourceProvenance};

pub(crate) struct ImportedExternalDocument {
    pub(crate) envelope: DocumentEnvelope,
    pub(crate) summary: ExternalDocumentSummary,
    pub(crate) provenance: ExternalSourceProvenance,
}

/// Bounded failures produced before an external source can enter the registry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum ExternalDocumentImportError {
    FileNotFound,
    ReadFailed,
    PackageTooLarge,
    Docx { cause: DocxImportError },
    InvalidCanonicalDocument { cause: DocumentEnvelopeError },
}

pub(crate) fn import_docx_source(
    source_path: &Path,
) -> Result<ImportedExternalDocument, ExternalDocumentImportError> {
    let canonical_source = canonicalize_source(source_path)?;
    let source_bytes = read_bounded_source(&canonical_source)?;
    let parsed = parse_docx_package(&source_bytes)?;
    let display_name = display_name(source_path)?;
    let title = document_title(source_path, &display_name);
    let envelope = DocumentEnvelope::create_imported(title, parsed.document)
        .map_err(|cause| ExternalDocumentImportError::InvalidCanonicalDocument { cause })?;
    let provenance = ExternalSourceProvenance::imported_docx(
        canonical_source,
        &source_bytes,
        &envelope,
        parsed.fidelity,
    );
    let summary =
        ExternalDocumentSummary::imported_docx(display_name, &provenance, &envelope, &source_bytes);
    Ok(ImportedExternalDocument {
        envelope,
        summary,
        provenance,
    })
}

fn canonicalize_source(source_path: &Path) -> Result<PathBuf, ExternalDocumentImportError> {
    fs::canonicalize(source_path).map_err(map_read_error)
}

fn read_bounded_source(source_path: &Path) -> Result<Vec<u8>, ExternalDocumentImportError> {
    let file = fs::File::open(source_path).map_err(map_read_error)?;
    let length = file.metadata().map_err(map_read_error)?.len();
    if length > MAX_DOCX_IMPORT_PACKAGE_BYTES as u64 {
        return Err(ExternalDocumentImportError::PackageTooLarge);
    }
    let mut bytes = Vec::with_capacity(length as usize);
    file.take((MAX_DOCX_IMPORT_PACKAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(map_read_error)?;
    if bytes.len() > MAX_DOCX_IMPORT_PACKAGE_BYTES {
        return Err(ExternalDocumentImportError::PackageTooLarge);
    }
    Ok(bytes)
}

fn map_read_error(error: io::Error) -> ExternalDocumentImportError {
    match error.kind() {
        io::ErrorKind::NotFound => ExternalDocumentImportError::FileNotFound,
        _ => ExternalDocumentImportError::ReadFailed,
    }
}

fn display_name(source_path: &Path) -> Result<String, ExternalDocumentImportError> {
    source_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .ok_or(ExternalDocumentImportError::ReadFailed)
}

fn document_title(source_path: &Path, display_name: &str) -> String {
    source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(display_name)
        .to_owned()
}

impl From<DocxImportError> for ExternalDocumentImportError {
    fn from(cause: DocxImportError) -> Self {
        Self::Docx { cause }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::test_support::TestDocumentPath;

    #[test]
    fn oversized_source_fails_before_package_parsing() {
        let source = TestDocumentPath::with_extension("oversized-docx", "docx");
        source.write(&vec![0; MAX_DOCX_IMPORT_PACKAGE_BYTES + 1]);

        assert!(matches!(
            import_docx_source(source.path()),
            Err(ExternalDocumentImportError::PackageTooLarge)
        ));
    }
}
