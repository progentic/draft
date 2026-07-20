# DRAFT Toolchain

## Current phase

Phases 0 through 45 are complete at the current checkpoint. Phase 46 adds the
visible v1 workflow and interaction-clarity evidence described below. The
Phase 1 toolchain remains locked, and the Phase 2
verification command runs locally and in GitHub Actions,
the React/Tiptap workspace shell has focused frontend tests, and the first
typed Tauri command, frontend IPC, finite event, and worker-cancellation
boundaries are enforced. Rust also owns a validated version 1 document
envelope, a process-local single-live-handle registry, typed native-dialog
open/save commands, a hardened atomic replacement path, and a validated
reference record, local SQLite store, versioned citation-node resolution
boundary, pure bibliography-consistency check, centralized network client, and
typed DOI metadata providers plus a Rust-owned system-browser handoff. Rust
also owns explicit PDF validation and watched-file stable-write intake without
adding a watcher dependency, plus a persistent PDF import-job state machine
with hashed opaque claims and restart recovery. A provider-independent Rust AI
boundary now assembles bounded provenance-tagged context and coordinates typed,
cancellable generated-analysis streams without a production provider or
network call. The Phase 5, Phase 10, Phase 15, Phase 20, Phase 25, Phase 30,
Phase 35, Phase 40, and Phase 45 audits are recorded in
`docs/maintainers/REALIGNMENT.md`.

Rust also owns a versioned Python helper runner with a canonical fixed
entrypoint, closed protocol allowlist, isolated cleared environment, bounded
standard streams, timeout, cancellation, and child reaping. The
`contract_probe` verifies only that process boundary and is not a product
analysis feature.

The helper allowlist also includes deterministic `text_analysis` version 1.
Python returns only five closed review codes and UTF-8 ranges; Rust validates
them and supplies fixed advisory wording. Phase 46 exposes exactly those five
checks through one typed command and transient review panel. No score,
replacement, persistence, provider, model, network, or document mutation path
is added.

Rust also owns a pure formatting-check module with bounded immutable inputs,
closed APA 7, MLA 9, and Chicago 17 author-date identifiers, deterministic
heading and citation-style consistency findings, and fixed content-free review
wording. It has no document integration, complete style rules, persistence,
filesystem access, Python, network, worker, or export authority. One typed
command and validated frontend wrapper expose those findings in a transient
workspace review band. Explicit heading-level actions use Tiptap only while
the captured run and target node remain current.

One process-local Rust connectivity policy now defaults online and is shared by
the metadata client and system-browser handoff. Typed get/set commands and
frontend clients expose only the effective closed mode. The header toggle adds
no dependency, persistence, connectivity monitor, retry queue, or alternate
transport.

Rust also owns a lazy OS-native service API-key store. `keyring` 4.1.4 selects
Keychain, Credential Manager, or Secret Service, while `zeroize` 1.9.0 clears
owned secret bytes on drop. The store has no command, event, frontend,
provider, network, config, database, filesystem, environment, or log surface.

Rust also owns one strict local diagnostic snapshot command and validating
TypeScript wrapper. No component consumes it. The frontend separately maps
only the four already-visible runtime, connectivity, formatting, and citation
failure surfaces to bounded copy and closed recovery dispositions.

Rust also owns a strict DOCX compiler and atomic export service. It uses
`quick-xml` 0.41.0 for escaped event-based XML and `zip` 8.6.0 with default
features disabled for deterministic stored package entries. The bounded DOCX
reader enables only the `deflate` feature needed for common DOCX packages and
enforces package, entry, compression, XML, relationship, and structure limits
before producing canonical document data. Compilation is bounded and completes
before the shared atomic writer touches a `.docx` target. Phase 46 exposes the
export service; Phase 47 adds bounded DOCX import and a separate confirmed
same-format source-replacement boundary. Rust retains source paths and
fingerprints throughout both operations.

One `cfg(test)` critical-path module composes the existing document lifecycle,
reference store, citation resolution, and DOCX exporter. It widens no
production visibility and adds no command or user workflow.

The package configuration activates only the macOS `app` target. The owned
`npm run package:macos` entrypoint builds and validates an unsigned native
Apple Silicon bundle; portable configuration checks run in GitHub Actions.

Phase 44 adds an offline RC-hardening check that validates the classified
blocker inventory against current CSP, unsigned package, visible workflow,
pre-release version, tag, and generated-artifact evidence. Passing it does not
mean the Phase 52 entry gate is satisfied.

Phase 45 closes only `GATE-45`. Phase 46 supplies the visible document,
reference/citation, local text-check, and DOCX workflows plus accessibility and
interaction evidence, but its RC rows and gate remain open pending complete
packaged validation. Accepted ADR-003 assigns
interoperability to Phase 47, desktop workflow integration to Phase 48,
usability and performance to Phase 49, realignment to Phase 50, security to
Phase 51, and candidate distribution to Phase 52.

Accepted ADR-002 keeps the provider-independent model orchestration boundary
internal and denies external model, credential, and generative-analysis product
authority. Phase 46 implements the accepted local deterministic workflow
without widening that boundary.

The guard also rejects model SDK dependencies and endpoints, provider
credential environment variables, packaged model artifacts, runtime model
downloads, direct frontend provider or secret authority, and unsupported public
capability language. It preserves the exact five-check ceiling and the
measurement, heuristic, and model-backed interpretation distinction.

This checkpoint includes bounded New, Open, Save, Close, manual-reference,
citation-insertion, five-check local analysis, and DOCX export controls. It does
not include reference edit/delete/import, complete citation formatting,
rendered bibliographies, autosave, crash recovery, provider metadata lookup UI,
browser-handoff controls, PDF intake controls, filesystem watcher execution,
import processing workers, production model providers, model credentials,
credential settings, formatting finding persistence, citation style conversion,
complete document formatting, PDF export, downloaded Python runtime, release
automation, signing, notarization, release publication, or a visible diagnostics
workflow.

Accepted ADR-001 defers choosing a PDF engine. No PDF library, renderer, binary,
font bundle, conversion process, command, frontend control, or packaged resource
is part of the toolchain. Reconsideration requires accepted font, layout,
accessibility, cross-platform rendering, dependency/licensing, resource-bound,
parser-verification, deterministic-failure, and source-preservation policies.

## Toolchain decisions

- Rust uses Cargo with `Cargo.lock` committed for the desktop application.
- Serde provides explicit command, error, and document-envelope serialization.
- `serde_json` preserves structured Tiptap JSON inside the validated envelope.
- `quick-xml` 0.41.0 writes escaped Office Open XML events without manual
  interpolation or XML serialization features.
- `zip` 8.6.0 with default features disabled and only `deflate` enabled creates
  deterministic stored DOCX exports and reads bounded compressed DOCX imports.
  DRAFT emits no compressed, encrypted, timed, or active-content export entry.
- `tauri-plugin-dialog` keeps native open/save path selection inside Rust.
- `tempfile` provides owned same-directory temporary files and platform-specific
  atomic replacement. DRAFT synchronizes content before replacement and the
  parent directory on Unix.
- `rusqlite` with bundled SQLite provides cross-platform local reference
  persistence without depending on a system SQLite installation.
- The same locked SQLite library provides the separate versioned PDF job store.
  SHA-256 stores only claim-token digests; raw UUID v4 claims stay in Rust.
- `reqwest` 0.13.4 with only its Rustls feature provides the centralized
  HTTPS-only client without cookie or system-proxy features.
- `tauri-plugin-opener` 2.5.4 provides the Rust-only default-browser adapter.
  Its guest plugin is not initialized, and the WebView receives no opener
  package or capability.
- A Rust `Mutex<HashMap<...>>` serializes process-local document handle
  ownership without introducing persistence.
- A separate Rust `Mutex<()>` serializes document open/save lifecycle
  coordination so disk and registry snapshots cannot be reordered.
- `tokio-util` provides cooperative cancellation tokens for Rust-owned workers.
- The same cancellation token races each Phase 27 adapter read without spawning
  a task. Deterministic in-memory adapters are test infrastructure only.
- A direct Tokio dependency provides bounded asynchronous process I/O, timeout,
  and cancellation selection for the Python helper runner. Rust invokes the
  fixed entrypoint directly without a shell or detached task.
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
- Python uses the standard library only and `pyproject.toml` declares an empty
  dependency list. A dependency manager must be chosen and locked before any
  third-party helper dependency is added.
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

## Available Local Review Tools

`stylelint` is available at `/opt/homebrew/bin/stylelint`. Use it only for
stylesheets changed by the active work and only against an explicit CSS
contract; its presence does not authorize repository-wide formatting churn.
Phase 48 runs a scoped built-in-rule configuration against `src/styles.css`
after replacing the stale header badge with the generated product mark. The
temporary configuration is not committed and no unrelated stylesheet is
reformatted.

`ksnip` is available at `/Applications/ksnip.app` for manual screenshots and
annotations. Captures must identify the tested package and remain inside the
repository's ignored `.tmp/manual-evidence/` workspace. Tool availability is not
evidence by itself; only an actual capture tied to the executable hash may
support a finding or gate disposition.

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
  document, and external-access client tests
- Rust formatting, Clippy, compile checks, command/event/cancellation/envelope/
  registry/persistence/atomic-write/citation/bibliography/network/browser/PDF
  intake/job/AI/Python-helper/text-analysis/formatting/DOCX scans, the PDF
  deferral guard, cross-bridge name parity, and tests
- TypeScript type checking and a frontend production build
- Python unit tests without bytecode or test caches
- Bash syntax checks
- frontend, Rust, Python, credential-field, and Bash-runtime boundary scans
- required-source and generated-file hygiene checks
- offline documentation sanity checks
- `git diff --check`

The local `AGENTS.md` is checked when present but is not required by clean
checkout verification because repository policy intentionally ignores it.

`shellcheck` and Ruff add blocking checks when installed. `shfmt -i 2` reports
historical formatting drift informationally until its version and baseline are
pinned through the dedicated tooling work tracked by `MAINT-03`. Missing
optional tools are reported and do not hide any required check. Frontend
formatting and ESLint are not configured yet; frontend tests, TypeScript, and
production-build checks remain required.

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
