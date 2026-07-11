# Rust Command Boundary

**Status:** Implemented checkpoint guide; not an accepted contract under
`GOVERNANCE.md` §7.

## Purpose

Phase 6 establishes the Tauri request/response boundary used by trusted Rust
work. It provides the enforcement pattern every later command must follow.

The Phase 7 typed frontend wrappers are documented in
`docs/maintainers/FRONTEND_COMMAND_CLIENT.md`.

## Runtime status command

`get_runtime_status` confirms that the Rust process has valid application
metadata compiled into it.

The command accepts a bounded request envelope with no fields:

```json
{
  "request": {}
}
```

Unknown request fields are rejected before domain work begins. The command
returns:

```json
{
  "version": "0.1.0"
}
```

The response version comes from Rust package metadata. The frontend does not
supply or decide it.

The command-specific error codes are:

```json
{ "code": "invalid_application_version" }
{ "code": "event_delivery_failed" }
```

The delivery error is returned when the documented Phase 8 runtime-status
event cannot reach current frontend listeners.

No generic string, `anyhow::Error`, `serde_json::Value`, or boxed error crosses
the IPC boundary.

## Worker cancellation command

Phase 9 established `cancel_worker`, which accepts a Rust-generated UUID in a bounded
camel-case request:

```json
{
  "request": {
    "workerId": "00000000-0000-4000-8000-000000000001"
  }
}
```

It returns either `cancellation_requested` or `already_ended`. Its typed errors
distinguish an invalid UUID, an unknown worker, and an unavailable registry.
The complete lifecycle is documented in
`docs/maintainers/CANCELLATION_BOUNDARY.md`.

## Document file commands

Phase 13 adds `open_document` and `save_document`. `open_document` accepts an
empty request because Rust owns native file selection. `save_document` accepts
one untrusted `snapshot` value; Rust validates the envelope before registry or
filesystem work.

Both commands return typed opened/saved/cancelled responses and bounded nested
errors. Phase 14 adds typed atomic-write stages and a distinct
`durability_uncertain` failure after a complete replacement whose parent sync
fails. The commands expose no path field to the frontend. The full lifecycle
and atomic-write behavior are documented in
`docs/maintainers/DOCUMENT_SAVE_LOAD.md`.

## Citation resolution command

Phase 18 adds `resolve_citation`. It accepts one untrusted `attrs` JSON value so
Rust can return typed missing, malformed, and unsupported-field failures instead
of transport-deserialization strings. Domain validation requires version 1,
the shared citekey shape, and `apa7`, then resolves the exact citekey through
managed `ReferenceStore` state.

The response carries only schema version, citekey, render style, and a
disposable display marker. Typed errors distinguish invalid attrs, a missing
reference, and bounded unavailable/read/corruption store categories. The full
contract is documented in `docs/maintainers/CITATION_NODE.md`.

## External browser handoff command

Phase 23 adds `open_external_access`. It accepts one tagged publisher URL,
institutional URL, DOI, or Google Scholar query. Rust validates or constructs
the final HTTPS URL and delegates one launch to the default system browser.

The response reports only `opened` and the destination. Typed errors
distinguish an invalid URL, invalid DOI, invalid search query, and unavailable
browser. No URL, browser detail, credential, cookie, token, or session state
returns to the frontend. The full boundary is documented in
`docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`.

## Formatting review command

Phase 34 adds `run_formatting_review`. It accepts one bounded immutable
snapshot containing a closed style identifier, ordered heading levels and
titles, and validated citation citekeys and declared styles. Unknown fields,
invalid identifiers, invalid heading values, and collection limits fail with
one of six closed content-free codes.

The response contains Rust-owned finding codes, severities, indexed targets,
fixed wording, and ordered actions. Heading findings may include one bounded
`apply_heading_level`; citation findings never do. The command receives no
path, document identity, reference record, credential, or export target and
performs no persistence, filesystem, network, Python, worker, or export work.
The complete integration is documented in
`docs/maintainers/FORMATTING_UX.md`.

## Connectivity mode commands

Phase 36 adds `get_connectivity_mode` and `set_connectivity_mode`. The read
command accepts an empty request. The set command accepts only `online` or
`offline`. Both return the effective Rust-owned mode and expose only
`connectivity_unavailable` when shared state cannot be read.

The commands change process-local session policy only. They accept no URL,
provider, proxy, credential, retry instruction, or persistence option. The
complete behavior is documented in `docs/maintainers/OFFLINE_MODE.md`.

## Ownership layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `run` | Registers commands and starts the Tauri runtime. |
| Mid | `get_runtime_status` | Coordinates domain status construction and maps the result to command DTOs. |
| Mid | `cancel_worker` | Validates the worker ID and maps cancellation outcomes to command DTOs. |
| Mid | `open_document` | Selects a file in Rust and delegates validated loading. |
| Mid | `save_document` | Accepts an explicit snapshot and delegates atomic persistence. |
| Mid | `resolve_citation` | Validates attrs and delegates local reference resolution. |
| Mid | `open_external_access` | Validates a research destination and delegates one system-browser launch. |
| Mid | `run_formatting_review` | Validates one bounded snapshot and maps pure findings to closed actions. |
| Mid | `get_connectivity_mode` | Returns the effective Rust-owned session mode. |
| Mid | `set_connectivity_mode` | Applies one closed process-local mode. |
| Mid | `current_runtime_status` | Builds Rust-owned application status from compiled metadata. |
| Mid | `WorkerCancellationRegistry` | Owns transient worker identity and cancellation state. |
| Low | `validated_version` | Normalizes and rejects an empty package version. |

The command module owns only Tauri-facing DTOs and error mapping. The
application module owns status construction. This keeps Tauri mechanics out of
domain logic and prevents React from becoming authoritative for runtime state.

## Required command shape

Every Tauri command added after this checkpoint must have:

- a bounded request DTO that rejects unknown fields
- a concrete serialized response type
- a command-specific serialized error enum
- a thin handler that calls Rust-owned domain logic
- explicit registration in `tauri::generate_handler!`
- a compile-time signature test named `command_signature_is_typed`
- a request test named `request_deserialization_is_stable`
- a response test named `response_serialization_is_stable`
- an error test named `error_serialization_is_stable`

Commands must not hide durable side effects. A response or documented event
must make the result observable to the frontend.

`get_runtime_status` emits the finite event documented in
`docs/maintainers/EVENT_BOUNDARY.md` before returning its response.

## Enforcement

Rust tests cover valid and blank version inputs, exact command signatures,
bounded request deserialization, stable JSON for success and error values,
cancellation lifecycle outcomes, Phase 13 document commands, Phase 14
atomic-write failure shapes, Phase 18 citation resolution, and Phase 23
external browser handoff. Phase 34 adds the same signature, request, response,
error, and rejected-content evidence for formatting review.
Phase 36 adds two complete command-contract sets for connectivity get/set and
extends external access with typed offline policy failures.

`scripts/check-invariants.sh` rejects generic Rust error patterns and compares
the number of Tauri commands with registered handlers, typed signature tests,
request deserialization tests, response serialization tests, and error
serialization tests. The aggregate local verifier and GitHub Actions run that
same scan.

Run the focused Rust evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
bash scripts/check-invariants.sh
```

## Related boundaries

- Phase 7 establishes the TypeScript wrapper pattern described in
  `docs/maintainers/FRONTEND_COMMAND_CLIENT.md`.
- Phase 8 establishes the typed event path described in
  `docs/maintainers/EVENT_BOUNDARY.md`.
- Phase 9 establishes worker cancellation behavior described in
  `docs/maintainers/CANCELLATION_BOUNDARY.md`.
- Phases 13 and 14 establish document file behavior described in
  `docs/maintainers/DOCUMENT_SAVE_LOAD.md`.
- Phase 18 establishes citation validation and resolution described in
  `docs/maintainers/CITATION_NODE.md`.
- Phase 23 establishes Rust-owned browser handoff described in
  `docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`.
- Phase 34 establishes the bounded formatting review command described in
  `docs/maintainers/FORMATTING_UX.md`.
- Phase 36 establishes the session connectivity commands described in
  `docs/maintainers/OFFLINE_MODE.md`.
- Product commands are introduced only in their owning phases with their
  domain models and negative-path tests.
