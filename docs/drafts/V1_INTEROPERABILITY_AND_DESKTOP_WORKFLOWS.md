# DRAFT v1 Interoperability And Desktop Workflows

**Status:** Proposed and non-binding  
**Decision:** Proposed ADR-003  
**Scope:** Successor release requirements after Phase 46

## Purpose

This proposed contract defines the downstream acceptance contract that would follow
ADR-003. It does not change the accepted Phase 45 usability contract, close a
release row, renumber the current roadmap, or authorize implementation while
ADR-003 remains Proposed.

If ADR-003 is accepted, this proposal must complete the `contract-doc` lifecycle
before Phase 47 implementation begins. The accepted successor must preserve
the existing usability qualities and evidence thresholds while assigning them
to the revised Phase 47 through 53 sequence.

## Required Product Boundary

The proposed v1 document boundary is:

| Format | Open/import requirement | Save requirement | v1 disposition |
| :--- | :--- | :--- | :--- |
| `.draft` | Open as the native authoritative envelope. | Save atomically to its Rust-owned target. | Required. |
| `.txt` | Import deterministic UTF-8 paragraphs and line breaks without invented formatting. | Save back only while plain-text compatible; otherwise require Save As. | Required. |
| `.md` | Parse one named subset into editable structure. | Serialize only the supported subset; reject or require Save As when representation would be lossy. | Required. |
| `.docx` | Import one named structural and formatting subset with explicit fidelity classes. | Permit safe same-format save only while representable; otherwise require Save As. | Required. |
| `.odt` | Use one bounded, tested subset if shipped. | Same-format save must obey the lossiness contract. | Required or separately deferred by accepted v1 decision. |
| `.rtf` | Use one bounded, tested subset if shipped. | Same-format save must obey the lossiness contract. | Required or separately deferred by accepted v1 decision. |
| `.doc` | Do not parse as DOCX or claim native support. | No save-back path. | Explicitly unsupported for v1. |

An accepted ODT or RTF deferral must name user conversion guidance, preserve
truthful file-dialog filters and public wording, and leave no active control
that implies support.

## Format Fidelity

Each supported external format must define an allowlisted mapping into and out
of the DRAFT document model. Every construct is classified as exactly one of:

- **Faithfully represented:** editable and serialized without known semantic
  or visible loss inside the supported subset.
- **Approximated:** converted deterministically with the disclosed difference
  visible before save.
- **Preserved but not editable:** retained for round-trip output without being
  exposed as editable DRAFT content.
- **Unsupported:** rejected before the current document, registry, source file,
  or destination changes.

Unknown content cannot be silently deleted. Approximation is not permission to
claim full format fidelity.

### Text

Text import is bounded UTF-8. Newline handling is deterministic. DRAFT does not
invent headings, lists, emphasis, or other structure. Save-back is disabled as
soon as the document contains content the text writer cannot represent without
loss.

### Markdown

The minimum parsed subset includes headings, bold, italic, blockquotes,
ordered and unordered lists, horizontal rules, links, inline code, paragraph
breaks, and fenced code blocks only if the editor contract supports them.
Unsupported Markdown or DRAFT-only constructs must be rejected, disclosed as
an approximation, or require Save As to a richer format.

### DOCX

The minimum mapping evaluates paragraphs, heading levels, bold, italic,
underline, supported font family and size, lists, blockquotes, alignment, page
breaks, hyperlinks, tables only when the editor can preserve them, and
citations only when DRAFT can represent them safely. ZIP/XML limits,
relationship safety, active-content rejection, and source-preservation rules
apply before registry or write activity.

DOCX import and DOCX export are separate implementations. Existing export
evidence does not prove import or round-trip fidelity.

## Rust-Owned Lifecycle

Document identity and persistence authority remain separate. A Rust-owned
in-memory ID does not imply a writable source.

The accepted implementation must distinguish at least these lifecycle
meanings:

```text
new
opened_draft
opened_external
imported_external
exported_copy
```

These names are contract meanings, not required wire identifiers.

Rust owns:

- source path and canonical identity;
- native format;
- read-only or writable source capability;
- current save target;
- round-trip and lossiness state;
- parser/writer selection and resource limits;
- atomic replacement and source preservation; and
- typed cancel, success, validation, compatibility, and write failures.

The frontend may display a basename, format, compatibility state, and
available action. It cannot receive or retain a full external source path as
save authority.

## Save Rules

Opening and closing an external file without edits does not write it. The
original bytes remain unchanged.

After edits:

- same-format Save is enabled only when the writer can preserve the current
  document within the accepted subset;
- a known approximation must be disclosed before write and covered by exact
  fixtures;
- an unsupported or lossy state denies same-format Save and requires Save As;
- cancellation leaves document state and source bytes unchanged;
- a failed parse, validation, conversion, or write leaves both source and
  destination unchanged; and
- successful replacement uses the Rust-owned atomic writer and updates
  lifecycle state only after the complete target exists.

Export creates a derived copy and never changes source ownership.

## Phase 47 - Document Interoperability

Phase 47 owns:

- parsed Markdown import and supported-subset serialization;
- DOCX import and safe supported-subset round-trip save;
- bounded text import and compatibility-aware save-back;
- ODT and RTF implementation or separately accepted v1 deferrals;
- explicit legacy `.doc` rejection and conversion guidance;
- Rust-owned external-format lifecycle and lossiness state;
- no-edit source preservation;
- format-specific typed failures and limits;
- representative, hostile, and visual comparison fixtures; and
- user and maintainer documentation for exact support.

Phase 47 excludes native menus, broad visual redesign, security completion,
signing, notarization, release upload, PDF implementation, and external office
suite dependencies unless separately accepted through governance.

## Phase 48 - Desktop UI And Native Workflow Integration

All primary visible workflows must also be reachable through the native macOS
menu bar. Required top-level groups are DRAFT, File, Edit, Text, Paragraph,
Research, Review, View, Window, and Help where the shipped action set makes the
group applicable.

Native menus, visible controls, and keyboard shortcuts use one shared action
dispatcher. File and trusted actions terminate in typed Rust commands;
editor-only actions target the active Tiptap editor. No native-menu handler may
become a second persistence, import, export, citation, or document-identity
implementation.

Menu enablement follows live application state. At minimum, Save, Save As,
Close, Undo, Redo, citation insertion, findings navigation, export, and
formatting actions must not appear available when their existing workflow
cannot honor them. Conventional macOS shortcuts use their platform meaning and
remain documented and tested.

Phase 48 also owns:

- visible application/icon consistency;
- file, edit, text, paragraph, research, review, and view grouping;
- clear active, disabled, and unavailable states;
- editor canvas and outline composition;
- spacing, alignment, density, and hierarchy;
- responsive overflow at narrow widths;
- keyboard and focus continuity; and
- packaged light/dark and scaled-window validation.

It does not authorize controls for capabilities that remain unavailable.

## Phase 49 - Usability And Performance Validation

The existing first-time-user, visible-language, state, recovery, realistic
workload, and measured/perceived responsiveness requirements move to Phase 49.
The accepted participant counts and thresholds do not become weaker.

Validation must include native-menu discovery, toolbar/menu terminology
agreement, state-sensitive enablement, import-format choice, lossiness
understanding, Save versus Save As prediction, source-safety confidence, and
recovery from one controlled compatibility failure.

## Phase 50 - Documentation And Drift Realignment

Phase 50 remains the mandatory fifth-phase checkpoint. It reconciles the
implemented format matrix, native-menu action inventory, lifecycle authority,
lossiness wording, tests, Wiki source, configuration limits, release ledger,
and packaged evidence without adding product behavior.

## Phase 51 - Security Review

The existing security and secure-usability scope moves from Phase 48 to Phase
51. It must additionally review archive/XML limits, parser dependencies,
external relationship handling, malformed document behavior, path ownership,
source overwrite safety, native-menu authority, and lossiness disclosure.

## Phase 52 - Final Release Candidate

The exact signed/notarized candidate package must rerun the complete supported
workflow, including native and external document paths, native menus,
keyboard-only use, format limitations, source preservation, lossiness
recovery, local analysis, citations, and supported export.

## Phase 53 - v1.0.0 Release

Versioning, user release notes, tag, GitHub release, download, launch,
onboarding, format-support matrix, shortcuts, recovery guidance, and known
limitations occur only after the candidate and every prior row are closed.

## Successor Gate Chain

The accepted successor contract must use this dependency order:

| Row | Owner | Closure basis |
| :--- | :--- | :--- |
| `RC-01` through `RC-04`, `GATE-46` | Phase 46 | Existing workflow and accessibility evidence closes independently. |
| `RC-07`, `GATE-47` | Phase 47 | Format matrix, lifecycle authority, round-trip, lossiness, source-preservation, and fixture evidence pass. |
| `RC-08`, `GATE-48` | Phase 48 | Shared native/visible action dispatch, menus, shortcuts, state, icon, layout, keyboard, and packaged desktop evidence pass. |
| `GATE-49` | Phase 49 | First-time-user thresholds plus measured and perceived performance pass. |
| `GATE-50` | Phase 50 | Repository, user, architecture, contract, release, and implementation truth agree. |
| `RC-05`, `GATE-51` | Phase 51 | CSP, trust-boundary, parser, dependency, path, native-menu, and secure-usability review passes. |
| `RC-06` | Phase 52 | Exact candidate distribution and complete packaged workflow pass. |
| Release | Phase 53 | Every prior row is closed with evidence and v1 publication checks pass. |

The current open `GATE-47` and `GATE-48` meanings are superseded only after
ADR-003 and this successor contract are accepted. Renumbering is not closure,
and no existing row may be removed to manufacture a passing release ledger.

## Required Evidence

Phase 47 evidence must cover:

- valid, malformed, oversized, deeply nested, and hostile inputs;
- every supported and unsupported format decision;
- deterministic parsing and serialization;
- fidelity-class assignment;
- no-edit byte preservation;
- safe edited-document output in compatible readers;
- lossless save, rejected lossy save, Save As, cancellation, and recovery;
- source and destination non-mutation on every failure stage; and
- absence of frontend path or filesystem authority.

Phase 48 evidence must cover:

- one shared action dispatcher;
- native menu and visible control parity;
- focus-aware state and enablement;
- conventional shortcuts and keyboard-only operation;
- no duplicate command implementation;
- exact visible icon/branding in the packaged app;
- normal, narrow, scaled, light, dark, and reduced-motion layouts; and
- no unsupported active control.

Human evidence cannot be replaced by source scans. Parser presence cannot be
treated as format fidelity, and package-icon byte equality cannot be treated as
visible branding proof.

## Non-Goals

This draft does not implement a parser, serializer, lifecycle state, menu,
dispatcher, control, visual redesign, security fix, release package, or phase
completion. It does not authorize PDF, legacy `.doc`, an external office-suite
runtime, arbitrary format plugins, frontend filesystem access, or silent
lossy conversion.
