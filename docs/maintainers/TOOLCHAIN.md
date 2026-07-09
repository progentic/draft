# DRAFT Toolchain

## Current phase

Phase 2 is complete at the current checkpoint. The Phase 1 toolchain remains
locked, and one local command now checks the build, tests, formatting,
boundaries, documentation, and repository hygiene.

This checkpoint does not include GitHub Actions, the launched desktop shell,
or product workflows. Those belong to later phases.

## Toolchain decisions

- Rust uses Cargo with `Cargo.lock` committed for the desktop application.
- Frontend dependencies use npm with `package-lock.json` committed.
- TypeScript, React, and Tiptap run inside the Tauri WebView.
- Python currently uses the standard library only. Third-party helper
  dependencies will be pinned when a concrete helper contract requires them.
- Bash is limited to local and CI orchestration.

These choices implement the stack already accepted in `ARCHITECTURE.md` and
`GOVERNANCE.md`. They do not change a trust boundary and therefore do not
require an ADR.

## Prerequisites

- Rust 1.96.0 with Cargo, rustfmt, and Clippy.
- Node.js 22.12.0 or newer with npm.
- Python 3.12 or newer.
- Bash.
- Git.
- ripgrep (`rg`).

`just` is the preferred task runner but is optional during local setup because
every target delegates to a documented Bash script.

## Bootstrap

Run from the repository root:

```bash
bash scripts/bootstrap.sh
```

The command installs the locked frontend dependency tree under `node_modules`,
fetches the locked Rust dependency tree through Cargo, and verifies that the
Python helper package imports. It does not start the application or write
application data.

## Local verification

After bootstrap, run:

```bash
just verify
```

When `just` is unavailable, run the equivalent entry point:

```bash
bash scripts/verify.sh
```

Verification is offline after bootstrap. Cargo runs with `--locked --offline`,
and npm uses the installed dependency tree. Missing required tools or missing
frontend dependencies fail with an actionable message.

The verifier runs:

- npm dependency-tree validation
- Rust formatting, Clippy, compile checks, and tests
- TypeScript type checking and a frontend production build
- Python unit tests without bytecode or test caches
- Bash syntax checks
- frontend, Rust, Python, credential-field, and Bash-runtime boundary scans
- required-source and generated-file hygiene checks
- offline documentation sanity checks
- `git diff --check`

The local `AGENTS.md` is checked when present but is not required by clean
checkout verification because repository policy intentionally ignores it.

`shellcheck`, `shfmt`, and Ruff add checks when installed. Their absence is
reported and does not hide any required Phase 2 check. Frontend formatting and
ESLint are not configured yet; TypeScript and production-build checks remain
required.

## Command surface

| Command | Direct fallback | Purpose |
| :--- | :--- | :--- |
| `just verify` | `bash scripts/verify.sh` | Run the full local health check. |
| `just format` | `bash scripts/format.sh` | Format supported source files. |
| `just check-invariants` | `bash scripts/check-invariants.sh` | Scan implemented trust boundaries. |
| `just docs-check` | `bash scripts/check-docs.sh` | Run offline documentation sanity checks. |
| `just build` | `bash scripts/build.sh` | Typecheck/build the frontend and check Rust. |

Phase 3 must call these same scripts from GitHub Actions. It must not duplicate
their logic inside workflow YAML.
