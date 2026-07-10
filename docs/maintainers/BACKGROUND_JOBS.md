# Persistent PDF Import Jobs

## Status

This guide records implemented Phase 26 behavior. The requirements in
`docs/drafts/BACKGROUND_JOBS.md` remain non-binding until they complete the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 26 adds a Rust-owned, versioned SQLite state machine for PDF import jobs.
It promotes a validated Phase 24 `PendingPdfImport` candidate into durable
state before any later processing can begin.

The phase adds no PDF parser, metadata extractor, external request, Tauri
command, frontend model, visible import workflow, filesystem watcher,
scheduler, background execution loop, worker spawn, or reference mutation.

## Candidate Promotion

`PdfImportJobStore::promote_candidate` runs in an immediate SQLite transaction.
The Phase 24 `PdfImportId` is the deduplication identity and has a unique
database constraint. The first promotion creates a separate Rust-generated job
ID. Concurrent and repeated promotion of the same candidate returns that job
without resetting its state, attempts, checkpoint, error, or timestamps.

Source kind, canonical path, and byte length are immutable consistency fields.
A matching candidate ID with different fields returns `CandidateConflict`.
Different candidate IDs are not merged by path or size because Phase 24 has no
content fingerprint.

## Persisted State

The private `jobs.sqlite3` database stores:

- job and candidate identities;
- the fixed `pdf_import` kind;
- `Pending`, `InProgress`, `Resolved`, `Failed`, `NeedsManualInput`, or
  `Cancelled`;
- attempt count and `IntakeValidated` checkpoint;
- bounded typed last failure;
- source kind, platform-preserving canonical path bytes, and byte length;
- creation, update, and claim timestamps;
- durable cancellation intent; and
- an optional claim-token digest.

The version 1 schema is strict and verified at open. Unknown schema versions,
missing constraints, malformed identities, invalid rows, invalid paths, and
invalid stored failures fail with bounded typed errors.

## Claim Ownership

`claim` changes one `Pending` row to `InProgress`, increments `attempt_count`,
and writes a claim timestamp and SHA-256 digest of a new UUID v4 token in the
same immediate transaction. Exactly one concurrent claim can succeed.

The raw token exists only in the Rust `PdfImportJobClaim`. It is not serializable,
is absent from job snapshots and SQLite, and has redacted `Debug` output.
Checkpoint, completion, failure, manual-input, and cancellation-acknowledgment
updates hash the presented token and include its digest, `InProgress`, and the
required cancellation predicate in the SQL `WHERE` clause. A zero-row update
or foreign token returns `ClaimOwnershipLost` without changing state.

## Retry And Terminal State

`Resolved` and `Cancelled` are immutable. `Failed` and `NeedsManualInput` reject
worker mutations after the claim is cleared. The explicit `retry_failed` and
`reopen_manual_input` controls require the expected state and attempt count
before returning a job to `Pending`. Requeueing does not increment attempts;
the next successful claim does.

Failed attempts use only `SourceUnavailable`, `SourceChanged`,
`ProcessingFailed`, or `RetryLimitReached` and a non-empty message of at most
512 bytes. Retry preserves the last failure until a later failed attempt
replaces it.

## Cancellation And Recovery

Cancellation intent is transactional and durable. An unclaimed job becomes
`Cancelled` immediately. An in-progress job retains its current claim long
enough to acknowledge cancellation, but checkpoint, completion, and failure
updates reject once the intent is set.

Store startup validates every row, then recovers interrupted `InProgress` jobs.
Jobs without cancellation intent return to `Pending`; jobs with intent become
`Cancelled`. Recovery clears the old claim digest and timestamp while
preserving attempts, checkpoint, last failure, and immutable candidate data. A
later claim receives a new token, so pre-restart workers cannot mutate the job.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `promote_candidate` | Creates or returns one durable job before work. |
| High | claim and terminal methods | Coordinate validated lifecycle transitions. |
| Mid | transition preconditions | Enforce state, attempt, cancellation, and ownership rules. |
| Mid | recovery | Requeues or cancels interrupted claims without losing checkpoints. |
| Low | SQLite and encoding helpers | Perform transactions, row decoding, path encoding, and digest comparison. |

## Verification

Twenty Rust tests cover schema validation, durable promotion, exact candidate
deduplication, same-path non-deduplication, immutable-field conflicts,
two-connection promotion and claim races, hashed/redacted claims, foreign and
stale ownership rejection, checkpoint persistence, retries, manual-input
reopen, cancellation, restart recovery, terminal immutability, bounded
failures, and corrupt identity handling.

`scripts/check-invariants.sh` requires those behavioral tests and the core
schema, transaction, ownership, digest, recovery, and startup-registration
markers. It rejects raw token persistence or logging, serialization, Tauri or
frontend authority, network/reference coupling, source-file mutation, watcher
dependencies, and worker spawning.

## Known Limits

Opening the store is the recovery boundary for the single desktop process.
Phase 26 defines no multi-process lease, timer-based claim expiry, scheduler,
automatic retry, progress event, or worker integration. The only checkpoint is
`IntakeValidated` because all PDF processing remains deferred.

## Configuration Index

Job schema, filename, SQLite timeout, and failure-message bound are indexed in
`docs/maintainers/CONFIGURATION.md`.
