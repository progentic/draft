# DRAFT — Invariants

**Status:** Draft v0.4  
**Product name:** D.R.A.F.T — Document Research, Analysis, Formatting & Text-analysis  
**Purpose:** Define the non-negotiable properties the implementation must preserve, and how each property is enforced in local development and GitHub Actions.

An invariant is a rule whose violation is a bug by definition. It is not a preference. It protects a boundary that would be expensive, unsafe, or confusing to recover from after the fact.

---

## 1. Enforcement Model

Each accepted invariant has two enforcement surfaces:

- **Local development:** the check a developer runs before opening or updating a PR.
- **GitHub Actions:** the check that blocks merge when the invariant is violated.

Required local aggregate command:

```bash
just verify
```

Required invariant-specific local command:

```bash
just check-invariants
```

Current GitHub Actions baseline:

```text
.github/workflows/verify.yml
```

Current GitHub Actions job name:

```text
verify
```

The production target still includes a dedicated `invariants` workflow and job.
Phase 3 uses the aggregate `verify` job while the current repository has only
one small implementation surface.

Local checks and CI checks should use the same scripts where possible. CI-only behavior is allowed only when the check depends on GitHub metadata, such as PR age or labels.

---

## 2. Status Definitions

- **Accepted:** binding. Must have local-development and GitHub Actions enforcement.
- **Proposed:** not binding yet. May describe intended future protection, but code cannot rely on it as enforced.
- **Retired:** no longer binding. Must link to the ADR or invariant that replaced it.

No invariant may be marked `Accepted` unless it has both local and GitHub Actions enforcement.

---

## 3. Invariant Table

| ID | Status | Invariant | Protects | Local development enforcement | GitHub Actions enforcement |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INV-01` | Accepted | Institutional and publisher credentials never pass through DRAFT process memory, config, logs, database, document files, Python helpers, Bash scripts, or storage. | `ARCHITECTURE.md` §9 and `GOVERNANCE.md` §9 | Phase 2 scans implemented source surfaces for denied credential fields. Full secret, schema, helper-input, and log checks remain open. | The Phase 3 `verify` job runs the current source scan. Full secret, schema, helper-input, and log checks remain open. |
| `INV-02` | Accepted | No Tauri command returns an untyped or generic error. | `ARCHITECTURE.md` §5.1 and §12 | Phase 6 adds a typed runtime-status command with request, signature, and serialization tests. The invariant scan rejects generic errors and requires matching command registration and contract-test counts. | The `verify` job runs the same Rust tests and command-contract scan. |
| `INV-03` | Accepted | Frontend code never calls external network services, filesystem APIs, or secret stores directly. All trusted work goes through Rust commands. | `ARCHITECTURE.md` §4.1 and §13 | `scripts/check-invariants.sh` blocks direct trusted APIs plus raw command and event APIs outside `src/ipc/`. Typed command and event tests pin request, response, payload, error, ordering, and cleanup behavior. | The `verify` job runs the same frontend tests and boundary scans. |
| `INV-04` | Accepted | Citation node attrs are validated against a declared schema version before render, analysis, formatting, save, or export. Invalid or unknown versions are migration cases, never silent render cases. | `ARCHITECTURE.md` §8 and §11 | No citation module exists yet. Schema enforcement begins in Phase 18. | Planned after the citation surface exists. |
| `INV-05` | Accepted | Background jobs persist state per record and resume from the last valid checkpoint after interruption. | `ARCHITECTURE.md` §10 | No background-job module exists yet. Resumability tests begin in Phase 26. | Planned after the job surface exists. |
| `INV-06` | Accepted | A document can have only one live editing handle at a time. No two Tiptap instances may hold a live handle to the same document. | `ARCHITECTURE.md` §6 | No document registry exists yet. Registry tests begin in Phase 12. | Planned after the registry exists. |
| `INV-07` | Accepted | Every user-initiated long-running Rust worker that emits progress events has a user-visible cancellation or abort path unless documented as non-cancelable and idempotent. | `ARCHITECTURE.md` §5.3 and §10 | No long-running worker exists yet. Cancellation enforcement begins in Phase 9. | Planned after the worker surface exists. |
| `INV-08` | Accepted | A watched-folder file enters the import pipeline only after stable-write confirmation. | `ARCHITECTURE.md` §10.1 | No watched-folder importer exists yet. Stable-write tests begin in Phase 24. | Planned after the import surface exists. |
| `INV-09` | Accepted | Document saves use atomic write-then-rename. The on-disk file is always the prior complete version or the new complete version, never a partial write. | `ARCHITECTURE.md` §11 | No document-save path exists yet. Atomic-save enforcement begins in Phase 14. | Planned after the save surface exists. |
| `INV-10` | Accepted | All outbound requests to external services go through the centralized Rust network client. Feature code, frontend code, and Python helpers must not create ad hoc external network clients. | `ARCHITECTURE.md` §13 | `scripts/check-invariants.sh` scans current frontend, Rust, and Python source for denied clients. | The Phase 3 `verify` job runs the same frontend, Rust, and Python scans. |
| `INV-11` | Accepted | Python helpers are allowlisted, versioned, typed worker tools. They do not own persistence, secrets, source-document mutation, or external network access by default. | `ARCHITECTURE.md` §4.3, §5.3, and §11 | Phase 2 runs Python tests and denied-import scans. Typed protocol, allowlist, timeout, and output validation begin in Phase 28. | The Phase 3 `verify` job runs current Python tests and scans. Typed worker enforcement expands in Phase 28. |
| `INV-12` | Accepted | Bash is local/CI orchestration only. The product runtime must not use Bash for document processing, credential handling, external network access, or user-supplied path execution. | `ARCHITECTURE.md` §4.4 and `GOVERNANCE.md` §8 | Phase 2 runs Bash syntax checks and scans Rust for Bash runtime execution. `shellcheck` and `shfmt` run when installed. | The Phase 3 `verify` job runs Bash syntax and runtime-boundary checks. Optional tools run when present. |
| `INV-13` | Accepted | Local verification and GitHub Actions verification use the same underlying scripts where practical. A check that blocks CI must be runnable locally unless it depends on GitHub metadata. | `ARCHITECTURE.md` §14 and `GOVERNANCE.md` §8 | The root `justfile` delegates to repository scripts, with direct Bash fallbacks. | `.github/workflows/verify.yml` calls `scripts/verify.sh`; `scripts/check-ci-local-parity.sh` enforces that mapping. |

---

## 4. Detailed Enforcement Notes

### INV-01: Credential Boundary

DRAFT may open an external page in the user's system browser. DRAFT must not receive, store, proxy, log, submit, or pass through institutional or publisher credentials.

Credential protection applies to:

- Rust process memory where DRAFT controls input.
- TypeScript frontend state.
- SQLite.
- document files.
- config files.
- logs.
- Python helper payloads.
- Bash scripts.
- GitHub Actions logs.

Minimum local checks:

```bash
just check-secrets
cargo test credential_schema_has_no_publisher_login_fields
scripts/check-credential-fields.sh
scripts/check-helper-inputs-no-credentials.sh
scripts/check-logs-no-credential-fields.sh
```

Minimum GitHub Actions checks:

```yaml
- gitleaks
- credential_schema_has_no_publisher_login_fields
- check-credential-fields
- check-helper-inputs-no-credentials
- check-logs-no-credential-fields
```

Minimum denylisted field names in app-owned config, storage, document schemas, helper inputs, and logs:

```text
publisher_username
publisher_password
institution_username
institution_password
scholar_username
scholar_password
library_username
library_password
api_key_for_publisher
```

Generic `api_key` is allowed only for documented API integrations stored through the OS credential manager path. It must not be allowed for publisher or institutional login.

---

### INV-02: Typed Command Errors

Every Tauri command returns a command-specific error enum. The frontend needs typed errors so it can choose the right user response.

Denied command return patterns:

```rust
Result<T, anyhow::Error>
Result<T, Box<dyn std::error::Error>>
Result<T, String>
Result<T, serde_json::Value>
```

Allowed shape:

```rust
Result<T, SearchReferenceError>
Result<T, SaveDocumentError>
Result<T, ImportPdfError>
Result<T, RunTextAnalysisError>
Result<T, RunFormattingCheckError>
```

Local and CI checks must inspect command signatures, not just compile the project.

Phase 6 establishes the enforced command pattern with `get_runtime_status`.
Its request rejects unknown fields, and its response and error enum serialize
to stable JSON shapes. Rust tests pin the function signature and all three
boundary forms. `scripts/check-invariants.sh` also requires every discovered
Tauri command to have a registered handler and matching request, signature,
response, and error tests.

---

### INV-03: Frontend Boundary

Frontend code may request trusted work only by invoking Rust commands. It may not directly access external network services, local filesystems, or secret stores.

Denied frontend patterns:

```text
fetch(
axios
XMLHttpRequest
new WebSocket(
EventSource(
navigator.sendBeacon(
localStorage secret usage
filesystem access APIs
@tauri-apps/api/core outside src/ipc
raw invoke(...) outside src/ipc
generic invokeCommand(...) outside src/ipc
@tauri-apps/api/event outside src/ipc
raw listen(...) outside src/ipc
generic listenToEvent(...) outside src/ipc
```

Exception: frontend may subscribe to Tauri events through typed wrappers under
`src/ipc/` because those events are local IPC, not external network calls.

Phase 7 establishes `src/ipc/client.ts` as the only raw command invocation
adapter. Command-specific wrappers validate unknown response data and classify
command or transport failures before React receives them. Components and
feature hooks do not import Tauri command APIs directly.

---

### INV-04: Citation Schema Validation

Citation nodes must include `schema_version`. Version mismatch means migration or validation failure, not best-effort rendering.

Required minimum node shape:

```json
{
  "schema_version": 1,
  "citekey": "string",
  "render_style": "apa7"
}
```

Denied behavior:

- Rendering a citation node with no schema version.
- Rendering a citation node with unknown attrs by ignoring the unknowns.
- Rendering a citation node whose `citekey` cannot resolve to a reference record.
- Treating embedded CSL JSON as citation source of truth.
- Passing invalid citation nodes into formatting, text-analysis, save, or export paths.

---

### INV-05: Background Job Resumability

Background job state must be stored in SQLite before work that depends on the state occurs. A process crash must not drop or restart work from zero unless the job contract explicitly says the job is non-resumable and safe to restart.

Required minimum persisted fields:

```text
job_id
record_id
job_kind
state
attempt_count
last_error
last_checkpoint
created_at
updated_at
cancel_requested
```

This applies to research, metadata resolution, retraction checks, long formatting checks, and long text-analysis jobs.

---

### INV-06: Single Live Document Handle

A document can be visible in multiple navigation surfaces, but it cannot have two live editing handles. A second open request focuses the existing view or returns `AlreadyOpen`.

Required test:

```text
open_document_twice_returns_already_open
```

---

### INV-07: Cancellation for Long-Running Workers

This invariant applies to user-initiated workers that can continue after the initiating command returns.

Examples:

- AI generation stream.
- Batch PDF metadata resolution.
- Retraction-check run started by the user.
- Long formatting check.
- Long text-analysis run.
- Python helper invocation with visible progress.

It does not apply to simple events that only announce a completed bounded command.

The Phase 8 `draft://runtime-status` event is one such bounded event. Its
listener is registered before `get_runtime_status`, Rust emits one `ready`
payload during the command, and no work continues after the command returns.
If that producer becomes asynchronous or persistent, cancellation enforcement
must be added before the change is accepted.

A worker may be documented as non-cancelable only if it is idempotent, short-lived, and safe to complete.

---

### INV-08: Stable-Write Before Import

A watched file is not ready because it appeared. It is ready only after DRAFT observes that writing has stabilized.

Required test behavior:

1. Write a fake PDF in chunks.
2. Emit filesystem events during the write.
3. Assert import does not begin during partial write.
4. Stop writing.
5. Wait for debounce and stable-size confirmation.
6. Assert import begins once stable.

---

### INV-09: Atomic Save

The source document save path must use one atomic writer. Feature code must not hand-roll partial save logic.

Required save sequence:

```text
write temp file
fsync temp file
rename temp file over target
fsync parent directory where supported
```

Denied behavior:

- Truncating the target before writing the new file.
- Writing directly to the target path.
- Treating export failure as source document save failure.
- Allowing Python helpers to write source documents directly.
- Allowing Bash scripts to write source documents at product runtime.

---

### INV-10: Centralized Network Client

All external requests go through the Rust network client. This protects rate limiting, backoff, logging policy, offline detection, and User-Agent behavior.

Denied Rust patterns outside the network crate or module:

```rust
reqwest::Client::new()
reqwest::get(...)
ureq::get(...)
hyper::Client::new(...)
```

Denied frontend patterns are covered by `INV-03`.

Denied Python patterns in helper workers unless explicitly approved by ADR:

```python
requests.get(...)
requests.post(...)
httpx.get(...)
urllib.request.urlopen(...)
aiohttp.ClientSession(...)
```

The exact allowed Rust module path should be set once the repository layout stabilizes. Until then, the check should use an allowlist such as:

```text
crates/network/**
src-tauri/src/network/**
```

---

### INV-11: Python Helper Boundary

Python helpers are tools, not authorities.

Required helper properties:

- Helper is registered in an allowlist.
- Helper has a version.
- Helper has typed input and output schemas.
- Helper has a timeout.
- Helper has pinned dependencies.
- Helper stdout and stderr are captured by Rust.
- Helper output is validated by Rust before use.
- Helper cannot mutate source documents directly.

Denied Python behavior:

```python
subprocess.run(..., shell=True)
os.system(...)
open(arbitrary_user_path, "w")
requests.get(...)
httpx.get(...)
keyring.get_password(...)
```

Allowed Python behavior:

- Read an input file path explicitly provided by Rust.
- Write to a temp output path explicitly provided by Rust.
- Return JSON to stdout.
- Run local deterministic formatting or text-analysis checks.

Minimum local checks:

```bash
python -m pytest
scripts/check-python-helper-boundary.sh
scripts/check-python-dependencies-pinned.sh
```

---

### INV-12: Bash Runtime Boundary

Bash is for local development and CI orchestration only.

Allowed Bash usage:

- `just format`
- `just verify`
- `just check-invariants`
- `just build`
- GitHub Actions scripts
- developer setup scripts

Denied Bash usage:

- Product runtime document processing.
- Product runtime network access.
- Product runtime credential handling.
- Executing commands derived from user document content.
- Executing commands derived from untrusted user file paths without strict quoting and validation.

Minimum checks:

```bash
shellcheck scripts/*.sh
shfmt -w scripts/*.sh
scripts/check-bash-runtime-boundary.sh
```

---

### INV-13: Local and CI Parity

A developer must be able to run the same meaningful checks locally that GitHub Actions runs in CI.

Allowed CI-only checks:

- PR age.
- PR labels.
- CODEOWNERS review state.
- branch protection status.

Not allowed as CI-only:

- Rust lint.
- TypeScript typecheck.
- Python tests.
- Bash lint.
- citation schema tests.
- document save tests.
- network boundary checks.
- frontend boundary checks.

Minimum parity check:

```bash
scripts/check-ci-local-parity.sh
```

That script must verify that core CI jobs call repository scripts or `just` targets rather than hiding unique logic inside workflow YAML.

---

## 5. Required Local Command Surface

The root `justfile` must expose these commands:

```bash
just format
just verify
just check-invariants
just build
```

Recommended internal targets:

```bash
just rust-format
just rust-verify
just ts-format
just ts-verify
just python-format
just python-verify
just bash-format
just bash-verify
just tauri-build
```

The exact JavaScript and Python package managers are repository decisions. The invariant is not the specific package manager. The invariant is that the local command surface and GitHub Actions command surface stay aligned.

---

## 6. Required GitHub Actions Workflows

Required workflows:

```text
.github/workflows/verify.yml
.github/workflows/invariants.yml
.github/workflows/build.yml
```

Minimum job coverage:

```text
verify:
  - rust
  - typescript
  - python
  - bash

invariants:
  - command-error-boundary
  - frontend-boundary
  - network-boundary
  - python-helper-boundary
  - bash-runtime-boundary
  - citation-schema
  - document-save
  - job-resume
  - ci-local-parity

build:
  - tauri-desktop-build
```

GitHub Actions should call `just` targets or repository scripts, not duplicate large logic inside YAML.

Phase 3 implements `.github/workflows/verify.yml` with the required `verify`
job. It aggregates all currently implemented checks through
`bash scripts/verify.sh`. The dedicated `invariants.yml` and `build.yml`
workflows remain production targets rather than Phase 3 baseline requirements.

---

## 7. Current Local and CI Enforcement

Phase 2 provides the local enforcement layer:

```bash
just verify
just check-invariants
```

Equivalent direct commands are available when `just` is not installed:

```bash
bash scripts/verify.sh
bash scripts/check-invariants.sh
```

Current scans cover credential-field names, frontend trusted APIs, raw Tauri
command and event placement, typed event contract coverage, Python network and
process imports, generic Rust command errors, typed command contract coverage,
ad hoc Rust network clients, and Bash invocation from product runtime. The
verifier also checks locked offline builds, tests, required source visibility,
generated-file hygiene, and documentation sanity.

Phase 3 runs that same aggregate command in `.github/workflows/verify.yml`:

```bash
bash scripts/verify.sh
```

`scripts/check-ci-local-parity.sh` runs inside the verifier and rejects workflow
drift from the local bootstrap and verification scripts.

The policy status of the invariants is unchanged. Mechanical enforcement is
still incomplete for feature modules that do not yet exist and for the future
dedicated `invariants` and `build` jobs. The repository must not claim full
production enforcement before those checks are present.

---

## 8. Open Enforcement Items

- Add dedicated `invariants.yml` and `build.yml` workflows as their complete
  enforcement surfaces mature.
- Add a full secret scanner plus config, log-field, and helper-input tests.
- Extend typed signature and serialization coverage with every new Tauri
  command.
- Add citation, document, job, cancellation, import, and save invariant tests
  in their owning phases.
- Make `shellcheck`, `shfmt`, Ruff, frontend formatting, and frontend linting
  required once their versions and CI installation paths are pinned.
- Add local-link, ADR filename, contract frontmatter, and invariant-reference
  documentation checks when those documentation surfaces exist.
