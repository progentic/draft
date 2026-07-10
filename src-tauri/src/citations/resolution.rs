use std::{error::Error, fmt};

use serde::Serialize;
use serde_json::Value;

use crate::{
    citations::node::{CitationNodeAttributes, CitationNodeError, CitationRenderStyle},
    references::store::{ReferenceStore, ReferenceStoreError},
};

/// Disposable editor display data returned after Rust-owned reference resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCitation {
    schema_version: u64,
    citekey: String,
    render_style: CitationRenderStyle,
    display_marker: String,
}

/// Bounded failures from citation validation and local reference resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum CitationResolutionError {
    InvalidCitation { cause: CitationNodeError },
    ReferenceNotFound,
    ReferenceStore { cause: CitationStoreError },
}

/// Store failures reduced to the distinctions a citation caller can act on.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum CitationStoreError {
    Unavailable,
    ReadFailed,
    CorruptReference,
}

/// Resolves one validated citation against the Rust-owned local reference store.
pub fn resolve_citation(
    store: &ReferenceStore,
    attrs: Value,
) -> Result<ResolvedCitation, CitationResolutionError> {
    let citation = CitationNodeAttributes::from_json_value(attrs)
        .map_err(|cause| CitationResolutionError::InvalidCitation { cause })?;
    require_reference(store, citation.citekey())?;
    Ok(resolved_citation(citation))
}

impl fmt::Display for CitationResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for CitationResolutionError {}

impl CitationResolutionError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidCitation { .. } => "citation attributes are invalid",
            Self::ReferenceNotFound => "citation reference was not found",
            Self::ReferenceStore { .. } => "citation reference store failed",
        }
    }
}

fn require_reference(store: &ReferenceStore, citekey: &str) -> Result<(), CitationResolutionError> {
    match store
        .get_by_citekey(citekey)
        .map_err(citation_store_failure)?
    {
        Some(_) => Ok(()),
        None => Err(CitationResolutionError::ReferenceNotFound),
    }
}

fn citation_store_failure(cause: ReferenceStoreError) -> CitationResolutionError {
    CitationResolutionError::ReferenceStore {
        cause: map_store_error(cause),
    }
}

fn map_store_error(error: ReferenceStoreError) -> CitationStoreError {
    match error {
        ReferenceStoreError::ReadFailed => CitationStoreError::ReadFailed,
        ReferenceStoreError::MalformedStoredJson
        | ReferenceStoreError::InvalidStoredRecord { .. }
        | ReferenceStoreError::StoredSchemaMismatch
        | ReferenceStoreError::StoredIdentityMismatch => CitationStoreError::CorruptReference,
        _ => CitationStoreError::Unavailable,
    }
}

fn resolved_citation(citation: CitationNodeAttributes) -> ResolvedCitation {
    let display_marker = format!("[@{}]", citation.citekey());
    ResolvedCitation {
        schema_version: citation.schema_version(),
        citekey: citation.citekey().to_owned(),
        render_style: citation.render_style(),
        display_marker,
    }
}

#[cfg(test)]
#[path = "resolution_tests.rs"]
mod tests;
