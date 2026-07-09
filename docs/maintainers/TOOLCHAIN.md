# DRAFT Toolchain

## Current phase

Phase 3 is complete at the current checkpoint. The Phase 1 toolchain remains
locked, and the Phase 2 verification command now runs locally and in the
GitHub Actions baseline.

This checkpoint does not include the launched desktop shell, product workflows,
release automation, or packaging. Those belong to later phases.

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
| `just check-ci-parity` | `bash scripts/check-ci-local-parity.sh` | Verify that CI delegates to local scripts. |
| `just build` | `bash scripts/build.sh` | Typecheck/build the frontend and check Rust. |

## GitHub Actions baseline

`.github/workflows/verify.yml` defines the `Verify` workflow and `verify` job.
It runs for pull requests targeting `main` and pushes to `main`.

The job performs environment setup, then runs the same repository commands used
locally:

```bash
bash scripts/bootstrap.sh
bash scripts/verify.sh
```

The workflow uses these pinned runtime assumptions:

- `ubuntu-24.04`
- `actions/checkout@v7` with persisted credentials disabled
- `actions/setup-node@v6` with Node.js 24 and npm's download cache
- `actions/setup-python@v6` with Python 3.12
- `dtolnay/rust-toolchain@1.96.0` with Clippy and rustfmt

Each action is pinned to the verified commit currently referenced by that
version selector. The comments in `verify.yml` retain the readable version.

The npm cache contains package-manager downloads, not `node_modules` or other
project-controlled output.

Tauri compile checks on Ubuntu require the current Debian development packages:

```text
build-essential
file
libayatana-appindicator3-dev
librsvg2-dev
libssl-dev
libwebkit2gtk-4.1-dev
libxdo-dev
```

These packages follow Tauri's Linux prerequisites. The exact Ubuntu job cannot
be executed on a macOS workstation, so the first GitHub Actions run remains the
authoritative validation of the Linux package set.

The workflow has read-only repository permission, uses no secrets, publishes
nothing, and uploads no artifacts. Optional tools remain environment-dependent
and are reported by `scripts/verify.sh`; all required Phase 2 checks run in both
environments.

`scripts/check-ci-local-parity.sh` prevents the workflow from bypassing the
local verifier or adding publishing, deployment, write permissions, secrets,
or failure masking.

The dedicated `invariants` and `build` workflows in the production architecture
remain future work. Phase 3 intentionally provides one aggregate baseline job
for all checks that exist today.
