# Document Save and Load

## Status

This is an implemented Phase 13 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. It implements existing ownership and atomic-save rules without changing the
architecture or invariants.

## Scope

Phase 13 adds typed Rust `open_document` and `save_document` commands plus
frontend request/response wrappers. The visible workspace does not invoke them
yet, so this is a protected backend lifecycle rather than a completed user
workflow.

The phase includes the minimum atomic replacement primitive because any real
write-to-target implementation would violate `INV-09`. Phase 14 remains
mandatory for interruption and replacement-failure hardening.

Phase 13 does not add:

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

All Phase 13 writes route through
`src-tauri/src/documents/atomic_write.rs`. The wrapper uses
`atomic-write-file` to create a temporary file in the target directory. DRAFT
writes all bytes, calls `sync_all`, then commits the temporary file. Commit
renames the complete temporary file over the target and synchronizes the
parent directory on supported platforms.

Registry mutation occurs only after the atomic writer succeeds. If opening,
writing, syncing, or committing the replacement fails, the prior registry
snapshot and source-path ownership remain unchanged.

Feature and command code do not call `File::create`, `fs::write`, or
`OpenOptions::new` for document targets. The invariant scan rejects those
direct-write patterns outside the writer boundary.

The current tests prove complete creation and replacement. Phase 14 must still
prove that failures before sync, before commit, during replacement, and during
concurrent saves leave the prior complete document intact and clean up
temporary files.

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
- `write_failed`

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

Rust tests cover command serialization, complete atomic create/replace, missing
parents, malformed JSON, unsupported schema versions, duplicate load,
explicit snapshot save, first-save path selection, cancellation, retained
paths, path ownership conflicts, failed-write registry rollback, and
save-close-reopen round trips. Registry tests pin every serialized nested error
code used by document commands.

Frontend tests cover envelope mirroring, exact command arguments, opened/saved/
cancelled responses, malformed responses, nested typed failures, and transport
classification.

`scripts/check-invariants.sh` requires these tests and sources, checks command
name parity, rejects frontend path/dialog authority, and rejects direct target
writes. The same script runs through `scripts/verify.sh` locally and in the
GitHub Actions `verify` job.

## Phase 14 Gate

Phase 14 is not optional cleanup. It must add deterministic fault injection and
tests for interrupted writes, failed replacement, temporary-file cleanup,
concurrent saves, and platform replacement behavior before atomic save is
considered hardened.
