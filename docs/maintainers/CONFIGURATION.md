# Configuration And Limits

## Purpose

This index names significant configuration values, schemas, limits, timeouts,
thresholds, and feature switches in the implemented repository. The source file
remains authoritative; this guide explains ownership and change impact so
maintainers do not have to discover policy by searching literals.

Changing a value that alters ownership, persistence compatibility, security,
export behavior, or an invariant requires the governance process. Ordinary
tool-version or presentation-size changes still require their owning tests and
documentation to remain aligned.

## Inclusion Rule

This index includes only values that affect observable behavior, compatibility,
safety, bounded resource use, external access, packaging, or troubleshooting.
It excludes local variable names, incidental constants, test-only literals, and
implementation choices that can change without altering one of those contracts.

## Application And Toolchain

| Value | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| Product name | `DRAFT` | `src-tauri/tauri.conf.json` | Desktop display and bundle name. |
| Application version | `0.1.0` | `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `package.json` | Pre-release compatibility line; not evidence of a published release. |
| Bundle identifier | `com.progentic.draft` | `src-tauri/tauri.conf.json` | Platform application identity. |
| Rust toolchain | `1.96.0` | `rust-toolchain.toml`, `src-tauri/Cargo.toml` | Pinned compiler, rustfmt, and clippy baseline. |
| Node engine | `>=22.12.0` | `package.json` | Supported local runtime floor. GitHub Actions currently verifies Node 24. |
| npm version | `11.16.0` | `package.json` | Locked package-manager contract. |
| Tauri CLI | `2.11.4` | `package.json`, `package-lock.json` | Pinned desktop build and icon-generation tool. |
| Python CI version | `3.12` | `.github/workflows/verify.yml` | Hosted helper-test runtime. |
| Production Python minimum | `3.9` | `pyproject.toml` | Matches the Apple system runtime used by the initial macOS package. |
| CI timeout | 30 minutes | `.github/workflows/verify.yml` | Upper bound for the aggregate Verify job. |
| Development URL | `http://localhost:1420` | `src-tauri/tauri.conf.json` | Vite endpoint used only by Tauri development. |
| Frontend output | `../dist` | `src-tauri/tauri.conf.json` | Production WebView assets consumed by Tauri builds, relative to `src-tauri/`. |
| macOS package command | `npm run package:macos` | `package.json`, `scripts/package-macos.sh` | Builds and validates the unsigned Apple Silicon `.app`; signing and publication are excluded. |

## Window And Capability Defaults

| Value | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| Main window | 1200 by 800 pixels | `src-tauri/tauri.conf.json` | Initial desktop workspace size. |
| Minimum window | 760 by 560 pixels | `src-tauri/tauri.conf.json` | Smallest supported shell geometry. |
| Content security policy | `null` | `src-tauri/tauri.conf.json` | No custom CSP is configured yet; Phase 44 records this as release blocker `RC-05` for Phase 48 closure. |
| WebView capability | event listen and unlisten only | `src-tauri/capabilities/main.json` | Main window can receive typed Rust events; it has no direct filesystem, opener, or network capability. |
| Bundle activation | `true` | `src-tauri/tauri.conf.json` | Tauri bundling is active for the supported configured target. |
| Bundle targets | `app` only | `src-tauri/tauri.conf.json` | Phase 42 produces an unsigned macOS application bundle, not a DMG or another platform installer. |
| Desktop icon paths | `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico` | `src-tauri/tauri.conf.json` | Exact assets embedded by the app package path. |
| `DOCUMENT_EXTENSIONS` | `draft`, `json` | `documents/dialog.rs` | Native open-dialog extension filters. |
| `DEFAULT_DOCUMENT_FILE_NAME` | `Untitled.draft` | `documents/dialog.rs` | Initial native save-dialog name. |

## Schemas And Persistence

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `DOCUMENT_ENVELOPE_SCHEMA_VERSION` | 1 | `documents/envelope.rs` | Accepted document envelope schema. |
| `CITATION_NODE_SCHEMA_VERSION` | 1 | Rust and TypeScript citation modules | Accepted citation attrs schema. |
| `REFERENCE_RECORD_SCHEMA_VERSION` | 1 | `references/record.rs` | Accepted reference payload schema. |
| `REFERENCE_STORE_SCHEMA_VERSION` | 1 | `references/store.rs` | SQLite reference-store schema. |
| `REFERENCE_STORE_FILENAME` | `references.sqlite3` | `references/store.rs` | Database under the Rust-resolved application data directory. |
| `REFERENCE_STORE_BUSY_TIMEOUT` | 5 seconds | `references/store.rs` | Maximum wait for a competing SQLite writer. |
| `JOB_STORE_SCHEMA_VERSION` | 1 | `jobs/store.rs` | SQLite PDF import-job schema. |
| `JOB_STORE_FILENAME` | `jobs.sqlite3` | `jobs/store.rs` | Job database under the Rust-resolved application data directory. |
| `JOB_STORE_BUSY_TIMEOUT` | 5 seconds | `jobs/store.rs` | Maximum wait for a competing SQLite writer. |
| `MAX_JOB_FAILURE_MESSAGE_BYTES` | 512 bytes | `jobs/pdf_import.rs` | Durable diagnostic-message bound. |

The reference store has one strict `reference_records` table. The job store has
one strict `pdf_import_jobs` table with one record per candidate identity,
closed states, one `intake_validated` checkpoint, attempt count, typed failure,
durable cancellation intent, and a hashed claim token. Both stores initialize
schema version 1 from an empty database and reject unknown future versions.
Document, citation, and reference payload version 1 is the first released
baseline, so no older known payload migration exists. Phase 43 requires lower
and future payloads to fail without mutation. The policy and requirements for a
later explicit transition are documented in `DATA_MIGRATION.md`.

## Secret Storage

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `NATIVE_SERVICE_NAME` | `com.progentic.draft` | `secrets/store.rs` | Fixed OS credential-manager service namespace. |
| `API_KEY_ACCOUNT_PREFIX` | `service-api-key/` | `secrets/store.rs` | Fixed prefix for internal service API-key slots. |
| `MAX_INTEGRATION_NAME_BYTES` | 64 bytes | `secrets/store.rs` | Maximum normalized lowercase ASCII integration identifier. |
| `MAX_SECRET_BYTES` | 4,096 bytes | `secrets/store.rs` | Maximum owned binary secret value. |

The store uses `keyring` 4.1.4 and `zeroize` 1.9.0. Native access is lazy; these
settings do not create or probe a credential during startup. See
`docs/maintainers/SECRET_STORAGE.md`.

## Local Diagnostics

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION` | 1 | `diagnostics.rs` | Strict local diagnostic response schema. |
| `MAX_DIAGNOSTIC_SNAPSHOT_BYTES` | 2,048 bytes | `diagnostics.rs` | Maximum complete serialized snapshot size. |
| `MAX_APPLICATION_VERSION_BYTES` | 64 bytes | `diagnostics.rs` | Maximum compiled package-version value admitted to a snapshot. |

The report also names six existing schema/protocol constants already indexed
above and below. Its fixed subsystem states perform no health, network, file,
database, Python, or credential-store probe. See
`docs/maintainers/AUDIT_DIAGNOSTICS.md`.

## Network And Intake

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `DEFAULT_CONNECTIVITY_MODE` | `online` | `network/connectivity.rs` | Initial process-local mode for every new application session. |
| `NETWORK_CONNECT_TIMEOUT` | 10 seconds | `network/client.rs` | Connection-establishment bound. |
| `NETWORK_REQUEST_TIMEOUT` | 30 seconds | `network/client.rs` | Complete request bound. |
| `PROVIDER_REQUEST_INTERVAL` | 1 second | `network/client.rs` | Minimum interval per metadata provider. |
| `MAX_METADATA_RESPONSE_BYTES` | 1 MiB | `network/client.rs` | Maximum retained metadata response. |
| `MAX_RATE_LIMIT_BACKOFF` | 60 seconds | `network/client.rs` | Maximum bounded HTTP 429 delay. |
| `STABLE_WRITE_DEBOUNCE` | 1 second | `imports/pdf.rs` | Required quiet period before watched PDF confirmation. |
| `MAX_EXTERNAL_URL_LENGTH` | 2,048 characters | `research/external_access.rs` | Maximum accepted browser-handoff URL. |
| `MAX_SCHOLAR_QUERY_LENGTH` | 2,048 characters | `research/external_access.rs` | Maximum accepted Google Scholar query. |
| `MAX_DOI_LENGTH` | 2,048 characters | `research/metadata.rs` | Maximum normalized DOI input. |
| `MAX_CONTACT_EMAIL_LENGTH` | 254 characters | `research/metadata.rs` | Maximum provider contact identity. |
| `DOI_RESOLVER_BASE_URL` | `https://doi.org` | `research/external_access.rs` | Fixed DOI handoff origin. |
| `GOOGLE_SCHOLAR_BASE_URL` | `https://scholar.google.com/scholar` | `research/external_access.rs` | Fixed Scholar handoff origin. |
| `CROSSREF_BASE_URL` | `https://api.crossref.org/v1` | Crossref provider | Fixed API origin. |
| `SEMANTIC_SCHOLAR_BASE_URL` | `https://api.semanticscholar.org/graph/v1` | Semantic Scholar provider | Fixed API origin. |
| `UNPAYWALL_BASE_URL` | `https://api.unpaywall.org/v2` | Unpaywall provider | Fixed API origin. |

The watched-PDF gate also requires unchanged byte length across the quiet
period. It cannot detect an unreported same-size in-place modification.

## Analysis And Python Helpers

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `MAX_AI_INSTRUCTION_BYTES` | 4 KiB | `analysis/context.rs` | Analysis instruction bound. |
| `MAX_AI_EXCERPTS_PER_CLASS` | 64 | `analysis/context.rs` | Separate document and evidence count bound. |
| `MAX_AI_EXCERPT_BYTES` | 8 KiB | `analysis/context.rs` | Per-excerpt bound. |
| `MAX_AI_CONTEXT_CLASS_BYTES` | 32 KiB | `analysis/context.rs` | Separate document and evidence byte budget. |
| `MAX_EVIDENCE_ID_BYTES` | 128 bytes | `analysis/context.rs` | Verified evidence identity bound. |
| `MAX_CITEKEY_BYTES` | 256 bytes | `analysis/context.rs` | AI request citekey bound. |
| `MAX_AI_STREAM_CHUNK_BYTES` | 16 KiB | `analysis/ai.rs` | Per-event chunk bound. |
| `MAX_AI_STREAM_CHUNKS` | 4,096 | `analysis/ai.rs` | Stream event count bound. |
| `MAX_AI_STREAM_BYTES` | 1 MiB | `analysis/ai.rs` | Cumulative generated-output bound. |
| `PYTHON_HELPER_PROTOCOL_VERSION` | 1 | `workers/python/protocol.rs`, `python/draft_helpers/worker.py` | Rust/Python wire protocol version. |
| `CONTRACT_PROBE_VERSION` | 1 | Rust and Python helper protocol | Contract-probe operation version. |
| `TEXT_ANALYSIS_VERSION` | 1 | Rust and Python helper protocol | Text-analysis operation version. |
| `MAX_CONTRACT_PROBE_TEXT_BYTES` | 32 KiB | `workers/python/protocol.rs` | Probe text bound. |
| `MAX_TEXT_ANALYSIS_TEXT_BYTES` | 32 KiB | `workers/python/protocol.rs` | Review text bound. |
| `MAX_PYTHON_HELPER_REQUEST_BYTES` | 64 KiB | `workers/python/protocol.rs` | Serialized request bound. |
| `MAX_PYTHON_HELPER_STDOUT_BYTES` | 64 KiB | `workers/python/runner.rs` | Captured stdout bound. |
| `MAX_PYTHON_HELPER_STDERR_BYTES` | 16 KiB | `workers/python/runner.rs` | Captured stderr bound. |
| `PYTHON_HELPER_TIMEOUT` | 5 seconds | `workers/python/runner.rs` | Process execution bound. |
| `MAX_TEXT_ANALYSIS_FINDINGS` | 100 | Rust text-analysis boundary | Validated result bound. |
| `SUPPORTED_LOCALE` | `en-US` | `python/draft_helpers/worker.py` | Only accepted helper locale. |
| Production executable | `/usr/bin/python3` | `commands/text_analysis.rs` | Fixed initial-platform runtime; no user-selected executable. |
| Packaged helper resource | `python/draft_helpers` | `tauri.conf.json` | Trusted helper files embedded in the `.app`. |
| Isolated helper `TMPDIR` | `/tmp` | `workers/python/runner.rs` | Sole value restored after `env_clear` to keep Apple system Python silent. |
| `MAX_FINDINGS_PER_CHECK` | 20 | `python/draft_helpers/worker.py` | Maximum findings emitted by one heuristic. |
| `LONG_SENTENCE_WORDS` | 30 words | `python/draft_helpers/worker.py` | Clarity heuristic threshold. |
| `MIN_ALL_CAPS_LETTERS` | 5 letters | `python/draft_helpers/worker.py` | Tone heuristic threshold. |
| `MIN_REPEATED_OPENER_LETTERS` | 4 letters | `python/draft_helpers/worker.py` | Cohesion heuristic threshold. |

## Formatting And Export

| Symbol | Current setting | Source | Meaning |
| :--- | :--- | :--- | :--- |
| `MAX_FORMATTING_HEADINGS` | 512 | `formatting/checks.rs` | Heading snapshot bound. |
| `MAX_FORMATTING_CITATIONS` | 512 | `formatting/checks.rs` | Citation declaration bound. |
| `MAX_HEADING_TITLE_BYTES` | 512 bytes | `formatting/checks.rs` | Per-heading title bound. |
| Heading levels | 1 through 6 | `formatting/checks.rs` | Accepted outline range. |
| Formatting styles | `apa7`, `mla9`, `chicago17_author_date` | `formatting/checks.rs` | Closed consistency identifiers, not complete style conformance. |
| `DEFAULT_FORMATTING_STYLE` | `apa7` | `src/ipc/formattingReview.ts` | Initial review selection; either other closed identifier may be selected before a run. |
| Font-family identifiers | `arial`, `georgia`, `times_new_roman`, `courier_new` | Rust and TypeScript text-format modules | Complete accepted family allowlist. These map exactly to Arial, Georgia, Times New Roman, and Courier New in DOCX output. |
| `MIN_FONT_SIZE_POINTS` | 8 points | Rust and TypeScript text-format modules | Smallest accepted text size. |
| `MAX_FONT_SIZE_POINTS` | 72 points | Rust and TypeScript text-format modules | Largest accepted text size. Sizes are whole points in one-point increments and export as DOCX half-points. |
| `MAX_DOCX_SOURCE_BYTES` | 8 MiB | `exports/docx.rs` | Serialized source-document bound. |
| `MAX_DOCX_NODES` | 100,000 | `exports/docx.rs` | Structural object count bound. |
| `MAX_DOCX_NESTING_DEPTH` | 16 | `exports/docx.rs` | Recursive parser depth bound. |
| `MAX_DOCX_ARTIFACT_BYTES` | 16 MiB | `exports/docx.rs` | Complete package bound before filesystem replacement. |

DOCX compilation supports paragraphs, headings, text, hard breaks, the closed
bold, italic, and underline marks, and the canonical font-family and font-size
marks above. Unknown fields, unsupported nodes
or marks, citations, active content, external relationships, malformed XML
characters, and resource-limit violations fail explicitly.

## Change Checklist

When changing a listed value:

1. Update the owning source and focused tests.
2. Update the owning maintainer guide and this index.
3. Check `ARCHITECTURE.md`, `INVARIANTS.md`, and ADR requirements.
4. Keep Rust/Python and Rust/TypeScript mirrored values equal where applicable.
5. Run `bash scripts/check-docs.sh` and `bash scripts/verify.sh`.
