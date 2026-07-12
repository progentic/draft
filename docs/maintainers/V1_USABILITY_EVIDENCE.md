# DRAFT v1 Usability Evidence

## Tested Artifact

- Commit: `154c34c96183ff67d4ecd6acd790b0410403dd58`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `e4ab1618764363b066706353149b8bbf700111a6f0a44fbce66ebc02cd4687f0`
- Identity result: the manually tested executable matched the recorded Phase 46 artifact.
- Session date: 2026-07-11

Artifact provenance does not prove workflow usability. Every release-candidate
row remains governed by its own closure evidence.

## Corrected Retest Candidate

- Implementation commit: `3308a3acfda02b2e247abdb3f23299585067b076`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `a8333b023dbc48b3e111cfa685118f1e4aa63340ab6183d956955faeb38de542`
- Mechanical result: package construction, arm64 validation, embedded icon
  validation, and the embedded deterministic text-analysis helper probe passed.
- Human result: partial; Save completed in the observed attempt, while New,
  font breadth, and text/Markdown import exposed additional open findings.

This package was built from the source tree that became the implementation
commit above. The evidence-record commit changes documentation only. It does
not change the tested executable. The findings below remain open until a
complete direct human workflow validates this corrected package.

## Phase 46

### Automated Evidence

- Local `scripts/verify.sh` passed on commit `154c34c` with 370 Rust tests, 252
  frontend tests, and 11 Python tests.
- GitHub Actions Verify passed on commit `154c34c` in PR #36.
- `scripts/package-macos.sh` built the unsigned Apple Silicon application and
  executed the embedded deterministic text-analysis helper successfully.
- Automated evidence does not replace the incomplete packaged human workflow.

### Packaged Application Evidence

| Area | Result | Evidence |
| :--- | :--- | :--- |
| Artifact identity | Pass | Commit and executable hash matched the intended `154c34c` package. |
| Launch | Pass | The packaged application opened for direct manual interaction. |
| Document lifecycle | Fail | Invoking Save made DRAFT unresponsive, displayed the macOS beach ball, and required force-quit. Save did not complete and no recovery action was available. Create and New could not be validated reliably in that session. |
| Formatting controls | Partial | Existing formatting controls were visible, but no font-family or font-size control was available. |
| References and citations | Pending | Not reached after the lifecycle failure. |
| Five local text checks | Pending | Not reached after the lifecycle failure. |
| Finding navigation and focus | Pending | Not reached after the lifecycle failure. |
| DOCX export and source integrity | Pending | Not reached after the lifecycle failure. |
| Recovery and keyboard accessibility | Fail | The unresponsive Save state offered no recovery and required force-quit; remaining keyboard checks are pending. |

No permissions or application-trust cause was established.

The `154c34c` implementation used blocking native dialog APIs from synchronous
Tauri commands for Open, Save, and Export. The installed dialog API documentation
states that blocking dialog APIs must not run on the main thread. This invalid
execution pattern is recorded separately from the manual observation as the
original defect mechanism. The corrected candidate replaced that path, but the
finding remains open until the complete packaged lifecycle is repeated.

### Corrected Candidate Partial Retest

The repository owner directly exercised the corrected candidate identified
above. Save completed in that attempt without the prior beach ball, but the
complete repeated lifecycle was not finished, so `UX-46-001` remains open. New
opened seeded text instead of a blank caret-ready page. The font controls were
present, but the family list was too narrow. Open did not offer `.txt` or `.md`
files. Font persistence, DOCX fidelity, import source preservation, references,
text checks, complete recovery states, and keyboard completion were not proven.

This partial session is historical defect evidence for the `3308a3a` package.
It cannot validate the replacement artifact produced by the current fixes.

## Replacement Human-Retest Artifact

- Implementation commit: `68aa08d8a0577ec32a128cd3368ea830be7f91f5`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `ae66d3dae64fbe738fcd371b776b27d022bea3182eb9920c89773498dcf289f9`
- Mechanical result: package construction, arm64 validation, embedded icon
  validation, and embedded deterministic text-analysis helper execution passed.
- Human result: partial; desktop-shell, interoperability, and round-trip
  ownership findings remain open.

This artifact contains the blank New document, explicit lifecycle origins,
bounded literal `.txt` and `.md` import, source-preserving `.draft` first Save,
eleven-family formatting allowlist, and exact DOCX mappings. Those statements
describe mechanically verified implementation scope, not packaged usability
evidence. All findings and release rows remain open until direct human retest.

### Replacement Artifact Product-Boundary Review

Owner review of the replacement artifact found that the mechanically valid
bundled `.icns` does not establish correct visible window branding. The
in-window icon presentation was incorrect, the command bar and editor layout
lacked expected desktop-editor hierarchy, and the current import/save model did
not meet the intended academic interoperability boundary.

Literal `.txt` import behaved within its documented contract. Markdown also
behaved as implemented, but literal source display is now insufficient for the
v1 product requirement because headings, emphasis, lists, quotations, links,
code, and separators are not parsed into document structure. DOCX, RTF, and
OpenDocument import are absent. Imported external formats cannot be saved back
to their source format, and DRAFT has no lossiness or round-trip capability
model. Legacy binary `.doc` remains a distinct unsupported format and is not
treated as equivalent to DOCX.

These observations do not authorize implementation in Phase 46. They require a
separate governed interoperability and desktop-product boundary. PR #36
remains draft, and no finding or release row closes from this review.

### Human Task Results

The repository owner directly tested the recorded packages. The first session
stopped after Save caused an unrecoverable beach ball and force-quit. Later
sessions produced the additional partial and product-boundary findings above.
No untested task is counted as passed.

### Findings And Dispositions

| ID | Severity | Status | Evidence | Disposition |
| :--- | :--- | :--- | :--- | :--- |
| UX-46-001 | UX-0 | Open | During manual validation of packaged artifact `154c34c`, invoking Save caused DRAFT to become unresponsive and display the macOS beach ball. Save did not complete, no recovery action was available, and the application required force-quit. | Replace the synchronous blocking dialog path with async Tauri commands and non-blocking Rust-owned dialog callbacks, then rebuild, rehash, and rerun the complete packaged workflow. |
| UX-46-002 | UX-2 | Open | The first artifact exposed no font-family control. The corrected candidate exposed a bounded control, but its family list was too narrow for the intended workflow. | Expand one canonical allowlist across validation, toolbar, editor marks, persistence, and DOCX mapping; then rebuild, rehash, and validate persistence and export in the corrected package. |
| UX-46-003 | UX-2 | Open | The packaged editor exposed no font-size control. | Add a bounded point-size control only if the DRAFT envelope persists it, reopen restores it, and DOCX preserves it accurately. Rebuild, rehash, and validate the corrected package before disposition. |
| UX-46-004 | UX-2 | Open | New opened seeded content instead of a blank page ready for immediate typing. | Return one empty paragraph from Rust, focus its caret only after success, then validate the replacement package. |
| UX-46-005 | UX-2 | Open | Open did not offer plain-text files. | Import bounded UTF-8 `.txt` as a new unsaved Rust-owned envelope, preserve the source, and validate first Save to a new `.draft` target. |
| UX-46-006 | UX-2 | Open | Open did not offer Markdown files. | Import bounded UTF-8 `.md` as literal editable text without parsing or preview claims, then validate the packaged workflow. |
| UX-46-007 | UX-1 | Open | Text and Markdown import source-preservation behavior could not be exercised because those inputs were unavailable. | Prove the source path never becomes save authority, first Save selects a new `.draft` target, and the source remains byte-for-byte unchanged in automated and packaged tests. |
| UX-46-008 | UX-2 | Open | The replacement artifact contained the tracked bundle icon, but the application icon did not render correctly in the visible packaged window chrome. | Keep bundle-icon validation separate from visible branding; inspect the title-bar/header asset path and validate the corrected packaged window in light and dark appearance. |
| UX-46-009 | UX-1 | Open | File, research, review, and export controls share one sparse command row without sufficient grouping or predictable desktop-editor hierarchy, and native macOS menu integration does not exist. | Move this release-blocking workflow problem to a governed desktop UI phase with grouped controls, native menus, state-sensitive enablement, responsive overflow, and shared action dispatch. |
| UX-46-010 | UX-1 | Open | Markdown opens as literal source, so headings, emphasis, lists, quotations, links, code, and separators are not represented as editable document structure. | Define and implement a bounded Markdown parser/serializer contract in the governed interoperability phase; unsupported constructs must fail or disclose loss rather than disappear. |
| UX-46-011 | UX-1 | Open | DOCX import is unavailable even though DOCX export exists. | Add a separately governed DOCX import and safe round-trip contract with fidelity classes, fixtures, source preservation, and explicit unsupported-content behavior. |
| UX-46-012 | UX-2 | Open | RTF import and save are unavailable. | The interoperability decision must either implement a bounded RTF subset or accept an explicit v1 deferral with user guidance. |
| UX-46-013 | UX-2 | Open | OpenDocument import and save are unavailable. | The interoperability decision must either implement bounded ODT support or accept an explicit v1 deferral with user guidance. |
| UX-46-014 | UX-1 | Open | Imported external formats become unsaved DRAFT documents and cannot be safely saved back to their original format. | Define Rust-owned external source identity, writable-format capability, lossiness state, no-edit byte preservation, overwrite safety, Save As behavior, and compatibility tests before enabling round-trip save. |
| UX-46-015 | UX-2 | Open | Command spacing, grouping, editor canvas composition, and outline layout do not meet the intended desktop-product quality threshold. | Address visual hierarchy and layout in the governed desktop UI phase, then validate normal, narrow, scaled, keyboard, and reduced-motion states from the packaged app. |

`RC-01` through `RC-04` and `GATE-46` remain open.
