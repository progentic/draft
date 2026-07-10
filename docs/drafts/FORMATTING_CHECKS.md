# Formatting Check Requirements Draft

## Status

This is a non-binding Phase 31 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/FORMATTING_CHECKS.md`. This draft does
not become an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`.

## Purpose

DRAFT needs a small, deterministic formatting domain before export or visible
review workflows are added. The first boundary must identify explainable style
and outline inconsistencies without changing a document or claiming complete
conformance with an external style manual.

## Scope

Phase 31 creates a pure Rust formatting check over one explicit immutable
snapshot. The snapshot declares one closed document style, an ordered heading
outline, and ordered citation-style declarations supplied by trusted Rust code.

The supported style identifiers are:

- `apa7`
- `mla9`
- `chicago17_author_date`

Supporting an identifier means DRAFT can check that the snapshot uses that
declared style consistently. It does not mean DRAFT renders complete citations,
validates every rule in the named manual, or certifies a document as compliant.

## Input Boundary

`FormattingSnapshot` contains:

- one `FormattingStyle`;
- up to 512 ordered `HeadingEntry` values; and
- up to 512 ordered `CitationStyleDeclaration` values.

Each heading has a one-based level from 1 through 6 and a non-blank title of at
most 512 UTF-8 bytes. Each citation declaration has a citekey accepted by the
existing reference validator and one `FormattingStyle`. Constructors reject an
invalid entry or excessive collection before checks run.

Phase 31 does not parse arbitrary Tiptap JSON or infer formatting from rendered
HTML, CSS, fonts, spacing, or page layout. A later integration must construct the
snapshot only from already validated document and citation data.

## Checks

`run_formatting_checks` returns deterministic review findings for exactly these
conditions:

| Code | Target | Meaning |
| :--- | :--- | :--- |
| `first_heading_not_level_one` | First heading index | The outline begins below level 1. |
| `heading_level_skipped` | Heading index | The heading is more than one level deeper than the preceding heading. |
| `citation_style_mismatch` | Citation index | The citation declaration differs from the snapshot's selected style. |

The first two checks are style-independent structure checks. The third applies
equally to APA 7, MLA 9, and Chicago 17 author-date snapshots. Phase 31 adds no
style-specific typography, title-page, running-head, margin, capitalization,
punctuation, bibliography-rendering, or page-layout rules.

## Finding Boundary

One `FormattingFinding` contains only:

- a closed finding code;
- a fixed Rust-owned severity, title, and explanation; and
- a typed heading or citation index.

Findings are ordered by target collection, target index, and code. At most one
finding is emitted for one check and target. Results contain no source text,
rendered citation, score, replacement, patch, apply instruction, file path, or
document identity.

Findings are advisory. A finding means the user should review a declared style
or outline relationship; it does not prove that the source document is wrong.

## Failure Behavior

Input validation returns bounded typed errors for excessive heading or citation
collections, an invalid heading level or title, or an invalid citekey. Errors
contain no heading title, citekey, document text, path, or serialized snapshot.

Checking valid input is pure and infallible. Equal snapshots produce equal
ordered findings.

## Non-Destructive Boundary

Phase 31 does not read or write files, mutate a document or citation node, save
or export content, persist findings, call Python, call the network, start a
worker, emit an event, register a Tauri command, or expose frontend state.

Any future accepted-change workflow must require explicit user action and use
the existing Rust-owned document validation and save boundaries. Export remains
owned by Phases 32 and 33, and visible finding review remains owned by Phase 34.

## Verification

Tests and scans must cover:

- all three closed style identifiers;
- first-heading and skipped-level boundaries;
- multiple valid sibling and ancestor transitions;
- citation-style consistency and mismatches for each style;
- input collection, heading-title, level, and citekey bounds;
- deterministic ordering and fixed Rust-owned wording;
- no source text, score, replacement, patch, or apply authority;
- no document parsing or mutation, persistence, filesystem, export, Python,
  network, worker, Tauri, event, or frontend authority; and
- local/GitHub Actions parity.

Phase 31 must replace the formatting absence gate in
`scripts/check-invariants.sh` with behavioral tests and authority scans.

## Non-Goals

Phase 31 does not implement complete APA, MLA, or Chicago conformance; citation
or bibliography rendering; citation-node schema expansion; Tiptap extraction;
page layout; font, spacing, margin, title-page, or running-head rules; DOCX or
PDF export; automatic repair; finding persistence; accepted edits; Tauri IPC;
frontend controls; or visible formatting issue cards.
