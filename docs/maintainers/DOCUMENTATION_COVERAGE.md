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
| Typed Tauri command client | Sixteen registered Rust commands and matching wrappers under `src/ipc/` | `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `FORMATTING_UX.md`, `OFFLINE_MODE.md`, `AUDIT_DIAGNOSTICS.md`, `PHASE46_WORKFLOWS.md` | Document, reference/citation, text-check, formatting, connectivity, and DOCX workflows use validated wrappers | None | `INV-01`, `INV-02`, `INV-03`, `INV-10`, `INV-16` | Command serialization tests, wrapper tests, bridge-name parity scan | Implemented and documented; internal commands remain unconsumed where no visible workflow exists. |
| Transient worker cancellation | `WorkerCancellationRegistry`, `WorkerRegistration`, `WorkerCancellation`, `cancel_worker` | `CANCELLATION_BOUNDARY.md`, `COMMAND_BOUNDARY.md` | `docs/wiki/Current-Limitations.md` | None | `INV-07` | Cancellation registry, command, helper, and analysis tests | Intentionally internal and not user-facing; no visible worker exists. |
| Document envelope | `DocumentEnvelope`, `DocumentId`, `DocumentEnvelopeError`, `from_json_value`; bounded `fontFamily` and `fontSize` marks | `DOCUMENT_ENVELOPE.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | None | `INV-04`, `INV-09` | Envelope validation, serialization, citation, mirrored font-shape, and command tests | Implemented and used by the visible document workflow without frontend path authority or arbitrary font values. |
| Document registry | `DocumentRegistry::{open,open_from_path,update,update_source,close}` | `DOCUMENT_REGISTRY.md`, `PHASE46_WORKFLOWS.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md` | None | `INV-06` | Registry concurrency, duplicate-open, close, lifecycle, and visible-session tests | Implemented and used by the visible document session; exactly one live handle remains enforced. |
| Document create, open, import, save, close, and atomic replacement | `create_document`, `open_document`, `save_document`, `close_document`, `text_import`, `write_document_atomically`, `useDocumentSession`; async native dialog adapters | `DOCUMENT_SAVE_LOAD.md`, `PHASE46_WORKFLOWS.md`, `ERROR_MESSAGES.md`, `CRITICAL_PATHS.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md` | None | `INV-03`, `INV-06`, `INV-09` | Rust-owned identity, explicit origin, bounded UTF-8 import, source preservation, async dialog, persistence, cancellation non-mutation, interruption, durability, reopen, dirty-state, handle-release, and Phase 41 critical-path tests | Implemented and visible; imported `.txt` and `.md` remain unsaved until a new `.draft` target is chosen. Corrected packaged evidence remains pending. |
| Critical-flow evidence | `critical_paths_tests`; document, reference, citation, and DOCX paths; Phase 46 visible integration tests | `CRITICAL_PATHS.md`, `PHASE46_WORKFLOWS.md`, and owning subsystem guides | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md` | None | `INV-04`, `INV-06`, `INV-09` | Save/close/reopen, duplicate-open, citation-resolution, citation-rejection, package-reopen, source-preservation, and interaction assertions | Implemented across Rust boundaries and visible frontend coordination without test-only production orchestration. |
| Reference record | `ReferenceRecord`, `ReferenceId`, `ReferenceRecordError`, schema version 1 | `REFERENCE_RECORD.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | None | `INV-03` | Record validation, serialization, command, and interaction tests | Manual creation is visible; edit, delete, import, and full stored payloads remain unexposed. |
| Reference store | `ReferenceStore::{open,create,get,get_by_citekey,list,update,delete}`; `add_reference`, `list_references`; `reference_records` | `REFERENCE_STORE.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md` | None | `INV-03` | SQLite CRUD, migration, corruption, concurrency, command, and interaction tests | Bounded create/list summaries are visible; update/delete/import IPC remains absent. |
| Data migration baseline | Version 1 document envelope, citation attrs, reference payload, and reference-store dispatch | `DATA_MIGRATION.md`, `DOCUMENT_ENVELOPE.md`, `CITATION_NODE.md`, `REFERENCE_RECORD.md`, `REFERENCE_STORE.md`, `CONFIGURATION.md` | None; no visible migration workflow exists | None | `INV-04`, `INV-09` | Lower/future version rejection, source/registry/row non-mutation, transactional store initialization and rollback | Implemented and documented as a fail-closed baseline; no released older payload schema exists and no speculative transform is present. |
| Citation node and resolution | `CitationNodeAttrs`, `collect_document_citations`, `resolve_citation`; TypeScript/Tiptap mirrors and visible insertion | `CITATION_NODE.md`, `PHASE46_WORKFLOWS.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CRITICAL_PATHS.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | None | `INV-04` | Rust validation/resolution, persisted-reopen integration, insertion interaction, and fail-closed rendering tests | Manual citation insertion is visible; final citation formatting and bibliography generation remain unavailable. |
| Bibliography consistency | `check_bibliography_consistency`, `BibliographyConsistencyReport` | `BIBLIOGRAPHY_CONSISTENCY.md` | `docs/wiki/Current-Limitations.md` | None | `INV-04` | Missing, orphaned, duplicate, ordering, and side-effect tests | Intentionally internal and not user-facing; no visible bibliography workflow exists. |
| Central network client | `NetworkClient::{new,get_metadata}`, `ConnectivityPolicy`, `NetworkService`, typed request errors | `NETWORK_CLIENT.md`, `OFFLINE_MODE.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-01`, `INV-03`, `INV-10` | Construction, timeout, rate-limit, response-bound, offline-denial, and scan tests | Implemented and documented; connectivity mode is visible, while metadata lookup remains internal. |
| Metadata providers | `lookup_crossref`, `lookup_semantic_scholar`, `lookup_unpaywall`; normalized `MetadataRecord` | `METADATA_LOOKUP.md`, `NETWORK_CLIENT.md` | `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-10` | Provider request and malformed-response tests | Intentionally internal and not user-facing; candidates are non-persistent. |
| External browser handoff | `open_in_system_browser`, `open_external_access`, `openExternalAccess`, shared offline policy | `EXTERNAL_BROWSER_HANDOFF.md`, `OFFLINE_MODE.md`, `COMMAND_BOUNDARY.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Baseline external-service policy | `INV-01`, `INV-03`, `INV-10` | URL validation, fixed-origin, offline denial, command, wrapper, and opener scans | Intentionally internal and not user-facing; no research control exists. |
| Offline session policy | `ConnectivityPolicy`, `get_connectivity_mode`, `set_connectivity_mode`, `useConnectivityMode`, `ConnectivityModeControl` | `OFFLINE_MODE.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | Baseline offline policy | `INV-10` | State, command, pre-dispatch denial, IPC, stale-read, failure, toggle, and authority tests | Implemented and documented; process-local only, with live Wiki publication synchronized through Phase 39 at `1bddd52`. |
| OS-native secret storage | `SecretStore`, `SecretId`, `SecretValue`, `NativeSecretBackend`, managed `secret_store` state | `SECRET_STORAGE.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Baseline Rust secret ownership | `INV-01`, `INV-03` | Identifier/value bounds, replacement, missing/delete, malformed value, native-error mapping, managed-state, and authority scans | Implemented but intentionally internal: no credential, provider, command, event, frontend state, or visible settings workflow exists; the visible limitation is published at `96e15c7`. |
| Local diagnostic snapshot | `DiagnosticSnapshot`, `get_diagnostic_snapshot`, `getDiagnosticSnapshot` | `AUDIT_DIAGNOSTICS.md`, `COMMAND_BOUNDARY.md`, `FRONTEND_COMMAND_CLIENT.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Current-Limitations.md` | Existing Rust and frontend ownership | `INV-01`, `INV-02`, `INV-03` | Exact schema/order, byte bound, redaction, closed error, IPC validation, and authority scans | Implemented but intentionally internal: no control, hook, export, persistence, upload, telemetry, or support-submission workflow exists; the visible limitation is published at `96e15c7`. |
| PDF intake candidate | `prepare_explicit_pdf`, `WatchedPdfIntake::{record_change,confirm_stable}`, `PendingPdfImport` | `PDF_IMPORT.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-08` | Explicit, containment, symlink, quiet-period, and stable-size tests | Intentionally internal and not user-facing; watcher and UI are absent. |
| Durable PDF import jobs | `PdfImportJobStore` lifecycle; `pdf_import_jobs`; claim token, checkpoint, retry, cancellation, recovery | `BACKGROUND_JOBS.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | None | `INV-05` | Promotion/claim races, stale ownership, recovery, cancellation, and migration tests | Intentionally internal and not user-facing; scheduler and processing are absent. |
| AI orchestration | `AiAnalysisRequest`, `assemble_model_request`, `prepare_ai_analysis`, `run_ai_analysis`; stream traits/events | `AI_ORCHESTRATION.md`, `CANCELLATION_BOUNDARY.md`, `CONFIGURATION.md` | `docs/wiki/Current-Limitations.md` | ADR-002 accepted | `INV-07`, `INV-14` | Context, provenance, stream, cancellation, failure tests, and decision guard | Intentionally internal and not user-facing; accepted ADR-002 keeps provider-backed orchestration outside v1.0.0. |
| Python helper process | `PythonHelperRunner`; protocol and helper versions; packaged `draft_helpers.worker.process_request` | `PYTHON_HELPERS.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md`, `PACKAGING.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | None | `INV-11` | Rust process/protocol tests, Python contract tests, supported-host runtime test, package probe, authority scans | Implemented and packaged for the visible five-check path; Python remains Rust-owned and receives no path, credential, provider, or network authority. |
| Text-analysis findings | `TextAnalysisInput`, `run_text_analysis`, `runTextAnalysis`, `TextAnalysisPanel`; five closed finding codes; supporting measurements and Python heuristics | `TEXT_ANALYSIS.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | ADR-002 accepted | `INV-15` | Rust validation, Python determinism, ordering, threshold, offset, limit, IPC, stale-generation, interaction, offline, packaged-runtime, authority, and capability-language guards | Implemented and visible as exactly five non-authoritative local heuristic checks; canonical Wiki publication and stable complete packaged interaction evidence remain pending. |
| Formatting review | `FormattingSnapshot`, `run_formatting_checks`, `run_formatting_review`, `runFormattingReview`, `useFormattingReview`, `FormattingReviewPanel` | `FORMATTING_CHECKS.md`, `FORMATTING_UX.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | None | `INV-16` | Domain, command, IPC validator, generation, target, interaction, keyboard, and accessible-label tests | Implemented and documented; live recovery guidance is synchronized through Phase 39 at `1bddd52`. |
| Editor font formatting | `text_format.rs`, `textFormatting.ts`, `TextFormattingMarks`, `EditorToolbar` | `DOCUMENT_ENVELOPE.md`, `WORKSPACE_UI.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md` | `docs/wiki/Workspace.md`, `docs/wiki/Current-Limitations.md` | None | `INV-03`, `INV-09`, `INV-UX-06` | Mirrored malformed-shape rejection, exact eleven-family allowlist, point-bound, mark-preservation, selection, keyboard, JSON, save/reopen, focus, and every-family DOCX mapping tests | Mechanically implemented for eleven canonical families and integer sizes 8 through 72; both UX-2 findings remain open pending corrected packaged validation. |
| DOCX export | `compile_docx`, `export_docx`, `export_document`, `useDocxExport`; strict model/package compiler and atomic write | `DOCX_EXPORT.md`, `PHASE46_WORKFLOWS.md`, `CONFIGURATION.md`, `ERROR_MESSAGES.md`, `CRITICAL_PATHS.md` | `docs/wiki/Workspace.md`, `docs/wiki/Troubleshooting.md`, `docs/wiki/Current-Limitations.md` | None | `INV-04`, `INV-09` | Parser, package, XML, font run-property, mixed-format, resource-limit, citation-rejection, atomic-export, source-preservation, IPC, and interaction tests | Implemented and visible through a Rust-owned async target dialog; PDF and unsupported DOCX content remain unavailable. |
| Error presentation | `errorPresentation.ts`; runtime/connectivity session state; formatting panel; citation node view | `ERROR_UX.md`, `ERROR_MESSAGES.md`, owning boundary guides | `docs/wiki/Troubleshooting.md`, `docs/wiki/Workspace.md` | None | `INV-02`, `INV-03`, `INV-10` | Exhaustive code/disposition, fallback, retained-state, announcement, focus, and control-semantics tests | Implemented and published through Phase 39 at `1bddd52`; typed but unwired failures intentionally have no speculative user copy. |
| Verification and repository tooling | `justfile`; `scripts/{bootstrap,build,format,verify,check-*}.sh`; `.github/workflows/verify.yml` | `TOOLCHAIN.md`, `CONFIGURATION.md`, this matrix | No user workflow | Baseline governance policy | `INV-12`, `INV-13` | Script syntax, scans, repository hygiene, and CI/local parity | Intentionally internal and not user-facing. |
| Packaging and application icons | `package:macos`; `package-macos.sh`; `check-packaging.sh`; active `app` target; five explicit icon paths | `PACKAGING.md`, `CONFIGURATION.md`, `TOOLCHAIN.md` | `docs/wiki/Current-Limitations.md` | None | `INV-13` for verification parity | Structured config checks plus clean unsigned Apple Silicon build, plist, architecture, executable, and embedded-icon validation | Implemented and documented for an unsigned macOS Apple Silicon `.app`; signing, notarization, installer images, publication, and other platforms remain release work. |
| Release-candidate hardening | `check-release-candidate.sh`; blocker and gate inventory | `RELEASE_CANDIDATE.md`, `PACKAGING.md`, `CONFIGURATION.md`, `TOOLCHAIN.md`, `REALIGNMENT.md` | Existing canonical limitations; no user behavior changed | ADR-001 for accepted PDF deferral | Existing invariants remain authoritative | Structured inventory counts, binding usability rule, live CSP/package/workflow evidence, pre-release version/tag state, generated-artifact scan, and full verifier | Phase 45 closes only `GATE-45`; six release blockers and three mandatory pre-49 gates remain open, so this is not final-candidate readiness. |
| v1 usability acceptance | Phase 46 visible workflows plus release enforcement in `check-docs.sh`, `check-invariants.sh`, and `check-release-candidate.sh` | `V1_USABILITY_ACCEPTANCE.md`, `PHASE46_WORKFLOWS.md`, `RELEASE_CANDIDATE.md`, `V1_USABILITY_EVIDENCE.md` | Workspace, recovery, and limitation guidance follows the implemented workflows | None; refines the accepted Phase 45 release rule | `INV-UX-01` through `INV-UX-06` | Phase 46 automated interaction evidence plus exact packaged artifacts and manual findings are recorded; a stable complete packaged workflow and later human evidence remain required | Implemented but not release-closed. The ledger tracks `UX-46-001` through `UX-46-021`; the specifically retested New, Save, and effective font-control findings are closed. `RC-01` through `RC-04` and `GATE-46` remain open. |
| Document interoperability and desktop workflows | Phase 47 and Phase 48 contract only; implementation remains absent | ADR-003, accepted `V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`, `RELEASE_CANDIDATE.md` | Existing limitations remain authoritative; no new workflow is published | ADR-003 Accepted | Existing `INV-03`, `INV-06`, `INV-09`, and `INV-UX-01` through `INV-UX-06`; `INV-UX-07` remains Proposed | Accepted-state documentation, invariant, source-absence, and release-gate checks | Implemented behavior absent: accepted policy and phase ownership do not constitute Phase 47 or Phase 48 evidence. |
| PDF export decision | No dependency, command, runtime path, control, or generated PDF | ADR-001 and `PDF_EXPORT_DECISION.md` | `docs/wiki/Current-Limitations.md` | ADR-001 accepted | Accepted PDF deferral guard | Absence scan and full verifier | Implemented and documented as an accepted deferral; PDF remains intentionally unavailable. |
| Public Rust API comments | Externally reachable modules, types, functions, methods, variants, and fields | Owning subsystem guides and this matrix | No user surface | None | None | `cargo rustdoc -- -D missing_docs` audit probe | Documented but enforcement missing: 457 granular lint findings remain, mostly variants, fields, accessors, and module exports. A focused source-documentation change is required before enabling the lint. |
| Live GitHub Wiki | Canonical pages under `docs/wiki/`; <https://github.com/progentic/draft/wiki> | `DOCUMENTATION.md`, this matrix | Home, Workspace, Troubleshooting, Current Limitations | None | None | Offline source checks, remote-tree/hash comparison, and rendered navigation review | Live commit `1bddd52` matches all merged sources through Phase 39. |

## ADR-003 Accepted Coverage

These identifiers cover the accepted ADR-003 contract surfaces. They do not
claim implementation, user documentation, or release evidence.

| Identifier | Accepted area | Owning contract surface |
| :--- | :--- | :--- |
| `ADR003-COV-INTEROP` | Document interoperability | ADR-003 format boundary and successor contract Phase 47. |
| `ADR003-COV-ROUNDTRIP` | Round-trip save policy and no-edit source preservation | ADR-003 round-trip ownership and successor save rules. |
| `ADR003-COV-FIDELITY` | Format fidelity and lossiness | Successor format-fidelity classes and required evidence. |
| `ADR003-COV-NATIVE-MENU` | Native macOS menu integration | ADR-003 native desktop workflow and successor Phase 48. |
| `ADR003-COV-DESKTOP-UI` | Desktop command grouping and window branding | Successor Phase 48 visual and interaction scope. |
| `ADR003-COV-USABILITY-PERF` | Product usability, documentation comprehension, and performance validation | Phase 49. |
| `ADR003-COV-REALIGNMENT` | Mandatory plain-language, maintainer-onboarding, terminology, cross-link, and fifth-phase drift realignment | Preserved Phase 50. |
| `ADR003-COV-GATE-REMAP` | Remapped security, final candidate, and release phases | Phases 51, 52, and 53 plus the accepted gate chain. |

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
