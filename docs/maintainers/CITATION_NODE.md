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

Phase 46 adds bounded manual-reference summaries and visible citation
insertion. It does not implement full reference CRUD IPC, complete APA
formatting, visible bibliographies, network lookup, import, jobs, or Python
ownership.

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

Only a Rust-returned success may show the display marker. Phase 39 gives invalid
attrs, missing references, store failures, invalid responses, and transport
failures distinct bounded copy. It never suggests a citation-management action
that the workspace does not expose. Each live node is one atomic polite region.
A revision counter prevents a late response for old attrs from replacing a
newer marker, and destroyed node views ignore pending responses. The mapping
policy is documented in `docs/maintainers/ERROR_UX.md`.

The reference panel can insert a valid citation node at the cursor. There is no
citation editor or automatic repair workflow.

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

- 14 attrs, schema, collection-order, nested-path, opacity, and error-shape tests;
- 5 known/missing/invalid/corrupt resolution tests;
- 4 command signature/request/response/error tests;
- 2 envelope round-trip/failure tests; and
- 2 open/save pre-mutation rejection tests.

Frontend evidence covers attrs/error guards, nested envelope validation, typed
resolution response and failure classification, Tiptap JSON preservation,
static fail-closed HTML, every live display state, and stale response
suppression.

Phase 41 adds crate-level evidence that a citation survives save, close, and
reopen before resolving against a persisted reference. The same test preserves
the current typed DOCX rejection instead of rendering or omitting the citation.

`scripts/check-invariants.sh` requires these sources and named Rust tests,
checks schema/version and Tiptap fail-closed markers, rejects embedded metadata,
and replaces the former citation absence gate. Phase 19 replaces the former
bibliography absence gate with the consistency behavior documented in
`docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md`; network, import, job, and helper
absence gates remain active.

The same tests and scans run through `scripts/verify.sh` locally and the GitHub
Actions `verify` job.

## Phase 19 Integration

Phase 19 reuses the fallible document citation collector to compare one
validated document with an explicit candidate bibliography of validated
reference records. It does not turn document nodes into metadata authority,
embed records, read the complete store, add network lookup, or rewrite source
documents.

Phase 43 treats citation attrs version 1 as the first released baseline. Lower
and future versions fail the containing document without mutation. Citation
transitions must be owned by an explicit document migration as described in
`docs/maintainers/DATA_MIGRATION.md`.

## Configuration Index

The mirrored citation schema version is indexed in
`docs/maintainers/CONFIGURATION.md`.
