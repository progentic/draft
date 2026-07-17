# DOCX Interoperability

## Purpose

This guide explains how DRAFT reads a bounded part of a DOCX file without
putting the original at risk. It is the maintainer reference for DOCX package
safety, paragraph-property conversion, fidelity results, and external-source
ownership.

## Problem

A DOCX file can contain much more than editable paragraphs. It is a ZIP
package of related XML parts and may include styles, links, layout behavior,
or malformed content that DRAFT cannot preserve. Opening the package as if it
were a native DRAFT document could silently discard that behavior. Reusing
DOCX export as an overwrite operation would create the same risk because
export proves only that DRAFT can create a new package from its own model.

## Solution

Rust reads the selected package under fixed limits, validates its paths and
relationships, and converts only the accepted paragraph subset into the
canonical document model. Every successful import includes a closed fidelity
result. Unsupported but valid behavior is disclosed and requires retaining the
untouched source rather than pretending the imported copy is exact.

Internal OPC relationship targets are resolved from the part that owns the
relationship. A target such as `../customXml/item1.xml` is valid from
`word/document.xml` because it remains inside the package root. Absolute,
URI-like, backslash, and root-escaping targets still fail before parsing.

The external source remains registered in Rust, but it never becomes the
native `.draft` save target. The frontend receives only the source filename,
format, fidelity class, and a same-format save disposition. Saving the imported
work through the visible workflow selects a new `.draft` target. Exporting
creates a separate DOCX copy.

A separate same-format source-write command now exists for exact supported
DOCX content and explicitly accepted canonical normalization. Rust alone keeps
the source path and fingerprints, rechecks source identity immediately before
replacement, and restores the original bytes if post-replacement durability or
registry commit fails. The visible **Save Back to Source** workflow first asks
Rust for a non-mutating eligibility result, names every known canonical
transformation, and requires explicit Replace or Cancel confirmation before
replacement. A stale source is denied again immediately before the atomic
write. This path is separate from native `.draft` Save and derived-copy DOCX
export.

## Trade-offs

- The importer is deliberately smaller than Microsoft Word's document model.
- Text, accepted run marks, page breaks, and accepted paragraph properties
  remain editable. Unsupported run, style, or package behavior is disclosed and
  remains available only in the untouched source.
- An untouched source reports `no_changes`; edited exact content is eligible
  for the bounded writer, while normalized content requires explicit consent.
- Unsupported source behavior, missing provenance, unknown fidelity, a missing
  source, and an externally changed source deny replacement.
- ZIP parts are read conservatively under the XML-part bound. Large embedded
  binary parts are therefore rejected rather than retained opaquely.
- Local macOS text extraction and configured LibreOffice conversion reopen
  exact and accepted-normalized replacements mechanically. The failed
  `a60f877` packaged session did not establish complete compatible-reader or
  human fidelity evidence, so the release gate remains open.

## Technical Contract

### Supported canonical mapping

The importer accepts paragraphs, heading levels 1 through 6, text, hard line
breaks, canonical page-break blocks, and these directly declared properties:

| DOCX property | Canonical DRAFT value |
| :--- | :--- |
| `w:jc` | `left`, `center`, `right`, or `justify` |
| `w:spacing/@w:line` with `auto` | Whole five-hundredths from 100 through 300 |
| `w:spacing/@w:before` and `@w:after` | 0 through 2,880 twips |
| `w:ind/@w:left` and `@w:right` | 0 through 2,880 twips |
| `w:ind/@w:firstLine` or `@w:hanging` | One mutually exclusive value from 0 through 1,440 twips |
| `w:rFonts` | One matching explicit family from DRAFT's eleven-family allowlist |
| `w:sz` and `w:szCs` | One matching whole-point size from 8 through 72 |
| `w:b`, `w:i`, and single `w:u` | Canonical bold, italic, and underline marks |
| `w:br w:type="page"` | A canonical top-level `pageBreak` block |
| `w:pageBreakBefore` | A disclosed normalization to a canonical `pageBreak` block |

Missing paragraph properties remain absent. The importer never serializes a
complete default `paragraphStyle` object. `Heading 1` through `Heading 6` are
canonicalized to the accepted style identifiers and recorded as normalization.

The editor renders each canonical `pageBreak` as a full-width gap and visible
page edges between distinct white page surfaces. It does not show punctuation
or a dashed line as document content. This presentation is exact only for
explicit page-break nodes: DRAFT does not infer pagination from content flow,
margins, font metrics, printer geometry, or Word's automatic layout.

Exact and at-least line spacing, list numbering, unsupported inherited styles,
and unsupported document structures are typed unsupported failures. Borders,
shading, tab stops, contextual spacing, pagination controls other than page
breaks, run properties outside the accepted marks, theme or conflicting font
declarations, external relationships, noncanonical styles, and additional
package parts are classified as unsupported but source-preservable. Supported
direct properties remain in the canonical document even when an unrelated
source feature requires preservation. Values that would require rounding or
clamping are classified as lossy and rejected.

### Fidelity classes

`ExternalFidelity` has stable ordered variants:

1. `exact`
2. `canonically_normalized`
3. `unsupported_preservable`
4. `lossy`
5. `malformed_external_input`
6. `unsupported_external_feature`
7. `unsafe`

Feature lists use stable enum order and contain no duplicates. Errors and
summaries contain no document text, XML, package entries, filesystem paths, or
source fingerprints.

### Source and save ownership

`ExternalSourceProvenance` retains the canonical source path, source SHA-256, imported
envelope SHA-256, format, fidelity, access mode, and writer capability inside
Rust. The document registry prevents a second live handle from claiming the
same external path. It reports no native source path for the imported handle.

`SameFormatSaveDisposition` distinguishes unchanged, exact, normalized,
unsupported, read-only, missing-provenance, unknown-fidelity,
writer-unavailable, missing-source, and changed-source outcomes. Exact imports
receive exact writer support. Canonically normalized imports receive writer
support that requires explicit acceptance. Unsupported-preservable imports
remain read-only.

`save_external_document` is separate from native `.draft` Save and derived-copy
DOCX export. It compiles a complete replacement in memory, rechecks that source
bytes still match the planned bytes, writes through the shared atomic writer,
and commits the new source and envelope fingerprints only after replacement.
If parent-directory durability or registry commit fails after replacement, it
atomically restores the original bytes. A successful rollback is a typed
failure with preserved source; a failed rollback is a distinct uncertain-state
failure. Cancellation and every pre-replacement failure leave source bytes and
registry state unchanged.

The closed `inspect` decision performs the same current-source and
representability checks without writing. The visible workflow permits only
`allowed_exact` and `allowed_after_accepted_normalization` to reach a
confirmation. Missing, externally changed, unsupported, lossy, read-only, and
uncertain sources remain unavailable. Successful replacement retains the
external source identity and basename display name.

### Resource limits

| Limit | Value |
| :--- | :--- |
| Complete DOCX package | 16 MiB |
| ZIP entries | 128 |
| One extracted part | 8 MiB |
| Total declared uncompressed bytes | 64 MiB |
| Compression ratio | 100:1 |
| XML depth | 64 |
| Imported canonical nodes | 100,000 |

`DRAFT_DOCX_TRACE` is unset by default. Setting it for a local diagnostic run
emits path-free Open command, dialog-selection, source-classification,
typed-result, frontend-payload-ready, import, and export stage names plus
counts, sizes, and closed failure categories to stderr. It never emits document
text, XML, source names, or filesystem paths.

The outer file read is stream-bounded even if the source grows after metadata
inspection. ZIP entry paths must use relative normal components. Internal OPC
targets are normalized from their owning part and may use parent segments only
while the resolved target remains inside the package root. Symlinks, encrypted
entries, duplicate central-directory entries, unsafe relationship targets,
doctypes, unknown entities, and exceeded limits fail before registry mutation.

## Implementation Notes

| Layer | Code | Responsibility |
| :--- | :--- | :--- |
| High | `interoperability::import_docx_source` | Reads one selected source, builds a validated envelope, and returns path-free presentation data plus internal provenance. |
| Mid | `interoperability::fidelity` and `provenance` | Own closed fidelity categories, source fingerprints, and save eligibility. |
| Mid | `DocumentRegistry::open_imported_external` | Own one live external-source registration without a native save target. |
| Low | `interoperability::docx_import::package` | Validates ZIP, parts, relationships, content types, and XML bounds. |
| Low | `interoperability::docx_import::document` | Maps the accepted XML subset into canonical Tiptap JSON. |
| Low | `docx_trace` | Emits opt-in path-free Open, import, and export stage diagnostics for local failure investigation. |
| Presentation | `src/ipc/externalDocument.ts` | Validates the path-free DTO and rejects unknown or unstable variants. |
| Presentation | `src/ipc/externalDocumentSave.ts` | Validates eligibility, save, cancellation, denial, and bounded recovery outcomes. |
| Presentation | `useExternalSourceSave` | Coordinates inspection, confirmation, cancellation, and replacement without owning a path. |
| Presentation | `SaveBackToSourceDialog` | Presents overwrite and normalization warnings with keyboard-contained confirmation. |

The Open command returns `imported_external`, distinct from `opened_draft`,
`imported_text`, and `cancelled`. React tracks Rust registration separately
from native DRAFT persistence. Ordinary Save creates a `.draft` target, Export
DOCX creates a separate copy, and Save Back retains external registration. The
document dispatcher keeps these actions single-flight so a later operation or
session result cannot be hidden by settled export feedback. Starting a
non-export document action clears that feedback before the action reports its
own pending and terminal state. The editor generation cannot accept a stale
eligibility result.

## Failure Modes

- A malformed package returns `malformed_package`.
- A package that exceeds safety limits returns `unsafe_package` with one closed
  safety reason. User copy identifies package, XML, or document-size limits and
  suggests reducing large embedded content without exposing internal package
  details.
- A valid feature that cannot enter the canonical model returns
  `unsupported_external_feature`.
- A value that would require undisclosed approximation returns
  `lossy_import_denied`.
- A failed canonical envelope conversion returns `invalid_canonical_document`.
- A missing, unreadable, or oversized source fails before registration.
- A missing or externally changed source denies same-format replacement.
- A failed replacement reports whether the original bytes were restored or
  whether source state is uncertain.

Visible recovery says whether the original remained unchanged. Cancellation is
a normal no-op. No failure changes the active document, source bytes, target,
registry identity, or displayed filename.

## Tests

Rust unit tests cover every accepted paragraph and direct run property, absent
defaults, canonical heading and page-break normalization, malformed and
unsupported properties, supported formatting beside unrelated preservable
behavior, content types, relationships, duplicate entries, path
traversal, package limits, compression ratio, XML depth, and deterministic
ordering. The canonical stored-package fixture SHA-256 is
`c284d54886d21d2fda1d0fa51099ac2db65cbaf830ce133d8f6608c21c4bf35a`.

The independent `word-custom-xml.docx` fixture starts from a blank document
created by Microsoft Word, contains fixed DRAFT-owned visible text and one
deterministic custom-XML relationship, and has SHA-256
`9929f84423e135a5100ab43b8c454a6734d78cfaf41eea1e4274e707c0d1cbe6`.
It reproduces Word's valid `../customXml/item1.xml` relationship shape without
using DRAFT's exporter as the import oracle. The production-path regression
opens that fixture through the registry, exports it atomically, reopens the
result, compares visible text, and confirms the source bytes remain unchanged.
LibreOffice independently opens both files and extracts byte-identical visible
text.

Persistence tests prove no-edit open/close preservation, canonical path-alias
deduplication, duplicate-source rejection, cancelled and failed Save
preservation, import-to-new-`.draft`, and export-to-new-DOCX separation.
Source-write tests cover exact replacement, normalization consent, source
changes before replacement, compilation and write failure, durability and
registry rollback, rollback failure, fingerprint refresh, and non-mutating
eligibility. A macOS-only reader check opens both exact and accepted-normalized
replacements through `textutil`. TypeScript tests cover page-break JSON and
HTML preservation, every fidelity and source-write outcome, unknown variants,
path-bearing DTO rejection, exhaustive path-free recovery, stale generations,
confirmation, cancellation, rendered font/paragraph/page-break import, and
visible dispatcher parity. Existing atomic-export tests continue to prove target
promotion and partial-output cleanup.

These are mechanical and local reader-open results. Packaged human validation
of confirmation, cancellation, failure, and recovery; broader compatible-reader
comparison; complete format coverage; `RC-07`; and `GATE-47` remain open.

## Related Documents

- `docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/contracts/PARAGRAPH_FORMATTING.md`
- `docs/maintainers/PARAGRAPH_FORMATTING.md`
- `docs/maintainers/DOCUMENT_SAVE_LOAD.md`
- `docs/maintainers/DOCX_EXPORT.md`
- `docs/maintainers/CONFIGURATION.md`
- `docs/maintainers/RELEASE_CANDIDATE.md`
