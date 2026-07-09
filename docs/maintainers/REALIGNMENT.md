# DRAFT Drift Realignment Log

## Purpose

This log records the evidence and decisions from each required documentation
and drift realignment phase. It is a checkpoint record, not a replacement for
the roadmap, phasemap, architecture, governance, invariants, or changelog.

## Phase 5 - 2026-07-09

The audited implementation baseline is commit `5fe83df`, the completed Phase 4
workspace shell. Its hosted verification run passed:

<https://github.com/progentic/draft/actions/runs/29048740679>

### Surfaces reviewed

- `README.md`, `CHANGELOG.md`, and `AGENTS.md`
- architecture, governance, invariants, coding style, and documentation policy
- roadmap and phasemap sequencing
- maintainer and user documentation
- Tauri, Rust, npm, TypeScript, and Python manifests
- local scripts, the root `justfile`, and GitHub Actions
- React/Tiptap component tests and current invariant scans
- tracked repository shape and generated-file rules

The `docs/adr`, `docs/contracts`, and `docs/wiki` surfaces do not exist yet.
Their absence is intentional: no accepted architecture change, stable data
contract, or user-support topic currently requires those documents.

### Alignment findings

- The workspace owns only presentation, editor interaction, and transient UI
  state. It performs no file, persistence, secret, worker, or network work.
- The UI explicitly reports that the current document is unsaved. The user
  guide warns that reload or application exit discards edits.
- Rust remains the Tauri runtime owner. No Tauri commands or frontend IPC
  wrappers exist before their Phase 6 and Phase 7 boundaries.
- `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` agree
  on version `0.1.0`.
- `scripts/verify.sh` runs the Phase 4 frontend tests before the existing Rust,
  frontend build, Python, documentation, repository, and invariant checks.
- `.github/workflows/verify.yml` still calls the local bootstrap and aggregate
  verification scripts with no secrets, publishing, or failure masking.
- `README.md` remains unchanged. Its pre-1.0 disclaimer still distinguishes
  the implemented scaffold from planned production capabilities.
- No architecture decision or invariant changed, so this checkpoint does not
  require an ADR or contract document.

### Planned gaps, not drift

- Durable create, open, save, close, and reopen behavior begins with the
  document-core phases.
- Typed Rust commands, frontend IPC wrappers, event streams, and cancellation
  begin in Phases 6 through 9.
- Dedicated `invariants` and `build` workflows remain production targets after
  those surfaces have independent responsibilities.
- Frontend formatting and ESLint, required `shfmt`, and required Ruff remain
  open until their versions and installation paths are pinned.
- The current frontend production build reports a single bundle above Vite's
  default 500 kB advisory threshold. This is not a correctness failure, but it
  remains visible for later performance and packaging work.

### Verification evidence

The checkpoint used these executable gates:

```bash
npm test
npm run typecheck
npm run tauri -- dev --no-watch
bash scripts/verify.sh
git diff --check
```

The Tauri development process compiled, launched, remained running without a
runtime error, and shut down cleanly. The aggregate verifier passed locally,
and the Phase 4 GitHub Actions run passed the same repository verification
entry point.

No known documentation, architecture, invariant, build, or workflow drift
remains at this checkpoint.

## Phase 10 - 2026-07-09

The audited implementation baseline is commit `81b9fb7`, the completed Phase 9
worker-cancellation boundary. Its hosted verification run passed:

<https://github.com/progentic/draft/actions/runs/29052618027>

### Surfaces reviewed

- `README.md`, `CHANGELOG.md`, and the local `AGENTS.md`
- architecture, governance, invariants, coding style, and documentation policy
- roadmap and phasemap sequencing through the Phase 10 gate
- command, frontend command-client, event, and cancellation implementation guides
- all current Rust command/event/worker modules and TypeScript IPC wrappers
- frontend and Rust contract/lifecycle tests
- user workspace documentation and actual rendered workspace behavior
- manifests, local scripts, `justfile`, GitHub Actions, and hosted run history
- tracked, ignored, generated, and untracked repository state

### Drift corrected

- Documentation policy now reflects the actual repository layout: public entry
  documents remain at the root, while governance and execution guides live in
  `/docs`. The ignored local `AGENTS.md` is no longer described as required in
  a clean checkout.
- The changelog no longer presents a fake dated release template or a link to a
  different repository. It states that no versioned release exists and keeps
  manifest versions distinct from release evidence.
- Coding-style examples now use the implemented registry-generated worker ID,
  synchronous cancellation API, registration guard, and typed frontend event
  listener instead of obsolete or boundary-bypassing calls.
- Bridge guides are explicitly labeled as implemented checkpoint notes rather
  than accepted contracts. Governance requires a PR lifecycle and two-week
  stability window before promotion under `docs/contracts/`.
- Local and CI invariant scans now compare registered Rust command names and
  Rust event names against frontend wrapper names, then require those names to
  appear in maintainer documentation.
- Script comments and optional-formatting messages no longer describe the
  aggregate verifier as frozen at Phase 2.
- `ROADMAP.md` and `PHASEMAP.md` now agree that Phases 0 through 10 are complete
  and Phase 11 has not started.
- `ARCHITECTURE.md` now separates implemented Phase 9 boundaries from future
  persistence, document, reference, analysis, formatting, and export targets.
- The invariant enforcement table now records cancellation lifecycle coverage
  and the Phase 10 command/event name parity check used locally and in CI.
- Accepted future invariants no longer point only to planned enforcement.
  Phase-gate scans now reject premature citation, persistent-job,
  document-registry, watched-import, save, and Python-helper protocol surfaces;
  each owning phase must replace its absence gate with behavioral tests.

### Contract reconciliation

The audited bridge names match across implementation, tests, and guides:

```text
commands: cancel_worker, get_runtime_status
events: draft://runtime-status
```

Rust and frontend tests pin the request, response, payload, and error shapes.
The cancellation lifecycle tests pin active, repeated, already-ended,
malformed, unknown-worker, registration-drop, and registry-shutdown behavior.
No command, event, or cancellation example claims a product worker or durable
job state that does not exist.

### Governance decisions

- Architecture and invariant ownership rules remain semantically unchanged;
  current-checkpoint and enforcement wording now reflects actual code.
- No ADR is required because this checkpoint does not alter trust, ownership,
  persistence, network, worker, or verification architecture.
- `docs/adr`, `docs/contracts`, and `docs/wiki` remain absent intentionally.
  No architecture change, stable accepted contract, or recurring user-support
  topic has completed the relevant governance lifecycle.
- `ROADMAP.md` and `PHASEMAP.md` retain their execution-guide role while a
  narrow current-checkpoint line prevents phase-sequence ambiguity.
- `docs/user/WORKSPACE.md` remains accurate and did not need a wording change.
- `docs/drafts/DOCUMENT_ENVELOPE.md` defines the non-binding Phase 11 readiness
  gate without implementing persistence or promoting an accepted contract.

### Constrained and external state

- `README.md` was reviewed but not modified, following the explicit user
  instruction. Its "initial application toolchain scaffold" status sentence is
  narrower than the implemented Phase 9 boundary and remains a documented
  exception rather than a hidden claim of full alignment.
- `src/App 2.tsx` and `src/styles 2.css` were confirmed as the obsolete minimal
  Phase 4 shell, were not imported, and were still scanned by TypeScript's
  `src` include. They were deleted as stale local artifacts.
- Generated `dist`, Cargo, Vite, TypeScript, and operating-system files remain
  ignored. No generated output became tracked.

### Planned gaps, not Phase 10 drift

- Durable document schemas, registry, save/load, and atomic writes begin in
  Phases 11 through 14.
- Accepted bridge contracts remain deferred until the governance stability and
  review requirements are satisfied.
- Dedicated `invariants` and `build` workflows, frontend formatting/linting,
  required `shfmt`, and required Ruff remain documented future hardening work.
- The frontend build still reports Vite's advisory bundle-size warning. It is
  not a correctness failure and remains visible for packaging/performance work.

### Verification evidence

The checkpoint uses these executable gates:

```bash
npm test
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
bash scripts/check-invariants.sh
bash scripts/check-docs.sh
bash scripts/verify.sh
git diff --check
```

All Phase 10 bridge, documentation, invariant, build, and workflow drift within
the permitted scope is reconciled. The README status exception remains explicit
and the stale untracked alternate files are removed.

## Phase 15 - 2026-07-09

The audited implementation baseline is commit `2f2e669`, the completed Phase 14
atomic-save hardening checkpoint. Its hosted verification run passed:

<https://github.com/progentic/draft/actions/runs/29057687404>

### Surfaces reviewed

- `README.md`, `CHANGELOG.md`, `.gitignore`, and the local `AGENTS.md`
- architecture, governance, invariants, coding style, and documentation policy
- roadmap and phasemap sequencing through the Phase 15 gate
- envelope, registry, save/load, command, event, cancellation, frontend-client,
  toolchain, workspace, and prior realignment guides
- Rust document, command, registry, worker, event, and application modules
- TypeScript IPC wrappers, runtime-status feature code, React/Tiptap workspace,
  and all current frontend tests
- Cargo, npm, Python, Tauri, TypeScript, and Rust toolchain manifests
- local scripts, `justfile`, GitHub Actions, and hosted run history
- tracked, ignored, generated, duplicate, and untracked repository state

### Drift corrected

- The registry guide no longer says its errors are only a future IPC concern;
  Phase 13 open/save commands already expose them as typed nested causes.
- Architecture now records the Phase 14 registry-owned file-operation lock
  separately from the handle map mutex and keeps filesystem calls outside the
  registry module.
- Governance no longer lists Phase 11 envelope-draft refinement as unfinished.
- Command, toolchain, save/load, documentation-policy, roadmap, and phasemap
  checkpoint text now describes the completed Phase 14 behavior and Phase 15
  audit without implying that Phase 16 has started.
- Documentation sanity now requires the Phase 16 readiness draft and requires
  roadmap/phasemap agreement through Phase 15.
- The future-feature scan now rejects a reference store before Phase 17 while
  still allowing the Phase 16 in-memory reference type.

### Document-core truth

- Rust owns the version 1 document envelope and remains its validation
  authority. The TypeScript mirror validates IPC data for display/request
  safety only.
- Rust owns one process-local live handle and one exclusive source path per open
  document. Duplicate IDs and path aliases fail with typed errors.
- Rust owns native file selection, validated load, explicit-snapshot save,
  same-directory temporary writes, content sync, atomic replacement, and Unix
  parent-directory sync.
- Open/save lifecycle coordination is process-local and serialized so
  concurrent saves cannot leave disk and registry snapshots out of order.
- Failures before replacement preserve the prior complete source and clean up
  temporary files. A parent-sync failure after replacement returns
  `durability_uncertain` while retaining the new complete registry snapshot.
- React and Tiptap remain presentation and transient editor state only. The
  workspace does not invoke open/save, so reload or exit still discards visible
  edits.
- No reference record, reference store, citation node, bibliography, network
  client, import pipeline, persistent job, Python-helper protocol, export path,
  autosave, recovery, or close command exists yet.

### Next-phase readiness

`docs/drafts/REFERENCE_RECORD.md` bounds Phase 16 to a version 1 in-memory Rust
record with explicit identity, citekey, bibliographic fields, contributor
shape, partial date, identifiers, provenance, typed failures, and serialization
tests. It explicitly defers storage, uniqueness, IPC, document embedding,
citations, network lookup, import, merge, and reliability scoring.

The document envelope remains unchanged. Reference records are future
reference-library source data and must not be embedded as full citation
metadata in a document or citation node.

### Governance decisions

- No trust, ownership, persistence, network, worker, or invariant decision
  changed, so no ADR is required.
- The current implementation guides and readiness drafts remain non-binding.
  `docs/adr`, `docs/contracts`, and `docs/wiki` remain absent intentionally.
- Local and CI verification continue to use `scripts/verify.sh`; the aggregate
  GitHub Actions `verify` job remains the only implemented hosted workflow.

### Constrained and external state

- `README.md` was reviewed and left unchanged under the explicit user
  instruction. Its "initial application toolchain scaffold" status phrase is
  stale relative to the implemented document core and remains a documented
  exception rather than being turned into a phase status report.
- No `src/App 2.tsx`, `src/styles 2.css`, or other duplicate source artifact is
  present or tracked.
- Current GitHub Actions validates Ubuntu. The platform-neutral replacement
  test also passes locally on macOS; supported-platform package validation
  remains assigned to Phase 42.
- The frontend production build still reports Vite's advisory bundle-size
  warning. It remains a visible performance/packaging concern, not a hidden
  correctness failure.

### Verification evidence

The checkpoint uses these executable gates:

```bash
npm test
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
bash scripts/check-invariants.sh
bash scripts/check-docs.sh
bash scripts/check-ci-local-parity.sh
bash scripts/check-repository.sh
bash scripts/verify.sh
npm run tauri -- dev --no-watch
git diff --check
```

All Phase 15 documentation, document-core, repository-shape, and verification
drift within the permitted scope is reconciled. Phase 16 may begin only within
the bounded readiness draft; Phase 17 persistence and Phase 18 citation gates
remain intact.
