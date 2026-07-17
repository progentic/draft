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
  "buildCommit": "0123456789abcdef0123456789abcdef01234567",
  "buildProfile": "release",
  "version": "0.1.0"
}
```

The response version comes from Rust package metadata. Packaging supplies the
exact clean-worktree Git commit, and Cargo supplies the closed `debug` or
`release` profile. A local non-package build uses `development` as its commit
marker. The frontend does not supply or decide any of these values.

The command-specific error codes are:

```json
{ "code": "invalid_application_version" }
{ "code": "invalid_build_metadata" }
{ "code": "event_delivery_failed" }
```

The delivery error is returned when the documented Phase 8 runtime-status
event cannot reach current frontend listeners.

No generic string, `anyhow::Error`, `serde_json::Value`, or boxed error crosses
the IPC boundary.

## macOS application-open command

Phase 47 adds `open_application_document` for `.draft` files delivered by the
macOS application run loop. The request contains only the closed `open` or
`dismiss` disposition. It never contains a path. Rust consumes one queued path,
routes it through the existing document-open implementation, and returns the
same typed open outcome nested inside `opened`. `none` and `dismissed` are
successful no-mutation outcomes.

The path-free `draft://application-open` event tells React only that a request
is available. Typed errors distinguish an unavailable queue, multiple files,
an unsupported file URL, and the existing bounded document-open causes. The
frontend can preserve unsaved-work decisions without receiving source-path
authority.

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
one untrusted `snapshot`, a closed `save` or `save_as` mode, a basename display
name, and the closed lifecycle origin previously returned by Rust. Rust
validates the envelope and derives the native `.draft` filename suggestion
before registry or filesystem work. Normal Save selects a target only when none
is registered. Save As always selects a replacement target and preserves the
prior file.

Both commands return typed opened/saved/cancelled responses and bounded nested
errors. Phase 14 adds typed atomic-write stages and a distinct
`durability_uncertain` failure after a complete replacement whose parent sync
fails. The commands expose no path field to the frontend. The full lifecycle
and atomic-write behavior are documented in
`docs/maintainers/DOCUMENT_SAVE_LOAD.md`.

## Native window-title command

`set_window_title` accepts only an optional basename display name and one
transient Unsaved boolean. Rust rejects blank, path-like, control-character, or
oversized names before calling the native window API. It returns only
`{ "applied": true }`; typed errors distinguish invalid title input from a
platform update failure.

The command mirrors presentation state. It cannot receive a filesystem path,
change a save target, register a document, or persist content. Clean,
modified, imported, and new title formats are pinned in Rust and frontend
interaction tests.

## Native menu state command

Phase 48 adds `set_native_menu_state`. It accepts exactly seven booleans for
New, Open, Close, Save, Save As, Save Back to Source, and DOCX export
availability. The request contains no path, content, document ID, or arbitrary
action name. Rust applies the state to the menu items it created and returns
`{ "applied": true }` or the closed `menu_update_failed` error.

Native selection travels in the other direction as a bounded
`draft://native-menu-action` event. The event is not trusted until the typed
frontend wrapper validates its one action field. Both menu and toolbar actions
then use `useWorkspaceActions`; the state command performs no document work.

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

## Diagnostic snapshot command

Phase 38 adds `get_diagnostic_snapshot`. It accepts an exact empty request and
returns one strict schema-versioned snapshot containing only fixed support
metadata. The command performs no probe, I/O, persistence, collection, event,
or background work.

Three closed errors distinguish an invalid compiled application version,
serialization failure, and the fixed snapshot byte limit. No raw detail enters
the response. The complete boundary is documented in
`docs/maintainers/AUDIT_DIAGNOSTICS.md`.

## Phase 46 visible-workflow commands

Phase 46 adds six typed commands for existing Rust-owned capabilities:

- `create_document` generates and validates one Rust-identified initial
  envelope without persistence or a registry entry;
- `close_document` releases one live registry handle;
- `add_reference` validates and stores one manual reference, then returns a
  bounded summary;
- `list_references` returns bounded reference summaries without stored payloads;
- `run_text_analysis` validates one immutable text snapshot and returns the
  five accepted local heuristic finding types; and
- `export_document` validates the current envelope, asks Rust for a DOCX target,
  and delegates to the existing atomic exporter.

The commands expose no filesystem path, database row, full reference payload,
Python process configuration, helper stderr, or export implementation detail.
The complete visible lifecycle and recovery contract is documented in
`docs/maintainers/PHASE46_WORKFLOWS.md`.

Phase 46 also extends the existing `open_document` outcome without adding a
path-bearing request or response. `opened_draft` identifies a registered native
document, `imported_text` identifies a new unsaved envelope derived from
bounded UTF-8 text, and `cancelled` preserves the active session. Imported
titles contain a filename only and never become save authority.

Phase 47 adds `imported_external` to the same command. Rust reads and validates
one selected DOCX, registers its source provenance without a native save
target, and returns only a canonical envelope plus path-free fidelity summary.
Malformed, unsafe, unsupported, and lossy packages return distinct nested
typed failures. No DOCX package, relationship, XML, path, or fingerprint crosses
the command boundary. See `docs/maintainers/DOCX_INTEROPERABILITY.md`.

Phase 47 also adds the separate `save_external_document` command. It accepts
one current envelope and one closed decision: exact save, accepted canonical
normalization, or cancellation. Rust obtains the external source only from the
registry, checks its fingerprint before and after compilation, and returns a
saved, unchanged, confirmation-required, denied, or cancelled outcome. Errors
distinguish invalid input, registry/read/compilation/write failures, successful
rollback, and failed rollback. Responses contain no source path, fingerprint,
package bytes, or operating-system detail. The strict frontend wrapper exists
for contract validation but no component or hook invokes it yet.

## Ownership layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `run` | Registers commands and starts the Tauri runtime. |
| Mid | `get_runtime_status` | Coordinates domain status construction and maps the result to command DTOs. |
| Mid | `cancel_worker` | Validates the worker ID and maps cancellation outcomes to command DTOs. |
| Mid | `open_document` | Selects a file in Rust and delegates validated loading. |
| Mid | `save_document` | Accepts an explicit snapshot and delegates atomic persistence. |
| Mid | `set_window_title` | Applies validated basename and Unsaved presentation to the native window. |
| Mid | `set_native_menu_state` | Applies path-free transient availability to Rust-owned native items. |
| Mid | `resolve_citation` | Validates attrs and delegates local reference resolution. |
| Mid | `open_external_access` | Validates a research destination and delegates one system-browser launch. |
| Mid | `run_formatting_review` | Validates one bounded snapshot and maps pure findings to closed actions. |
| Mid | `get_connectivity_mode` | Returns the effective Rust-owned session mode. |
| Mid | `set_connectivity_mode` | Applies one closed process-local mode. |
| Mid | `get_diagnostic_snapshot` | Maps one explicit request to bounded local support metadata. |
| Mid | `create_document` | Generates one validated Rust-owned initial envelope. |
| Mid | `close_document` | Releases the current Rust-owned document handle. |
| Mid | `add_reference` | Validates and persists one manual reference through the managed store. |
| Mid | `list_references` | Returns bounded summaries from the managed reference store. |
| Mid | `run_text_analysis` | Validates one snapshot and delegates to the fixed packaged local helper. |
| Mid | `export_document` | Selects a DOCX target in Rust and delegates atomic export. |
| Mid | `save_external_document` | Applies the closed same-format policy to one Rust-owned external source. |
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
Phase 38 adds the same four command-contract tests for the local diagnostic
snapshot and exercises its closed domain failures separately.
Phase 46 adds six complete command-contract sets plus domain, lifecycle,
packaged-helper, and source-preservation evidence for the visible workflows.

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
- Phase 38 establishes the local snapshot command described in
  `docs/maintainers/AUDIT_DIAGNOSTICS.md`.
- Phase 46 establishes the visible document, reference, local text-check, and
  DOCX command paths described in `docs/maintainers/PHASE46_WORKFLOWS.md`.
- Product commands are introduced only in their owning phases with their
  domain models and negative-path tests.
