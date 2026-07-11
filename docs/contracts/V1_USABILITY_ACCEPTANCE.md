---
status: Accepted
adr: None - refines the accepted Phase 45 release rule
upholds: [INV-UX-01, INV-UX-02, INV-UX-03, INV-UX-04, INV-UX-05, INV-UX-06]
owners: [frontend, core, release]
---

# DRAFT v1.0.0 Usability Acceptance

## Purpose

This accepted contract turns the binding Phase 45 usability rule into
executable release criteria for Phases 46 through 50. It defines the supported
workflow, required human evidence, measurable thresholds, and release-blocking
finding classes.

> DRAFT v1.0.0 is not releasable unless a first-time user can understand the
> application's purpose, identify the primary controls, complete the supported
> document workflow, recognize the application's current state, and recover
> from ordinary failures without maintainer guidance.

Passing tests proves that DRAFT behaves as implemented. It does not prove that
users can understand or successfully use it. v1.0.0 requires both mechanical
correctness and demonstrated user comprehension.

This contract does not reopen Phase 45, accept ADR-002, close an `RC-*` or
`GATE-*` row, or claim that a currently absent workflow exists.

## Required Qualities

The supported workflow must be:

1. **Discoverable:** a first-time user can find the required action without
   maintainer guidance.
2. **Understandable:** labels, descriptions, and results communicate what the
   action does in plain language.
3. **Predictable:** behavior follows consistent desktop-editor conventions and
   does not surprise the user with hidden side effects.
4. **Recoverable:** a failure explains what happened, whether document data is
   safe, and which currently available action is a valid next step.

## Supported v1 Workflow

The authoritative release workflow is:

1. Launch DRAFT.
2. Create a document or open a supported DRAFT document.
3. Enter content and apply supported formatting.
4. Save the document.
5. Close and reopen the document.
6. Add or resolve a supported citation.
7. Run formatting and local text-analysis checks.
8. Understand a finding and locate its relevant passage.
9. Correct the document through an explicit user action.
10. Export a supported DOCX document.
11. Confirm that export did not change the DRAFT source.

The analysis step remains blocked while ADR-002 is Proposed. This contract does
not authorize Phase 46 to rely on that proposal. If ADR-002 is accepted, the
local text-analysis portion is limited to its exact five-check scope.

## Phase 46 - Understandable Visible Workflows

Phase 46 must close `RC-01` through `RC-04` before `GATE-46` may close. Every
new visible workflow must satisfy the supported-workflow and accessibility
requirements together; interaction clarity cannot be postponed to Phase 47.

For local text analysis, each of the five allowed checks must:

- have one plain-language name;
- explain what pattern it detects;
- describe findings as review signals rather than authoritative conclusions;
- let the user locate the relevant passage;
- distinguish informational review from action that may be needed;
- provide an explanatory empty state;
- expose visible pending, completion, and typed failure states;
- preserve the source document on failure and offer only valid recovery; and
- use consistent terminology in menus, controls, panels, errors,
  accessibility labels, and documentation.

No label may imply artificial intelligence, semantic reasoning, quality
scoring, originality detection, or model-backed interpretation. Analysis
interaction tests must cover labels, empty states, findings, relevant-passage
navigation, focus behavior, announcements, and recovery.

## Phase 47 - Usability And Perceived Performance Validation

Phase 47 audits the complete visible application and records evidence in
`docs/maintainers/V1_USABILITY_EVIDENCE.md`. It must not close `GATE-47` from
benchmarks alone.

### Visible-language inventory

Inventory every user-visible string in menus, toolbar controls, buttons,
dialogs, panels, status indicators, empty states, tooltips, errors, and
notifications. Classify unclear terminology, implementation language,
inconsistent naming, ambiguous verbs, unexplained abbreviations, unsupported
capability claims, and controls whose result is not predictable.

### Menu and control audit

Verify that:

- File contains document lifecycle and export actions;
- Edit contains editing actions;
- View contains layout and panel controls;
- Help contains user assistance and troubleshooting;
- important actions are not represented by unexplained icons alone;
- similar actions have clearly different names;
- disabled controls explain why they are unavailable;
- keyboard shortcuts follow supported-platform conventions; and
- destructive or irreversible actions are unmistakable.

### State visibility

The visible application must clearly distinguish ready, opening, saving,
saved, unsaved, checking, exporting, offline, failed, and completed states.
No supported operation may leave the user unsure whether it started or
completed.

### Perceived performance

Measure:

- time until the interface responds;
- time until progress feedback appears;
- whether controls appear frozen;
- whether repeated activation can duplicate an operation;
- whether the document remains interactive during safe background work;
- whether cancellation is available when the operation supports it; and
- whether completion is visibly and accessibly announced.

### Realistic scenarios

Run the supported workflow with an empty document, a short document, a long
academic paper, many headings, many citations, narrow windows, display
scaling, keyboard-only navigation, reduced motion, and offline mode. Record
both measured limits and observed ambiguity.

## First-Time-User Task Validation

Before Phase 47 closes, at least five people who have not worked on the
repository must attempt the workflow without coaching. Participants do not
need to be professional usability researchers. Do not explain a confusing
control and then count the task as successful.

Each participant must attempt to:

1. explain what DRAFT is for;
2. create or open a document;
3. format a heading;
4. save, close, and reopen the document;
5. locate the citation workflow;
6. run a document check;
7. explain one finding in their own words;
8. correct the finding;
9. export the supported document; and
10. recover from one controlled failure.

Record anonymized results for task completion, assistance, elapsed time,
incorrect actions, hesitation, misunderstood labels, missing expected
controls, confidence that work remained safe, and willingness to use DRAFT
again. Do not record participant names, contact details, document contents, or
other unnecessary personal data.

## Measurable Release Thresholds

### Critical tasks

Every participant must create or open a document, save it, recognize unsaved
changes, close and reopen it, understand whether export succeeded, and recover
without losing work. Any data-loss event, unexplained failure, or inability to
save is a release blocker.

### Supported workflow

At least 80 percent of participants must complete each non-critical supported
task without coaching.

### Terminology

At least 80 percent of participants must correctly explain what each primary
control does before activating it.

### Recovery

At least 80 percent of participants must identify a valid next step from each
tested error message without assistance.

### Satisfaction

After the workflow, participants rate these statements on a five-point
agreement scale:

- DRAFT was easy to understand.
- I felt confident that my document was safe.
- The names of controls matched what they did.
- I would use DRAFT for this kind of work.

Any statement with a median below 4 creates a Phase 47 finding that requires
disposition before Phase 49. These thresholds are practical v1 gates, not a
claim of statistical research.

## Phase 48 - Secure Usability

The Phase 48 security review must include a secure-usability section proving
that:

- security restrictions explain why an action is unavailable;
- offline behavior is understandable;
- no secret, token, internal path, or raw payload appears in the interface;
- warnings distinguish actual danger from ordinary limitations;
- permission and boundary failures offer a safe available recovery action;
- security controls never silently discard user work;
- packaging and trust warnings are accurate; and
- users are never instructed to weaken system security to run DRAFT.

## Phase 49 - Packaged Release-Candidate Gate

Phase 49 must rerun the complete workflow using the exact release-candidate
package rather than a development server. Evidence must cover packaged launch,
first run, primary workflow completion, labels and menus, keyboard-only use,
realistic documents, analysis comprehension, export, controlled-error
recovery, and source preservation.

The package must contain no unsupported active control, stale placeholder,
maintainer language, open `UX-0`, or open `UX-1` finding.

Use this blocker taxonomy:

- **UX-0:** data loss, inaccessible critical action, or inability to complete
  the primary workflow.
- **UX-1:** misleading label, hidden primary control, unrecoverable confusion,
  or unsupported capability claim.
- **UX-2:** meaningful friction that should be fixed before release.
- **UX-3:** enhancement suitable for later maintenance.

Any open `UX-0` or `UX-1` blocks Phase 49. Every `UX-2` requires an explicit
fix, accepted limitation, or owner-approved deferral recorded with rationale.

## Phase 50 - Release Entry Point

Before tagging v1.0.0, Phase 50 must provide:

- a concise first-run guide;
- a Start Here workflow;
- clear supported and unsupported capability wording;
- a keyboard-shortcut reference;
- recovery and troubleshooting guidance;
- release notes written for users;
- a verified download and launch path; and
- no active control labeled as coming soon.

The first-run material must fit on one screen or a short guided sequence. It
must not require architecture or maintainer documentation.

## Evidence And Enforcement

`docs/maintainers/V1_USABILITY_EVIDENCE.md` is the cumulative evidence ledger
for Phases 46 through 49. It must separate automated evidence, packaged-browser
or application evidence, human task results, findings, dispositions, and exact
tested commit/package identifiers.

Create the ledger only when real Phase 46 evidence exists. It must use headings
from `## Phase 46` through `## Phase 49` as those phases run. Each phase section
must contain `### Automated Evidence` and `### Findings And Dispositions`;
Phase 47 must also contain anonymized task and threshold results, Phase 48 must
contain secure-usability evidence, and Phase 49 must contain
`### Packaged Workflow Evidence`.

Finding rows use this shape:

```text
| ID | Severity | Status | Evidence | Disposition |
```

IDs are unique. Severity is exactly `UX-0`, `UX-1`, `UX-2`, or `UX-3`.
Status is `Open` or `Closed`. A closed `UX-2` row must name its fix, accepted
limitation, or owner-approved deferral and rationale.

Mechanical enforcement is owned by:

- `scripts/check-docs.sh` for document presence and cross-links;
- `scripts/check-invariants.sh` for the six UX invariant definitions;
- `scripts/check-release-candidate.sh` for open-gate integrity and conditional
  evidence requirements; and
- `scripts/verify.sh` for local and hosted parity.

Human comprehension evidence cannot be replaced by source scans or automated
tests. Automated checks must not claim that absence of a prohibited string
proves discoverability or understanding.

## Non-Goals

This contract does not require feature parity with a full office suite,
statistically representative research, speculative product controls, a visual
redesign, model-backed analysis, or implementation during a governance cooling
period. It requires familiar behavior, obvious primary actions, consistent
vocabulary, visible state, and justified confidence that user work is safe.
