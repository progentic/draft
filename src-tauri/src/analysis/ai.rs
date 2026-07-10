use std::{error::Error, fmt, future::Future, pin::Pin};

use crate::{
    analysis::context::{AiAnalysisRequest, AiModelRequest, assemble_model_request},
    events::ai_stream::{
        AiOutputClassification, AiStreamContextSummary, AiStreamEvent, AiStreamEventSink,
        AiStreamFailureCode,
    },
    workers::cancellation::{
        WorkerCancellation, WorkerCancellationError, WorkerCancellationRegistry, WorkerId,
        WorkerRegistration,
    },
};

/// Maximum size accepted for one streamed model chunk.
pub const MAX_AI_STREAM_CHUNK_BYTES: usize = 16 * 1024;

/// Maximum chunks accepted for one generated-analysis stream.
pub const MAX_AI_STREAM_CHUNKS: u32 = 4_096;

/// Maximum cumulative model output retained across one stream.
pub const MAX_AI_STREAM_BYTES: usize = 1024 * 1024;

/// Cancel-safe future returned by one model stream read.
pub type AiChunkFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<String>, AiModelAdapterError>> + Send + 'a>>;

/// Provider-independent Rust model-call boundary.
pub trait AiModelAdapter {
    type Stream: AiModelStream;

    fn start_stream(&self, request: &AiModelRequest) -> Result<Self::Stream, AiModelAdapterError>;
}

/// One cancel-safe provider stream owned by the Rust orchestration loop.
pub trait AiModelStream {
    fn next_chunk(&mut self) -> AiChunkFuture<'_>;
    fn cancel(&mut self);
}

/// Bounded provider-adapter failures with no raw response or transport detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiModelAdapterError {
    Unavailable,
    RequestRejected,
    StreamInterrupted,
    InvalidResponse,
}

/// Synchronously prepared generation that owns its worker registration.
pub struct PreparedAiGeneration {
    model_request: AiModelRequest,
    registration: WorkerRegistration,
}

/// Failures before an adapter call can begin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiPreparationError {
    CancellationRegistryUnavailable,
}

/// Failures that prevent the required event lifecycle from completing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiRunError {
    EventDeliveryFailed,
}

/// Terminal outcome after a successfully delivered terminal event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiGenerationOutcome {
    Completed {
        chunk_count: u32,
        total_bytes: usize,
    },
    Cancelled,
    Failed {
        code: AiStreamFailureCode,
    },
}

struct StreamProgress {
    chunk_count: u32,
    total_bytes: usize,
}

/// Validates context and reserves one cancellable stream identity before work.
pub fn prepare_ai_generation(
    registry: &WorkerCancellationRegistry,
    request: &AiAnalysisRequest,
) -> Result<PreparedAiGeneration, AiPreparationError> {
    let model_request = assemble_model_request(request);
    let registration = registry
        .register()
        .map_err(map_cancellation_registry_error)?;
    Ok(PreparedAiGeneration {
        model_request,
        registration,
    })
}

/// Runs one bounded model stream without spawning or persisting work.
pub async fn run_ai_generation<A, S>(
    prepared: PreparedAiGeneration,
    adapter: &A,
    sink: &mut S,
) -> Result<AiGenerationOutcome, AiRunError>
where
    A: AiModelAdapter,
    S: AiStreamEventSink,
{
    let stream_id = prepared.registration.worker_id();
    let cancellation = prepared.registration.cancellation();
    emit_started(sink, stream_id, &prepared.model_request)?;
    if cancellation.is_cancelled() {
        return finish_cancelled(sink, stream_id);
    }
    let mut stream = match adapter.start_stream(&prepared.model_request) {
        Ok(stream) => stream,
        Err(_) => return finish_failed(sink, stream_id, AiStreamFailureCode::AdapterStartFailed),
    };
    run_model_stream(&mut stream, cancellation, sink, stream_id).await
}

impl PreparedAiGeneration {
    pub fn stream_id(&self) -> WorkerId {
        self.registration.worker_id()
    }

    pub fn model_request(&self) -> &AiModelRequest {
        &self.model_request
    }
}

impl fmt::Display for AiModelAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unavailable => "model adapter is unavailable",
            Self::RequestRejected => "model adapter rejected the request",
            Self::StreamInterrupted => "model stream was interrupted",
            Self::InvalidResponse => "model adapter returned an invalid response",
        })
    }
}

impl Error for AiModelAdapterError {}

impl fmt::Display for AiPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AI cancellation registry is unavailable")
    }
}

impl Error for AiPreparationError {}

impl fmt::Display for AiRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AI stream event delivery failed")
    }
}

impl Error for AiRunError {}

async fn run_model_stream<M, S>(
    stream: &mut M,
    cancellation: WorkerCancellation,
    sink: &mut S,
    stream_id: WorkerId,
) -> Result<AiGenerationOutcome, AiRunError>
where
    M: AiModelStream,
    S: AiStreamEventSink,
{
    let mut progress = StreamProgress::new();
    loop {
        let result = cancellation.run_until_cancelled(stream.next_chunk()).await;
        match result {
            None => {
                stream.cancel();
                return finish_cancelled(sink, stream_id);
            }
            Some(Err(_)) => {
                stream.cancel();
                return finish_failed(sink, stream_id, AiStreamFailureCode::AdapterStreamFailed);
            }
            Some(Ok(None)) => return finish_completed(sink, stream_id, progress),
            Some(Ok(Some(chunk))) => {
                if progress.accept(&chunk).is_err() {
                    stream.cancel();
                    return finish_failed(sink, stream_id, AiStreamFailureCode::InvalidOutput);
                }
                if emit_chunk(sink, stream_id, &progress, chunk).is_err() {
                    stream.cancel();
                    return Err(AiRunError::EventDeliveryFailed);
                }
            }
        }
    }
}

impl StreamProgress {
    fn new() -> Self {
        Self {
            chunk_count: 0,
            total_bytes: 0,
        }
    }

    fn accept(&mut self, chunk: &str) -> Result<(), ()> {
        if chunk.trim().is_empty() || chunk.len() > MAX_AI_STREAM_CHUNK_BYTES {
            return Err(());
        }
        let next_count = self.chunk_count.checked_add(1).ok_or(())?;
        let next_bytes = self.total_bytes.checked_add(chunk.len()).ok_or(())?;
        if next_count > MAX_AI_STREAM_CHUNKS || next_bytes > MAX_AI_STREAM_BYTES {
            return Err(());
        }
        self.chunk_count = next_count;
        self.total_bytes = next_bytes;
        Ok(())
    }

    fn sequence(&self) -> u32 {
        self.chunk_count - 1
    }
}

fn emit_started<S: AiStreamEventSink>(
    sink: &mut S,
    stream_id: WorkerId,
    request: &AiModelRequest,
) -> Result<(), AiRunError> {
    emit(
        sink,
        AiStreamEvent::Started {
            stream_id: stream_id.to_string(),
            classification: AiOutputClassification::GeneratedAnalysis,
            context: context_summary(request),
        },
    )
}

fn emit_chunk<S: AiStreamEventSink>(
    sink: &mut S,
    stream_id: WorkerId,
    progress: &StreamProgress,
    text: String,
) -> Result<(), AiRunError> {
    emit(
        sink,
        AiStreamEvent::Chunk {
            stream_id: stream_id.to_string(),
            sequence: progress.sequence(),
            classification: AiOutputClassification::GeneratedAnalysis,
            text,
        },
    )
}

fn finish_completed<S: AiStreamEventSink>(
    sink: &mut S,
    stream_id: WorkerId,
    progress: StreamProgress,
) -> Result<AiGenerationOutcome, AiRunError> {
    emit(
        sink,
        AiStreamEvent::Completed {
            stream_id: stream_id.to_string(),
            classification: AiOutputClassification::GeneratedAnalysis,
            chunk_count: progress.chunk_count,
            total_bytes: progress.total_bytes,
        },
    )?;
    Ok(AiGenerationOutcome::Completed {
        chunk_count: progress.chunk_count,
        total_bytes: progress.total_bytes,
    })
}

fn finish_cancelled<S: AiStreamEventSink>(
    sink: &mut S,
    stream_id: WorkerId,
) -> Result<AiGenerationOutcome, AiRunError> {
    emit(
        sink,
        AiStreamEvent::Cancelled {
            stream_id: stream_id.to_string(),
            classification: AiOutputClassification::GeneratedAnalysis,
        },
    )?;
    Ok(AiGenerationOutcome::Cancelled)
}

fn finish_failed<S: AiStreamEventSink>(
    sink: &mut S,
    stream_id: WorkerId,
    code: AiStreamFailureCode,
) -> Result<AiGenerationOutcome, AiRunError> {
    emit(
        sink,
        AiStreamEvent::Failed {
            stream_id: stream_id.to_string(),
            classification: AiOutputClassification::GeneratedAnalysis,
            code,
        },
    )?;
    Ok(AiGenerationOutcome::Failed { code })
}

fn emit<S: AiStreamEventSink>(sink: &mut S, event: AiStreamEvent) -> Result<(), AiRunError> {
    sink.emit_event(event)
        .map_err(|_| AiRunError::EventDeliveryFailed)
}

fn context_summary(request: &AiModelRequest) -> AiStreamContextSummary {
    AiStreamContextSummary {
        retained_document_count: request.retained_document_count(),
        omitted_document_count: request.omitted_document_count(),
        retained_evidence_count: request.retained_evidence_count(),
        omitted_evidence_count: request.omitted_evidence_count(),
        verified_evidence_ids: request.verified_evidence_ids().to_vec(),
    }
}

fn map_cancellation_registry_error(_: WorkerCancellationError) -> AiPreparationError {
    AiPreparationError::CancellationRegistryUnavailable
}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod tests;
