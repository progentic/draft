# Phase 46 Visible Workflows

## Purpose

Phase 46 connects four existing Rust-owned capabilities to the workspace:
document lifecycle, manual references and citation insertion, five local text
checks, and DOCX export. The frontend coordinates presentation and transient
state. It never receives a filesystem path, opens SQLite directly, runs Python,
or chooses an export target.

## Ownership

| Layer | Surface | Responsibility |
| --- | --- | --- |
| High | `DraftWorkspace` | Composes one command bar and one active workflow panel. |
| High | `useDocumentSession` | Coordinates visible lifecycle state and unsaved-change decisions. |
| Mid | `ReferenceLibraryPanel` | Collects manual fields and inserts an existing citation node. |
| Mid | `TextAnalysisPanel` | Presents advisory findings and locates their current passages. |
| Mid | `useDocxExport` | Presents export progress, completion, and bounded recovery. |
| Low | IPC clients under `src/ipc` | Validate exact command responses and sanitize unknown failures. |
| Low | Rust commands under `src-tauri/src/commands` | Own registry, store, helper, dialog, and exporter access. |

## Command Contract

The visible workflow uses these typed commands:

- `create_document`, `save_document`, `open_document`, and `close_document` for
  the single visible document session;
- `add_reference` and `list_references` for bounded manual reference summaries;
- `resolve_citation` for an inserted citation node;
- `run_text_analysis` for one immutable plain-text snapshot; and
- `export_document` for a Rust-selected `.docx` target.

`create_document` accepts no fields and returns the fixed initial envelope with
a Rust-generated document identity and one empty paragraph. React never
generates a document ID. New content receives focus at the empty paragraph only
after creation succeeds.
`close_document` releases the Rust registry handle. Opening a replacement first
validates and registers it, then releases the prior clean handle. If release
fails, DRAFT closes the replacement and keeps the current document visible.
Saving before a pending destructive action updates the current identity before
that action continues, so the newly registered handle is also released.

## Unsaved Changes

New, Open, and Close ask for a decision only after the editor changes. The
alert dialog offers three real actions: Save and continue, Discard changes, and
Keep editing. Focus enters the dialog, Tab and Shift+Tab remain inside it,
Escape keeps editing, and focus returns to the invoking control.

There is no autosave, crash recovery, or hidden persistence. A successful save
is the only transition from unsaved to saved.

Open, first Save, and DOCX Export use asynchronous Rust-owned native dialog
callbacks. The Tauri commands await a typed cancel, success, or failure result
without running a blocking dialog call on the application thread. While one of
these operations is pending, the workspace prevents conflicting lifecycle and
panel actions. React never receives the selected filesystem path.

A successful Save returns the Rust-owned document ID, the target basename for
display, and whether Rust selected a new target. The selected path remains in
the registry. The workspace immediately shows the returned filename, while a
later Save reuses the registered path without reopening a dialog. Cancellation
or failure preserves the prior display name and unsaved state.

The save request includes only the current basename and the closed lifecycle
origin previously returned by Rust. Rust derives the native suggestion:
existing DRAFT basename, imported source stem with `.draft`, or
`Untitled.draft` for a new document. Native and in-window titles mirror the
basename plus transient Unsaved state without receiving the selected path.

Open returns one typed origin outcome. `opened_draft` carries a validated
native envelope whose path remains registered in Rust. `imported_text` carries
a new unsaved Rust-owned envelope created from a bounded UTF-8 `.txt` or `.md`
source. `imported_external` carries the canonical supported subset of a DOCX
document plus path-free fidelity and save-policy metadata. Imported content has
no native save target, and the first Save selects a new `.draft` target.
`cancelled` leaves content, selection, title, dirty state, and origin unchanged.
Markdown syntax remains literal text; CRLF and LF line endings become
deterministic editor paragraphs. Rust keeps imported source identity and bytes
out of React, and the original source remains untouched.

## Font Formatting

The editor exposes eleven canonical families: Arial, Avenir Next, Baskerville,
Courier New, Georgia, Helvetica, Menlo, Palatino, Times New Roman, Trebuchet MS,
and Verdana. It accepts whole point sizes from 8 through 72 in one-point
increments. The document-default commands remove the selected family or size
mark without changing other marks.

The controls report effective formatting rather than mark presence. Unmarked
body text shows Georgia and 13 pt; headings show their 24, 18, or 14 pt
document size. A caret inside explicitly formatted text shows that family and
size, while a heterogeneous range shows Mixed fonts or Mixed sizes. Use
document font and Use document size remove explicit marks. Selection changes,
formatting commands, and reopened content refresh the displayed values.

The controls write bounded Tiptap `fontFamily` and `fontSize` marks. The marks
remain inside the existing document envelope, survive save, close, and reopen,
render in the editor, and export as explicit DOCX `w:rFonts`, `w:sz`, and
`w:szCs` run properties. Rust validation rejects arbitrary family identifiers,
CSS, fractional sizes, and sizes outside the range. There is no frontend-only
font state or silent family substitution. Pasted HTML does not import arbitrary
`font-family` or `font-size` CSS; only DRAFT's canonical
`data-draft-font-family` and `data-draft-font-size` attributes are recognized
when editor HTML is parsed. A malformed value fails as a typed
`invalid_envelope` before a dialog or filesystem operation begins.

## Manual References And Citations

The frontend submits citekey, title, author, and four-digit year. Rust creates a
validated version 1 manual `ReferenceRecord`, writes it through the existing
SQLite store, and returns only citekey and title. The frontend never receives a
record ID, provenance, path, database row, or full stored payload.

Insert citation creates the existing version 1 `citation` Tiptap node at the
cursor. Rendering still resolves through `resolve_citation`; the reference
store remains metadata authority. The visible workflow does not edit, delete,
import, search, synchronize, or automatically repair references.

## Local Text Checks

`run_text_analysis` is the sole command allowed to construct
`PythonHelperRunner`. It resolves the packaged `python/draft_helpers` resource,
uses `/usr/bin/python3`, clears the process environment, restores only the fixed
`TMPDIR=/tmp` needed by Apple system Python, and runs the existing allowlisted
worker with a five-second timeout.

The complete visible check set is:

1. Repeated word.
2. Long sentence, above 30 words.
3. Extended capital emphasis, five or more letters.
4. Repeated sentence opening, four or more letters.
5. First-person perspective shift between singular and plural forms.

Findings are fixed deterministic heuristics. They are suggestions for review,
not conclusions. Supporting measurements remain internal and do not create a
sixth visible capability. Results are ordered, limited to 100, and tied to
UTF-8 byte ranges in a maximum 32 KiB text snapshot.

The frontend maps byte ranges back to editor positions. Editing invalidates the
generation, so a stale run cannot replace the current result. Show in document
selects a still-mappable passage but never edits it. No network, credential,
provider, persistence, model runtime, automatic replacement, or model-backed
interpretation is present.

## DOCX Export

`export_document` validates the current envelope, asks Rust to select a DOCX
target, and calls the existing atomic exporter. Cancellation changes no file.
A successful export reports bounded byte count data to the client but the UI
announces only completion and source preservation.

The strict subset includes the eleven canonical families and bounded whole-point
sizes. Citation nodes and other unsupported content fail explicitly; they are
never omitted or silently substituted. The DRAFT source document is not
modified by export.

## Visible Failures

Visible copy is bounded and contains no paths, source text, helper stderr,
database details, or raw Rust errors. Recovery is offered only when the current
workflow can honor it:

- invalid manual fields direct the user to correct those fields;
- a duplicate citekey directs the user to choose another citekey;
- stale text results direct the user to run the same check again;
- unavailable text runtime is reported as unavailable, without a fictitious
  install or provider action;
- unsupported DOCX content directs the user to edit the document and retry;
- citation-bearing DOCX export explains that citations are not currently
  included and that the DRAFT source remains unchanged; and
- unknown transport or payload failures use a bounded retry message.

## Verification

Rust command tests enforce typed signatures plus request, response, and error
serialization for all six new commands. Domain and runner tests cover exact
input bounds, all finding codes, deterministic order, UTF-8 ranges, timeout,
cancellation, unavailable runtime, offline execution, and Apple system Python.

Frontend tests cover strict IPC validation, lifecycle ownership, unsaved focus
containment, explicit origins, literal text import, source-path absence,
cancelled replacement non-mutation, reference insertion,
pending/success/failure states, stale runs,
passage mapping, accessible labels, announcements, and export source-safety
copy. Font tests cover allowlist validation, malformed values, Tiptap JSON,
collapsed and selected text, mark preservation, keyboard focus, lifecycle
restoration, and mixed DOCX runs. `scripts/package-macos.sh` also executes the
embedded worker from the finished unsigned Apple Silicon `.app`.
