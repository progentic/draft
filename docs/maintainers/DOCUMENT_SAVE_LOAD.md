# Document Save and Load

## Status

This is an implemented Phase 13/14 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. It implements existing ownership and atomic-save rules without changing the
architecture or invariants.

## Scope

Phase 13 adds typed Rust `open_document` and `save_document` commands plus
frontend request/response wrappers. Phase 14 hardens the writer against
interruption, replacement failure, cleanup failure, and concurrent saves. The
visible workspace does not invoke these commands yet, so this remains a
protected backend lifecycle rather than a completed user workflow.

These phases do not add:

- direct frontend dialog or filesystem access
- a frontend-provided filesystem path
- implicit reads from the live Tiptap instance
- autosave, recovery, or a close command
- reference records, CSL JSON, or citation metadata in the envelope
- export, migration, network, analysis, or formatting behavior

## Open Command

`open_document` receives an empty request. Rust opens the native file picker
through `tauri-plugin-dialog` with `.draft` and `.json` filters. The frontend
cannot choose or inspect a path through IPC.

After selection, Rust:

1. Reads the selected bytes.
2. Parses JSON.
3. Validates the version 1 envelope through `DocumentEnvelope`.
4. Rejects unsupported schema versions explicitly.
5. Registers the envelope and source path as one live handle.
6. Returns the validated envelope or a cancellation response.

Malformed, invalid, or duplicate files never replace an existing registry
entry.

## Save Command

`save_document` receives one explicit `snapshot` JSON value. Rust never reaches
into the WebView or Tiptap instance for live state. The normal future caller
must serialize the current editor state and place it in the envelope before
invoking the command.

Rust validates the entire snapshot before opening a dialog or writing. A known
document reuses the source path retained by the registry. An unsaved or new
document invokes the Rust native save dialog and attaches the selected path to
its live handle. Rust rejects a path already owned by another live document.
Cancellation and failed writes do not register, replace, or attach the
document snapshot.

Unknown top-level envelope fields remain invalid, so save cannot add reference
records, embedded citation metadata, analysis output, or export state.

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
- `file_not_found`
- `read_failed`
- `malformed_json`
- `invalid_envelope` with a typed envelope `cause`
- `registry` with a typed registry `cause`

Save errors are typed as:

- `unsupported_file_location`
- `invalid_envelope` with a typed envelope `cause`
- `serialization_failed`
- `registry` with a typed registry `cause`
- `write_failed` with a typed atomic-write `cause`
- `durability_uncertain` after complete replacement but failed parent sync

Atomic-write causes distinguish temporary-file open, write, content sync,
target replacement, temporary cleanup, and parent-directory sync stages. Raw
paths and operating-system errors never cross IPC.

User cancellation is a successful `cancelled` response, not an error.

## Frontend Boundary

`src/ipc/documentOpen.ts` and `src/ipc/documentSave.ts` are the only frontend
command wrappers. `src/ipc/documentEnvelope.ts` mirrors the Rust envelope for
response validation and request typing. Rust remains the validation authority.

No frontend source imports `@tauri-apps/plugin-dialog`,
`@tauri-apps/plugin-fs`, Node filesystem APIs, or another direct file surface.
The save wrapper accepts a snapshot, not a path.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `open_document` / `save_document` commands | Coordinate one typed IPC request. |
| Mid | persistence `open_document` / `save_document` | Enforce validation, registry, and lifecycle policy. |
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

Frontend tests cover envelope mirroring, exact command arguments, opened/saved/
cancelled responses, malformed responses, registry failures, typed write-stage
failures, durability uncertainty, and transport classification.

`scripts/check-invariants.sh` requires these tests and sources, checks command
name parity, rejects frontend path/dialog authority, and rejects direct target
writes. The same script runs through `scripts/verify.sh` locally and in the
GitHub Actions `verify` job.

## Phase 15 Gate

Phase 15 must audit this implementation guide, `ARCHITECTURE.md`, `INVARIANTS.md`,
the document envelope and registry guides, user workspace claims, local
verification, and GitHub Actions before reference-schema work begins.
