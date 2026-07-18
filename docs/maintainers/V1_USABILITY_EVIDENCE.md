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

## Save-Identity Retest Artifact

- Correction commit: `a0f1ab8d5cc0def97fe98d501324633e341bef74`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `3b4e996091d9a6618d62570070fcc3d412b394690b855b502114d4f2cc1e7dd0`
- Mechanical result: package construction, arm64 validation, embedded icon
  validation, and embedded deterministic text-analysis helper execution passed.
- Human result: partial; New, Open, and Save passed, while the font controls,
  native File menu, and packaged icon exposed findings `UX-46-019` through
  `UX-46-021`.

The artifact returned basename-only save identity and updated the displayed
filename only after success. Direct testing confirmed New, Open, and Save work
as expected, closing the specific historical findings `UX-46-001`,
`UX-46-004`, and `UX-46-018`. The complete packaged workflow and every RC row
remain open.

## Font-State Retest Artifact

- Correction commit: `228bce73e9ea210e9f6f842d8bb2683b70031de4`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `8870dcc412dfb04ada5ae0ba28ea630eb37925bc94b55b4f182e423b5afd9eb4`
- Mechanical result: package construction, arm64 validation, embedded icon
  validation, and embedded deterministic text-analysis helper execution passed.
- Human result: partial; effective font family and size, Open, Save, and Close
  passed. The complete eight-step workflow did not pass because the native File
  menu and packaged application identity remain defective.

This artifact derives the font controls from exact document defaults, explicit
caret marks, or mixed selections. Direct testing confirmed that the family and
size controls show their effective current values, closing `UX-46-019`.
Automated tests continue to cover immediate updates, reset behavior, JSON
restoration, save/reopen restoration, and existing DOCX font fidelity. All RC
rows and `GATE-46` remain open because this was not a complete workflow pass.
Phase 48 findings `UX-46-020` and `UX-46-021` remain confirmed failures.

## Phase 48 Mechanical Retest Candidate

- Implementation commit: `6b1003273773563cfef06b30a76355f0062d25a3`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `b9ebc25ee5cf3822024bf8f488385407c262679fb807056c70c08fadae60f558`
- Canonical icon source SHA-256: `ce7cc5a5df592ac11873ff0f49d9c150e5a3a64e0c0ef9ffd1e05162da5fb043`
- Tracked and embedded `icon.icns` SHA-256: `fd07d079de1dd38bdc84eb222ab8ee90d856d488aad7f0550860c8a369b94236`
- Mechanical result: the arm64 application package built successfully,
  `Info.plist` names `icon.icns`, and the embedded icon is byte-for-byte
  identical to the tracked generated asset.
- Human result: partial; manual review confirmed a usable native File menu and
  closed `UX-46-020`. The command-bar, status-placement, and visible-icon
  findings remain open.

Automated tests and structural checks cover the File menu contract,
state-aware shared dispatcher, Save As authority, and icon chain. Manual review
confirmed that the native File menu provides a usable document workflow. It
did not complete the following packaged checks:

- toolbar and native-menu parity;
- disabled-action behavior during busy states;
- Save As rebinding and Save As cancellation;
- Finder, Dock, and application-switcher icons;
- the in-window purple identity; or
- behavior after clearing stale macOS icon caches.

Finding `UX-46-020` is closed by direct manual review. Findings `UX-46-021`
through `UX-46-024`, `RC-08`, `GATE-48`, and every other release row remain
open. The package hash and icon comparison are mechanical evidence only and
cannot close a visible icon finding.

## Phase 48 Compact Chrome Retest Candidate

- Package source commit: `36d999d9dd853c6a760721d4339e15bebbfb435b`
- Product implementation ancestor: `1358e41e452a877b958b4e54ff6a9d93d2db00ba`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `75373ffbb2a0b8aedd995ace95a4387f403a2ab38fbb6b457143ce976ce6cb37`
- Mechanical result: the arm64 package built successfully, the embedded helper
  probe passed, `Info.plist` names `icon.icns`, and the embedded icon matches the
  tracked generated asset byte-for-byte.
- Rendered-browser result: 1200 by 800 and 760 by 560 viewports had no page
  overflow, clipped controls, or overlap between workspace bars. The menu stayed
  inside the viewport, skipped disabled actions, and retained visible focus.
- Human result: passed on 2026-07-13 against the exact executable hash above.

This artifact contains the compact New action, icon-only Open, Save, and Close
controls, one labeled overflow menu for secondary actions, and a bottom status
bar for document, operation, recovery, and connectivity state. Direct human
testing confirmed:

- the native File menu actions and standard shortcuts;
- the compact top action bar and overflow behavior;
- document, connectivity, operation, and recovery placement in the bottom bar;
- toolbar, overflow, native-menu, and shortcut dispatcher parity;
- disabled-action behavior during busy states;
- Save As rebinding and cancellation;
- Finder, Dock, application-switcher, and in-window purple identity; and
- narrow-window behavior and keyboard accessibility.

This pass closes findings `UX-46-021`, `UX-46-022`, and `UX-46-023` for the
tested Phase 48 implementation. It is not approval of the final UI design; the
broader v3 workspace target, paragraph formatting, research tools, sharing, and
later visual refinement remain outside this evidence. Finding `UX-46-024`
remains open. This historical artifact predates ADR-004's accepted Phase 47
model implementation and contains no paragraph controls. `RC-08`, `GATE-48`,
and every other RC and GATE row remain open.

## Phase 47 Visible Source-Save Manual Gate

- Implementation commit: `a60f877148cf9a430fc801d0570362b8c4882788`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `910a204fd94cfa892fe634f7da6ffbddb6047b8b0edc69c8c07d78a139ac9f44`
- Mechanical result: package construction and the complete local verifier passed.
- Human result: failed on 2026-07-13.

The direct session produced three passing observations: exact replacement
reopened in the tested reader, cancellation preserved the current state, and
visible recovery messages exposed neither paths nor raw XML. These observations
do not close the workflow because normalized replacement, stale-source
protection, native/overflow parity, and complete compatible-reader evidence
failed.

| Area | Result | Evidence |
| :--- | :--- | :--- |
| Exact replacement | Pass | The tested exact source was replaced and reopened without exposing source authority to the frontend. |
| Normalized replacement | Fail | The package did not provide a usable warning-and-consent path backed by output that passed compatible-reader validation. |
| Cancellation | Pass | Cancelling preserved the source, document identity, and modified state in the tested flow. |
| Stale-source protection | Fail | External modification did not surface the required typed rejection during the manual workflow. |
| Safe recovery copy | Pass | The observed messages contained no source path, fingerprint, or raw XML. |
| Native and overflow parity | Fail | The two entry points did not demonstrate equivalent Save Back behavior and disabled states. |
| Compatible-reader evidence | Fail | The complete exact and normalized output set did not pass the required compatible-reader gate. |

This is a failed release gate. Any correction requires a new commit, rebuilt
package, new executable hash, and complete retest. `INV-17`, `UX-46-024`, every
RC row, and every GATE row remain open.

The uncommitted correction candidate names the one supported heading-style
normalization, uses explicit Replace and Cancel choices, carries the closed
normalization list through strict IPC validation, and proves stale rejection
from real DOCX import through the immediate pre-write fingerprint check. The
Rust eligibility DTO now omits obsolete normalization metadata when a
normalized source has become stale, allowing the typed denial to remain valid
at the frontend boundary. The
shared dispatcher now presents the same busy or stale reason for overflow and
native actions. Local text extraction and a configured LibreOffice 26.8
headless conversion reopened exact and accepted-normalized replacements; the
test records source and replacement hashes. These are mechanical corrections,
not packaged-human evidence, and close no row.

### Phase 47 Replacement Retest Candidate

- Implementation commit: `0b357d436fce614c132b4f5692dda9f321656e85`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `cb36338524e042291fd2e2ac34bf8c09c855e461cb22ede451e2c92584415e83`
- Package result: construction and mechanical packaging checks passed.
- Human result: failed on 2026-07-13.

The configured compatible-reader test used LibreOfficeDev 26.8 in headless
conversion mode. The exact source SHA-256 was
`74c38bdd8940a1524baa2c25394aaae4a29ea1ed742e8255dd31ba56e1fb05c1`;
the normalized source SHA-256 was
`0a42b331afc608253cca854a77ae6fa65149d0fcc7a0aeb2490d61ce36530a0f`.
Both accepted replacements produced SHA-256
`4009080983f73aa047b15862e80245d7d7918117cb782459d2a98429669443ea`,
differed from their source, and converted to nonempty PDF output. This proves
bounded package acceptance by the configured reader, not visual fidelity or
the packaged user workflow.

The manual session invalidated this package as a Phase 47 evidence candidate
before the complete source-save retest could finish. The running app exposed
only product version `0.1.0`, so a reviewer could not confirm the expected
build from inside DRAFT. A selected DOCX did not produce a visible open result,
and DOCX export did not produce visible completion or recovery. Native `.draft`
files still lacked a DRAFT association and document identity. The executable
hash remains authoritative historical identity, but this package closes no
row.

The next package still requires the complete manual gate: visible build
identity, DOCX open and export disposition, normalized warning and cancellation,
stale-source rejection before replacement, native and overflow parity,
busy-state denial, exact and normalized compatible-reader inspection, Markdown
disclosure, DOCX safety recovery, reset-label review, and `.draft` double-click
handling. No row may close from mechanical evidence alone.

### Phase 47 Build-Identity And Document-Association Retest Candidate

- Implementation commit: `7ec149de8fcf71b0e3670687dbc0246e439ff09f`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `c3b2b54c8ce6c50fef1bd093b210b94d4925a51ca556d9ba4b2602200da881e0`
- Package result: construction and mechanical packaging checks passed.
- Human result: pending.

The package embeds the full implementation commit, and the running workspace
is expected to show short commit `7ec149de` with profile `release`. Its
`Info.plist` owns `.draft` as `com.progentic.draft.document` with Editor role,
Owner rank, and `icon.icns`. The packaged and tracked icon hashes both equal
`fd07d079de1dd38bdc84eb222ab8ee90d856d488aad7f0550860c8a369b94236`.
These checks establish artifact provenance and bundle registration only. They
do not prove visible build identity, DOCX outcomes, Launch Services behavior,
or double-click opening.

Manual review must still run the complete gate against this exact executable.
No prior artifact evidence transfers to this candidate, and this mechanical
record closes no finding, invariant, RC row, or roadmap gate.

### Replacement Artifact Product-Boundary Review

Owner review of the replacement artifact found that the mechanically valid
bundled `.icns` does not establish correct visible window branding. The
in-window icon presentation was incorrect, the command bar and editor layout
lacked expected desktop-editor hierarchy, and the current import/save model did
not meet the intended academic interoperability boundary.

Literal `.txt` import behaved within its documented contract. Markdown also
behaved as implemented, but literal source display is now insufficient for the
v1 product requirement because headings, emphasis, lists, quotations, links,
code, and separators are not parsed into document structure. Phase 47 now has
mechanical coverage for a bounded DOCX paragraph import, closed fidelity
categories, Rust-owned source provenance, and no-edit source preservation.
The visible same-format DOCX path now has typed eligibility, confirmation,
source-conflict denial, rollback, and local macOS reader-open evidence. Its
packaged human evidence, broader compatible-reader fidelity evidence, RTF, and
OpenDocument import remain absent. Legacy binary `.doc` remains a distinct
unsupported format and is not treated as equivalent to DOCX.

These observations do not authorize implementation in Phase 46. They require a
separate governed interoperability and desktop-product boundary. PR #36
remains draft, and no finding or release row closes from this review.

A later manual review of the same replacement executable confirmed that Open
and Close complete without changing the source file. It also found that the
native save panel presented an Open action during Save, the workspace did not
replace the provisional title with the saved filename, the native File menu did
not follow expected desktop document ordering, and the visible menu icon and
toolbar grouping remained stale. The menu and grouping findings remain assigned
to accepted Phase 48. The save-panel and post-save identity finding remains a
Phase 46 correction and requires a rebuilt, rehashed manual retest.

Source tracing confirms that the Save command calls `select_save_document`,
which calls `tauri_plugin_dialog::save_file`; the desktop plugin delegates to
`rfd::AsyncFileDialog::save_file`, whose macOS backend constructs an
`NSSavePanel`. The Save command contains no open-dialog selector. This trace
does not invalidate the observed Open label or count as packaged correction
evidence.

### Human Task Results

The repository owner directly tested the recorded packages. The first session
stopped after Save caused an unrecoverable beach ball and force-quit. Later
sessions produced the additional partial and product-boundary findings above.
No untested task is counted as passed.

### Phase 47 P0 DOCX Workflow Blocker

Artifact `c3b2b54c` displayed the expected `7ec149de` release build identity,
but the primary DOCX workflow still failed. Opening a normal Word package left
the editor unusable behind a generic safety-limit message, and Export did not
produce a usable output or terminal disposition during the same session.
That artifact established the historical P0: **DOCX primary workflow is
non-functional in the packaged application**. Later artifacts split that
failure into independently reviewed Open, export, and fidelity outcomes.

A Word-authored 12,031-byte fixture reproduced the import failure as the closed
internal reason `RelationshipTarget`. Word correctly resolves
`../customXml/item1.xml` from `word/document.xml`; DRAFT incorrectly treated the
raw target as package-root traversal. The correction resolves targets from
their owning part while continuing to reject root escape. Production-path
tests now open the Word fixture, create a canonical document, export atomically,
reopen the result, preserve the source, and produce visible text that matches
in LibreOffice. These are correction tests, not packaged-human closure evidence.

Replacement artifact `8e974736` ran commit `14363903` in the release profile.
Manual review confirmed that DOCX export completed with visible success, but
DOCX Open still did not produce an imported document. The post-close workspace
showed `No document open` while retaining the settled `DOCX export complete`
notice. That stale export notice outranked the active document-session outcome
and made the later Open disposition impossible to observe. The exact Word
fixture reaches a valid three-paragraph `imported_external` response in Rust.
The correction clears settled export feedback before non-export document
actions and renders that exact response through the real Open action in the
workspace test. A replacement packaged Open run remains required.

The replacement candidate was built from exact implementation commit
`e734cae26068636edb574ff6217837c08ba4e4c0`. Its packaged executable SHA-256 is
`2dfe312b446051946102ce40a074ac86e468dc074299af34802e90bf0c23d326`.
Mechanical package validation confirmed the embedded commit identity, Apple
Silicon executable, helper, icon, and bundle metadata. Manual review confirmed
that DOCX Open now creates a readable imported document, closing only the basic
Open failure in `UX-47-010`. The same session found that explicit Times New
Roman 12-point text, bold and italic spans, paragraph appearance, page breaks,
and source-recognizable academic formatting were flattened. Basic Open passes;
source-format fidelity and the overall Phase 47 Open gate fail.

The correction under review preserves the accepted explicit run and paragraph
properties, represents page breaks as canonical blocks, re-exports those
blocks, and proves the typed result renders through the actual workspace Open
action. The replacement fidelity candidate was built from exact implementation
commit `dc1d5d6b65e17ef8f6ebbfbb37ad886f64e1acbf`. Its packaged executable
SHA-256 is
`91fe1ba93dc2e1ea08a4096e9c3b863b460d481f2a00ee024f2bf477f78f40cc`.
Mechanical package validation confirmed the embedded commit identity, Apple
Silicon executable, helper, icon, and bundle metadata. No human fidelity or
compatible-reader result has been recorded for this candidate. `UX-47-013`,
`RC-07`, `GATE-47`, and all release gates remain open until the exact package
passes the compatible-reader and human fidelity workflow.

Direct review of that exact candidate later confirmed that supported run and
paragraph formatting was substantially retained, but the application still
showed one continuous white canvas with a dashed page-break marker instead of
separate page surfaces. The same session found build metadata in the document
inspector, no visible Unsaved state beside the document title, and weak native
Save filename suggestions. These are partial product-boundary results, not
fidelity or release closure evidence.

The correction under review renders only explicit canonical `pageBreak` nodes
as separate page surfaces. It does not infer page boundaries from content
flow, margins, font metrics, or printer geometry. It also moves exact build
identity to About DRAFT and the bottom-right status bar, synchronizes
basename-only clean and Unsaved titles with the native window, and lets Rust
derive bounded `.draft` Save suggestions from the typed document origin.

The replacement presentation candidate was built from exact head
`f7f19c35d8cf8071ee17fbd80a23b2f631635a34`. Its packaged executable SHA-256
is `1634d6d24642705bb2f20cd01af3d7426da63c63473cf06c86c7c744a5594244`.
Mechanical validation confirmed the embedded commit identity, Apple Silicon
executable, deterministic helper, tracked icon, document registration, and
bundle metadata. Human review confirmed that imported DOCX page breaks render
with separate page-surface spacing, the bottom-right build identity is correct,
and About DRAFT shows the expected version and build edition. That closes only
the explicit page-surface defect. Clean and Unsaved title transitions, Save
suggestions, cancellation, post-save identity, and complete fidelity remain
pending.

The same review confirmed that Save As does not offer `.draft`, `.docx`, and
`.txt` format choices and that DOCX output remains a separate Export action.
That is a failed multi-format workflow result, not a failure of the existing
atomic DOCX exporter. This candidate does not implement format selection,
future sidebar placeholders, spelling, or inferred pagination. PDF remains
unavailable under accepted ADR-001 and cannot be added to this workflow without
its prerequisite rendering policies and a separately accepted implementation
boundary. `UX-47-013` and `UX-47-015` through `UX-47-019`, `RC-07`, `GATE-47`,
and all release gates remain open.

The current PR candidate replaces that failed workflow mechanically. One typed
Save As selector offers exactly DRAFT, Word, and plain text. DRAFT output may
rebind only after atomic persistence; Word and text are converted copies that
preserve document identity and dirty state. The standalone DOCX command and
menu action are removed. This is implementation evidence only. `UX-47-017`
remains open until a newly hashed package proves selection, filename handling,
cancellation, failure recovery, and non-rebinding converted output.

That replacement package was built from exact implementation commit
`27fe00ac9cd83cb58107a8f978761a311dfdd1d3`. Its Apple Silicon executable
SHA-256 is
`fa72b0c71414f135cfba40f6216e50d0efb8f371bc8b8421341cf939c2319898`.
Mechanical validation confirmed the embedded commit identity, bundle contract,
tracked icon, and deterministic helper. Human validation is pending. This
record closes no finding, RC row, or release gate.

### Findings And Dispositions

| ID | Severity | Status | Evidence | Disposition |
| :--- | :--- | :--- | :--- | :--- |
| UX-46-001 | UX-0 | Closed | Artifact `154c34c` beach-balled during Save and required force-quit. Direct validation of artifact `3b4e9960` confirmed Save works as expected through the asynchronous dialog path. | Closed for the specific unresponsive Save defect; the broader packaged workflow and RC rows remain open. |
| UX-46-002 | UX-2 | Open | The first artifact exposed no font-family control. The corrected candidate exposed a bounded control, but its family list was too narrow for the intended workflow. | Expand one canonical allowlist across validation, toolbar, editor marks, persistence, and DOCX mapping; then rebuild, rehash, and validate persistence and export in the corrected package. |
| UX-46-003 | UX-2 | Open | The packaged editor exposed no font-size control. | Add a bounded point-size control only if the DRAFT envelope persists it, reopen restores it, and DOCX preserves it accurately. Rebuild, rehash, and validate the corrected package before disposition. |
| UX-46-004 | UX-2 | Closed | The earlier package opened seeded content. Direct validation of artifact `3b4e9960` confirmed New now opens the expected blank document. | Closed for the specific New-document defect; remaining lifecycle and release evidence stays open. |
| UX-46-005 | UX-2 | Open | Open did not offer plain-text files. | Import bounded UTF-8 `.txt` as a new unsaved Rust-owned envelope, preserve the source, and validate first Save to a new `.draft` target. |
| UX-46-006 | UX-2 | Open | Open did not offer Markdown files. | Import bounded UTF-8 `.md` as literal editable text without parsing or preview claims, then validate the packaged workflow. |
| UX-46-007 | UX-1 | Open | Text and Markdown import source-preservation behavior could not be exercised because those inputs were unavailable. | Prove the source path never becomes save authority, first Save selects a new `.draft` target, and the source remains byte-for-byte unchanged in automated and packaged tests. |
| UX-46-008 | UX-2 | Open | The replacement artifact contained the tracked bundle icon, but the application icon did not render correctly in the visible packaged window chrome. | Keep bundle-icon validation separate from visible branding; inspect the title-bar/header asset path and validate the corrected packaged window in light and dark appearance. |
| UX-46-009 | UX-1 | Open | File, research, review, and export controls share one sparse command row without sufficient grouping or predictable desktop-editor hierarchy, and native macOS menu integration does not exist. | Move this release-blocking workflow problem to a governed desktop UI phase with grouped controls, native menus, state-sensitive enablement, responsive overflow, and shared action dispatch. |
| UX-46-010 | UX-1 | Open | Markdown opens as literal source, so headings, emphasis, lists, quotations, links, code, and separators are not represented as editable document structure. | Define and implement a bounded Markdown parser/serializer contract in the governed interoperability phase; unsupported constructs must fail or disclose loss rather than disappear. |
| UX-46-011 | UX-1 | Open | Phase 47 mechanically implements bounded DOCX paragraph import, closed fidelity categories, source preservation, and typed failure behavior. The `a60f877` packaged gate did not establish reliable normalized or complete compatible-reader output. | Keep open until the accepted interoperability contract has reproducible fixtures, compatible-reader, packaged, and human evidence for the supported subset and explicit unsupported-content behavior. |
| UX-46-012 | UX-2 | Open | RTF import and save are unavailable. | The interoperability decision must either implement a bounded RTF subset or accept an explicit v1 deferral with user guidance. |
| UX-46-013 | UX-2 | Open | OpenDocument import and save are unavailable. | The interoperability decision must either implement bounded ODT support or accept an explicit v1 deferral with user guidance. |
| UX-46-014 | UX-1 | Open | Phase 47 exposes one path-free Save Back workflow over the Rust-owned DOCX identity, fingerprint, fidelity, and writer boundary. The `a60f877` package passed exact replacement, cancellation, and safe-copy observations, but failed normalized consent, stale-source presentation, entry-point parity, and complete reader evidence. | Keep open until a replacement package proves confirmation, cancellation, stale-source denial, failure recovery, source identity, display-name behavior, and compatible-reader fidelity. |
| UX-46-015 | UX-2 | Open | Command spacing, grouping, editor canvas composition, and outline layout do not meet the intended desktop-product quality threshold. | Address visual hierarchy and layout in the governed desktop UI phase, then validate normal, narrow, scaled, keyboard, and reduced-motion states from the packaged app. |
| UX-46-016 | UX-1 | Open | Manual review confirmed that the native File menu does not expose New Document, Open, Close, Save, Save As, and Export in expected desktop-document order with state-aware shortcuts. | Phase 48 must implement one shared dispatcher for native and visible commands, standard macOS shortcuts, and document-state enablement before packaged retest. |
| UX-46-017 | UX-1 | Open | Manual review confirmed that the visible menu icon remains stale and toolbar grouping does not consistently separate document lifecycle commands from research and analysis commands. | Phase 48 must replace the stale icon, align labels and icons, remove conflicting command locations, and validate the resulting hierarchy in the packaged app. |
| UX-46-018 | UX-1 | Closed | An earlier package presented Open during Save and did not show the selected filename. Direct validation of artifact `3b4e9960` confirmed Save works as expected with the corrected typed result and visible filename transition. | Closed for the specific save-panel and filename defect; complete packaged recovery evidence remains required by the open RC rows. |
| UX-46-019 | UX-1 | Closed | Artifact `8870dcc4` displayed the effective current font family and size during direct packaged retest. | Closed for the specific false-default control state; the complete eight-step workflow and release rows remain open. |
| UX-46-020 | UX-1 | Closed | Artifact `8870dcc4` exposed only Close Window in the native macOS File menu. Direct manual review of artifact `b9ebc25e` confirmed that the native File menu now provides a usable document workflow. | Closed for the native File-menu hierarchy. Other toolbar, state, icon, and complete-workflow evidence remains open. |
| UX-46-021 | UX-1 | Closed | Exact artifact `75373ffb` passed direct review of the purple identity in the application, Finder, Dock, and application switcher after the canonical icon chain passed mechanical comparison. | Closed for the Phase 48 identity defect. This does not approve a final UI design or final release package. |
| UX-46-022 | UX-1 | Closed | Exact artifact `75373ffb` passed direct review of the compact top bar, icon-only common actions, overflow behavior, shortcuts, focus, state-sensitive enablement, shared dispatch, and narrow-window behavior. | Closed for the over-labeled Phase 48 command-bar defect. The documented v3 workspace remains a later design target. |
| UX-46-023 | UX-2 | Closed | Exact artifact `75373ffb` kept document, connectivity, operation, and recovery state in the bottom status bar during direct packaged review. | Closed for the Phase 48 header-status placement defect. Later status-bar refinements remain subject to the v3 target and release validation. |
| UX-46-024 | UX-1 | Open - implementation and evidence pending | Manual review found that the editor lacks the paragraph controls required for alignment, line and paragraph spacing, and indentation. ADR-004 is accepted, and Phase 47 implements the canonical data, migration, persistence, editor-preservation, and DOCX-export foundation. | Keep the finding open until commands, mixed-selection and reset behavior, external-format fidelity, visible controls, accessibility, and packaged manual evidence satisfy the accepted contract. The model foundation alone does not close the finding. |
| UX-47-001 | UX-1 | Open | Artifact `910a204f` did not provide a usable normalized-replacement warning and consent flow backed by compatible-reader output. | Enumerate the known normalization, present explicit Replace and Cancel choices, preserve state on cancellation, and prove accepted output without silent unsupported-content removal. |
| UX-47-002 | UX-0 | Open | External modification did not surface the required stale-source rejection in artifact `910a204f`. | Prove the real import-to-registry fingerprint, immediate pre-write recheck, source and registry non-mutation, and path-free reopen guidance in automated and packaged evidence. |
| UX-47-003 | UX-1 | Open | Native and overflow Save Back entry points did not demonstrate equivalent exact, normalized, busy, stale, cancellation, and denial behavior. | Route both through one state-aware dispatcher and one typed eligibility result, then retest both surfaces. |
| UX-47-004 | UX-0 | Open | Normalized output and the complete exact/normalized set did not pass compatible-reader validation. | Record the reader, deterministic source hash, replacement hash, observed rendering result, and source mutation disposition for both supported classes. |
| UX-47-005 | UX-2 | Open - partial correction pending package | The reset option now says `Reset to document font` or `Reset to document size`; missing-font fallback policy remains undefined. | Retest the corrected labels in the replacement package; keep substitution or unavailable-font handling blocked until a deterministic fidelity policy is accepted and implemented. |
| UX-47-006 | UX-2 | Open - correction pending package | Rust now returns a closed Markdown import format and the visible notice says Markdown remains literal text without parsing or preview. | Retest the notice and source preservation in the replacement package without adding parsing behavior. |
| UX-47-007 | UX-2 | Open - correction pending package | The DOCX safety rejection now identifies package, XML, or document-size limits and suggests reducing large embedded content without exposing internal detail. | Retest the bounded recovery copy while retaining the exact typed safety reason only in maintainer and test evidence. |
| UX-47-008 | UX-2 | Open | Native `.draft` files use a generic desktop identity and have no friendly application association. The structured JSON envelope is not intended as a prose format. | Assign file-association and icon work to the desktop packaging boundary; do not redesign `.draft` as plain text or claim human-readable source formatting. |
| UX-47-009 | UX-1 | Open - failed artifact proves identity only | Artifact `c3b2b54c` visibly reported commit `7ec149de` and release profile, proving that the newer package was running. The same artifact failed the primary DOCX workflow and cannot close a Phase 47 finding. | Confirm the visible version, short commit, profile, and executable hash again on the corrected replacement package. |
| UX-47-010 | UX-0 | Closed - artifact 2dfe312b | Artifact `c3b2b54c` exposed the relationship-target parser defect. Artifact `8e974736` exposed stale export feedback. Artifact `2dfe312b`, visibly identified as implementation commit `e734cae`, opened the selected DOCX into a readable imported document. | Closed only for basic DOCX Open. Formatting fidelity moved to `UX-47-013`; unsupported, safety-limit, malformed, cancellation, recovery, `RC-07`, and `GATE-47` remain open. |
| UX-47-011 | UX-0 | Closed - artifact 8e974736 | Artifact `8e974736`, visibly identified as commit `14363903` in the release profile, produced a DOCX export and displayed the success disposition. The existing atomic round-trip reopens the output and matches visible text in LibreOffice. | Closed for the packaged export failure observed on `c3b2b54c`. This does not close DOCX Open, source-replacement evidence, `RC-07`, `GATE-47`, or any release gate. |
| UX-47-012 | UX-1 | Open - manual retest pending | The `.draft` envelope remained a generic JSON document with no verified DRAFT desktop association or double-click workflow. | Replacement package `c3b2b54c` declares the owned UTI and icon and routes activation through Rust; confirm Finder identity and double-click opening. |
| UX-47-013 | UX-0 | Open - packaged fidelity retest pending | Artifact `2dfe312b` opened readable DOCX text but flattened explicit Times New Roman 12-point runs, bold and italic spans, paragraph appearance, page breaks, and source-recognizable academic formatting. Candidate `1634d6d2` mechanically contains the accepted run, paragraph, and page-break corrections. | Compare the source and exported output from the exact package in Word or LibreOffice and confirm the original source hash remains unchanged without guessed semantic headings or unsupported font substitution. |
| UX-47-014 | UX-1 | Closed - artifact 1634d6d2 | Artifact `91fe1ba9` rendered an explicit page break as a dashed marker inside one continuous page surface. Direct review of artifact `1634d6d2` confirmed that imported canonical page breaks now render with visible spacing between distinct page surfaces. | Closed only for explicit canonical page-break presentation. DRAFT still does not infer pagination from layout, content flow, fonts, margins, or printer geometry, and complete DOCX fidelity remains open. |
| UX-47-015 | UX-1 | Open - partial artifact pass | Artifact `91fe1ba9` did not place Unsaved beside the document name and kept build identity in the document inspector. Direct review of artifact `1634d6d2` confirmed the bottom-right build identity and About DRAFT version/build edition, but did not complete clean and Unsaved title-transition evidence. | Retest basename plus Unsaved state in both in-window and native titles across new, imported, saved, modified, cancelled, and reopened states. |
| UX-47-016 | UX-1 | Open - packaged retest pending | Save and Save As did not consistently suggest the current document basename; imported and new sessions could fall back to an unhelpful Untitled name. Candidate `1634d6d2` mechanically contains the Rust-owned suggestion correction. | Retest existing, imported, and new suggestions, state preservation on cancellation, and immediate basename updates after success. |
| UX-47-017 | UX-1 | Open - packaged failure; governance required | Direct review of artifact `1634d6d2` confirmed that Save As had no `.draft`, `.docx`, or `.txt` selector. The current PR candidate implements that exact closed selector and retires the standalone DOCX action, but has no replacement-package evidence. | Retest all three choices, cancellation, invalid targets, visible completion, and authority/dirty-state preservation. PDF remains excluded by ADR-001 until its rendering prerequisites and a separate implementation boundary are accepted. |
| UX-47-018 | UX-2 | Open - future workspace scope | The proposed research and analysis sidebar regions are not present. | Keep unsupported capabilities absent until their commands and contracts exist. A later accepted workspace implementation may use clearly disabled placeholders without implying availability. |
| UX-47-019 | UX-2 | Open - future governed capability | DRAFT has no spelling highlight, suggestion, or correction workflow. | Define dictionary source, locale, privacy, document-mark, ignore, correction, undo, accessibility, and failure behavior before implementation. |

Every RC and GATE row remains open. Closing the isolated Open and export
failures does not close source-format fidelity, a release blocker, or a roadmap
gate.
