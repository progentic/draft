# Python Helper Boundary

## Status

This guide records implemented Phase 28 behavior. The requirements in
`docs/drafts/PYTHON_HELPERS.md` remain non-binding until they complete the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 28 adds a Rust-owned, versioned stdin/stdout process protocol and one
allowlisted `contract_probe` helper. The probe returns only the UTF-8 byte count
of validated text. It verifies the boundary and is not a product text-analysis
feature.

The phase adds no grammar, clarity, tone, cohesion, voice, formatting, parsing,
PDF, metadata, model, Tauri, frontend, event, persistence, finding, document
mutation, third-party Python dependency, or packaged-runtime discovery.

## Typed Protocol

Protocol version 1 carries a Rust-generated UUID request ID, the closed
`contract_probe` helper name, helper version 1, bounded text, and the closed
`en-US` locale. Text must be non-empty and no larger than 32 KiB. Complete JSON
requests are limited to 64 KiB.

The Python worker validates the exact field set, protocol and helper versions,
canonical UUID string, locale, and text bounds before dispatch. A successful
response repeats the request identity and versions and returns one typed
`utf8Bytes` result. Rust rejects unknown fields, malformed JSON, wrong identity
or versions, and a byte count inconsistent with the validated request.

Python failures use only `invalid_json`, `invalid_request`,
`unsupported_protocol`, `unsupported_helper`, or `internal_failure`, then exit
non-zero. Rust maps those values to `PythonHelperFailureCode` and never returns
Python exception text.

## Process Ownership

`PythonHelperRunner::new` accepts trusted Rust configuration only. It requires
an absolute canonical Python executable and package root, then derives and
canonicalizes the fixed `draft_helpers/worker.py` entrypoint. A symlink outside
the package root fails validation. Helper requests contain no path, command,
argument, environment, credential, or persistence field.

Rust starts the executable directly with `-I` and `-B`; no shell is involved.
The command clears inherited environment variables, fixes the working directory
to the validated package root, pipes all standard streams, and enables
kill-on-drop.

Rust writes one JSON request and closes stdin. It drains stdout and stderr
concurrently while waiting for process exit. Stdout retains at most 64 KiB and
stderr at most 16 KiB; excess bytes are drained without unbounded allocation.
Successful helpers must leave stderr empty. Captured stderr is never returned or
logged by this boundary.

## Timeout And Cancellation

One `WorkerRegistration` is moved into `run_contract_probe` and remains alive
for the complete child lifetime. The runner races the process exchange against
the existing cooperative cancellation token and a fixed five-second timeout.

Cancellation before spawn returns immediately. Cancellation or timeout after
spawn sends a kill request and waits for the child to exit before returning.
Dropping the future or application shutdown also reaches a kill-on-drop child,
so the runner never intentionally detaches helper work.

## Python Authority

The production helper uses only Python's standard library for JSON, data
classes, UUID validation, typing, and standard streams. `pyproject.toml` keeps
an empty dependency list.

The helper does not read or write files, inspect directories or environment,
open sockets, access credentials or databases, invoke subprocesses, or mutate
application state. Rust alone decides whether a later helper result may become
durable or affect a document.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `run_contract_probe` | Owns one validated helper call through terminal cleanup. |
| Mid | protocol and terminal classifiers | Enforce allowlist, identity, state, bounds, and typed outcomes. |
| Low | command, stream, and path helpers | Canonicalize trusted files, spawn Python, move bytes, kill, and reap. |
| Python | `process_request` | Validate and dispatch one deterministic allowlisted operation. |

## Verification

Seventeen focused Rust tests cover request/response serialization, input bounds,
strict fields, response identity, Unicode, canonical runtime configuration,
real isolated subprocess execution, cleared environment, timeout, cancellation,
reaping, malformed and oversized stdout, stderr on success, non-zero typed
failure, registration cleanup, and bounded error strings.

Six Python tests independently cover the exported protocol types, stable success
shape, invalid JSON, unknown fields, exact allowlist and versions, UUID, locale,
text and request bounds, and closed failure documents.

`scripts/check-invariants.sh` requires those tests and the fixed protocol,
process, timeout, cancellation, environment, stream, and dependency markers. It
rejects production Python network, credential, database, filesystem,
environment, and subprocess authority plus Tauri, frontend, persistence,
document/reference mutation, and external requests in the Rust boundary.

The hostile fixture at
`src-tauri/src/workers/python/worker_fixture.py` exists only to test timeout,
cancellation, environment clearing, stderr, excessive output, malformed output,
and non-zero exits. It is not under the production `python/draft_helpers`
package, is reachable only through a `cfg(test)` constructor, and is excluded
only from production-helper API scans that would otherwise reject its deliberate
test behavior.

## Current Limits

No application state initializes `PythonHelperRunner`, and no Tauri or frontend
surface can start it. Packaging still needs a trusted Python runtime/resource
location. Phase 29 may add an actual text-analysis helper by extending the
closed protocol and tests without weakening the Phase 28 process boundary.
