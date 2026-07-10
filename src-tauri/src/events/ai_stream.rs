use serde::Serialize;

/// Output provenance attached to every model-generated update.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiOutputClassification {
    GeneratedAnalysis,
}

/// Bounded terminal failure codes exposed by the stream contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStreamFailureCode {
    AdapterStartFailed,
    AdapterStreamFailed,
    InvalidOutput,
}

/// Context accounting disclosed without sending source text in events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStreamContextSummary {
    pub(crate) retained_document_count: usize,
    pub(crate) omitted_document_count: usize,
    pub(crate) retained_evidence_count: usize,
    pub(crate) omitted_evidence_count: usize,
    pub(crate) verified_evidence_ids: Vec<String>,
}

/// Typed lifecycle payload for one generated-analysis stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AiStreamEvent {
    Started {
        stream_id: String,
        classification: AiOutputClassification,
        context: AiStreamContextSummary,
    },
    Chunk {
        stream_id: String,
        sequence: u32,
        classification: AiOutputClassification,
        text: String,
    },
    Completed {
        stream_id: String,
        classification: AiOutputClassification,
        chunk_count: u32,
        total_bytes: usize,
    },
    Cancelled {
        stream_id: String,
        classification: AiOutputClassification,
    },
    Failed {
        stream_id: String,
        classification: AiOutputClassification,
        code: AiStreamFailureCode,
    },
}

/// Opaque event-delivery failure without user or model content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AiStreamEventSinkError;

/// Transport-neutral sink for generated-analysis lifecycle updates.
pub trait AiStreamEventSink {
    fn emit_event(&mut self, event: AiStreamEvent) -> Result<(), AiStreamEventSinkError>;
}

impl<F> AiStreamEventSink for F
where
    F: FnMut(AiStreamEvent) -> Result<(), AiStreamEventSinkError>,
{
    fn emit_event(&mut self, event: AiStreamEvent) -> Result<(), AiStreamEventSinkError> {
        self(event)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn stream_payload_serialization_is_stable() {
        let events = vec![
            AiStreamEvent::Started {
                stream_id: "stream-1".to_owned(),
                classification: AiOutputClassification::GeneratedAnalysis,
                context: AiStreamContextSummary {
                    retained_document_count: 1,
                    omitted_document_count: 2,
                    retained_evidence_count: 3,
                    omitted_evidence_count: 4,
                    verified_evidence_ids: vec!["evidence-1".to_owned()],
                },
            },
            AiStreamEvent::Chunk {
                stream_id: "stream-1".to_owned(),
                sequence: 0,
                classification: AiOutputClassification::GeneratedAnalysis,
                text: "Generated text".to_owned(),
            },
            AiStreamEvent::Completed {
                stream_id: "stream-1".to_owned(),
                classification: AiOutputClassification::GeneratedAnalysis,
                chunk_count: 1,
                total_bytes: 14,
            },
            AiStreamEvent::Cancelled {
                stream_id: "stream-2".to_owned(),
                classification: AiOutputClassification::GeneratedAnalysis,
            },
            AiStreamEvent::Failed {
                stream_id: "stream-3".to_owned(),
                classification: AiOutputClassification::GeneratedAnalysis,
                code: AiStreamFailureCode::AdapterStreamFailed,
            },
        ];

        assert_eq!(
            serde_json::to_value(events).unwrap(),
            json!([
                {
                    "status": "started",
                    "stream_id": "stream-1",
                    "classification": "generated_analysis",
                    "context": {
                        "retainedDocumentCount": 1,
                        "omittedDocumentCount": 2,
                        "retainedEvidenceCount": 3,
                        "omittedEvidenceCount": 4,
                        "verifiedEvidenceIds": ["evidence-1"]
                    }
                },
                {
                    "status": "chunk",
                    "stream_id": "stream-1",
                    "sequence": 0,
                    "classification": "generated_analysis",
                    "text": "Generated text"
                },
                {
                    "status": "completed",
                    "stream_id": "stream-1",
                    "classification": "generated_analysis",
                    "chunk_count": 1,
                    "total_bytes": 14
                },
                {
                    "status": "cancelled",
                    "stream_id": "stream-2",
                    "classification": "generated_analysis"
                },
                {
                    "status": "failed",
                    "stream_id": "stream-3",
                    "classification": "generated_analysis",
                    "code": "adapter_stream_failed"
                }
            ])
        );
    }
}
