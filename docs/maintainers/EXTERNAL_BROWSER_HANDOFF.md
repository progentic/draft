# External Browser Handoff

## Status

This guide records implemented Phase 23 behavior. The requirements in
`docs/drafts/EXTERNAL_BROWSER_HANDOFF.md` remain non-binding until they complete
the contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

`open_external_access` opens one publisher, institutional, DOI, or Google
Scholar destination in the user's default system browser. The command is
bounded and synchronous: no DRAFT-owned work continues after the operating
system accepts or rejects the launch.

Phase 23 adds no visible workspace control. The typed Rust command and
TypeScript wrapper are ready for a later research workflow to call.

## Request Boundary

The command accepts a tagged request:

```json
{ "destination": "publisher", "url": "https://publisher.example/article" }
{ "destination": "institutional", "url": "https://library.example/item" }
{ "destination": "doi", "doi": "10.1000/example" }
{ "destination": "google_scholar", "query": "example research title" }
```

Publisher and institutional values must be bounded HTTPS URLs with a host and
without URL username or password fields. DOI values use the shared validated
`Doi` type. Google Scholar queries are bounded, nonblank text without control
characters.

Rust constructs `https://doi.org/...` and
`https://scholar.google.com/scholar?q=...` through `url::Url`. The frontend
cannot replace either service origin.

## Runtime Ownership

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `open_external_access` | Maps the typed IPC request and response. |
| Mid | `open_in_system_browser` | Coordinates target validation and one launch. |
| Mid | `external_access_url` | Selects target-specific URL construction. |
| Low | URL and query validators | Reject malformed or credential-bearing input. |
| Low | `SystemBrowser` | Calls the default OS browser adapter. |

`SystemBrowser` calls the Rust function `tauri_plugin_opener::open_url` with no
specific application. DRAFT does not initialize the opener guest plugin,
install its JavaScript package, or grant an opener capability. The WebView can
request this operation only through `src/ipc/externalAccess.ts`.

## Response and Failures

Success returns only the status and destination:

```json
{ "status": "opened", "destination": "publisher" }
```

The command does not return the opened URL. Typed errors are:

```json
{ "code": "invalid_url" }
{ "code": "invalid_doi" }
{ "code": "invalid_search_query" }
{ "code": "browser_unavailable" }
```

Errors do not retain raw URL, query, dependency, browser, process, credential,
or environment details.

## Security Boundary

The handoff launches the user's browser. DRAFT does not make the resulting
network request and cannot inspect the browser session. Authentication,
downloads, and browser history remain outside DRAFT.

The implementation has no embedded browser, selected browser executable,
network client, persistence, filesystem, background job, retry, polling,
cookie, token, scraping, proxying, interception, or browser automation path.

## Verification

Six Rust domain tests cover valid publisher/institutional URLs, denied schemes
and URL credentials, DOI URL construction, Google Scholar URL construction,
invalid input before launch, and bounded launch failures. Four command tests
cover the standard typed signature, request, response, and error shapes.
Thirteen frontend tests cover every target, exact arguments, destination
mismatch, malformed responses,
all command errors, and bounded transport errors.

`scripts/check-invariants.sh` requires those tests, fixed service origins, the
Rust-only opener dependency, and the typed frontend wrapper. It rejects direct
frontend open APIs, opener plugin registration or capabilities, alternate Rust
launchers, and network or persistence authority in the handoff modules.

No test invokes `SystemBrowser`, opens an application, or performs a network
request. The aggregate verifier runs the same evidence locally and in GitHub
Actions.

## Next Boundary

Phase 24 owns explicit and watched-folder PDF import. It must confirm a watched
file is stable before import and must keep filesystem authority in Rust.
