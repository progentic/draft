# Formatting UX Requirements Draft

## Status

This is the non-binding requirements draft for the next implementation phase,
Phase 34. ADR-001 is accepted, the Phase 33 architecture PR is merged, and its
post-merge GitHub Actions run is green. Implemented behavior must be recorded
separately in `docs/maintainers/FORMATTING_UX.md`.

## Purpose

DRAFT needs a visible review surface for the narrow formatting findings created
in Phase 31. The workflow should help users understand and act on a structural
or style-consistency issue without claiming complete style-manual compliance or
changing document content silently.

## Scope

Phase 34 groups current findings into two review sections:

- **Structure:** first-heading and skipped-heading-level findings.
- **Citations:** citation-style declaration mismatches.

The frontend may start one bounded check against an immutable snapshot, display
Rust-owned finding wording, focus the related heading or citation, and record a
transient user choice for the current run. Results remain advisory and
non-persistent.

## Command Boundary

One typed Rust command may accept only the bounded Phase 31 snapshot extracted
from an already validated current document and its validated citation data.
Rust runs the existing pure checks and returns content-free findings with typed
targets and closed actions.

The command must not receive a filesystem path, raw reference record, credential,
export target, or arbitrary style identifier. It must not save, export, call the
network, invoke Python, start a detached worker, or mutate a document.

## Review Actions

Every finding supports **Inspect**. Inspecting a heading finding focuses the
corresponding heading. Inspecting a citation mismatch focuses the corresponding
citation without changing its render style.

Heading findings may offer an explicit **Apply level** action only when Phase 34
can map the typed target to the same current Tiptap node. The user must trigger
the edit, Tiptap must perform it through the normal editor transaction, and the
existing save boundary remains the only path to durable document change.

**Reject** or **Dismiss** hides a finding for the current result set only. It
does not persist a suppression rule. Citation mismatches remain inspect-only
until a later phase implements complete citation rendering for the selected
style.

## Stale Result Protection

Each check run must be tied to the current editor generation and a unique
frontend run identifier. If the document changes, a newer run starts, or the
target no longer maps to the same node, the old result becomes stale and no
apply action may run.

The frontend must never infer that an indexed target still identifies the same
content after an editor change. Stale findings may be discarded or visibly
marked for rerun, but they cannot mutate the document.

## User Experience

The review surface must:

- state the selected APA 7, MLA 9, or Chicago 17 author-date identifier;
- avoid language that certifies complete style compliance;
- group findings by Structure and Citations;
- show the fixed explanation and affected target for each finding;
- provide inspect, apply, reject, or dismiss only where the closed action allows;
- represent empty, running, ready, stale, and failed states; and
- remain usable with keyboard navigation and readable labels.

The interface must not expose internal indexes, serialized document content,
file paths, citekeys, or raw Rust errors as user-facing text.

## Acceptance Tests

Phase 34 must prove:

- all three supported style identifiers render without claiming full compliance;
- structure and citation findings appear in the correct groups and order;
- inspect focuses the typed current target without changing document content;
- a permitted heading-level action requires explicit user input;
- stale, missing, or remapped targets reject an apply action;
- reject and dismiss affect only the current transient result set;
- citation mismatches have no apply action;
- command and UI failures remain typed and content-free;
- no finding persistence, export, PDF, network, Python, worker, or filesystem
  authority is added; and
- local and GitHub Actions verification use the same tests and boundary scans.

## Non-Goals

Phase 34 does not implement complete APA, MLA, or Chicago rules; citation or
bibliography rendering; automatic citation-style conversion; finding
persistence; reusable suppression rules; page layout; fonts; margins; title
pages; DOCX or PDF controls; export dialogs; document save controls; Python
formatting; network calls; background work; or automatic document repair.
