# Reference Store

## Status

This is an implemented Phase 17 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/REFERENCE_STORE.md` until the contract lifecycle is complete.

## Scope

`src-tauri/src/references/store.rs` is the only production SQLite boundary for
reference records. It persists only validated `ReferenceRecord` values and
validates stored data again on every read.

Phase 18 registers the store as managed Tauri state during desktop startup.
Rust resolves the Tauri application-data directory and opens the database
before command handling begins. Startup fails closed when location, migration,
or schema verification fails.

`resolve_citation` performs exact citekey resolution and returns no record
metadata. Phase 46 adds bounded manual create and summary-list commands. The
frontend receives only citekey and title; update, delete, full-record, bulk
import, and synchronization commands remain absent.

## Dependency and Location

DRAFT pins `rusqlite` `0.40.1` with bundled SQLite. The bundled feature avoids
depending on a platform-specific system SQLite installation.

`reference_store_path` joins the stable filename `references.sqlite3` to a
Rust-resolved application-data directory. `ReferenceStore::open` creates the
parent directory, opens the connection, configures a five-second busy timeout,
migrates the schema, and verifies the current table before returning a handle.

Tests use UUID-isolated paths under:

```text
src-tauri/target/reference-store-tests/
```

The directory is ignored build output inside the repository.

## Schema and Migration

`REFERENCE_STORE_SCHEMA_VERSION` is `1`. SQLite `PRAGMA user_version` owns the
store schema version independently from the record payload schema version.

The version 1 `reference_records` table is strict and contains:

| Column | Role |
| :--- | :--- |
| `reference_id` | Primary key and validated UUID index. |
| `citekey` | Case-sensitive unique lookup index. |
| `schema_version` | Checked version 1 payload index. |
| `payload_json` | Complete serialized `ReferenceRecord` source data. |

`migrate_store` dispatches version `0` to `migrate_zero_to_one`, accepts
verified version `1`, and rejects every unknown version. Initialization runs in
an immediate transaction. A claimed current schema without the expected table
and columns returns `invalid_store_schema`.

Phase 43 clarifies that version 0 means an empty, uninitialized SQLite store,
not an older record-data schema. A conflicting version 0 database rolls back
and fails. Stored payload versions outside the current version fail without row
mutation. The complete policy is in `docs/maintainers/DATA_MIGRATION.md`.

## Transaction Boundary

`ReferenceStore` owns one `Mutex<Connection>`. Every create, update, and delete
holds that process-local lock and an immediate SQLite transaction through
conflict checks, row mutation, and commit.

The mutex serializes callers sharing one store. SQLite's immediate transaction
also acquires write authority before conflict checks, so separate connections
cannot race a citekey/UUID check with mutation. A five-second busy timeout
returns a typed write failure instead of waiting indefinitely.

## CRUD Behavior

| Method | Behavior |
| :--- | :--- |
| `create` | Inserts one validated record; distinguishes duplicate UUID and citekey. |
| `get` | Reads by UUID and returns `None` when absent. |
| `get_by_citekey` | Reads by exact case-sensitive citekey. |
| `list` | Returns validated records ordered by citekey then UUID. |
| `update` | Replaces one existing UUID and may change its citekey. |
| `delete` | Validates, deletes, and returns one record; missing is typed. |

All write inputs are complete validated records. The store does not patch
individual JSON fields or implement metadata merge behavior.

## Read Validation

Stored data is never trusted because it came from SQLite. Reads parse payload
JSON, route it through `ReferenceRecord::from_json_value`, then compare the
indexed schema version, UUID, and citekey with the validated payload.

Malformed payloads, invalid records, schema-index mismatches, and
identity/citekey mismatches fail closed. Delete validates before mutation, so a
corrupt row is not silently removed.

## Failure Shape

`ReferenceStoreError` is a bounded serialized enum:

- `storage_location_unavailable`
- `open_failed`
- `schema_read_failed`
- `schema_migration_failed`
- `unsupported_store_schema` with `found`
- `invalid_store_schema`
- `store_unavailable`
- `serialization_failed`
- `write_failed`
- `read_failed`
- `duplicate_reference_id`
- `duplicate_citekey`
- `reference_not_found`
- `malformed_stored_json`
- `invalid_stored_record` with typed record `cause`
- `stored_schema_mismatch`
- `stored_identity_mismatch`

No path, SQL, raw SQLite error, or payload content is serialized in a failure.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `ReferenceStore` CRUD methods | Coordinate one complete store operation. |
| Mid | conflict, migration, load, and decode helpers | Enforce transaction and stored-data policy. |
| Low | `rusqlite` connection, transaction, query, and row helpers | Perform SQLite mechanics. |

## Verification

Twenty-seven Rust tests cover path policy, schema initialization, idempotent
reopen, unsupported/invalid/malformed schema states, migration failure, full
CRUD, persistence after reopen, case-sensitive uniqueness, conflict rollback,
deterministic listing, corrupt stored data, indexed/payload mismatch,
concurrent create, poisoned state, and stable typed failures.

`scripts/check-invariants.sh` requires the store source, schema constant,
bundled dependency, transaction/schema primitives, and all 27 named tests. It
rejects SQLite use outside the store boundary and rejects Tauri APIs inside the
store. Phase 18 replaces the citation absence gate with behavior checks. Phase
19 replaces the bibliography absence gate with a pure consistency check;
network, import, job, and helper gates remain active.

`scripts/check-repository.sh` requires the production store and test support to
remain visible to Git. The same checks run through `scripts/verify.sh` locally
and in the GitHub Actions `verify` job.

## Phase 18 Integration

Phase 18 defines the versioned Tiptap citation node and exact store-backed
resolution path documented in `docs/maintainers/CITATION_NODE.md`. It does not
change the store schema, expose CRUD, embed full record metadata, add network
lookup, or turn display markers into source data.

Phase 19 does not call `ReferenceStore::list`, because the complete shared
library is not one document's bibliography. A future caller must select the
candidate records explicitly before invoking the consistency module documented
in `docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md`.

## Configuration Index

Database filename, schema version, and SQLite busy timeout are indexed in
`docs/maintainers/CONFIGURATION.md`.
