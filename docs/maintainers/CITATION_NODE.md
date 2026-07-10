# Citation Node

## Status

This is an implemented Phase 18 checkpoint guide. It records current behavior
for maintainers but is not an accepted contract under `GOVERNANCE.md` section
7. The non-binding requirements remain in
`docs/drafts/CITATION_NODE.md` until the contract lifecycle is complete.

## Scope

Phase 18 implements a versioned inline Tiptap citation node, Rust validation,
Rust-backed local citekey resolution, a typed `resolve_citation` command and
frontend wrapper, and explicit live/static display states.

It does not implement reference CRUD IPC, visible citation insertion controls,
complete APA formatting, bibliographies, citation consistency checks, network
lookup, import, export, jobs, or Python helpers.

## Source of Truth

The SQLite reference store remains the metadata source of truth. A citation
node stores only:

```json
{
  "schema_version": 1,
  "citekey": "smith2025",
  "render_style": "apa7"
}
```

No reference ID, title, contributor, date, identifier, provenance, CSL JSON,
bibliography entry, or rendered citation is stored in node attrs.

`CITATION_NODE_SCHEMA_VERSION` is `1`. The only version 1 rendering request is
`apa7`. The current response is a disposable editor marker such as
`[@smith2025]`; it is not final APA output and is never written into attrs.

## Rust Validation

`src-tauri/src/citations/node.rs` owns durable attrs validation. It rejects:

- non-object attrs;
- unknown node or attrs fields;
- missing, malformed, and unsupported schema versions;
- missing or malformed citekeys; and
- missing or unsupported render styles.

The citekey validator is shared with the reference-record contract, so both
surfaces accept the same case-sensitive ASCII shape.

`validate_document_citations` walks only Tiptap `content` arrays. When it finds
a `type: "citation"` node, it requires exactly `type` and `attrs`, validates the
attrs, and returns a structural path with the typed cause on failure. Other
Tiptap node kinds remain opaque document data.

`DocumentEnvelope::from_json_value` runs this scan after root validation. Open
therefore rejects an invalid citation before registry insertion; save rejects
one before path selection, serialization, registry mutation, or disk writes.
The document envelope remains schema version 1 and gains no citation fields.

## Resolution Boundary

At desktop startup, `application::reference_store` resolves the Tauri app-data
directory, opens `<app_data_dir>/references.sqlite3`, and registers one
`ReferenceStore` as managed Rust state. Startup fails closed if path resolution,
schema migration, or store verification fails.

`resolve_citation` accepts one outer request with an untrusted `attrs` JSON
value. The command delegates to `citations::resolution::resolve_citation`, which:

1. validates attrs in Rust;
2. performs an exact citekey lookup through `ReferenceStore`;
3. reuses the store's full payload and index validation; and
4. returns only schema version, citekey, render style, and display marker.

No SQLite API appears in the command, application initializer, frontend, or
Tiptap extension. No record metadata crosses IPC.

## Failure Shape

`CitationNodeError` distinguishes node shape, unknown fields, each missing
required attr, invalid schema/citekey values, unsupported versions, and
unsupported render styles.

`CitationResolutionError` distinguishes:

- `invalid_citation` with a typed citation cause;
- `reference_not_found`; and
- `reference_store` with `unavailable`, `read_failed`, or
  `corrupt_reference`.

The resolution boundary deliberately reduces the wider store error enum to
these caller-relevant categories. It exposes no path, SQL, raw SQLite detail,
or stored payload.

## Frontend Mirror

`src/citations/citationNode.ts` mirrors the attrs and error schema for UI and
IPC safety. It recursively checks citation nodes in Rust-returned document
envelopes. This mirror cannot authorize save/load or replace Rust validation.

`src/ipc/citationResolution.ts` owns the `resolve_citation` command name, exact
request envelope, unknown-response validation, typed command-error guard, and
transport fallback. React and editor modules do not import raw Tauri APIs.

## Tiptap Extension

`src/editor/CitationNode.ts` defines `citation` as an inline atom with no marks
or child content. Tiptap JSON preserves exactly the three attrs.

Static HTML never claims resolution. Valid attrs serialize as
`requires-resolution`; invalid attrs serialize as `invalid`. The live node view
shows one explicit state:

- `invalid`
- `resolving`
- `resolved`
- `unavailable`
- `failed`

Only a Rust-returned success may show the display marker. Missing references
and failures show `Citation unavailable`. A revision counter prevents a late
response for old attrs from replacing a newer marker, and destroyed node views
ignore pending responses.

No insertion toolbar or citation editor exists yet, so the visible default
document contains no citation node.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `resolve_citation` command | Coordinates one typed request against managed Rust state. |
| Mid | `citations::resolution::resolve_citation` | Enforces validation, lookup, and response policy. |
| Mid | `validate_document_citations` | Enforces citation validity inside envelope content. |
| Mid | `CitationNode` extension | Defines editor schema and delegates live presentation. |
| Low | attrs parsers, store lookup, IPC guards, DOM state helpers | Perform primitive validation, access, and display mechanics. |

## Verification

Rust evidence includes:

- 13 attrs, schema, nested-path, opacity, and error-shape tests;
- 5 known/missing/invalid/corrupt resolution tests;
- 4 command signature/request/response/error tests;
- 2 envelope round-trip/failure tests; and
- 2 open/save pre-mutation rejection tests.

Frontend evidence covers attrs/error guards, nested envelope validation, typed
resolution response and failure classification, Tiptap JSON preservation,
static fail-closed HTML, every live display state, and stale response
suppression.

`scripts/check-invariants.sh` requires these sources and named Rust tests,
checks schema/version and Tiptap fail-closed markers, rejects embedded metadata,
and replaces the former citation absence gate. It keeps the Phase 19
bibliography, network, import, job, and helper absence gates active.

The same tests and scans run through `scripts/verify.sh` locally and the GitHub
Actions `verify` job.

## Phase 19 Gate

Phase 19 may add consistency analysis across citation nodes and local reference
records. It must define missing, orphaned, and duplicate citekey semantics
before implementation. It must not turn document nodes into metadata authority,
embed full records, add network lookup, or silently rewrite source documents.
