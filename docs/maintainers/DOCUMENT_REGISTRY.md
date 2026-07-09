# Document Registry

## Status

This is an implemented Phase 12 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. It implements the ownership rule in `ARCHITECTURE.md` section 6 and
`INVARIANTS.md` `INV-06` without changing either decision.

## Scope

Phase 12 provides one process-local Rust registry for live document handles.
It accepts only a `DocumentEnvelope` that already passed Phase 11 validation.
The Tauri runtime manages the registry as application state, but no command or
frontend code can open or close a document yet.

The registry does not:

- create or generate documents
- read, write, save, reload, or autosave files
- accept filesystem paths or open native dialogs
- persist handles across process restarts
- create Tiptap instances or track frontend view identity
- focus a window or tab
- export, migrate, cite, analyze, or format content

Those capabilities remain assigned to later phases.

## Ownership Model

`src-tauri/src/documents/registry.rs` defines `DocumentRegistry`. The runtime
creates one instance in `src-tauri/src/lib.rs`.

Each active entry is keyed by its Rust-validated `DocumentId` and stores one
private `LiveDocumentHandle`. The handle owns the validated in-memory envelope
until close. No caller can clone or construct the private handle directly.

An open operation follows this sequence:

1. Lock the registry.
2. Read the validated document ID from the envelope.
3. Reject an occupied ID with `AlreadyOpen`.
4. Insert one private handle for a vacant ID.

A close operation removes the handle by document ID and returns its envelope.
Returning the envelope is an in-memory ownership transfer, not a save.

## Duplicate Behavior

Phase 12 chooses `AlreadyOpen` rather than focusing an existing view because no
frontend view identity or document command exists yet. A later command may map
this result to focus behavior without weakening the registry rule.

Duplicate rejection never replaces the active envelope. This prevents a
second request from silently changing the document state protected by the
first live handle.

## Concurrency

A mutex guards the complete check-and-insert operation. Concurrent requests for
the same `DocumentId` therefore produce exactly one success and one
`AlreadyOpen` result. Distinct document IDs may both remain open in the same
registry.

The mutex is process-local coordination only. It is not persistence, a file
lock, a cross-process lock, or a multi-user synchronization model.

## Failure Shape

`DocumentRegistryError` is a bounded Rust domain enum:

| Failure | Meaning |
| :--- | :--- |
| `AlreadyOpen` | The document ID already owns one live handle. |
| `NotOpen` | A close request named a document with no live handle. |
| `SourcePathInUse` | Another live document already owns the selected source path. |
| `RegistryUnavailable` | The registry mutex is poisoned and state cannot be trusted. |

These errors did not cross IPC in Phase 12. Phase 13 open/save commands now map
them as the typed nested `cause` of command-specific registry failures rather
than returning an untyped blob.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `DocumentRegistry::open` / `close` | Coordinate one registry operation. |
| Mid | `register_handle` / `remove_handle` | Enforce live-handle lifecycle rules. |
| Low | `HashMap::entry` / `Mutex::lock` | Perform synchronized collection access. |

## Verification

Focused Rust tests prove:

- opening the same document twice returns `AlreadyOpen`
- a rejected duplicate does not replace the active envelope
- closing releases the handle and permits reopening
- closing an unknown document returns `NotOpen`
- distinct documents open independently
- concurrent same-document opens produce one live handle
- a poisoned registry returns `RegistryUnavailable`

`scripts/check-invariants.sh` requires those tests, verifies runtime state
registration, and rejects filesystem or Tauri command APIs in the registry
module. `scripts/check-repository.sh` requires the source file to remain visible
to Git. Both checks run through `scripts/verify.sh` locally and in the GitHub
Actions `verify` job.

## Save/Load Integration

Phase 13 retains Rust-selected source paths inside each live handle. Validated
loads call `open_from_path`; saves replace the existing snapshot while retaining
or attaching that path; close and reopen still release and recreate exactly one
handle. One path cannot back two live document IDs, and failed writes do not
advance the handle snapshot or attach a path. Phase 14 adds a registry-owned
file-operation lock around open/save lifecycle coordination. This serializes
disk replacement with the corresponding handle update so concurrent saves
cannot leave the registry and source file on different snapshots.
