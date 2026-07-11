# Audit And Diagnostics Requirements Draft

## Status

This is the non-binding requirements draft for Phase 38. It defines the next
implementation boundary after Phase 37. It is not an accepted contract under
`docs/GOVERNANCE.md`.

## Purpose

DRAFT needs bounded local diagnostics that help a user or maintainer understand
supportable failures without exposing document content, evidence, secrets,
credential presence, filesystem paths, native error details, or external
service payloads.

## Scope

Phase 38 adds a Rust-owned diagnostic snapshot assembled on explicit request.
The snapshot may report fixed application/runtime versions, closed subsystem
availability categories, schema versions, and bounded typed failure codes that
already exist. Every field must have a documented support purpose.

The diagnostic model is local data. It does not transmit, upload, synchronize,
or automatically persist a report. A visible export or support-submission flow
is outside Phase 38 unless separately bounded before implementation.

## Redaction Contract

The snapshot must not include:

- document titles, text, citations, evidence, prompts, findings, or exports;
- secret values, identifiers, account names, existence checks, or native
  credential-manager details;
- source, destination, database, temporary, or application-data paths;
- URLs, DOI values, provider responses, request bodies, or response bodies;
- raw Rust, Tauri, Python, SQLite, keyring, operating-system, or transport
  errors; or
- environment variables, usernames, hostnames, process identifiers, or logs.

Unknown values remain explicit closed states rather than guessed details.

## Resource And Authority Limits

The snapshot has fixed field and byte limits, deterministic ordering, and no
unbounded collection. Rust owns assembly and validation. Python, Bash, and the
frontend cannot inspect trusted subsystem state directly.

Phase 38 adds no telemetry, crash reporter, background collector, log upload,
network request, support account, secret probe, document scan, arbitrary file
read, or shell command.

## Acceptance Tests

Phase 38 must prove:

- the diagnostic schema is strict, versioned, bounded, and deterministic;
- every included field has a fixed source and closed value set;
- redacted categories cannot enter the model or serialized output;
- failures produce typed bounded results without raw details;
- secret presence and secret-store operations are never queried;
- no network, filesystem, Python, Bash, telemetry, or background authority is
  added; and
- local and GitHub Actions verification run the same tests and scans.

## Non-Goals

Phase 38 does not complete the Phase 39 error UX, add a support bundle, expose
application logs, collect performance traces, submit a bug report, upload a
crash report, or create a user-visible diagnostics workflow without a separate
bounded interaction contract.
