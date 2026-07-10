# Python Helper Protocol Requirements Draft

## Status

This is a non-binding Phase 28 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/PYTHON_HELPERS.md`. This draft does not
become an accepted contract without the lifecycle in `docs/GOVERNANCE.md`.

## Purpose

DRAFT needs one Rust-owned process boundary for deterministic Python tools.
Helpers must receive bounded typed input, return bounded typed output, and
remain unable to own application state, secrets, networking, source-document
mutation, or process orchestration.

## Scope

Phase 28 creates protocol version 1 and one allowlisted `contract_probe` helper.
The probe reports only the UTF-8 byte length of validated input. It proves the
process boundary and is not a user-facing text-analysis capability.

Rust owns helper selection, request identity, executable and entrypoint
configuration, process creation, standard streams, timeout, cancellation, exit
interpretation, response validation, and all later use of a result. Python owns
only the deterministic transformation inside the fixed helper entrypoint.

## Request Contract

One request is one UTF-8 JSON document written to stdin and followed by EOF.
The exact version 1 shape is:

```json
{
  "protocolVersion": 1,
  "requestId": "00000000-0000-4000-8000-000000000000",
  "helper": "contract_probe",
  "helperVersion": 1,
  "input": {
    "text": "bounded input",
    "locale": "en-US"
  }
}
```

Rust generates the UUID request ID. Closed Rust enums select the helper and
locale; callers cannot supply an entrypoint, command argument, environment
variable, file path, or arbitrary helper name. Text is non-empty after trimming
and at most 32 KiB. Serialized requests are at most 64 KiB.

## Response Contract

Success exits with code zero and writes exactly one JSON document to stdout:

```json
{
  "protocolVersion": 1,
  "requestId": "00000000-0000-4000-8000-000000000000",
  "helper": "contract_probe",
  "helperVersion": 1,
  "status": "ok",
  "result": {
    "utf8Bytes": 13
  }
}
```

Rust rejects unknown fields, invalid JSON, invalid UTF-8, wrong protocol or
helper versions, mismatched request identity, excessive output, and success
payloads from non-zero exits. Stdout is limited to 64 KiB. Stderr is captured
and drained with a separate 16 KiB retention limit but is never returned in an
error or logged by the boundary.

The helper emits a bounded machine-readable error document and exits non-zero
for invalid JSON, invalid requests, unsupported protocol versions, unsupported
helpers, or internal failure. Rust maps those codes to a closed error enum and
does not expose Python exception text.

## Process Boundary

The runner accepts only trusted Rust configuration for an absolute Python
executable and package root. It canonicalizes both, derives the fixed
`draft_helpers/worker.py` entrypoint beneath that root, and never accepts a path
from helper input or the frontend.

Rust starts Python directly without a shell, with isolated mode, bytecode
disabled, a cleared environment, piped standard streams, and kill-on-drop. The
default timeout is five seconds. Timeout or cooperative cancellation kills and
reaps the child before returning a typed terminal error. Application shutdown
also drops a kill-on-drop child rather than detaching it.

## Python Boundary

The helper uses the Python 3.12-or-newer standard library only. It may parse the
one request, validate the exact schema, run the allowlisted deterministic
operation, and serialize the one response.

The helper must not read or write files, inspect directories, access environment
variables, open sockets, import network, credential, database, or subprocess
packages, execute a shell, print input to stderr, or create another process.

## Failure Shape

Rust failures distinguish invalid local input, invalid runtime configuration,
spawn or I/O failure, timeout, cancellation, excessive stdout, failed execution,
helper rejection, invalid output, and response mismatch. Error display strings
contain no request text, response text, stderr, path, environment value, process
argument, or raw operating-system error.

## Verification

Tests and scans must cover:

- stable Rust and Python request/response shapes;
- input, request, stdout, and stderr bounds;
- exact allowlist and protocol/helper versions;
- Unicode byte counting;
- real isolated subprocess round trips;
- canonical executable and fixed-entrypoint validation;
- cleared environment and no shell invocation;
- timeout, cancellation, child termination, and reaping;
- non-zero exit, malformed output, excessive output, and mismatched response;
- bounded errors without payload, stderr, path, or raw process details;
- denied Python network, credential, database, filesystem-write, environment,
  and subprocess APIs;
- no Tauri command, frontend helper model, persistence, document/reference
  mutation, or external request; and
- local/GitHub Actions parity.

## Non-Goals

Phase 28 does not add grammar, clarity, tone, cohesion, voice, formatting,
parsing, PDF processing, metadata extraction, model calls, third-party Python
dependencies, file-based helper input, temporary output files, Tauri commands,
frontend controls, events, persistent findings, automatic document edits,
worker spawning, or packaged Python-runtime discovery.
