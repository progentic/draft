# Document Envelope

## Status

This guide records the implemented envelope boundary from Phases 11 and 47. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/DOCUMENT_ENVELOPE.md` until the contract lifecycle is complete.

## Scope

The envelope gives DRAFT one validated document shape before registry,
persistence, or export code can act. Direct parsing performs no filesystem or
registry work. Persisted loading uses a separate migration entry point.

The current version 2 JSON shape is:

```json
{
  "schema_version": 2,
  "document_id": "00000000-0000-4000-8000-000000000001",
  "title": "Untitled document",
  "document": {
    "type": "doc",
    "content": [
      { "type": "paragraph" }
    ]
  }
}
```

The envelope has no path, timestamps, reference records, top-level citation
metadata, analysis output, formatting findings, or export state. Versioned
citation-node attrs may exist only inside Tiptap document content. Unknown
top-level fields fail validation instead of becoming implicit schema extensions.

The JSON example is the fixed Phase 46 New-document result. The broader version
2 validator still accepts other valid Tiptap document content. Text imports use
the same envelope shape with a Rust-generated ID, source filename as display
title, and literal paragraph content; they add no path field.

## Rust Authority

`src-tauri/src/documents/envelope.rs` is the validation authority. The module
provides:

- `DOCUMENT_ENVELOPE_SCHEMA_VERSION`, fixed at `2`
- `DocumentEnvelope`, whose fields are private after validation
- `DocumentId`, a Rust-parsed UUID identity
- `DocumentEnvelopeError`, a bounded serializable failure enum
- `DocumentEnvelope::from_json_value`, the untrusted JSON entry point
- `DocumentEnvelope::from_persisted_json_value`, the only v1 migration entry point

Phase 11 did not expose the envelope to TypeScript. Phase 13 adds a mirrored
response guard and request type under `src/ipc/` because open/save commands now
cross the bridge. The mirror protects UI code from malformed responses; it
does not replace Rust validation.

## Validation Order

Validation is deterministic:

1. Require a top-level JSON object.
2. Reject unknown top-level fields.
3. Require integer `schema_version` equal to `2`.
4. Require `document_id` to parse as a UUID.
5. Require `title` to be a string that is non-empty after trimming.
6. Require `document` to be an object with `type: "doc"`.
7. Require `document.content` to be an array.
8. Validate every nested `citation` node through the Phase 18 Rust contract.
9. Require every nested mark to be an object with a string `type`, then validate
   every `fontFamily` and `fontSize` mark through the bounded text-format
   contract.
10. Validate every explicit paragraph style through the closed Phase 47 model.

Other nested document JSON remains opaque data. Basic mark shape, citation
validation, and the two font attrs are narrow exceptions; they do not turn the
envelope into a general Tiptap schema validator.

## Failure Shape

Failures serialize as tagged objects with a stable snake-case `code`.
Field-specific context is included where the caller needs it.

| Failure | Meaning |
| :--- | :--- |
| `invalid_envelope_object` | The top-level value is not an object. |
| `unknown_envelope_field` | An undeclared top-level field is present; includes `field`. |
| `missing_schema_version` | `schema_version` is absent. |
| `invalid_schema_version` | The version is not an unsigned integer. |
| `unsupported_schema_version` | Direct parsing received an integer other than `2`, or persisted loading received an unsupported version; includes `found`. |
| `missing_document_id` / `invalid_document_id` | Identity is absent or not a UUID. |
| `missing_title` / `invalid_title` | Title is absent, non-string, or blank. |
| `missing_document` / `invalid_document_root` | Document is absent or not a `doc` object. |
| `invalid_document_content` | Root content is absent or not an array. |
| `invalid_citation_node` | Nested citation data is invalid; includes structural `path` and typed `cause`. |
| `invalid_text_format` | A font-family or font-size mark is malformed or unsupported; includes structural `path` and typed `cause`. |
| `invalid_paragraph_style` | Paragraph block data is malformed, misplaced, incomplete, or outside accepted bounds; includes structural `path` and typed `cause`. |
| `migration_failed` | The named v1-to-v2 transition rejected legacy data; includes only versions and a typed cause. |

Invalid inputs are never normalized, migrated, or replaced with defaults.
Open, Save, and Export wrap these failures in their typed `invalid_envelope`
command error before native dialog or filesystem work.

## Serialization

Serde owns both serialization and deserialization. The serialized field names
match the version 2 JSON shape exactly. UUIDs use canonical string output, and
the original valid title and nested Tiptap JSON are preserved. Tests compare
parsed JSON values so whitespace and object-key formatting are not part of the
contract.

## Verification

`scripts/check-invariants.sh` requires the version constant, the complete
Phase 11 test set, and the absence of Tauri or filesystem APIs in the envelope
module. `scripts/check-repository.sh` requires the Rust source to remain visible
to Git. Both scripts run through `scripts/verify.sh` locally and in the GitHub
Actions `verify` job.

Focused Rust coverage includes minimal deserialization, stable serialization,
round trips, every missing field, unknown fields, malformed and unsupported
versions, malformed identity and title metadata, invalid root/content shapes,
stable typed errors, nested Unicode JSON preservation, and valid/invalid nested
citation behavior.
Phase 46 coverage also rejects malformed font marks, unsupported canonical
families, and non-integer point sizes outside 8 through 72.
Phase 47 adds mirrored paragraph validation, current-only parsing, detached
v1 migration, stable typed failures, and default-by-absence coverage.

## Registry Integration

Phase 12 stores this validated domain type behind one Rust-owned live handle per
document. `docs/maintainers/DOCUMENT_REGISTRY.md` records that behavior. File
lifecycle and atomic writes are implemented by Phases 13 and 14.

Phase 18 citation scanning is documented in
`docs/maintainers/CITATION_NODE.md`. It changes validation behavior but does not
add an envelope field or change `DOCUMENT_ENVELOPE_SCHEMA_VERSION`.

Phase 46 adds only canonical `fontFamily` and `fontSize` marks inside Tiptap
content. The identifiers and point bounds are documented in
`docs/maintainers/CONFIGURATION.md`; they do not add an envelope field or
change the schema version.

Phase 43 established version 1 as the first document schema. Phase 47 adds the
named `1 -> 2` transition. Persisted version 1 data migrates in memory without
an explicit paragraph override; direct parsing and saves remain version 2 only.
Transition rules are documented in `docs/maintainers/DATA_MIGRATION.md`.

Paragraph values and implementation ownership are documented in
`docs/maintainers/PARAGRAPH_FORMATTING.md`.

## Configuration Index

Schema versions and document-dialog defaults are indexed in
`docs/maintainers/CONFIGURATION.md`.
