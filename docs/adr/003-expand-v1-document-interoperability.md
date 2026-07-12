# ADR-003: Expand v1 Document Interoperability

Date: 2026-07-12
Status: Proposed
Deciders: @progentic

## Context

New v1 document-interoperability and native desktop workflow requirements
conflict with the current release sequence because Phases 47 through 50 assume
that Phase 46 can hand directly to usability validation, security review, a
final candidate, and release realignment.

Manual review of the packaged Phase 46 artifact recorded in draft PR #36 found
that DRAFT's current boundary is not yet a credible academic document editor:

- Markdown is imported as literal source instead of editable structure.
- DOCX, RTF, and OpenDocument import are unavailable.
- Imported files cannot be saved safely to their original format.
- The application has no explicit lossiness or round-trip ownership model.
- Primary actions exist only in the WebView and are not integrated with native
  macOS menus.
- Command grouping, editor composition, and visible branding do not yet meet
  the accepted desktop-product usability threshold.

These are not minor Phase 46 corrections. Importing and saving external
formats changes document ownership, compatibility, source-preservation, and
failure policy. Native menus also require one state-aware action path shared
with visible controls and keyboard shortcuts. Absorbing those commitments into
Phase 46 would mix a broad architecture change into an already reviewable
workflow and accessibility phase.

The following alternatives were considered:

- **Ship the existing boundary:** Keep native `.draft`, literal text and
  Markdown import, and DOCX export. This preserves schedule but does not meet
  the owner's v1 academic-document workflow requirement.
- **Expand Phase 46:** Add parsers, round-trip persistence, native menus, and a
  visual redesign directly to PR #36. This would erase the phase boundary,
  make its evidence difficult to review, and couple unrelated failure modes.
- **Use an external office suite:** Delegate conversion through LibreOffice or
  another installed application. This introduces a runtime dependency,
  licensing review, version variability, platform discovery, and opaque
  conversion failures.
- **Support every requested format, including legacy `.doc`:** This treats the
  binary Word format as equivalent to ZIP/XML formats and adds a substantially
  different parser and compatibility burden.
- **Insert bounded interoperability and desktop-workflow phases:** Preserve
  Phase 46, implement format ownership separately, then integrate native
  desktop actions before usability, security, candidate, and release work.

## Decision

Propose two release-blocking phases after Phase 46 and move the existing
release sequence accordingly:

| Phase | Proposed purpose |
| :--- | :--- |
| 47 | Document interoperability |
| 48 | Desktop UI and native workflow integration |
| 49 | Usability and performance validation |
| 50 | Documentation and drift realignment |
| 51 | Security review |
| 52 | Final release candidate |
| 53 | v1.0.0 release |

Phase 50 remains the mandatory fifth-phase documentation and drift
realignment checkpoint. The proposal does not renumber or reopen completed
Phases 0 through 46.

### Format boundary

DRAFT remains authoritative for native `.draft` documents. Phase 47 must add
bounded, format-aware behavior for the following external formats:

- **Text:** deterministic paragraph and line-break import. Save-back is
  permitted only while the document remains plain-text compatible.
- **Markdown:** parsed headings, emphasis, lists, quotations, links, code, and
  separators for one explicit supported subset. Save-back is permitted only
  while the document remains representable by that subset.
- **DOCX:** import and safe round-trip save for one explicit supported subset,
  with content classified as faithfully represented, approximated, preserved
  but not editable, or unsupported.
- **ODT and RTF:** each must receive either bounded import/save support or a
  separate accepted v1 deferral with accurate user guidance before Phase 47
  closes.
- **Legacy `.doc`:** explicitly unsupported in v1.0.0. Users must convert it to
  a supported modern format outside DRAFT. This ADR does not authorize an
  external office-suite runtime or conversion dependency.

Unsupported structures must not disappear silently. A parser or writer must
reject the operation, preserve the structure without claiming editability, or
disclose a deterministic approximation or loss before a write is allowed.

### Round-trip ownership

Rust owns external source identity, native format, save capability,
round-trip status, lossiness state, target selection, validation, and every
filesystem mutation. The frontend receives only typed presentation state and
cannot retain a source path as persistence authority.

The lifecycle distinguishes:

- an opened external document that may be saved to its original format when
  the current content remains safely representable;
- an imported external document whose source is read-only and whose first save
  requires a new target;
- a native DRAFT document saved as `.draft`; and
- an exported copy that never changes source ownership.

Opening and closing an external document without edits must not rewrite its
source bytes. Exact byte identity after an edit is not required, but the
supported structure, appearance, and compatibility contract must pass
format-specific fixtures. When current content cannot be represented safely,
same-format save-back is denied and the user must use Save As to a compatible
format. DRAFT never silently flattens or substitutes content.

### Native desktop workflow

Phase 48 must expose every primary visible DRAFT workflow through the native
macOS menu bar with conventional labels, grouping, shortcuts, and
state-sensitive enablement. Native menu items, visible toolbar controls, and
keyboard shortcuts dispatch through one shared frontend action layer; they do
not implement actions independently.

Editor-only actions may target the active Tiptap editor. Document identity,
file dialogs, import, persistence, export, reference storage, and other trusted
operations continue through typed Rust commands. Menu state must follow the
active document, focus, editor history, available references, findings, and
conflicting lifecycle operations.

Phase 48 also owns the visible icon correction, command grouping, hierarchy,
spacing, alignment, editor/outline composition, and responsive overflow needed
for a coherent desktop editor. It is not permission to copy a full office
suite ribbon or add unsupported controls.

### Documentation comprehension

Adopt this standing engineering principle:

> Write documentation so that a competent engineer who has never seen DRAFT
> can understand what the subsystem does, why it exists, and how to change it
> safely before reading the implementation.

Documentation optimizes human comprehension first and precision second by
separating plain-language explanation, technical explanation, and normative
specification. Proposed `INV-UX-07` records the intended protection but remains
Proposed until existing major maintainer guides are realigned and structural
heading enforcement exists.

Phase 49 would include documentation terminology and comprehension review in
addition to product usability and performance. Phase 50 would review plain
language, terminology consistency, maintainer onboarding, unnecessary
implementation jargon, and documentation cross-links before accepting
`INV-UX-07` through a separate governed status change.

### Release-gate sequence

`RC-01` through `RC-04` and `GATE-46` remain open and continue to belong to
Phase 46. Acceptance of this ADR does not close them.

The successor release contract must replace the old future numbering without
recording false closure:

- `RC-07` and `GATE-47` cover document interoperability.
- `RC-08` and `GATE-48` cover desktop UI and native workflow integration.
- `GATE-49` covers first-time-user usability and measured/perceived
  performance.
- `GATE-50` covers the mandatory drift realignment.
- `RC-05` and `GATE-51` cover security and secure usability.
- `RC-06` closes only through the exact packaged candidate in Phase 52.
- Phase 53 may publish v1.0.0 only after every prior row is closed with its
  named evidence.

The existing open `GATE-47` and `GATE-48` rows are not considered closed by
renumbering. If this ADR is accepted, their old meanings are replaced through
the governed release-contract update before Phase 47 implementation begins.

## Consequences

DRAFT gains a reviewable path to credible academic-document interoperability
without weakening Rust authority or source preservation. Each external format
receives an explicit compatibility boundary, and native menus become a real
application action surface rather than a second implementation.

The v1 schedule becomes longer. Format parsers and writers require dependency
and licensing review, hostile and representative fixtures, resource bounds,
format-specific typed failures, compatibility testing, and packaged macOS
validation. DOCX round-trip fidelity cannot be inferred from the current DOCX
export compiler. ODT and RTF need separate decisions if they are deferred.

This decision makes several things harder:

- document state must distinguish in-memory identity from source-format
  ownership and save capability;
- lossiness must be computed and communicated before mutation;
- no-edit preservation and edited-document fidelity require different tests;
- native and WebView actions must remain synchronized as focus and state
  change;
- existing maintainer guides need layered plain-language review before
  documentation readability can become an accepted invariant;
- first-time-user and performance evidence moves to a later phase; and
- security, candidate, and release work move from Phases 48 through 50 to
  Phases 51 through 53.

Affected downstream surfaces include `ARCHITECTURE.md`, `INVARIANTS.md`,
`ROADMAP.md`, `PHASEMAP.md`, the accepted v1 usability contract, the release
candidate ledger, document lifecycle and export guides, configuration and
coverage indexes, Wiki/user limitations, and documentation, invariant, and
release-candidate enforcement.

## Enforcement

While this ADR is proposed, it is non-binding. Phase 46 remains active, draft
PR #36 remains unmerged, and no interoperability, round-trip, native-menu, or
visual-redesign implementation may rely on this proposal.

The proposal guard requires ADR and draft-contract proposal language, the
exact Phase 47 through 53 sequence, preservation of Phase 50 as realignment,
open current release rows, and Proposed `INV-UX-07` language. It rejects production source surfaces for
external-document lifecycle authority, round-trip/lossiness state, format
parsers, save-back commands, and native menu dispatch while the ADR remains
Proposed.

After acceptance, Phase 47 and Phase 48 must replace absence checks with
behavioral tests. Required evidence includes parser and serializer limits,
format fixtures, no-edit source hashes, lossless and rejected-lossy saves,
atomic replacement, frontend path denial, shared menu/toolbar dispatch,
state-sensitive enablement, conventional shortcuts, keyboard/focus behavior,
packaged interaction, visible icon validation, and exact release-ledger gate
mapping. Local verification and GitHub Actions must run the same enforcement.

## Links

- `ARCHITECTURE.md` §4.1, §4.2, §6, and §11
- `INVARIANTS.md` `INV-03`, `INV-06`, `INV-09`, and `INV-UX-01` through
  `INV-UX-06`
- `docs/drafts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/contracts/V1_USABILITY_ACCEPTANCE.md`
- `docs/maintainers/RELEASE_CANDIDATE.md`
- Draft PR #36 manual evidence, `UX-46-008` through `UX-46-015`
