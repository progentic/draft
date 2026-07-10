# DRAFT Toolchain

## Current phase

Phase 20 is complete at the current checkpoint. The Phase 1 toolchain remains
locked, the Phase 2 verification command runs locally and in GitHub Actions,
the React/Tiptap workspace shell has focused frontend tests, and the first
typed Tauri command, frontend IPC, finite event, and worker-cancellation
boundaries are enforced. Rust also owns a validated version 1 document
envelope, a process-local single-live-handle registry, typed native-dialog
open/save commands, a hardened atomic replacement path, and a validated
reference record, local SQLite store, versioned citation-node resolution
boundary, and pure bibliography-consistency check. The Phase 5, Phase 10,
Phase 15, and Phase 20 audits are recorded in
`docs/maintainers/REALIGNMENT.md`.

This checkpoint does not include reference CRUD IPC, visible citation controls,
complete citation formatting, rendered bibliographies, workspace file controls,
a close command, autosave, recovery, product research or analysis workflows,
centralized network client, release automation, or packaging.

## Toolchain decisions

- Rust uses Cargo with `Cargo.lock` committed for the desktop application.
- Serde provides explicit command, error, and document-envelope serialization.
- `serde_json` preserves structured Tiptap JSON inside the validated envelope.
- `tauri-plugin-dialog` keeps native open/save path selection inside Rust.
- `tempfile` provides owned same-directory temporary files and platform-specific
  atomic replacement. DRAFT synchronizes content before replacement and the
  parent directory on Unix.
- `rusqlite` with bundled SQLite provides cross-platform local reference
  persistence without depending on a system SQLite installation.
- A Rust `Mutex<HashMap<...>>` serializes process-local document handle
  ownership without introducing persistence.
- A separate Rust `Mutex<()>` serializes document open/save lifecycle
  coordination so disk and registry snapshots cannot be reordered.
- `tokio-util` provides cooperative cancellation tokens for Rust-owned workers.
- UUID values identify validated documents; UUID v4 values identify transient
  workers without frontend-generated IDs.
- Frontend dependencies use npm with `package-lock.json` committed.
- TypeScript, React, and Tiptap run inside the Tauri WebView.
- `@tiptap/core` provides the explicit custom inline citation-node API at the
  same locked version as the React and starter-kit packages.
- `@tauri-apps/api/core` is isolated behind typed wrappers in `src/ipc/`.
- `@tauri-apps/api/event` is isolated behind typed listeners in `src/ipc/`.
- Tauri capabilities grant the main WebView event listen/unlisten access only.
- Lucide React provides the workspace's interface icons.
- Vitest, Testing Library, and jsdom provide frontend component tests.
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
- React/Tiptap workspace plus typed command, event, cancellation, citation,
  and document client tests
- Rust formatting, Clippy, compile checks, command/event/cancellation/envelope/
  registry/persistence/atomic-write/citation/bibliography scans, cross-bridge
  name parity, and tests
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
reported and does not hide any required check. Frontend formatting and ESLint
are not configured yet; frontend tests, TypeScript, and production-build checks
remain required.

Run the focused frontend suite directly with:

```bash
npm test
```

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
ripgrep
```

The development libraries follow Tauri's Linux prerequisites. The `ripgrep`
package satisfies DRAFT's repository scan prerequisite. The exact Ubuntu job
cannot be executed on a macOS workstation, so GitHub Actions remains the
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
