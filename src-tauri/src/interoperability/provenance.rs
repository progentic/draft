use std::path::PathBuf;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::documents::envelope::DocumentEnvelope;

use super::fidelity::{ExternalDocumentFormat, ExternalFidelity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExternalAccessMode {
    ImportedReadOnly,
    #[allow(dead_code)]
    OpenedWritable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExternalWriterSupport {
    #[allow(dead_code)]
    Exact,
    #[allow(dead_code)]
    AcceptedNormalization,
    Unavailable,
}

/// Bounded same-format save result decided only by the Rust core.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SameFormatSaveDisposition {
    NoChanges,
    AllowedExact,
    AllowedAfterAcceptedNormalization,
    DeniedUnsupportedSourceBehavior,
    DeniedReadOnly,
    #[allow(dead_code)]
    DeniedMissingProvenance,
    DeniedFidelityUnknown,
    DeniedWriterUnavailable,
    DeniedSourceMissing,
    DeniedSourceChanged,
}

/// Path-free presentation data returned after one external import.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalDocumentSummary {
    format: ExternalDocumentFormat,
    display_name: String,
    fidelity: ExternalFidelity,
    same_format_save: SameFormatSaveDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExternalSourceProvenance {
    source_path: PathBuf,
    source_fingerprint: [u8; 32],
    imported_document_fingerprint: [u8; 32],
    format: ExternalDocumentFormat,
    fidelity: ExternalFidelity,
    access_mode: ExternalAccessMode,
    writer_support: ExternalWriterSupport,
}

pub(crate) enum CurrentSource<'a> {
    Bytes(&'a [u8]),
    #[allow(dead_code)]
    Missing,
}

impl ExternalDocumentSummary {
    pub(crate) fn imported_docx(
        display_name: String,
        provenance: &ExternalSourceProvenance,
        envelope: &DocumentEnvelope,
        source_bytes: &[u8],
    ) -> Self {
        Self {
            format: provenance.format,
            display_name,
            fidelity: provenance.fidelity.clone(),
            same_format_save: provenance
                .save_disposition(envelope, CurrentSource::Bytes(source_bytes)),
        }
    }
}

impl ExternalSourceProvenance {
    pub(crate) fn imported_docx(
        source_path: PathBuf,
        source_bytes: &[u8],
        envelope: &DocumentEnvelope,
        fidelity: ExternalFidelity,
    ) -> Self {
        Self {
            source_path,
            source_fingerprint: fingerprint_bytes(source_bytes),
            imported_document_fingerprint: fingerprint_envelope(envelope),
            format: ExternalDocumentFormat::Docx,
            fidelity,
            access_mode: ExternalAccessMode::ImportedReadOnly,
            writer_support: ExternalWriterSupport::Unavailable,
        }
    }

    pub(crate) fn source_path(&self) -> &std::path::Path {
        &self.source_path
    }

    pub(crate) fn save_disposition(
        &self,
        envelope: &DocumentEnvelope,
        current_source: CurrentSource<'_>,
    ) -> SameFormatSaveDisposition {
        evaluate_source_state(self, envelope, current_source)
    }
}

fn evaluate_source_state(
    provenance: &ExternalSourceProvenance,
    envelope: &DocumentEnvelope,
    current_source: CurrentSource<'_>,
) -> SameFormatSaveDisposition {
    let CurrentSource::Bytes(source_bytes) = current_source else {
        return SameFormatSaveDisposition::DeniedSourceMissing;
    };
    if fingerprint_bytes(source_bytes) != provenance.source_fingerprint {
        return SameFormatSaveDisposition::DeniedSourceChanged;
    }
    evaluate_document_state(provenance, envelope)
}

fn evaluate_document_state(
    provenance: &ExternalSourceProvenance,
    envelope: &DocumentEnvelope,
) -> SameFormatSaveDisposition {
    if fingerprint_envelope(envelope) == provenance.imported_document_fingerprint {
        return SameFormatSaveDisposition::NoChanges;
    }
    if provenance.fidelity.is_source_preservation_required() {
        return SameFormatSaveDisposition::DeniedUnsupportedSourceBehavior;
    }
    if provenance.access_mode == ExternalAccessMode::ImportedReadOnly {
        return SameFormatSaveDisposition::DeniedReadOnly;
    }
    writer_disposition(provenance)
}

fn writer_disposition(provenance: &ExternalSourceProvenance) -> SameFormatSaveDisposition {
    match provenance.writer_support {
        ExternalWriterSupport::Exact if provenance.fidelity.is_exact() => {
            SameFormatSaveDisposition::AllowedExact
        }
        ExternalWriterSupport::AcceptedNormalization if provenance.fidelity.is_normalized() => {
            SameFormatSaveDisposition::AllowedAfterAcceptedNormalization
        }
        ExternalWriterSupport::Unavailable => SameFormatSaveDisposition::DeniedWriterUnavailable,
        _ => SameFormatSaveDisposition::DeniedFidelityUnknown,
    }
}

fn fingerprint_envelope(envelope: &DocumentEnvelope) -> [u8; 32] {
    let bytes = serde_json::to_vec(envelope).expect("validated envelopes must serialize");
    fingerprint_bytes(&bytes)
}

fn fingerprint_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION;
    use crate::interoperability::fidelity::ExternalFeature;

    #[test]
    fn imported_source_reports_no_change_then_rejects_edits_as_read_only() {
        let original = envelope("Original");
        let provenance = imported_provenance(&original, ExternalFidelity::Exact);

        assert_eq!(
            provenance.save_disposition(&original, CurrentSource::Bytes(b"source")),
            SameFormatSaveDisposition::NoChanges
        );
        assert_eq!(
            provenance.save_disposition(&envelope("Edited"), CurrentSource::Bytes(b"source")),
            SameFormatSaveDisposition::DeniedReadOnly
        );
    }

    #[test]
    fn unsupported_source_behavior_precedes_read_only_disposition() {
        let original = envelope("Original");
        let fidelity = ExternalFidelity::UnsupportedPreservable {
            features: vec![ExternalFeature::ParagraphBorder],
        };
        let provenance = imported_provenance(&original, fidelity);

        assert_eq!(
            provenance.save_disposition(&envelope("Edited"), CurrentSource::Bytes(b"source")),
            SameFormatSaveDisposition::DeniedUnsupportedSourceBehavior
        );
    }

    #[test]
    fn missing_and_changed_sources_fail_before_writer_policy() {
        let original = envelope("Original");
        let provenance = imported_provenance(&original, ExternalFidelity::Exact);

        assert_eq!(
            provenance.save_disposition(&original, CurrentSource::Missing),
            SameFormatSaveDisposition::DeniedSourceMissing
        );
        assert_eq!(
            provenance.save_disposition(&original, CurrentSource::Bytes(b"changed")),
            SameFormatSaveDisposition::DeniedSourceChanged
        );
    }

    #[test]
    fn writer_policy_is_closed_and_deterministic() {
        let original = envelope("Original");
        for (fidelity, support, expected) in [
            (
                ExternalFidelity::Exact,
                ExternalWriterSupport::Exact,
                SameFormatSaveDisposition::AllowedExact,
            ),
            (
                ExternalFidelity::CanonicallyNormalized { features: vec![] },
                ExternalWriterSupport::AcceptedNormalization,
                SameFormatSaveDisposition::AllowedAfterAcceptedNormalization,
            ),
            (
                ExternalFidelity::Exact,
                ExternalWriterSupport::Unavailable,
                SameFormatSaveDisposition::DeniedWriterUnavailable,
            ),
        ] {
            let mut provenance = imported_provenance(&original, fidelity);
            provenance.access_mode = ExternalAccessMode::OpenedWritable;
            provenance.writer_support = support;
            assert_eq!(
                provenance.save_disposition(&envelope("Edited"), CurrentSource::Bytes(b"source")),
                expected
            );
        }
    }

    #[test]
    fn summary_serialization_is_path_free() {
        let envelope = envelope("Original");
        let provenance = imported_provenance(&envelope, ExternalFidelity::Exact);
        let summary = ExternalDocumentSummary::imported_docx(
            "paper.docx".to_owned(),
            &provenance,
            &envelope,
            b"source",
        );
        let value = serde_json::to_value(summary).unwrap();

        assert_eq!(value["displayName"], "paper.docx");
        assert!(value.get("path").is_none());
        assert!(value.get("fingerprint").is_none());
    }

    fn imported_provenance(
        envelope: &DocumentEnvelope,
        fidelity: ExternalFidelity,
    ) -> ExternalSourceProvenance {
        ExternalSourceProvenance::imported_docx(
            PathBuf::from("paper.docx"),
            b"source",
            envelope,
            fidelity,
        )
    }

    fn envelope(title: &str) -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(json!({
            "schema_version": DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": "00000000-0000-4000-8000-000000000001",
            "title": title,
            "document": { "type": "doc", "content": [] }
        }))
        .unwrap()
    }
}
