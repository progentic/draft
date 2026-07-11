# DRAFT — Architecture

**Status:** Draft v0.4  
**Product name:** D.R.A.F.T — Document Research, Analysis, Formatting & Text-analysis  
**Scope:** System decomposition, process boundaries, data flow, runtime ownership, and build surfaces for the DRAFT desktop application.  
**Non-goals of this document:** API signatures, exact schemas, UI layout, and detailed test implementation. Those belong in downstream contract documents once this shape is accepted.

This document is the current system shape. It describes what owns what, what crosses each boundary, and what must not cross. Process rules live in `GOVERNANCE.md`. Binding invariants and their enforcement live in `INVARIANTS.md`.

---

## 1. Product Definition

DRAFT is a local-first desktop application for scholarly writing, document quality control, and research-supported drafting.

The name defines the major product surface:

- **Document Research:** retrieves peer-reviewed scholarly sources, manages citations, builds bibliographies, and interacts with a structured reference library or knowledge base.
- **Analysis:** supports semantic understanding, summarization, argument evaluation, fact-checking, voice consistency, and source reliability scoring.
- **Formatting:** enforces citation style, layout consistency, headings, structure, and document-ready output.
- **Text-analysis:** checks grammar, syntax, tone, clarity, cohesion, and voice consistency across the document.

These are product capabilities, not separate trust zones. Trust zones are defined by process ownership and data boundaries below.

---

## 2. System Shape

DRAFT is a Tauri 2 desktop application with four implementation surfaces:

- **Frontend WebView:** TypeScript, pure React, and Tiptap. Owns presentation, editor interaction, transient UI state, and user input capture.
- **Core:** Rust. Owns persistence, network I/O, secrets, background jobs, filesystem access, document state, document compilation, and worker orchestration.
- **Helper workers:** Python. Used for deterministic formatting and text-analysis helpers where Python libraries provide clear value. Python workers are invoked by Rust through typed input/output contracts and must not own secrets, persistence, or external network access by default.
- **Automation scripts:** Bash. Used for local development, formatting, verification, and GitHub Actions orchestration. Bash is not part of the product runtime path.

The dividing line is trust and durability, not convenience. Anything that must survive a crash, touch the filesystem, hold a credential, call an external service, coordinate a background worker, or write source documents lives behind Rust. Anything that only affects what the user sees right now may live in the frontend.

Borderline state is resolved with one question: if the WebView reloads, should this survive? If yes, Rust owns it. If no, the frontend may own it.

Relevant invariants: `INV-03`, `INV-10`, `INV-11`, and `INV-12` in `INVARIANTS.md`.

### 2.1 Current implementation checkpoint

The implemented application through Phase 42 is
deliberately smaller than the full system described in this architecture:

- Rust exposes typed runtime-status, worker-cancellation, document-open,
  document-save, citation-resolution, external-access, formatting-review, and
  connectivity-mode commands with command-specific request, response, and
  error types.
- TypeScript calls those commands only through typed wrappers under `src/ipc/`.
- Rust emits the typed finite `draft://runtime-status` event, and the frontend
  validates it before React displays connection state.
- Rust owns a process-local worker cancellation registry and cooperative token.
  The internal analysis and Python-helper boundaries use registrations while
  their caller awaits completion, but no product worker starts from a Tauri
  command, so no cancellation control is displayed.
- Rust owns the version 1 document envelope, UUID identity parsing, root-shape
  validation, typed failures, and Serde round trips. Typed open/save commands
  now carry that envelope, while Rust remains the validation authority.
- Rust owns a separate version 1 reference record with UUID identity,
  citekey, structured contributors, partial date, identifiers, provenance,
  resolution state, typed failures, and Serde round trips. Full records are
  stored locally but are never attached to a document.
- Rust owns a SQLite reference-store module with transactional CRUD,
  case-sensitive citekey uniqueness, schema initialization, migration dispatch,
  indexed/payload consistency checks, and typed corruption failures. The store
  initializes as managed Rust state for citation resolution; CRUD IPC and a
  visible library workflow remain absent.
- The version 1 document, citation, and reference payloads are the first
  released-schema baseline. Lower and future versions fail before mutation.
  Empty SQLite reference stores initialize transactionally from version 0;
  conflicting or unknown schemas fail without best-effort repair.
- Rust validates exact version 1 citation attrs inside document JSON before
  open/save mutation and resolves citekeys through the local store. The typed
  response contains only a disposable display marker, not reference metadata.
- Rust can compare those validated citation nodes with an explicit candidate
  bibliography of validated reference records. The pure check reports missing,
  orphaned, and duplicate citekeys without reading the complete shared library
  or mutating either input.
- Rust owns one HTTPS-only network client with controlled application identity,
  explicit timeouts, per-provider rate limiting and backoff, bounded responses,
  and typed transport failures. Crossref, Semantic Scholar, and Unpaywall
  modules use it for DOI metadata lookup and return non-persistent candidates.
- Rust owns one process-local online/offline policy shared by that client and
  the system-browser handoff. The workspace can request one closed session
  value, but Rust stores and enforces it before external work. New sessions
  default online; the value is not persisted and no connectivity probe runs.
- Rust owns one lazy OS-native service API-key store. Secret identifiers and
  values are bounded, values zeroize on drop, and native failures collapse to
  closed categories. No secret command, frontend state, provider integration,
  fallback file/database store, or startup credential probe exists.
- Rust can assemble one strict local diagnostic snapshot on explicit request.
  It reports compiled application and contract versions plus closed startup or
  non-probe subsystem states under a fixed byte limit. It does not inspect
  content, paths, logs, credentials, network state, files, databases, helpers,
  or background work, and no visible diagnostic workflow exists.
- The frontend maps only already-visible runtime, connectivity, formatting,
  and citation failures to bounded copy plus retryable, actionable, or
  terminal dispositions. Exhaustive mappings fail closed when typed variants
  change; unwired backend errors receive no speculative user message.
- Rust validates publisher and institutional HTTPS URLs, DOI resolver targets,
  and Google Scholar queries before handing one URL to the default system
  browser. The WebView has no direct opener API or capability.
- Rust can validate an explicitly approved PDF or confirm a watched PDF after
  one quiet second and an unchanged byte length. That creates a Rust-only
  candidate without starting processing.
- Rust owns a versioned SQLite PDF import-job store. Candidate promotion is
  identity-deduplicated, in-progress updates require a hashed opaque claim, and
  startup recovery invalidates stale claims while preserving checkpoints.
- Rust owns a provider-independent analysis seam with bounded request and
  context assembly, explicit document/evidence provenance, generated-analysis
  output tags, typed stream limits, and cooperative cancellation. Only
  deterministic in-memory test adapters exist; no model or external request is
  configured.
- Rust owns a versioned Python helper runner with a fixed canonical entrypoint,
  closed allowlist, isolated environment, bounded standard streams, timeout,
  cooperative cancellation, child reaping, and typed output validation. Its
  contract probe proves the process boundary, and its text-analysis operation
  returns only closed review codes and ranges.
- Rust validates five deterministic text-analysis finding codes and UTF-8 byte
  ranges returned by the helper, then supplies fixed categories, severities,
  titles, and explanations. Findings are immutable, non-persistent review
  prompts with no replacement or apply authority.
- Rust owns a pure formatting-check domain over one explicit bounded snapshot.
  It compares APA 7, MLA 9, or Chicago 17 author-date declarations and checks
  heading-level structure without parsing or mutating a document. These are
  consistency checks, not complete style-manual conformance.
- Rust exposes that domain through one typed formatting-review command. React
  builds one bounded snapshot from the current editor, validates the closed
  response, and shows transient Structure and Citation findings. A heading
  level changes only after explicit user input and only while the captured run
  generation and target node still match.
- Rust compiles a strict bounded Tiptap subset into deterministic in-memory DOCX
  packages and atomically replaces only Rust-owned `.docx` targets. Unsupported
  nodes, marks, fields, and citations fail explicitly; no source document or
  registry state is changed.
- Rust owns a process-local document registry. It stores each validated
  envelope behind one private live handle and returns `AlreadyOpen` for a
  duplicate or concurrent open request.
- Rust opens native file dialogs, validates selected files before registration,
  retains one exclusive source path per live document, and atomically writes
  only the explicit immutable snapshot submitted by the frontend. File
  lifecycle operations are serialized, failures before replacement preserve
  prior state, and a post-replacement durability failure advances the registry
  to the complete on-disk snapshot while returning a typed error.
- One crate-level Phase 41 test composes those existing document, reference,
  citation, and DOCX boundaries. It adds no runtime orchestration or visible
  workflow; citation-bearing DOCX remains an explicit typed rejection.
- The release toolchain owns one unsigned macOS Apple Silicon `.app` package
  path. A Bash entrypoint builds the configured app target and validates its
  plist identity, native executable, and tracked icon without signing or
  publishing it.
- React and Tiptap own only the transient writing surface and presentation
  state. The citation node preserves three attrs and shows explicit invalid,
  resolving, resolved, unavailable, or failed states. Reloading still discards
  the current document.
- The current workspace does not expose open/save controls yet. No close command,
  autosave, recovery, reference CRUD UI, citation insertion, rendered
  bibliography workflow, provider lookup or browser-handoff control, production
  model provider, analysis start command, frontend analysis listener, visible
  analysis workflow, visible text-analysis issue cards or controls, PDF import
  control or watcher, job scheduler or processing worker, complete
  style-manual formatting, citation conversion, finding persistence, citation
  renderer, visible DOCX export control, or PDF export path is implemented.

Sections below define the accepted target ownership and safety rules. They do
not imply that their product capabilities already exist.

---

## 3. Capability Ownership

### 3.1 Document Research

Document Research includes metadata lookup, source retrieval workflows, citation record management, bibliography generation, and reference-library interaction.

Ownership:

- Rust owns metadata resolution, local reference persistence, network calls, source provenance, and citation consistency checks.
- TypeScript/React displays search results, citation controls, source cards, and user choices.
- Python may assist with local document parsing or text extraction only when invoked by Rust through the helper-worker boundary.
- Bash is limited to development and CI checks for this area.

DRAFT may query documented APIs such as Crossref, Semantic Scholar, and Unpaywall. It must not scrape Google Scholar, publisher sites, institutional portals, or research databases.

### 3.2 Analysis

Analysis includes summarization, argument evaluation, fact-checking support, voice consistency, source reliability scoring, and semantic understanding.

Ownership:

- Rust owns AI orchestration, context-window assembly, model-provider calls, truncation policy, cancellation, and persistence of analysis state.
- TypeScript/React displays analysis results and captures user feedback.
- Python may run local deterministic analysis helpers, such as readability metrics, grammar-oriented checks, or structural document inspection, when those helpers are versioned and invoked through Rust.

Analysis output must remain distinguishable from source evidence. The application must not silently convert AI-generated claims into verified facts.

Phase 27 implements only the internal Rust orchestration seam. Typed context
keeps `UserDocument` and `VerifiedSourceEvidence` distinct, while every stream
update is `GeneratedAnalysis`. Provider integration, credentials, network
execution, persistence, Tauri start IPC, and UI remain future work.

Proposed ADR-002 is under review. It would keep that model orchestration seam
internal for v1.0.0 and make the separately bounded local deterministic
text-analysis helper the only production analysis path for the initial release.
The proposal adds no current command, provider, credential, external request,
model runtime, or visible workflow and is not accepted architecture until its
governed PR merges.

The proposal separates deterministic measurements, deterministic heuristics,
and model-backed interpretation. Measurements may support exactly the five
existing text-review checks internally. Those five heuristics are the complete
proposed Phase 46 user-visible analysis scope and remain non-authoritative
signals. Semantic, generative, or inferential model-backed interpretation is
outside v1.0.0.

### 3.3 Formatting

Formatting includes APA, MLA, Chicago, heading structure, layout consistency, document-ready export, and bibliography generation.

Ownership:

- Rust owns document compilation, save/export paths, citation-engine integration, and source-document safety.
- Python may run formatting or lint-style document checks if called through a typed helper contract.
- Bash may run formatter commands in local development and GitHub Actions.
- TypeScript/React displays formatting issues and user choices.

Formatting automation must not mutate the source document without a Rust-owned transaction or explicit user action.

Phase 31 implements a pure review boundary. Its immutable snapshot declares
one closed style, ordered heading levels, and ordered citation styles. Findings
identify a non-level-one first heading, skipped heading level, or citation-style
mismatch by index. Phase 34 adds one typed command and transient review surface
around that checker. React extracts only the bounded current snapshot, and a
generation plus exact-node guard rejects stale inspect or apply actions.
Heading changes require explicit user input through the normal Tiptap
transaction; citation findings remain inspect-only. No style rendering,
persistence, automatic repair, filesystem, network, Python, worker, or export
authority is present.

Phase 32 adds a separate Rust-only DOCX foundation. It validates a strict
paragraph, heading, text, hard-break, and inline-mark subset; builds fixed
escaped XML parts in a deterministic stored ZIP; and reuses the atomic writer
for `.docx` targets. It adds no citation rendering, application state, Tauri
command, frontend control, PDF path, or source mutation.

ADR-001 defers native PDF generation until DRAFT has accepted
policies for fonts, pagination and layout, accessibility, cross-platform
rendering, dependencies and licensing, bounded resources, parser-based output
verification, deterministic failures, and source preservation. Under that
accepted decision, no PDF dependency, renderer, runtime path, command, process,
frontend control, or capability claim is authorized.

### 3.4 Text-analysis

Text-analysis includes grammar, syntax, tone, clarity, cohesion, and voice validation.

Ownership:

- Rust owns orchestration, durable issue records, and document mutation boundaries.
- Python may run local linguistic checks as helper workers.
- TypeScript/React displays issues, explanations, and proposed edits.

Text-analysis may suggest changes. Applying changes to the document must go through the same document mutation path as any other edit.

Phase 29 implements five narrow local review heuristics: adjacent repeated
words, sentences over 30 lexical words, all-capital words of at least five
letters, repeated substantial sentence openers, and mixed singular/plural
first-person pronouns. Python returns only closed codes and UTF-8 byte ranges.
Rust validates those ranges and owns all explanatory wording. No score,
replacement, persistence, command, event, frontend, or edit path exists.

---

## 4. Process Responsibilities

### 4.1 Frontend WebView: TypeScript + Pure React + Tiptap

The frontend:

- Renders the editor, sidebars, document controls, settings, and interactive UI.
- Owns the Tiptap editor instance and its frontend schema.
- Holds transient UI state only: current selection, active panel, draft search text, loading indicators, and display errors.
- Never calls external services directly.
- Never watches the filesystem directly.
- Never stores secrets.
- Sends bounded requests to Rust through Tauri commands.
- Subscribes to Rust-emitted events for streamed output and background job status.

Relevant invariants: `INV-03`, `INV-06`, and `INV-10` in `INVARIANTS.md`.

### 4.2 Core: Rust

Rust owns every operation that must be durable, trusted, or externally visible:

- **Persistence:** SQLite database for user preferences, custom AI prompts, reference records, job state, document metadata, and durable issue records.
- **Metadata resolution:** Queries documented APIs for citation metadata and open-access links.
- **Full-text acquisition boundary:** Opens external pages in the user's browser when needed. It does not authenticate, proxy, scrape, or automate publisher or institutional login sessions.
- **AI orchestration:** Owns context-window assembly, truncation policy, LLM API calls, token streaming, and cancellation.
- **Citation engine:** Formats resolved metadata into supported citation styles and checks consistency between in-text citation markers and bibliography entries.
- **Formatting orchestration:** Owns document-ready export and any source-document mutation boundary.
- **Text-analysis orchestration:** Owns durable issue records and the application of accepted edits.
- **Python worker orchestration:** Invokes allowlisted Python helpers through typed contracts.
- **Retraction checking:** Runs as a persistent background job modeled as a state machine.
- **File watching:** Owns watched-folder import and stable-write confirmation for PDF intake.
- **Network client:** Owns centralized outbound request handling, rate limiting, backoff, User-Agent policy, and external-service routing.
- **Secrets:** Stores LLM and service API keys only through the OS-native credential manager: Keychain, Credential Manager, or Secret Service.
- **Filesystem:** Owns native save/load dialogs, atomic document saves, `.docx` compilation, and `.pdf` compilation.

Relevant invariants: `INV-01`, `INV-02`, `INV-05`, `INV-07`, `INV-08`, `INV-09`, `INV-10`, and `INV-11` in `INVARIANTS.md`.

### 4.3 Python Helper Workers

Python is allowed for formatting and text-analysis helpers, not as an independent application authority.

A Python helper may:

- Receive a typed, bounded payload from Rust.
- Return typed JSON or another documented output format to Rust.
- Run deterministic formatting, text-analysis, parsing, or document-quality checks.
- Use pinned dependencies declared in the repository.

A Python helper must not:

- Read arbitrary user files outside the input path or payload approved by Rust.
- Write directly to the source document.
- Store credentials.
- Call external network services unless an ADR explicitly approves the helper and the call still routes through the Rust network policy.
- Shell out to arbitrary commands.
- Become the source of truth for document state.

Phase 28 implements this boundary with protocol version 1 and only the
`contract_probe` allowlist entry. Rust canonicalizes a trusted Python executable
and fixed package entrypoint, invokes it without a shell under an empty inherited
environment, and owns bounded stdin/stdout/stderr, a five-second timeout,
cooperative cancellation, kill, reap, exit interpretation, and exact response
validation. No application state, command, event, frontend, persistence, or
product text-analysis flow invokes the runner yet.

Relevant invariant: `INV-11` in `INVARIANTS.md`.

### 4.4 Bash Automation

Bash is allowed for local development and GitHub Actions orchestration.

Bash may:

- Run formatters.
- Run lint checks.
- Run tests.
- Compose verification scripts.
- Provide local setup commands.

Bash must not be used as a product runtime mechanism for document processing, credential handling, network access, or user-supplied path execution.

Relevant invariant: `INV-12` in `INVARIANTS.md`.

---

## 5. The Bridge: Tauri IPC and Worker Calls

DRAFT uses Tauri IPC for frontend-to-Rust communication and Rust-emitted events for ongoing work. It also uses Rust-controlled helper-worker calls for Python helpers.

These mechanisms must not be collapsed into one abstraction.

### 5.1 Commands: Request/Response

Commands are used when the frontend initiates an action and expects one bounded answer.

Examples:

- `search_reference`
- `insert_citation`
- `import_pdf`
- `compile_document`
- `save_document`
- `start_ai_generation`
- `cancel_ai_generation`
- `run_text_analysis`
- `run_formatting_review`

Each command has:

- A typed input.
- A typed `Result<T, E>` output.
- A command-specific error enum.
- No hidden side effects the frontend cannot observe in the response or through a documented event stream.

Generic strings, `anyhow::Error`, and untyped error blobs are not acceptable command outputs.

Relevant invariant: `INV-02` in `INVARIANTS.md`.

### 5.2 Events: Rust-Initiated Updates

Events are used when Rust needs to report ongoing work after the initial command returns.

Examples:

- AI stream chunks.
- AI stream terminal state.
- Formatting-check progress.
- Text-analysis progress.
- Retraction-check state transitions.
- Background import progress.
- Metadata-resolution state transitions.

Rule of thumb: if the frontend is waiting for one answer to one question, use a command. If Rust is narrating ongoing work, use an event.

### 5.3 Long-Running Worker Cancellation

The frontend cannot stop Rust-owned work by ignoring events. If a user-initiated long-running worker emits progress events, the user must have a command that cancels or aborts that worker unless the worker is documented as non-cancelable and idempotent.

For AI generation:

- `start_ai_generation(...)` returns a `stream_id` synchronously.
- `cancel_ai_generation(stream_id)` aborts the underlying LLM request and tears down the stream worker.
- Cancel is idempotent. Canceling an already-finished or already-canceled stream is not an error.
- Cancellation emits a final terminal event, such as `AiStream::Cancelled`, so UI state resolves cleanly.

For Python helpers:

- Rust starts the helper process.
- Rust owns timeout, cancellation, stdin/stdout handling, and exit-code interpretation.
- Rust must kill or terminate the helper on user cancellation, timeout, or application shutdown.
- The helper result is not trusted until Rust validates the output shape.

Relevant invariants: `INV-07` and `INV-11` in `INVARIANTS.md`.

---

## 6. Document Model: Single Live Handle per Document

DRAFT allows multiple documents open at once across tabs or windows. A given document can only be open in one live editing view at a time. Opening an already-open document focuses the existing view rather than creating a second live copy.

This avoids a multi-actor conflict-resolution problem instead of pretending it has been solved. CRDTs, Yjs, and similar tools are for concurrent edits by different actors on the same document. That is not a current product goal. DRAFT's current rule is one document, one Tiptap instance, one Rust-side document handle.

If real-time multi-user collaboration becomes a goal, this section and the downstream synchronization model must be revisited by ADR before implementation starts.

Each open document gets its own independent Tiptap editor instance and its own Rust-side document handle. Documents do not share editor state. They may share the reference library, which is a separate cross-document resource by design.

Phase 12 implements the Rust-side registry and chooses the typed `AlreadyOpen`
result until a frontend view identity exists. The registry owns validated
in-memory envelopes and returns the envelope when its handle closes. Phase 14
adds a separate registry-owned lock that serializes persistence open/save
coordination around disk and handle updates. The registry module does not call
filesystem or Tauri APIs itself.

Relevant invariant: `INV-06` in `INVARIANTS.md`.

---

## 7. Reference Library: Bounded, Not a Literature Cache

The local database is not a cache of global scholarship. It is bounded to what the user has searched, imported, manually created, or otherwise chosen to keep.

Metadata search hits documented services directly per query. Nothing is pre-populated. A reference record becomes local only after user action or an explicit application workflow stores it.

The detailed reference-record schema is deferred to a downstream data-model contract. That contract must define fields, provenance, resolution state, merge behavior, manual-edit behavior, and source-reliability scoring inputs.

Phase 16 implements the non-binding in-memory schema checkpoint in
`maintainers/REFERENCE_RECORD.md`. Rust validates one record and preserves
manual-override provenance as data. It does not persist records, enforce
library-wide citekey uniqueness, resolve metadata, merge sources, compute
reliability, expose IPC, or alter the document envelope.

Phase 17 implements the Rust-owned local store in
`maintainers/REFERENCE_STORE.md`. The store persists validated record JSON in
SQLite, indexes UUID/citekey/schema fields, serializes writes with immediate
transactions, and validates stored payloads on every read. Production path
policy is `<app_data_dir>/references.sqlite3`; Rust must resolve the app-data
directory before opening the production store.

Phase 18 initializes that store as managed Tauri state solely for the typed
`resolve_citation` path documented in `maintainers/CITATION_NODE.md`. The
frontend receives no full record data, and no reference CRUD command exists.

Phase 19 adds the pure comparison documented in
`maintainers/BIBLIOGRAPHY_CONSISTENCY.md`. A caller supplies the validated
records selected for one candidate bibliography. The checker does not treat the
complete cross-document reference library as one document's bibliography.

---

## 8. Citation Node Contract

The Tiptap citation node is not a free-form block. It is a rendering surface for a Rust-owned reference record.

The reference library is the citation metadata source of truth. Citation nodes do not embed full CSL JSON. They carry only enough information to identify and render the citation against the Rust-owned reference library.

Minimum citation node attrs:

```json
{
  "schema_version": 1,
  "citekey": "string",
  "render_style": "apa7"
}
```

Rules:

- `schema_version` is required.
- `citekey` must resolve to a Rust-owned reference record before render or export.
- `render_style` must be one of the supported style identifiers.
- A citation node whose attrs do not match the current schema is a migration case, not a silent render case.
- Invalid citation attrs must fail closed with a validation or migration error.
- Rendered citation text may be cached for UI performance only if it is explicitly marked as disposable display cache. It is not source data.

Phase 18 implements this boundary with exact Rust attrs validation, nested
document scanning, local-store resolution, typed IPC, and explicit Tiptap
display states. The current `[@citekey]` marker confirms resolution for the
editor only; it is not complete APA output. Phase 19 reuses the Rust scanner to
check citekey consistency against an explicit candidate bibliography. Citation
insertion, bibliography generation and formatting, and export remain later
work.

Relevant invariant: `INV-04` in `INVARIANTS.md`.

---

## 9. Full-Text Acquisition

DRAFT uses two separate pipelines that must not share a contract.

### 9.1 Metadata Search

Rust queries documented metadata and open-access APIs. This is the automated, programmatic metadata path.

### 9.2 Full-Text PDF Acquisition

When no open-access link exists, DRAFT does not authenticate or scrape on the user's behalf.

The allowed flow is:

1. DRAFT opens the relevant publisher, database, DOI, or Google Scholar page in the user's default system browser.
2. The user authenticates with institutional or research-database credentials outside DRAFT.
3. The user downloads the PDF outside DRAFT.
4. The user imports the PDF through explicit import or watched folder.
5. Rust resumes metadata resolution against the imported file.

Opening a URL in the user's system browser is not automated access. DRAFT must not script, scrape, intercept, proxy, or automate that browser session.

This keeps institutional and publisher credentials permanently outside DRAFT's trust boundary.

Phase 23 implements the typed handoff documented in
`maintainers/EXTERNAL_BROWSER_HANDOFF.md`. Rust accepts one tagged publisher,
institutional, DOI, or Google Scholar target, constructs or validates a bounded
HTTPS URL, and calls only the default OS browser. No opener guest plugin,
frontend capability, embedded browser, session inspection, or persistence is
added.

Relevant invariant: `INV-01` in `INVARIANTS.md`.

---

## 10. Background Job Pattern

Phase 26 implements the first bounded form of this pattern for PDF import jobs.
No scheduler, queue consumer, background product worker, progress event, or
frontend job surface exists.

Retraction checking, PDF metadata resolution, formatting checks, and long text-analysis runs are async, external-service-dependent or CPU-bound, and must survive interruption without silently losing state.

Minimum state shape:

```text
Pending -> InProgress -> Resolved | Failed(reason) | NeedsManualInput | Cancelled
```

State must be persisted per record in SQLite, not held only in memory. A crash
mid-check must not drop the job silently. Later worker integrations must emit
state changes as events so the frontend can reflect status without polling;
Phase 26 emits no job events because it starts no work.

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

The Phase 26 PDF job store also persists immutable candidate details, a claim
timestamp, and only the SHA-256 digest of an opaque UUID v4 claim token.
Promotion is atomic and idempotent by `PdfImportId`; separately validated
candidates are not merged by path or size. Every in-progress mutation checks
the current digest and expected state in its transactional update. Recovery
clears the prior claim and returns interrupted work to `Pending`, or to
`Cancelled` when durable cancellation intent was already set.

### 10.1 Watched-Folder Import: Stable-Write Requirement

Any filesystem watching added later must be Rust-owned. The frontend never
watches the filesystem directly, and no watcher exists at the Phase 26
checkpoint.

A watched file may enter the import pipeline only after Rust confirms the file is stable. A file appearing on disk is not enough.

Required confirmation:

- Debounce window with no further modification events.
- Stable-size check, file-lock check, or equivalent platform-safe confirmation.
- Only then does the file enter the metadata-resolution state machine.

Phase 24 implements the intake gate documented in
`maintainers/PDF_IMPORT.md`. `WatchedPdfIntake` records Rust-supplied change
observations, requires one quiet second, confirms the byte length is unchanged,
then validates the PDF signature while rechecking file length. Paths are
canonical and confined to one watched root. The resulting
`PendingPdfImport` is a process-local candidate only; no work begins until a
durable job owns it. Phase 26 can perform that promotion before work, but does
not process the job. This size-based gate cannot detect an in-place content
change that keeps the same byte length when no filesystem event is delivered;
any delivered event resets the quiet period even when size is unchanged.

Relevant invariants: `INV-05` and `INV-08` in `INVARIANTS.md`.

---

## 11. Save Lifecycle, Document Mutation, and Export

The document's working representation is Tiptap JSON wrapped in an
application-level envelope. Phase 11 implements the minimum version 1 envelope
as a Rust domain type with `schema_version`, `document_id`, `title`, and
`document` fields. The exact implemented checkpoint is documented in
`maintainers/DOCUMENT_ENVELOPE.md`; it is not yet an accepted contract.

Phase 13 connects the envelope to typed Rust open/save commands. Rust selects
paths through native dialogs, validates loaded bytes before registry entry,
and receives an explicit immutable frontend snapshot for save. TypeScript only
mirrors the envelope for response safety and command requests.

Phase 14 hardens the atomic writer. Rust creates a same-directory temporary
file, writes and synchronizes its bytes, atomically replaces the target, then
synchronizes the parent directory on Unix. Deterministic checkpoints prove
that failures before replacement preserve the prior complete source and clean
up temporary files. One Rust lifecycle lock serializes open/save operations so
disk replacement and registry mutation cannot be reordered by concurrent
saves.

The saved source document contains:

- Tiptap document JSON.
- Citation node references into the reference library.
- Application metadata needed to reopen and validate the document.

The saved source document does not embed full citation metadata as the citation source of truth.

Save rules:

- On save, the frontend serializes current Tiptap state and passes it to Rust through `save_document` as an immutable payload.
- Rust does not read live state out of the frontend. It receives a snapshot.
- Rust writes using atomic write-then-rename: write temporary file, fsync, then rename over the target path.
- The on-disk file is always either the prior complete version or the new complete version, never a partial write.

Document mutation rules:

- Formatting and text-analysis suggestions do not mutate the source document automatically.
- Applying a suggestion must go through a Rust-owned document mutation path or an explicit frontend edit captured by Tiptap and saved through Rust.
- Python helpers may propose edits, but Rust validates and applies accepted edits.

Export rules:

- `.docx` and `.pdf` compilation are explicit export operations.
- Export failure must not corrupt the source document.
- Export output is not the authoritative source format.

Phase 32 implements the internal `.docx` operation for a strict fail-closed
subset. Compilation completes in memory before atomic target replacement.
Source DRAFT bytes and live registry state are outside the export mutation
surface. PDF export remains unimplemented. Accepted ADR-001 keeps it deferred
until the prerequisite rendering policies and verification contract are
accepted through governance.

Relevant invariants: `INV-04`, `INV-09`, and `INV-11` in `INVARIANTS.md`.

---

## 12. Error Handling

Every Tauri command has its own error enum. "Something went wrong" is not an acceptable terminal state for any command.

The frontend needs typed errors so it can distinguish at least:

- Network timeout.
- Not found.
- Rate limited.
- Authentication required for external browser workflow.
- Malformed response.
- Offline.
- Validation failure.
- Migration required.
- Worker timeout.
- Worker cancelled.
- Worker output invalid.

Exact enum definitions belong in per-command contract docs. This section locks the requirement before implementation starts.

Relevant invariant: `INV-02` in `INVARIANTS.md`.

### 12.1 Offline Behavior

DRAFT runs in a defined degraded state when the user selects offline mode or a
network connection attempt returns the typed offline failure.

- Rust owns the explicit session policy and denies new metadata requests and
  system-browser handoffs before transport or opener work while it is offline.
- Commands that require external access return a distinct offline variant rather than a generic timeout.
- Local citation formatting, document editing, local reference-library access, local formatting checks, local text-analysis helpers, and saving remain functional offline.
- Reconnection is not actively polled for. Explicit mode remains offline until
  the user returns online or restarts DRAFT; transport failures do not silently
  change the selected mode.

---

## 13. Network Client and Rate Limiting

External API calls do not each open independent HTTP clients. Rust owns a centralized network client responsible for:

- Per-service request queuing.
- Exponential backoff on rate-limit responses.
- Offline detection.
- Consistent User-Agent identification where applicable.
- Logging policy that avoids secrets and sensitive document text.

This belongs to the network client, not to individual features. Feature code must not bypass it by creating ad hoc HTTP clients.

Phases 21 and 22 implement the boundary documented in
`maintainers/NETWORK_CLIENT.md` and `maintainers/METADATA_LOOKUP.md`. Rust builds
one HTTPS-only client during startup and owns request execution, per-provider
rate limiting, capped backoff, bounded response reads, and typed transport
classification. Provider modules build validated DOI requests and normalize
candidate metadata without persistence or frontend authority.

Relevant invariant: `INV-10` in `INVARIANTS.md`.

---

## 14. Build and Verification Surfaces

DRAFT has two supported build and verification surfaces:

- **Local development:** developer workstation.
- **GitHub Actions:** repository CI.

The local and CI paths run the same `scripts/verify.sh` checks. The `justfile`
is the stable developer interface; underlying commands may call Cargo, npm,
Python tools, Bash scripts, or Tauri commands. The Apple Silicon package build
is intentionally local because the hosted verifier runs on Ubuntu; hosted CI
still enforces its portable configuration contract.

Required local commands:

```bash
just format
just verify
just check-invariants
just build
npm run package:macos
```

Required GitHub Actions workflow:

```text
.github/workflows/verify.yml
```

Expected tool coverage:

- Rust: `cargo fmt`, `cargo clippy`, `cargo test`.
- TypeScript/React/Tiptap: formatter, linter, typecheck, unit tests.
- Tauri 2: portable package-contract verification in CI and an unsigned native
  `.app` build on macOS Apple Silicon.
- Python: formatter, linter, dependency check, unit tests for helper workers.
- Bash: `shellcheck`, `shfmt`, and script smoke tests.
- Cross-boundary checks: invariant scripts from `INVARIANTS.md`.

Relevant invariant: `INV-13` in `INVARIANTS.md`.

---

## 15. Open Questions

These must be resolved before granular contract docs become binding:

- Reference-record schema: fields, provenance, resolution states, reliability scoring inputs, and source merge behavior.
- Document-envelope evolution: fields and migrations beyond the implemented
  version 1 minimum.
- AI context-window assembly policy: token budget, truncation strategy, source prioritization, and user-visible disclosure.
- Formatting contract beyond the Phase 31 consistency checks: exact APA, MLA,
  Chicago, heading, rendering, and export rules.
- Text-analysis contract: grammar, syntax, tone, clarity, cohesion, and voice-validation issue model.
- Python helper contract: allowed helpers, package management, pinned dependencies, timeout defaults, and output schemas.
- Network client interface: queue structure, backoff parameters, retry policy, and per-service limits.
- Watched-folder debounce window and stable-size threshold.
