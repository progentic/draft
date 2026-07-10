# Background Job Requirements Draft

## Status

This is a non-binding Phase 26 requirements draft. No persistent job store,
scheduler, queue, or background product worker exists at the Phase 25
checkpoint. Implemented behavior must be recorded separately and must not be
described as an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`. The implemented Phase 26 boundary is now recorded in
`docs/maintainers/BACKGROUND_JOBS.md`.

## Purpose

Long-running work must survive a process interruption without losing its last
valid checkpoint or silently repeating unrecorded work. Rust must persist job
ownership before processing begins.

## Scope

Phase 26 adds a Rust-owned SQLite state machine for durable jobs. It may adopt a
validated Phase 24 `PendingPdfImport` candidate as a PDF-import job, but it does
not perform PDF parsing, metadata resolution, file copying, reference mutation,
or any other import work.

The state machine is an internal Rust boundary. Phase 26 does not add a Tauri
command, frontend job model, visible import flow, filesystem watcher, scheduler
loop, worker thread, network request, Python helper, or background execution.

## Persisted Record

Every job row must contain:

- a Rust-generated opaque `job_id`;
- the adopted candidate's `PdfImportId` as `record_id`;
- `PdfImport` as the only Phase 26 `job_kind`;
- one valid lifecycle state;
- `attempt_count`;
- an optional bounded `last_error` with a typed code and a message no longer
  than 512 bytes;
- `IntakeValidated` as the initial typed `last_checkpoint`;
- `created_at` and `updated_at` timestamps; and
- `cancel_requested`; and
- an optional claim-token digest and `claimed_at` timestamp present only while
  `InProgress`.

PDF-import jobs must retain enough private Rust-owned source information to
resume after restart. Paths and raw operating-system errors must not appear in
frontend payloads, logs, display errors, or document data. Full PDF contents
must not be stored in the job row.

## Promotion And Deduplication

Candidate identity is the Rust-generated `PdfImportId` created by Phase 24.
Source kind, canonical path, and byte length are immutable consistency fields,
not alternative identities. Promotion is transactional and idempotent by that
candidate identity: the first call creates one Rust-generated job identity, and
every concurrent or repeated promotion returns the same stored job. A matching
identity with different immutable fields is a typed candidate conflict.

Phase 26 does not merge separately validated candidates by path or byte length.
The Phase 24 size-only contract does not provide a content fingerprint, so two
candidate IDs remain distinct even when they name the same path. Content-based
cross-candidate deduplication remains deferred until a later phase owns a
stronger identity signal.

## Lifecycle

The only states are:

```text
Pending
InProgress
Resolved
Failed
NeedsManualInput
Cancelled
```

Allowed transitions are:

```text
Pending -> InProgress | Cancelled
InProgress -> Resolved | Failed | NeedsManualInput | Cancelled
NeedsManualInput -> Pending | Cancelled
Failed -> Pending | Cancelled
Resolved -> no transition
Cancelled -> no transition
```

Every transition must be transactional, reject stale expected-state updates,
and update `updated_at`. Claiming `Pending` as `InProgress` increments
`attempt_count`. A pending job with `cancel_requested` set cannot be claimed. A
checkpoint is written in the same transaction as the state change that makes
it valid.

Claiming creates a random opaque claim token and claim timestamp. Exactly one
claim may exist. Every in-progress checkpoint, completion, failure,
manual-input, and cancellation-acknowledgment mutation requires `InProgress`,
the current token, and an allowed transition in the same SQLite statement. A
zero-row update returns a typed ownership error. Process IDs are not ownership
proof, and Phase 26 adds no lease timer or worker identity.

The raw token is a secret-like Rust capability generated from the UUID v4
cryptographic random source. It is never serialized, logged, included in a job
snapshot, or persisted. SQLite stores only its SHA-256 digest; mutation
predicates compare the digest of the presented token. Claim `Debug` output is
redacted.

Leaving `InProgress` clears the claim. `Failed` and `NeedsManualInput` are
immutable to worker mutations; only explicit retry or reopen operations may
return them to `Pending`. Those control operations require the expected state
and attempt count but no expired worker token. `Resolved` and `Cancelled` are
fully immutable. Claiming, not retrying, increments `attempt_count`.

Typed failed-attempt codes are limited to `source_unavailable`,
`source_changed`, `processing_failed`, and `retry_limit_reached`. Failure state
and bounded diagnostic context are persisted in one transaction. Retrying
preserves that value as `last_error` until a later failure replaces it.

Cancellation is durable. A request sets `cancel_requested` transactionally.
An unclaimed job may move directly to `Cancelled`; claimed work must observe
the request before recording another checkpoint or terminal state. Once intent
is recorded, completion and failure updates reject even the current token; only
the current claim may acknowledge cancellation and enter `Cancelled`.

## Crash And Restart

Opening the job store after interruption must preserve every row and its last
valid checkpoint. Recovery moves interrupted `InProgress` rows without a
cancellation request back to `Pending` without clearing the checkpoint, error
history, or attempt count. An interrupted row with `cancel_requested` set moves
to `Cancelled` instead. A later claim can continue from the preserved
checkpoint only after recovery returns the job to `Pending`.

Recovery clears the old token and claim timestamp. Reassignment always creates
a new token, so a pre-crash or losing worker receives the typed ownership error
and cannot advance a checkpoint or terminal state.

No work may begin before the initial `Pending` row is committed. Phase 26 tests
the durable recovery contract through database close and reopen; it does not
start a real worker to prove it.

## Storage Boundary

The job store uses a versioned SQLite schema and explicit migration dispatch.
Rust owns all connections and transactions. Frontend code, Python helpers, and
documents cannot read or write job tables directly.

Schema initialization, adoption of an intake candidate, state transition,
checkpoint update, cancellation, recovery, and terminal-state handling return
typed bounded errors. Corrupt rows, unknown schema versions, unknown job kinds,
unknown states, invalid transitions, stale updates, and database failures fail
closed.

## Verification

Tests and scans must cover:

- schema initialization and exact persisted fields;
- Rust-generated identities and candidate adoption before any work;
- two concurrent promotions returning one durable job;
- candidate-ID deduplication without path-based cross-candidate merging;
- every allowed transition and representative rejected transitions;
- transactional expected-state checks under competing claims;
- one winning claim and typed stale/foreign-token rejection;
- attempt-count and checkpoint behavior;
- durable cancellation requests;
- database close, reopen, and interrupted-job recovery;
- terminal states that cannot restart silently;
- typed corruption, migration, transition, and storage failures;
- no direct frontend, Python, document, network, or source-file authority; and
- local/GitHub Actions parity.

The acceptance race uses two store connections. Two callers promote and claim
the same candidate concurrently. One row and one active token may exist. The
loser and any token invalidated by recovery cannot change the checkpoint,
cancellation result, completion, or failure state.

Phase 26 replaces the pre-implementation persistent-job absence gate with these
behavioral checks. `INV-05` no longer relies on naming scans for the implemented
state machine.

## Non-Goals

Phase 26 does not add filesystem event subscriptions, background polling,
concurrent worker execution, retry timers, backoff policy, progress events,
Tauri commands, user controls, PDF parsing, metadata lookup, AI orchestration,
Python helper execution, or import completion. Later phases must add those
behaviors through their owning boundaries and invariants.
