# Document Save and Load

## Status

This is an implemented Phase 13/14 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. It implements existing ownership and atomic-save rules without changing the
architecture or invariants.

## Scope

Phase 13 adds typed Rust `open_document` and `save_document` commands plus
frontend request/response wrappers. Phase 14 hardens the writer against
interruption, replacement failure, cleanup failure, and concurrent saves.
Phase 46 adds `create_document` and `close_document`, then connects the complete
bounded lifecycle to the visible workspace. Phase 47 adds a bounded DOCX import
outcome and Rust-owned external-source registration.

These phases do not add:

- direct frontend dialog or filesystem access
- a frontend-provided filesystem path
- implicit reads from the live Tiptap instance
- autosave or crash recovery
- reference records, CSL JSON, or citation metadata in the envelope
- export, migration, network, analysis, or formatting behavior

## Open Command

`open_document` receives an empty request. Rust opens the native file picker
through `tauri-plugin-dialog` with `.draft`, compatible legacy `.json`, `.txt`,
`.md`, and `.docx` filters. The frontend cannot choose or inspect a path through
IPC.

The command and dialog adapter are asynchronous. The adapter parents the
picker to the main window, uses the plugin callback API, and awaits a one-shot
typed result. It never uses a blocking dialog call on the application thread.

For `.draft` and `.json`, Rust:

1. Reads the selected bytes.
2. Parses JSON.
3. Migrates persisted version 1 input when required, then validates the current version 2 envelope through `DocumentEnvelope`.
4. Rejects unsupported schema versions explicitly.
5. Registers the envelope and source path as one live handle.
6. Returns the validated envelope or a cancellation response.

Malformed, invalid, or duplicate files never replace an existing registry
entry.

The macOS package registers `.draft` as `com.progentic.draft.document`. When
Finder or another desktop surface asks DRAFT to open one file, the Rust run
loop places its file URL in a private queue and emits only a path-free
availability event. React applies the existing unsaved-work policy, then asks
Rust to open or dismiss that queued request. Rust converts and consumes the
path and reuses the same validation, migration, registration, and typed outcome
as the native Open dialog. Multiple URLs and non-file URLs fail closed.

For `.txt` and `.md`, Rust reads at most 8 MiB of UTF-8, creates a new validated
unsaved envelope, and returns `imported_text` with the closed `plain_text` or
`markdown` format. Every LF-delimited line becomes one paragraph; a terminal
CR is removed so CRLF input behaves consistently. Markdown punctuation remains
literal text. The response title contains only the source filename. Rust does
not register or return the source path, and the source bytes are never changed.
Invalid UTF-8, oversized input, unreadable files, and unsupported extensions
fail before registration or persistence.

For `.docx`, Rust validates a bounded ZIP/XML package and converts only the
accepted paragraph subset. Validation, fidelity classification, and canonical
envelope construction finish before registry mutation. A successful response
is `imported_external` with basename-only display data. Rust registers the live
external source and retains its path and fingerprints, but reports no native
save target. Opening, closing, cancellation, and failed import leave the source
bytes unchanged. See `docs/maintainers/DOCX_INTEROPERABILITY.md`.

## Create And Close Commands

`create_document` accepts an exact empty request. Rust generates the UUID and
validates the fixed initial envelope before returning it. The command does not
open a path, persist the document, or register a live handle. React never
generates a durable document identity.

`close_document` accepts one Rust-issued document ID and releases its live
registry handle. Closing an unsaved document requires no Rust mutation because
it has no registered source path. New, Open, and Close protect edited content
through the visible unsaved-changes dialog before replacement.

The visible lifecycle is explicit:

1. Rust creates a validated envelope with an unsaved identity and one empty paragraph.
2. The frontend edits that envelope as transient content.
3. The first successful Save selects a path and establishes the durable handle.
4. Later saves update the same registered document.
5. Close releases the active handle; an unsaved document has no handle to release.

The frontend records one explicit origin: `new`, `imported_text`,
`imported_external`, or `opened_draft`. It separately tracks whether Rust owns
a live registration and whether a native DRAFT save succeeded. A Rust-owned ID
is in-memory identity, not proof of persistence. `opened_draft` has a native
save target. `imported_external` has a Rust registration but no native target.
Successful first Save transitions an imported or new origin to
`opened_draft`; cancellation preserves the prior origin and visible state.

## Save Command

`save_document` receives one explicit `snapshot` JSON value. Rust never reaches
into the WebView or Tiptap instance for live state. The visible frontend client
serializes the current editor state and places it in the envelope before
invoking the command.

Rust validates the entire snapshot before opening a dialog or writing. A known
document reuses the source path retained by the registry. An unsaved or new
document invokes the Rust native save dialog and attaches the selected path to
its live handle. Rust rejects a path already owned by another live document.
Cancellation and failed writes do not register, replace, or attach the
document snapshot.

A newly selected target must use the `.draft` extension. Rust enforces this
before writer invocation; the native dialog filter is not treated as the
authority. Existing compatible `.json` documents may continue saving only to
their already registered path.

A read-only preflight decides whether a target dialog is required. The actual
save path validates again while holding the existing lifecycle lock, so the
preflight does not grant registry or filesystem authority. First-save target
selection uses the same asynchronous callback contract as Open and Export.

Unknown top-level envelope fields remain invalid, so save cannot add reference
records, embedded citation metadata, analysis output, or export state.
Canonical font-family and whole-point font-size marks are validated nested
document content, not new top-level envelope fields.

Phase 47 makes document envelope version 2 current. A persisted version 1 file
passes through the separate migration boundary and enters the registry as
canonical version 2 state without changing source bytes. Direct snapshots sent
to Save must already be version 2. The first successful save of a migrated
document atomically writes version 2; opening or failed saving never rewrites
the legacy source.

Canonical paragraph data is optional nested block state. Rust validates it
before path selection or writing. Absence means document defaults; migration
does not fabricate a complete default object.

## Atomic Replacement

All document writes route through `src-tauri/src/documents/atomic_write.rs`.
The wrapper uses `tempfile` to create an owned temporary file in the target
directory. DRAFT writes the complete serialized envelope, calls `sync_all`,
atomically persists the temporary file over the target, and synchronizes the
parent directory on Unix. `tempfile` uses the platform replacement primitive,
including replace-existing behavior on Windows.

The writer explicitly closes a temporary file after every detected failure
before replacement. A real failed persist returns ownership of the temporary
file, allowing the same cleanup path to run. Deterministic checkpoints cover
failure before writing, after a partial write, before content sync, before
replacement, and before parent-directory sync.

The registry serializes open/save lifecycle operations with one process-local
lock. Registry mutation occurs after successful replacement while that lock is
held, preventing concurrent saves from reordering the disk and registry
snapshots. Failures before replacement leave both snapshots unchanged. If
parent-directory sync fails after replacement, Rust advances the registry to
the new complete source and returns `durability_uncertain`.

Feature and command code do not call `File::create`, `fs::write`, or
`OpenOptions::new` for document targets. The invariant scan rejects those
direct-write patterns outside the writer boundary.

The platform replacement test runs on every Rust test host. Current GitHub
Actions coverage is Ubuntu, while local verification on macOS exercises the
Unix implementation. Supported-platform package validation remains a Phase 42
release gate.

## Failure Shape

Open errors are typed as:

- `unsupported_file_location`
- `unsupported_file_type`
- `file_not_found`
- `read_failed`
- `malformed_json`
- `invalid_text_encoding`
- `text_too_large`
- `external_import` with a typed DOCX/package `cause`
- `invalid_envelope` with a typed envelope `cause`
- `registry` with a typed registry `cause`

Save errors are typed as:

- `unsupported_file_location`
- `invalid_target` when a new target does not end in `.draft`
- `invalid_envelope` with a typed envelope `cause`
- `serialization_failed`
- `registry` with a typed registry `cause`
- `write_failed` with a typed atomic-write `cause`
- `durability_uncertain` after complete replacement but failed parent sync

Atomic-write causes distinguish temporary-file open, write, content sync,
target replacement, temporary cleanup, and parent-directory sync stages. Raw
paths and operating-system errors never cross IPC.

User cancellation is a successful `cancelled` response, not an error.

Open success is also explicit: `opened_draft` means Rust retained a native
save target; `imported_text` means the returned envelope has no registration or
target; `imported_external` means Rust retained source provenance but granted
no native `.draft` save target; `cancelled` means no session state changed. An
independent same-format DOCX command may use the external registration only
when its closed fidelity and fingerprint rules allow it. The visible Save Back
workflow requests typed eligibility and explicit confirmation; it never turns
the external source into a native `.draft` target.

Save requests include a closed mode: `save` or `save_as`. Normal Save reuses an
existing Rust-owned target and selects a target only for a new or imported
document. Save As always opens the Rust-owned save panel. A successful Save As
writes the selected `.draft` file atomically, preserves the old file, rebinds
the registry to the new target, and returns only the document ID, basename
display name, and `wasSaveAs: true`. Cancellation or failure leaves the current
target, display name, and dirty state unchanged.

## Frontend Boundary

`src/ipc/documentCreate.ts`, `documentOpen.ts`, `documentSave.ts`, and
`documentClose.ts` are the frontend lifecycle wrappers. `applicationOpen.ts`
adds only a path-free request signal and closed open/dismiss decision for macOS
document activation.
`src/ipc/documentEnvelope.ts` mirrors the Rust envelope for response validation
and request typing. Rust remains the identity and validation authority.
`src/documents/paragraphFormatting.ts` mirrors only the accepted paragraph
shape and bounds for early feedback.

No frontend source imports `@tauri-apps/plugin-dialog`,
`@tauri-apps/plugin-fs`, Node filesystem APIs, or another direct file surface.
The save wrapper accepts a snapshot, not a path.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | create/open/save/close commands | Coordinate one typed IPC request. |
| Mid | persistence `open_document` / `save_document` / `save_document_as` | Enforce validation, registry, and lifecycle policy. |
| Mid | `save_external_document` | Enforce same-format eligibility, source identity, atomic replacement, and rollback. |
| Mid | `DocumentRegistry` | Own one handle, source path, and current snapshot. |
| Low | dialog helpers / JSON / atomic writer | Perform native API, parsing, and filesystem mechanics. |

## Verification

Rust tests cover command serialization, complete atomic create and platform
replacement, missing parents, deterministic interrupted writes, real failed
replacement, temporary cleanup, post-replacement durability uncertainty,
concurrent disk/registry consistency, malformed JSON, unsupported schema
versions, duplicate load, explicit snapshot save, first-save path selection,
cancellation, retained paths, path ownership conflicts, failed-write registry
rollback, and save-close-reopen round trips. Registry tests pin every serialized
nested error code used by document commands.

Phase 46 adds async-dialog and font-format coverage: no blocking native dialog
API remains; preflight behavior matches registry state; malformed font marks
fail before target selection; and valid family and size marks persist through
first save, close, and reopen.

Frontend tests cover envelope mirroring, exact command arguments, opened/saved/
cancelled responses, malformed responses, registry failures, typed write-stage
failures, durability uncertainty, citation-node causes, and transport
classification. Phase 18 adds open/save tests proving malformed citation attrs
fail before registry insertion or path selection.

Phase 41 adds crate-level evidence that first save, retained-path update,
close, reopen, duplicate-open rejection, citation resolution, and export use
these production paths together. Phase 46 adds the visible lifecycle without
changing those persistence paths.

`scripts/check-invariants.sh` requires these tests and sources, checks command
name parity, rejects frontend path/dialog authority, and rejects direct target
writes. The same script runs through `scripts/verify.sh` locally and in the
GitHub Actions `verify` job.

## Phase 15 Audit

Phase 15 audits this guide, `ARCHITECTURE.md`, `INVARIANTS.md`, the document
envelope and registry guides, user workspace claims, local verification, and
GitHub Actions. The evidence is recorded in
`docs/maintainers/REALIGNMENT.md`.

Phase 18 validates citation nodes nested inside the existing envelope without
adding top-level fields or changing envelope version 1. The implementation is
documented in `docs/maintainers/CITATION_NODE.md`. Reference metadata,
bibliography behavior, network lookup, and imports remain outside the document
file contract.

## Configuration Index

Document extensions, the default save name, and schema versions are indexed in
`docs/maintainers/CONFIGURATION.md`.
