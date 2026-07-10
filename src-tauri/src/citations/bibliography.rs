use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use crate::{documents::envelope::DocumentEnvelope, references::record::ReferenceRecord};

use super::node::{CitationNodeError, LocatedCitationError, document_citations};

/// Deterministic differences between document citations and one candidate bibliography.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BibliographyConsistencyReport {
    missing_citekeys: Vec<String>,
    orphaned_citekeys: Vec<String>,
    duplicate_citekeys: Vec<String>,
}

/// Fail-closed citation extraction error from a bibliography consistency check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BibliographyConsistencyError {
    InvalidCitation {
        path: String,
        cause: CitationNodeError,
    },
}

/// Compares validated in-text citations with an explicit candidate bibliography.
pub fn check_bibliography_consistency(
    document: &DocumentEnvelope,
    bibliography: &[ReferenceRecord],
) -> Result<BibliographyConsistencyReport, BibliographyConsistencyError> {
    let cited_citekeys = collect_cited_citekeys(document)?;
    let bibliography_counts = count_bibliography_citekeys(bibliography);
    Ok(build_report(cited_citekeys, bibliography_counts))
}

impl BibliographyConsistencyReport {
    /// Returns citekeys used in text but absent from the candidate bibliography.
    pub fn missing_citekeys(&self) -> &[String] {
        &self.missing_citekeys
    }

    /// Returns bibliography citekeys that are not used in the document.
    pub fn orphaned_citekeys(&self) -> &[String] {
        &self.orphaned_citekeys
    }

    /// Returns citekeys repeated in the candidate bibliography.
    pub fn duplicate_citekeys(&self) -> &[String] {
        &self.duplicate_citekeys
    }

    /// Reports whether every citation and bibliography citekey agrees.
    pub fn is_consistent(&self) -> bool {
        self.missing_citekeys.is_empty()
            && self.orphaned_citekeys.is_empty()
            && self.duplicate_citekeys.is_empty()
    }
}

impl fmt::Display for BibliographyConsistencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("document contains an invalid citation")
    }
}

impl Error for BibliographyConsistencyError {}

impl From<LocatedCitationError> for BibliographyConsistencyError {
    fn from(error: LocatedCitationError) -> Self {
        Self::InvalidCitation {
            path: error.path,
            cause: error.cause,
        }
    }
}

fn collect_cited_citekeys(
    document: &DocumentEnvelope,
) -> Result<BTreeSet<String>, BibliographyConsistencyError> {
    document_citations(document.document())
        .map(|citations| {
            citations
                .into_iter()
                .map(|citation| citation.citekey().to_owned())
                .collect()
        })
        .map_err(Into::into)
}

fn count_bibliography_citekeys(bibliography: &[ReferenceRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in bibliography {
        *counts.entry(record.citekey().to_owned()).or_default() += 1;
    }
    counts
}

fn build_report(
    cited_citekeys: BTreeSet<String>,
    bibliography_counts: BTreeMap<String, usize>,
) -> BibliographyConsistencyReport {
    let bibliography_citekeys = bibliography_counts.keys().cloned().collect();
    BibliographyConsistencyReport {
        missing_citekeys: set_difference(&cited_citekeys, &bibliography_citekeys),
        orphaned_citekeys: set_difference(&bibliography_citekeys, &cited_citekeys),
        duplicate_citekeys: duplicate_citekeys(&bibliography_counts),
    }
}

fn set_difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

fn duplicate_citekeys(counts: &BTreeMap<String, usize>) -> Vec<String> {
    counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(citekey, _)| citekey.clone())
        .collect()
}

#[cfg(test)]
#[path = "bibliography_tests.rs"]
mod tests;
