# DOCX Export Foundation Requirements Draft

## Status

This is a non-binding Phase 32 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/DOCX_EXPORT.md`. This draft does not
become an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`.

## Purpose

DRAFT needs a trustworthy DOCX export foundation that produces a complete
derived artifact without making that artifact the source of truth. Unsupported
document content must fail explicitly rather than disappear from an export, and
an export failure must not change either the DRAFT source document or a prior
complete export.

## Scope

Phase 32 adds a Rust-owned DOCX compiler and atomic export service for one
validated immutable `DocumentEnvelope`. Compilation finishes in memory before
the target file is opened. The service accepts only a Rust-owned export target
whose final extension is `.docx`.

The initial supported Tiptap subset is:

- a `doc` root with ordered content;
- `paragraph` blocks;
- `heading` blocks at levels 1 through 6;
- `text` inline nodes; and
- `hardBreak` inline nodes.

Text may carry only the explicitly supported `bold`, `italic`, and `underline`
marks. Empty paragraphs and headings are preserved. Unknown fields, nodes,
attributes, marks, or malformed content fail with a typed unsupported-content
error that identifies a bounded structural path but includes no source text.

Citation nodes fail explicitly in Phase 32. DRAFT must not export their
disposable editor markers as final citations before a governed citation-rendering
contract exists.

Before traversal, Rust limits the serialized Tiptap document to 8 MiB. Traversal
accepts at most 100,000 total nodes and a nesting depth of 16. These limits apply
before package construction and fail with bounded typed errors.

## Package Boundary

The compiler returns one bounded in-memory DOCX artifact. It uses a locked Rust
ZIP package implementation and a structured XML writer or equivalent escaping
API; untrusted text is never interpolated into XML markup manually.

The package contains the minimum deterministic Office Open XML parts required
for a word-processing document:

- `[Content_Types].xml`
- `_rels/.rels`
- `word/document.xml`
- `word/_rels/document.xml.rels`
- `word/styles.xml`

Package entry names are fixed by Rust. Duplicate names, absolute paths, parent
traversal, macros, external relationships, embedded files, remote templates,
and active content are not accepted or generated. Metadata timestamps and ZIP
ordering are fixed so equal validated input produces equal bytes.

The complete DOCX artifact is limited to 16 MiB. Oversized source structures or
compiled output fail before filesystem mutation.

## Rendering Rules

Phase 32 preserves source order, Unicode text, paragraph boundaries, hard
breaks, heading levels, and the three supported inline marks. It maps these
values to fixed internal DOCX styles owned by Rust.

The compiler does not infer page layout or claim APA, MLA, or Chicago
conformance. It does not render bibliographies, citations, comments, footnotes,
tables, lists, links, images, equations, headers, footers, page numbers, or
tracked changes. Encountering any unsupported non-empty node or mark fails the
whole export.

## Filesystem Boundary

Rust validates the `.docx` target and writes the complete compiled bytes through
the existing same-directory temporary-file, content-sync, atomic-replacement,
and parent-directory-sync primitive.

Failure before replacement leaves any prior target byte-for-byte unchanged and
cleans the temporary file. A parent-directory sync failure after replacement is
reported as durability uncertain while preserving the new complete DOCX. The
source `.draft` or `.json` document is never opened for writing by export.

Phase 32 adds no Tauri command or frontend-selected path. Tests inject targets
inside repository-owned test fixtures through Rust-only interfaces. A later
visible export flow must use a native Rust dialog and typed command boundary.

## Failure Behavior

Closed errors distinguish:

- invalid export target;
- invalid or unsupported document structure;
- unsupported citation content;
- package construction failure;
- artifact too large;
- atomic write stage failure; and
- post-replacement durability uncertainty.

Errors contain no document title, source text, citekey, XML, package bytes,
filesystem path, or raw library/operating-system detail.

## Verification

Tests and scans must cover:

- stable package entry names and deterministic byte output;
- package reopening with an independent ZIP reader;
- well-formed required XML parts;
- Unicode, paragraphs, heading levels, hard breaks, and supported marks;
- empty blocks and source-order preservation;
- 8 MiB source, 100,000-node, depth-16, and 16 MiB output bounds;
- unknown fields, malformed shapes, unknown nodes and marks, and citation-node
  rejection without silent omission;
- `.docx` target validation;
- create, replace, interruption, cleanup, and durability-uncertain behavior;
- source-document bytes and document-registry state remain unchanged;
- bounded typed failures without content, paths, XML, or raw errors;
- no macros, external relationships, embedded resources, Python, network,
  persistence, Tauri, frontend, or PDF authority; and
- local/GitHub Actions parity.

Phase 32 must replace the DOCX-export absence gate in
`scripts/check-invariants.sh` with package, atomicity, and authority checks.

## Non-Goals

Phase 32 does not add PDF export, complete style-manual conformance, citation or
bibliography rendering, broad Tiptap compatibility, page-layout controls,
templates, macros, images, tables, lists, links, equations, notes, comments,
tracked changes, application state, a Tauri command, native-dialog integration,
frontend controls, export history, persistence, background workers, or release
packaging.
