# ADR-001: Defer Native PDF Export

Date: 2026-07-10
Status: Proposed
Deciders: @iangordon

## Context

DRAFT can compile a bounded document subset into a deterministic DOCX package,
but that foundation does not settle the separate requirements for reliable PDF
output. Native PDF export needs accepted policies for pagination, layout, font
selection and embedding rights, accessibility, cross-platform rendering,
resource limits, licensing, verification, and release maintenance.

Adding PDF output before those policies exist could create an unreliable export
path, an undeclared platform dependency, or output that appears complete while
silently losing document meaning. The constraint protecting source safety and
truthful capability claims still applies because DRAFT cannot yet validate a PDF
renderer against an accepted output contract.

The following alternatives were considered:

- **Native Rust PDF generation:** Keeps authority in Rust, but requires DRAFT to
  define and maintain its own accepted layout, font, accessibility, and rendering
  model before choosing a library.
- **HTML/CSS-to-PDF:** Reuses web layout concepts, but introduces browser-engine,
  print-CSS, font, pagination, accessibility, packaging, and platform-consistency
  requirements that are not accepted.
- **DOCX-to-PDF through an external office suite:** Builds on the DOCX artifact,
  but depends on an external executable and version-specific conversion behavior
  outside the current trusted runtime boundary.
- **OS print pipeline:** Uses platform facilities, but does not provide one
  bounded, testable, deterministic contract across supported platforms.
- **Explicit deferral:** Preserves the current DOCX foundation and prevents an
  unsupported PDF claim until the prerequisite policies are accepted.

## Decision

Defer native PDF generation until the prerequisite rendering policies are
accepted.

DOCX remains DRAFT's supported export foundation. No PDF dependency, bundled
resource, runtime path, command, process, frontend control, or capability claim
may be added under this decision. Later PDF work requires a new implementation
phase after all of these conditions are met:

- an accepted font-selection, fallback, embedding, and licensing policy;
- an accepted pagination and layout model;
- accepted accessibility expectations;
- an accepted cross-platform rendering strategy;
- dependency, security-update, and licensing review;
- parser-based output verification;
- bounded input, memory, page, time, and output-size limits;
- deterministic, typed failure behavior; and
- explicit preservation of source-file bytes and live source state.

Reconsidering PDF export requires a new ADR if the selected implementation
changes an accepted architecture boundary. The implementation must remain
separate from DOCX compilation and must not treat DOCX support as evidence that
PDF support is complete.

## Consequences

DRAFT does not produce PDF files at this checkpoint and must not present PDF as
an available workflow. The repository gains no PDF library, renderer, binary,
font resource, conversion process, command, or UI control.

The existing DOCX compiler remains the only export foundation. Because no
visible export workflow exists yet, this decision does not add or change a
public download capability.

Deferral keeps source preservation, offline operation, and capability claims
honest while rendering policy is unresolved. It also avoids committing releases
to an engine whose packaging, licensing, output fidelity, accessibility, or
security-update obligations have not been evaluated.

This decision makes immediate PDF availability harder. Future implementation
must satisfy the prerequisite policies and verification evidence instead of
selecting a convenient converter in isolation. The affected downstream
documents and enforcement are `ARCHITECTURE.md`, `INVARIANTS.md`, `ROADMAP.md`,
`PHASEMAP.md`, `docs/maintainers/TOOLCHAIN.md`,
`docs/maintainers/PDF_EXPORT_DECISION.md`, and
`scripts/check-invariants.sh`.

## Enforcement

The Phase 33 invariant scan denies PDF export types, compilers, commands, and
runtime paths. The accepted living documents must identify that scan as the PDF
deferral guard and continue to state that no PDF workflow exists. Local
`just verify` and `just check-invariants` results must match GitHub Actions.

The guard may be removed or narrowed only in the governed PR that accepts a PDF
implementation decision and adds parser-based output tests, bounded-resource
tests, deterministic failure tests, and source-preservation tests.

## Links

- `ARCHITECTURE.md` §3.3 and §11
- `INVARIANTS.md` `INV-04`, `INV-09`, and `INV-11`
- `docs/drafts/PDF_EXPORT_DECISION.md`
