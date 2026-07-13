# PDF Export Decision Requirements Draft

## Status

This is a non-binding Phase 33 decision draft. Phase 33 must produce an ADR and
record implemented or deferred behavior separately in
`docs/maintainers/PDF_EXPORT_DECISION.md`. This draft does not become an accepted
contract without the lifecycle in `docs/GOVERNANCE.md`.

## Purpose

PDF export introduces font, pagination, layout, rendering-engine, packaging,
licensing, and cross-platform behavior that DOCX export does not settle. DRAFT
must make an explicit architectural decision before adding a PDF dependency or
runtime path. A placeholder, hidden conversion process, or untested platform
assumption is not an implementation.

## Required Decision

Phase 33 completes exactly one of these outcomes:

1. Accept an ADR for a bounded Rust-owned PDF architecture and implement the
   approved minimum with tests; or
2. accept an ADR that defers PDF export, records the blocking constraints, keeps
   the mechanical absence gate, and documents the current DOCX-only limit.

The ADR must compare at least:

- direct PDF generation from the validated DRAFT document;
- conversion from the deterministic Phase 32 DOCX artifact;
- platform-native print or office automation;
- a bundled local rendering engine; and
- explicit deferral.

The comparison must address supported platforms, offline behavior, font
selection and embedding rights, pagination consistency, accessibility metadata,
determinism, package size, security updates, licensing, failure isolation, test
evidence, and release maintenance.

## Implementation Gate

If the ADR selects implementation, Phase 33 must define and enforce:

- Rust-owned bounded input from a validated immutable document or Phase 32
  artifact;
- no shell, office automation, WebView printing, network service, credential,
  Python helper, or user-supplied executable;
- a pinned rendering dependency and explicit font/resource policy;
- complete in-memory generation before atomic `.pdf` target replacement;
- no source-document or document-registry mutation;
- closed content-free failures and post-replacement durability handling;
- parser-based validation of the PDF header, trailer, page tree, text, and bounds;
- deterministic output where the selected engine can guarantee it, with any
  unavoidable metadata variation stated explicitly; and
- local/GitHub Actions parity on every supported platform.

The implementation must reject unsupported document content rather than omit it
silently. It must not claim complete APA, MLA, Chicago, citation, bibliography,
or accessibility support that has not been implemented and tested.

## Deferral Gate

If the ADR selects deferral, Phase 33 adds no PDF library, binary, resource,
command, process, file writer, or frontend control. The ADR and current docs must
state:

- why reliable PDF output is not supportable at this checkpoint;
- which technical or licensing decisions remain unresolved;
- that DOCX is the only implemented derived format;
- that the visible DOCX workflow does not imply PDF support; and
- what evidence is required to reconsider the decision.

The Phase 33 PDF absence gate remains active and is relabeled as an accepted
deferral guard. Deferral does not remove PDF from the product direction; it
prevents an unsafe or misleading implementation.

## Shared Safety Requirements

Both outcomes must preserve these truths:

- the DRAFT document remains the source of truth;
- export cannot change source bytes or live registry state;
- failures before replacement preserve any prior complete target;
- no generated citation marker is presented as a final rendered citation;
- no external service receives document content; and
- no UI claims PDF support before a tested backend path exists.

## Verification

The Phase 33 change must include:

- one ADR with alternatives, consequences, enforcement, and affected docs;
- architecture, invariant, roadmap, phasemap, toolchain, and maintainer updates;
- an explicit implementation or deferral test/scan gate;
- no unreviewed dependency or generated binary drift;
- unchanged README unless a real downloadable user-facing PDF capability exists;
  and
- local/GitHub Actions parity.

## Non-Goals

Phase 33 does not add formatting-review UI, DOCX UI, broad layout controls,
templates, citation rendering, bibliography rendering, cloud conversion,
printing, release packaging, or Phase 34 accept/reject workflows. The phase is
an architectural PDF decision plus only the implementation specifically approved
by that decision.
