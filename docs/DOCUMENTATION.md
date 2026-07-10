# DOCUMENTATION.md

## 1. Purpose

This document defines how the DRAFT app keeps documentation accurate, useful, and reviewable.

Documentation is part of the product. It protects future maintainers from guessing why code exists, protects users from unclear behavior, and protects the project from silent drift between implementation, architecture, and public claims.

This file does not replace `README.md`, `GOVERNANCE.md`, `INVARIANTS.md`, `ARCHITECTURE.md`, `CODING_STYLE.md`, or `CHANGELOG.md`. It explains how documentation is maintained across the repository.

## 2. Core Rule

Every meaningful change must leave the documentation in a true state.

If a change adds, removes, renames, or changes behavior in code, the author must check whether documentation also needs to change.

This applies to:

- public-facing behavior
- architecture boundaries
- invariants
- major functions
- major types
- important variables and constants
- user workflows
- source and citation workflows
- formatting behavior
- text-analysis behavior
- build and verification commands
- known limitations
- wiki knowledge articles

## 3. Documentation Surfaces

### Root public documents

Root documents are for readers who arrive at the repository without context.

Required root documents:

- `README.md` — public-facing product description.
- `LICENSE` — MIT license text.
- `CHANGELOG.md` — released notable changes only.

Root documents must stay plain enough for a new maintainer to understand before reading source code.

`AGENTS.md` contains local agent operating rules when present. Repository
policy intentionally ignores it, so a clean checkout does not require it.

### `/docs`

The `/docs` folder is the long-term knowledge base.

Use `/docs` for:

- `ARCHITECTURE.md`, `GOVERNANCE.md`, `INVARIANTS.md`, `CODING_STYLE.md`,
  `DOCUMENTATION.md`, `ROADMAP.md`, and `PHASEMAP.md`
- architecture notes that are too detailed for `ARCHITECTURE.md`
- accepted contract documents
- ADRs
- wiki knowledge articles
- user guides
- maintainer guides
- troubleshooting pages
- workflow explanations
- source and citation handling notes
- formatting and export behavior
- research-library behavior
- text-analysis behavior

Agents and maintainers must review `/docs` before making architectural, workflow, or user-facing changes.

Recommended structure:

```text
docs/
  ARCHITECTURE.md
  CODING_STYLE.md
  DOCUMENTATION.md
  GOVERNANCE.md
  INVARIANTS.md
  PHASEMAP.md
  ROADMAP.md
  maintainers/
  user/
  adr/          # when an ADR is required
  contracts/    # after the accepted-contract lifecycle
  drafts/       # while a contract is exploratory
  wiki/         # when a recurring support topic exists
```

## 4. Architecture Decisions

Architecture decisions are tracked through ADRs.

Use an ADR when a change alters:

- trust boundaries
- data ownership
- persistence behavior
- network behavior
- citation or source handling
- document save/export behavior
- Python helper authority
- Bash orchestration authority
- local/GitHub Actions verification
- user-data handling
- any invariant

ADRs belong in:

```text
docs/adr/
```

An ADR must explain:

- the problem
- the decision
- alternatives considered
- consequences
- enforcement
- affected docs

Do not use `ARCHITECTURE.md` as a history log. It describes the current accepted shape. ADRs explain why the shape changed.

## 5. Major Functions and Types

Major functions and types must be documented where a maintainer will look first.

A function is major when it:

- crosses the Tauri IPC boundary
- reads or writes files
- reads or writes SQLite
- calls the network client
- starts, stops, resumes, or cancels background work
- changes citation records
- changes document structure
- performs formatting or export work
- invokes a Python helper
- touches user-imported files or metadata
- enforces an invariant

A type is major when it:

- crosses a process boundary
- is serialized or persisted
- represents user data
- represents citation or source metadata
- represents command input or output
- represents an error returned to the frontend
- appears in a contract document

Required documentation:

- Rust: use `///` rustdoc for public functions, public structs, public enums, and command handlers.
- TypeScript: use explicit interfaces and TSDoc for exported types, Tauri command wrappers, and Tiptap extensions.
- Python: use docstrings for helper entry points and any function that reads, writes, transforms, or scores document text.
- Bash: use a short header comment explaining what the script does, what it writes, and whether it is safe to run locally.

Comments must explain why the operation exists, what boundary it protects, or what failure it prevents. Do not restate obvious syntax.

## 6. Important Variables and Constants

Important variables and constants must have names that explain their role.

Document a variable or constant when it controls:

- debounce windows
- retry counts
- timeout values
- rate limits
- token budgets
- text-analysis thresholds
- formatting rules
- citation style behavior
- export behavior
- file-size limits
- path allowlists
- schema versions
- invariant enforcement

Example:

```rust
/// Current citation-node schema accepted by the renderer.
/// Documents with a different version must enter migration handling.
pub const CITATION_NODE_SCHEMA_VERSION: u16 = 1;
```

Bad:

```rust
const V: u16 = 1;
```

## 7. User-Facing Documentation

User-facing documentation must describe what the app does in plain language.

It should answer:

- what the feature does
- what problem it solves
- what the user provides
- what DRAFT does automatically
- what DRAFT does not do
- what data stays local
- what requires network access
- what requires user action
- what happens when something fails

Do not expose internal terms unless they help the user understand a real action.

Prefer:

```text
DRAFT opens the source page in your browser. You sign in there. DRAFT never sees your school or publisher password.
```

Avoid:

```text
The external-service boundary prevents credential ingress into the core process.
```

## 8. Wiki Knowledge Articles

Wiki articles explain concepts, workflows, and recurring decisions.

Use wiki articles for knowledge that future contributors may need more than once.

Examples:

- how citation metadata is resolved
- how imported PDFs enter the pipeline
- how APA formatting is validated
- how text-analysis scoring works
- how offline behavior works
- how source reliability scoring is interpreted
- how document export differs from document save

Recommended article shape:

```markdown
# Title

## What this is

## Problem it solves

## How DRAFT handles it

## What can go wrong

## Related files

## Related ADRs
```

Wiki articles belong in:

```text
docs/wiki/
```

## 9. Contract Documents

Contract documents define exact behavior that code may rely on.

Use contract documents for:

- IPC command input/output
- error enums
- persisted schemas
- citation node schema
- reference-record schema
- document envelope schema
- network-client behavior
- Python helper input/output
- formatting output expectations
- text-analysis score formats

Accepted contracts belong in:

```text
docs/contracts/
```

Draft contracts may live in:

```text
docs/drafts/
```

A contract document must include:

- purpose
- scope
- non-goals
- normative requirements
- schema or type shape
- examples
- failure behavior
- invariants upheld
- enforcement

## 10. Public Claims

Public documentation must not overstate the product.

Do not claim DRAFT can do something unless the repository contains the implementation or the statement is clearly framed as a design goal.

Do not use public docs as a changelog.

Do not use public docs as a governance document.

Do not make security, privacy, citation, formatting, or AI-quality claims that are stronger than the architecture and tests support.

## 11. Documentation Update Triggers

Update documentation when a change affects:

- command names
- command inputs or outputs
- error variants
- saved file format
- citation node schema
- reference-record schema
- document export behavior
- source import behavior
- network-service behavior
- Python helper behavior
- Bash verification behavior
- local development commands
- GitHub Actions workflows
- user-visible labels or flows
- public-facing claims
- invariants
- architecture boundaries

When unsure, update the smallest relevant document.

## 12. Review Checklist

A documentation review should ask:

- Does the implementation still match `ARCHITECTURE.md`?
- Does the change affect an invariant in `INVARIANTS.md`?
- Does the change require an ADR?
- Does `/docs` need a new or updated article?
- Did any public-facing behavior change?
- Did any command, schema, or error type change?
- Did any setup, build, or verification command change?
- Are examples still accurate?
- Are links and file paths still valid?
- Are public claims still supported by code or tests?

A pull request should not merge when documentation is knowingly false.

## 13. Local and CI Verification

Documentation must be checkable where practical.

Local verification should include:

```bash
just docs-check
```

When `just` is unavailable, use the equivalent offline command:

```bash
bash scripts/check-docs.sh
```

The Phase 2 check verifies required document presence, top-level headings in
`/docs`, machine-specific path absence, and the changelog's no-`Unreleased`
rule. External URLs, Markdown anchor targets, ADR filenames, and contract
frontmatter are not checked yet because those surfaces do not exist or would
require broader tooling.

The Phase 3 `Verify` workflow runs `bash scripts/verify.sh`, which invokes
`scripts/check-docs.sh`. Documentation sanity therefore uses the same script
locally and in GitHub Actions rather than duplicating checks in workflow YAML.

Phase 4 established the user-facing workspace guide at `docs/user/WORKSPACE.md` and
React/Tiptap component tests through `npm test`. The aggregate verifier runs
that frontend suite before the language-specific build checks.

Phase 6 documented the first typed Tauri command in
`docs/maintainers/COMMAND_BOUNDARY.md`. Rustdoc describes the command DTOs,
error enum, domain status type, and boundary entry points. Rust tests and the
invariant script enforce the documented signature and serialization pattern.

Phase 7 documented the only raw frontend command adapter, typed runtime-status
wrapper, error classification, and transient connection hook in
`docs/maintainers/FRONTEND_COMMAND_CLIENT.md`. Wrapper tests and the invariant
scan enforce the documented IPC placement and boundary shapes.

Phase 8 documented Rust event emission, frontend payload validation, listener
ordering, finite lifecycle, and cleanup in
`docs/maintainers/EVENT_BOUNDARY.md`. Rust and frontend tests plus the invariant
scan enforce the event name, payload, placement, and lifecycle claims.

Phase 9 documented the process-local worker registry, cooperative cancellation
token, idempotent cancel command, typed frontend wrapper, and future worker
integration rules in `docs/maintainers/CANCELLATION_BOUNDARY.md`. Rust and
frontend tests plus the invariant scan enforce requested, repeated,
already-ended, malformed, unknown-worker, teardown, and shutdown behavior.

Phase 10 reconciles the bridge implementation guides, code examples,
repository layout, changelog state, invariant scripts, and CI-visible bridge
name parity. The checkpoint evidence is recorded in
`docs/maintainers/REALIGNMENT.md`. The current bridge guides are implementation
notes, not accepted contracts under the governance lifecycle.

Phase 11 documents the implemented Rust-owned version 1 envelope in
`docs/maintainers/DOCUMENT_ENVELOPE.md`. The readiness requirements remain in
`docs/drafts/DOCUMENT_ENVELOPE.md` until the governance lifecycle permits an
accepted contract. Rust tests and the invariant scan enforce schema version,
required fields, typed failures, root shape, and structured JSON round trips.

Phase 12 documents the process-local Rust document registry in
`docs/maintainers/DOCUMENT_REGISTRY.md`. Rust tests and the invariant scan
enforce one live handle per document, typed duplicate and unknown-close
failures, close/reopen behavior, independent documents, and concurrent-open
serialization without introducing a file or frontend lifecycle.

Phase 13 documents the Rust-owned native-dialog, validated load, explicit
snapshot save, registry source-path, and atomic replacement flow in
`docs/maintainers/DOCUMENT_SAVE_LOAD.md`. Rust and frontend tests plus the
invariant scan enforce typed command shapes, malformed-file rejection,
round-trip behavior, Rust-only path authority, and the absence of direct
write-to-target calls.

Phase 14 extends that guide with interruption checkpoints, replacement-failure
cleanup, serialized file lifecycle operations, typed write-stage failures, and
disk/registry concurrency behavior. The invariant scan requires the matching
Rust and frontend tests locally and in GitHub Actions.

Phase 15 reconciles document-core guides, architecture, invariant enforcement,
user-facing workspace truth, tracked repository shape, and local/CI parity.
The checkpoint evidence is recorded in `docs/maintainers/REALIGNMENT.md`, and
the bounded Phase 16 readiness requirements live in the non-binding
`docs/drafts/REFERENCE_RECORD.md`.

Phase 16 documents the implemented Rust-owned version 1 reference record in
`docs/maintainers/REFERENCE_RECORD.md`. Rust tests and the invariant scan enforce
declared fields, nested validation, provenance semantics, typed failures,
structured JSON round trips, Rust-only authority, and the absence of Phase 17
persistence.

Phase 17 documents the implemented Rust-owned SQLite reference store in
`docs/maintainers/REFERENCE_STORE.md`. Store tests and invariant scans enforce
schema initialization, migration dispatch, transactional CRUD, uniqueness,
reopen behavior, corruption handling, and SQLite confinement.

Phase 18 documents the implemented citation-node boundary in
`docs/maintainers/CITATION_NODE.md`. Rust/frontend tests and invariant scans
enforce exact attrs, nested envelope validation, pre-mutation open/save
rejection, managed-store resolution, typed IPC, fail-closed Tiptap states,
stale-response suppression, and the absence of embedded reference metadata.

Phase 19 documents the Rust-owned bibliography-consistency check in
`docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md`. The non-binding semantics remain
in `docs/drafts/BIBLIOGRAPHY_CONSISTENCY.md`. Rust tests and invariant scans
enforce missing, orphaned, duplicate, repeated-citation, case-sensitive,
deterministic, no-side-effect, and no-frontend-authority behavior.

Phase 20 reconciles the citation/reference source-of-truth model, Phase 19
semantics, public and internal documentation, verification scripts, repository
shape, and hosted CI evidence. The audit is recorded in
`docs/maintainers/REALIGNMENT.md`, and bounded Phase 21 readiness requirements
live in the non-binding `docs/drafts/NETWORK_CLIENT.md`.

Phase 21 documents centralized Rust network-client construction in
`docs/maintainers/NETWORK_CLIENT.md`. Rust tests and invariant scans enforce
controlled User-Agent and timeout policy, HTTPS-only configuration, managed
startup state, bounded failures, no request execution, and no ad hoc client
authority outside the accepted network module.

Recommended checks:

- Markdown formatting
- dead-link detection for local links
- heading style validation
- required root-document presence
- required `/docs` structure presence
- ADR filename validation
- contract frontmatter validation
- invariant reference validation

Documentation checks must not require network access unless the check is explicitly marked as external-link validation.

## 14. Documentation Tone

Use plain language.

Prefer short sentences.

Define technical terms the first time they matter.

Use examples when a rule affects behavior.

Explain what a rule protects or prevents.

Do not write decorative documentation. Write documentation that helps the next reader make the correct change safely.
