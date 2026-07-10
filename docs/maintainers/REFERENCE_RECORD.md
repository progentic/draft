# Reference Record

## Status

This is an implemented Phase 16 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/REFERENCE_RECORD.md` until the contract lifecycle is complete.

## Scope

Phase 16 defines and validates one in-memory version 1 reference record in
`src-tauri/src/references/record.rs`. Rust is the only validation authority.
No command, TypeScript mirror, React state, reference store, database, file,
network client, import path, citation node, bibliography, or document-envelope
field is added.

The record contains these declared top-level fields:

```text
schema_version
reference_id
citekey
kind
title
contributors
issued
container_title
publisher
volume
issue
pages
resolution_state
identifiers
provenance
```

Every field is required in the serialized object. Nullable bibliographic
values remain explicit `null` values, and absent identifier lists remain empty
arrays. Unknown top-level and nested fields fail validation.

## Rust Domain Types

`ReferenceRecord` composes three flat serialized groups:

- identity: schema version, UUID, citekey, kind, and title
- bibliography: contributors, partial date, container, publisher, volume,
  issue, and pages
- tracking: resolution state, identifiers, and provenance

The public domain surface is limited to:

- `ReferenceId`
- `ReferenceRecord`
- `ReferenceRecordError`
- `REFERENCE_RECORD_SCHEMA_VERSION`

Reference kind, contributor, date, identifier, resolution, source, override,
and provenance representation types remain module-private so Rust callers
cannot construct invalid fragments around the aggregate validator. Record
fields remain private after validation. `ReferenceRecord::from_json_value` is
the untrusted JSON entry point, and custom Serde deserialization routes through
the same validator.

## Validation Order

Validation is deterministic:

1. Require a top-level object and reject unknown fields.
2. Require integer `schema_version` equal to `1`.
3. Parse `reference_id` as a UUID.
4. Validate the ASCII citekey shape.
5. Validate the reference kind and non-blank title.
6. Validate every contributor and tagged person/organization name.
7. Validate the nullable partial issued date.
8. Validate nullable bibliographic text fields.
9. Validate resolution state and identifiers.
10. Validate provenance, manual-source consistency, and unique manual
    overrides.

The citekey validator accepts an ASCII letter or digit first, followed by
ASCII letters, digits, colon, underscore, or hyphen. It does not enforce
library-wide uniqueness; that requires the Phase 17 store.

## Bibliographic Shape

Version 1 supports these reference kinds:

```text
article
book
chapter
report
thesis
webpage
other
```

Contributors may be authors, editors, or translators. Person names require at
least one non-blank given or family value. Organization names require one
non-blank literal value. Anonymous works may use an empty contributor list.

An issued date is `null` or contains a year from 1 through 9999 plus nullable
month and day values. A day without a month is invalid. Phase 16 validates
shape and bounds, not metadata truth or calendar provenance.

DOI and URL values are nullable non-blank strings. ISBN values are a list of
non-blank strings. Phase 16 does not normalize, resolve, query, or verify any
identifier.

## Provenance

The declared source is one of:

```text
manual
crossref
semantic_scholar
unpaywall
pdf_import
```

Manual provenance requires a null source-record ID and an empty manual-override
list. Non-manual provenance may mark normalized bibliographic fields as manual
overrides. Duplicate or unknown override fields fail with their list index.

`resolution_state` is `unresolved`, `resolved`, or `needs_review`. It is stored
as descriptive data only. Phase 16 performs no transition, lookup, merge, or
reliability scoring, and `resolved` does not claim that metadata is verified.

## Failure Shape

`ReferenceRecordError` is a bounded tagged enum. It distinguishes:

- non-object input and unknown field paths
- missing, malformed, and unsupported schema versions
- missing or malformed UUID and citekey identity
- unsupported kind and invalid title
- missing or malformed contributors and contributor indexes
- missing or malformed partial dates
- missing or malformed nullable bibliographic fields
- missing or malformed identifiers and ISBN indexes
- missing or malformed resolution and provenance data
- unsupported provenance sources and invalid manual-override indexes

Errors include only controlled field paths, unsupported schema integers, or
list indexes. Rejected user values and raw parser details do not cross the
domain boundary.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `ReferenceRecord::from_json_value` | Coordinate one complete record validation. |
| Mid | `parse_identity` / `parse_bibliography` / `parse_tracking` | Build the three domain groups. |
| Mid | contributor, date, identifier, and provenance parsers | Enforce nested domain rules. |
| Low | map, text, UUID, number, and citekey helpers | Perform primitive JSON and string checks. |

## Verification

Twenty-four Rust tests cover:

- minimum deserialization, stable serialization, and full round trips
- person and organization contributors
- absent, partial, and complete issued dates
- Unicode bibliographic content
- every accepted kind, contributor role, resolution state, provenance source,
  and manual-override field
- fully nullable bibliography and identifier values
- every missing top-level field
- non-object and unknown top-level/nested input
- malformed and unsupported schema versions
- invalid identity, citekeys, kinds, titles, contributors, dates, optional
  fields, identifiers, resolution states, and provenance
- every serialized error code plus contextual path, version, and index fields

`scripts/check-invariants.sh` requires the source, schema constant, and all 24
named tests. It rejects filesystem, synchronization, database, and Tauri command
APIs in the reference module, rejects frontend reference authority, and keeps
the Phase 17 reference-store absence gate active.

The same checks run through `scripts/verify.sh` locally and in the GitHub
Actions `verify` job. `scripts/check-repository.sh` requires the reference
module to remain visible to Git.

## Phase 17 Gate

Phase 17 may persist validated records in a Rust-owned local store and enforce
library-wide uniqueness. It must define CRUD failure behavior, transaction
authority, storage location, schema initialization, and a migration stub before
the current store-absence gate is replaced.

Phase 17 must not add citation nodes, bibliography rendering, network lookup,
PDF import, or document-envelope metadata embedding.
