# ADR-004: Govern Paragraph Formatting

Date: 2026-07-13
Status: Accepted
Deciders: @progentic
Accepted through: PR #40

## Context

Manual review of the packaged desktop workflow found that DRAFT does not yet
provide the paragraph controls expected from an academic writing application.
The requested capability includes alignment, line spacing, space before and
after a paragraph, left and right indentation, first-line indentation, hanging
indentation, and a reset to document defaults.

This is not a toolbar-only change. Paragraph formatting must survive editing,
save and reopen, copy and paste, undo and redo, and DOCX import or export. It
also changes the strict document envelope, mixed-selection behavior, migration
policy, validation failures, and the format-fidelity decision made before an
external document can be saved safely.

Accepted ADR-003 already separates these responsibilities. Phase 47 owns the
document model, persistence, interoperability, fidelity, and lossiness rules.
Phase 48 owns desktop controls only for capabilities that are available. Adding
controls first would create frontend-only formatting that Rust could neither
validate nor preserve.

The following alternatives were considered:

- **Add UI-only controls in Phase 48:** Keep paragraph state in Tiptap or CSS
  without changing the envelope. This would make formatting disappear after a
  save or produce unsupported DOCX output.
- **Support presets only:** Expose alignment and a few spacing presets without
  a complete persisted model. This is simpler but does not satisfy bounded
  custom spacing or indentation and leaves mixed selections underspecified.
- **Store arbitrary CSS or editor attributes:** Preserve whatever the WebView
  supplies. This weakens validation, permits unbounded values, and cannot map
  deterministically to document formats.
- **Defer paragraph formatting until after v1.0.0:** Preserve the current
  schedule and document the limitation. This leaves an identified academic
  writing requirement unresolved for the release candidate.
- **Govern one strict paragraph model before adding controls:** Let Phase 47
  establish the validated, persistent, interoperable capability, then let
  Phase 48 expose it through the shared desktop action surface.

## Decision

DRAFT will use one strict paragraph-formatting model shared by the
document envelope, Tiptap commands, Rust validation, persistence, migration,
and supported document-format mappings.

Phase 47 will own the data model and interoperability implementation. Phase 48
may expose paragraph controls only after the Phase 47 boundary exists and its
behavioral evidence passes. This decision does not renumber either phase or
authorize paragraph controls before the underlying capability is proven.

The proposed v1 property set is:

- alignment: left, center, right, or justified;
- line spacing: single, 1.15, 1.5, double, or a bounded custom value;
- spacing before and after a paragraph;
- left and right indentation;
- exactly one of no special indentation, first-line indentation, or hanging
  indentation; and
- removal of explicit paragraph formatting to restore document defaults.

Paragraph formatting is block state for paragraphs, headings, and paragraph
content inside supported list structures. It is not an inline mark. The
canonical values, bounds, command semantics, DOCX mapping, migration behavior,
and evidence requirements are specified in the accepted contract at
`docs/contracts/PARAGRAPH_FORMATTING.md`.

No arbitrary CSS, HTML style, font substitution, or unknown paragraph
attribute may enter the persistent model. Unsupported values fail before
document, registry, source, or destination mutation. Same-format save is
denied when an external document contains paragraph behavior that DRAFT cannot
represent without undisclosed loss.

## Consequences

DRAFT gains one reviewable paragraph model instead of separate editor,
persistence, and DOCX interpretations. Users can predict that a supported
paragraph change will survive reopen and export, while unsupported content
receives an explicit fidelity outcome.

This makes several things harder:

- the document envelope requires an explicit version 1 to version 2 migration;
- Tiptap must apply block changes in one transaction and report mixed values;
- copy and paste must reject arbitrary style injection while preserving
  canonical DRAFT attributes;
- DOCX import and export must classify unsupported paragraph constructs before
  save-back;
- reset behavior must resolve stable document defaults rather than frontend
  placeholders; and
- Phase 48 paragraph controls remain blocked until Phase 47 supplies the
  underlying capability.

The decision narrows v1 rather than reproducing a full word processor.
Paragraph tabs, borders, shading, pagination controls, columns, section
layout, widow/orphan settings, and arbitrary style inheritance remain outside
this proposal.

Affected downstream surfaces include `ARCHITECTURE.md`, `INVARIANTS.md`, the
document envelope and migration contracts, Tiptap command and selection tests,
DOCX import/export guides, format-fidelity evidence, Phase 47 and Phase 48
maintainer documentation, user limitations, and release-gate enforcement.

## Enforcement

Following acceptance, repository checks require:

- ADR-004 and its promoted contract to remain explicitly Accepted;
- `INV-17` to remain Proposed;
- all current release rows to remain open; and
- paragraph controls to remain absent until Phase 47 supplies the model and
  behavioral evidence.

Phase 47 must replace the acceptance-record absence guard with behavioral tests
covering strict validation, migration non-mutation, selection semantics,
save/reopen, DOCX fidelity, source preservation, typed failures, and bounded
resource use. Phase 48 must then add keyboard, focus, accessible-name,
mixed-state, reset, and packaged interaction evidence before its paragraph
controls can close.

PR #40 merged the decision after required self-review, green local and hosted
verification, and a recorded one-time owner override of the remaining cooling
period. The override did not amend `GOVERNANCE.md` or accept proposed
`INV-17`; merge remained the acceptance event.

## Links

- `ARCHITECTURE.md` §6 and §11
- `INVARIANTS.md` `INV-03`, `INV-04`, `INV-09`, and proposed `INV-17`
- ADR-003
- `docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/contracts/PARAGRAPH_FORMATTING.md`
