# Audit And Diagnostics

## Status

This guide records implemented Phase 38 behavior. The original requirements in
`docs/drafts/AUDIT_DIAGNOSTICS.md` remain non-binding historical input.

## Purpose

Phase 38 adds one local, content-free diagnostic snapshot for supportable
runtime failures. Rust owns the snapshot schema, field sources, ordering,
validation, size limit, and command errors.

This is an internal request boundary. DRAFT has no diagnostics control,
support-bundle export, automatic collection, report persistence, upload, or
support-submission workflow.

## Snapshot Contract

`get_diagnostic_snapshot` accepts an empty request and returns schema version 1
with four exact fields:

| Field | Source | Support purpose |
| :--- | :--- | :--- |
| `schemaVersion` | `DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION` | Lets a later caller reject an unknown report shape. |
| `applicationVersion` | Compiled Cargo package metadata | Identifies the running DRAFT build without reading the environment. |
| `contractVersions` | Six existing Rust constants | Identifies the document, citation, reference, job-store, and helper contracts compiled into the build. |
| `subsystems` | Fixed startup and non-probe states | Distinguishes initialized Rust boundaries from the intentionally unqueried Python helper. |

Contract names and subsystem names are closed enums. Both arrays have six
entries in lexical order. A snapshot serializes to at most
`MAX_DIAGNOSTIC_SNAPSHOT_BYTES`, currently 2,048 bytes.

## Availability Meaning

`ready` means the boundary is a startup prerequisite for a running command or
is managed unconditionally by the Rust runtime. It is not a health check and
does not perform an operation against that subsystem.

The Python helper is `not_checked`. Snapshot assembly does not start a helper
process. Native credential storage is omitted entirely: the command does not
receive a `SecretStore`, query an account, inspect credential presence, or call
the operating-system credential manager.

## Redaction Boundary

The model has no fields for document or evidence content, prompts, findings,
exports, identifiers, paths, URLs, request or response bodies, environment
values, usernames, hostnames, process identifiers, logs, credentials, account
names, or secret values.

The snapshot performs no filesystem, SQLite, network, native credential,
Python, shell, logging, telemetry, event, or background-worker operation. It is
assembled only after an explicit command request and remains in memory.

## Command And Client

Rust registers `get_diagnostic_snapshot` with an exact empty request, a
transparent strict response, and three closed error codes:

- `invalid_application_version`
- `snapshot_serialization_failed`
- `snapshot_too_large`

`src/ipc/diagnosticSnapshot.ts` receives the response as `unknown`. It validates
the schema version, package-version shape, exact array length and ordering,
every closed name/status pair, and positive integer contract versions. Unknown
responses and transport failures remain separate bounded client errors.

No React component or hook imports the wrapper. Phase 39 maps only failures
that already reach visible surfaces; diagnostic errors remain typed and
unwired, with no speculative user action.

## Enforcement

Rust tests prove deterministic exact serialization, the byte limit, closed
application-version failures, and the absence of redacted field categories.
Command tests pin the signature, request, response, and all error codes.
Frontend tests pin exact IPC arguments, strict validation, ordering, closed
statuses, exhaustive known errors, and raw-detail rejection.

`scripts/check-invariants.sh` requires those sources and tests. It rejects new
filesystem, persistence, network, credential-manager, Python-runner, process,
logging, telemetry, event, and worker authority in the diagnostic boundary. A
separate scan prevents any visible frontend consumer.

Run focused evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml diagnostic
npm test -- src/ipc/diagnosticSnapshot.test.ts
bash scripts/check-invariants.sh
```

All behavioral constants are indexed in
`docs/maintainers/CONFIGURATION.md`.

## Ownership Layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `get_diagnostic_snapshot` | Maps one explicit request to the Rust-owned report and closed command errors. |
| Mid | `current_diagnostic_snapshot` | Coordinates fixed metadata assembly and validation. |
| Low | `validate_application_version` | Enforces the compiled-version bound. |
| Low | `validate_serialized_size` | Enforces the complete snapshot byte limit. |
