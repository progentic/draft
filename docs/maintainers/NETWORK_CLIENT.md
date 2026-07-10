# Network Client

## Status

This is an implemented Phase 21 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in `docs/drafts/NETWORK_CLIENT.md` until
the contract lifecycle is complete.

## Scope

`src-tauri/src/network/client.rs` owns the only configured outbound HTTP client
in product source. Phase 21 constructs that client, registers it as Tauri-managed
Rust state, and exposes no request operation.

Construction performs no DNS lookup, socket connection, provider request,
retry, background work, persistence, or frontend IPC. Provider-specific
metadata lookup begins in Phase 22.

## Dependency and TLS

DRAFT directly pins `reqwest` `0.13.4` with default features disabled and only
the `rustls` feature enabled. This avoids implicit cookie and system-proxy
features while providing platform certificate verification through the locked
Rustls dependency graph.

The builder sets `https_only(true)`. A future provider cannot use this client
for plain HTTP without an explicit policy change.

No second HTTP library or ad hoc client constructor exists in product source.

## Construction Policy

`NetworkClient::new` derives policy from the Cargo package version compiled
into the Rust core. The version is trimmed and must contain only ASCII letters,
digits, period, hyphen, or plus. Invalid version metadata fails before client
construction.

The controlled User-Agent is:

```text
DRAFT/<version> (+https://github.com/progentic/draft)
```

The current timeout policy is:

| Setting | Value | Purpose |
| :--- | :--- | :--- |
| Connect timeout | 10 seconds | Bounds connection establishment. |
| Request timeout | 30 seconds | Bounds one complete future request. |

The constants are named and documented in source. Provider-specific retry,
backoff, rate-limit queues, and offline behavior remain Phase 22 work.

## Runtime Ownership

`application::network_client::initialize_network_client` constructs one client
during Tauri setup and registers it with `app.manage`. Startup fails closed if
construction fails.

The managed type retains the configured `reqwest::Client` privately. It has no
raw-client accessor and no request method at this checkpoint. Phase 22 must add
only the minimum internal operation required by real provider modules.

The frontend and Python package receive no network object, URL, response, or
transport authority.

## Failure Shape

`NetworkClientError` is bounded to:

- `InvalidApplicationVersion`; and
- `ClientBuildFailed`.

The error does not retain or expose raw `reqwest` details, environment values,
URLs, paths, document content, or credentials. No error crosses IPC in Phase
21 because no network command exists.

## Security Boundaries

Phase 21 deliberately excludes:

- cookie-store support or browser-session state;
- publisher, institutional, library, or Google Scholar credentials;
- proxy configuration owned by a feature module;
- request or response logging;
- frontend or Python network calls;
- HTTP startup probes, telemetry, analytics, or update checks; and
- an external request in any test.

The existing credential, frontend, Python, and Bash scans continue to run
unchanged.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `initialize_network_client` | Registers one shared client during trusted startup. |
| Mid | `NetworkClient::new` | Coordinates controlled policy and transport construction. |
| Mid | `network_client_policy` | Defines User-Agent and timeout policy. |
| Low | `validated_application_version` | Validates controlled metadata characters. |
| Low | `reqwest::Client::builder` | Applies TLS and transport mechanics. |

## Verification

Five Rust tests cover:

- current manifest metadata construction;
- deterministic User-Agent policy;
- explicit connect and request timeouts;
- malformed application versions; and
- bounded failure messages.

`scripts/check-invariants.sh` requires the source, tests, application
initializer, managed-state registration, exact direct dependency features,
HTTPS-only configuration, named timeout constants, and all five tests. It
rejects request execution, cookie configuration, raw `reqwest` errors, and any
`reqwest` use outside `src-tauri/src/network/`.

`scripts/check-repository.sh` requires every Phase 21 source to remain visible
to Git. `scripts/check-docs.sh` requires this guide and roadmap/phasemap
agreement through Phase 21. The aggregate verifier runs the same checks locally
and in GitHub Actions.

## Phase 22 Gate

Phase 22 may add typed Crossref, Semantic Scholar, and Unpaywall metadata
lookup through this client. It must define provider-specific request and
response shapes, rate-limit, timeout, offline, and malformed-response behavior
before implementation. It must not add scraping, browser automation,
credential capture, or a direct frontend network path.
