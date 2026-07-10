# Background Job Requirements Draft

## Status

This is a non-binding Phase 26 requirements draft. No persistent job store,
scheduler, queue, or background product worker exists at the Phase 25
checkpoint. Implemented behavior must be recorded separately and must not be
described as an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`.

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
- `cancel_requested`.

PDF-import jobs must retain enough private Rust-owned source information to
resume after restart. Paths and raw operating-system errors must not appear in
frontend payloads, logs, display errors, or document data. Full PDF contents
must not be stored in the job row.

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
InProgress -> Pending | Resolved | Failed | NeedsManualInput | Cancelled
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

Cancellation is durable. A request sets `cancel_requested` transactionally.
An unclaimed job may move directly to `Cancelled`; claimed work must observe
the request before recording another checkpoint or terminal state.

## Crash And Restart

Opening the job store after interruption must preserve every row and its last
valid checkpoint. Recovery moves interrupted `InProgress` rows without a
cancellation request back to `Pending` without clearing the checkpoint, error
history, or attempt count. An interrupted row with `cancel_requested` set moves
to `Cancelled` instead. A later claim can continue from the preserved
checkpoint only after recovery returns the job to `Pending`.

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
- every allowed transition and representative rejected transitions;
- transactional expected-state checks under competing claims;
- attempt-count and checkpoint behavior;
- durable cancellation requests;
- database close, reopen, and interrupted-job recovery;
- terminal states that cannot restart silently;
- typed corruption, migration, transition, and storage failures;
- no direct frontend, Python, document, network, or source-file authority; and
- local/GitHub Actions parity.

Phase 26 must replace the pre-implementation persistent-job absence gate with
these behavioral checks. `INV-05` cannot rely on naming scans once the state
machine exists.

## Non-Goals

Phase 26 does not add filesystem event subscriptions, background polling,
concurrent worker execution, retry timers, backoff policy, progress events,
Tauri commands, user controls, PDF parsing, metadata lookup, AI orchestration,
Python helper execution, or import completion. Later phases must add those
behaviors through their owning boundaries and invariants.
