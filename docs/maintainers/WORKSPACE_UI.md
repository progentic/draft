# Workspace UI

## Status

The frontend exposes one local writing workspace with explicit document
lifecycle, manual references and citation insertion, formatting review, five
local text checks, DOCX output through Save As, an outline, statistics, and Rust runtime status.
Rust remains authoritative for persistence, filesystem dialogs, reference
storage, helper execution, and export.

## V3 Workspace Target

This section records the intended workspace direction. It is a layout target,
not evidence that every named command or panel capability exists. Current
behavior remains defined by the implementation sections below and by accepted
contracts. A target item cannot become active merely because its control has
been drawn.

### Layout Work

The native macOS menu bar is the complete command hierarchy. The target top
level is File, Edit, View, Insert, Format, Tools, Citations, and Help. The target
File menu contains New Document, Open…, Recent, Save, Save As…, conditional Save
Back to Source, Export, and Close Window in conventional groups. The current
implementation guarantees only the accepted File actions documented under
Native File Actions; Recent and additional menu groups remain unavailable until
real commands and state rules exist.

The window title is centered and follows `<basename> — DRAFT` for a clean saved
document and `<basename> — Unsaved — DRAFT` for modified or imported content.
A new document uses `Untitled document — Unsaved — DRAFT`. The in-window title
shows the same basename and Unsaved state. This is presentation metadata only
and cannot become path or persistence authority.

The in-window command strip stays smaller than the native menu. Its v3 primary
set is New, Open, Save, and More. Close, Save As, Export, References, and Text
checks belong in the overflow or native menu rather than competing with the
writing surface. New may keep a short visible label; familiar actions use icons
with accessible names and tooltips. This target moves the current primary Close
control into overflow when the v3 layout is implemented.

The target shell has three clear regions:

- a left library and citation region;
- a centered document canvas; and
- a right research and review region.

The left region may expose the currently implemented manual reference list, Add
Reference, and Insert Citation actions. Search, All/Recent/Favorites tabs,
collection counts, richer reference cards, and library management are capability
work and must not appear active before their data and command contracts exist.

The center region keeps the page visually dominant. A centered white page,
readable margins, restrained shadow, visible caret, and citation chips are
layout work. A ruler may appear only as an accurate, non-interactive page guide
until paragraph indents, tabs, and page geometry have accepted behavior; it must
not look draggable before those actions exist.

The right region is reserved for supported research and review workflows. Local
Formatting review and Text checks may use it. Source search, result filters,
Summarize, Paraphrase, Find Related, Outline, bibliography generation, citation
conversion, DOI lookup, and source chat are capability work. Provider-backed or
model-backed actions remain excluded from v1 under ADR-002 and must not appear
active.

The bottom status bar remains the only compact operational strip. It may show
document state, active operation, compact runtime identity, connectivity mode, and real
word or character counts when space permits. Page count cannot appear until a
pagination model exists. Active style or document mode appears only when the
application has a real selectable state to report. Status values do not return
to the primary command strip. The title may repeat only the standard Unsaved
document state needed to identify the current native window.

### Capability Work

The v3 formatting direction groups font family, font size, inline formatting,
paragraph formatting, case, color, highlight, zoom, and sharing in a familiar
order. Only the implemented subset may be active today: font family, whole-point
font size, bold, italic, and strikethrough, plus the existing history, heading,
list, blockquote, and formatting-review controls.

The following require separate capability work before activation:

- Underline needs persistence, reopen, validation, paste, and DOCX behavior.
- Alignment, justification, line spacing, paragraph spacing, and indentation
  remain governed by the paragraph-formatting contract.
- Case Shift needs deterministic selection and undo behavior.
- Text color and highlight need canonical values, persistence, paste, and DOCX
  mappings.
- Zoom needs a bounded viewport-only policy that cannot mutate document data.
- Share needs an accepted export or handoff workflow and privacy boundary.
- Research, bibliography, DOI, and source-chat actions need their own accepted
  data, network, failure, privacy, and evidence contracts.

Pure presentation changes may arrange space for accepted controls, but they do
not authorize document marks, paragraph attributes, external requests, model
actions, citation mutation, or sharing.

## Availability And Visual Noise Policy

The workspace uses one rule for every menu, toolbar, panel, and status surface:

> A control may be enabled only when its complete operation exists and its
> current state permits the operation.

An implemented command that is temporarily unavailable because of document
state, a pending operation, or missing selection stays in its stable location
and is disabled with native or semantic disabled behavior. Its visual treatment
is neutral gray with no active accent, pressed state, or misleading hover state.
The disabled control cannot dispatch through pointer, keyboard, native menu, or
stale event delivery.

An unsupported or unapproved capability is omitted from the production
workspace. A future accepted layout may reserve its compact control as disabled,
but only when stable placement materially helps command discovery. Do not fill
panels with disabled feature cards merely to preview a roadmap. In particular,
unavailable AI, provider, bibliography, DOI, sharing, paragraph, and library
management actions must not make the current product look more capable than it
is.

When an accepted v3 target control is present before its operation is available,
the control stays disabled and gray. It is not replaced by an explanation,
empty-state card, warning banner, or promotional message. One unavailable
function should consume no more space than its normal compact control.

Unavailable state does not justify permanent explanatory copy. Do not add
`Coming soon` labels, promotional cards, instructional paragraphs, duplicate
status badges, or capability disclaimers to the writing chrome. When a reason
or recovery action is necessary, use one concise tooltip, menu hint, or existing
status/error region. The message must name a recovery action the current product
can honor; otherwise the disabled visual state is sufficient.

## Component Ownership

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| Shell | `DraftWorkspace` | Owns outline visibility, one active workflow panel, workspace composition, and placement of the bottom status bar. |
| Header | `WorkspaceHeader` | Exposes application identity, basename-only document title, and transient Unsaved state. It owns no save target, import, operation, or connectivity authority. |
| Document actions | `WorkspaceCommandBar` | Exposes compact primary document actions and an accessible overflow menu. All actions use the shared workspace dispatcher and mirror the native File menu where applicable. |
| Shared actions | `useWorkspaceActions` | Routes toolbar and validated native-menu actions through one availability policy. |
| Status | `WorkspaceStatusBar` | Displays document lifecycle, import state, active operation, connectivity mode, and compact build identity without taking durable authority. |
| Document session | `useDocumentSession`, `UnsavedChangesDialog` | Coordinates explicit snapshots, Rust-owned commands, dirty state, and recovery choices. |
| Outline | `DocumentOutline` | Derives headings from the current Tiptap state and moves the selection. |
| Editor | `DraftEditor`, `useDraftEditor` | Owns the transient Tiptap instance and initial document. |
| Formatting | `EditorToolbar`, `TextFormattingMarks` | Invokes Tiptap commands, reports selected state, and constrains font family and size to the document contract. |
| Formatting review | `FormattingReviewPanel`, `useFormattingReview` | Runs bounded checks and owns transient review state and explicit actions. |
| References | `ReferenceLibraryPanel` | Adds bounded manual records and inserts existing citation nodes. |
| Text checks | `TextAnalysisPanel`, `textAnalysisSnapshot` | Runs five fixed checks, invalidates stale generations, and locates passages. |
| Save As output | `useDocumentSession`, `SaveAsDialog` | Presents the closed DRAFT, DOCX, or text choice and Rust-owned results. |
| Inspector | `DocumentInspector` | Derives document word, character, and heading metrics only. |
| Native title | `useWindowTitle`, `windowTitle.ts`, `set_window_title` | Mirrors validated basename and Unsaved state into the native window without receiving a path. |
| Runtime session | `useRuntimeStatus`, `startRuntimeStatusSession` | Coordinates the typed command and event wrappers without adding durable state. |
| Connectivity session | `useConnectivityMode`, `ConnectivityModeControl` | Mirrors and changes the Rust-owned online/offline session policy. |
| Error presentation | `errorPresentation.ts` | Maps only visible typed failures to stable copy and recovery dispositions. |

Outline visibility, selection, metrics, panel state, and findings are transient
WebView state. Document and reference persistence crosses only typed IPC.

## Semantic Structure

The shell renders one `main` landmark named `DRAFT workspace`. The visible DRAFT
wordmark is the application-level `h1`. Outline and document panels use `h2`
headings; their internal sections use `h3`. A user-created heading inside the
editor remains document content inside the named textbox and does not replace
the shell heading.

The outline and inspector are complementary landmarks with distinct accessible
names. The hidden outline uses both `aria-hidden` and `inert`, so its controls
leave the focus order when the panel is closed.

The compact document controls are a navigation region named `Document actions`.
Their primary controls form a group named `Primary document actions`. The More
trigger exposes `aria-haspopup="menu"`, expanded state, and the controlled menu
identifier. The menu has its own accessible name and uses only `menuitem` and
`menuitemcheckbox` roles.

The bottom status bar is a named `footer` labeled `Workspace status`. Document
and operation labels remain ordinary text. Only bounded feedback uses a polite
`status` live region, while a connectivity failure uses its existing dedicated
alert. The status bar omits the operation label when it would duplicate the
document state, so one transition is not repeated across visible status text.

Formatting, reference, and text-check panels are labeled sections controlled by
their corresponding buttons. Closed panels use `hidden`. Open panels expose
headings, plain-language controls, and live status messages.

The unsaved-changes surface is an `alertdialog`. It receives focus, contains
Tab navigation, handles Escape as Keep editing, and restores focus to the
invoking action.

## Document Action Keyboard Contract

New retains a short visible label. Open, Save, Close, and More are icon-only
controls with visible tooltips and accessible names. Ordinary Tab navigation
reaches each enabled primary action and the overflow trigger.

Enter or Space activates a focused document control. Down Arrow opens the More
menu from its trigger. Within the menu:

- Up and Down Arrow move between enabled items and wrap at each end.
- Home moves to the first enabled item.
- End moves to the last enabled item.
- Enter or Space activates the focused item.
- Escape closes the menu and restores focus to the More trigger.
- disabled items are skipped and cannot dispatch.

Activating a menu item also closes the menu and restores trigger focus. Pointer
dismissal closes the menu without forcing focus. HTML controls leave disabled
actions out of keyboard focus; the native menu may display disabled items under
platform conventions, but neither surface can dispatch them. Primary controls,
overflow items, native menu items, and keyboard shortcuts all call the same
`useWorkspaceActions` dispatcher.

## Formatting Toolbar Keyboard Contract

The formatting toolbar is one named toolbar with grouped history, inline,
structure, and review controls. Exactly one enabled control participates in ordinary Tab
navigation. Within the toolbar:

- Left and Right Arrow move between enabled controls and wrap at each end.
- Home moves to the first enabled control.
- End moves to the last enabled control.
- disabled controls are skipped.

Formatting buttons use toggle semantics only when they represent persistent
selection state. Undo and Redo are commands and do not expose `aria-pressed`.
All icon-only controls have visible focus treatment, accessible names, and
tooltips.

Font family and font size are adjacent labeled select controls rather than
members of the roving button toolbar. Family choices use the eleven canonical
families, and size choices use whole points from 8 through 72. Use document
font and Use document size remove only the corresponding mark. A change
preserves other marks and returns focus to the editor without scrolling it.
`fontControlState.ts` resolves the effective document default, explicit caret
marks, or a mixed-range sentinel from Tiptap state; React does not keep a
separate authoritative formatting value.
Pasted HTML styling is not a font-authority path. The marks parse only
`data-draft-font-family` and `data-draft-font-size`, then revalidate the
canonical value before rendering.

The document session stores one explicit origin: `new`, `imported_text`,
`imported_external`, or `opened_draft`. New and text-import sessions lack Rust
registration. An external DOCX has Rust-owned source registration but no native
`.draft` target. No path enters React. Save changes any non-native origin to
persisted DRAFT state, while confirmed Save Back retains the external origin
and basename display identity.

## Native File Actions

The native File menu and visible command bar use the same action identifiers
and `useWorkspaceActions` dispatcher. Their labels are New Document, Open…,
Close, Save, Save As…, and Save Back to Source. The File menu uses Command-N,
Command-O, Command-W, Command-S, and Shift-Command-S; Save Back has no shortcut.

The dispatcher derives availability from the current document operation. It
checks availability again when a native event arrives, so
an event emitted before a state update cannot begin a stale operation. While a
save, open, close, create, or conversion operation is pending, competing
document and workflow-panel actions remain unavailable. Starting a new
document operation replaces settled feedback so its pending and terminal notice
remains authoritative.

Rust owns native menu objects and receives a bounded six-boolean state request.
No path, content, or document identity enters the menu state. Invalid events or
state-update failures leave the visible toolbar available with bounded recovery
copy. See `NATIVE_DESKTOP_WORKFLOW.md` for the complete contract.

The command bar keeps New visible with a short label. Open, Save, and Close are
icon-only controls with accessible names and tooltips. Save As, conditional Save
Back to Source, References, and Text checks are in the More
document actions menu. The menu skips disabled actions during keyboard
navigation, supports Arrow, Home, End, and Escape, and returns focus to its
trigger. Every item still uses the shared dispatcher; the compact layout adds
no second command path.

The header is reserved for DRAFT identity, the current document basename, and
its standard Unsaved state. A
bottom status bar presents the document lifecycle state, current background
operation, bounded recovery message, connectivity control, and compact build
identity. These values are transient presentation state and create no new
persistence or network authority.

## Connectivity Control

The bottom status bar contains one binary session connectivity control. `Work
offline` represents the online state; `Go online` represents offline and uses
`aria-pressed="true"`.
The control remains available when the document inspector is hidden at narrow
widths. Pending changes disable repeat activation. Failed changes retain the
last confirmed visible mode and announce a bounded alert; an unreadable initial
mode offers a retry.

Command, invalid-response, and transport failures retain distinct messages.
Retry labels refer only to the existing mode control. The mapping contract is
documented in `docs/maintainers/ERROR_UX.md`.

The control reports explicit DRAFT policy, not operating-system reachability.
It does not persist and does not make the frontend authoritative for network or
browser behavior.

## Motion And Layout

Normal workspace panel transitions remain enabled. Under
`prefers-reduced-motion: reduce`, the workspace grid and outline transitions
are removed. The media query changes no spacing, color, or component behavior.

The header, compact document controls, formatting toolbar, panels, status bar,
and editor use constrained dimensions so status text, icons, focus rings, and
changing metrics do not resize the primary layout unexpectedly.

Secondary document actions remain in the overflow menu at every width, which
keeps the command row bounded when the window narrows. The formatting toolbar
may scroll horizontally when its controls need more room. The workspace shell
must not grow beyond the viewport or allow either toolbar to overlap the editor
or status bar.

## Explicit Page Surfaces

DRAFT renders canonical `pageBreak` nodes as distinct page surfaces. The
presentation uses a full-width workspace gap with visible page edges rather
than punctuation or a dashed rule. The node remains one selectable, accessible
separator in the canonical editor document.

DRAFT does not infer page boundaries from content flow, margins, font metrics,
or printer geometry. Content without an explicit `pageBreak` remains on one
continuous canonical surface even when a compatible reader would paginate it.
Automatic pagination, page counts, widows, orphans, and printer-specific layout
remain outside this implementation.

## Visible Runtime Messages

| State | Visible message | User guidance |
| :--- | :--- | :--- |
| Checking | `Checking build` | Wait for desktop startup. |
| Ready | `v<version> · <commit>` | Confirm the short commit against the package under manual review; About DRAFT also shows the profile. |
| Transport unavailable | `Core unavailable` | Use the desktop app or restart it. Browser preview has no Rust core. |
| Invalid payload or response | `Core status invalid` | Restart DRAFT; report the version if it repeats. |
| `invalid_application_version` | `DRAFT received an unsupported application version.` | Install a matching DRAFT build. |
| `invalid_build_metadata` | `DRAFT could not verify this application build.` | Replace it with one complete package and report the visible build identity. |
| `event_delivery_failed` | `DRAFT could not deliver the core status event.` | Restart DRAFT; report the failure if it repeats. |
| Unknown command code | `DRAFT could not read the core status.` | Restart DRAFT and report the version and message. |

The same recovery guidance is written for users in
`docs/wiki/Troubleshooting.md`. Typed errors for unwired commands remain in
`ERROR_MESSAGES.md`; they must not receive speculative visible copy before a
real workflow owns the recovery action.

Phase 39 centralizes presentation for runtime status, connectivity, formatting
review, and citation rendering. Each visible failure is retryable, actionable,
or terminal. Unknown runtime command input uses one outer fallback; validated
known variants remain exhaustive.

Phase 46 feature copy remains owned by each bounded visible workflow. Messages
contain no raw path, source text, helper output, database detail, or internal
error name. See `PHASE46_WORKFLOWS.md`.

Phase 47 uses one conditional operation notice below the document command bar
for Open and Save As pending and terminal outcomes. The notice is absent
when no result exists; compact document, connectivity, and active-operation
state remains in the bottom status bar.

## Formatting Review State

The formatting review represents idle, running, ready, stale, and failed
states. A style change invalidates an existing result. Editor updates and newer
runs invalidate older responses, and target guards reject moved, removed, or
changed nodes. Dismissal changes only the current React result. Heading apply
is available only where Rust returned a bounded level and the user activates
that command; citations remain inspect-only.

## Verification

`src/App.test.tsx` covers shell landmarks, heading order, outline state,
editable content, toolbar semantics, keyboard navigation, disabled-control
skipping, review-panel disclosure, runtime labels, and error distinctions.
It also verifies that document and connectivity state remain in the bottom
status bar rather than the header.

`src/components/WorkspaceCommandBar.test.tsx` covers the compact New action,
icon-only accessible names and tooltips, primary and overflow dispatch, disabled
item skipping, active-panel state, overflow focus order, Escape dismissal, and
trigger-focus restoration. `useWorkspaceActions.test.tsx` proves toolbar and
native-menu parity, stale-action rejection, availability synchronization,
bounded state-update recovery, and contained native-listener setup failure.
`WorkspaceStatusBar.test.tsx` covers document, operation, connectivity, and
recovery placement. The component suppresses a repeated operation label and
keeps bounded feedback separate from connectivity alerts.

Formatting review suites cover IPC validation, generations, editor targets,
interactions, and accessible labels. Other component and hook suites cover
Tiptap, citation rendering, runtime sessions, and typed wrappers.
Connectivity suites cover get/set IPC, stale reads, failure retention, toggle
semantics, retry, and workspace integration.
The Phase 39 policy suite covers every visible typed variant, disposition,
existing-control label, and outer fallback.

Phase 46 suites cover command validation, save/close/reopen, first-save handle
release, unsaved dialog focus, manual reference insertion, text-check
pending/success/stale states, UTF-8 passage mapping, DOCX source safety, and
rendered narrow-window behavior. They also cover font controls, keyboard entry,
selection isolation, JSON round trips, save/close/reopen restoration, invalid
font rejection, and mixed DOCX run properties.

The reduced-motion contract is checked against the production stylesheet.
Browser-level inspection is still required when a change affects real focus
movement, focus visibility, or rendered landmark order. Phase 48 rendered
inspection covers normal and narrow windows, viewport overflow, control
overlap, overflow-menu containment, disabled-item skipping, and visible focus.
Packaged human review remains required for native shortcuts, toolbar/menu
parity, busy-state behavior, status placement, and visible application identity.

Any v3 implementation must add focused evidence that:

- every enabled control completes one implemented operation;
- state-disabled controls are visibly gray, semantically disabled, and cannot
  dispatch through pointer, keyboard, menu, shortcut, or stale event;
- unsupported controls remain absent unless an accepted layout explicitly
  requires a stable disabled position;
- no disabled function adds persistent explanatory or promotional copy;
- command labels, accessible names, tooltips, and native-menu names remain
  consistent; and
- three-panel and toolbar changes preserve normal, narrow, scaled, keyboard,
  reduced-motion, and status-announcement behavior.

Run:

```bash
npm test
npm run typecheck
npm run build:frontend
bash scripts/verify.sh
```

## Current Limits

- There is no autosave, crash recovery, or version history.
- Manual references cannot be edited, deleted, imported, or synchronized from
  the visible workspace.
- Text and formatting findings are advisory, transient, and never automatic.
- Font formatting is limited to eleven named families and whole point sizes from
  8 through 72.
- Paragraph alignment, line spacing, spacing before and after paragraphs, and
  indentation controls are not implemented. The paragraph-formatting finding
  remains implementation- and evidence-blocked and must not become an active
  control until the accepted contract is fully implemented and tested.
- Word Save As rejects citation nodes and other unsupported content.
- The native File menu, compact document controls, bottom status bar, and
  tracked purple icon chain are implemented. Exact artifact `75373ffb` passed
  direct review of overflow interaction, menu parity, busy states, Save As,
  status placement, Finder, Dock, application switcher, in-window branding,
  narrow layout, and keyboard behavior. This is not final UI-design approval;
  `RC-08` and `GATE-48` remain open.
- The v3 menu hierarchy, centered window title, three-panel composition, richer
  formatting order, ruler, library organization, and research region are target
  direction only. They are not current capability or completion evidence.
- PDF intake, metadata lookup, diagnostics, credentials, provider-backed
  orchestration, and background jobs remain without visible workflows.
