---
status: Proposed
adr: ADR-004
upholds: [INV-03, INV-04, INV-09, INV-17, INV-UX-01, INV-UX-06]
owners: [core, frontend, release]
---

# Paragraph Formatting

This contract is Proposed and non-binding. It defines the behavior ADR-004
would authorize if the architecture decision is accepted. No product code may
rely on it while the proposal remains open.

The draft may reference proposed `INV-17`, but no accepted contract or product
implementation may claim to uphold that invariant yet. Accepting ADR-004 would
authorize the architecture and contract direction; it would not by itself
prove `INV-17` enforcement or make the invariant Accepted. A separate governed
status change requires implemented enforcement and evidence.

## Purpose

Define one bounded paragraph-formatting model that DRAFT can edit, validate,
save, reopen, migrate, and map to supported document formats without silent
loss.

## Problem

A paragraph control is useful only when the result survives the complete
document workflow. UI-only alignment or spacing can look correct in the editor
and then disappear after reopen or export. Unbounded CSS values also make it
impossible for Rust to validate documents or decide whether an external file
can be saved safely.

## Solution

DRAFT will represent paragraph appearance as validated block data. The same
closed values will drive editor commands, envelope validation, persistence,
mixed-selection state, migration, and DOCX mapping. Removing explicit
paragraph data returns the block to stable document defaults.

Phase 47 implements and proves this model. Phase 48 may add controls only after
the model is available. The frontend remains responsible for interaction and
presentation; Rust remains responsible for validation, persistence, migration,
format fidelity, and filesystem mutation.

Rust validation is authoritative at persistence, import, migration, and export
boundaries. TypeScript performs equivalent early validation for immediate
interaction feedback, but a TypeScript acceptance never overrides a Rust
rejection. Bounds and identifiers must be generated from, imported from, or
mechanically checked against one canonical definition.

## Trade-offs

The model deliberately omits many advanced word-processing features. This
keeps validation and interoperability bounded, but documents using tabs,
paragraph borders, contextual spacing, exact line heights, or pagination
controls may require a disclosed lossy outcome or a rejected same-format save.

Adding persistent block data also requires an explicit envelope migration.
That work is more expensive than local editor state, but it prevents formatting
from changing meaning at process boundaries.

## Technical Contract

### Supported blocks

Paragraph style applies to paragraph and heading blocks, including paragraph
content inside a supported list item. It is block data, not an inline mark.

Within a supported list item, `paragraphStyle` is stored on the paragraph or
heading block containing the text. It is never stored on the list item or list
container. List depth, marker type, numbering, and structural indentation
remain list properties. Applying `leftIndentTwips`, `rightIndentTwips`, or a
special paragraph indent must not change list nesting. DOCX import may need to
interpret list indentation and paragraph indentation together, but it must
preserve them as separate canonical concerns.

### Serialized shape

An explicit paragraph override uses this strict shape:

```json
{
  "paragraphStyle": {
    "schemaVersion": 1,
    "alignment": "left",
    "lineSpacingHundredths": 100,
    "spaceBeforeTwips": 0,
    "spaceAfterTwips": 0,
    "leftIndentTwips": 0,
    "rightIndentTwips": 0,
    "specialIndent": {
      "kind": "none",
      "twips": 0
    }
  }
}
```

If `paragraphStyle` exists, every field is required and no unknown field is
allowed. Omitting `paragraphStyle` means use the document defaults. Persisted
values use integers and canonical identifiers; arbitrary CSS and HTML style
strings are invalid.

### Values and bounds

| Property | Accepted value |
| :--- | :--- |
| Alignment | `left`, `center`, `right`, or `justify` |
| Line spacing | Integer hundredths from 100 through 300 in increments of 5; presets are 100, 115, 150, and 200 |
| Space before/after | Integer twips from 0 through 2880 |
| Left/right indent | Integer twips from 0 through 2880 |
| Special indent kind | `none`, `first_line`, or `hanging` |
| Special indent amount | Integer twips from 0 through 1440; `none` requires zero |

One point equals 20 twips. The UI may display points, but persistence and
format conversion use twips. First-line and hanging indentation are mutually
exclusive.

The v1 document defaults are left alignment, single line spacing, zero spacing
before and after, zero left and right indentation, and no special indentation.
Reset removes the explicit `paragraphStyle` object instead of writing another
copy of those defaults.

When a block has no explicit `paragraphStyle`, a command that changes one
property begins with that block's effective document defaults, changes only the
requested property, and writes one complete canonical object. When an explicit
object exists, the command preserves every unchanged property.

### Editor semantics

- A collapsed selection applies the requested property to the current block
  and to content subsequently entered in that block.
- A range applies the property to every intersected supported block in one
  Tiptap transaction.
- Changing one property preserves every other paragraph property and all
  inline marks.
- Undo and redo treat one user action as one transaction.
- A selection calculates state independently for each property. A range may,
  for example, report mixed alignment while reporting uniform line spacing and
  space-after values. Each property reports its canonical effective value when
  all selected blocks match and a distinct mixed state when they differ.
- Reset removes explicit paragraph style from every selected supported block
  in one transaction. It does not change inline marks, heading type, list
  structure, block identity, or selection. Each block then resolves its
  effective document defaults, including any future style-specific defaults
  accepted through governance.
- A completed toolbar or menu action returns focus to the editor.
- Pasted HTML cannot create paragraph data. Only canonical DRAFT attributes or
  a Rust-validated format importer may establish the serialized shape.

### DOCX mapping and lossiness

"Same-format save" means writing an externally opened or imported document
back to its original external format. It does not mean saving the authoritative
`.draft` document envelope.

| DRAFT property | DOCX paragraph property |
| :--- | :--- |
| Alignment | `w:jc` |
| Line spacing | `w:spacing` with `w:lineRule="auto"` and an integer `w:line` value measured in units of 1/240 of a line |
| Space before/after | `w:spacing` `w:before` and `w:after` |
| Left/right indent | `w:ind` `w:left` and `w:right` |
| First-line indent | `w:ind` `w:firstLine` |
| Hanging indent | `w:ind` `w:hanging` |

The conversion is deterministic. For automatic line spacing, DOCX stores
`w:line` in units of 1/240 of a line rather than ordinary twips. DRAFT
calculates the value as `240 * lineSpacingHundredths / 100`. The accepted
five-hundredths increment guarantees an integer result. The `exact` and
`atLeast` line rules use different semantics, remain outside the supported
subset, and must not be silently converted to automatic spacing. DRAFT must
not silently substitute or clamp unsupported values.

DOCX exact or at-least line rules, tab stops, contextual spacing, paragraph
borders, shading, pagination controls, and unsupported style inheritance are
outside this subset. Import classifies them under the accepted fidelity model.
Same-format save is denied when preserving them would require undisclosed loss.

Import distinguishes four cases:

- malformed DRAFT paragraph values are invalid and rejected;
- valid external values outside the DRAFT subset are classified as unsupported
  or lossy rather than corrupt;
- supported external values convert exactly into canonical DRAFT values; and
- an approximate conversion is prohibited unless the fidelity result records
  and presents the loss before mutation.

### Migration and compatibility

Paragraph formatting requires a named, ordered document-envelope migration
from schema version 1 to version 2. Loading a valid version 1 document does not
change its source bytes. The in-memory migration supplies the version 2 shape
without inventing explicit paragraph overrides. The first successful atomic
save writes version 2.

Migration is transactional and idempotent. Failure preserves source bytes,
registry state, related stores, temporary-file promotion state, and the
envelope version. Migration completes against a detached in-memory
representation before replacing registry state. No registry entry, durable
store, source file, target file, temporary promotion record, or envelope
version value may change unless the complete migration and atomic save succeed.
Lower unsupported and future versions continue to fail explicitly. No
best-effort field repair or guessed value conversion is allowed.

### Validation and failures

Validation rejects missing fields, unknown fields, wrong storage types,
fractional values, out-of-range values, conflicting special indents, and
unsupported block placement before persistence or export work.

Production boundaries use closed typed categories for invalid paragraph style,
unsupported paragraph feature, lossy format, and migration failure. Errors do
not contain document text, raw XML, arbitrary paths, or provider data. Visible
copy follows the accepted error-presentation and data-safety contracts.

### Complexity and resource bounds

- Validation is linear in the number of affected blocks.
- One user command creates one Tiptap transaction.
- A selection command traverses only the intersected supported blocks; a
  collapsed selection inspects only its current block.
- Migration is linear in document node count.
- Validation and migration do not recurse beyond the accepted document-envelope
  nesting limit.
- Existing document byte, node-count, nesting, and output limits apply before
  paragraph processing can allocate unbounded work.

Exact numeric document limits remain owned by the implementation configuration
contract. Paragraph formatting may narrow those limits but may not bypass or
silently increase them.

## Implementation Notes

The exact Rust and TypeScript type names are not binding in this proposal.
Implementation should keep policy, validation, editor coordination, and raw
DOCX XML work at separate abstraction levels. One canonical value table should
feed validation and format mapping rather than duplicate bounds across layers.

Phase 47 owns model, migration, import, export, and fidelity behavior. Phase 48
owns the later visible controls, native Paragraph menu integration, mixed-state
presentation, keyboard interaction, and focus behavior.

## Failure Modes

- Invalid serialized paragraph data is rejected before current state changes.
- Unsupported external paragraph behavior produces a typed fidelity result and
  cannot be silently dropped during same-format save.
- Migration failure leaves the original source and all durable state unchanged.
- Export failure leaves the DRAFT source unchanged and promotes no partial
  target.
- A stale or unavailable editor target cannot receive a paragraph command.
- Cancellation leaves selection, content, lifecycle state, and source identity
  unchanged.

## Tests

Acceptance requires evidence for:

- every valid value, boundary, malformed shape, unknown field, and conflicting
  indentation state in authoritative Rust validation and mechanically aligned
  TypeScript early validation;
- collapsed, ranged, multi-block, mixed, reset, copy/paste, focus, keyboard,
  undo, and redo behavior;
- editor JSON round trip plus save, close, and reopen;
- version 1 to version 2 migration, idempotence, and all non-mutation cases;
- deterministic DOCX XML properties, import, export, reopen, and compatible
  reader behavior for mixed paragraph formatting;
- no-edit external-source preservation and rejected-lossy-save preservation;
- typed failure presentation, bounded resource use, and stable ordering; and
- packaged manual validation of controls, defaults, mixed state, recovery, and
  DOCX fidelity.

No release row closes from structural checks or component presence alone.

## Non-goals

- A full Microsoft Word Paragraph dialog.
- Tabs, paragraph borders, shading, columns, section layout, or page geometry.
- Widow/orphan, keep-with-next, or other pagination controls.
- Arbitrary CSS, arbitrary HTML style import, or unbounded custom units.
- PDF output, legacy `.doc` support, or external office-suite conversion.
- Automatic repair of unsupported or malformed paragraph data.

## Related Documents

- `docs/adr/004-govern-paragraph-formatting.md`
- `docs/adr/003-expand-v1-document-interoperability.md`
- `docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/ARCHITECTURE.md`
- `docs/INVARIANTS.md`
- `docs/maintainers/DOCUMENTATION_COVERAGE.md`
