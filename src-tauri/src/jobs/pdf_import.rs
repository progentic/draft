use std::{error::Error, fmt, path::Path};

use uuid::Uuid;

use crate::imports::pdf::{PdfImportId, PdfImportSource, PendingPdfImport};

/// Maximum diagnostic message size retained with a failed attempt.
pub const MAX_JOB_FAILURE_MESSAGE_BYTES: usize = 512;

/// Durable Rust-generated identity for one PDF import job.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PdfImportJobId(pub(crate) Uuid);

/// Milliseconds from the Unix epoch supplied to deterministic job transitions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct JobTimestamp(u64);

/// Persisted lifecycle states for a PDF import job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportJobState {
    Pending,
    InProgress,
    Resolved,
    Failed,
    NeedsManualInput,
    Cancelled,
}

/// Last durable processing boundary known for a PDF import job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportJobCheckpoint {
    IntakeValidated,
}

/// Closed failure codes for one unsuccessful PDF import attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportJobFailureCode {
    SourceUnavailable,
    SourceChanged,
    ProcessingFailed,
    RetryLimitReached,
}

/// Bounded diagnostic retained with a failed or manual-input transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PdfImportJobFailure {
    code: PdfImportJobFailureCode,
    message: String,
}

/// Validation failures for terminal diagnostic construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportJobFailureError {
    EmptyMessage,
    MessageTooLong,
}

/// Opaque proof that one caller owns the current in-progress attempt.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PdfImportJobClaim {
    pub(crate) job_id: PdfImportJobId,
    pub(crate) token: Uuid,
}

/// Rehydrated durable PDF import job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PdfImportJob {
    pub(crate) job_id: PdfImportJobId,
    pub(crate) candidate_id: PdfImportId,
    pub(crate) source: PdfImportSource,
    pub(crate) source_path: std::path::PathBuf,
    pub(crate) byte_length: u64,
    pub(crate) state: PdfImportJobState,
    pub(crate) attempt_count: u32,
    pub(crate) last_failure: Option<PdfImportJobFailure>,
    pub(crate) checkpoint: PdfImportJobCheckpoint,
    pub(crate) created_at: JobTimestamp,
    pub(crate) updated_at: JobTimestamp,
    pub(crate) cancel_requested: bool,
    pub(crate) claimed_at: Option<JobTimestamp>,
}

impl PdfImportJobId {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for PdfImportJobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl JobTimestamp {
    pub const fn from_unix_millis(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_unix_millis(self) -> u64 {
        self.0
    }
}

impl PdfImportJobState {
    pub(crate) const fn database_value(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Resolved => "resolved",
            Self::Failed => "failed",
            Self::NeedsManualInput => "needs_manual_input",
            Self::Cancelled => "cancelled",
        }
    }

    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "resolved" => Some(Self::Resolved),
            "failed" => Some(Self::Failed),
            "needs_manual_input" => Some(Self::NeedsManualInput),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

impl PdfImportJobCheckpoint {
    pub(crate) const fn database_value(self) -> &'static str {
        match self {
            Self::IntakeValidated => "intake_validated",
        }
    }

    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "intake_validated" => Some(Self::IntakeValidated),
            _ => None,
        }
    }
}

impl PdfImportJobFailureCode {
    pub(crate) const fn database_value(self) -> &'static str {
        match self {
            Self::SourceUnavailable => "source_unavailable",
            Self::SourceChanged => "source_changed",
            Self::ProcessingFailed => "processing_failed",
            Self::RetryLimitReached => "retry_limit_reached",
        }
    }

    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "source_unavailable" => Some(Self::SourceUnavailable),
            "source_changed" => Some(Self::SourceChanged),
            "processing_failed" => Some(Self::ProcessingFailed),
            "retry_limit_reached" => Some(Self::RetryLimitReached),
            _ => None,
        }
    }
}

impl PdfImportJobFailure {
    pub fn new(
        code: PdfImportJobFailureCode,
        message: impl Into<String>,
    ) -> Result<Self, PdfImportJobFailureError> {
        let message = message.into();
        validate_failure_message(&message)?;
        Ok(Self { code, message })
    }

    pub fn code(&self) -> PdfImportJobFailureCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn from_stored(
        code: PdfImportJobFailureCode,
        message: String,
    ) -> Result<Self, PdfImportJobFailureError> {
        Self::new(code, message)
    }
}

impl fmt::Display for PdfImportJobFailureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyMessage => "job failure message is empty",
            Self::MessageTooLong => "job failure message is too long",
        })
    }
}

impl Error for PdfImportJobFailureError {}

impl PdfImportJobClaim {
    pub fn job_id(&self) -> PdfImportJobId {
        self.job_id
    }

    pub(crate) fn token(&self) -> Uuid {
        self.token
    }
}

impl fmt::Debug for PdfImportJobClaim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PdfImportJobClaim")
            .field("job_id", &self.job_id)
            .field("token", &"<redacted>")
            .finish()
    }
}

impl PdfImportJob {
    pub fn job_id(&self) -> PdfImportJobId {
        self.job_id
    }

    pub fn candidate_id(&self) -> PdfImportId {
        self.candidate_id
    }

    pub fn source(&self) -> PdfImportSource {
        self.source
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn byte_length(&self) -> u64 {
        self.byte_length
    }

    pub fn state(&self) -> PdfImportJobState {
        self.state
    }

    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    pub fn last_failure(&self) -> Option<&PdfImportJobFailure> {
        self.last_failure.as_ref()
    }

    pub fn checkpoint(&self) -> PdfImportJobCheckpoint {
        self.checkpoint
    }

    pub fn created_at(&self) -> JobTimestamp {
        self.created_at
    }

    pub fn updated_at(&self) -> JobTimestamp {
        self.updated_at
    }

    pub fn cancel_requested(&self) -> bool {
        self.cancel_requested
    }

    pub fn claimed_at(&self) -> Option<JobTimestamp> {
        self.claimed_at
    }

    pub(crate) fn matches_candidate(&self, candidate: &PendingPdfImport) -> bool {
        self.candidate_id == candidate.import_id()
            && self.source == candidate.source()
            && self.source_path == candidate.path()
            && self.byte_length == candidate.byte_length()
    }
}

fn validate_failure_message(message: &str) -> Result<(), PdfImportJobFailureError> {
    if message.trim().is_empty() {
        return Err(PdfImportJobFailureError::EmptyMessage);
    }
    if message.len() > MAX_JOB_FAILURE_MESSAGE_BYTES {
        return Err(PdfImportJobFailureError::MessageTooLong);
    }
    Ok(())
}
