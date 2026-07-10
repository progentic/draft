# Network Client Requirements Draft

## Status

This document is a non-binding Phase 21 requirements draft. Implemented
behavior must be recorded separately in `docs/maintainers/NETWORK_CLIENT.md`
once the phase is complete. This draft does not become an accepted contract
without the review lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 21 owns the first centralized Rust network client and its construction
policy. It establishes the only production module allowed to construct an
external HTTP client.

Phase 21 does not implement Crossref, Semantic Scholar, Unpaywall, publisher,
institutional, or Google Scholar workflows. Provider lookup, response parsing,
rate-limit behavior, offline classification, and user-visible commands begin
in later owning phases.

## Ownership

Rust owns the network client, transport configuration, and future outbound
request boundary. The frontend and Python helpers receive no HTTP client,
credential, URL-fetching, or direct external-service authority.

The production client belongs under:

```text
src-tauri/src/network/
```

No feature module may construct its own HTTP client. Bash remains local and CI
orchestration only.

## Client Construction

Phase 21 must provide one concrete Rust-owned client type with:

- one centralized underlying HTTP client construction path;
- a bounded construction error type;
- a DRAFT User-Agent derived from controlled application metadata;
- explicit transport timeout configuration;
- no cookie store or browser-session integration;
- no proxying of publisher or institutional credentials; and
- Rust-managed application state when the desktop runtime needs shared access.

The implementation must not add a provider plugin system, generic service
registry, or speculative request abstraction before a real provider workflow
exists.

## Request Boundary

Phase 21 may expose only the minimum internal API needed to prove centralized
ownership. It must not issue an external request during startup or tests.

Future provider modules must receive the centralized client rather than create
their own transport. Phase 22 will define typed provider request, response,
timeout, rate-limit, offline, and malformed-response behavior.

## Security and Privacy

The client must not:

- accept publisher, institutional, library, or Google Scholar credentials;
- enable a browser cookie jar;
- log user document text, secrets, response bodies, or sensitive query data;
- bypass the external browser handoff assigned to Phase 23; or
- expose raw network objects to TypeScript or Python.

Adding API credentials or OS-native secret storage is outside Phase 21 and
remains governed by later phases.

## Failure Behavior

Client construction failures must be typed and bounded. Raw dependency errors,
URLs, filesystem paths, environment values, or secrets must not cross an IPC
boundary.

No Tauri command is required in Phase 21. If no command exists, the failure
remains an internal startup or construction error and must fail closed.

## Verification

Phase 21 tests and scans must cover:

- centralized client construction in the accepted network module;
- deterministic User-Agent policy;
- explicit timeout policy;
- typed construction failure behavior where injectable;
- absence of external requests during construction and tests;
- absence of ad hoc Rust clients outside the network module;
- absence of frontend and Python network authority; and
- local/GitHub Actions parity.

The Phase 20 network-client absence gate must be replaced by these behavioral
checks when implementation begins.

## Explicit Non-Goals

Phase 21 does not add:

- provider-specific metadata lookup;
- rate-limit queues, retry loops, or exponential backoff;
- offline detection or reconnect polling;
- external browser handoff;
- PDF import or watched folders;
- API-key storage or another secret surface;
- frontend network commands or search UI;
- background jobs, AI calls, or Python helper networking; or
- release telemetry, analytics, or update checks.
