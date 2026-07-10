# PDF Export Decision Boundary

## Status

This guide records the Phase 33 proposal under review in ADR-001. The ADR is
`Proposed`, not accepted, and Phase 33 is not complete. The requirements in
`docs/drafts/PDF_EXPORT_DECISION.md` remain non-binding until they complete the
governance lifecycle in `docs/GOVERNANCE.md`.

## Proposed Decision

Defer native PDF generation until DRAFT has accepted the rendering policies and
verification contract needed for reliable output.

The proposal adds no product behavior. DOCX remains the only implemented export
foundation, and no visible DOCX or PDF workflow exists. There is no PDF library,
renderer, binary, font bundle, conversion process, command, Tauri capability,
frontend control, Python helper, network service, or packaged resource.

## Alternatives Reviewed

Native Rust PDF generation would keep authority in Rust but would require DRAFT
to define and maintain its own layout, font, accessibility, and renderer policy
before choosing a dependency.

HTML/CSS-to-PDF would reuse web layout concepts but would add print CSS,
browser-engine, font, pagination, packaging, accessibility, and platform
consistency requirements that are not accepted.

DOCX-to-PDF through an external office suite would build on Phase 32 output but
would depend on an external executable and version-specific conversion behavior
outside the trusted runtime boundary.

The operating-system print pipeline would use platform facilities but would not
provide one bounded, deterministic, parser-verifiable contract across supported
platforms.

Explicit deferral preserves the tested DOCX foundation and prevents a PDF claim
until those requirements can be enforced.

## Reconsideration Gate

PDF implementation cannot begin until all of these prerequisites are accepted:

- font selection, fallback, embedding, and licensing policy;
- pagination and layout model;
- accessibility expectations;
- cross-platform rendering strategy;
- dependency, security-update, and licensing review;
- parser-based output verification;
- bounded input, memory, page, time, and output-size limits;
- deterministic typed failure behavior; and
- explicit source-file and live-state preservation.

Later PDF work requires a separate implementation phase. If its renderer changes
an accepted architecture boundary, it also requires a new ADR. DOCX support is
not evidence that a PDF renderer meets these prerequisites.

## Deferral Guard

`scripts/check-invariants.sh` contains a named PDF export deferral guard. It
denies PDF export symbols and frontend claims plus known renderer dependencies,
conversion executables, and bundled runtime paths.

The guard preserves absence; it does not prove PDF behavior and does not make
ADR-001 accepted. Replacing or narrowing it requires governed implementation
evidence for parser validation, resource bounds, deterministic failures, and
source preservation.

## Governance Gate

The architecture PR must remain open for at least 24 hours, carry the
`architecture` label, include the required self-review, and pass local and
GitHub Actions verification. Before merge, verification must be rerun, ADR-001
must be updated to `Accepted`, and the living documents must record Phase 33 as
complete. Merge is the acceptance event.

Phase 34 implementation cannot begin until the Phase 33 PR merges and its
post-merge GitHub Actions run is green.
