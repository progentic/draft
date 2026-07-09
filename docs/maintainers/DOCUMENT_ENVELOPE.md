# Document Envelope

## Status

This is an implemented Phase 11 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/DOCUMENT_ENVELOPE.md` until the contract lifecycle is complete.

## Scope

Phase 11 defines and validates one in-memory document envelope. It does not
create documents, maintain open handles, call Tauri, read or write files,
save, reload, autosave, export, migrate, or persist data.

The version 1 JSON shape is:

```json
{
  "schema_version": 1,
  "document_id": "00000000-0000-4000-8000-000000000001",
  "title": "Untitled document",
  "document": {
    "type": "doc",
    "content": []
  }
}
```

The envelope has no path, timestamps, reference records, citation metadata,
analysis output, formatting findings, or export state. Unknown top-level fields
fail validation instead of becoming implicit schema extensions.

## Rust Authority

`src-tauri/src/documents/envelope.rs` is the validation authority. The module
provides:

- `DOCUMENT_ENVELOPE_SCHEMA_VERSION`, fixed at `1`
- `DocumentEnvelope`, whose fields are private after validation
- `DocumentId`, a Rust-parsed UUID identity
- `DocumentEnvelopeError`, a bounded serializable failure enum
- `DocumentEnvelope::from_json_value`, the untrusted JSON entry point

Phase 11 did not expose the envelope to TypeScript. Phase 13 adds a mirrored
response guard and request type under `src/ipc/` because open/save commands now
cross the bridge. The mirror protects UI code from malformed responses; it
does not replace Rust validation.

## Validation Order

Validation is deterministic:

1. Require a top-level JSON object.
2. Reject unknown top-level fields.
3. Require integer `schema_version` equal to `1`.
4. Require `document_id` to parse as a UUID.
5. Require `title` to be a string that is non-empty after trimming.
6. Require `document` to be an object with `type: "doc"`.
7. Require `document.content` to be an array.

Phase 11 preserves all nested document JSON as data. Full Tiptap node and
attribute validation belongs to later editor-schema work.

## Failure Shape

Failures serialize as tagged objects with a stable snake-case `code`.
Field-specific context is included where the caller needs it.

| Failure | Meaning |
| :--- | :--- |
| `invalid_envelope_object` | The top-level value is not an object. |
| `unknown_envelope_field` | An undeclared top-level field is present; includes `field`. |
| `missing_schema_version` | `schema_version` is absent. |
| `invalid_schema_version` | The version is not an unsigned integer. |
| `unsupported_schema_version` | The integer is not `1`; includes `found`. |
| `missing_document_id` / `invalid_document_id` | Identity is absent or not a UUID. |
| `missing_title` / `invalid_title` | Title is absent, non-string, or blank. |
| `missing_document` / `invalid_document_root` | Document is absent or not a `doc` object. |
| `invalid_document_content` | Root content is absent or not an array. |

Invalid inputs are never normalized, migrated, or replaced with defaults.

## Serialization

Serde owns both serialization and deserialization. The serialized field names
match the version 1 JSON shape exactly. UUIDs use canonical string output, and
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
stable typed errors, and nested Unicode JSON preservation.

## Registry Integration

Phase 12 stores this validated domain type behind one Rust-owned live handle per
document. `docs/maintainers/DOCUMENT_REGISTRY.md` records that behavior. File
lifecycle and atomic writes remain assigned to Phases 13 and 14.
