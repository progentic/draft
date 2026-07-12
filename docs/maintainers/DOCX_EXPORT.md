# DOCX Export Boundary

## Status

This guide records implemented Phase 32 behavior. The requirements in
`docs/drafts/DOCX_EXPORT.md` remain non-binding until they complete the contract
lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 32 adds a Rust-only DOCX compiler and atomic export service for one
validated immutable `DocumentEnvelope`. It creates a derived artifact and never
makes DOCX the source of truth.

Phase 46 adds one typed Tauri command, Rust-owned native export dialog,
frontend wrapper, and visible control. It adds no export history, persistence,
worker, Python helper, network call, citation renderer, PDF path, or broad
style-manual claim.

## Strict Document Subset

The compiler accepts a `doc` root containing ordered `paragraph` and `heading`
blocks. Headings accept levels 1 through 6. Inline content accepts `text` and
`hardBreak`; text accepts only `bold`, `italic`, and `underline` marks.

Empty paragraphs and headings, Unicode text, source order, paragraph boundaries,
heading levels, hard breaks, and supported marks are preserved. XML-invalid
control characters fail before package construction.

Unknown fields, nodes, attributes, marks, duplicate marks, and malformed shapes
fail with a typed structural path containing indexes only. Citation nodes return
`UnsupportedCitation`; the compiler never exports the disposable editor marker
as a final citation.

## Resource Limits

Before parsing, a non-allocating Serde writer limits the serialized Tiptap value
to 8 MiB. Recursive validation permits at most 100,000 typed structural objects
and a JSON nesting depth of 16. The complete DOCX package is limited to 16 MiB.

Limits fail before target-file work. Errors contain no document title, source
text, citekey, XML, package bytes, path, or raw library/operating-system detail.

## Package Construction

`quick-xml` 0.41.0 writes XML events and escapes all user text. `zip` 8.6.0 runs
with default features disabled and stores entries without compression,
encryption, or time features. `SimpleFileOptions::DEFAULT` fixes the ZIP
timestamp at its deterministic baseline. Separate package policy emits no active
content.

The package contains exactly these ordered parts:

- `[Content_Types].xml`
- `_rels/.rels`
- `word/document.xml`
- `word/_rels/document.xml.rels`
- `word/styles.xml`

Relationship targets are fixed internal parts. The compiler generates no
external targets, macros, embedded resources, remote templates, absolute paths,
parent traversal, or duplicate entries. Equal validated documents produce equal
bytes.

## Atomic Export

`export_docx` accepts only a Rust-owned path whose extension is `.docx`
case-insensitively. It validates the target, compiles the complete in-memory
artifact, and then calls the existing same-directory atomic writer.

Open, write, content-sync, replace, and temporary-cleanup failures map to closed
`DocxWriteStage` values. A parent-directory sync failure after replacement maps
to `DurabilityUncertain`; the target contains the new complete DOCX, but the
durability guarantee is not overstated.

Compilation failure leaves a prior target unchanged. Real create/replace tests
prove that export changes only the `.docx` target while source DRAFT bytes remain
unchanged. The exporter never reads from or writes to `DocumentRegistry`.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `export_docx` | Validates the target, compiles fully, and coordinates atomic replacement. |
| High | `compile_docx` | Produces one bounded immutable artifact without filesystem work. |
| Mid | `parse_docx_document` | Enforces limits and converts the strict Tiptap subset into a typed model. |
| Mid | `build_docx_package` | Builds fixed XML parts and deterministic ZIP bytes. |
| Low | XML/ZIP event helpers and atomic writer | Escape text, write fixed package structures, sync, and replace. |

## Verification

Eighteen focused Rust tests cover stable safe entries, archive reopening,
deterministic bytes, XML parsing, Unicode, headings, hard breaks, supported
marks, empty blocks, unknown and malformed content, citation rejection, source
and output limits, target validation, real create/replace behavior, source
preservation, every atomic failure stage, durability uncertainty, bounded errors,
and the absence of external or active package content.

The existing atomic-writer interruption and cleanup suite remains active.
`scripts/check-invariants.sh` requires the dependencies, limits, package parts,
structured XML and stored ZIP markers, atomic writer, named tests, and this
guide. It rejects direct filesystem/persistence authority, manual XML
interpolation, Tauri, frontend, Python, network, worker, application-state, and
unsafe package expansion. The Phase 33 PDF-export absence gate remains active.

Phase 41 adds crate-level evidence that a reopened citation-bearing document
fails export explicitly, while a later supported saved snapshot exports to a
package that reopens with the final text and leaves the DRAFT source unchanged.

## Current Limits

The strict subset does not support citations, bibliographies, lists, tables,
links, images, equations, notes, comments, tracked changes, headers, footers,
page numbers, templates, layout controls, or complete APA/MLA/Chicago rendering.
Unsupported content fails the whole export. The visible Phase 46 flow is
documented in `PHASE46_WORKFLOWS.md`.

## Configuration Index

Source, node, nesting, artifact, and supported-subset limits are indexed in
`docs/maintainers/CONFIGURATION.md`.
