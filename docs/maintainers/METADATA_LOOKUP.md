# Metadata Lookup

## Status

This guide records implemented Phase 22 behavior. The requirements draft in
`docs/drafts/METADATA_LOOKUP.md` remains non-binding until it completes the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Rust can retrieve one DOI record from Crossref, Semantic Scholar, or Unpaywall.
Each provider returns a normalized `MetadataRecord` candidate with a provider
identifier, DOI, title, authors, optional year, optional venue, and optional
HTTPS open-access URL.

Candidate metadata is not persisted and is not a `ReferenceRecord`. Phase 22
adds no Tauri command, frontend workflow, automatic merge, PDF import, browser
automation, API-key storage, background job, or cache.

## Input Boundary

`Doi` accepts a bounded printable ASCII `10.<registrant>/<suffix>` value and
normalizes it to lowercase. `ContactEmail` accepts a bounded ASCII address and
normalizes it to lowercase. Crossref and Unpaywall use the contact address only
for provider request identification; Semantic Scholar requests are anonymous.

Provider URLs are built with `url::Url`. Lookup is limited to the documented
Crossref REST v1, Semantic Scholar Academic Graph v1, and Unpaywall v2 APIs.

## Network Policy

Every lookup calls `NetworkClient::get_metadata`. The centralized client owns:

- one request per provider per second;
- independent process-local provider state;
- exponential HTTP 429 backoff capped at 60 seconds;
- bounded `Retry-After` seconds handling;
- a one-megabyte response limit;
- existing 10-second connect and 30-second request timeouts; and
- typed status and transport failures.

Only `src-tauri/src/network/client.rs` executes an HTTP request. Provider
modules build URLs and normalize response JSON; they do not construct clients.

## Failure Shape

`MetadataLookupError` distinguishes not found, rate limited, timeout, offline,
access denied, provider unavailable, rejected request, oversized response,
read failure, unavailable client state, and invalid provider response. Errors
do not carry raw URLs, response bodies, contact addresses, dependency errors,
credentials, or document text.

## Verification

Rust tests cover input validation, exact provider URLs, representative response
normalization, DOI mismatch, malformed data, rate intervals, backoff, status
mapping, transport mapping, and response-size enforcement. Tests do not perform
external requests.

`scripts/check-invariants.sh` requires those tests and endpoints, confines
request execution to the centralized client, rejects provider persistence or
IPC authority, and keeps direct network libraries out of feature modules.
Repository and documentation checks require every Phase 22 source and guide.

## Next Boundary

Phase 23 implements the separate system-browser handoff documented in
`docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`. Metadata candidates remain
non-persistent and are not exposed through IPC at this checkpoint.

## Configuration Index

Provider origins, DOI/contact bounds, and network limits are indexed in
`docs/maintainers/CONFIGURATION.md`.
