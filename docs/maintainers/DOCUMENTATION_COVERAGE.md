# Documentation Coverage

## Status

This matrix records the code-outward documentation audit for the implemented
repository state through Phase 42. It does not imply that an
internal Rust boundary has a visible product workflow.

ADR-001 is accepted and Phase 33 is complete. Its accepted deferral is an
explicit boundary in this matrix and does not imply that PDF export exists.

## Audit Method

The audit starts from tracked implementation rather than existing prose. It
reviews:

- public and crate-visible Rust modules, boundary types, lifecycle functions,
  commands, stores, workers, constants, schemas, and failure enums;
- exported TypeScript clients, hooks, state types, presentation mappings, and
  visible messages;
- the Python helper protocol, versions, thresholds, and process entry point;
- Bash command surfaces, scans, and GitHub Actions parity;
- Tauri, Cargo, npm, capability, toolchain, persistence, export, and packaging
  configuration;
- tests that establish each implemented guarantee; and
- maintainer, user, Wiki-source, policy, architecture, invariant, and ADR
  ownership.

Accessors and enum variants are covered by their owning boundary type and
maintainer guide. The audit does not add repetitive comments that merely
narrate a field name. New public boundary behavior still requires a focused
documentation comment under `CODING_STYLE.md`.

## Coverage Matrix

The Gap column uses these coverage states:

- **Implemented and documented:** code, owning documentation, and evidence agree.
- **Implemented but user documentation absent:** an internal or repository
  source exists, but a real user workflow or publication surface is missing.
- **Documented but enforcement missing:** the rule is written down but no
  mechanical check proves it yet.
- **Proposed and blocked by governance:** the text remains non-binding until its
  governed decision completes.
- **Intentionally internal and not user-facing:** implementation exists, and
  user documentation explains its visible absence instead of inventing a flow.

| Subsystem | Code Surface | Maintainer Doc | User Doc | ADR | Invariant | Tests | Gap |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Desktop runtime and managed state | `src-tauri/src/lib.rs`; `application::{network_client,reference_store,job_store,secret_store,runtime_status}`; `run` | `TOOLCHAIN.md`, `COMMAND_BOUNDARY.md`, `CONFIGURATION.md`, `SECRET_STORAGE.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | Baseline stack decision | `INV-01`, `INV-03`, `INV-10`, `INV-13` | Rust application, command, store, and parity tests | Implemented and documented. |
| Workspace shell and editor | `DraftWorkspace`, `WorkspaceHeader`, `DocumentOutline`, `DocumentInspector`, `DraftEditor`, `EditorToolbar`, `ConnectivityModeControl` | `WORKSPACE_UI.md`, `OFFLINE_MODE.md`, `PERFORMANCE_MEASUREMENT.md` | `docs/wiki/Workspace.md` | None | `INV-03`, `INV-10` | `src/App.test.tsx`, connectivity, editor, and benchmark tests | Implemented and documented; live Wiki publication is synchronized through Phase 39 at `1bddd52`. |
| Runtime status and visible failures | `get_runtime_status`; `draft://runtime-status`; `startRuntimeStatusSession`; `useRuntimeStatus`; `runtimeFailurePresentation` | `COMMAND_BOUNDARY.md`, `EVENT_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `ERROR_MESSAGES.md`, `ERROR_UX.md` | `docs/wiki/Troubleshooting.md` | None | `INV-02`, `INV-03` | Rust command/event tests and exhaustive runtime presentation suites | Implemented and documented; live recovery guidance is synchronized through Phase 39 at `1bddd52`. |
| Typed Tauri command client | Ten registered Rust commands and matching wrappers under `src/ipc/` | `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `FORMATTING_UX.md`, `OFFLINE_MODE.md`, `AUDIT_DIAGNOSTICS.md` | Formatting review and connectivity mode use validated wrappers | None | `INV-01`, `INV-02`, `INV-03`, `INV-10`, `INV-16` | Command serialization tests, wrapper tests, bridge-name parity scan | Implemented and documented; only the existing visible workflows consume wrappers. |
| Transient worker cancellation | `WorkerCancellationRegistry`, `WorkerRegistration`, `WorkerCancellation`, `cancel_worker` | `CANCELLATION_BOUNDARY.md`, `COMMAND_BOUNDARY.md` | `docs/wiki/Current-Limitations.md` | None | `INV-07` | Cancellation registry, command, helper, and analysis tests | Intentionally internal and not user-facing; no visible worker exists. |
| Document envelope | `DocumentEnvelope`, `DocumentId`, `DocumentEnvelopeError`, `from_json_value` | `DOCUMENT_ENVELOPE.md` | `docs/wiki/Current-Limitations.md` | None | `INV-04`, `INV-09` | Envelope validation, serialization, citation, and command tests | Intentionally internal and not user-facing; no file controls exist. |
| Document registry | `DocumentRegistry::{open,open_from_path,update,update_source,close}` | `DOCUMENT_REGISTRY.md` | `docs/wiki/Current-Limitations.md` | None | `INV-06` | Registry concurrency, duplicate-open, close, and lifecycle tests | Intentionally internal and not user-facing; no file controls exist. |
| Document open, save, and atomic replacement | `open_document`, `save_document`, `write_document_atomically`; native dialog adapters | `DOCUMENT_SAVE_LOAD.md`, `ERROR_MESSAGES.md`, `CRITICAL_PATHS.md` | `docs/wiki/Current-Limitations.md` | None | `INV-03`, `INV-06`, `INV-09` | Command, persistence, interruption, durability, reopen, and Phase 41 critical-path tests | Intentionally internal and not user-facing; only the backend boundary exists. |
| Critical-flow evidence | `critical_paths_tests`; existing document, reference, citation, and DOCX paths | `CRITICAL_PATHS.md` and owning subsystem guides | Existing limitations only; no workflow was added | None | `INV-04`, `INV-06`, `INV-09` | Save/close/reopen, duplicate-open, citation-resolution, citation-rejection, package-reopen, and source-preservation assertions | Implemented as crate-level test evidence; no product orchestration or visible end-to-end workflow exists. |
| Reference record | `ReferenceRecord`, `ReferenceId`, `ReferenceRecordError`, schema version 1 | `REFERENCE_RECORD.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-03` | Record validation and serialization tests | Intentionally internal and not user-facing; no reference UI exists. |
| Reference store | `ReferenceStore::{open,create,get,get_by_citekey,list,update,delete}`; `reference_records` | `REFERENCE_STORE.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | None | `INV-03` | SQLite CRUD, migration, corruption, and concurrency tests | Intentionally internal and not user-facing; CRUD IPC is absent. |
| Citation node and resolution | `CitationNodeAttrs`, `collect_document_citations`, `resolve_citation`; TypeScript/Tiptap mirrors | `CITATION_NODE.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CRITICAL_PATHS.md` | `docs/wiki/Current-Limitations.md` | None | `INV-04` | Rust validation/resolution, persisted-reopen integration, and frontend fail-closed rendering tests | Intentionally internal and not user-facing; citation insertion is absent. |
| Bibliography consistency | `check_bibliography_consistency`, `BibliographyConsistencyReport` | `BIBLIOGRAPHY_CONSISTENCY.md` | `docs/wiki/Current-Limitations.md` | None | `INV-04` | Missing, orphaned, duplicate, ordering, and side-effect tests | Intentionally internal and not user-facing; no visible bibliography workflow exists. |
| Central network client | `NetworkClient::{new,get_metadata}`, `ConnectivityPolicy`, `NetworkService`, typed request errors | `NETWORK_CLIENT.md`, `OFFLINE_MODE.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-01`, `INV-03`, `INV-10` | Construction, timeout, rate-limit, response-bound, offline-denial, and scan tests | Implemented and documented; connectivity mode is visible, while metadata lookup remains internal. |
| Metadata providers | `lookup_crossref`, `lookup_semantic_scholar`, `lookup_unpaywall`; normalized `MetadataRecord` | `METADATA_LOOKUP.md`, `NETWORK_CLIENT.md` | `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-10` | Provider request and malformed-response tests | Intentionally internal and not user-facing; candidates are non-persistent. |
| External browser handoff | `open_in_system_browser`, `open_external_access`, `openExternalAccess`, shared offline policy | `EXTERNAL_BROWSER_HANDOFF.md`, `OFFLINE_MODE.md`, `COMMAND_BOUNDARY.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-01`, `INV-03`, `INV-10` | URL validation, fixed-origin, offline denial, command, wrapper, and opener scans | Intentionally internal and not user-facing; no research control exists. |
| Offline session policy | `ConnectivityPolicy`, `get_connectivity_mode`, `set_connectivity_mode`, `useConnectivityMode`, `ConnectivityModeControl` | `OFFLINE_MODE.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | Baseline offline policy | `INV-10` | State, command, pre-dispatch denial, IPC, stale-read, failure, toggle, and authority tests | Implemented and documented; process-local only, with live Wiki publication synchronized through Phase 39 at `1bddd52`. |
| OS-native secret storage | `SecretStore`, `SecretId`, `SecretValue`, `NativeSecretBackend`, managed `secret_store` state | `SECRET_STORAGE.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Baseline Rust secret ownership | `INV-01`, `INV-03` | Identifier/value bounds, replacement, missing/delete, malformed value, native-error mapping, managed-state, and authority scans | Implemented but intentionally internal: no credential, provider, command, event, frontend state, or visible settings workflow exists; the visible limitation is published at `96e15c7`. |
| Local diagnostic snapshot | `DiagnosticSnapshot`, `get_diagnostic_snapshot`, `getDiagnosticSnapshot` | `AUDIT_DIAGNOSTICS.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Existing Rust and frontend ownership | `INV-01`, `INV-02`, `INV-03` | Exact schema/order, byte bound, redaction, closed error, IPC validation, and authority scans | Implemented but intentionally internal: no control, hook, export, persistence, upload, telemetry, or support-submission workflow exists; the visible limitation is published at `96e15c7`. |
| PDF intake candidate | `prepare_explicit_pdf`, `WatchedPdfIntake::{record_change,confirm_stable}`, `PendingPdfImport` | `PDF_IMPORT.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-08` | Explicit, containment, symlink, quiet-period, and stable-size tests | Intentionally internal and not user-facing; watcher and UI are absent. |
| Durable PDF import jobs | `PdfImportJobStore` lifecycle; `pdf_import_jobs`; claim token, checkpoint, retry, cancellation, recovery | `BACKGROUND_JOBS.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-05` | Promotion/claim races, stale ownership, recovery, cancellation, and migration tests | Intentionally internal and not user-facing; scheduler and processing are absent. |
| AI orchestration | `AiAnalysisRequest`, `assemble_model_request`, `prepare_ai_analysis`, `run_ai_analysis`; stream traits/events | `AI_ORCHESTRATION.md`, `CANCELLATION_BOUNDARY.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-07`, `INV-14` | Context, provenance, stream, cancellation, and failure tests | Intentionally internal and not user-facing; provider and UI are absent. |
| Python helper process | `PythonHelperRunner`; protocol and helper versions; `draft_helpers.worker.process_request` | `PYTHON_HELPERS.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-11` | Rust process/protocol tests, Python contract tests, authority scans | Intentionally internal and not user-facing; packaged runtime discovery is absent. |
| Text-analysis findings | `TextAnalysisInput`, `TextAnalysisFinding`, five closed finding codes; Python heuristics | `TEXT_ANALYSIS.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-15` | Rust validation, Python heuristic, offset, limit, and authority tests | Intentionally internal and not user-facing; issue-card UI is absent. |
| Formatting review | `FormattingSnapshot`, `run_formatting_checks`, `run_formatting_review`, `runFormattingReview`, `useFormattingReview`, `FormattingReviewPanel` | `FORMATTING_CHECKS.md`, `FORMATTING_UX.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | None | `INV-16` | Domain, command, IPC validator, generation, target, interaction, keyboard, and accessible-label tests | Implemented and documented; live recovery guidance is synchronized through Phase 39 at `1bddd52`. |
| DOCX export | `compile_docx`, `export_docx`, strict model/package compiler, atomic write | `DOCX_EXPORT.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md`, `CRITICAL_PATHS.md` | `docs/wiki/Current-Limitations.md` | None | `INV-04`, `INV-09` | Parser, package, XML, resource-limit, citation-rejection, atomic-export, and reopened-source integration tests | Intentionally internal and not user-facing; no visible export control exists. |
| Error presentation | `errorPresentation.ts`; runtime/connectivity session state; formatting panel; citation node view | `ERROR_UX.md`, `ERROR_MESSAGES.md`, owning boundary guides | `docs/wiki/Troubleshooting.md`, `docs/wiki/Workspace.md` | None | `INV-02`, `INV-03`, `INV-10` | Exhaustive code/disposition, fallback, retained-state, announcement, focus, and control-semantics tests | Implemented and published through Phase 39 at `1bddd52`; typed but unwired failures intentionally have no speculative user copy. |
| Verification and repository tooling | `justfile`; `scripts/{bootstrap,build,format,verify,check-*}.sh`; `.github/workflows/verify.yml` | `TOOLCHAIN.md`, `CONFIGURATION.md`, this matrix | No user workflow | Baseline governance policy | `INV-12`, `INV-13` | Script syntax, scans, repository hygiene, and CI/local parity | Intentionally internal and not user-facing. |
| Packaging and application icons | `package:macos`; `package-macos.sh`; `check-packaging.sh`; active `app` target; five explicit icon paths | `PACKAGING.md`, `CONFIGURATION.md`, `TOOLCHAIN.md` | `docs/wiki/Current-Limitations.md` | None | `INV-13` for verification parity | Structured config checks plus clean unsigned Apple Silicon build, plist, architecture, executable, and embedded-icon validation | Implemented and documented for an unsigned macOS Apple Silicon `.app`; signing, notarization, installer images, publication, and other platforms remain release work. |
| PDF export decision | No dependency, command, runtime path, control, or generated PDF | ADR-001 and `PDF_EXPORT_DECISION.md` | `docs/wiki/Current-Limitations.md` | ADR-001 accepted | Accepted PDF deferral guard | Absence scan and full verifier | Implemented and documented as an accepted deferral; PDF remains intentionally unavailable. |
| Public Rust API comments | Externally reachable modules, types, functions, methods, variants, and fields | Owning subsystem guides and this matrix | No user surface | None | None | `cargo rustdoc -- -D missing_docs` audit probe | Documented but enforcement missing: 457 granular lint findings remain, mostly variants, fields, accessors, and module exports. A focused source-documentation change is required before enabling the lint. |
| Live GitHub Wiki | Canonical pages under `docs/wiki/`; <https://github.com/progentic/draft/wiki> | `DOCUMENTATION.md`, this matrix | Home, Workspace, Troubleshooting, Current Limitations | None | None | Offline source checks, remote-tree/hash comparison, and rendered navigation review | Live commit `1bddd52` matches all merged sources through Phase 39. |

## Detected Drift And Resolution

The audit detected these concrete gaps:

- the visible workspace had user guidance but no maintainer ownership guide;
- app-icon generation and explicit bundle paths had no packaging guide;
- significant defaults and limits were spread across implementation guides
  without one source-name index;
- the live GitHub Wiki was enabled but had no initialized page repository,
  resolved by publishing the four canonical pages; the accepted Phase 33 sync
  was live commit `1daf72b`;
- visible runtime failures had copy mappings but no user recovery article; and
- documentation verification did not enforce subsystem, configuration,
  recovery, Wiki-source, or README-scope coverage.

The new guides, canonical Wiki source, live publication, and
`scripts/check-docs.sh` checks close the repository-owned and user-publication
gaps. The matrix keeps the remaining public-API comment debt explicit instead
of treating presence checks as proof of completion. Future Wiki publication
must continue to use the same page contents without creating a second source
of truth.

## Live Publication Evidence

Live Wiki publication verified through Phase 39 at commit `1bddd52`:

- the remote Wiki tree contains only `Home.md`, `Workspace.md`,
  `Troubleshooting.md`, and `Current-Limitations.md`;
- each remote file has the same SHA-256 digest as its merged `docs/wiki/`
  source;
- Home opens Workspace, Troubleshooting, and Current Limitations;
- Workspace links to Troubleshooting and Current Limitations;
- every non-Home page returns to Home;
- rendered headings and lists preserve the canonical structure; and
- no `.md` navigation target, initialization placeholder, or live-only wording
  remains.

The Phase 36 Workspace, Troubleshooting, and Current Limitations updates and
the Phase 37 credential-workflow limitation were published after their
verified main merges. Remote hashes and rendered headings, lists, and
navigation were checked against the merged canonical sources.
Phase 39 Workspace and Troubleshooting recovery updates were then published at
`1bddd52` after the verified main merge. All four remote hashes and rendered
navigation paths were rechecked against merged canonical source.

## Audit Boundaries

- `README.md` remains the marketing landing page.
- `CHANGELOG.md` remains released changes only.
- No product behavior, command, schema, state transition, or phase status is
  changed by this audit.
- Accepted ADR-001 is not restated as implemented PDF behavior.
- A future visible workflow must add or update both its maintainer guide and a
  page under `docs/wiki/` before merge.
