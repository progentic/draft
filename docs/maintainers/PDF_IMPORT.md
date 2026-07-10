# PDF Import

## Status

This guide records implemented Phase 24 behavior. The requirements in
`docs/drafts/PDF_IMPORT.md` remain non-binding until they complete the contract
lifecycle in `docs/GOVERNANCE.md`.

## Scope

`src-tauri/src/imports/pdf.rs` prepares Rust-only `PendingPdfImport` candidates
from an explicit file path or a watched-folder observation. A candidate carries
a Rust-generated UUID, source provenance, private canonical path, and byte
length.

A candidate is not a persistent job and starts no processing. Phase 24 adds no
Tauri command, file dialog, visible UI, watcher dependency or thread, database
row, PDF parser, text extraction, file copy, or reference mutation.

## Explicit Intake

`prepare_explicit_pdf` accepts one path already approved by a future Rust-owned
selection flow. It requires a case-insensitive `.pdf` extension, existing
regular file, non-symlink final path, canonicalization, stable metadata while
reading, and a `%PDF-` signature.

The function reads only the five-byte signature. It does not read or retain the
PDF body.

## Watched Intake

`WatchedPdfIntake` is rooted to one existing canonical non-symlink directory.
Every candidate path is canonicalized and must remain under that root. A final
symlink fails before containment is evaluated; a parent symlink that escapes
the root fails the canonical containment check.

The intake receives observations from future Rust watcher orchestration:

1. `record_change` captures the current byte length and monotonic event time.
2. Every event replaces the observation and resets the deadline.
3. `confirm_stable` returns `Waiting` before one quiet second has elapsed.
4. A changed byte length at confirmation records a new observation and returns
   `Waiting`.
5. After the quiet interval, Rust reads the PDF signature and rechecks the file
   length on the same open handle.
6. Only an unchanged valid PDF returns `Pending`.

Tests inject `Instant` values. They do not sleep and do not rely on wall-clock
timing.

## Candidate Boundary

`PdfImportSource` distinguishes `Explicit` and `WatchedFolder`. The canonical
path remains private to Rust and is not serializable. `PdfImportId` is generated
with UUID v4 so a later persistent job can adopt an opaque Rust-owned identity.

No background work begins at candidate creation. Phase 26 must persist a job
before parsing, metadata resolution, copying, or other resumable work starts.

## Failure Shape

`PdfImportError` distinguishes:

- unsupported file type;
- unavailable file;
- non-regular file;
- symbolic link;
- metadata failure;
- read failure;
- invalid PDF signature;
- file change during explicit validation;
- invalid watched folder;
- path outside watched folder;
- missing change observation; and
- out-of-order observation time.

Display messages do not include paths, file contents, imported metadata, or raw
OS errors.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `prepare_explicit_pdf` | Produces one explicit pending candidate. |
| High | `WatchedPdfIntake::confirm_stable` | Coordinates quiet-time, size, and signature gates. |
| Mid | `observation_is_ready` | Applies monotonic debounce and stable-size policy. |
| Mid | `inspect_pdf` | Confirms a stable signature read on one file handle. |
| Low | path and metadata helpers | Enforce extension, file type, symlink, canonical path, and root rules. |

## Verification

Six Rust tests cover explicit intake, malformed files, final symlinks, chunked
writes, debounce, an unreported size change, watched-root escape, and typed
observation failures. The required chunked-write test records events while the
fake PDF is partial, proves intake remains waiting, stops writing, and proves
the file becomes pending only after the full quiet interval.

`scripts/check-invariants.sh` requires the source, tests, named debounce,
signature, symlink check, and root containment. It rejects import IPC,
frontend authority, persistence, network access, file mutation, watcher
dependencies, and unmanaged thread/task spawning. Repository and documentation
checks require every Phase 24 source and guide.

## Known Limitations

Phase 24 does not subscribe to operating-system filesystem events. It provides
the Rust-owned intake called by that future lifecycle. It also does not persist
candidates across process restart, parse PDF contents, or expose an import
workflow. Those behaviors must not be inferred from the `Pending` name.

## Next Boundary

Phase 25 is the mandatory documentation and drift realignment for the complete
research boundary through PDF intake. Phase 26 may then add the persistent job
state machine.
