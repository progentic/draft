use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    thread,
};

use rusqlite::Connection;

use super::*;
use crate::{
    imports::pdf::prepare_explicit_pdf,
    jobs::pdf_import::{MAX_JOB_FAILURE_MESSAGE_BYTES, PdfImportJobFailureError},
};

const COMPLETE_PDF: &[u8] = b"%PDF-1.7\nbody\n%%EOF";

#[test]
fn job_store_path_uses_app_data_directory() {
    let app_data = Path::new("app-data");
    assert_eq!(job_store_path(app_data), app_data.join(JOB_STORE_FILENAME));
}

#[test]
fn new_store_initializes_and_reopens_schema() {
    let target = TestJobStorePath::new("initialize");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    assert_eq!(read_schema(&store), JOB_STORE_SCHEMA_VERSION);
    assert!(job_table_exists(&store));
    drop(store);

    let reopened = PdfImportJobStore::open_at(target.path(), timestamp(2)).unwrap();
    assert_eq!(read_schema(&reopened), JOB_STORE_SCHEMA_VERSION);
}

#[test]
fn unsupported_or_incomplete_schema_fails_closed() {
    let unsupported = TestJobStorePath::new("unsupported");
    let connection = Connection::open(unsupported.path()).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    assert_eq!(
        PdfImportJobStore::open_at(unsupported.path(), timestamp(1)).err(),
        Some(PdfImportJobStoreError::UnsupportedStoreSchema { found: 2 })
    );

    let incomplete = TestJobStorePath::new("incomplete");
    let connection = Connection::open(incomplete.path()).unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    drop(connection);
    assert_eq!(
        PdfImportJobStore::open_at(incomplete.path(), timestamp(1)).err(),
        Some(PdfImportJobStoreError::InvalidStoreSchema)
    );
}

#[test]
fn candidate_promotion_persists_pending_job() {
    let target = TestJobStorePath::new("promotion");
    let candidate_file = TestPdf::new("promotion");
    let candidate = candidate_file.candidate();
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();

    let job = store.promote_candidate(&candidate, timestamp(10)).unwrap();

    assert_eq!(job.candidate_id(), candidate.import_id());
    assert_eq!(job.source(), candidate.source());
    assert_eq!(job.source_path(), candidate.path());
    assert_eq!(job.byte_length(), candidate.byte_length());
    assert_eq!(job.state(), PdfImportJobState::Pending);
    assert_eq!(job.attempt_count(), 0);
    assert_eq!(job.checkpoint(), PdfImportJobCheckpoint::IntakeValidated);
    assert_eq!(job.created_at(), timestamp(10));
    assert_eq!(job.updated_at(), timestamp(10));
    assert_eq!(store.get(job.job_id()).unwrap(), Some(job));
}

#[test]
fn repeated_promotion_returns_existing_job_without_reset() {
    let target = TestJobStorePath::new("idempotent-promotion");
    let candidate_file = TestPdf::new("idempotent-promotion");
    let candidate = candidate_file.candidate();
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let first = store.promote_candidate(&candidate, timestamp(10)).unwrap();
    let claim = store.claim(first.job_id(), timestamp(11)).unwrap();
    let failed = store.fail(claim, &failure(), timestamp(12)).unwrap();

    let repeated = store.promote_candidate(&candidate, timestamp(20)).unwrap();

    assert_eq!(repeated, failed);
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn separately_validated_candidates_are_not_path_deduplicated() {
    let target = TestJobStorePath::new("separate-candidates");
    let candidate_file = TestPdf::new("separate-candidates");
    let first = candidate_file.candidate();
    let second = candidate_file.candidate();
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();

    let first_job = store.promote_candidate(&first, timestamp(10)).unwrap();
    let second_job = store.promote_candidate(&second, timestamp(11)).unwrap();

    assert_ne!(first.import_id(), second.import_id());
    assert_ne!(first_job.job_id(), second_job.job_id());
    assert_eq!(store.list().unwrap().len(), 2);
}

#[test]
fn conflicting_immutable_candidate_fields_fail_typed() {
    let target = TestJobStorePath::new("candidate-conflict");
    let candidate_file = TestPdf::new("candidate-conflict");
    let candidate = candidate_file.candidate();
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store.promote_candidate(&candidate, timestamp(10)).unwrap();
    store
        .connection
        .lock()
        .unwrap()
        .execute(
            "UPDATE pdf_import_jobs SET byte_length = byte_length + 1 WHERE job_id = ?1",
            [job.job_id().to_string()],
        )
        .unwrap();

    assert_eq!(
        store.promote_candidate(&candidate, timestamp(11)),
        Err(PdfImportJobStoreError::CandidateConflict)
    );
}

#[test]
fn concurrent_promotions_return_one_durable_job() {
    let target = TestJobStorePath::new("promotion-race");
    let candidate_file = TestPdf::new("promotion-race");
    let candidate = Arc::new(candidate_file.candidate());
    let stores = two_open_stores(&target);
    let barrier = Arc::new(Barrier::new(2));

    let outcomes = thread::scope(|scope| {
        let handles = stores
            .iter()
            .map(|store| {
                let candidate = Arc::clone(&candidate);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    store.promote_candidate(&candidate, timestamp(10))
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(outcomes.iter().all(Result::is_ok));
    let first_id = outcomes[0].as_ref().unwrap().job_id();
    assert_eq!(outcomes[1].as_ref().unwrap().job_id(), first_id);
    assert_eq!(stores[0].list().unwrap().len(), 1);
}

#[test]
fn concurrent_claims_allow_one_owner() {
    let target = TestJobStorePath::new("claim-race");
    let candidate_file = TestPdf::new("claim-race");
    let stores = two_open_stores(&target);
    let job = stores[0]
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let job_id = job.job_id();
    let barrier = Arc::new(Barrier::new(2));

    let outcomes = thread::scope(|scope| {
        let handles = stores
            .iter()
            .map(|store| {
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    store.claim(job_id, timestamp(11))
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert_eq!(outcomes.iter().filter(|value| value.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|value| matches!(
                value,
                Err(PdfImportJobStoreError::JobNotClaimable {
                    state: PdfImportJobState::InProgress
                })
            ))
            .count(),
        1
    );
    let claimed = stores[0].get(job_id).unwrap().unwrap();
    assert_eq!(claimed.state(), PdfImportJobState::InProgress);
    assert_eq!(claimed.attempt_count(), 1);
}

#[test]
fn claim_capability_is_hashed_and_debug_redacted() {
    let target = TestJobStorePath::new("claim-secret");
    let candidate_file = TestPdf::new("claim-secret");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let claim = store.claim(job.job_id(), timestamp(11)).unwrap();
    let raw_token = claim.token().to_string();
    let stored_hash: Vec<u8> = store
        .connection
        .lock()
        .unwrap()
        .query_row(
            "SELECT claim_token_hash FROM pdf_import_jobs WHERE job_id = ?1",
            [job.job_id().to_string()],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(stored_hash.len(), 32);
    assert!(
        !stored_hash
            .windows(raw_token.len())
            .any(|value| value == raw_token.as_bytes())
    );
    assert!(!format!("{claim:?}").contains(&raw_token));
    assert!(format!("{claim:?}").contains("<redacted>"));
}

#[test]
fn foreign_claim_cannot_checkpoint_or_finish() {
    let target = TestJobStorePath::new("foreign-claim");
    let candidate_file = TestPdf::new("foreign-claim");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let current = store.claim(job.job_id(), timestamp(11)).unwrap();
    let foreign = PdfImportJobClaim {
        job_id: current.job_id(),
        token: Uuid::new_v4(),
    };

    assert_eq!(
        store.persist_checkpoint(
            foreign,
            PdfImportJobCheckpoint::IntakeValidated,
            PdfImportJobCheckpoint::IntakeValidated,
            timestamp(12)
        ),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    assert_eq!(
        store.resolve(foreign, timestamp(12)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    assert_eq!(
        store.fail(foreign, &failure(), timestamp(12)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    assert_eq!(store.get(job.job_id()).unwrap().unwrap().attempt_count(), 1);
}

#[test]
fn checkpoint_and_typed_failure_survive_retry_and_reopen() {
    let target = TestJobStorePath::new("retry-reopen");
    let candidate_file = TestPdf::new("retry-reopen");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let claim = store.claim(job.job_id(), timestamp(11)).unwrap();
    store
        .persist_checkpoint(
            claim,
            PdfImportJobCheckpoint::IntakeValidated,
            PdfImportJobCheckpoint::IntakeValidated,
            timestamp(12),
        )
        .unwrap();
    let failed = store.fail(claim, &failure(), timestamp(13)).unwrap();
    assert_eq!(failed.state(), PdfImportJobState::Failed);
    assert_eq!(failed.last_failure(), Some(&failure()));
    assert_eq!(
        store.resolve(claim, timestamp(14)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    let pending = store.retry_failed(job.job_id(), 1, timestamp(15)).unwrap();
    assert_eq!(pending.attempt_count(), 1);
    assert_eq!(pending.last_failure(), Some(&failure()));
    drop(store);

    let reopened = PdfImportJobStore::open_at(target.path(), timestamp(16)).unwrap();
    let rehydrated = reopened.get(job.job_id()).unwrap().unwrap();
    assert_eq!(
        rehydrated.checkpoint(),
        PdfImportJobCheckpoint::IntakeValidated
    );
    assert_eq!(rehydrated.last_failure(), Some(&failure()));
    reopened.claim(job.job_id(), timestamp(17)).unwrap();
    assert_eq!(
        reopened.get(job.job_id()).unwrap().unwrap().attempt_count(),
        2
    );
}

#[test]
fn retry_and_reopen_require_expected_terminal_state_and_attempt() {
    let target = TestJobStorePath::new("retry-preconditions");
    let candidate_file = TestPdf::new("retry-preconditions");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    assert_eq!(
        store.retry_failed(job.job_id(), 0, timestamp(11)),
        Err(PdfImportJobStoreError::InvalidTransition {
            from: PdfImportJobState::Pending,
            to: PdfImportJobState::Pending,
        })
    );
    let claim = store.claim(job.job_id(), timestamp(12)).unwrap();
    let manual = store
        .require_manual_input(claim, &failure(), timestamp(13))
        .unwrap();
    assert_eq!(manual.state(), PdfImportJobState::NeedsManualInput);
    assert_eq!(
        store.reopen_manual_input(job.job_id(), 0, timestamp(14)),
        Err(PdfImportJobStoreError::AttemptCountMismatch)
    );
    assert_eq!(
        store
            .reopen_manual_input(job.job_id(), 1, timestamp(14))
            .unwrap()
            .state(),
        PdfImportJobState::Pending
    );
}

#[test]
fn durable_cancellation_blocks_progress_until_owner_acknowledges() {
    let target = TestJobStorePath::new("cancellation");
    let candidate_file = TestPdf::new("cancellation");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let claim = store.claim(job.job_id(), timestamp(11)).unwrap();

    let requested = store
        .request_cancellation(job.job_id(), timestamp(12))
        .unwrap();
    assert_eq!(requested.state(), PdfImportJobState::InProgress);
    assert!(requested.cancel_requested());
    assert_eq!(
        store.resolve(claim, timestamp(13)),
        Err(PdfImportJobStoreError::CancellationRequested)
    );
    assert_eq!(
        store.fail(claim, &failure(), timestamp(13)),
        Err(PdfImportJobStoreError::CancellationRequested)
    );
    let cancelled = store
        .acknowledge_cancellation(claim, timestamp(14))
        .unwrap();
    assert_eq!(cancelled.state(), PdfImportJobState::Cancelled);
    assert_eq!(
        store.resolve(claim, timestamp(15)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
}

#[test]
fn restart_invalidates_old_claim_and_reassignment_uses_new_token() {
    let target = TestJobStorePath::new("recovery");
    let candidate_file = TestPdf::new("recovery");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let stale = store.claim(job.job_id(), timestamp(11)).unwrap();
    drop(store);

    let recovered = PdfImportJobStore::open_at(target.path(), timestamp(20)).unwrap();
    let pending = recovered.get(job.job_id()).unwrap().unwrap();
    assert_eq!(pending.state(), PdfImportJobState::Pending);
    assert_eq!(pending.attempt_count(), 1);
    assert_eq!(
        pending.checkpoint(),
        PdfImportJobCheckpoint::IntakeValidated
    );
    let current = recovered.claim(job.job_id(), timestamp(21)).unwrap();
    assert_ne!(stale, current);
    assert_eq!(
        recovered.persist_checkpoint(
            stale,
            PdfImportJobCheckpoint::IntakeValidated,
            PdfImportJobCheckpoint::IntakeValidated,
            timestamp(22),
        ),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    assert_eq!(
        recovered.fail(stale, &failure(), timestamp(22)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
    assert_eq!(
        recovered.resolve(current, timestamp(22)).unwrap().state(),
        PdfImportJobState::Resolved
    );
}

#[test]
fn restart_turns_cancelled_in_progress_job_terminal() {
    let target = TestJobStorePath::new("cancel-recovery");
    let candidate_file = TestPdf::new("cancel-recovery");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let stale = store.claim(job.job_id(), timestamp(11)).unwrap();
    store
        .request_cancellation(job.job_id(), timestamp(12))
        .unwrap();
    drop(store);

    let recovered = PdfImportJobStore::open_at(target.path(), timestamp(20)).unwrap();
    let cancelled = recovered.get(job.job_id()).unwrap().unwrap();
    assert_eq!(cancelled.state(), PdfImportJobState::Cancelled);
    assert_eq!(
        recovered.acknowledge_cancellation(stale, timestamp(21)),
        Err(PdfImportJobStoreError::ClaimOwnershipLost)
    );
}

#[test]
fn terminal_resolution_is_immutable() {
    let target = TestJobStorePath::new("terminal");
    let candidate_file = TestPdf::new("terminal");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    let claim = store.claim(job.job_id(), timestamp(11)).unwrap();
    store.resolve(claim, timestamp(12)).unwrap();

    assert_eq!(
        store.request_cancellation(job.job_id(), timestamp(13)),
        Err(PdfImportJobStoreError::TerminalStateImmutable {
            state: PdfImportJobState::Resolved,
        })
    );
    assert_eq!(
        store.claim(job.job_id(), timestamp(13)),
        Err(PdfImportJobStoreError::JobNotClaimable {
            state: PdfImportJobState::Resolved,
        })
    );
}

#[test]
fn terminal_failures_are_bounded_and_typed() {
    assert_eq!(
        PdfImportJobFailure::new(PdfImportJobFailureCode::ProcessingFailed, " "),
        Err(PdfImportJobFailureError::EmptyMessage)
    );
    assert_eq!(
        PdfImportJobFailure::new(
            PdfImportJobFailureCode::ProcessingFailed,
            "x".repeat(MAX_JOB_FAILURE_MESSAGE_BYTES + 1),
        ),
        Err(PdfImportJobFailureError::MessageTooLong)
    );
    assert_eq!(failure().code(), PdfImportJobFailureCode::SourceUnavailable);
    assert_eq!(failure().message(), "source unavailable at claim time");
}

#[test]
fn malformed_stored_identity_fails_rehydration() {
    let target = TestJobStorePath::new("corrupt-identity");
    let candidate_file = TestPdf::new("corrupt-identity");
    let store = PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap();
    let job = store
        .promote_candidate(&candidate_file.candidate(), timestamp(10))
        .unwrap();
    store
        .connection
        .lock()
        .unwrap()
        .execute(
            "UPDATE pdf_import_jobs SET job_id = 'invalid' WHERE job_id = ?1",
            [job.job_id().to_string()],
        )
        .unwrap();
    drop(store);

    assert_eq!(
        PdfImportJobStore::open_at(target.path(), timestamp(11)).err(),
        Some(PdfImportJobStoreError::MalformedStoredIdentity)
    );
}

#[test]
fn store_error_messages_are_bounded() {
    let cases = [
        (
            PdfImportJobStoreError::ClaimOwnershipLost,
            "PDF import job claim ownership was lost",
        ),
        (
            PdfImportJobStoreError::CancellationRequested,
            "PDF import job cancellation was requested",
        ),
        (
            PdfImportJobStoreError::CandidateConflict,
            "PDF candidate conflicts with its durable job",
        ),
        (
            PdfImportJobStoreError::InvalidStoredFailure,
            "stored PDF import job failure is invalid",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

fn timestamp(value: u64) -> JobTimestamp {
    JobTimestamp::from_unix_millis(value)
}

fn failure() -> PdfImportJobFailure {
    PdfImportJobFailure::new(
        PdfImportJobFailureCode::SourceUnavailable,
        "source unavailable at claim time",
    )
    .unwrap()
}

fn two_open_stores(target: &TestJobStorePath) -> [PdfImportJobStore; 2] {
    [
        PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap(),
        PdfImportJobStore::open_at(target.path(), timestamp(1)).unwrap(),
    ]
}

fn read_schema(store: &PdfImportJobStore) -> u64 {
    let version: i64 = store
        .connection
        .lock()
        .unwrap()
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    version.try_into().unwrap()
}

fn job_table_exists(store: &PdfImportJobStore) -> bool {
    store
        .connection
        .lock()
        .unwrap()
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = 'pdf_import_jobs'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

struct TestJobStorePath {
    _directory: tempfile::TempDir,
    path: PathBuf,
}

impl TestJobStorePath {
    fn new(label: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(format!("{label}.sqlite3"));
        Self {
            _directory: directory,
            path,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

struct TestPdf {
    _directory: tempfile::TempDir,
    path: PathBuf,
}

impl TestPdf {
    fn new(label: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(format!("{label}.pdf"));
        fs::write(&path, COMPLETE_PDF).unwrap();
        Self {
            _directory: directory,
            path,
        }
    }

    fn candidate(&self) -> PendingPdfImport {
        prepare_explicit_pdf(&self.path).unwrap()
    }
}
