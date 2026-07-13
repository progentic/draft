# Python Helper Boundary

## Status

This guide records the implemented Phase 28 process boundary and its Phase 29
protocol extension. The requirements in `docs/drafts/PYTHON_HELPERS.md` and
`docs/drafts/TEXT_ANALYSIS.md` remain non-binding until they complete the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 28 adds a Rust-owned, versioned stdin/stdout process protocol and one
allowlisted `contract_probe` helper. The probe returns only the UTF-8 byte count
of validated text and verifies the process boundary.

Phase 29 extends the same closed protocol with `text_analysis` version 1. That
operation returns only five deterministic review codes and UTF-8 byte ranges;
Rust validates them and owns all user-facing wording. The detailed behavior is
recorded in `docs/maintainers/TEXT_ANALYSIS.md`.

Together these phases add no formatting helper, parsing, PDF or metadata work,
model call, event, persistence, document mutation, or third-party Python
dependency. Phase 46 adds the production Tauri/frontend call path and packaged
resource discovery without expanding the helper protocol or Python authority.

## Typed Protocol

Protocol version 1 carries a Rust-generated UUID request ID, one closed helper
name, helper version 1, bounded text, and the closed `en-US` locale. The
allowlist contains `contract_probe` and `text_analysis`. Text must be non-empty
and no larger than 32 KiB. Complete JSON requests are limited to 64 KiB.

The Python worker validates the exact field set, protocol and helper versions,
canonical UUID string, locale, and text bounds before dispatch. A successful
response repeats the request identity and versions and returns the exact result
for the selected helper: `utf8Bytes` for the probe or bounded closed findings
for text analysis. Rust rejects unknown fields, malformed JSON, wrong identity
or versions, and helper/result mismatches.

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
The command clears inherited environment variables, restores only the fixed
`TMPDIR=/tmp` required by Apple system Python, fixes the working directory to
the validated package root, pipes all standard streams, and enables kill-on-drop.

Rust writes one JSON request and closes stdin. It drains stdout and stderr
concurrently while waiting for process exit. Stdout retains at most 64 KiB and
stderr at most 16 KiB; excess bytes are drained without unbounded allocation.
Successful helpers must leave stderr empty. Captured stderr is never returned or
logged by this boundary.

## Timeout And Cancellation

One `WorkerRegistration` is moved into `run_contract_probe` or
`run_text_analysis` and remains alive for the complete child lifetime. The
runner races the process exchange against the existing cooperative cancellation
token and a fixed five-second timeout.

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
| High | `run_contract_probe` / `run_text_analysis` | Owns one validated helper call through terminal cleanup. |
| Mid | protocol and terminal classifiers | Enforce allowlist, identity, state, bounds, and typed outcomes. |
| Low | command, stream, and path helpers | Canonicalize trusted files, spawn Python, move bytes, kill, and reap. |
| Python | `process_request` | Validate and dispatch one deterministic allowlisted operation. |

## Verification

The Phase 28 baseline has seventeen focused Rust tests covering
request/response serialization, input bounds,
strict fields, response identity, Unicode, canonical runtime configuration,
real isolated subprocess execution, cleared environment, timeout, cancellation,
reaping, malformed and oversized stdout, stderr on success, non-zero typed
failure, registration cleanup, and bounded error strings.

Six Phase 28 Python tests independently cover the exported protocol types, stable success
shape, invalid JSON, unknown fields, exact allowlist and versions, UUID, locale,
text and request bounds, and closed failure documents.

Phase 29 adds twelve Rust and five Python tests for its second allowlist entry,
all five findings, strict result validation, Unicode ranges, deterministic
ordering, thresholds, false-positive guards, and the unchanged real subprocess
boundary. The complete helper suite therefore has twenty-nine focused Rust tests
and eleven Python tests.

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

## Phase 46 Production Path

The `run_text_analysis` command is the only production constructor for
`PythonHelperRunner`. It resolves the packaged `python/draft_helpers` resource,
uses the fixed `/usr/bin/python3` executable on the supported macOS platform,
and delegates one validated snapshot through the existing process boundary.
The unsigned package check executes the embedded helper under an isolated
environment before accepting the bundle.

The visible workflow exposes exactly the existing five heuristic finding
types. It adds no provider, credential, model runtime, network, persistence, or
document-mutation authority. Phase 31 formatting checks remain separately
bounded pure Rust work.

## Configuration Index

Protocol versions, process timeout, request, stdout, and stderr bounds are
indexed in `docs/maintainers/CONFIGURATION.md`.
