# DRAFT — Phasemap

**Purpose:** Break DRAFT development into reviewable phases from blank repository to `v1.0.0`.

This phasemap is an execution guide. It is not a changelog. It is not a substitute for `ARCHITECTURE.md`, `GOVERNANCE.md`, `INVARIANTS.md`, `CODING_STYLE.md`, or accepted ADRs.

Every phase should leave the repository in a reviewable state. Every fifth phase is reserved for documentation and drift realignment.

**Current execution checkpoint:** Phases 0 through 38 are complete. Phase 39 is
the next implementation phase.

The non-binding Phase 11 requirements remain in
`docs/drafts/DOCUMENT_ENVELOPE.md`. Implemented behavior is recorded in
`docs/maintainers/DOCUMENT_ENVELOPE.md`, Phase 12 registry behavior in
`docs/maintainers/DOCUMENT_REGISTRY.md`, and Phase 13/14 file behavior in
`docs/maintainers/DOCUMENT_SAVE_LOAD.md`. These implementation guides are not
accepted contracts under the governance lifecycle.

Phase 14 hardens the atomic replacement primitive with deterministic
interruption checkpoints, failed-replacement cleanup, serialized file
lifecycle operations, and disk/registry concurrency tests.

Phase 15 evidence is recorded in `docs/maintainers/REALIGNMENT.md`. Phase 16
requirements remain in the non-binding `docs/drafts/REFERENCE_RECORD.md`, and
implemented behavior is recorded in
`docs/maintainers/REFERENCE_RECORD.md`. Phase 16 does not authorize a reference
store, citation node, document-envelope extension, network lookup, or import
behavior.

Phase 17 requirements remain in the non-binding
`docs/drafts/REFERENCE_STORE.md`, and implemented SQLite behavior is recorded
in `docs/maintainers/REFERENCE_STORE.md`.

Phase 18 requirements remain in the non-binding
`docs/drafts/CITATION_NODE.md`, and implemented validation, resolution, IPC,
and Tiptap behavior is recorded in `docs/maintainers/CITATION_NODE.md`. The
reference store now initializes as Rust-managed state for exact citekey
resolution, but CRUD controls and full citation formatting remain absent.

Phase 19 requirements remain in the non-binding
`docs/drafts/BIBLIOGRAPHY_CONSISTENCY.md`, and the implemented pure Rust check
is recorded in `docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md`. It compares
validated citation nodes with an explicit candidate bibliography, reports
unique sorted missing, orphaned, and duplicate citekeys, and treats repeated
in-text citations as valid. It adds no persistence, IPC, formatting, or visible
workflow.

Phase 20 evidence is recorded in `docs/maintainers/REALIGNMENT.md`. Phase 21
network construction is recorded in `docs/maintainers/NETWORK_CLIENT.md`.
Phase 22 requirements and behavior are recorded in
`docs/drafts/METADATA_LOOKUP.md` and
`docs/maintainers/METADATA_LOOKUP.md`. Crossref, Semantic Scholar, and
Unpaywall DOI lookups now route through the centralized client and return
non-persistent normalized candidates with typed bounded failures.

Phase 23 requirements and implemented behavior are recorded in
`docs/drafts/EXTERNAL_BROWSER_HANDOFF.md` and
`docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`. Rust validates tagged
publisher, institutional, DOI, and Google Scholar targets and delegates one
launch to the default system browser. The frontend has no direct opener API or
capability, and no visible handoff control exists yet.

Phase 24 requirements and implemented behavior are recorded in
`docs/drafts/PDF_IMPORT.md` and `docs/maintainers/PDF_IMPORT.md`. Rust validates
explicit PDFs and gates watched PDFs on canonical root containment, a
one-second quiet interval, unchanged byte length, and stable signature read.
The resulting pending value starts no work and is not the Phase 26 persistent
job state machine.

Phase 25 evidence is recorded in `docs/maintainers/REALIGNMENT.md`. The audit
confirms that Phase 24 produces intake candidates only: no filesystem watcher,
background worker, queue, persistent job store, or visible import workflow
exists. Phase 26 requirements remain non-binding in
`docs/drafts/BACKGROUND_JOBS.md`; Phase 25 fixed them before implementation
began.

Phase 26 implemented behavior is recorded in
`docs/maintainers/BACKGROUND_JOBS.md`. A Phase 24 candidate is promoted
transactionally by `PdfImportId`; concurrent promotion returns one durable job,
and concurrent claim allows one opaque-token owner. Checkpoints, typed failures,
cancellation intent, attempts, and recovery persist without adding a worker,
scheduler, watcher, parser, network call, reference mutation, or UI workflow.

Phase 27 requirements and implemented behavior are recorded in
`docs/drafts/AI_ORCHESTRATION.md` and
`docs/maintainers/AI_ORCHESTRATION.md`. Rust now assembles bounded typed context,
preserves document and verified-evidence provenance, classifies every output as
generated analysis, and coordinates a cancel-safe adapter stream. No production
provider, credential, network call, persistence, Tauri start command, frontend
listener, document mutation, or spawned worker exists.

Phase 28 requirements and implemented behavior are recorded in
`docs/drafts/PYTHON_HELPERS.md` and
`docs/maintainers/PYTHON_HELPERS.md`. Rust now owns a fixed, isolated Python
entrypoint, versioned typed JSON, bounded standard streams, timeout,
cancellation, child reaping, and output validation. Phase 28's initial
`contract_probe` proves the boundary without adding text-analysis findings,
Tauri or frontend controls, persistence, document mutation, networking, or a
packaged Python runtime.

Phase 29 requirements and implemented behavior are recorded in
`docs/drafts/TEXT_ANALYSIS.md` and
`docs/maintainers/TEXT_ANALYSIS.md`. The Python helper now emits bounded closed
codes and UTF-8 byte ranges for five deterministic review heuristics. Rust
validates ordering, counts, ranges, and identity, then supplies fixed
explanations. No score, replacement, mutation, persistence, Tauri command,
event, frontend model, or visible issue workflow exists.

Phase 30 evidence is recorded in `docs/maintainers/REALIGNMENT.md`. The audit
distinguishes internal Rust analysis/helper lifecycles from product workers and
Tauri events, records the two-helper protocol truth, and changes no runtime
behavior. Phase 31 requirements remain non-binding in
`docs/drafts/FORMATTING_CHECKS.md`; the implementation stayed within the bounded
pure Rust review model defined there.

Phase 31 implemented behavior is recorded in
`docs/maintainers/FORMATTING_CHECKS.md`. A bounded immutable Rust snapshot now
supports three closed style identifiers, heading-outline checks, and
citation-style consistency findings. It adds no document parser, complete style
rules, mutation, persistence, filesystem access, export, Python, Tauri, frontend,
or visible formatting workflow. Phase 32 requirements remain non-binding in
`docs/drafts/DOCX_EXPORT.md`.

Phase 32 implemented behavior is recorded in
`docs/maintainers/DOCX_EXPORT.md`. Rust compiles a strict bounded Tiptap subset
into five deterministic DOCX package parts and atomically replaces only a
Rust-owned `.docx` target. Unsupported fields, nodes, marks, and citations fail
explicitly. No application state, Tauri command, native dialog, frontend,
persistence, network, Python, worker, PDF path, or visible export flow exists.
Phase 33 decision requirements remain non-binding in
`docs/drafts/PDF_EXPORT_DECISION.md`.

ADR-001 is accepted. Phase 33 defers native PDF generation while retaining a
mechanical absence guard, and adds no PDF runtime path or product workflow. PR
#1 merged under a documented one-time owner waiver of the remaining cooling
period; the standing governance rule is unchanged. Its post-merge verification
passed before Phase 34 began. Phase 34 implementation is recorded in
`docs/maintainers/FORMATTING_UX.md`; its original requirements remain
non-binding in `docs/drafts/FORMATTING_UX.md`.

Phase 34 exposes the bounded formatting checks in a transient workspace review
band. Findings are grouped into Structure and Citations, every action is
closed and Rust-owned, heading edits require an explicit user command, and
generation plus target checks reject stale results. No finding persistence,
complete style conformance, citation conversion, export control, or automatic
repair is added. Phase 35 is the next realignment boundary.

Phase 35 evidence is recorded in `docs/maintainers/REALIGNMENT.md`. The audit
reconciles the pure checker, typed review command/client, transient workspace
actions, strict internal DOCX foundation, accepted PDF deferral, user limits,
and live Wiki publication. It adds no product code or new capability. Phase 36
is the next implementation phase; its offline degraded behavior and exclusions
are bounded in the non-binding `docs/drafts/OFFLINE_MODE.md`.

Phase 36 implemented behavior is recorded in
`docs/maintainers/OFFLINE_MODE.md`. One Rust-owned session policy defaults to
online, typed get/set commands expose the effective mode, and metadata plus
system-browser boundaries fail before external work when offline. The header
control is transient and accessible; no persistence, connectivity probe,
retry queue, alternate transport, telemetry, proxy, credential, or secret
storage is added. Phase 37 is the next implementation phase.

Phase 37 implemented behavior is recorded in
`docs/maintainers/SECRET_STORAGE.md`. Tauri manages one lazy Rust store backed
by the native desktop credential manager. Bounded service API-key values are
zeroized and never cross IPC or another untrusted boundary. No credential,
provider, command, event, frontend state, prompt, fallback store, or network
integration is added. Phase 38 remains separate and is bounded in the
non-binding `docs/drafts/AUDIT_DIAGNOSTICS.md`.

Phase 38 implemented behavior is recorded in
`docs/maintainers/AUDIT_DIAGNOSTICS.md`. One typed command returns a strict,
versioned, deterministic local snapshot through a validating IPC wrapper. The
snapshot contains fixed support metadata only; it performs no probe, I/O,
collection, persistence, transmission, or secret-store operation. No component
or hook consumes the wrapper. Phase 39 remains separate and is bounded in the
non-binding `docs/drafts/ERROR_UX.md`.

---

## 1. Phase Rules

Each phase must have one clear purpose.

Each phase should include:

- a bounded implementation goal
- tests or verification appropriate to the change
- documentation updates when behavior, architecture, or user workflow changes
- local development checks
- GitHub Actions checks, when the phase introduces enforceable behavior

Do not combine unrelated work in one phase.

Do not add speculative abstractions before the product needs them.

Do not weaken an invariant to finish a phase. If an invariant is wrong, change it through governance.

---

## 2. Phase Map

| Phase | Target | Primary Outcome | Required Gate |
| :--- | :--- | :--- | :--- |
| 0 | Repository baseline | Root documentation and public project identity exist. | `v0.0.0` baseline committed. |
| 1 | Toolchain scaffold | Rust, Tauri 2, TypeScript, React, Tiptap, Python, and Bash surfaces are present. | Local bootstrap command succeeds. |
| 2 | Local verification | `just verify` or equivalent runs format, lint, and tests locally. | Local checks are documented. |
| 3 | GitHub Actions baseline | CI runs the core verification path. | GitHub Actions mirrors local critical checks. |
| 4 | App shell | Tauri app launches and renders a minimal DRAFT workspace. | Rust and frontend builds pass. |
| 5 | Documentation and drift realignment | Docs, scripts, workflows, and actual repo shape are reconciled. | No known doc/build drift remains. |
| 6 | Rust command boundary | Typed Tauri command pattern exists. | No generic command error pattern. |
| 7 | Frontend command client | TypeScript command wrapper exists for IPC calls. | UI does not call raw external network APIs. |
| 8 | Event stream pattern | Rust-to-frontend event pattern exists. | Event payloads are typed. |
| 9 | Cancellation pattern | Long-running user-initiated worker cancellation shape exists. | Cancellation has success, already-ended, and error tests. |
| 10 | Documentation and drift realignment | Bridge docs, invariants, and examples are reconciled. | IPC and event contracts match docs. |
| 11 | Document envelope | A versioned in-memory document envelope is defined. | Malformed envelope tests fail safely. |
| 12 | Document registry | Rust owns open-document handles. | Double-open returns focus or `AlreadyOpen`, not a second handle. |
| 13 | Save/load | Tiptap JSON save and reload path exists. | Round-trip tests pass. |
| 14 | Atomic save | Write-temp, fsync, rename save path exists. | Interrupted-save test never leaves a partial source document. |
| 15 | Documentation and drift realignment | Document model docs and tests match implementation. | Save/load claims match tested behavior. |
| 16 | Reference schema | Reference-record schema and source provenance fields exist. | Invalid reference records fail validation. |
| 17 | Local reference store | Rust-owned local reference persistence exists. | CRUD tests and migration stub pass. |
| 18 | Citation node contract | Tiptap citation node uses `schema_version`, `citekey`, and `render_style`. | Invalid citation attrs do not render silently. |
| 19 | Bibliography consistency | In-text citations and bibliography records can be checked. | Missing, orphaned, and duplicate citekey tests pass. |
| 20 | Documentation and drift realignment | Citation and reference docs are reconciled. | Source-of-truth model is consistent across docs and code. |
| 21 | Network client | Central Rust network client exists. | No ad-hoc HTTP clients outside the accepted network module. |
| 22 | Metadata lookup | Crossref, Semantic Scholar, and Unpaywall lookup path exists. | Rate limit, timeout, offline, and malformed-response paths are typed. |
| 23 | External browser handoff | Publisher, institutional, and Google Scholar access opens in system browser only. | No scraping, proxying, or credential capture path exists. |
| 24 | PDF intake | Explicit and watched-file candidates pass Rust-owned intake checks. | Stable-write confirmation test passes without starting work. |
| 25 | Documentation and drift realignment | Research workflow docs match implemented boundaries. | Candidate/job distinctions and absent workflow claims match code. |
| 26 | Background jobs | Rust-owned persistent job state machine exists. | Reopen recovery preserves the last checkpoint and valid transitions. |
| 27 | AI orchestration | Rust owns model calls, context assembly, streaming, and cancellation. | AI output is distinguishable from verified evidence. |
| 28 | Python helper contract | Python helper worker protocol exists. | Helpers receive typed input, emit typed output, and cannot own persistence or secrets. |
| 29 | Text-analysis checks | Grammar, clarity, tone, cohesion, or voice checks surface as reviewable findings. | Findings are explainable and non-destructive. |
| 30 | Documentation and drift realignment | Analysis and helper-worker docs match behavior. | Python boundary and AI boundary are documented and tested. |
| 31 | Formatting checks | APA, MLA, Chicago, headings, and structure checks begin. | Style issues are surfaced without silent document mutation. |
| 32 | Export foundation | DOCX export path exists. | Failed export does not corrupt source document. |
| 33 | PDF export decision | PDF export is either implemented or explicitly deferred by ADR. | Decision and user-facing limitation are documented. |
| 34 | Formatting UX | Formatting findings are grouped into actionable review surfaces. | User can accept, reject, or inspect changes. |
| 35 | Documentation and drift realignment | Formatting/export docs match supported behavior. | User docs explain export limits and style behavior. |
| 36 | Offline mode | Defined degraded behavior exists for offline use. | Network commands return typed offline errors. |
| 37 | Secrets boundary | OS-native credential storage is integrated. | Secrets never enter frontend-reachable storage. |
| 38 | Audit and diagnostics | Local diagnostics and audit reports exist for supportable failures. | Reports avoid secrets and user source content by default. |
| 39 | Error UX | Typed errors map to clear user-facing actions. | Unsupported states are not silently ignored. |
| 40 | Documentation and drift realignment | Security, offline, diagnostics, and error docs are reconciled. | Maintainer docs match actual failure behavior. |
| 41 | E2E critical paths | End-to-end tests cover core user flows. | Create/open/save/reopen/cite/export path is covered. |
| 42 | Packaging | Supported platform package path exists. | Package build is reproducible through documented commands. |
| 43 | Data migration | Migration strategy exists for document and reference data. | Old known schemas fail safely or migrate explicitly. |
| 44 | Release candidate hardening | Known high-risk bugs are fixed or documented as release blockers. | Release-candidate checklist passes. |
| 45 | Documentation and drift realignment | Release docs, public docs, maintainer docs, and changelog are reconciled. | No release-blocking documentation drift remains. |
| 46 | Accessibility pass | Core workspace accessibility is checked. | Keyboard navigation and readable labels exist for critical flows. |
| 47 | Performance pass | Large-document and reference-library behavior is measured. | Known performance limits are documented. |
| 48 | Security review | Trusted boundaries are reviewed against invariants. | Invariant tests and CI checks pass. |
| 49 | Final release candidate | Final pre-1.0 candidate is cut. | No P0 invariant violations remain. |
| 50 | v1.0.0 release realignment | Final docs, release notes, tags, and production checklist are aligned. | `v1.0.0` may be tagged only after all release gates pass. |

---

## 3. Documentation and Drift Realignment Phases

Phases 5, 10, 15, 20, 25, 30, 35, 40, 45, and 50 are realignment phases.

A realignment phase must verify that repository truth, product behavior, and documentation still match.

Minimum checklist:

- Root README still describes what the app does without making unsupported status claims.
- `ARCHITECTURE.md` matches implemented ownership boundaries.
- `GOVERNANCE.md` matches the actual change process.
- `INVARIANTS.md` lists only rules with local and GitHub Actions enforcement paths or clearly marks planned enforcement.
- `CODING_STYLE.md` still matches the languages and patterns in use.
- `DOCUMENTATION.md` still describes where documentation belongs.
- `AGENTS.md` still protects repository-root boundaries.
- `CHANGELOG.md` reflects only released notable changes.
- ADRs exist for accepted architectural changes.
- Contract docs exist for stable APIs, schemas, commands, workers, and data models.
- Wiki/user/maintainer docs match visible behavior.
- Local verification and GitHub Actions enforce the same critical boundaries.
- Tests exist for every accepted invariant that can be mechanically tested.

A realignment phase may produce documentation, tests, workflow fixes, lint rules, or ADRs. It should not introduce unrelated product features.

---

## 4. Phase Exit Criteria

A phase is complete only when all applicable criteria are true:

- The intended change is implemented or explicitly deferred.
- Local verification passes.
- GitHub Actions passes for the same critical checks.
- New behavior has tests.
- Negative paths are tested when the behavior protects data, credentials, files, external services, or user trust.
- User-facing behavior is documented.
- Maintainer-facing contracts are documented.
- Invariants are not weakened silently.
- `CHANGELOG.md` has the release entry only when preparing a release.

---

## 5. v1.0.0 Production Release Checklist

Before tagging `v1.0.0`, DRAFT must satisfy these checks:

### Document safety

- Create, open, edit, save, close, and reopen work reliably.
- Source document saves are atomic.
- Failed exports do not damage source documents.
- Unsupported document schemas fail safely.

### Research and citation safety

- Reference library is the citation metadata source of truth.
- Citation nodes are schema-versioned.
- Citation and bibliography consistency checks exist.
- External metadata lookup uses documented APIs only.
- Browser handoff does not become scraping, automation, proxying, or credential capture.

### Analysis and text-analysis safety

- AI output is visibly separate from verified source evidence.
- Long-running analysis work can be cancelled or has documented terminal behavior.
- Python helpers operate through typed Rust-owned contracts.
- Text-analysis findings are reviewable and non-destructive.

### Formatting and export safety

- Style checks are explainable.
- Document export paths are documented.
- Known export limits are visible to users.

### Security and operations

- Secrets are stored through OS-native credential storage.
- Frontend code does not call external network services directly.
- Rust owns persistence, filesystem access, network access, and helper orchestration.
- Bash is not part of product runtime behavior.
- Local and GitHub Actions checks enforce critical invariants.

### Documentation

- Public README is accurate and nontechnical.
- User docs explain core workflows.
- Maintainer docs explain architecture, contracts, tests, and release process.
- Known limitations are documented.
- Changelog entry for `v1.0.0` is prepared.
