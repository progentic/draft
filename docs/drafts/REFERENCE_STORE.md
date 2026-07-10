# Reference Store Requirements Draft

**Status:** Draft, non-binding

**Implementation checkpoint:** Phase 17 complete

**Owners:** Rust core

**Related invariants:** `INV-03`, `INV-04`, and `INV-10`

## Purpose

This draft records the requirements used for Phase 17. The implemented
checkpoint is documented in `docs/maintainers/REFERENCE_STORE.md`. This file
does not authorize citation nodes, bibliography rendering, metadata lookup,
PDF import, document-envelope metadata, frontend persistence, or a visible
reference-library workflow.

## Storage Authority and Location

Rust owns the reference store and every SQLite call. Feature code, frontend
code, Python helpers, and Bash runtime code may not open or mutate the database.

The production path policy is:

```text
<Rust-resolved app_data_dir>/references.sqlite3
```

The store accepts a Rust-selected path and creates its parent directory. It
does not ask the frontend for a path. Phase 17 does not initialize the store at
application startup because no current command or visible workflow uses it.

Tests create database files only under the ignored Cargo target directory
inside the repository.

## SQLite Schema Version 1

SQLite `PRAGMA user_version` is the store schema version. Version `0` migrates
transactionally to version `1`; version `1` is verified before use; unknown
versions fail without mutation.

Version 1 contains one strict table:

```sql
CREATE TABLE reference_records (
    reference_id TEXT PRIMARY KEY NOT NULL,
    citekey TEXT NOT NULL UNIQUE COLLATE BINARY,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    payload_json TEXT NOT NULL
) STRICT;
```

The validated JSON payload remains the complete record source of truth. The
three indexed columns support stable identity, case-sensitive citekey
uniqueness, and schema consistency checks. Every read validates that those
columns match the payload.

## CRUD Semantics

The store provides:

- create one validated record
- read by UUID
- read by exact case-sensitive citekey
- list all records in deterministic citekey/UUID order
- update one existing UUID with a complete validated replacement
- delete and return one validated record

Create rejects duplicate UUIDs and duplicate citekeys distinctly. Update keeps
the UUID stable, may change the citekey, and rejects ownership conflicts before
mutation. Missing update/delete targets return `reference_not_found`. Delete
does not remove malformed or invalid stored data silently.

Every write uses an immediate SQLite transaction. Failed conflicts, invalid
stored data, migration failures, and commit failures leave prior rows intact.

## Stored Data Validation

Reads perform this sequence:

1. Read indexed UUID, citekey, schema version, and payload text.
2. Parse payload text as JSON.
3. Validate it through `ReferenceRecord::from_json_value`.
4. Compare indexed schema, UUID, and citekey with the validated payload.
5. Return only the validated `ReferenceRecord`.

Malformed JSON, invalid records, schema mismatches, and identity/citekey index
mismatches are distinct typed failures. Raw SQLite and JSON parser details do
not escape the store boundary.

## Migration Stub

The migration dispatcher has explicit branches for:

- version `0` to version `1`
- current version `1`
- unsupported future versions

The version `0` migration creates the strict table and updates `user_version`
inside one immediate transaction. A database claiming version `1` must expose
the expected table and columns before `ReferenceStore::open` succeeds.

Future migrations must extend this dispatcher and add upgrade/rollback safety
tests. They must not normalize invalid records silently.

## Failure Shape

`ReferenceStoreError` distinguishes storage location, open, schema read,
migration, unsupported/invalid schema, poisoned state, serialization, read,
write, duplicate ID, duplicate citekey, not found, malformed JSON, invalid
record, stored schema mismatch, and stored identity mismatch failures.

Context is bounded to a schema version or nested typed record error. Database
paths, SQL text, raw SQLite errors, and stored payload contents are not exposed.

## Implemented Tests

Phase 17 tests cover:

- production path construction and parent creation
- new schema creation, idempotent reopen, and expected table/constraint verification
- unsupported, malformed, and failed-migration databases
- create/read/reopen, update, delete, citekey lookup, and deterministic list
- duplicate UUID, duplicate citekey, and case-sensitive citekeys
- failed-update transaction preservation
- malformed JSON, invalid records, missing tables, and index mismatches
- concurrent duplicate create and poisoned store state
- stable serialized failure codes and nested causes

## Explicit Phase 17 Non-goals

Phase 17 does not add:

- Tauri commands, runtime store registration, TypeScript IPC, or React state
- citation nodes, citation rendering, or bibliography consistency
- Crossref, Semantic Scholar, Unpaywall, or other network calls
- PDF import, watched folders, or metadata extraction
- document-envelope reference fields or embedded metadata
- merge, deduplication, reliability scoring, or manual-edit UI

## Promotion Gate

This draft records Phase 17 requirements but may not move to `docs/contracts/`
until it satisfies the proposal, review, stability, frontmatter, and acceptance
requirements in `GOVERNANCE.md` section 7.
