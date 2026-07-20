# DOCX Export Boundary

## Status

This guide records implemented Phase 32 behavior. The requirements in
`docs/drafts/DOCX_EXPORT.md` remain non-binding until they complete the contract
lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 32 adds a Rust-only DOCX compiler and atomic export service for one
validated immutable `DocumentEnvelope`. It creates a derived artifact and never
makes DOCX the source of truth.

Phase 46 first exposed the exporter through a dedicated command. Phase 47 now
routes DOCX copies through the unified typed Save As command and a
format-constrained Rust-owned dialog. The former standalone command, hook, and
menu action are removed. This adds no export history, persistence, worker,
Python helper, network call, citation renderer, PDF path, or broad style-manual
claim.

## Strict Document Subset

The compiler accepts a `doc` root containing ordered `paragraph`, `heading`,
and canonical `pageBreak` blocks. Headings accept levels 1 through 6. Inline
content accepts `text` and `hardBreak`; text accepts `bold`, `italic`,
`underline`, `fontFamily`, and `fontSize` marks. A `pageBreak` emits one
explicit `w:br w:type="page"` run inside its own paragraph.

Font family accepts only `arial`, `avenir_next`, `baskerville`, `courier_new`,
`georgia`, `helvetica`, `menlo`, `palatino`, `times_new_roman`, `trebuchet_ms`,
and `verdana`. They map exactly to their named families in `w:rFonts`. Font
size accepts whole points from 8 through 72 and converts
deterministically to DOCX half-points in `w:sz` and `w:szCs`. Unsupported or
malformed values fail; the compiler never substitutes a family or size.

Phase 47 adds optional canonical paragraph data for paragraphs and headings.
Alignment maps to `w:jc`; automatic line spacing and space before/after map to
`w:spacing`; left/right and first-line or hanging indentation map to `w:ind`.
All explicit values are emitted deterministically. A block without
`paragraphStyle` emits no default paragraph override. The bounded DOCX importer
uses the inverse mapping for accepted paragraph values. Import and export are
separate policies; this exporter does not authorize overwriting an imported
DOCX. The independent Save Back workflow applies its own fidelity, source-
identity, confirmation, and rollback policy. See
`docs/maintainers/DOCX_INTEROPERABILITY.md`.

Empty paragraphs and headings, Unicode text, source order, paragraph
boundaries, heading levels, hard and page breaks, and supported marks are
preserved. XML-invalid control characters fail before package construction.

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

The Save As command validates the current envelope, calls `compile_docx`, and
uses the owned atomic artifact writer. Its `converted_output` response contains
only the output basename, format, and byte count. It explicitly reports that
document authority and dirty state did not change.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `export_docx` | Validates the target, compiles fully, and coordinates atomic replacement. |
| High | `compile_docx` | Produces one bounded immutable artifact without filesystem work. |
| Mid | `parse_docx_document` | Enforces limits and converts the strict Tiptap subset into a typed model. |
| Mid | `build_docx_package` | Builds fixed XML parts and deterministic ZIP bytes. |
| Low | XML/ZIP event helpers and atomic writer | Escape text, write fixed package structures, sync, and replace. |

## Verification

Focused Rust tests cover stable safe entries, archive reopening,
deterministic bytes, XML parsing, Unicode, headings, hard and page breaks, supported
marks, every family mapping, mixed family and size run properties, paragraph
property mapping, default-by-absence, empty
blocks, unknown and malformed content, citation rejection, source
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

The strict subset does not support arbitrary fonts or sizes, citations, bibliographies, lists, tables,
links, images, equations, notes, comments, tracked changes, headers, footers,
page numbers, templates, unsupported paragraph rules, layout controls, or complete APA/MLA/Chicago rendering.
Unsupported content fails the whole export. The visible Save As flow is
documented in `PHASE46_WORKFLOWS.md` and `DOCUMENT_SAVE_LOAD.md`.

## Configuration Index

Source, node, nesting, artifact, and supported-subset limits are indexed in
`docs/maintainers/CONFIGURATION.md`.
