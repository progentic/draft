# Bibliography Consistency Requirements Draft

## Status

This document is a non-binding Phase 19 requirements draft. Implemented
behavior is recorded separately in
`docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md` once the phase is complete. This
draft does not become an accepted contract without the review lifecycle in
`docs/GOVERNANCE.md`.

## Scope

Phase 19 owns one Rust domain check that compares citation nodes in a validated
document with a candidate bibliography made from validated reference records.
It reports missing, orphaned, and duplicate citekeys without changing the
document or the reference library.

The candidate bibliography is an explicit record list selected for one
document. It is not the complete cross-document reference library. Phase 19
must not classify every uncited library record as an orphan.

## Consistency Semantics

- A missing citekey appears in the document but not in the candidate
  bibliography.
- An orphaned citekey appears in the candidate bibliography but not in the
  document.
- A duplicate citekey appears more than once in the candidate bibliography.
- Repeated in-text citations to the same source are valid and count as one
  cited citekey for consistency comparison.
- Citekeys remain case-sensitive, matching the citation-node and reference-store
  contracts.
- Each result list is unique and sorted by citekey for deterministic output.
- A report is consistent only when all three result lists are empty.

The categories are independent. An uncited citekey that appears twice in the
candidate bibliography is both orphaned and duplicate.

## Ownership

Rust owns citation extraction and consistency logic. The check receives a
validated `DocumentEnvelope` and validated `ReferenceRecord` values. It does
not accept paths, raw database rows, frontend-owned metadata, or embedded
bibliography data.

The reference library remains the metadata source of truth. Citation nodes
continue to store only `schema_version`, `citekey`, and `render_style`.

## Failure Behavior

Citation extraction reuses the Rust citation validator. If a future internal
change allows invalid citation data to reach the checker, the operation fails
closed with the citation path and typed validation cause instead of omitting
the node.

## Verification

Phase 19 tests must cover:

- a fully consistent document and bibliography;
- missing citekeys;
- orphaned citekeys;
- duplicate bibliography citekeys;
- repeated in-text citations that are not treated as duplicates;
- independent overlapping categories;
- deterministic case-sensitive ordering; and
- an empty document with an empty bibliography.

Local verification and GitHub Actions must run the same tests and invariant
scan.

## Explicit Non-Goals

Phase 19 does not add:

- bibliography formatting or rendered entries;
- citation insertion or editing controls;
- a bibliography field in the document envelope;
- a scan of the complete reference library for one document;
- reference CRUD or bibliography IPC;
- persistence, database schema changes, or document mutation;
- network lookup, import, export, background jobs, or Python helpers; or
- APA, MLA, Chicago, or another style engine.
