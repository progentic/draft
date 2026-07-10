# Formatting Review UX

## Status

This guide records implemented Phase 34 behavior. It integrates the pure Phase
31 checks into the transient workspace without changing the formatting domain,
document persistence, or export boundaries.

## User Workflow

The formatting toolbar opens one review band inside the document workspace.
The user selects APA 7, MLA 9, or Chicago 17 author-date and starts a bounded
check. The selected identifier controls only the consistency checks documented
in `FORMATTING_CHECKS.md`; it does not certify complete style-manual
conformance.

Findings appear under Structure or Citations. Every finding can be inspected or
dismissed for the current result set. A heading finding may expose one
Rust-owned target level. The user must choose that action before Tiptap changes
the heading. Citation mismatches remain inspect-only.

## Boundary Flow

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| High | `FormattingReviewPanel`, `useFormattingReview` | Coordinate visible state, runs, and explicit actions. |
| Mid | `collectFormattingSnapshot`, target guards | Build bounded inputs and require the same current editor node. |
| Low | `runFormattingReview`, `run_formatting_review` | Validate IPC and return Rust-owned findings and closed actions. |

The request contains only the selected style, ordered heading levels and
titles, and validated citation citekeys and declared styles. It contains no
path, document identity, reference record, credential, export target, or
arbitrary style value. Rust validates the request again before running the
existing pure checker.

## Stale Result Protection

Each run records a unique frontend run ID and the current editor generation.
An editor update invalidates the active run. A newer run supersedes an older
one. Inspect and apply also verify that the indexed target still maps to the
same node type, position, heading level and text, or citation attributes.

Missing, moved, or changed targets make the review stale. A stale result cannot
advance selection or change a heading. Dismissal is transient and creates no
suppression record.

## Visible States And Failures

The panel represents idle, running, ready, stale, and failed states. Command
codes are mapped to bounded user messages in `FormattingReviewPanel.tsx`.
Invalid responses and transport failures have separate generic messages. Raw
errors, paths, titles, citekeys, and document text are not displayed from a
failure object.

User recovery guidance lives in `docs/wiki/Troubleshooting.md`. The complete
configuration and limit index is `docs/maintainers/CONFIGURATION.md`.

## Verification

Rust tests prove the action policy and typed command contract. Frontend tests
cover exact IPC validation, all typed command errors, generation races,
missing and remapped targets, explicit actions, citation restrictions,
keyboard navigation, accessible labels, and visible states.

`scripts/check-invariants.sh` keeps the Phase 31 checker pure and denies
persistence, filesystem, export, PDF, network, Python, worker, and automatic
document authority in the Phase 34 bridge.

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline formatting::
npm test -- --run src/ipc/formattingReview.test.ts src/features/formatting-review src/App.test.tsx
bash scripts/check-invariants.sh
```

## Current Limits

Findings are advisory, transient, and limited to heading structure and citation
style declarations. DRAFT does not implement full APA, MLA, or Chicago rules,
citation conversion, bibliography formatting, layout, margins, fonts, title
pages, persistence, automatic repair, or an export control in this workflow.
