# Paragraph Formatting Model

## Purpose

This model gives paragraph appearance one durable meaning across the editor,
the DRAFT file, DOCX export, and the bounded DOCX paragraph importer. It is
Phase 47 infrastructure. It does not add paragraph controls, same-format DOCX
overwrite, or claim that external document round trips are complete.

## Problem

Alignment or spacing stored only in React or CSS disappears when a document is
saved, reopened, or exported. Allowing arbitrary values would also make it
impossible for Rust to decide whether a document is valid or can be preserved
without loss.

## Solution

DRAFT stores an optional, complete `paragraphStyle` object on paragraph and
heading blocks. Rust validates that object before persistence or export.
TypeScript mirrors the same values for early rejection, and Tiptap preserves
the canonical object without treating arbitrary pasted CSS as document data.
When the object is absent, the block uses document defaults.

## Technical Contract

The accepted normative shape and bounds live in
`docs/contracts/PARAGRAPH_FORMATTING.md`. The implementation keeps these rules:

- paragraph style schema version is `1` inside document envelope version `2`;
- the complete object is required whenever `paragraphStyle` exists;
- unknown fields and unsupported block placement fail;
- alignment is `left`, `center`, `right`, or `justify`;
- line spacing is 100 through 300 hundredths in increments of 5;
- paragraph spacing and left/right indentation are 0 through 2,880 twips;
- first-line or hanging indentation is 0 through 1,440 twips; and
- `none` requires zero and no object is written for defaults.

Rust is authoritative. TypeScript acceptance cannot override a Rust rejection.
No frontend code creates a filesystem path or bypasses envelope validation.

## Implementation Notes

| Layer | Code | Responsibility |
| :--- | :--- | :--- |
| Policy | `documents/envelope.rs` | Runs paragraph validation with other envelope rules. |
| Domain | `documents/paragraph_format.rs` | Owns canonical values, bounds, parsing, and DOCX-facing typed accessors. |
| Migration | `documents/migration.rs` | Changes a detached valid v1 envelope into a v2 candidate without inventing style. |
| Frontend mirror | `documents/paragraphFormatting.ts` | Rejects malformed UI state early and removes unset editor attributes from snapshots. |
| Editor model | `editor/ParagraphFormatting.ts` | Preserves and renders canonical block data; adds no commands or controls. |
| Export | `exports/docx_model.rs`, `exports/docx_package.rs` | Maps validated block data to fixed WordprocessingML paragraph properties. |

DOCX automatic line spacing uses `240 * hundredths / 100`. First-line and
hanging indents map to mutually exclusive attributes. An absent style emits no
paragraph override.

## Failure Modes

- Invalid, incomplete, unknown, out-of-range, or misplaced values return
  `invalid_paragraph_style` with a structural path and typed cause.
- Version 1 input containing paragraph data returns a typed migration failure.
- Migration failure leaves source bytes and registry state unchanged.
- Export cannot silently clamp, substitute, or omit a validated property.
- Arbitrary pasted CSS and malformed canonical HTML attributes create no style.

## Tests

Rust tests cover every value family, numeric boundary, malformed shape,
unsupported placement, v1 migration, first-save promotion, source and registry
non-mutation, typed error serialization, and deterministic DOCX XML. Frontend
tests mirror invalid values and prove editor JSON/HTML round trips plus pasted
CSS rejection. The release checks pin these files and named tests while keeping
`INV-17`, `UX-46-024`, `RC-07`, and `GATE-47` open.

## Related Documents

- `docs/contracts/PARAGRAPH_FORMATTING.md`
- `docs/maintainers/DOCUMENT_ENVELOPE.md`
- `docs/maintainers/DATA_MIGRATION.md`
- `docs/maintainers/DOCUMENT_SAVE_LOAD.md`
- `docs/maintainers/DOCX_EXPORT.md`
- `docs/maintainers/CONFIGURATION.md`
- `docs/ARCHITECTURE.md` section 15.1
- `docs/INVARIANTS.md` proposed `INV-17`
