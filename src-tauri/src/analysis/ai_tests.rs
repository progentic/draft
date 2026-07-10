use std::{
    collections::VecDeque,
    future::{pending, ready},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use crate::{
    analysis::context::{AiAnalysisTask, AiContextBlock, AiEvidenceExcerpt},
    events::ai_stream::{
        AiOutputClassification, AiStreamEvent, AiStreamEventSink, AiStreamEventSinkError,
        AiStreamFailureCode,
    },
    workers::cancellation::{CancelWorkerOutcome, WorkerCancellationRegistry, WorkerId},
};

use super::*;

#[test]
fn preparation_registers_stream_before_adapter_work() {
    let registry = WorkerCancellationRegistry::new();
    let request = request();

    let prepared = prepare_ai_generation(&registry, &request).unwrap();
    let stream_id = prepared.stream_id();

    assert_eq!(prepared.model_request().instruction(), "Review the claim.");
    assert_eq!(
        registry.cancel(stream_id),
        Ok(CancelWorkerOutcome::CancellationRequested)
    );
    drop(prepared);
    assert_eq!(
        registry.cancel(stream_id),
        Ok(CancelWorkerOutcome::AlreadyEnded)
    );
}

#[test]
fn successful_stream_is_ordered_and_generated_analysis_only() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let stream_id = prepared.stream_id();
    let adapter = FakeAdapter::chunks(["First ", "second"]);
    let mut sink = RecordingSink::default();

    let outcome = tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink));

    assert_eq!(
        outcome,
        Ok(AiGenerationOutcome::Completed {
            chunk_count: 2,
            total_bytes: 12,
        })
    );
    assert_started(&sink.events[0], stream_id);
    assert_chunk(&sink.events[1], 0, "First ");
    assert_chunk(&sink.events[2], 1, "second");
    assert!(matches!(
        sink.events[3],
        AiStreamEvent::Completed {
            classification: AiOutputClassification::GeneratedAnalysis,
            chunk_count: 2,
            total_bytes: 12,
            ..
        }
    ));
    assert_eq!(
        registry.cancel(stream_id),
        Ok(CancelWorkerOutcome::AlreadyEnded)
    );
}

#[test]
fn adapter_receives_typed_provenance_not_flattened_text() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let adapter = FakeAdapter::chunks(["analysis"]);
    let mut sink = RecordingSink::default();

    tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink)).unwrap();
    let captured = adapter.captured.lock().unwrap().clone().unwrap();

    assert!(matches!(
        &captured.blocks()[0],
        AiContextBlock::VerifiedSourceEvidence { evidence_id, citekey, .. }
            if evidence_id == "evidence-1" && citekey == "smith2025"
    ));
    assert!(
        captured
            .blocks()
            .iter()
            .any(|block| matches!(block, AiContextBlock::UserDocument { .. }))
    );
}

#[test]
fn cancellation_before_run_avoids_adapter_start() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    registry.cancel(prepared.stream_id()).unwrap();
    let adapter = FakeAdapter::chunks(["unused"]);
    let mut sink = RecordingSink::default();

    let outcome = tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink));

    assert_eq!(outcome, Ok(AiGenerationOutcome::Cancelled));
    assert_eq!(adapter.start_count.load(Ordering::SeqCst), 0);
    assert!(matches!(sink.events[1], AiStreamEvent::Cancelled { .. }));
}

#[test]
fn cancellation_during_read_cancels_adapter_and_emits_terminal() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let stream_id = prepared.stream_id();
    let cancelled = Arc::new(AtomicBool::new(false));
    let adapter = CancellingAdapter {
        registry: registry.clone(),
        stream_id,
        cancelled: Arc::clone(&cancelled),
    };
    let mut sink = RecordingSink::default();

    let outcome = tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink));

    assert_eq!(outcome, Ok(AiGenerationOutcome::Cancelled));
    assert!(cancelled.load(Ordering::SeqCst));
    assert!(matches!(sink.events[1], AiStreamEvent::Cancelled { .. }));
}

#[test]
fn adapter_start_and_stream_failures_emit_bounded_terminal_events() {
    let registry = WorkerCancellationRegistry::new();
    let start_prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let start_adapter = FakeAdapter::start_failure();
    let mut start_sink = RecordingSink::default();
    let start_outcome = tauri::async_runtime::block_on(run_ai_generation(
        start_prepared,
        &start_adapter,
        &mut start_sink,
    ));
    assert_eq!(
        start_outcome,
        Ok(AiGenerationOutcome::Failed {
            code: AiStreamFailureCode::AdapterStartFailed,
        })
    );
    assert_failed(
        &start_sink.events[1],
        AiStreamFailureCode::AdapterStartFailed,
    );

    let stream_prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let stream_adapter = FakeAdapter::stream_failure();
    let mut stream_sink = RecordingSink::default();
    let stream_outcome = tauri::async_runtime::block_on(run_ai_generation(
        stream_prepared,
        &stream_adapter,
        &mut stream_sink,
    ));
    assert_eq!(
        stream_outcome,
        Ok(AiGenerationOutcome::Failed {
            code: AiStreamFailureCode::AdapterStreamFailed,
        })
    );
    assert!(stream_adapter.cancelled.load(Ordering::SeqCst));
    assert_failed(
        &stream_sink.events[1],
        AiStreamFailureCode::AdapterStreamFailed,
    );
}

#[test]
fn invalid_or_excessive_chunks_cancel_stream_and_fail_typed() {
    let registry = WorkerCancellationRegistry::new();
    let empty_prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let empty_adapter = FakeAdapter::chunks(["   "]);
    let mut empty_sink = RecordingSink::default();
    let empty_outcome = tauri::async_runtime::block_on(run_ai_generation(
        empty_prepared,
        &empty_adapter,
        &mut empty_sink,
    ));
    assert_eq!(
        empty_outcome,
        Ok(AiGenerationOutcome::Failed {
            code: AiStreamFailureCode::InvalidOutput,
        })
    );
    assert!(empty_adapter.cancelled.load(Ordering::SeqCst));

    let large_prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let large_adapter = FakeAdapter::chunks(["x".repeat(MAX_AI_STREAM_CHUNK_BYTES + 1)]);
    let mut large_sink = RecordingSink::default();
    let large_outcome = tauri::async_runtime::block_on(run_ai_generation(
        large_prepared,
        &large_adapter,
        &mut large_sink,
    ));
    assert_eq!(
        large_outcome,
        Ok(AiGenerationOutcome::Failed {
            code: AiStreamFailureCode::InvalidOutput,
        })
    );
}

#[test]
fn cumulative_output_limit_is_enforced() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let chunks = vec!["x".repeat(MAX_AI_STREAM_CHUNK_BYTES); 65];
    let adapter = FakeAdapter::chunks(chunks);
    let mut sink = RecordingSink::default();

    let outcome = tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink));

    assert_eq!(
        outcome,
        Ok(AiGenerationOutcome::Failed {
            code: AiStreamFailureCode::InvalidOutput,
        })
    );
    assert_eq!(
        sink.events
            .iter()
            .filter(|event| matches!(event, AiStreamEvent::Chunk { .. }))
            .count(),
        64
    );
}

#[test]
fn event_delivery_failure_stops_adapter_without_content_error() {
    let registry = WorkerCancellationRegistry::new();
    let prepared = prepare_ai_generation(&registry, &request()).unwrap();
    let stream_id = prepared.stream_id();
    let adapter = FakeAdapter::chunks(["first", "second"]);
    let mut sink = RecordingSink {
        events: Vec::new(),
        fail_at: Some(1),
    };

    let outcome = tauri::async_runtime::block_on(run_ai_generation(prepared, &adapter, &mut sink));

    assert_eq!(outcome, Err(AiRunError::EventDeliveryFailed));
    assert!(adapter.cancelled.load(Ordering::SeqCst));
    assert_eq!(
        registry.cancel(stream_id),
        Ok(CancelWorkerOutcome::AlreadyEnded)
    );
    assert_eq!(
        AiRunError::EventDeliveryFailed.to_string(),
        "AI stream event delivery failed"
    );
}

#[test]
fn adapter_and_preparation_errors_do_not_include_context() {
    assert_eq!(
        AiModelAdapterError::RequestRejected.to_string(),
        "model adapter rejected the request"
    );
    assert_eq!(
        AiPreparationError::CancellationRegistryUnavailable.to_string(),
        "AI cancellation registry is unavailable"
    );
}

fn request() -> AiAnalysisRequest {
    AiAnalysisRequest::new(
        AiAnalysisTask::FactCheckSupport,
        "Review the claim.",
        vec!["The document makes a claim.".to_owned()],
        vec![
            AiEvidenceExcerpt::new(
                "evidence-1",
                "smith2025",
                "The source provides relevant evidence.",
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn assert_started(event: &AiStreamEvent, stream_id: WorkerId) {
    assert!(matches!(
        event,
        AiStreamEvent::Started {
            stream_id: actual,
            classification: AiOutputClassification::GeneratedAnalysis,
            context,
        } if actual == &stream_id.to_string()
            && context.verified_evidence_ids == ["evidence-1"]
    ));
}

fn assert_chunk(event: &AiStreamEvent, sequence: u32, expected_text: &str) {
    assert!(matches!(
        event,
        AiStreamEvent::Chunk {
            sequence: actual_sequence,
            classification: AiOutputClassification::GeneratedAnalysis,
            text,
            ..
        } if *actual_sequence == sequence && text == expected_text
    ));
}

fn assert_failed(event: &AiStreamEvent, expected: AiStreamFailureCode) {
    assert!(matches!(
        event,
        AiStreamEvent::Failed {
            classification: AiOutputClassification::GeneratedAnalysis,
            code,
            ..
        } if *code == expected
    ));
}

#[derive(Default)]
struct RecordingSink {
    events: Vec<AiStreamEvent>,
    fail_at: Option<usize>,
}

impl AiStreamEventSink for RecordingSink {
    fn emit_event(&mut self, event: AiStreamEvent) -> Result<(), AiStreamEventSinkError> {
        if self.fail_at == Some(self.events.len()) {
            return Err(AiStreamEventSinkError);
        }
        self.events.push(event);
        Ok(())
    }
}

struct FakeAdapter {
    outcomes: Vec<Result<Option<String>, AiModelAdapterError>>,
    start_error: Option<AiModelAdapterError>,
    start_count: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    captured: Arc<Mutex<Option<AiModelRequest>>>,
}

impl FakeAdapter {
    fn chunks(chunks: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut outcomes = chunks
            .into_iter()
            .map(|chunk| Ok(Some(chunk.into())))
            .collect::<Vec<_>>();
        outcomes.push(Ok(None));
        Self::new(outcomes, None)
    }

    fn start_failure() -> Self {
        Self::new(vec![], Some(AiModelAdapterError::Unavailable))
    }

    fn stream_failure() -> Self {
        Self::new(vec![Err(AiModelAdapterError::StreamInterrupted)], None)
    }

    fn new(
        outcomes: Vec<Result<Option<String>, AiModelAdapterError>>,
        start_error: Option<AiModelAdapterError>,
    ) -> Self {
        Self {
            outcomes,
            start_error,
            start_count: Arc::new(AtomicUsize::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
            captured: Arc::new(Mutex::new(None)),
        }
    }
}

impl AiModelAdapter for FakeAdapter {
    type Stream = FakeStream;

    fn start_stream(&self, request: &AiModelRequest) -> Result<Self::Stream, AiModelAdapterError> {
        self.start_count.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.start_error {
            return Err(error);
        }
        *self.captured.lock().unwrap() = Some(request.clone());
        Ok(FakeStream {
            outcomes: self.outcomes.clone().into(),
            cancelled: Arc::clone(&self.cancelled),
        })
    }
}

struct FakeStream {
    outcomes: VecDeque<Result<Option<String>, AiModelAdapterError>>,
    cancelled: Arc<AtomicBool>,
}

impl AiModelStream for FakeStream {
    fn next_chunk(&mut self) -> AiChunkFuture<'_> {
        Box::pin(ready(self.outcomes.pop_front().unwrap_or(Ok(None))))
    }

    fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

struct CancellingAdapter {
    registry: WorkerCancellationRegistry,
    stream_id: WorkerId,
    cancelled: Arc<AtomicBool>,
}

impl AiModelAdapter for CancellingAdapter {
    type Stream = CancellingStream;

    fn start_stream(&self, _request: &AiModelRequest) -> Result<Self::Stream, AiModelAdapterError> {
        Ok(CancellingStream {
            registry: self.registry.clone(),
            stream_id: self.stream_id,
            cancelled: Arc::clone(&self.cancelled),
        })
    }
}

struct CancellingStream {
    registry: WorkerCancellationRegistry,
    stream_id: WorkerId,
    cancelled: Arc<AtomicBool>,
}

impl AiModelStream for CancellingStream {
    fn next_chunk(&mut self) -> AiChunkFuture<'_> {
        let registry = self.registry.clone();
        let stream_id = self.stream_id;
        Box::pin(async move {
            registry.cancel(stream_id).unwrap();
            pending::<Result<Option<String>, AiModelAdapterError>>().await
        })
    }

    fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
