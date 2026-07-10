# Formatting Check Boundary

## Status

This guide records implemented Phase 31 behavior. The requirements in
`docs/drafts/FORMATTING_CHECKS.md` remain non-binding until they complete the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 31 adds a pure Rust formatting domain over one explicit immutable
snapshot. It provides review-only style-consistency and heading-structure
findings without parsing a document, rendering citations, changing source data,
or claiming complete conformance with an external style manual.

The closed style identifiers are `apa7`, `mla9`, and
`chicago17_author_date`. They identify the style selected for one snapshot. At
this checkpoint, support means only that DRAFT can compare citation declarations
with the selected style consistently.

## Input Boundary

`FormattingSnapshot` contains one `FormattingStyle`, up to 512 ordered
`HeadingEntry` values, and up to 512 ordered `CitationStyleDeclaration` values.

A heading level is between 1 and 6. Its non-blank title is at most 512 UTF-8
bytes. A citation declaration reuses the existing case-sensitive citekey
validator and carries one closed style. Invalid entries fail before checking.

The snapshot contains no document ID, file path, Tiptap JSON, rendered HTML,
CSS, font, spacing, margin, page, bibliography, reference record, or mutation
instruction. A later integration must construct it from already validated data.

## Checks

`run_formatting_checks` performs exactly three checks:

| Code | Severity | Target | Review meaning |
| :--- | :--- | :--- | :--- |
| `FirstHeadingNotLevelOne` | Advice | First heading index | The outline begins below its top level. |
| `HeadingLevelSkipped` | Warning | Heading index | A heading is more than one level deeper than its predecessor. |
| `CitationStyleMismatch` | Warning | Citation index | A declaration differs from the selected document style. |

Valid sibling headings, ancestor transitions, and one-level descents produce no
finding. Citation-style comparison behaves the same for all three closed style
identifiers. The checker does not apply style-specific typography or citation
rules.

## Finding Boundary

Each `FormattingFinding` contains only a closed code, fixed Rust-owned severity,
fixed title and explanation, and a typed heading or citation index. Findings
contain no heading title, citekey, source text, rendered output, score,
replacement, patch, apply operation, path, or document identity.

Heading findings remain in source order and precede citation findings, which
also remain in source order. Equal valid snapshots produce equal results. A
finding asks for review; it does not certify that source content is wrong.

## Process And Ownership

Checking is synchronous, pure, process-local, and infallible after input
construction. It reads only the borrowed snapshot and allocates one bounded
result. It performs no filesystem, persistence, network, Python, worker, event,
Tauri, frontend, save, or export work.

No application state initializes this boundary. No command or visible workflow
can invoke it at this checkpoint.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `run_formatting_checks` | Coordinates outline and citation-style checks. |
| Mid | heading and citation finding builders | Apply deterministic review rules in target order. |
| Low | input validators and fixed policies | Enforce bounds, citekeys, identifiers, and content-free wording. |

## Verification

Twelve focused Rust tests cover all three style identifiers, valid snapshots,
first-heading and skipped-level boundaries, sibling and ancestor transitions,
style mismatches for every selected style, deterministic target order, heading
and collection limits, Unicode byte bounds, citekey validation, fixed policies,
and bounded content-free errors.

`scripts/check-invariants.sh` requires the source, guide, draft, constants,
identifiers, policy markers, and named tests. It rejects scoring, edit authority,
document or citation-node coupling, persistence, filesystem access, networking,
Python, worker spawning, Tauri commands, application initialization, and
frontend models. The Phase 32 DOCX-export absence gate remains active.

## Current Limits

The checker receives an explicit snapshot and does not extract headings or
citations from Tiptap. Style support is consistency-only: it does not implement
manual-specific title pages, running heads, margins, typography, capitalization,
punctuation, or bibliography rules. Findings are not persisted or visible, and
no document can be changed from a finding.

## Configuration Index

Snapshot counts, heading bounds, and closed style identifiers are indexed in
`docs/maintainers/CONFIGURATION.md`.
