# Bibliography Consistency

## Status

This is an implemented Phase 19 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/BIBLIOGRAPHY_CONSISTENCY.md` until the contract lifecycle is
complete.

## Scope

`src-tauri/src/citations/bibliography.rs` compares citation nodes in one
validated document with an explicit candidate bibliography of validated
reference records. It returns a deterministic report and performs no mutation.

Phase 19 does not add a bibliography to the document envelope. It does not list
the complete reference store automatically, because the store is shared across
documents and uncited library records are not necessarily bibliography errors.

No command, frontend control, rendered bibliography, style engine, database
schema, network path, import path, export path, job, or Python helper is added.

## Input Boundary

`check_bibliography_consistency` receives:

- one validated `DocumentEnvelope`; and
- one borrowed slice of validated `ReferenceRecord` values selected as the
  candidate bibliography for that document.

Rust remains the only durable citation-validation authority. The checker
reuses the citation-node traversal from `citations::node`, and the reference
records retain their existing validated case-sensitive citekeys.

The function does not accept raw JSON records, database rows, paths, frontend
metadata, or a `ReferenceStore`. A future caller must decide which records form
the candidate bibliography before invoking the pure consistency check.

## Report Semantics

`BibliographyConsistencyReport` contains three unique sorted lists:

| List | Meaning |
| :--- | :--- |
| `missing_citekeys` | Citekeys used by citation nodes but absent from the candidate bibliography. |
| `orphaned_citekeys` | Candidate bibliography citekeys not used by a citation node. |
| `duplicate_citekeys` | Citekeys represented more than once in the candidate bibliography. |

Repeated in-text citations are expected scholarly behavior. Citation
occurrences are reduced to a set before comparison, so citing one source more
than once does not create a duplicate finding.

Bibliography counts remain separate from citation membership. An uncited
citekey repeated in the candidate bibliography is both orphaned and duplicate.

Comparison is case-sensitive, matching the citation node, reference record,
and reference store. `Alpha2025` and `alpha2025` are different citekeys.

`is_consistent` returns true only when every result list is empty.

## Deterministic Ordering

The checker uses ordered sets and maps for citekey membership and counts. Every
result list is therefore deduplicated and sorted without depending on document
order, bibliography input order, hash randomization, or database order.

Document traversal still preserves citation order internally. Phase 19 uses
set membership only because the consistency result describes citekey
agreement, not citation placement.

## Failure Behavior

The document envelope already rejects invalid citation nodes during creation,
open, and save. The consistency checker still uses the fallible Rust citation
collector rather than assuming raw nested data is valid.

If a future internal change breaks the envelope invariant, the checker returns
`BibliographyConsistencyError::InvalidCitation` with the structural path and
typed `CitationNodeError`. It does not omit the invalid node or produce a
partially trusted report.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `check_bibliography_consistency` | Coordinates citation extraction, bibliography counting, and report creation. |
| Mid | `collect_cited_citekeys` | Converts validated citation attrs into document membership. |
| Mid | `count_bibliography_citekeys` | Counts candidate records without changing their metadata. |
| Mid | `build_report` | Applies missing, orphaned, and duplicate semantics. |
| Low | ordered set difference and count filtering | Produces unique deterministic citekey lists. |

## Verification

Eight focused Rust tests cover:

- matching citations and records;
- missing citekeys;
- orphaned citekeys;
- duplicate bibliography citekeys;
- repeated in-text citations;
- overlapping orphaned and duplicate categories;
- deterministic case-sensitive ordering; and
- empty inputs.

The citation-node tests also require document-order collection from nested
Tiptap content.

`scripts/check-invariants.sh` requires the module, test source, all named Phase
19 tests, and ordered collection primitives. It rejects filesystem, SQLite,
reference-store, Tauri command, and frontend bibliography authority in this
phase.

`scripts/check-repository.sh` requires the new Rust sources to remain visible
to Git. `scripts/check-docs.sh` requires the draft and maintainer guide and
requires roadmap/phasemap agreement through Phase 20. The same checks run
through `scripts/verify.sh` locally and in the GitHub Actions `verify` job.

## Phase 20 Audit

Phase 20 audits the citation/reference source-of-truth model, Phase 19
semantics, documentation, tests, scripts, repository shape, and local/CI
parity. The evidence is recorded in `docs/maintainers/REALIGNMENT.md`; the
realignment adds no unrelated product behavior.
