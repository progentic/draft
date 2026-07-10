# Metadata Lookup Requirements Draft

## Status

This document is a non-binding Phase 22 requirements draft. Implemented
behavior must be recorded separately in `docs/maintainers/METADATA_LOOKUP.md`
once the phase is complete. This draft does not become an accepted contract
without the review lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 22 owns DOI metadata lookup through the Phase 21 centralized Rust network
client for:

- Crossref REST API v1;
- Semantic Scholar Academic Graph API v1; and
- Unpaywall REST API v2.

The phase provides Rust domain APIs and normalized metadata only. It does not
add a Tauri command, search UI, reference-store mutation, automatic merge,
background job, API-key storage, browser handoff, PDF import, or scraping.

Official provider references:

- <https://www.crossref.org/documentation/retrieve-metadata/rest-api/>
- <https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication/>
- <https://www.semanticscholar.org/product/api/tutorial>
- <https://unpaywall.org/products/api>

## Input Boundary

Phase 22 accepts:

- one validated DOI; and
- one validated contact email for Crossref polite-pool identification and the
  required Unpaywall `email` query parameter.

DOIs are normalized to lowercase and must use a bounded printable ASCII
`10.<registrant>/<suffix>` shape. The contact email is request identification,
not a publisher credential or API secret. It is not persisted in this phase.

Semantic Scholar requests are anonymous. API-key integration is deferred until
the OS-native secret boundary exists.

## Endpoints

Phase 22 uses exact DOI lookup only:

```text
GET https://api.crossref.org/v1/works/<doi>?mailto=<email>
GET https://api.semanticscholar.org/graph/v1/paper/DOI:<doi>?fields=<bounded-fields>
GET https://api.unpaywall.org/v2/<doi>?email=<email>
```

URLs must be built with the `url` parser. Provider modules must not concatenate
or percent-encode untrusted input manually.

## Normalized Result

Each provider response is reduced to one `MetadataRecord` containing:

- provider;
- provider record ID;
- DOI;
- title;
- author display names;
- optional publication year;
- optional venue; and
- optional open-access URL.

The normalized result is candidate metadata. It is not a `ReferenceRecord`, is
not persisted, and cannot replace manually curated data without a later
explicit merge workflow.

Raw provider payloads, abstracts, citation counts, and full-text content are not
retained.

## Network Behavior

All requests route through `NetworkClient`. The client adds:

- a one-second minimum interval per provider;
- independent per-provider rate state;
- exponential backoff after HTTP 429, capped at 60 seconds;
- `Retry-After` seconds handling when present;
- a one-megabyte response-body limit; and
- existing connect and request timeouts.

Rate limiting is process-local. Persistent job retry state remains Phase 26
work.

## Typed Failures

Lookup failures must distinguish:

- invalid DOI or contact email before network work;
- not found;
- local or remote rate limiting with bounded retry duration;
- timeout;
- offline/connection failure;
- access denied;
- provider unavailable;
- rejected request;
- oversized response;
- response read failure; and
- malformed or incomplete provider JSON.

Raw URLs, response bodies, dependency errors, document text, contact email, and
credentials must not appear in the error value.

## Verification

Tests and scans must cover:

- DOI and contact-email validation;
- exact provider URL and query construction;
- normalization of representative official response shapes;
- missing and malformed required response fields;
- not-found, rate-limit, timeout, offline, access, service, response-size, and
  malformed-response mappings;
- independent per-provider request intervals;
- exponential rate-limit backoff and cap;
- bounded response accumulation;
- no HTTP client outside `src-tauri/src/network/`;
- no external requests in tests; and
- local/GitHub Actions parity.

## Explicit Non-Goals

Phase 22 does not add:

- title or keyword search;
- batch or bulk endpoints;
- Semantic Scholar API keys;
- Crossref Metadata Plus credentials;
- reference creation, update, merge, or reliability scoring;
- frontend search or import controls;
- browser automation, publisher access, or institutional credentials;
- PDF download, watched folders, or full-text parsing;
- persistent jobs, retries across restart, or background scheduling; or
- provider response caching.
