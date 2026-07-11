# Network Client

## Status

This is an implemented Phase 22 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in `docs/drafts/NETWORK_CLIENT.md` until
the contract lifecycle is complete.

## Scope

`src-tauri/src/network/client.rs` owns the only configured outbound HTTP client
and the only request execution in product source. Phase 21 constructed the
client; Phase 22 adds the bounded metadata request operation used by three
provider modules. No frontend IPC, persistence, or background work is added.

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

The client also enforces a one-second interval per provider, exponential 429
backoff capped at 60 seconds, bounded `Retry-After` seconds, and a one-megabyte
response limit.

## Runtime Ownership

`application::network_client::initialize_network_client` constructs one client
during Tauri setup, creates one shared Phase 36 `ConnectivityPolicy`, and
registers both with `app.manage`. Startup fails closed if construction fails.

The managed type retains the configured `reqwest::Client` privately and has no
raw-client accessor. `get_metadata` is the single bounded request operation for
Rust provider modules.

The frontend and Python package receive no network object, URL, response, or
transport authority.

`get_metadata` requires the shared session to be online before rate
reservation, URL validation, request construction, or socket work. Explicit
offline mode returns `NetworkRequestError::Offline`. A connection failure maps
to the same typed request category but does not change the selected mode.

## Failure Shape

`NetworkClientError` is bounded to:

- `InvalidApplicationVersion`; and
- `ClientBuildFailed`.

The error does not retain or expose raw `reqwest` details, environment values,
URLs, paths, document content, or credentials. No metadata error crosses IPC
because no metadata command exists. The separate connectivity commands expose
only the closed effective mode and `connectivity_unavailable`.

## Security Boundaries

The current boundary excludes:

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

Fourteen Rust tests cover construction plus request policy:

- current manifest metadata construction;
- deterministic User-Agent policy;
- explicit connect and request timeouts;
- malformed application versions; and
- bounded failure messages;
- independent provider intervals and capped backoff;
- typed transport and status failures; and
- bounded response accumulation;
- offline denial before URL or transport work; and
- unchanged online URL validation.

`scripts/check-invariants.sh` requires the source, tests, application
initializer, managed-state registration, exact direct dependency features,
HTTPS-only configuration, named timeout and response constants, and all twelve
tests. It confines request execution to the centralized client, rejects cookie
configuration, and rejects `reqwest` use outside `src-tauri/src/network/`.

`scripts/check-repository.sh` requires every network source to remain visible
to Git. `scripts/check-docs.sh` requires this guide and roadmap/phasemap
agreement through Phase 23. The aggregate verifier runs the same checks locally
and in GitHub Actions.

Phase 36 session policy is documented in
`docs/maintainers/OFFLINE_MODE.md`.

## Browser Handoff Boundary

Phase 23 browser handoff is implemented separately in
`docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`. Launching the user's default
browser is not a request through this client: DRAFT neither performs nor
observes the browser's network work. Automated DRAFT requests remain confined
to this module.

## Configuration Index

Network timeouts, rate limits, response bounds, and provider origins are indexed
in `docs/maintainers/CONFIGURATION.md`.
