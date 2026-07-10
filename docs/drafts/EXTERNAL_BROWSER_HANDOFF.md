# External Browser Handoff Requirements Draft

## Status

This is a non-binding Phase 23 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md`. This
draft does not become an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`.

## Purpose

DRAFT must let a user continue research on publisher, institutional, DOI, and
Google Scholar pages without bringing browser sessions or login credentials
inside the application.

## Scope

Phase 23 adds one user-initiated Rust command that opens a validated URL in the
default system browser. The command accepts one of four tagged targets:

```json
{ "destination": "publisher", "url": "https://publisher.example/article" }
{ "destination": "institutional", "url": "https://library.example/resource" }
{ "destination": "doi", "doi": "10.1000/example" }
{ "destination": "google_scholar", "query": "example research title" }
```

Publisher and institutional targets accept a bounded HTTPS URL without URL
userinfo. Rust constructs DOI resolver and Google Scholar search URLs from
bounded validated input. The frontend does not choose those service origins.

## Ownership

Rust owns input validation, URL construction, and the system-browser call.
TypeScript owns only the typed request wrapper and transient result handling.

The Tauri opener crate is called from Rust. DRAFT does not register its guest
plugin, install JavaScript bindings, or grant opener permissions to the
WebView. Tests replace the concrete launcher and never open an application.

## Result and Failures

A successful command reports the opened destination but does not return the
URL. Typed failures distinguish:

- invalid external URL;
- invalid DOI;
- invalid Google Scholar query; and
- failure to start the default system browser.

Errors must not contain raw URLs, queries, dependency errors, credentials,
browser state, or environment details.

## Security Boundary

Phase 23 must not:

- make a network request from DRAFT;
- open `http`, file, shell, custom, `mailto`, or script schemes;
- accept URL usernames or passwords;
- select a specific browser or application;
- register a direct frontend opener capability;
- authenticate, scrape, script, proxy, intercept, or inspect a browser session;
- receive or store publisher, institutional, library, or Google Scholar login
  credentials;
- persist browser URLs, queries, cookies, tokens, or history; or
- add background work, retries, polling, or browser lifecycle tracking.

## Verification

Tests and scans must cover:

- exact tagged request and response serialization;
- rejection of unknown or mismatched request fields;
- HTTPS and URL-userinfo validation;
- deterministic DOI resolver URL construction;
- deterministic Google Scholar query URL construction;
- invalid DOI and query rejection before launch;
- typed browser-launch failure without raw detail;
- exact typed frontend command arguments and result validation;
- no direct frontend opener API or capability;
- no shell process, browser automation, scraping, credential, persistence, or
  network authority in the handoff modules; and
- local/GitHub Actions verification parity.

## Non-Goals

Phase 23 does not add a visible research panel, metadata-search command,
reference persistence, PDF download or import, watched folders, embedded
browser, login form, cookie store, deep linking, analytics, or browser-close
callback.
