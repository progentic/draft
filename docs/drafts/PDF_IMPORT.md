# PDF Import Requirements Draft

## Status

This is a non-binding Phase 24 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/PDF_IMPORT.md`. This draft does not
become an accepted contract without the lifecycle in `docs/GOVERNANCE.md`.

## Purpose

DRAFT needs a safe Rust-owned boundary for PDFs selected explicitly or noticed
inside a watched folder. A watched file must not become an import candidate
while another process is still writing it.

## Scope

Phase 24 creates Rust-only `PendingPdfImport` candidates. A candidate records a
Rust-generated ID, explicit or watched-folder provenance, a private canonical
path, and byte length. It is not a reference record, parsed document, or
persistent background job.

Explicit intake validates one caller-approved path immediately. Watched intake
is rooted to one canonical directory and has two operations:

1. record a filesystem change observation; and
2. confirm the file after the debounce deadline.

A future Rust watcher may call these operations. Phase 24 does not start an
unmanaged watcher or expose watched-folder configuration.

## File Boundary

Both intake paths require:

- a `.pdf` extension, compared case-insensitively;
- an existing regular file;
- no final-path symbolic link;
- a canonical path;
- the `%PDF-` file signature; and
- metadata and signature reads that complete without a file-length change.

Watched files must remain inside the canonical watched root. Directory escapes
and symbolic links fail before a candidate is created.

The intake boundary reads no full PDF body and performs no text extraction,
metadata parsing, copying, moving, deletion, or mutation.

## Stable-Write Rule

The Phase 24 debounce is one second. Every recorded change resets the deadline,
including an event where the observed byte length is unchanged.

Confirmation requires:

- no later change event within the debounce window;
- the current byte length to match the recorded byte length; and
- the byte length to remain unchanged while Rust reads the PDF signature.

If the length changed without a recorded event, the intake records a new
observation at confirmation time and returns `Waiting`. Only a stable valid PDF
returns `Pending`.

## Typed Failures

Failures distinguish unsupported file type, unavailable file, non-regular
file, symbolic link, metadata failure, read failure, invalid PDF signature,
file change during explicit validation, invalid watched root, path outside the
watched root, missing observation, and out-of-order observation time.

Errors do not expose paths, file contents, OS errors, or imported metadata.

## Verification

Tests and scans must cover:

- explicit valid PDF intake and provenance;
- extension, missing-file, directory, symbolic-link, and signature rejection;
- a fake PDF written in chunks with change observations;
- no pending candidate before the debounce deadline;
- byte-length changes that occur without a corresponding event;
- pending intake only after stable-size confirmation;
- watched-root containment;
- typed observation failures;
- no frontend, Tauri command, persistence, network, parser, or unmanaged worker
  authority; and
- local/GitHub Actions parity.

## Non-Goals

Phase 24 does not add a file dialog, Tauri import command, visible import UI,
filesystem watcher dependency, watcher thread, persistent import queue,
background job, PDF parser, text extraction, OCR, metadata merge, reference
creation, file copy, watched-folder preference, or restart recovery.
