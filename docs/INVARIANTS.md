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
Phase 3 established the aggregate `verify` job, which remains the implemented
hosted entry point while dedicated invariant and build workflows are deferred.

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
| `INV-01` | Accepted | Institutional and publisher credentials never pass through DRAFT process memory, config, logs, database, document files, Python helpers, Bash scripts, or storage. Service API keys use only the Rust-owned OS-native secret store and never enter frontend-reachable storage. | `ARCHITECTURE.md` §4.2 and §9 and `GOVERNANCE.md` §9 | Phase 23 keeps login sessions in the system browser. Phase 37 adds a lazy native service API-key store with bounded zeroizing values, closed errors, injected non-native tests, and narrow scans. Phase 38 omits credential state entirely and denies secret-store access from diagnostics. | The `verify` job runs the same handoff, secret-store, diagnostic-redaction, authority, dependency, value-trait, and native-adapter tests and scans. |
| `INV-02` | Accepted | No Tauri command returns an untyped or generic error. | `ARCHITECTURE.md` §5.1 and §12 | Phases 6, 9, 13, 18, and 23 establish the initial typed commands. Later commands retain the same request, response, error, registration, and test contract. Phase 38 adds three closed diagnostic errors without raw details. Phase 39 exhaustively maps only typed failures already visible in the workspace and keeps unknown outer failures in deterministic fallbacks. | The `verify` job runs the same Rust tests, command-contract counts, bridge-name parity scan, and visible error-presentation tests. |
| `INV-03` | Accepted | Frontend code never calls external network services, filesystem APIs, secret stores, or external opener APIs directly. All trusted work goes through Rust commands. | `ARCHITECTURE.md` §4.1, §9, and §13 | `scripts/check-invariants.sh` blocks direct trusted APIs, opener bindings, `window.open`, and raw command/event APIs outside `src/ipc/`. Typed wrappers carry bounded inputs without paths, SQLite authority, full reference metadata, browser-session authority, or frontend-generated document identities. Phase 46 creates document IDs in Rust, returns only validated envelopes, and awaits Rust-owned native dialog callbacks instead of blocking the application thread. | The `verify` job runs the same frontend tests, boundary scans, async-dialog guard, document-identity guard, diagnostic non-consumer scan, authority scans, and bridge-name parity check. |
| `INV-04` | Accepted | Citation node attrs are validated against a declared schema version before render, analysis, formatting, save, or export. Invalid or unknown versions are migration cases, never silent render cases. | `ARCHITECTURE.md` §8 and §11 | Phase 18 requires exact Rust attrs tests, nested envelope and pre-mutation open/save rejection, store-backed resolution, typed IPC guards, Tiptap fail-closed tests, and an embedded-metadata scan. Phase 19 reuses validated attrs for deterministic bibliography-consistency tests. Phase 32 rejects every validated citation explicitly rather than exporting its editor marker. Phase 43 proves lower and future document, citation, and reference versions fail without changing source or stored state. | The `verify` job runs the same Rust/frontend tests, consistency and DOCX citation-rejection tests, Phase 43 non-mutation tests, and citation invariant scans through `scripts/verify.sh`. |
| `INV-05` | Accepted | Background jobs persist state per record and resume from the last valid checkpoint after interruption. | `ARCHITECTURE.md` §10 | Phase 26 transactionally promotes one candidate identity into one SQLite job, permits one hashed opaque claim, requires current ownership for every in-progress mutation, persists attempts/checkpoints/cancellation/failures, and invalidates stale claims during recovery. Two-connection races, close/reopen recovery, typed ownership errors, and terminal immutability are required. | The `verify` job runs the same Phase 26 job tests and source-boundary scans through `scripts/verify.sh`. |
| `INV-06` | Accepted | A document can have only one live editing handle at a time. No two Tiptap instances may hold a live handle to the same document. | `ARCHITECTURE.md` §6 | Phase 12 establishes the process-local registry. Phase 13 retains exclusive Rust-selected source paths there and tests duplicate load, path alias rejection, save, close, and reopen behavior without creating a second handle. | The `verify` job runs the same registry, persistence, command, and frontend wrapper tests plus the invariant scan. |
| `INV-07` | Accepted | Every user-initiated long-running Rust worker that emits progress events has a user-visible cancellation or abort path unless documented as non-cancelable and idempotent. | `ARCHITECTURE.md` §5.3 and §10 | Phase 9 provides the Rust cancellation registry/token, typed cancel command and wrapper, active/repeated/already-ended/error/shutdown tests, and a scan that confines worker spawning to `src-tauri/src/workers/`. Phases 27 through 29 exercise internal cancellation lifetimes without adding a Tauri start command, detached product worker, progress event, or visible workflow. | The `verify` job runs the same Rust/frontend tests, cancellation-boundary scan, Phase 27 analysis cancellation tests, Phase 28/29 helper cancellation and reaping tests, and bridge-name parity check. Worker-specific terminal-event tests are required when start commands are introduced. |
| `INV-08` | Accepted | A watched-folder file enters the import pipeline only after stable-write confirmation. | `ARCHITECTURE.md` §10.1 | Phase 24 enforces root-confined canonical paths, event-driven deadline resets, a one-second quiet window, unchanged byte length, signature validation, and the chunked-write regression test before returning a candidate. The size-only rule cannot detect an unreported same-size in-place change. The intake gate has no watcher, worker, persistence, or frontend flow; Phase 26 persistence begins only after candidate promotion. | The `verify` job runs the same Rust tests and import-boundary scans through `scripts/verify.sh`. |
| `INV-09` | Accepted | Document saves and derived exports use atomic write-then-rename. A target is always the prior complete version or the new complete version, never a partial write, and export never changes source state. | `ARCHITECTURE.md` §11 | Phase 14 routes saves through one same-directory synchronized atomic writer. Phase 32 compiles DOCX fully before reusing that writer, maps every write stage, preserves source bytes, and reports post-replacement durability uncertainty. Phase 46 validates eleven canonical font families and whole point sizes from 8 through 72 before save or export, then emits exact DOCX run properties without substitution. Accepted ADR-001 keeps a named PDF deferral guard until a governed implementation can prove the same source-safety contract. Interruption, cleanup, replacement, export, font-fidelity, and disk/registry tests plus direct-write and deferral scans are required. | The `verify` job runs the same atomic-writer, persistence, DOCX package/export, font-contract, direct-write, command, frontend, and PDF-deferral boundary checks through `scripts/verify.sh`. |
| `INV-10` | Accepted | All outbound requests to external services go through the centralized Rust network client, and the Rust-owned offline session policy denies new metadata or browser work before dispatch. Feature code, frontend code, and Python helpers must not create ad hoc external network clients or bypass the policy. | `ARCHITECTURE.md` §12.1 and §13 | Phases 21/22 centralize HTTPS request policy. Phase 36 shares one process-local policy with metadata and browser handoff, defaults online, returns typed offline failures, and exposes only closed get/set commands. Tests prove pre-dispatch denial and unchanged online behavior; scans deny persistence, probing, alternate transports, and formatting coupling. | The `verify` job runs construction, request-policy, provider, browser, connectivity command/client/hook/control, local-formatting, and boundary checks through `scripts/verify.sh`. |
| `INV-11` | Accepted | Python helpers are allowlisted, versioned, typed worker tools. They do not own persistence, secrets, source-document mutation, or external network access by default. | `ARCHITECTURE.md` §4.3, §5.3, and §11 | Phase 28 requires a fixed canonical entrypoint, closed protocol/helper versions, exact bounded JSON, isolated cleared environment, bounded captured streams, timeout, cooperative cancellation, kill/reap behavior, typed failures, empty dependency set, and Rust validation. Phase 29 extends only the closed allowlist and result validation. Python and Rust tests plus source scans deny helper network, credential, database, filesystem, environment, subprocess, persistence, mutation, Tauri, and frontend authority. | The `verify` job runs the same Phase 28/29 Python and Rust tests plus helper-boundary scans through `scripts/verify.sh`. |
| `INV-12` | Accepted | Bash is local/CI orchestration only. The product runtime must not use Bash for document processing, credential handling, external network access, or user-supplied path execution. | `ARCHITECTURE.md` §4.4 and `GOVERNANCE.md` §8 | Phase 2 runs Bash syntax checks and scans Rust for Bash runtime execution. `shellcheck` and `shfmt` run when installed. | The Phase 3 `verify` job runs Bash syntax and runtime-boundary checks. Optional tools run when present. |
| `INV-13` | Accepted | Local verification and GitHub Actions verification use the same underlying scripts where practical. A check that blocks CI must be runnable locally unless it depends on GitHub metadata. | `ARCHITECTURE.md` §14 and `GOVERNANCE.md` §8 | The root `justfile` delegates to repository scripts, with direct Bash fallbacks. | `.github/workflows/verify.yml` calls `scripts/verify.sh`; `scripts/check-ci-local-parity.sh` enforces that mapping. |
| `INV-14` | Accepted | Model-generated output remains explicitly classified as generated analysis. It must not be tagged, persisted, or promoted as verified source evidence. | `ARCHITECTURE.md` §3.2 | Phase 27 preserves typed `UserDocument` and `VerifiedSourceEvidence` context blocks, classifies every stream event as `GeneratedAnalysis`, reports evidence IDs only as context scope, and rejects unbounded input or output. Tests cover provenance, serialization, cancellation, and failures; scans deny provider, secret, network, persistence, mutation, Tauri-start, frontend, Python, and spawn authority. | The `verify` job runs the same Phase 27 tests and source-boundary scans through `scripts/verify.sh`. |
| `INV-15` | Accepted | Text-analysis output is review-only. A helper finding cannot mutate source text, carry an automatic replacement, or become durable without a separate Rust-owned user-action path. | `ARCHITECTURE.md` §3.4 and §11 | Phase 29 accepts only five closed finding codes and validated UTF-8 byte ranges, maps all review wording in Rust, and exposes immutable results with no source copy, replacement, score, apply, persistence, command, event, or frontend path. Rust/Python tests cover heuristics, limits, offsets, explanations, and false-positive guards; scans deny mutation and authority expansion. | The `verify` job runs the same Phase 29 Rust/Python tests and text-analysis boundary scans through `scripts/verify.sh`. |
| `INV-16` | Accepted | Formatting findings are review-only consistency signals. A supported style identifier does not claim complete conformance, and no finding changes content without an explicit current-target user action. | `ARCHITECTURE.md` §3.3 and §11 | Phase 31 validates a bounded immutable snapshot and returns content-free indexed findings. Phase 34 adds a typed command, closed actions, generation invalidation, and exact-node guards. Citation findings remain inspect-only; heading apply requires user input. Tests and scans deny persistence, filesystem, export, PDF, Python, network, worker, and automatic mutation authority. | The `verify` job runs the Rust domain/command tests, frontend IPC/generation/target/interaction tests, and formatting-boundary scans through `scripts/verify.sh`. |
| `INV-17` | Proposed | Paragraph formatting uses one strict canonical model across editor behavior, Rust validation, persistence, migration, and supported format mappings. Unsupported values and undisclosed lossy saves fail before mutation. | Accepted ADR-004 and `ARCHITECTURE.md` §15.1 | Phase 47 implements strict Rust and TypeScript values, current-only v2 parsing, detached v1 migration, persistence non-mutation, editor JSON preservation, and deterministic DOCX properties. Controls, external imports, lossiness decisions, and packaged evidence remain absent. | The `verify` job requires the model and named tests, pins schema v2 and the migration boundary, denies paragraph controls, and keeps the invariant Proposed. It cannot close a release gate or substitute for external-format and human evidence. |
| `INV-UX-01` | Accepted | Every enabled visible control invokes an implemented user workflow. Controls for unavailable capabilities do not appear active. | `ARCHITECTURE.md` §4.1 and §5.1 | Interaction tests and the v1 usability gate require implemented control outcomes and explicit unavailable states before workflow closure. | The `verify` job runs the same interaction tests and conditional release-evidence checks. |
| `INV-UX-02` | Accepted | User-facing text does not expose internal command, schema, provider, registry, job, IPC, or persistence terminology. | `ARCHITECTURE.md` §4.1 and §12 | Phase 49 inventories every visible string; prohibited implementation wording remains a release finding until corrected or removed. | The `verify` job requires the accepted contracts and blocks gate closure without the visible-language evidence ledger. |
| `INV-UX-03` | Accepted | Long-running or fallible visible operations expose deterministic pending, success, and failure states. | `ARCHITECTURE.md` §5.2, §5.3, and §12 | Operation interaction tests and perceived-performance evidence must prove visible state transitions without duplicate activation. | The `verify` job runs interaction tests and blocks Phase 49 closure without the required state and timing evidence. |
| `INV-UX-04` | Accepted | Every visible failure states whether document data remains safe and exposes only recovery actions the current interface can honor. | `ARCHITECTURE.md` §12 | Typed error-presentation tests remain exhaustive; Phase 46 and Phase 51 evidence must cover data-safety wording and safe recovery. | The `verify` job runs error presentation tests and conditional usability/security evidence checks. |
| `INV-UX-05` | Accepted | A user-facing concept has one canonical name across menus, controls, errors, documentation, and accessibility labels. | `ARCHITECTURE.md` §4.1 | Phase 49's visible-language inventory records each concept and resolves inconsistent terminology before release. | The `verify` job blocks Phase 49 closure without terminology and first-time-user comprehension evidence. |
| `INV-UX-06` | Accepted | The supported v1 workflow is completable with keyboard input without lost focus or inaccessible controls. | `ARCHITECTURE.md` §4.1, §5, and §12 | Phase 46 interaction tests and Phase 52 packaged validation must cover the complete keyboard-only workflow. | The `verify` job blocks the accessibility and candidate gates without exact keyboard evidence. |
| `INV-UX-07` | Proposed | Every major maintainer subsystem guide begins with a plain-language explanation of the problem and solution before normative requirements or implementation details. | `DOCUMENTATION.md` §2.1 and accepted ADR-003 Phases 49-50 | Accepted decision checks pin the human-first rule and required future section order. Phase 50 must realign existing major guides, add structural heading checks, and record maintainer-onboarding review before this invariant can become Accepted. | The `verify` job enforces Proposed status. Accepted enforcement remains pending and no current release gate may treat the invariant as closed. |

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

Phase 23 accepts no login fields. Publisher and institutional browser targets
reject URL username and password components before launch. The handoff uses the
default system browser and cannot inspect or receive the resulting session.

Phase 37 adds one lazy `SecretStore` for service API keys only. Its fixed
`com.progentic.draft` native namespace uses bounded normalized internal account
slots. `SecretValue` owns `Zeroizing<Vec<u8>>`, has no clone, formatting, or
serialization implementation, and never crosses a Tauri command or event.

The production adapter uses binary Keychain, Credential Manager, or Secret
Service operations through the pinned Rust keyring dependency. Native details
are discarded during closed error mapping. Tests inject an in-memory backend
and never access a real credential manager. There is no frontend, Python,
config, SQLite, filesystem, environment, log, provider, network, or fallback
secret path.

Phase 38 diagnostics omit the secret-storage subsystem and account identifiers.
The diagnostic command receives no secret state and scans reject native
credential or secret-store operations from its source boundary.

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

Phase 39 retains each visible typed failure through the presentation boundary.
Compile-time `Record` mappings and exhaustive switches cover runtime status,
connectivity, formatting review, and citation rendering. Unknown raw IPC input
still becomes the existing bounded transport fallback before presentation.

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
@tauri-apps/plugin-opener
window.open(...)
target="_blank"
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

Phase 39 adds only frontend presentation policy. It reads no trusted state and
adds no command, event, persistence, network, filesystem, secret, diagnostic,
or worker authority. Typed errors for surfaces with no visible workflow remain
unmapped and cannot acquire speculative copy through the shared policy.

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
- Guessing, downgrading, or silently normalizing an unsupported persisted schema.

Phase 43 establishes version 1 as the first released-schema baseline. There is
no older document, citation, or reference payload to transform. Lower and
future versions fail before registry or storage mutation; any later supported
transition must be explicit, validated, atomic or transactional, and covered by
rollback tests.

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

Phase 26 implements this invariant first for PDF import jobs. `PdfImportId` is
the exact deduplication key, and promotion is committed before any processing.
One `InProgress` row has one claim-token digest and claim timestamp. The raw
UUID v4 capability stays inside Rust, is never persisted or serialized, and has
redacted debug output.

Checkpoint, completion, failure, manual-input, and cancellation acknowledgment
updates require the current token digest and expected state in the same SQLite
statement. Cancellation intent blocks completion and failure even for the
current owner. Restart recovery clears the old claim, preserves the checkpoint
and attempt count, and requeues or cancels the job. Resolved and cancelled jobs
are immutable; failed and manual-input jobs reopen only through explicit
expected-attempt controls.

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

Phase 9 establishes the reusable transient-worker cancellation contract:

- Rust generates an opaque UUID and returns it from the future start command.
- The worker retains a registration guard and observes its cancellation token.
- `cancel_worker` returns `cancellation_requested` for an active worker.
- Repeated requests remain successful while the worker is active.
- A known terminal worker returns `already_ended` instead of an error.
- Malformed and unknown worker IDs return distinct typed errors.
- Dropping the application registry cancels every active token.
- The owning feature must emit its typed terminal event before its worker exits.

The registry is process-local and is not the persistent job state machine from
Phase 26. Phase 27 analysis coordination and the Phase 28/29 Python runner each
retain a registration and observe cancellation inside their Rust caller's
lifetime. They add no Tauri start command, detached product task, frontend
listener, or visible progress event. No long-running product worker exists at
this checkpoint. A future start command may not ship until it retains the
registration guard, observes the token, exposes the typed frontend cancel
action, and tests its terminal event.

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
6. Assert a pending intake candidate is returned once stable.

Phase 24 implements this as a Rust-only intake candidate, not a persistent job.
Every recorded change resets a one-second deadline. Confirmation checks the
current byte length against the recorded observation, then rechecks length
around the `%PDF-` signature read. A changed size returns to waiting. Only a
stable valid file under the canonical watched root becomes pending.

This is an implemented size-based contract, not a content-identity guarantee.
If another process changes bytes in place without changing the file length and
no filesystem event is delivered, Phase 24 cannot detect that modification. A
delivered event always resets the quiet period, including when the observed
length is unchanged. Phase 24 provides no watcher, worker, queue, persistence,
or UI import flow.

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

Phase 32 reuses the same atomic writer for a derived `.docx` target only after
strict validation and complete in-memory package construction. Export accepts a
Rust-owned `.docx` path, never a source `.draft` or `.json` target. Failures
before replacement leave an existing export unchanged; a parent-directory sync
failure reports durability uncertainty after a complete replacement.

DOCX tests reopen the deterministic package, parse every XML part, reject
unsupported or citation content, map every atomic stage, replace a real prior
target, and prove the source bytes remain unchanged. Source scans confine raw ZIP
writes to an in-memory `ZipWriter` and deny direct export filesystem authority
outside the shared atomic writer.

Accepted ADR-001 retains the Phase 33 PDF absence scan as a named deferral
guard. It denies PDF export symbols, frontend claims, known conversion
executables, renderer dependencies, and bundled runtime paths. The guard does
not imply that PDF behavior exists; it preserves the current absence until a
governed implementation adds parser-based output, resource-bound,
deterministic-failure, and source-preservation tests.

Accepted ADR-003 extends `INV-09` to external-format work. Source identity and
all writes remain in Rust, no-edit external files remain byte-for-byte
unchanged, and same-format save is denied when current content cannot be
represented safely. The accepted decision guard continues to reject
external-document ownership, round-trip/lossiness state, and format
parser/save-back commands before Phase 47. Phase 48 replaces the native-menu
absence check with structural enforcement for the Rust menu, typed event and
state boundaries, shared frontend dispatcher, path-free DTO, exact File action
set, and visible command parity. Packaged interaction evidence remains required
before `RC-08` or `GATE-48` can close.

---

### INV-10: Centralized Network Client

All external requests go through the Rust network client. This protects rate limiting, backoff, logging policy, offline detection, and User-Agent behavior.

A Phase 23 system-browser handoff is not a DRAFT network request. Rust launches
the user's default browser and neither performs nor observes that browser's
request. Any automated request made by the DRAFT process remains subject to
this invariant.

Phase 36 adds one shared `ConnectivityPolicy`. New sessions default online.
When explicitly offline, the centralized metadata client fails before rate
reservation, URL parsing, or socket work, and the browser handoff fails before
URL validation or opener invocation. Rust remains authoritative for the value;
frontend state mirrors typed get/set responses only.

The mode is process-local and resets on restart. It does not infer
operating-system reachability, probe the network, persist a preference, queue or
retry work, cancel already dispatched operations, or change local formatting
behavior. Transport connection failures remain typed `Offline` results without
silently changing the selected mode.

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

Phase 28 implements the process and protocol boundary with one non-product
`contract_probe`. Phase 29 adds `text_analysis` version 1 to the same closed
allowlist. Rust generates request identity, selects the helper and version,
canonicalizes a fixed entrypoint, clears inherited environment, and owns process
creation, bounded stdin/stdout/stderr, timeout, cancellation, kill/reap, exit
interpretation, and response validation. The request contains only bounded text
and a closed locale; it carries no path, command, environment, credential,
persistence, document, or reference field.

The production Python package uses the standard library only. Scans reject file
and directory access, environment inspection, network, credential, database,
and subprocess APIs. The probe emits a byte count; text analysis emits only five
closed codes and UTF-8 byte ranges. Rust validates both exact result shapes, and
Python never decides whether a result becomes durable state or changes a
document.

Minimum local checks:

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=python \
  python3 -m unittest discover -s python/tests -v
bash scripts/check-invariants.sh
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

### INV-14: Generated Analysis Provenance

Model output is assistance, not evidence. Every Phase 27 stream event is tagged
`GeneratedAnalysis`, including started, chunk, completed, cancelled, and failed
updates. Evidence IDs in the started context summary disclose which verified
inputs were available; they do not claim that the model used, checked, or
verified a statement.

Context retains the distinction between `UserDocument` and
`VerifiedSourceEvidence` through the adapter boundary. Verified blocks keep
their evidence ID and citekey, and the model request is never flattened into an
untyped prompt. Context limits are deterministic, preserve whole UTF-8 blocks,
and report omissions separately for each provenance class.

Phase 27 does not persist generated output, mutate documents or references,
call an external provider, accept credentials, or expose a Tauri start command
or frontend stream. Adding any verification or promotion path from generated
analysis to evidence requires an explicit governed contract and tests that keep
the two states distinguishable.

Accepted ADR-002 keeps provider-backed orchestration internal for v1.0.0 and
permits only the separate deterministic local text-analysis boundary. The
decision guard rejects production model providers,
credentials, external-model requests, generative-analysis IPC, and frontend
generative claims.

The guard also preserves the decision's three-layer distinction: internal
measurements cannot become unnamed product capabilities, the five existing
heuristics must remain explicitly bounded signals, and model-backed semantic,
generative, or inferential interpretation remains absent. It denies provider
SDKs and endpoints, credential environment variables, model artifacts and
downloads, direct frontend provider access, and unsupported documentation or UI
claims.

Minimum verification:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline analysis::
bash scripts/check-invariants.sh
```

---

### INV-15: Review-Only Text Analysis

Text-analysis findings identify passages for human review. They do not prove an
error and do not authorize a change. Phase 29 helper output contains only a
closed code and a half-open UTF-8 byte range. Rust rejects unknown, duplicate,
unsorted, excessive, reversed, out-of-bounds, or non-character-boundary
findings before constructing an immutable result.

Rust owns each code's category, severity, title, and explanation. Python cannot
inject source text or user-facing prose. No finding contains a score,
replacement, apply instruction, document identity, source path, citation,
reference, or persistence field.

The current result is process-local and has no Tauri command, event, frontend
model, durable store, or document mutation path. Any future accepted-edit flow
must require an explicit user action and pass through the existing Rust-owned
document mutation and save boundaries.

Minimum verification:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline workers::python::
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=python \
  python3 -m unittest discover -s python/tests -v
bash scripts/check-invariants.sh
```

---

### INV-16: Review-Only Formatting Checks

Phase 31 formatting findings identify inconsistent declarations or outline
relationships for human review. They do not certify a document against APA,
MLA, Chicago, or another complete style manual and do not authorize a change.

The immutable snapshot accepts at most 512 headings and 512 citation-style
declarations. Heading levels are 1 through 6, titles are non-blank and at most
512 UTF-8 bytes, and citekeys reuse the existing reference-domain validator.
The supported style identifiers are exactly `apa7`, `mla9`, and
`chicago17_author_date`.

The pure checker reports only a non-level-one first heading, a skipped heading
level, or a citation style that differs from the selected style. Each finding
contains a closed code, fixed Rust-owned severity and wording, and a heading or
citation index. It contains no source title, citekey, document text, score,
replacement, patch, apply instruction, path, or document identity.

Phase 34 exposes the checker through one typed command. Rust pairs each finding
with closed actions: inspect and dismiss for every finding, plus one bounded
heading level where the domain relationship determines it. Citation findings
never receive apply authority.

The frontend ties each response to a unique run and editor generation. Any
editor update or newer run invalidates the result. Inspect and apply verify that
the indexed node still has the captured type and content before acting. Only an
explicit user-triggered heading action uses the normal Tiptap transaction;
dismissal remains transient.

No event, persistence, filesystem call, network call, Python helper, worker,
parser, save, export, PDF path, automatic repair, or complete style-manual claim
can extend this boundary at the current checkpoint.

Minimum verification:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline formatting::
npm test -- --run src/ipc/formattingReview.test.ts src/features/formatting-review src/App.test.tsx
bash scripts/check-invariants.sh
```

---

### INV-UX-01 Through INV-UX-06: v1 Usability Integrity

The accepted `docs/contracts/V1_USABILITY_ACCEPTANCE.md` contract defines the
supported workflow, measurable thresholds, and evidence required to enforce
these invariants. Phase 46 implements the visible document, reference/citation,
five-check local analysis, and DOCX paths, but implementation alone does not
close their release rows.

Current enforcement preserves that distinction. `RC-01` through `RC-08` and
`GATE-46` through `GATE-51` remain open while stable complete packaged Phase 46
evidence and every later phase's required evidence are missing. The
release-candidate script rejects a closed gate unless the cumulative usability
evidence ledger contains the phase-specific automated, packaged, and human
evidence named by the accepted contracts. Phase 52 must additionally reject
open `UX-0` and `UX-1` findings and require a
disposition for every `UX-2` finding.

Phase 46 formatting evidence keeps font choices inside the same contract:
controls expose only eleven canonical families and whole point sizes from 8
through 72; Rust validates them before persistence or export; DOCX writes
explicit run properties; and editor focus returns after keyboard selection.
The exact packaged result remains open until the corrected artifact is rebuilt,
hashed, and manually exercised.

Document lifecycle state is explicit rather than inferred from identity.
`new` and `imported_text` both carry Rust-owned in-memory IDs without durable
paths, while `opened_draft` has a Rust-registered path. Imported text returns
only a source filename for display, remains unsaved until first Save, and can
never reuse the `.txt` or `.md` source as a persistence target.
The document-file scan requires the three exact frontend origins, the two
distinct Rust open outcomes, the 8 MiB import bound, async dialog filters, and
named source-preservation, malformed-input, first-save, later-save,
cancellation, and transition tests. The formatting scan compares the exact
Rust and TypeScript identifier sets and requires toolbar derivation, canonical
HTML attributes, bounded sizes, and every-family DOCX mapping evidence.

Automated source scans may protect bounded terminology and authority rules, but
they cannot prove that a first-time user understood a control. Uncoached human
task evidence remains mandatory for Phase 49 and packaged rerun evidence remains
mandatory for Phase 52.

Accepted ADR-003 moves human validation to Phase 49 after separate
interoperability and desktop-workflow phases, preserves Phase 50 as
realignment, and moves security and candidate work to Phases 51 and 52. The
decision remaps ownership without closing any release row or fabricating
evidence.

Minimum verification:

```bash
bash scripts/check-docs.sh
bash scripts/check-invariants.sh
bash scripts/check-release-candidate.sh
bash scripts/verify.sh
```

---

### INV-UX-07: Documentation Readability (Proposed)

Maintainer documentation is a teaching surface as well as a specification.
A competent engineer who has not seen DRAFT must be able to understand the
subsystem's purpose, problem, solution, trade-offs, change boundary, protected
rules, code location, failure modes, and test evidence before parsing
implementation detail.

The required layered structure is defined in `DOCUMENTATION.md` §2.1. New or
substantively revised major guides follow it immediately. Phase 49 reviews
documentation terminology and comprehension with engineers unfamiliar with
the repository. Phase 50 realigns existing major guides and adds
structural checks for the required sections.

This invariant remains Proposed. Presence checks cannot prove that an
explanation is clear, and the existing guide set has not completed the Phase 50
realignment. Human review plus mechanical section enforcement are both required
before a governed change may mark `INV-UX-07` Accepted.

Minimum current verification:

```bash
bash scripts/check-docs.sh
bash scripts/check-invariants.sh
bash scripts/check-release-candidate.sh
```

---

### INV-17: Paragraph-Formatting Integrity (Proposed)

Paragraph controls must not become a second formatting model. Accepted ADR-004
defines one strict block shape for what the editor can apply, what Rust can
validate and save, and what supported document formats can preserve.
Unsupported values and undisclosed lossy save-back fail before state or source
mutation.

This invariant remains Proposed. Phase 47 implements the canonical values,
validation, version 1 to version 2 migration, editor JSON preservation,
persistence behavior, and DOCX property mapping. It does not yet implement
paragraph commands, mixed-selection behavior, reset behavior, external-format
imports, lossiness handling, or packaged controls. Those remaining behaviors
and evidence are required before a governed change may mark `INV-17` Accepted.

Minimum current verification:

```bash
bash scripts/check-docs.sh
bash scripts/check-invariants.sh
bash scripts/check-release-candidate.sh
```

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
the Phase 11 document-envelope schema and malformed-shape tests, the Phase 12
single-live-handle registry contract, Phase 13 validated load/save, Phase 14
atomic-save hardening, the Phase 16 reference-record schema and malformed-shape
tests, Phase 17 transactional reference-store CRUD and migration tests, Phase
18 citation validation and resolution, Phase 19 bibliography consistency,
Phase 21 centralized network-client construction, Phase 22 metadata lookup and
request policy, Phase 23 external browser handoff, direct frontend opener APIs,
Phase 24 PDF intake and stable-write confirmation, Phase 26 persistent job
ownership and recovery, Phase 27 bounded AI orchestration and generated-output
provenance, Phase 28 Python helper protocol/process confinement, Phase 29
review-only text-analysis findings, Phase 31 review-only formatting checks,
Phase 32 strict atomic DOCX export, ad hoc Rust network clients, and Bash
invocation from product runtime. The verifier also checks locked offline builds,
tests, required source visibility, generated-file hygiene, and documentation
sanity.

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
- Add frontend view integration and close-command coverage when those surfaces
  exist.
- Add job, product-worker cancellation, and import invariant tests in their
  owning phases.
- Make `shellcheck`, `shfmt`, Ruff, frontend formatting, and frontend linting
  required once their versions and CI installation paths are pinned.
- Add local-link, ADR filename, contract frontmatter, and invariant-reference
  documentation checks when those documentation surfaces exist.
