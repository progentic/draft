# Critical Path Evidence

## Status

This guide records implemented Phase 41 test evidence. It does not add a user
workflow or widen any production boundary.

## Scope

`src-tauri/src/critical_paths_tests.rs` composes existing Rust production paths
inside one crate-level test. The module is registered only under `cfg(test)`, so
the evidence can use crate-owned lifecycle functions without making them public
or creating a test-only command.

The test covers this ordered path:

1. First save creates one complete DRAFT source and registers one live handle.
2. A second save commits a citation-bearing snapshot to the retained path.
3. Close releases the handle, and reopen restores the last committed envelope.
4. A second open fails with `AlreadyOpen` while the reopened handle and source
   path remain current.
5. A validated reference persists in the existing SQLite store, and the
   reopened citation resolves through the existing Rust citation boundary.
6. DOCX export rejects the citation-bearing envelope with
   `UnsupportedCitation` and creates no target.
7. After an explicit supported snapshot is saved, atomic DOCX export succeeds,
   the package reopens, and its document part contains the final saved text.
8. Export leaves the DRAFT source bytes unchanged, and one close fully releases
   the final live handle.

## Boundary

Phase 41 adds no application command, dialog, frontend wrapper, component,
navigation path, persistence schema, citation authority, export control, or
product orchestration. It does not bypass native path ownership in production;
the test supplies a path only to the existing generic persistence function that
the Rust command normally calls after native selection.

The current workspace still has no create, open, save, close, citation
insertion, citation management, or DOCX export control. The test establishes
core boundary interoperability, not a released end-to-end user workflow.

## Failure Policy

The evidence preserves current failure contracts. Duplicate opens remain
typed, missing or malformed references remain owned by citation resolution, and
citation-bearing DOCX remains unsupported. Phase 41 does not convert, omit, or
render a citation during export merely to make the success path pass.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `critical_document_path_is_durable_citable_and_exportable` | Coordinates the existing critical boundary sequence. |
| Mid | lifecycle, citation, and export assertion helpers | Verify one production contract at a time. |
| Low | fixture builders and package reader | Build bounded valid input and inspect deterministic output. |

## Verification

Run the focused evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml critical_document_path_is_durable_citable_and_exportable
bash scripts/check-invariants.sh
bash scripts/check-docs.sh
```

The aggregate `bash scripts/verify.sh` gate remains required locally, in the
pull request, and after merge.
