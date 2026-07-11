# Error UX Requirements Draft

## Status

This is the non-binding requirements draft for Phase 39. It defines the next
implementation boundary after Phase 38. It is not an accepted contract under
`docs/GOVERNANCE.md`.

## Purpose

DRAFT needs clear recovery actions for failures that are already visible in
the workspace. Typed Rust and TypeScript errors must remain distinguishable
through presentation state so the interface can explain what happened without
showing raw runtime details.

## Scope

Phase 39 maps existing visible runtime-status, connectivity, formatting-review,
and citation-rendering failures to concise messages and available next actions.
It may add shared presentation helpers only when they remove real duplication
across those current surfaces.

The phase must start from the typed codes already exposed by each command or
state boundary. Unknown command, invalid-response, and transport failures remain
separate fallbacks. A new visible message must have one documented recovery
action or state-preservation rule.

## Interaction Contract

- messages identify the failing action, not an internal subsystem;
- retry is shown only when repeating the action is safe;
- retained state remains visible when a failed change did not take effect;
- status announcements use appropriate live-region behavior without stealing
  focus;
- keyboard and screen-reader users receive the same action and state; and
- repeated failures do not create duplicate or unbounded notices.

## Redaction And Authority

Visible errors must not include document content, evidence, prompts, findings,
paths, URLs, secrets, credential presence, native error text, transport bodies,
logs, or stack traces.

Phase 39 does not let the frontend inspect Rust state directly. It adds no new
filesystem, network, secret, persistence, worker, telemetry, or diagnostic
collection authority.

## Acceptance Tests

Phase 39 must prove:

- every known error code for each integrated visible surface has one mapping;
- adding a known code without presentation coverage fails an exhaustive test;
- messages for distinct causes do not imply the wrong recovery action;
- unknown, invalid-response, and transport fallbacks remain available;
- retry and dismiss controls use correct button semantics and accessible names;
- announcements are bounded, deduplicated, and do not move focus;
- no typed but unwired backend failure receives speculative visible copy; and
- local and GitHub Actions verification run the same tests and scans.

## Non-Goals

Phase 39 does not expose document open/save, reference CRUD, provider login,
metadata lookup, PDF import, analysis, text-analysis cards, DOCX export,
diagnostic reports, support submission, logs, or crash reporting. It does not
redesign the workspace or claim release-wide error handling is complete.
