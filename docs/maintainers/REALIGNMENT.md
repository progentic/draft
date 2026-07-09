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
