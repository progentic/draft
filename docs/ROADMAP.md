# DRAFT — Roadmap

**Purpose:** Guide DRAFT from a blank repository to a production-ready `v1.0.0` release.

This roadmap describes the intended development path. It is not a changelog,
release note, governance record, or detailed status report. Released work is
recorded in `CHANGELOG.md`; phase evidence is recorded in
`docs/maintainers/REALIGNMENT.md`. Architecture changes are governed by
`GOVERNANCE.md`, `INVARIANTS.md`, and accepted ADRs.

**Current execution checkpoint:** Phases 0 through 45 are complete. Phase 46 is
the next implementation and interaction-clarity phase.

Phase 24 completed a Rust-only PDF intake gate. It validates explicit files and
supplied watched-file observations but adds no watcher, persistent job, worker,
queue, or visible import workflow. Phase 25 audited that boundary without
adding product behavior.

Phase 26 promotes one validated candidate identity into one durable,
recoverable PDF import job. It implements lifecycle state only; PDF processing,
watcher execution, workers, networking, reference mutation, and UI remain
deferred.

Phase 27 adds a provider-independent Rust analysis boundary for bounded context
assembly, typed generated output, and cooperative cancellation. Production
model providers, credentials, external requests, persistence, start commands,
frontend listeners, and visible analysis workflows remain deferred.

Phase 28 adds a Rust-owned, versioned Python helper protocol with a fixed
allowlist, bounded standard streams, isolated process execution, timeout,
cancellation, and typed failures. Its protocol probe is not a product
text-analysis feature; typed findings remain deferred to Phase 29.

Phase 29 adds five deterministic, explainable text-review finding types for
grammar, clarity, tone, cohesion, and voice. Rust validates closed codes and
UTF-8 ranges and owns all review wording. Findings remain non-persistent and
cannot edit a document; visible issue cards remain deferred.

Phase 30 audits the analysis, cancellation, event, and Python-helper boundaries
without adding product behavior. Phase 31 requirements are bounded in advance
to pure, review-only style-consistency and heading-structure checks.

Phase 31 adds those pure Rust checks for APA 7, MLA 9, and Chicago 17
author-date declarations plus heading structure. It does not parse or change a
document, render citations, claim complete style conformance, persist findings,
or expose a workflow. Phase 32 requirements now bound the DOCX export foundation.

Phase 32 adds a strict Rust-only DOCX compiler and atomic export service. It
preserves supported paragraphs, headings, text, hard breaks, and inline marks;
unsupported content and citations fail instead of disappearing. No visible
export workflow exists. Phase 33 decided PDF export through ADR-001 without
adding a PDF dependency or runtime path.

Accepted ADR-001 selects explicit deferral. It compares native Rust generation,
HTML/CSS rendering, DOCX conversion, operating-system printing, and deferral,
then keeps PDF unavailable until rendering and verification policies are
accepted. Phase 33 completed after the architecture PR passed local, exact
merge-tree, pull-request, and post-merge GitHub Actions verification. The
repository owner documented a one-time, non-precedential waiver of the remaining
cooling period in PR #1; `GOVERNANCE.md` and its standing rule are unchanged.

Phase 34 adds a transient formatting review band to the workspace. Users can
run the bounded Phase 31 consistency checks, inspect grouped findings, dismiss
them for the current run, and explicitly apply a permitted heading level. Run
generation and target checks prevent stale findings from changing the editor.
This workflow does not claim complete style conformance or add persistence,
automatic repair, citation conversion, or export controls.

Phase 35 reconciles formatting and export documentation with the implemented
Phase 31 through Phase 34 boundaries. It records the live Wiki publication,
corrects command/client and visibility drift, and strengthens documentation
checks without changing product behavior. Non-binding Phase 36 offline-mode
requirements are bounded in `docs/drafts/OFFLINE_MODE.md` before implementation
begins.

Phase 36 adds a Rust-owned online/offline session policy with a visible header
toggle. Offline mode denies new metadata requests and system-browser handoffs
before external work while local editing and formatting review remain
available. The mode is not persisted, does not probe operating-system
connectivity, and adds no retry queue, telemetry, proxy, credential, or secret
storage behavior. Phase 37 remains a separate native secret-storage boundary.

Phase 37 adds a lazy Rust-owned service API-key store backed by Keychain,
Credential Manager, or Secret Service. Secret values are bounded, zeroized on
drop, and never cross Tauri, frontend, Python, config, SQLite, filesystem,
environment, or logging boundaries. No key, provider, command, settings
control, or credential prompt is added. Non-binding Phase 38 audit and
diagnostics requirements are bounded in `docs/drafts/AUDIT_DIAGNOSTICS.md`.

Phase 38 adds one explicit Rust-owned local diagnostic snapshot. It reports
only the compiled application version, six existing contract versions, and six
closed startup/non-probe subsystem states under a 2 KiB serialized limit. It
does not inspect content, paths, logs, credential presence, external services,
the filesystem, persistence, Python, or background state. No visible control,
report export, upload, telemetry, or support workflow is added. Non-binding
Phase 39 error-UX requirements are bounded in `docs/drafts/ERROR_UX.md`.

Phase 39 maps only runtime-status, connectivity, formatting-review, and
citation-rendering failures that already reach visible frontend surfaces. Each
known variant has stable bounded copy and a retryable, actionable, or terminal
disposition. Existing connectivity and formatting controls keep native
semantics, repeated failures replace one message without moving focus, and
typed-but-unwired backend failures remain unexposed.

Phase 40 reconciles security, offline, diagnostic, error-presentation, and user
documentation without adding product behavior. Its evidence is recorded in
`docs/maintainers/REALIGNMENT.md`.

Phase 41 adds one crate-level critical-path test over the existing Rust
document, reference, citation, and DOCX boundaries. It proves create, save,
close, reopen, duplicate-open rejection, citation resolution, explicit
citation-export rejection, supported DOCX export, package reopening, and source
preservation. It adds no visible workflow or production authority.

Phase 42 activates one unsigned macOS Apple Silicon `.app` target and adds the
canonical `npm run package:macos` build. The script rejects unsupported hosts,
performs a clean app build, and validates bundle identity, native architecture,
executable layout, and the embedded icon. Portable configuration checks run in
local and hosted verification. Signing, notarization, DMG creation, update
channels, and release publication remain separate gates.

Phase 43 establishes the version 1 data-migration baseline for documents,
nested citation attrs, reference records, and the reference store. DRAFT has no
released older payload schema, so lower and future versions fail without
changing source bytes, registry state, or stored rows. Empty reference-store
initialization remains the only transactional `0 -> 1` transition.

Phase 44 establishes a checked release-candidate hardening baseline. It
classifies current product, CSP, and distribution blockers; mandatory pre-49
gates; accepted v1 limitations; P2 maintenance; and post-v1 work. Passing this
checkpoint means every known release-relevant finding has an owner, phase, and
closure condition. It does not declare DRAFT release-ready.

Phase 45 reconciles the release sequence without adding product behavior. It
closes only the documentation/governance gate, assigns the four visible
workflow blockers to Phase 46, keeps responsiveness in Phase 47, CSP/security
in Phase 48, and candidate distribution in Phase 49. Usability and interaction
clarity are now binding v1 release conditions. The accepted downstream criteria
live in `docs/contracts/V1_USABILITY_ACCEPTANCE.md`. Phase 46 is next.

Phase 46 must make each supported workflow discoverable, understandable,
predictable, recoverable, keyboard-operable, and explicit about current state.
Any local text-analysis workflow remains blocked until ADR-002 is accepted and
must then use the exact governed five-check scope without model-backed claims.

Phase 47 is Usability and Perceived Performance Validation. It audits every
visible label, menu, control, state, and realistic workload, then combines
measured responsiveness with uncoached task evidence from at least five
first-time users. Benchmarks alone cannot close its gate.

Phase 48 includes secure usability in the final trust-boundary review. Phase 49
reruns the supported workflow from the exact candidate package and blocks on
open `UX-0` or `UX-1` findings. Phase 50 requires a concise first-run entry
point, user release notes, supported-capability guidance, shortcuts, recovery
help, and a verified download and launch path.

The repository owner selected local deterministic text analysis as the proposed
v1.0.0 analysis boundary. ADR-002 is under architecture review and `RC-03`
remains open. Phase 46 analysis implementation cannot begin until the proposal
is accepted; no provider, credential, network transmission, generative model,
or packaged model runtime is implied by this proposal.

The proposal treats measurements as internal inputs, exactly five named
heuristics as non-authoritative review signals, and semantic, generative, or
inferential model-backed interpretation as outside v1.0.0. Supporting counts or
patterns cannot silently become additional product capabilities.

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
- Primary controls, labels, effects, recovery, and supported task flow are
  understandable without maintainer knowledge.
- Operations provide timely feedback and do not leave the user in an ambiguous
  waiting state.

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

- The accepted `docs/contracts/V1_USABILITY_ACCEPTANCE.md` contract passes with
  its named automated, packaged, and human-comprehension evidence.
- DRAFT is not ready for v1.0.0 unless a user can identify the primary controls,
  understand their labels, predict their effects, recover from visible
  failures, and complete the supported document workflow without relying on
  maintainer knowledge.
- No `UX-0` or `UX-1` finding remains open, and every `UX-2` finding has an
  explicit disposition.
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
- Startup, editor interaction, and supported operations provide clear feedback
  and remain responsive at documented realistic limits.
