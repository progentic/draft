# Citation Node Requirements Draft

## Status

This document is a non-binding Phase 18 requirements draft. Implemented
behavior is recorded separately in `docs/maintainers/CITATION_NODE.md` once the
phase is complete. This draft does not become an accepted contract without the
review lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 18 owns one versioned inline Tiptap citation node, Rust validation of its
attributes, Rust-backed citekey resolution, and explicit frontend display
states. It does not own reference creation, citation insertion controls,
bibliography generation, complete citation-style formatting, network lookup,
PDF import, export, or document-envelope fields.

The local reference store remains the metadata source of truth. The node keeps
only a stable lookup key and requested rendering style.

## Node Shape

The version 1 JSON shape is:

```json
{
  "type": "citation",
  "attrs": {
    "schema_version": 1,
    "citekey": "smith2025",
    "render_style": "apa7"
  }
}
```

The node is an inline atom with no child content and no marks. The only allowed
node fields are `type` and `attrs`. The only allowed attribute fields are
`schema_version`, `citekey`, and `render_style`.

## Validation Authority

Rust is the durable validation authority. TypeScript mirrors the current
schema so malformed IPC payloads and editor state can fail safely, but the
mirror does not replace Rust validation.

Rust recursively scans citation nodes in a document envelope before the
envelope is registered or saved. A malformed citation produces a typed
document-envelope failure with its structural path and citation cause. Other
Tiptap node kinds remain preserved as opaque document data.

## Attribute Rules

- `schema_version` is required and must be the unsigned integer `1`.
- A different unsigned version is an explicit unsupported-version failure.
- `citekey` is required and must use the same ASCII shape as reference records.
- `render_style` is required and version 1 supports only `apa7`.
- Missing, malformed, unsupported, or unknown values fail closed.
- No full reference record, CSL JSON, rendered citation, or bibliography data
  may be embedded in the node.

## Resolution

The typed `resolve_citation` command accepts one untrusted attrs value. Rust:

1. validates the attrs contract;
2. reads the exact case-sensitive citekey from the reference store;
3. validates the stored reference through the store read path; and
4. returns a disposable display marker only when the record exists.

The response does not return bibliographic metadata. The initial marker is a
resolved editor token, not final APA output and not document source data.

Resolution failures distinguish invalid attrs, missing references, and store
failures. Raw SQLite errors, paths, and SQL never cross IPC.

## Tiptap Behavior

The Tiptap extension preserves the three attrs in JSON. Static HTML output must
show a requires-resolution state rather than pretending that a citation has
been resolved. The live node view may display the Rust-returned marker only
after successful resolution.

The live node view has explicit states:

- invalid attrs;
- resolving;
- resolved;
- reference unavailable; and
- resolution failed.

Invalid or unresolved nodes do not silently render as valid citations. Stale
asynchronous responses must not replace a newer node state.

## Typed Failure Shape

Citation validation failures must distinguish:

- invalid attrs object;
- unknown attr;
- missing, malformed, or unsupported schema version;
- missing or malformed citekey; and
- missing or unsupported render style.

Resolution failures must distinguish invalid attrs, reference not found, and a
typed reference-store cause.

## Verification

Phase 18 tests must cover:

- valid attrs serialization and round trips;
- every required field and unknown-field rejection;
- malformed and unsupported schema versions;
- malformed citekeys and unsupported styles;
- nested envelope scanning and typed path failures;
- save/open envelope rejection before filesystem or registry mutation;
- known, missing, and corrupt-store resolution paths;
- command request, response, error, and registration parity;
- TypeScript guards and typed IPC classification;
- Tiptap JSON/HTML behavior and every display state; and
- stale resolution suppression.

Local verification and GitHub Actions must execute the same checks.

## Explicit Non-Goals

Phase 18 does not add:

- reference CRUD IPC or visible reference-library controls;
- citation insertion or editing controls;
- full APA rendering or another citation-style engine;
- bibliography generation or consistency checks;
- reference records or citation display caches inside document JSON;
- network clients, metadata search, browser handoff, or PDF import;
- export behavior, persistent jobs, or Python helpers; or
- a document-envelope schema version change.

These boundaries remain enforced until their owning phases.
