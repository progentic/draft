# Worker Cancellation Boundary

## Purpose

Phase 9 establishes the reusable cancellation shape for user-initiated Rust
workers that continue after their start command returns. It does not introduce
a product worker, background-job persistence, or a cancellation control with
no real work behind it.

The boundary separates four concerns:

- a start command creates a Rust-owned worker registration
- the worker observes a cooperative cancellation token
- the frontend requests cancellation through a typed command wrapper
- the feature emits its own typed terminal event before the worker exits

Ignoring progress events does not cancel trusted work.

## Wire contract

The registered `cancel_worker` command accepts:

```json
{
  "request": {
    "workerId": "00000000-0000-4000-8000-000000000001"
  }
}
```

Rust parses the worker ID as a UUID. The frontend cannot choose an arbitrary
identifier for a real worker; a future start command returns the ID generated
by `WorkerCancellationRegistry::register`.

An active worker returns:

```json
{ "status": "cancellation_requested" }
```

A known worker whose registration guard has already ended returns:

```json
{ "status": "already_ended" }
```

Both are successful terminal command responses. Repeating cancellation while
the worker is still active returns `cancellation_requested` again. Repeating it
after the worker exits returns `already_ended` again.

The command-specific errors are:

```json
{ "code": "invalid_worker_id" }
{ "code": "worker_not_found" }
{ "code": "registry_unavailable" }
```

An unknown but valid UUID is different from an ended worker. This prevents a
mistyped or stale identifier from being reported as a successful cancellation.

## Worker lifecycle

1. A feature start command calls `WorkerCancellationRegistry::register`.
2. Rust returns the registration's worker ID in the typed start response.
3. The spawned worker retains the registration guard for its full lifetime.
4. The worker waits on or checks the guard's `WorkerCancellation` signal.
5. The frontend calls `cancelWorker` with the returned ID.
6. `cancel_worker` validates the ID and signals the matching token.
7. The worker stops at a safe boundary, performs cleanup, and emits its
   feature-specific terminal event.
8. The registration guard drops and records the worker as ended.

The cancel command reports that cancellation was requested. It does not claim
the worker has already completed cleanup. The terminal event resolves that
later state for the UI.

Dropping the application registry signals all active cancellation tokens. This
provides the process-shutdown side of the lifecycle without giving the
frontend authority over worker storage or task handles.

## Ownership layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | Feature start/cancel session | Presents a real user action and resolves the terminal event. |
| Mid | `cancel_worker` | Validates the request and maps registry outcomes to command DTOs. |
| Mid | `WorkerCancellationRegistry` | Generates IDs, tracks active/ended state, and signals cancellation. |
| Low | `WorkerCancellation` | Wraps the cooperative `CancellationToken` observed by worker code. |
| Low | `cancelWorker` | Calls the command and validates unknown IPC output for TypeScript callers. |

Worker-specific modules remain responsible for work, cleanup, progress, and
terminal events. The generic registry must not interpret feature payloads or
persist product job state.

## Process-local scope

The Phase 9 registry retains active and ended worker identities for the current
application process. This supports idempotent cancellation during one runtime.
It is not crash-resumable and does not replace the Phase 26 persistent job
state machine.

When persistent jobs arrive, durable job state owns restart and checkpoint
behavior. The transient cancellation token still owns stopping the currently
running task.

## Frontend boundary

`src/ipc/workerCancellation.ts` is the typed frontend wrapper. It owns:

- the `cancel_worker` command name
- the camel-case request envelope
- exact success-response validation
- command-error classification
- invalid-response and transport categories

No React component calls this wrapper yet because no current UI starts a
long-running worker. Adding an inactive cancel button would overstate product
behavior.

## Required integration for future workers

A future user-initiated long-running worker is incomplete unless it:

- registers before spawning
- returns its Rust-generated worker ID
- moves the registration guard into the spawned task
- observes cancellation during blocking or streamed work
- performs bounded cleanup after cancellation
- emits a typed terminal event such as `cancelled`
- exposes a user-visible frontend action that calls `cancelWorker`
- tests success, repeated cancellation, already-ended behavior, and failures

Spawning is confined to `src-tauri/src/workers/` so worker lifecycle policy is
not reimplemented in command or feature modules.

## Enforcement

Rust tests pin command signature, request, response, and error serialization.
Lifecycle tests prove active cancellation, repeated cancellation,
already-ended idempotence, malformed IDs, unknown IDs, registration teardown,
and registry-shutdown signaling.

Frontend tests pin the command request, both successful outcomes, invalid
responses, every command error, and unknown transport failure.

`scripts/check-invariants.sh` requires the cancellation sources and named
lifecycle tests, and rejects direct Rust worker spawning outside the worker
module. The aggregate local verifier and GitHub Actions run the same checks.

Run focused evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
npm test
bash scripts/check-invariants.sh
```
