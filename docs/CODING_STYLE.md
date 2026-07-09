# DRAFT — Coding Style

**Status:** Draft v0.1  
**Applies to:** Rust, TypeScript, React, Tiptap, Tauri 2, Python helper workers, Bash automation, tests, and repository scripts.

---

## 1. Purpose

This document defines how code is written in the DRAFT repository.

The goal is not aesthetic consistency. The goal is code that is readable, reviewable, testable, and safe to operate on user documents, citations, source metadata, local files, and AI-assisted writing workflows.

DRAFT is a document research, analysis, formatting, and text-analysis application. It handles scholarly sources, citation records, source documents, user writing, local files, external APIs, and model output. Code should be direct, small, explicit, and boring.

Boring code is safer code. A reviewer should be able to see what a function reads, what it writes, what it calls, what it returns, and what boundary it crosses.

---

## 2. Core Rules

### 2.1 One task, one surface

Each change should touch the smallest reasonable surface area.

Do not mix unrelated work. Do not combine a feature, a refactor, a schema change, and a formatting sweep in one pull request.

Bad:

```text
Add citation search, rewrite document save logic, rename Rust modules, change sidebar routing, and reformat Python helpers.
```

Good:

```text
Add Crossref metadata search command.
```

Good:

```text
Extract citation node validation into citation_schema.rs.
```

### 2.2 Respect the trust boundary

The frontend shows state. Rust owns trusted work.

Rust owns persistence, network access, secrets, file watching, document saves, document compilation, job orchestration, AI orchestration, and Python worker invocation.

TypeScript/React owns presentation, editor interaction, transient UI state, and user input capture.

Python helpers run bounded formatting and text-analysis work only when Rust invokes them through typed contracts.

Bash runs local development and GitHub Actions orchestration only. Bash is not a product runtime surface.

Bad:

```tsx
const response = await fetch("https://api.crossref.org/works?query=leadership");
```

Good:

```tsx
const response = await searchReferences({ query: "leadership" });
```

Bad:

```python
requests.get("https://api.crossref.org/works", params={"query": query})
```

Good:

```python
# Python receives already-bounded text from Rust and returns local analysis only.
result = analyze_clarity(input_text)
```

### 2.3 Top-down organization

Put the public entry point first. Put helper functions below it.

A reader should understand the operation before reading implementation details.

Good Rust shape:

```rust
pub async fn save_document(
    state: State<'_, AppState>,
    request: SaveDocumentRequest,
) -> Result<SaveDocumentResponse, SaveDocumentError> {
    let document = validate_save_request(request)?;
    state.documents.save_atomic(document).await?;
    Ok(SaveDocumentResponse::saved())
}

fn validate_save_request(
    request: SaveDocumentRequest,
) -> Result<ValidatedDocumentSave, SaveDocumentError> {
    // helper details below public operation
}
```

Good TypeScript shape:

```tsx
export function CitationSearchPanel(props: CitationSearchPanelProps) {
  const [query, setQuery] = useState("");
  const search = useCitationSearch();

  return <CitationSearchView query={query} onQueryChange={setQuery} search={search} />;
}

function CitationSearchView(props: CitationSearchViewProps) {
  // rendering details below the public component
}
```

### 2.4 Avoid deep nesting

Return early. Make failure states visible.

Bad:

```rust
fn import_pdf(path: &Path) -> Result<ImportedPdf, ImportPdfError> {
    if path.exists() {
        if is_pdf(path) {
            if is_stable(path)? {
                return parse_pdf(path);
            }
        }
    }

    Err(ImportPdfError::InvalidInput)
}
```

Good:

```rust
fn import_pdf(path: &Path) -> Result<ImportedPdf, ImportPdfError> {
    if !path.exists() {
        return Err(ImportPdfError::NotFound);
    }

    if !is_pdf(path) {
        return Err(ImportPdfError::UnsupportedFileType);
    }

    if !is_stable(path)? {
        return Err(ImportPdfError::FileStillWriting);
    }

    parse_pdf(path)
}
```

Bad:

```tsx
if (document) {
  if (citation) {
    if (citation.citekey) {
      insertCitation(citation);
    }
  }
}
```

Good:

```tsx
if (!document) {
  return;
}

if (!citation?.citekey) {
  return;
}

insertCitation(citation);
```

### 2.5 Keep functions small

A function should do one clear operation.

Split parsing, validation, authorization, orchestration, execution, persistence, and rendering when they are distinct concepts.

Bad:

```rust
pub async fn run_analysis_and_save_and_export_and_notify(...) -> Result<(), Error> {
    // validates input
    // builds AI context
    // calls model provider
    // runs Python grammar helper
    // mutates document
    // saves file
    // exports docx
    // emits events
}
```

Good:

```rust
pub async fn run_text_analysis(
    request: RunTextAnalysisRequest,
) -> Result<RunTextAnalysisResponse, RunTextAnalysisError> {
    let job = create_analysis_job(request)?;
    let findings = run_local_text_checks(&job).await?;
    persist_analysis_findings(&job, findings).await?;
    Ok(RunTextAnalysisResponse::from_job(job))
}
```

### 2.6 Use human-readable names

Use names that describe the thing.

Good:

```rust
document_id
CitationRecord
reference_library
formatting_findings
```

Good:

```ts
citationSearchQuery
selectedReference
analysisIssueCount
```

Bad:

```rust
d
cr
lib2
x_result
```

Bad:

```ts
thing
stuff
mgr
payload2
```

Allowed short names are project-wide and obvious: `db`, `ctx`, `cfg`, `id`, `tx`, `rx`, `ui`.

### 2.7 No boolean flag arguments

Boolean flags hide behavior at the call site.

Bad:

```rust
run_formatting_check(document_id, true).await?;
```

Good:

```rust
run_formatting_check(document_id, FormattingMode::ApplyFixes).await?;
```

Bad:

```ts
saveDocument(document, true);
```

Good:

```ts
saveDocumentSnapshot(document);
exportDocumentCopy(document);
```

If an operation has several options, use an options type.

Good:

```rust
pub struct ExportOptions {
    pub format: ExportFormat,
    pub citation_style: CitationStyle,
    pub include_comments: bool,
}
```

### 2.8 No hidden side effects

Function names and types must make side effects obvious.

A function that writes to SQLite, disk, network, event streams, process state, worker processes, or global state must say so through its name, type, or documentation.

Prefer:

```rust
reference_store.save_record(ctx, record).await?;
```

over:

```rust
save(record).await?;
```

Prefer:

```ts
await invokeSaveDocumentSnapshot(snapshot);
```

over:

```ts
await sync(snapshot);
```

### 2.9 No global mutable state

Allowed globals:

- Constants.
- Static regular expressions compiled once.
- Sentinel errors where the language uses them.
- Embedded assets.
- Explicit one-time initialization for infrastructure that cannot be constructed otherwise.

Forbidden globals:

- Package-level mutable maps.
- Package-level mutable slices or arrays.
- Mutable registries changed by import side effects.
- Global clients that bypass the Rust network client.
- Shared state hidden behind package functions.
- Frontend module-level mutable state used as application state.

Bad:

```rust
static mut CURRENT_DOCUMENT_ID: Option<DocumentId> = None;
```

Good:

```rust
pub struct DocumentRegistry {
    open_documents: HashMap<DocumentId, DocumentHandle>,
}
```

Bad:

```ts
let activeDocumentId: string | undefined;
```

Good:

```ts
const activeDocumentId = useActiveDocumentId();
```

### 2.10 Prefer explicit types over clever abstractions

Use concrete domain types.

Good:

```rust
pub struct DocumentId(String);
pub struct Citekey(String);
pub enum CitationStyle {
    Apa7,
    Mla9,
    Chicago,
}
```

Bad:

```rust
HashMap<String, serde_json::Value>
```

Good:

```ts
export interface CitationNodeAttrs {
  schemaVersion: 1;
  citekey: string;
  renderStyle: CitationStyle;
}
```

Bad:

```ts
type CitationNodeAttrs = Record<string, any>;
```

Use `unknown` at untrusted boundaries. Narrow it with a type guard before use.

### 2.11 Functions should read top to bottom

Avoid callbacks that mutate outer scope.

Avoid deferred functions that change return values.

Avoid control flow that requires the reader to jump across the file to understand the result.

Bad:

```rust
let mut saved = false;
items.iter().for_each(|item| {
    if item.id == target_id {
        saved = save_item(item).is_ok();
    }
});
```

Good:

```rust
for item in items {
    if item.id != target_id {
        continue;
    }

    save_item(item)?;
    return Ok(());
}

Err(SaveItemError::NotFound)
```

### 2.12 Comments explain why, not what

Good:

```rust
// Publisher logins stay outside DRAFT. Opening the browser preserves the user's existing session without bringing credentials into our process.
open_in_system_browser(url)?;
```

Bad:

```rust
// Open URL.
open_in_system_browser(url)?;
```

Good:

```ts
// Tiptap can emit a partially updated selection during composition. Wait for the next editor transaction before showing citation controls.
```

Bad:

```ts
// Set state.
setIsCitationMenuOpen(true);
```

### 2.13 No dead code

Delete dead code.

Git remembers.

Do not leave commented-out implementations, unused modules, disabled tests, or abandoned helpers.

Bad:

```rust
// TODO: old export path, maybe needed later
// pub fn export_old_docx(...) { ... }
```

Good:

```text
Delete the old export path. Recover it from Git if it becomes necessary.
```

### 2.14 No speculative abstractions

Do not add an abstraction until the concrete need exists.

Do not add factories, registries, plugin systems, generic worker routers, or provider frameworks before the product has more than one real implementation or a test seam that materially improves correctness.

Bad:

```rust
trait UniversalDocumentTransformationProviderFactoryPluginRegistry {
    fn get(&self, name: &str) -> Box<dyn Provider>;
}
```

Good:

```rust
pub struct FormattingWorker {
    python: PythonHelperRunner,
}
```

### 2.15 No framework magic

Avoid hidden behavior.

Forbidden patterns:

- `init` or import side effects that mutate behavior.
- Reflection-heavy routing or persistence.
- Tauri commands that are registered indirectly through unclear macro layers.
- Python imports that execute work.
- Bash scripts that download or execute remote code.
- Code generation that reviewers cannot inspect.
- Hidden network clients inside feature modules.

Bad:

```rust
inventory::submit! {
    HiddenCommandRegistration::new("save_document", save_document)
}
```

Good:

```rust
pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        save_document,
        search_references,
        run_text_analysis,
    ])
}
```

### 2.16 No copy-paste logic

Extract common code at the second use within a package.

Do not force reuse across domain boundaries if it damages readability. Duplication across unrelated domains can be acceptable when the concepts are different.

Good reuse:

```rust
fn validate_citekey(value: &str) -> Result<Citekey, CitekeyError> {
    // used by citation insertion and bibliography generation
}
```

Bad reuse:

```rust
fn validate_everything(input: &str) -> Result<(), ValidationError> {
    // validates citekeys, file paths, model IDs, style names, and document titles
}
```

### 2.17 Invariants are code rules, not documentation wishes

If code touches a boundary covered by `INVARIANTS.md`, it must include enforcement.

A pull request that changes a boundary must update at least one of these:

- A test.
- A lint rule.
- A local verification script.
- A GitHub Actions workflow.
- An ADR explaining why the invariant changed.

Bad:

```text
This should be safe because the frontend only calls this in one place.
```

Good:

```text
This is safe because the Rust command revalidates the citation node and the regression test fails on unknown schema versions.
```

---

## 3. Repository Structure

The repository should make ownership obvious from the path.

Recommended shape:

```text
.
├── src-tauri/
│   ├── Cargo.toml
│   └── src/
│       ├── app_state.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── citation_commands.rs
│       │   ├── document_commands.rs
│       │   ├── reference_commands.rs
│       │   ├── analysis_commands.rs
│       │   └── formatting_commands.rs
│       ├── citation/
│       ├── document/
│       ├── reference_library/
│       ├── analysis/
│       ├── formatting/
│       ├── text_analysis/
│       ├── jobs/
│       ├── network/
│       ├── workers/
│       ├── storage/
│       └── errors/
├── src/
│   ├── app/
│   ├── components/
│   ├── editor/
│   ├── features/
│   │   ├── citation-search/
│   │   ├── document-settings/
│   │   ├── formatting-panel/
│   │   └── text-analysis-panel/
│   ├── ipc/
│   ├── types/
│   └── utils/
├── python/
│   ├── helpers/
│   ├── contracts/
│   └── tests/
├── scripts/
├── docs/
│   ├── adr/
│   └── contracts/
├── .github/
│   └── workflows/
├── ARCHITECTURE.md
├── GOVERNANCE.md
├── INVARIANTS.md
└── CODING_STYLE.md
```

Path rules:

- `src-tauri/src/network/` is the only place that constructs external network clients.
- `src-tauri/src/workers/` is the only place that invokes Python helpers.
- `src-tauri/src/document/` owns durable document mutation and save behavior.
- `src-tauri/src/commands/` exposes Tauri commands. It should stay thin.
- `src/ipc/` is the frontend's only Tauri command wrapper layer.
- `src/features/` contains UI feature flows, not trusted business rules.
- `python/helpers/` contains allowlisted helper entry points.
- `scripts/` contains local and CI automation only.

---

## 4. Rust Specifics

### 4.1 Rust owns trusted work

Rust code owns anything that can affect durable state, user files, external services, secrets, or long-running workers.

Do not move trusted logic into React because it is easier to implement there.

Bad:

```tsx
if (userConfirmed) {
  localStorage.setItem("llm_api_key", apiKey);
}
```

Good:

```tsx
await saveApiKeyToCredentialStore({ provider, apiKey });
```

### 4.2 Tauri commands stay thin

A command should parse, validate, call a domain service, and return a typed result.

Bad:

```rust
#[tauri::command]
pub async fn search_references(query: String) -> Result<String, String> {
    let url = format!("https://api.crossref.org/works?query={query}");
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    Ok(body)
}
```

Good:

```rust
#[tauri::command]
pub async fn search_references(
    state: State<'_, AppState>,
    request: SearchReferencesRequest,
) -> Result<SearchReferencesResponse, SearchReferencesError> {
    let query = SearchQuery::parse(request.query)?;
    state.reference_service.search(query).await
}
```

### 4.3 Use command-specific error enums

No Tauri command returns `anyhow::Error`, `String`, `serde_json::Value`, or `Box<dyn Error>`.

Bad:

```rust
#[tauri::command]
pub async fn save_document(request: SaveDocumentRequest) -> Result<(), anyhow::Error> {
    save(request).await?;
    Ok(())
}
```

Good:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SaveDocumentError {
    #[error("document validation failed")]
    ValidationFailed,

    #[error("document is already open elsewhere")]
    AlreadyOpen,

    #[error("failed to write document atomically")]
    AtomicWriteFailed,
}

#[tauri::command]
pub async fn save_document(
    request: SaveDocumentRequest,
) -> Result<SaveDocumentResponse, SaveDocumentError> {
    // ...
}
```

If the error crosses IPC, provide a deliberate serialized error shape.

Good:

```rust
#[derive(Debug, Serialize)]
pub struct CommandErrorDto {
    pub code: &'static str,
    pub message: String,
    pub recoverable: bool,
}
```

### 4.4 Wrap errors at the boundary where context is added

Wrap once when the caller can add useful context.

Bad:

```rust
let text = std::fs::read_to_string(path)
    .map_err(|err| SaveDocumentError::Io(format!("error: {err}")))?;
```

Good:

```rust
let text = std::fs::read_to_string(path)
    .map_err(|source| SaveDocumentError::ReadSourceDocument { path, source })?;
```

Do not log and return the same error unless the caller cannot log it.

### 4.5 Use domain types for IDs and state

Good:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DocumentId(String);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct JobId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobState {
    Pending,
    InProgress,
    Resolved,
    Failed,
    NeedsManualInput,
    Cancelled,
}
```

Bad:

```rust
pub type DocumentId = String;
pub type JobState = String;
```

### 4.6 Centralize network access

Only the Rust network module constructs external clients.

Bad:

```rust
pub async fn lookup_metadata(doi: &str) -> Result<Response, Error> {
    let client = reqwest::Client::new();
    client.get(format!("https://api.crossref.org/works/{doi}")).send().await?;
    // ...
}
```

Good:

```rust
pub async fn lookup_metadata(
    network: &NetworkClient,
    doi: Doi,
) -> Result<CitationMetadata, MetadataError> {
    network.crossref().lookup_work(doi).await
}
```

### 4.7 Use explicit cancellation for long-running work

Every user-initiated long-running worker must have a visible cancel path unless it is documented as short-lived, idempotent, and safe to complete.

Good:

```rust
pub async fn start_text_analysis(
    worker_registry: &WorkerCancellationRegistry,
    request: StartTextAnalysisRequest,
) -> Result<StartTextAnalysisResponse, StartTextAnalysisError> {
    let registration = worker_registry.register()?;
    let worker_id = registration.worker_id();
    spawn_text_analysis_worker(registration, request)?;
    Ok(StartTextAnalysisResponse { worker_id })
}

pub fn cancel_text_analysis(
    worker_registry: &WorkerCancellationRegistry,
    worker_id: WorkerId,
) -> Result<CancelWorkerOutcome, WorkerCancellationError> {
    worker_registry.cancel(worker_id)
}
```

Bad:

```rust
pub async fn start_text_analysis(request: Request) {
    tokio::spawn(async move {
        run_forever(request).await;
    });
}
```

### 4.8 Save atomically

Document saves must use the atomic writer. Do not hand-roll save logic in feature modules.

Bad:

```rust
std::fs::write(path, serialized_document)?;
```

Good:

```rust
atomic_document_writer.write(path, serialized_document.as_bytes()).await?;
```

### 4.9 Tests protect boundaries

Tests must cover negative paths, not just successful paths.

Required examples:

```text
unknown_citation_schema_version_returns_migration_error
frontend_cannot_call_external_network_directly
second_document_open_returns_already_open
python_helper_with_network_import_is_rejected
save_interrupted_mid_write_leaves_valid_document
```

Use table tests for validation logic. Use integration tests for boundary behavior. Use property or fuzz tests where invalid structured input is the risk.

---

## 5. TypeScript, React, and Tiptap Specifics

### 5.1 No `any`

Use explicit types. Use `unknown` only for untrusted input and narrow it immediately.

Bad:

```ts
function renderIssue(issue: any) {
  return issue.message;
}
```

Good:

```ts
interface TextAnalysisIssue {
  id: string;
  message: string;
  severity: "info" | "warning" | "error";
}

function renderIssue(issue: TextAnalysisIssue) {
  return issue.message;
}
```

Good boundary handling:

```ts
function isCitationNodeAttrs(value: unknown): value is CitationNodeAttrs {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const attrs = value as Record<string, unknown>;
  return attrs.schemaVersion === 1
    && typeof attrs.citekey === "string"
    && isCitationStyle(attrs.renderStyle);
}
```

### 5.2 Use a typed IPC layer

React components do not call `invoke` directly. They call typed functions from `src/ipc/`.

Bad:

```tsx
const result = await invoke("save_document", { document });
```

Good:

```tsx
const result = await saveDocument({ documentId, snapshot });
```

Good IPC wrapper:

```ts
export async function saveDocument(
  request: SaveDocumentRequest,
): Promise<SaveDocumentResponse> {
  return invokeCommand<SaveDocumentResponse>("save_document", request);
}
```

### 5.3 React components are display and interaction surfaces

Components should not own durable policy or trusted decisions.

Bad:

```tsx
function ApplyCitationButton({ citation }: Props) {
  if (citation.schemaVersion !== 1) {
    return null;
  }

  editor.commands.insertContent(citation);
}
```

Good:

```tsx
function ApplyCitationButton({ citation }: Props) {
  const insertCitation = useInsertCitation();

  return (
    <button onClick={() => insertCitation(citation.citekey)}>
      Insert citation
    </button>
  );
}
```

Rust still validates the citation before mutation.

### 5.4 Use props interfaces

Bad:

```tsx
export function IssueRow(props: { id: string; message: string; level?: string }) {
  // ...
}
```

Good:

```tsx
interface IssueRowProps {
  id: string;
  message: string;
  severity?: IssueSeverity;
}

export function IssueRow(props: IssueRowProps) {
  // ...
}
```

### 5.5 Hooks stay at the top

No conditional hooks. Effects clean up subscriptions, timers, and streams.

Bad:

```tsx
if (isOpen) {
  useEffect(() => {
    listen("analysis-progress", onProgress);
  }, []);
}
```

Good:

```tsx
useEffect(() => {
  if (!isOpen) {
    return;
  }

  let unlisten: (() => void) | undefined;

  listenToAnalysisProgress(onProgress).then((cleanup) => {
    unlisten = cleanup;
  });

  return () => {
    unlisten?.();
  };
}, [isOpen, onProgress]);
```

### 5.6 Tiptap node code must be explicit

Citation nodes must use a fixed schema. Unknown attrs are not ignored.

Bad:

```ts
addAttributes() {
  return {
    data: { default: {} },
  };
}
```

Good:

```ts
addAttributes() {
  return {
    schemaVersion: {
      default: 1,
      parseHTML: (element) => Number(element.getAttribute("data-schema-version")),
    },
    citekey: {
      default: null,
      parseHTML: (element) => element.getAttribute("data-citekey"),
    },
    renderStyle: {
      default: "apa7",
      parseHTML: (element) => element.getAttribute("data-render-style"),
    },
  };
}
```

The frontend schema is a rendering guard. Rust remains the authority before save, export, formatting, or analysis.

### 5.7 Keep components small

Components should stay under 200 lines.

Start extracting around 150 lines when there is a clear boundary.

Extract in this order:

1. Presentational child component.
2. Custom hook for local UI behavior.
3. Feature-level helper.
4. Shared utility only after a second real use.

### 5.8 State location rule

Frontend state is allowed when losing it on WebView reload is acceptable.

Allowed frontend state:

- Current sidebar tab.
- Search box text before submission.
- Hover state.
- Open menu state.
- Temporary loading state.

Rust-owned state:

- Document content.
- Reference records.
- Citation metadata.
- Job state.
- Analysis findings that should persist.
- Formatting findings that should persist.
- Secrets.
- User preferences.

---

## 6. Python Helper Specifics

### 6.1 Python is a helper surface, not an authority surface

Python helpers run bounded formatting and text-analysis operations. Rust invokes them. Rust validates their input and output. Rust decides whether results become durable state.

Bad:

```python
# helper owns the database and mutates document state directly
conn = sqlite3.connect("draft.db")
conn.execute("UPDATE documents SET body = ?", [new_body])
```

Good:

```python
def analyze_text(request: TextAnalysisRequest) -> TextAnalysisResponse:
    findings = run_local_checks(request.text)
    return TextAnalysisResponse(findings=findings)
```

### 6.2 Use typed input and output

A helper must have a documented request and response shape.

Good:

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class TextAnalysisRequest:
    text: str
    locale: str

@dataclass(frozen=True)
class TextAnalysisFinding:
    code: str
    message: str
    start_offset: int
    end_offset: int

@dataclass(frozen=True)
class TextAnalysisResponse:
    findings: list[TextAnalysisFinding]
```

Bad:

```python
def analyze(payload):
    return {"stuff": do_things(payload)}
```

### 6.3 Helpers read only approved input

Rust passes the payload or an approved temporary path. Python does not discover files on its own.

Bad:

```python
for path in Path.home().glob("**/*.docx"):
    analyze(path)
```

Good:

```python
def main() -> int:
    request = read_request_from_stdin()
    response = analyze_text(request)
    write_response_to_stdout(response)
    return 0
```

### 6.4 No network by default

Python helpers must not call external network services unless an ADR explicitly approves the helper and the request still follows Rust network policy.

Bad:

```python
import requests

requests.post("https://api.example.com/check", json={"text": text})
```

Good:

```python
# Local deterministic check. No network.
findings = check_sentence_length(text)
```

### 6.5 No shell execution from helpers

Bad:

```python
subprocess.run(f"pandoc {input_path} -o {output_path}", shell=True)
```

Allowed only when approved and argument-array based:

```python
subprocess.run(
    ["python", "-m", "some_allowlisted_tool", str(input_path)],
    check=True,
    timeout=30,
)
```

Prefer no subprocess inside helpers. Let Rust own worker orchestration.

### 6.6 Exit behavior is part of the contract

A helper returns:

- `0` for success with valid response JSON.
- Non-zero for failed execution.
- Machine-readable error output when possible.
- Human-readable diagnostics on stderr only when they do not include document text or secrets.

Bad:

```python
print("failed but continuing")
return {}
```

Good:

```python
raise HelperError(code="invalid_request", message="text field is required")
```

---

## 7. Bash Specifics

### 7.1 Bash is for local and CI orchestration

Bash scripts may format, lint, test, verify, and compose local developer commands.

Bash scripts must not become product runtime code.

Bad:

```rust
Command::new("bash")
    .arg("scripts/process_user_document.sh")
    .arg(user_path)
    .spawn()?;
```

Good:

```bash
#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm run typecheck
python -m pytest python/tests
```

### 7.2 Use strict mode

Every Bash script starts with:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

Use `IFS` only when needed and locally scoped.

### 7.3 Quote variables

Bad:

```bash
rm -rf $BUILD_DIR
```

Good:

```bash
rm -rf "${BUILD_DIR}"
```

### 7.4 No `eval`

Bad:

```bash
eval "cargo test ${USER_ARGS}"
```

Good:

```bash
cargo test --all-targets
```

### 7.5 No remote execution

Bad:

```bash
curl https://example.com/install.sh | bash
```

Good:

```bash
# Pin dependencies through project tooling and lockfiles.
npm ci
cargo fetch --locked
```

### 7.6 Scripts should be runnable locally and in GitHub Actions

A CI-blocking check should call a script that developers can run locally.

Good:

```yaml
- name: Check invariants
  run: just check-invariants
```

Good:

```bash
just check-invariants
```

Bad:

```yaml
- name: Run hidden CI-only checks
  run: |
    cargo clippy
    python some-inline-script-that-does-not-exist-locally.py
```

---

## 8. Formatting and Tooling

Use automated formatters. Do not debate hand formatting in review.

Expected tools:

```text
Rust:        rustfmt, clippy, cargo test
TypeScript:  eslint, prettier, tsc
React:       eslint react rules, hooks rules
Python:      ruff, black or ruff format, mypy/pyright when adopted, pytest
Bash:        shellcheck, shfmt
Markdown:    markdownlint or equivalent when adopted
```

Preferred local commands:

```bash
just format
just lint
just test
just check-invariants
just verify
```

`just verify` should be the broad local command that approximates GitHub Actions.

When `just` is not installed, `bash scripts/verify.sh` is the required
equivalent. The `justfile` delegates to repository scripts so local development
and future GitHub Actions can use the same implementation.

---

## 9. Documentation Comments

### 9.1 Public boundary items need comments

Use documentation comments for public commands, services, worker contracts, and boundary types.

Good:

```rust
/// Saves a validated document snapshot using the atomic write path.
///
/// This function is the only durable save entry point. Callers must not write
/// document files directly because interrupted writes can corrupt user work.
pub async fn save_document_snapshot(...) -> Result<..., ...> {
    // ...
}
```

Bad:

```rust
/// Saves document.
pub async fn save_document_snapshot(...) -> Result<..., ...> {
    // ...
}
```

### 9.2 Comments should name the risk

Good:

```rust
// A browser handoff keeps publisher credentials outside DRAFT's process boundary.
open_in_system_browser(url)?;
```

Bad:

```rust
// Opens browser.
open_in_system_browser(url)?;
```

### 9.3 Do not narrate obvious syntax

Bad:

```ts
// Create array of issues.
const issues = [];
```

Good:

```ts
// Keep unresolved issues visible so the user can review AI-suggested edits before applying them.
const unresolvedIssues = issues.filter(isUnresolved);
```

---

## 10. Review Rules

A reviewer should reject code that:

- Violates an invariant.
- Makes the frontend authoritative for trusted decisions.
- Adds direct frontend network, filesystem, or secret access.
- Adds ad hoc Rust network clients outside the network module.
- Lets Python helpers own persistence, secrets, source-document mutation, or network access.
- Uses Bash as product runtime code.
- Returns generic command errors.
- Silently ignores errors.
- Silently ignores unsupported citation schema versions.
- Applies AI, formatting, or text-analysis changes without a Rust-owned mutation path.
- Adds hidden privileged behavior.
- Expands the trusted computing base without justification.
- Adds broad abstractions before there is a concrete need.
- Uses shell execution where a typed integration layer should exist.
- Adds unaudited privileged mutation.
- Changes architecture, governance, contracts, or invariants without the required ADR/process.

Review questions:

```text
What boundary does this code cross?
What state does this code read?
What state does this code write?
What happens if it fails halfway through?
What happens if the WebView reloads?
What happens if the network is offline?
What happens if the Python helper fails?
What prevents this from violating an invariant?
Can this check run locally and in GitHub Actions?
```

---

## 11. Examples of Preferred Code Shape

### 11.1 Rust service shape

```rust
pub struct ReferenceService {
    network: NetworkClient,
    store: ReferenceStore,
}

impl ReferenceService {
    pub async fn search(
        &self,
        query: SearchQuery,
    ) -> Result<SearchReferencesResponse, SearchReferencesError> {
        let hits = self.network.crossref().search(query).await?;
        let records = normalize_crossref_hits(hits)?;
        self.store.save_search_results(&records).await?;

        Ok(SearchReferencesResponse { records })
    }
}

fn normalize_crossref_hits(
    hits: CrossrefSearchResponse,
) -> Result<Vec<CitationRecord>, SearchReferencesError> {
    // helper details
}
```

### 11.2 TypeScript IPC wrapper shape

```ts
export interface SearchReferencesRequest {
  query: string;
}

export interface SearchReferencesResponse {
  records: CitationRecordSummary[];
}

export async function searchReferences(
  request: SearchReferencesRequest,
): Promise<SearchReferencesResponse> {
  return invokeCommand<SearchReferencesResponse>("search_references", request);
}
```

### 11.3 React feature shape

```tsx
export function CitationSearchPanel() {
  const [query, setQuery] = useState("");
  const { results, status, search } = useCitationSearch();

  return (
    <CitationSearchView
      query={query}
      results={results}
      status={status}
      onQueryChange={setQuery}
      onSearch={() => search(query)}
    />
  );
}
```

### 11.4 Python helper shape

```python
from __future__ import annotations

import sys

from draft_helpers.contracts import read_request, write_response
from draft_helpers.text_analysis import analyze_text


def main() -> int:
    request = read_request(sys.stdin)
    response = analyze_text(request)
    write_response(sys.stdout, response)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

### 11.5 Bash verification shape

```bash
#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run typecheck
npm run lint
python -m pytest python/tests
shellcheck scripts/*.sh
```

---

## 12. Final Rule

Prefer the code a tired reviewer can still understand.

DRAFT protects user documents, source records, citation integrity, and writing workflow trust. The code should make those protections visible.
