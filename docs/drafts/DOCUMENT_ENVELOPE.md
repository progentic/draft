# Document Envelope Readiness Draft

**Status:** Draft, non-binding

**Target phase:** Phase 11

**Owners:** Rust core, with frontend snapshots as untrusted input

**Related invariants:** `INV-02`, `INV-06`, and `INV-09`

## Purpose

This draft fixes the entry requirements for Phase 11 before implementation
begins. It is not an accepted contract and does not authorize persistence,
save/load, filesystem access, document registry behavior, citation behavior,
or export.

## Candidate version 1 shape

Phase 11 will begin with this minimum envelope candidate:

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

No path, timestamps, reference metadata, rendered citations, analysis output,
formatting findings, or export state belongs in the Phase 11 minimum envelope.
Later fields require their owning phase and migration decision.

## Validation authority

Rust is the validation authority for the envelope. The frontend may construct
or display a snapshot, but a TypeScript type assertion is not validation.

Phase 11 must enforce:

- `schema_version` is an integer and only version `1` is accepted
- unknown top-level fields are rejected rather than ignored
- all four top-level fields are required
- `document_id` parses as a UUID generated or accepted by Rust
- `title` is a non-empty string after trimming
- `document` is an object with root `type` equal to `doc`
- `document.content` is an array, including for an empty document
- nested content remains JSON data and cannot trigger I/O or code execution

Full Tiptap node-schema and citation-attribute validation belongs to later
schema-owning phases. Phase 11 validates the envelope and document-root shape,
not citation resolution or editor extension behavior.

## Invalid-shape behavior

Invalid input must fail closed with bounded domain errors. Phase 11 should
define variants equivalent to:

```text
unsupported_schema_version
unknown_envelope_field
invalid_document_id
invalid_title
invalid_document_root
invalid_document_content
```

No invalid or unknown version may be normalized silently. Unsupported versions
enter explicit migration handling in a later migration phase.

## Serialization rules

Phase 11 serialization must:

- use Serde-owned Rust types instead of hand-built JSON strings
- serialize field names exactly as shown in the candidate shape
- preserve the Tiptap JSON value without converting it to HTML
- round-trip valid Unicode content and nested JSON values
- compare parsed JSON values in tests rather than relying on object key order or
  whitespace formatting
- avoid filesystem paths and platform-specific values

The schema version constant and serialized DTO must have rustdoc because they
control future migration behavior.

## Required Phase 11 tests

Phase 11 is incomplete until tests cover:

- minimal valid envelope deserialization
- stable serialization of the candidate shape
- serialization/deserialization round trip
- missing required fields
- unknown top-level fields
- schema versions `0` and `2`
- malformed UUID
- blank title
- non-object document root
- root type other than `doc`
- missing or non-array `document.content`
- nested Unicode and structured Tiptap JSON preservation

The malformed-envelope gate must run through `scripts/verify.sh` locally and in
the existing GitHub Actions `Verify` job.

## Explicit Phase 11 non-goals

Phase 11 must not add:

- create, open, close, focus, or double-open behavior
- a live document registry or Tauri document handle
- save/load commands or native file dialogs
- filesystem reads or writes
- temporary-file, fsync, rename, or atomic-save behavior
- SQLite or another persistence engine
- reference records, citation rendering, or bibliography generation
- formatting, analysis, network, or export behavior
- migration execution beyond returning an unsupported-version error

Those capabilities remain assigned to their later phasemap entries.

## Promotion gate

This draft may guide Phase 11 implementation but may not move to
`docs/contracts/` until it satisfies the proposal, review, stability,
frontmatter, and acceptance requirements in `GOVERNANCE.md` §7.
