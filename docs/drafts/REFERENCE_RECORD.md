# Reference Record Readiness Draft

**Status:** Draft, non-binding

**Implementation checkpoint:** Phase 16 complete

**Owners:** Rust core

**Related invariants:** `INV-03`, `INV-04`, and `INV-10`

## Purpose

This draft records the requirements used for Phase 16. The implemented
checkpoint is documented in `docs/maintainers/REFERENCE_RECORD.md`. This file
remains non-binding and does not authorize a reference store,
document-envelope changes, citation nodes, bibliography generation, network
lookup, PDF import, or workspace integration.

The record is future reference-library source data. It is not a citation-node
payload and must never be embedded as full metadata in the document envelope.

## Candidate Version 1 Shape

Phase 16 implements this normalized shape:

```json
{
  "schema_version": 1,
  "reference_id": "00000000-0000-4000-8000-000000000001",
  "citekey": "smith2025",
  "kind": "article",
  "title": "A bounded reference record",
  "contributors": [
    {
      "role": "author",
      "name": {
        "type": "person",
        "given": "Ada",
        "family": "Smith"
      }
    }
  ],
  "issued": { "year": 2025, "month": null, "day": null },
  "container_title": "Journal of Examples",
  "publisher": null,
  "volume": "12",
  "issue": "3",
  "pages": "1-12",
  "resolution_state": "resolved",
  "identifiers": {
    "doi": "10.1000/example",
    "isbn": [],
    "url": null
  },
  "provenance": {
    "source": "manual",
    "source_record_id": null,
    "manual_overrides": []
  }
}
```

Every record-owned object field is declared. Optional bibliographic values
serialize as `null` or an empty list instead of disappearing, so round trips
remain predictable. Unknown top-level or nested fields fail validation.

## Validation Authority

Rust is the durable validation authority. Phase 16 does not require a
TypeScript mirror because no command or UI consumes the record yet.

Phase 16 validates:

- `schema_version` is the integer `1`
- `reference_id` is a UUID
- `citekey` starts with an ASCII letter or digit and then contains only ASCII
  letters, digits, colon, underscore, or hyphen
- `kind` is `article`, `book`, `chapter`, `report`, `thesis`, `webpage`, or
  `other`
- `title` is non-empty after trimming
- contributor roles are `author`, `editor`, or `translator`
- person names have optional given and family values with at least one
  non-blank value present
- organization names use a non-empty literal name instead of person fields
- zero contributors are valid for an anonymous work
- an issued date is `null` or has a year from 1 through 9999, optional month
  from 1 through 12, and optional day from 1 through 31
- an issued day is invalid when its month is absent
- optional text and identifier values are either `null` or non-blank strings
- `isbn` entries are non-blank strings
- `resolution_state` is `unresolved`, `resolved`, or `needs_review`
- provenance source is `manual`, `crossref`, `semantic_scholar`, `unpaywall`,
  or `pdf_import`
- `source_record_id` is `null` or a non-blank string
- manual provenance has a null `source_record_id`
- manual provenance has an empty `manual_overrides` list
- `manual_overrides` contains unique values from `kind`, `title`,
  `contributors`, `issued`, `container_title`, `publisher`, `volume`, `issue`,
  `pages`, or `identifiers`

Phase 17 owns citekey uniqueness across stored records. Phase 16 validates one
record only and must not pretend to enforce library-wide uniqueness.

## Contributor Shape

Contributor names are a tagged union.

Person:

```json
{
  "role": "author",
  "name": {
    "type": "person",
    "given": "Ada",
    "family": "Smith"
  }
}
```

Organization:

```json
{
  "role": "author",
  "name": {
    "type": "organization",
    "literal": "Example Research Group"
  }
}
```

Mixed person and organization fields fail instead of being silently ignored.

## Provenance and Manual Edits

Provenance records where the candidate metadata originated; it does not prove
that the metadata is correct. `manual_overrides` identifies normalized fields
that a user changed after import so a future merge can preserve them. Phase 16
stores no reliability score and performs no resolution or merge.

`resolution_state` is descriptive data, not a Phase 16 workflow. `unresolved`
means metadata resolution has not completed, `resolved` means no resolution is
pending, and `needs_review` means conflicting or uncertain metadata requires a
future user decision. `resolved` does not mean verified or authoritative.
Phase 16 does not transition these states.

Provenance, identifiers, and resolution state are future reliability-scoring
inputs. The Phase 16 record contains no computed reliability score.

Future merge behavior must preserve explicit manual values and provenance. A
later metadata response may not silently overwrite a manually edited field.
That merge policy belongs to the reference-store and metadata phases and
requires its own tests before use.

## Failure Shape

Failures must be a bounded Rust enum, not strings. Callers must be able to
distinguish at least:

- non-object input and unknown fields
- missing, malformed, and unsupported schema versions
- missing or malformed identity and citekey
- unsupported reference kind
- missing or blank title
- malformed contributor role or name shape
- malformed issued date
- malformed optional bibliographic fields or identifiers
- malformed resolution state, provenance source, source-record identifier, or
  manual-override field

Indexed list failures should include the failing contributor or ISBN index
without echoing the rejected user value.

## Implemented Serialization Tests

Phase 16 tests prove:

- the minimum valid record deserializes
- all declared fields serialize with stable names
- valid records round trip as equal parsed JSON values
- person and organization contributors round trip
- partial and absent issued dates round trip
- Unicode bibliographic text is preserved
- every missing, malformed, unknown, and unsupported shape fails predictably
- every error variant has a stable serialized code

Tests compare parsed JSON values rather than relying on key order or whitespace.

## Explicit Phase 16 Non-goals

Phase 16 does not add:

- SQLite, files, a `ReferenceStore`, CRUD, or migrations
- a Tauri command, TypeScript IPC wrapper, React state, or visible controls
- document-envelope fields or embedded reference metadata
- a Tiptap citation node, citation rendering, or bibliography behavior
- citekey uniqueness across records
- Crossref, Semantic Scholar, Unpaywall, or other network calls
- PDF import, watched folders, or metadata extraction
- merge, deduplication, reliability scoring, or manual-edit workflows

The existing save, citation, network, import, job, and Python-helper gates must
remain green.

## Promotion Gate

This draft records Phase 16 requirements but may not move to
`docs/contracts/` until it satisfies the proposal, review, stability,
frontmatter, and acceptance requirements in `GOVERNANCE.md` section 7.
