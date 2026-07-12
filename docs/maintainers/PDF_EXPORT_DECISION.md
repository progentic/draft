# PDF Export Decision Boundary

## Status

This guide records the accepted Phase 33 decision in ADR-001. The ADR is
`Accepted`, Phase 33 is complete, and PDF export remains mechanically absent.
The requirements in `docs/drafts/PDF_EXPORT_DECISION.md` remain a non-binding
record of the decision gate rather than an implementation contract.

## Accepted Decision

Defer native PDF generation until DRAFT has accepted the rendering policies and
verification contract needed for reliable output.

The decision adds no PDF product behavior. DOCX remains the only implemented
export format and Phase 46 exposes its existing bounded exporter. No PDF
library, renderer, binary, font bundle, conversion process, command, Tauri
capability, frontend control, Python helper, network service, or packaged
resource exists.

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

The guard preserves absence; it does not prove PDF behavior. Replacing or
narrowing it requires governed implementation evidence for parser validation,
resource bounds, deterministic failures, and source preservation.

## Acceptance Record

PR #1 carried the `architecture` label, recorded the required self-review and
alternatives, preserved the reviewed head, passed local and pull-request
verification, and passed an exact prospective merge-tree verification. It
merged as `5587866`, which accepted the decision under `GOVERNANCE.md`. The
post-merge `main` Verify run also passed before this completion record began.

The repository owner authorized this exception in PR #1:

> **One-time owner override**
>
> The repository owner is manually waiving the remaining portion of the 24-hour cooling period for ADR-001.
>
> This is a one-off decision for this PR only. It does not change `GOVERNANCE.md`, weaken the standing cooling-period rule, or create a reusable expedited path.
>
> Rationale:
>
> * ADR-001 has remained open and unchanged long enough for review.
> * The alternatives and consequences have already been documented.
> * Hosted verification is green.
> * No new PDF requirement or contradictory architecture has appeared.
> * The unresolved PR is blocking the active roadmap path.
>
> The override is authorized directly by the repository owner.
>
> ADR-001 may proceed through final review and merge. Future architectural PRs remain subject to the full cooling period unless separately authorized and documented.

The override is historical evidence for this decision only. It does not amend
governance or authorize an expedited path for another architectural PR.
