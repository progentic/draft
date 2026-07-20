use serde::Serialize;
use serde_json::Value;

use crate::docx_trace;

use super::fidelity::{
    ExternalFeature, ExternalFidelity, ExternalSafetyReason, FidelityAccumulator,
};

mod document;
mod footnotes;
mod package;
mod table;

pub(crate) const MAX_DOCX_IMPORT_PACKAGE_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_DOCX_IMPORT_XML_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_DOCX_IMPORT_ENTRIES: usize = 128;
pub(crate) const MAX_DOCX_IMPORT_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const MAX_DOCX_IMPORT_XML_DEPTH: usize = 64;
pub(crate) const MAX_DOCX_IMPORT_COMPRESSION_RATIO: u64 = 100;

#[derive(Debug, PartialEq)]
pub(crate) struct ParsedDocx {
    pub(crate) document: Value,
    pub(crate) fidelity: ExternalFidelity,
}

/// Closed, path-free failures from the bounded DOCX reader.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DocxImportError {
    MalformedPackage { fidelity: ExternalFidelity },
    UnsafePackage { fidelity: ExternalFidelity },
    UnsupportedExternalFeature { fidelity: ExternalFidelity },
    LossyImportDenied { fidelity: ExternalFidelity },
}

pub(crate) fn parse_docx_package(bytes: &[u8]) -> Result<ParsedDocx, DocxImportError> {
    if bytes.len() > MAX_DOCX_IMPORT_PACKAGE_BYTES {
        let error = DocxImportError::unsafe_input(ExternalSafetyReason::PackageSize);
        trace_package_decision(&error);
        return Err(error);
    }
    let package = package::read_package(bytes).inspect_err(trace_package_decision)?;
    docx_trace::emit("package_limit_decision", format_args!("result=accepted"));
    let mut fidelity = FidelityAccumulator::default();
    for feature in package.features {
        fidelity.record_unsupported(feature);
    }
    docx_trace::emit(
        "document_xml_parse",
        format_args!("status=started bytes={}", package.document_xml.len()),
    );
    let footnotes = footnotes::FootnoteCatalog::parse(package.footnotes_xml.as_deref())?;
    docx_trace::emit(
        "footnote_parse",
        format_args!("status=completed notes={}", footnotes.len()),
    );
    let document = document::parse_document(&package.document_xml, &footnotes, &mut fidelity)?;
    docx_trace::emit("document_xml_parse", format_args!("status=completed"));
    docx_trace::emit(
        "paragraph_conversion",
        format_args!("blocks={}", canonical_block_count(&document)),
    );
    Ok(ParsedDocx {
        document,
        fidelity: fidelity.finish(),
    })
}

fn trace_package_decision(error: &DocxImportError) {
    docx_trace::emit(
        "package_limit_decision",
        format_args!("result=rejected error={error:?}"),
    );
}

fn canonical_block_count(document: &Value) -> usize {
    document["content"].as_array().map_or(0, Vec::len)
}

impl DocxImportError {
    pub(crate) fn malformed() -> Self {
        Self::MalformedPackage {
            fidelity: ExternalFidelity::MalformedExternalInput,
        }
    }

    pub(crate) fn unsafe_input(reason: ExternalSafetyReason) -> Self {
        Self::UnsafePackage {
            fidelity: ExternalFidelity::Unsafe { reason },
        }
    }

    pub(crate) fn unsupported(features: Vec<ExternalFeature>) -> Self {
        Self::UnsupportedExternalFeature {
            fidelity: ExternalFidelity::UnsupportedExternalFeature { features },
        }
    }

    pub(crate) fn lossy(features: Vec<ExternalFeature>) -> Self {
        Self::LossyImportDenied {
            fidelity: ExternalFidelity::Lossy { features },
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
