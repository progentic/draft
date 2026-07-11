# Data Migration

## Status

This is the implemented Phase 43 migration baseline. It records how persisted
document and reference data are versioned, rejected, and eventually migrated.
It does not define a legacy transform because DRAFT has no released schema
older than version 1.

## Current Baseline

| Persisted surface | Current version | Phase 43 behavior |
| :--- | :--- | :--- |
| Document envelope | 1 | Version 1 validates; lower, malformed, missing, and future versions fail before registry insertion or source writes. |
| Citation attrs inside documents | 1 | Version 1 validates; lower, malformed, missing, and future versions make the containing envelope fail closed. |
| Reference-record payload | 1 | Version 1 validates; lower, malformed, missing, and future versions fail before a record is returned. |
| Reference-store schema | 1 | SQLite version 0 initializes an empty store transactionally; version 1 is verified; every other version fails closed. |

SQLite `user_version = 0` means an uninitialized store only when the expected
tables are absent. It is not a legacy data schema. A conflicting version 0
database fails and the initialization transaction rolls back.

The PDF import-job store has its own versioned lifecycle contract. Phase 43
does not change that schema because the roadmap gate is limited to document and
reference data.

## Read Policy

Persisted input is untrusted. A read must identify the declared version and
dispatch only to the exact current validator or a named migration step.
Unknown older versions are not guessed. Future versions are never downgraded.

The current version 1 baseline has no migration step:

- document open reads and validates without modifying source bytes;
- failed document open creates no live registry handle;
- reference reads validate the stored JSON and its indexed identity;
- failed reference reads do not update, delete, or normalize the row; and
- typed errors contain version numbers and categories, not source content.

## Future Migration Rule

A later schema change must add an explicit step for each supported transition,
such as `1 -> 2`. Every step must:

1. inspect the source version before mutation;
2. transform one known version to the next in memory;
3. validate the complete transformed value with the destination schema;
4. preserve document source bytes until an atomic replacement succeeds;
5. keep reference rows and schema metadata in one immediate SQLite transaction;
6. roll back completely on transformation, validation, write, or commit failure;
7. reject skipped, unknown, and future versions with a typed error; and
8. have fixtures proving success, rollback, idempotent reopen, and source preservation.

Nested citation migrations belong to the document migration that owns the
containing envelope. A citation version must never be silently rewritten during
rendering, analysis, formatting, save, or export.

## Explicit Exclusions

Phase 43 adds no automatic repair, backup manager, downgrade path, startup
rewrite, migration command, frontend control, release import workflow, or
reference-library UI. No user-facing migration workflow exists.

## Verification

Focused Rust tests prove that document versions 0 and 2 leave source bytes and
the registry unchanged, citation versions 0 and 2 fail validation, and stored
reference payload versions 0 and 2 leave the SQLite row unchanged. Existing
store tests prove transactional empty-store initialization, conflicting-schema
rollback, current-schema verification, and rejection of unknown store versions.

`scripts/check-invariants.sh` pins these tests and the exact version-dispatch
markers. `scripts/check-docs.sh` requires this guide and its owning subsystem
cross-links. Both run through `scripts/verify.sh` locally and in GitHub Actions.
