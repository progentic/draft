use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OptionalExtension, Row, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    imports::pdf::{PdfImportId, PdfImportSource, PendingPdfImport},
    jobs::pdf_import::{
        JobTimestamp, PdfImportJob, PdfImportJobCheckpoint, PdfImportJobClaim, PdfImportJobFailure,
        PdfImportJobFailureCode, PdfImportJobId, PdfImportJobState,
    },
};

const CREATE_SCHEMA_V1: &str = r#"
    CREATE TABLE pdf_import_jobs (
        job_id TEXT PRIMARY KEY NOT NULL,
        record_id TEXT NOT NULL UNIQUE,
        job_kind TEXT NOT NULL CHECK (job_kind = 'pdf_import'),
        state TEXT NOT NULL CHECK (
            state IN ('pending', 'in_progress', 'resolved', 'failed',
                      'needs_manual_input', 'cancelled')
        ),
        attempt_count INTEGER NOT NULL CHECK (
            attempt_count >= 0 AND attempt_count <= 4294967295
        ),
        last_error_code TEXT CHECK (
            last_error_code IS NULL OR last_error_code IN (
                'source_unavailable', 'source_changed', 'processing_failed',
                'retry_limit_reached'
            )
        ),
        last_error_message TEXT,
        last_checkpoint TEXT NOT NULL CHECK (last_checkpoint = 'intake_validated'),
        source_kind TEXT NOT NULL CHECK (source_kind IN ('explicit', 'watched_folder')),
        source_path_encoding TEXT NOT NULL CHECK (
            source_path_encoding IN ('unix_bytes', 'windows_utf16', 'utf8')
        ),
        source_path BLOB NOT NULL,
        byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
        created_at INTEGER NOT NULL CHECK (created_at >= 0),
        updated_at INTEGER NOT NULL CHECK (updated_at >= created_at),
        cancel_requested INTEGER NOT NULL CHECK (cancel_requested IN (0, 1)),
        claim_token_hash BLOB,
        claimed_at INTEGER CHECK (claimed_at IS NULL OR claimed_at >= created_at),
        CHECK ((last_error_code IS NULL) = (last_error_message IS NULL)),
        CHECK (
            (state = 'in_progress' AND claim_token_hash IS NOT NULL AND claimed_at IS NOT NULL)
            OR
            (state <> 'in_progress' AND claim_token_hash IS NULL AND claimed_at IS NULL)
        )
    ) STRICT;
    PRAGMA user_version = 1;
"#;

const SELECT_JOB_COLUMNS: &str = "job_id, record_id, job_kind, state, attempt_count, \
    last_error_code, last_error_message, last_checkpoint, source_kind, \
    source_path_encoding, source_path, byte_length, created_at, updated_at, \
    cancel_requested, claim_token_hash, claimed_at";
const JOB_KIND: &str = "pdf_import";
const JOB_STORE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Current SQLite schema version for durable background jobs.
pub const JOB_STORE_SCHEMA_VERSION: u64 = 1;

/// Stable job database filename joined to the Rust-resolved app-data directory.
pub const JOB_STORE_FILENAME: &str = "jobs.sqlite3";

/// Rust-owned transactional persistence for PDF import jobs.
pub struct PdfImportJobStore {
    connection: Mutex<Connection>,
}

/// Bounded failures produced by PDF import job operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PdfImportJobStoreError {
    StorageLocationUnavailable,
    OpenFailed,
    ClockUnavailable,
    SchemaReadFailed,
    SchemaMigrationFailed,
    UnsupportedStoreSchema {
        found: u64,
    },
    InvalidStoreSchema,
    StoreUnavailable,
    ReadFailed,
    WriteFailed,
    CandidateConflict,
    JobNotFound,
    JobNotClaimable {
        state: PdfImportJobState,
    },
    ClaimOwnershipLost,
    CancellationRequested,
    CancellationNotRequested,
    CheckpointMismatch,
    AttemptCountMismatch,
    AttemptLimitReached,
    InvalidTransition {
        from: PdfImportJobState,
        to: PdfImportJobState,
    },
    TerminalStateImmutable {
        state: PdfImportJobState,
    },
    TimestampOutOfOrder,
    ValueOutOfRange,
    MalformedStoredIdentity,
    InvalidStoredValue,
    InvalidStoredPath,
    InvalidStoredFailure,
}

struct StoredPdfImportJob {
    job_id: String,
    record_id: String,
    job_kind: String,
    state: String,
    attempt_count: i64,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
    last_checkpoint: String,
    source_kind: String,
    source_path_encoding: String,
    source_path: Vec<u8>,
    byte_length: i64,
    created_at: i64,
    updated_at: i64,
    cancel_requested: i64,
    claim_token_hash: Option<Vec<u8>>,
    claimed_at: Option<i64>,
}

/// Returns the production job-store path for a Rust-resolved app-data directory.
pub fn job_store_path(app_data_directory: &Path) -> PathBuf {
    app_data_directory.join(JOB_STORE_FILENAME)
}

impl PdfImportJobStore {
    /// Opens, migrates, validates, and recovers the durable job store.
    pub fn open(path: &Path) -> Result<Self, PdfImportJobStoreError> {
        Self::open_at(path, current_timestamp()?)
    }

    /// Atomically promotes one validated candidate or returns its existing job.
    pub fn promote_candidate(
        &self,
        candidate: &PendingPdfImport,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        if let Some(existing) = load_by_candidate_id(&transaction, candidate.import_id())? {
            return finish_existing_promotion(transaction, existing, candidate);
        }
        let job = pending_job(candidate, now);
        insert_job(&transaction, &job)?;
        commit_write(transaction)?;
        Ok(job)
    }

    /// Loads one durable job by job identity.
    pub fn get(
        &self,
        job_id: PdfImportJobId,
    ) -> Result<Option<PdfImportJob>, PdfImportJobStoreError> {
        let connection = self.lock_connection()?;
        load_by_job_id(&connection, job_id)
    }

    /// Loads one durable job by its Phase 24 candidate identity.
    pub fn get_by_candidate_id(
        &self,
        candidate_id: PdfImportId,
    ) -> Result<Option<PdfImportJob>, PdfImportJobStoreError> {
        let connection = self.lock_connection()?;
        load_by_candidate_id(&connection, candidate_id)
    }

    /// Returns all jobs in deterministic creation and identity order.
    pub fn list(&self) -> Result<Vec<PdfImportJob>, PdfImportJobStoreError> {
        let connection = self.lock_connection()?;
        load_all(&connection)
    }

    /// Claims one pending job with a new opaque ownership token.
    pub fn claim(
        &self,
        job_id: PdfImportJobId,
        now: JobTimestamp,
    ) -> Result<PdfImportJobClaim, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let job = require_job(&transaction, job_id)?;
        require_claimable(&job, now)?;
        let claim = PdfImportJobClaim {
            job_id,
            token: Uuid::new_v4(),
        };
        update_claim(&transaction, &job, claim, now)?;
        commit_write(transaction)?;
        Ok(claim)
    }

    /// Persists a checkpoint only for the current in-progress owner.
    pub fn persist_checkpoint(
        &self,
        claim: PdfImportJobClaim,
        expected: PdfImportJobCheckpoint,
        checkpoint: PdfImportJobCheckpoint,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let job = require_active_claim(&transaction, claim, now, false)?;
        if job.checkpoint() != expected {
            return Err(PdfImportJobStoreError::CheckpointMismatch);
        }
        update_checkpoint(&transaction, claim, expected, checkpoint, now)?;
        finish_job_update(transaction, claim.job_id())
    }

    /// Completes one attempt only for the current in-progress owner.
    pub fn resolve(
        &self,
        claim: PdfImportJobClaim,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        self.finish_claimed_attempt(claim, PdfImportJobState::Resolved, None, now)
    }

    /// Records one typed failed attempt only for the current owner.
    pub fn fail(
        &self,
        claim: PdfImportJobClaim,
        failure: &PdfImportJobFailure,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        self.finish_claimed_attempt(claim, PdfImportJobState::Failed, Some(failure), now)
    }

    /// Records a manual-input stop only for the current owner.
    pub fn require_manual_input(
        &self,
        claim: PdfImportJobClaim,
        failure: &PdfImportJobFailure,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        self.finish_claimed_attempt(
            claim,
            PdfImportJobState::NeedsManualInput,
            Some(failure),
            now,
        )
    }

    /// Explicitly reopens one failed attempt without incrementing its count.
    pub fn retry_failed(
        &self,
        job_id: PdfImportJobId,
        expected_attempt_count: u32,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        self.requeue(
            job_id,
            PdfImportJobState::Failed,
            expected_attempt_count,
            now,
        )
    }

    /// Explicitly reopens one manual-input job without incrementing its count.
    pub fn reopen_manual_input(
        &self,
        job_id: PdfImportJobId,
        expected_attempt_count: u32,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        self.requeue(
            job_id,
            PdfImportJobState::NeedsManualInput,
            expected_attempt_count,
            now,
        )
    }

    /// Durably records cancellation intent without granting mutation ownership.
    pub fn request_cancellation(
        &self,
        job_id: PdfImportJobId,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let job = require_job(&transaction, job_id)?;
        require_monotonic_time(&job, now)?;
        match job.state() {
            PdfImportJobState::Resolved => Err(PdfImportJobStoreError::TerminalStateImmutable {
                state: PdfImportJobState::Resolved,
            }),
            PdfImportJobState::Cancelled => Ok(job),
            PdfImportJobState::InProgress => {
                request_claimed_cancellation(&transaction, &job, now)?;
                finish_job_update(transaction, job_id)
            }
            _ => {
                cancel_unclaimed_job(&transaction, &job, now)?;
                finish_job_update(transaction, job_id)
            }
        }
    }

    /// Acknowledges durable cancellation only for the current owner.
    pub fn acknowledge_cancellation(
        &self,
        claim: PdfImportJobClaim,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let job = require_active_claim(&transaction, claim, now, true)?;
        if !job.cancel_requested() {
            return Err(PdfImportJobStoreError::CancellationNotRequested);
        }
        acknowledge_claimed_cancellation(&transaction, claim, now)?;
        finish_job_update(transaction, claim.job_id())
    }

    fn open_at(path: &Path, now: JobTimestamp) -> Result<Self, PdfImportJobStoreError> {
        ensure_parent_directory(path)?;
        let mut connection = open_connection(path)?;
        configure_connection(&connection)?;
        migrate_store(&mut connection)?;
        validate_all_jobs(&connection)?;
        recover_interrupted_jobs(&mut connection, now)?;
        validate_all_jobs(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn finish_claimed_attempt(
        &self,
        claim: PdfImportJobClaim,
        target: PdfImportJobState,
        failure: Option<&PdfImportJobFailure>,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        require_active_claim(&transaction, claim, now, false)?;
        update_claimed_terminal(&transaction, claim, target, failure, now)?;
        finish_job_update(transaction, claim.job_id())
    }

    fn requeue(
        &self,
        job_id: PdfImportJobId,
        expected_state: PdfImportJobState,
        expected_attempt_count: u32,
        now: JobTimestamp,
    ) -> Result<PdfImportJob, PdfImportJobStoreError> {
        let mut connection = self.lock_connection()?;
        let transaction = begin_write(&mut connection)?;
        let job = require_job(&transaction, job_id)?;
        require_requeue_preconditions(&job, expected_state, expected_attempt_count, now)?;
        update_requeue(&transaction, &job, expected_state, now)?;
        finish_job_update(transaction, job_id)
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, PdfImportJobStoreError> {
        self.connection
            .lock()
            .map_err(|_| PdfImportJobStoreError::StoreUnavailable)
    }
}

impl fmt::Display for PdfImportJobStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for PdfImportJobStoreError {}

impl PdfImportJobStoreError {
    fn message(&self) -> &'static str {
        match self {
            Self::StorageLocationUnavailable => "job storage location is unavailable",
            Self::OpenFailed => "job store could not be opened",
            Self::ClockUnavailable => "job clock is unavailable",
            Self::SchemaReadFailed => "job store schema could not be read",
            Self::SchemaMigrationFailed => "job store schema migration failed",
            Self::UnsupportedStoreSchema { .. } => "job store schema is not supported",
            Self::InvalidStoreSchema => "job store schema is incomplete or invalid",
            Self::StoreUnavailable => "job store state is unavailable",
            Self::ReadFailed => "job store read failed",
            Self::WriteFailed => "job store write failed",
            Self::CandidateConflict => "PDF candidate conflicts with its durable job",
            Self::JobNotFound => "PDF import job was not found",
            Self::JobNotClaimable { .. } => "PDF import job cannot be claimed",
            Self::ClaimOwnershipLost => "PDF import job claim ownership was lost",
            Self::CancellationRequested => "PDF import job cancellation was requested",
            Self::CancellationNotRequested => "PDF import job cancellation was not requested",
            Self::CheckpointMismatch => "PDF import job checkpoint does not match",
            Self::AttemptCountMismatch => "PDF import job attempt count does not match",
            Self::AttemptLimitReached => "PDF import job attempt limit was reached",
            Self::InvalidTransition { .. } => "PDF import job transition is invalid",
            Self::TerminalStateImmutable { .. } => "PDF import job terminal state is immutable",
            Self::TimestampOutOfOrder => "PDF import job timestamp is out of order",
            Self::ValueOutOfRange => "PDF import job value is out of range",
            Self::MalformedStoredIdentity => "stored PDF import job identity is malformed",
            Self::InvalidStoredValue => "stored PDF import job value is invalid",
            Self::InvalidStoredPath => "stored PDF import source path is invalid",
            Self::InvalidStoredFailure => "stored PDF import job failure is invalid",
        }
    }
}

fn current_timestamp() -> Result<JobTimestamp, PdfImportJobStoreError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PdfImportJobStoreError::ClockUnavailable)?;
    let millis =
        u64::try_from(elapsed.as_millis()).map_err(|_| PdfImportJobStoreError::ClockUnavailable)?;
    Ok(JobTimestamp::from_unix_millis(millis))
}

fn ensure_parent_directory(path: &Path) -> Result<(), PdfImportJobStoreError> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|_| PdfImportJobStoreError::StorageLocationUnavailable)
}

fn open_connection(path: &Path) -> Result<Connection, PdfImportJobStoreError> {
    Connection::open(path).map_err(|_| PdfImportJobStoreError::OpenFailed)
}

fn configure_connection(connection: &Connection) -> Result<(), PdfImportJobStoreError> {
    connection
        .busy_timeout(JOB_STORE_BUSY_TIMEOUT)
        .map_err(|_| PdfImportJobStoreError::OpenFailed)
}

fn migrate_store(connection: &mut Connection) -> Result<(), PdfImportJobStoreError> {
    match read_store_schema(connection)? {
        0 => migrate_zero_to_one(connection),
        JOB_STORE_SCHEMA_VERSION => Ok(()),
        found => Err(PdfImportJobStoreError::UnsupportedStoreSchema { found }),
    }?;
    verify_current_schema(connection)
}

fn read_store_schema(connection: &Connection) -> Result<u64, PdfImportJobStoreError> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| PdfImportJobStoreError::SchemaReadFailed)?;
    u64::try_from(version).map_err(|_| PdfImportJobStoreError::SchemaReadFailed)
}

fn migrate_zero_to_one(connection: &mut Connection) -> Result<(), PdfImportJobStoreError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| PdfImportJobStoreError::SchemaMigrationFailed)?;
    transaction
        .execute_batch(CREATE_SCHEMA_V1)
        .map_err(|_| PdfImportJobStoreError::SchemaMigrationFailed)?;
    transaction
        .commit()
        .map_err(|_| PdfImportJobStoreError::SchemaMigrationFailed)
}

fn verify_current_schema(connection: &Connection) -> Result<(), PdfImportJobStoreError> {
    verify_schema_definition(connection)?;
    connection
        .prepare(&format!(
            "SELECT {SELECT_JOB_COLUMNS} FROM pdf_import_jobs LIMIT 0"
        ))
        .map(|_| ())
        .map_err(|_| PdfImportJobStoreError::InvalidStoreSchema)
}

fn verify_schema_definition(connection: &Connection) -> Result<(), PdfImportJobStoreError> {
    let definition = connection
        .query_row(
            "SELECT sql FROM sqlite_master
             WHERE type = 'table' AND name = 'pdf_import_jobs'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| PdfImportJobStoreError::InvalidStoreSchema)?
        .ok_or(PdfImportJobStoreError::InvalidStoreSchema)?;
    let required_fragments = [
        "job_id TEXT PRIMARY KEY NOT NULL",
        "record_id TEXT NOT NULL UNIQUE",
        "job_kind TEXT NOT NULL CHECK (job_kind = 'pdf_import')",
        "attempt_count INTEGER NOT NULL CHECK",
        "last_checkpoint TEXT NOT NULL CHECK (last_checkpoint = 'intake_validated')",
        "source_path BLOB NOT NULL",
        "cancel_requested INTEGER NOT NULL CHECK",
        "claim_token_hash BLOB",
        ") STRICT",
    ];
    if required_fragments
        .iter()
        .all(|fragment| definition.contains(fragment))
    {
        Ok(())
    } else {
        Err(PdfImportJobStoreError::InvalidStoreSchema)
    }
}

fn recover_interrupted_jobs(
    connection: &mut Connection,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let now = timestamp_to_database(now)?;
    let transaction = begin_write(connection)?;
    transaction
        .execute(
            "UPDATE pdf_import_jobs
             SET state = CASE cancel_requested WHEN 1 THEN 'cancelled' ELSE 'pending' END,
                 claim_token_hash = NULL,
                 claimed_at = NULL,
                 updated_at = MAX(updated_at, ?1)
             WHERE state = 'in_progress'",
            [now],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    commit_write(transaction)
}

fn validate_all_jobs(connection: &Connection) -> Result<(), PdfImportJobStoreError> {
    load_all(connection).map(|_| ())
}

fn begin_write(connection: &mut Connection) -> Result<Transaction<'_>, PdfImportJobStoreError> {
    connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| PdfImportJobStoreError::WriteFailed)
}

fn commit_write(transaction: Transaction<'_>) -> Result<(), PdfImportJobStoreError> {
    transaction
        .commit()
        .map_err(|_| PdfImportJobStoreError::WriteFailed)
}

fn finish_existing_promotion(
    transaction: Transaction<'_>,
    existing: PdfImportJob,
    candidate: &PendingPdfImport,
) -> Result<PdfImportJob, PdfImportJobStoreError> {
    if !existing.matches_candidate(candidate) {
        return Err(PdfImportJobStoreError::CandidateConflict);
    }
    commit_write(transaction)?;
    Ok(existing)
}

fn pending_job(candidate: &PendingPdfImport, now: JobTimestamp) -> PdfImportJob {
    PdfImportJob {
        job_id: PdfImportJobId(Uuid::new_v4()),
        candidate_id: candidate.import_id(),
        source: candidate.source(),
        source_path: candidate.path().to_owned(),
        byte_length: candidate.byte_length(),
        state: PdfImportJobState::Pending,
        attempt_count: 0,
        last_failure: None,
        checkpoint: PdfImportJobCheckpoint::IntakeValidated,
        created_at: now,
        updated_at: now,
        cancel_requested: false,
        claimed_at: None,
    }
}

fn insert_job(connection: &Connection, job: &PdfImportJob) -> Result<(), PdfImportJobStoreError> {
    let (path_encoding, path_bytes) = encode_source_path(job.source_path());
    connection
        .execute(
            "INSERT INTO pdf_import_jobs (
                job_id, record_id, job_kind, state, attempt_count,
                last_error_code, last_error_message, last_checkpoint, source_kind,
                source_path_encoding, source_path, byte_length, created_at, updated_at,
                cancel_requested, claim_token_hash, claimed_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, NULL, NULL, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                0, NULL, NULL
             )",
            params![
                job.job_id().to_string(),
                job.candidate_id().as_uuid().to_string(),
                JOB_KIND,
                job.state().database_value(),
                i64::from(job.attempt_count()),
                job.checkpoint().database_value(),
                source_database_value(job.source()),
                path_encoding,
                path_bytes,
                u64_to_database(job.byte_length())?,
                timestamp_to_database(job.created_at())?,
                timestamp_to_database(job.updated_at())?,
            ],
        )
        .map(|_| ())
        .map_err(|_| PdfImportJobStoreError::WriteFailed)
}

fn require_job(
    connection: &Connection,
    job_id: PdfImportJobId,
) -> Result<PdfImportJob, PdfImportJobStoreError> {
    load_by_job_id(connection, job_id)?.ok_or(PdfImportJobStoreError::JobNotFound)
}

fn require_claimable(job: &PdfImportJob, now: JobTimestamp) -> Result<(), PdfImportJobStoreError> {
    require_monotonic_time(job, now)?;
    if job.state() != PdfImportJobState::Pending || job.cancel_requested() {
        return Err(PdfImportJobStoreError::JobNotClaimable { state: job.state() });
    }
    if job.attempt_count() == u32::MAX {
        return Err(PdfImportJobStoreError::AttemptLimitReached);
    }
    Ok(())
}

fn update_claim(
    connection: &Connection,
    job: &PdfImportJob,
    claim: PdfImportJobClaim,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs
             SET state = 'in_progress', attempt_count = attempt_count + 1,
                 claim_token_hash = ?2, claimed_at = ?3, updated_at = ?3
             WHERE job_id = ?1 AND state = 'pending' AND attempt_count = ?4
               AND cancel_requested = 0 AND claim_token_hash IS NULL",
            params![
                job.job_id().to_string(),
                claim_token_hash(claim),
                timestamp_to_database(now)?,
                i64::from(job.attempt_count()),
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn require_active_claim(
    connection: &Connection,
    claim: PdfImportJobClaim,
    now: JobTimestamp,
    allow_cancellation: bool,
) -> Result<PdfImportJob, PdfImportJobStoreError> {
    let job = require_job(connection, claim.job_id())?;
    require_monotonic_time(&job, now)?;
    if job.state() != PdfImportJobState::InProgress || !claim_is_current(connection, claim)? {
        return Err(PdfImportJobStoreError::ClaimOwnershipLost);
    }
    if job.cancel_requested() && !allow_cancellation {
        return Err(PdfImportJobStoreError::CancellationRequested);
    }
    Ok(job)
}

fn update_checkpoint(
    connection: &Connection,
    claim: PdfImportJobClaim,
    expected: PdfImportJobCheckpoint,
    checkpoint: PdfImportJobCheckpoint,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs SET last_checkpoint = ?4, updated_at = ?5
             WHERE job_id = ?1 AND state = 'in_progress' AND claim_token_hash = ?2
               AND cancel_requested = 0 AND last_checkpoint = ?3",
            params![
                claim.job_id().to_string(),
                claim_token_hash(claim),
                expected.database_value(),
                checkpoint.database_value(),
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn update_claimed_terminal(
    connection: &Connection,
    claim: PdfImportJobClaim,
    target: PdfImportJobState,
    failure: Option<&PdfImportJobFailure>,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let failure_code = failure.map(|value| value.code().database_value());
    let failure_message = failure.map(PdfImportJobFailure::message);
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs
             SET state = ?3,
                 last_error_code = CASE WHEN ?4 IS NULL THEN last_error_code ELSE ?4 END,
                 last_error_message = CASE WHEN ?5 IS NULL THEN last_error_message ELSE ?5 END,
                 claim_token_hash = NULL, claimed_at = NULL, updated_at = ?6
             WHERE job_id = ?1 AND state = 'in_progress' AND claim_token_hash = ?2
               AND cancel_requested = 0",
            params![
                claim.job_id().to_string(),
                claim_token_hash(claim),
                target.database_value(),
                failure_code,
                failure_message,
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn require_requeue_preconditions(
    job: &PdfImportJob,
    expected_state: PdfImportJobState,
    expected_attempt_count: u32,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    require_monotonic_time(job, now)?;
    if job.state() != expected_state {
        return Err(PdfImportJobStoreError::InvalidTransition {
            from: job.state(),
            to: PdfImportJobState::Pending,
        });
    }
    if job.attempt_count() != expected_attempt_count {
        return Err(PdfImportJobStoreError::AttemptCountMismatch);
    }
    Ok(())
}

fn update_requeue(
    connection: &Connection,
    job: &PdfImportJob,
    expected_state: PdfImportJobState,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs
             SET state = 'pending', updated_at = ?4
             WHERE job_id = ?1 AND state = ?2 AND attempt_count = ?3
               AND cancel_requested = 0 AND claim_token_hash IS NULL",
            params![
                job.job_id().to_string(),
                expected_state.database_value(),
                i64::from(job.attempt_count()),
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    if affected == 1 {
        Ok(())
    } else {
        Err(PdfImportJobStoreError::InvalidTransition {
            from: job.state(),
            to: PdfImportJobState::Pending,
        })
    }
}

fn request_claimed_cancellation(
    connection: &Connection,
    job: &PdfImportJob,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs SET cancel_requested = 1, updated_at = ?3
             WHERE job_id = ?1 AND state = 'in_progress' AND attempt_count = ?2",
            params![
                job.job_id().to_string(),
                i64::from(job.attempt_count()),
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn cancel_unclaimed_job(
    connection: &Connection,
    job: &PdfImportJob,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs
             SET state = 'cancelled', cancel_requested = 1, updated_at = ?4
             WHERE job_id = ?1 AND state = ?2 AND attempt_count = ?3
               AND claim_token_hash IS NULL",
            params![
                job.job_id().to_string(),
                job.state().database_value(),
                i64::from(job.attempt_count()),
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn acknowledge_claimed_cancellation(
    connection: &Connection,
    claim: PdfImportJobClaim,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    let affected = connection
        .execute(
            "UPDATE pdf_import_jobs
             SET state = 'cancelled', claim_token_hash = NULL, claimed_at = NULL,
                 updated_at = ?3
             WHERE job_id = ?1 AND state = 'in_progress' AND claim_token_hash = ?2
               AND cancel_requested = 1",
            params![
                claim.job_id().to_string(),
                claim_token_hash(claim),
                timestamp_to_database(now)?,
            ],
        )
        .map_err(|_| PdfImportJobStoreError::WriteFailed)?;
    require_one_owned_row(affected)
}

fn finish_job_update(
    transaction: Transaction<'_>,
    job_id: PdfImportJobId,
) -> Result<PdfImportJob, PdfImportJobStoreError> {
    let job = require_job(&transaction, job_id)?;
    commit_write(transaction)?;
    Ok(job)
}

fn require_one_owned_row(affected: usize) -> Result<(), PdfImportJobStoreError> {
    if affected == 1 {
        Ok(())
    } else {
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    }
}

fn require_monotonic_time(
    job: &PdfImportJob,
    now: JobTimestamp,
) -> Result<(), PdfImportJobStoreError> {
    if now >= job.updated_at() {
        Ok(())
    } else {
        Err(PdfImportJobStoreError::TimestampOutOfOrder)
    }
}

fn load_by_job_id(
    connection: &Connection,
    job_id: PdfImportJobId,
) -> Result<Option<PdfImportJob>, PdfImportJobStoreError> {
    connection
        .query_row(
            &format!("SELECT {SELECT_JOB_COLUMNS} FROM pdf_import_jobs WHERE job_id = ?1"),
            [job_id.to_string()],
            map_stored_job,
        )
        .optional()
        .map_err(|_| PdfImportJobStoreError::ReadFailed)?
        .map(decode_stored_job)
        .transpose()
}

fn load_by_candidate_id(
    connection: &Connection,
    candidate_id: PdfImportId,
) -> Result<Option<PdfImportJob>, PdfImportJobStoreError> {
    connection
        .query_row(
            &format!("SELECT {SELECT_JOB_COLUMNS} FROM pdf_import_jobs WHERE record_id = ?1"),
            [candidate_id.as_uuid().to_string()],
            map_stored_job,
        )
        .optional()
        .map_err(|_| PdfImportJobStoreError::ReadFailed)?
        .map(decode_stored_job)
        .transpose()
}

fn load_all(connection: &Connection) -> Result<Vec<PdfImportJob>, PdfImportJobStoreError> {
    let mut statement = connection
        .prepare(&format!(
            "SELECT {SELECT_JOB_COLUMNS} FROM pdf_import_jobs ORDER BY created_at, job_id"
        ))
        .map_err(|_| PdfImportJobStoreError::ReadFailed)?;
    let rows = statement
        .query_map([], map_stored_job)
        .map_err(|_| PdfImportJobStoreError::ReadFailed)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| PdfImportJobStoreError::ReadFailed)?
        .into_iter()
        .map(decode_stored_job)
        .collect()
}

fn map_stored_job(row: &Row<'_>) -> rusqlite::Result<StoredPdfImportJob> {
    Ok(StoredPdfImportJob {
        job_id: row.get(0)?,
        record_id: row.get(1)?,
        job_kind: row.get(2)?,
        state: row.get(3)?,
        attempt_count: row.get(4)?,
        last_error_code: row.get(5)?,
        last_error_message: row.get(6)?,
        last_checkpoint: row.get(7)?,
        source_kind: row.get(8)?,
        source_path_encoding: row.get(9)?,
        source_path: row.get(10)?,
        byte_length: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        cancel_requested: row.get(14)?,
        claim_token_hash: row.get(15)?,
        claimed_at: row.get(16)?,
    })
}

fn decode_stored_job(stored: StoredPdfImportJob) -> Result<PdfImportJob, PdfImportJobStoreError> {
    if stored.job_kind != JOB_KIND {
        return Err(PdfImportJobStoreError::InvalidStoredValue);
    }
    let state = PdfImportJobState::from_database(&stored.state)
        .ok_or(PdfImportJobStoreError::InvalidStoredValue)?;
    let claimed_at = stored.claimed_at.map(database_timestamp).transpose()?;
    validate_claim_shape(state, stored.claim_token_hash.as_deref(), claimed_at)?;
    Ok(PdfImportJob {
        job_id: PdfImportJobId(parse_uuid(&stored.job_id)?),
        candidate_id: PdfImportId::from_uuid(parse_uuid(&stored.record_id)?),
        source: source_from_database(&stored.source_kind)?,
        source_path: decode_source_path(&stored.source_path_encoding, &stored.source_path)?,
        byte_length: database_u64(stored.byte_length)?,
        state,
        attempt_count: u32::try_from(stored.attempt_count)
            .map_err(|_| PdfImportJobStoreError::InvalidStoredValue)?,
        last_failure: decode_stored_failure(
            stored.last_error_code.as_deref(),
            stored.last_error_message,
        )?,
        checkpoint: PdfImportJobCheckpoint::from_database(&stored.last_checkpoint)
            .ok_or(PdfImportJobStoreError::InvalidStoredValue)?,
        created_at: database_timestamp(stored.created_at)?,
        updated_at: database_timestamp(stored.updated_at)?,
        cancel_requested: database_bool(stored.cancel_requested)?,
        claimed_at,
    })
}

fn parse_uuid(value: &str) -> Result<Uuid, PdfImportJobStoreError> {
    Uuid::parse_str(value).map_err(|_| PdfImportJobStoreError::MalformedStoredIdentity)
}

fn validate_claim_shape(
    state: PdfImportJobState,
    token_hash: Option<&[u8]>,
    claimed_at: Option<JobTimestamp>,
) -> Result<(), PdfImportJobStoreError> {
    match (state, token_hash, claimed_at) {
        (PdfImportJobState::InProgress, Some(value), Some(_)) if value.len() == 32 => Ok(()),
        (PdfImportJobState::InProgress, _, _) => Err(PdfImportJobStoreError::InvalidStoredValue),
        (_, None, None) => Ok(()),
        _ => Err(PdfImportJobStoreError::InvalidStoredValue),
    }
}

fn claim_is_current(
    connection: &Connection,
    claim: PdfImportJobClaim,
) -> Result<bool, PdfImportJobStoreError> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM pdf_import_jobs
                WHERE job_id = ?1 AND state = 'in_progress' AND claim_token_hash = ?2
             )",
            params![claim.job_id().to_string(), claim_token_hash(claim)],
            |row| row.get(0),
        )
        .map_err(|_| PdfImportJobStoreError::ReadFailed)
}

fn claim_token_hash(claim: PdfImportJobClaim) -> Vec<u8> {
    Sha256::digest(claim.token().as_bytes()).to_vec()
}

fn decode_stored_failure(
    code: Option<&str>,
    message: Option<String>,
) -> Result<Option<PdfImportJobFailure>, PdfImportJobStoreError> {
    match (code, message) {
        (None, None) => Ok(None),
        (Some(code), Some(message)) => {
            let code = PdfImportJobFailureCode::from_database(code)
                .ok_or(PdfImportJobStoreError::InvalidStoredFailure)?;
            PdfImportJobFailure::from_stored(code, message)
                .map(Some)
                .map_err(|_| PdfImportJobStoreError::InvalidStoredFailure)
        }
        _ => Err(PdfImportJobStoreError::InvalidStoredFailure),
    }
}

fn source_database_value(source: PdfImportSource) -> &'static str {
    match source {
        PdfImportSource::Explicit => "explicit",
        PdfImportSource::WatchedFolder => "watched_folder",
    }
}

fn source_from_database(value: &str) -> Result<PdfImportSource, PdfImportJobStoreError> {
    match value {
        "explicit" => Ok(PdfImportSource::Explicit),
        "watched_folder" => Ok(PdfImportSource::WatchedFolder),
        _ => Err(PdfImportJobStoreError::InvalidStoredValue),
    }
}

fn timestamp_to_database(value: JobTimestamp) -> Result<i64, PdfImportJobStoreError> {
    u64_to_database(value.as_unix_millis())
}

fn u64_to_database(value: u64) -> Result<i64, PdfImportJobStoreError> {
    i64::try_from(value).map_err(|_| PdfImportJobStoreError::ValueOutOfRange)
}

fn database_timestamp(value: i64) -> Result<JobTimestamp, PdfImportJobStoreError> {
    database_u64(value).map(JobTimestamp::from_unix_millis)
}

fn database_u64(value: i64) -> Result<u64, PdfImportJobStoreError> {
    u64::try_from(value).map_err(|_| PdfImportJobStoreError::InvalidStoredValue)
}

fn database_bool(value: i64) -> Result<bool, PdfImportJobStoreError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PdfImportJobStoreError::InvalidStoredValue),
    }
}

#[cfg(unix)]
fn encode_source_path(path: &Path) -> (&'static str, Vec<u8>) {
    use std::os::unix::ffi::OsStrExt;
    ("unix_bytes", path.as_os_str().as_bytes().to_vec())
}

#[cfg(unix)]
fn decode_source_path(encoding: &str, bytes: &[u8]) -> Result<PathBuf, PdfImportJobStoreError> {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    if encoding != "unix_bytes" {
        return Err(PdfImportJobStoreError::InvalidStoredPath);
    }
    Ok(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(windows)]
fn encode_source_path(path: &Path) -> (&'static str, Vec<u8>) {
    use std::os::windows::ffi::OsStrExt;
    let bytes = path
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect();
    ("windows_utf16", bytes)
}

#[cfg(windows)]
fn decode_source_path(encoding: &str, bytes: &[u8]) -> Result<PathBuf, PdfImportJobStoreError> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    if encoding != "windows_utf16" || !bytes.len().is_multiple_of(2) {
        return Err(PdfImportJobStoreError::InvalidStoredPath);
    }
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    Ok(PathBuf::from(OsString::from_wide(&words)))
}

#[cfg(not(any(unix, windows)))]
fn encode_source_path(path: &Path) -> (&'static str, Vec<u8>) {
    ("utf8", path.to_string_lossy().as_bytes().to_vec())
}

#[cfg(not(any(unix, windows)))]
fn decode_source_path(encoding: &str, bytes: &[u8]) -> Result<PathBuf, PdfImportJobStoreError> {
    if encoding != "utf8" {
        return Err(PdfImportJobStoreError::InvalidStoredPath);
    }
    String::from_utf8(bytes.to_vec())
        .map(PathBuf::from)
        .map_err(|_| PdfImportJobStoreError::InvalidStoredPath)
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
