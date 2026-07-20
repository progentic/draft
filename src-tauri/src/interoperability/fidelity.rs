use std::collections::BTreeSet;

use serde::Serialize;

/// External format currently accepted by the bounded interoperability layer.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDocumentFormat {
    Docx,
}

/// Stable, content-free external feature identifiers used in fidelity results.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalFeature {
    AlternateHeadingStyleName,
    AtLeastLineSpacing,
    ContextualSpacing,
    ExactLineSpacing,
    ExternalRelationship,
    Footnote,
    ListIndentation,
    PackagePart,
    PaginationControl,
    ParagraphBorder,
    ParagraphShading,
    ParagraphTab,
    RunFormatting,
    TableStructure,
    UnsupportedDocumentStructure,
    UnsupportedStyleInheritance,
}

/// Stable package and XML safety rejection categories.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSafetyReason {
    ArchiveEntryCount,
    ArchiveEntrySize,
    ArchivePath,
    ArchiveUncompressedSize,
    CompressionRatio,
    DuplicateEntry,
    EncryptedEntry,
    PackageSize,
    RelationshipTarget,
    SymlinkEntry,
    XmlNodeCount,
    XmlDoctype,
    XmlDepth,
    XmlEntity,
    XmlSize,
}

/// Closed fidelity result ordered from exact representation through rejection.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "classification", rename_all = "snake_case")]
pub enum ExternalFidelity {
    Exact,
    CanonicallyNormalized { features: Vec<ExternalFeature> },
    UnsupportedPreservable { features: Vec<ExternalFeature> },
    Lossy { features: Vec<ExternalFeature> },
    MalformedExternalInput,
    UnsupportedExternalFeature { features: Vec<ExternalFeature> },
    Unsafe { reason: ExternalSafetyReason },
}

#[derive(Default)]
pub(crate) struct FidelityAccumulator {
    lossy: BTreeSet<ExternalFeature>,
    normalized: BTreeSet<ExternalFeature>,
    unsupported: BTreeSet<ExternalFeature>,
}

impl FidelityAccumulator {
    pub(crate) fn record_normalization(&mut self, feature: ExternalFeature) {
        self.normalized.insert(feature);
    }

    pub(crate) fn record_unsupported(&mut self, feature: ExternalFeature) {
        self.unsupported.insert(feature);
    }

    pub(crate) fn record_lossy(&mut self, feature: ExternalFeature) {
        self.lossy.insert(feature);
    }

    pub(crate) fn finish(mut self) -> ExternalFidelity {
        if !self.lossy.is_empty() {
            self.lossy.append(&mut self.normalized);
            self.lossy.append(&mut self.unsupported);
            return ExternalFidelity::Lossy {
                features: self.lossy.into_iter().collect(),
            };
        }
        if !self.unsupported.is_empty() {
            return ExternalFidelity::UnsupportedPreservable {
                features: self.unsupported.into_iter().collect(),
            };
        }
        if !self.normalized.is_empty() {
            return ExternalFidelity::CanonicallyNormalized {
                features: self.normalized.into_iter().collect(),
            };
        }
        ExternalFidelity::Exact
    }
}

impl ExternalFidelity {
    pub(crate) fn is_source_preservation_required(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedPreservable { .. } | Self::Lossy { .. }
        )
    }

    pub(crate) fn is_exact(&self) -> bool {
        matches!(self, Self::Exact)
    }

    pub(crate) fn is_normalized(&self) -> bool {
        matches!(self, Self::CanonicallyNormalized { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fidelity_order_is_stable_from_exact_through_unsafe() {
        let ordered = [
            ExternalFidelity::Exact,
            ExternalFidelity::CanonicallyNormalized { features: vec![] },
            ExternalFidelity::UnsupportedPreservable { features: vec![] },
            ExternalFidelity::Lossy { features: vec![] },
            ExternalFidelity::MalformedExternalInput,
            ExternalFidelity::UnsupportedExternalFeature { features: vec![] },
            ExternalFidelity::Unsafe {
                reason: ExternalSafetyReason::PackageSize,
            },
        ];

        assert!(ordered.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn accumulator_deduplicates_and_sorts_features() {
        let mut accumulator = FidelityAccumulator::default();
        accumulator.record_unsupported(ExternalFeature::ParagraphTab);
        accumulator.record_unsupported(ExternalFeature::ParagraphBorder);
        accumulator.record_unsupported(ExternalFeature::ParagraphTab);

        assert_eq!(
            accumulator.finish(),
            ExternalFidelity::UnsupportedPreservable {
                features: vec![
                    ExternalFeature::ParagraphBorder,
                    ExternalFeature::ParagraphTab,
                ],
            }
        );
    }

    #[test]
    fn unsupported_features_take_precedence_over_normalization() {
        let mut accumulator = FidelityAccumulator::default();
        accumulator.record_normalization(ExternalFeature::AlternateHeadingStyleName);
        accumulator.record_unsupported(ExternalFeature::ParagraphShading);

        assert!(matches!(
            accumulator.finish(),
            ExternalFidelity::UnsupportedPreservable { .. }
        ));
    }

    #[test]
    fn lossy_features_take_precedence_and_retain_all_disclosures() {
        let mut accumulator = FidelityAccumulator::default();
        accumulator.record_normalization(ExternalFeature::PaginationControl);
        accumulator.record_unsupported(ExternalFeature::RunFormatting);
        accumulator.record_lossy(ExternalFeature::TableStructure);

        assert_eq!(
            accumulator.finish(),
            ExternalFidelity::Lossy {
                features: vec![
                    ExternalFeature::PaginationControl,
                    ExternalFeature::RunFormatting,
                    ExternalFeature::TableStructure,
                ],
            }
        );
    }

    #[test]
    fn lossy_fidelity_requires_source_preservation() {
        assert!(
            ExternalFidelity::Lossy {
                features: vec![ExternalFeature::Footnote],
            }
            .is_source_preservation_required()
        );
    }
}
