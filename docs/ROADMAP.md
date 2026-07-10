# DRAFT — Roadmap

**Purpose:** Guide DRAFT from a blank repository to a production-ready `v1.0.0` release.

This roadmap describes the intended development path. It is not a changelog,
release note, governance record, or detailed status report. Released work is
recorded in `CHANGELOG.md`; phase evidence is recorded in
`docs/maintainers/REALIGNMENT.md`. Architecture changes are governed by
`GOVERNANCE.md`, `INVARIANTS.md`, and accepted ADRs.

**Current execution checkpoint:** Phases 0 through 22 are complete. Phase 23 is
the next implementation phase; external browser handoff has not started.

---

## 1. Product Direction

DRAFT is a local-first desktop workspace for research-heavy writing.

It helps the user research sources, analyze claims, format documents, and review language quality without losing control of the document. The core product promise is not “AI writes for you.” The promise is that the app helps the author keep the chain between claims, sources, structure, and final output visible and reviewable.

The product grows through four capability tracks:

- **Document Research:** source discovery, citation records, bibliographies, reference-library workflows, and provenance.
- **Analysis:** summarization, argument review, fact-checking support, voice consistency, and source reliability scoring.
- **Formatting:** APA, MLA, Chicago, headings, layout consistency, document export, and document-ready structure.
- **Text-analysis:** grammar, syntax, tone, clarity, cohesion, and linguistic quality checks.

The implementation grows through four technical surfaces:

- **Rust core:** trusted runtime, persistence, filesystem access, network access, secrets, background jobs, save/export, and helper orchestration.
- **TypeScript, React, and Tiptap frontend:** editor surface, UI state, document interaction, source cards, sidebars, and user controls.
- **Python helper workers:** allowlisted formatting and text-analysis helpers invoked by Rust through typed contracts.
- **Bash automation:** local development and GitHub Actions orchestration only.

---

## 2. Release Policy

DRAFT uses pre-1.0 semantic versioning while the product is taking shape.

- `v0.0.x` is used for documentation-only changes, repository housekeeping, wording updates, minor corrections, and non-behavioral maintenance.
- `v0.x.0` is used for normal product, workflow, architecture, feature, implementation, and behavior changes.
- `v1.0.0` is the first production release line. It requires stable save/load, source handling, citation behavior, formatting/export behavior, security boundaries, local verification, GitHub Actions parity, and user-facing documentation.

No `[Unreleased]` changelog section is used. Release entries are prepared directly under the next version heading in `CHANGELOG.md`.

---

## 3. Development Milestones

### Milestone 0 — Repository Baseline

Goal: Create the public project shape before product implementation begins.

Expected foundation:

- Root-level public README.
- MIT license.
- Governance, architecture, invariants, coding style, documentation policy, agent policy, roadmap, and phasemap.
- `.gitignore` for Rust, Tauri, TypeScript, Python, Bash, local data, generated outputs, and tooling caches.
- Initial changelog at `v0.0.0`.

Version band: `v0.0.x`.

---

### Milestone 1 — Buildable Application Skeleton

Goal: Create the smallest buildable DRAFT application.

Expected product shape:

- Tauri 2 shell starts locally.
- Rust core compiles.
- TypeScript frontend compiles.
- React app renders a minimal DRAFT workspace.
- Tiptap editor mounts with a basic document surface.
- Local verification command exists.
- GitHub Actions runs the same meaningful checks as local development.

Version band: first `v0.x.0` line.

---

### Milestone 2 — Document Core

Goal: Make a local document safe to open, edit, save, reload, and protect from corruption.

Expected product shape:

- Document envelope schema.
- Tiptap JSON save/load path.
- Rust-owned document registry.
- Single-live-handle rule for open documents.
- Atomic save behavior.
- Offline editing and saving.
- Negative-path tests for malformed documents and interrupted saves.

Version band: `v0.x.0`.

---

### Milestone 3 — Reference Library and Citation Foundation

Goal: Connect document text to a structured local source library.

Expected product shape:

- Reference-record schema.
- Local reference persistence.
- Citation node contract.
- Citation schema versioning and migration error path.
- In-text citation insertion.
- Bibliography generation from local records.
- Consistency checks between in-text citations and bibliography entries.

Version band: `v0.x.0`.

---

### Milestone 4 — Document Research Workflows

Goal: Support safe source discovery and import without crossing credential or scraping boundaries.

Expected product shape:

- Crossref, Semantic Scholar, and Unpaywall metadata lookup through Rust.
- Centralized network client.
- Rate limiting and backoff.
- User-agent policy.
- External browser handoff for publisher, institutional, and Google Scholar access.
- Watched-folder and explicit PDF import.
- Stable-write confirmation before import processing.
- Source provenance tracking.

Version band: `v0.x.0`.

---

### Milestone 5 — Analysis and Text-analysis

Goal: Add writing intelligence without hiding evidence or weakening human control.

Expected product shape:

- Rust-owned AI orchestration.
- Context-window assembly policy.
- Streaming output with explicit cancellation.
- Analysis output clearly separated from verified source evidence.
- Python helper-worker contract for deterministic text-analysis routines.
- Grammar, clarity, tone, cohesion, and voice-consistency checks.
- Reviewable issue cards instead of silent rewrites.

Version band: `v0.x.0`.

---

### Milestone 6 — Formatting and Export

Goal: Produce document-ready output without making export the source of truth.

Expected product shape:

- APA, MLA, and Chicago style enforcement path.
- Heading and structure checks.
- Layout consistency checks.
- DOCX export.
- PDF export if supported by the accepted export architecture.
- Export failure does not corrupt source document state.
- Formatting warnings remain explainable to the user.

Version band: `v0.x.0`.

---

### Milestone 7 — Hardening, Packaging, and Release Candidate

Goal: Prepare the application for production use.

Expected product shape:

- Installer/package path for supported platforms.
- Clean onboarding path.
- Local data location policy.
- Backup and recovery guidance.
- Error messages mapped to user actions.
- Secrets stored only in OS-native credential storage.
- E2E tests for critical workflows.
- Release candidate checklist.

Version band: late `v0.x.0`.

---

### Milestone 8 — v1.0.0 Production Release

Goal: Release a stable production line.

Expected product shape:

- Core document workflows are stable.
- Research, citation, formatting, and text-analysis workflows are documented.
- Invariants are enforced locally and in GitHub Actions.
- Public user documentation is complete enough for a new user to understand the app.
- Maintainer documentation is complete enough for a future maintainer to continue development safely.
- Known limits are explicit.

Version: `v1.0.0`.

---

## 4. Drift Realignment Rule

Every fifth phase is a documentation and drift realignment phase.

Required realignment phases:

- Phase 5
- Phase 10
- Phase 15
- Phase 20
- Phase 25
- Phase 30
- Phase 35
- Phase 40
- Phase 45
- Phase 50

A drift realignment phase does not exist to add features. It exists to stop architectural decay before it becomes expensive.

Each drift realignment phase must check:

- `README.md`
- `ARCHITECTURE.md`
- `GOVERNANCE.md`
- `INVARIANTS.md`
- `CODING_STYLE.md`
- `DOCUMENTATION.md`
- `AGENTS.md`
- `CHANGELOG.md`
- `/docs/adr`
- `/docs/contracts`
- `/docs/wiki`
- `/docs/user`
- `/docs/maintainers`
- local verification scripts
- GitHub Actions workflows
- tests that enforce invariants

The expected output is not more prose for its own sake. The expected output is reduced drift between what the code does, what the docs claim, and what the build actually enforces.

---

## 5. v1.0.0 Release Gate

DRAFT may be called `v1.0.0` only when these conditions hold:

- The app can create, open, edit, save, close, and reopen a document without data loss.
- Save behavior is atomic and tested against interruption.
- The reference library is the citation metadata source of truth.
- Citation nodes are schema-versioned and rejected or migrated safely when incompatible.
- In-text citations and bibliographies can be checked for consistency.
- External metadata lookup uses documented APIs only.
- Institutional and publisher credentials never enter DRAFT process memory or storage.
- The frontend never calls external network services directly.
- Python helpers are invoked only through typed Rust-owned contracts.
- Bash is limited to local development and CI orchestration.
- Long-running user-initiated workers support explicit cancellation or documented non-cancelable terminal behavior.
- The app has defined offline behavior.
- Local verification and GitHub Actions enforce the same critical boundaries.
- User-facing documentation explains core workflows in plain language.
- Maintainer documentation explains architecture, contracts, tests, and release procedure.
- Known limitations are written down instead of hidden.
