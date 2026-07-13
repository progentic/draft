# Workspace UI

## Status

The frontend exposes one local writing workspace with explicit document
lifecycle, manual references and citation insertion, formatting review, five
local text checks, DOCX export, an outline, statistics, and Rust runtime status.
Rust remains authoritative for persistence, filesystem dialogs, reference
storage, helper execution, and export.

## Component Ownership

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| Shell | `DraftWorkspace` | Owns outline visibility, one active workflow panel, and workspace composition. |
| Header | `WorkspaceHeader` | Exposes the application heading, current document title and lifecycle status. |
| Document actions | `WorkspaceCommandBar` | Exposes labeled lifecycle, reference, review, export, and close commands. |
| Document session | `useDocumentSession`, `UnsavedChangesDialog` | Coordinates explicit snapshots, Rust-owned commands, dirty state, and recovery choices. |
| Outline | `DocumentOutline` | Derives headings from the current Tiptap state and moves the selection. |
| Editor | `DraftEditor`, `useDraftEditor` | Owns the transient Tiptap instance and initial document. |
| Formatting | `EditorToolbar`, `TextFormattingMarks` | Invokes Tiptap commands, reports selected state, and constrains font family and size to the document contract. |
| Formatting review | `FormattingReviewPanel`, `useFormattingReview` | Runs bounded checks and owns transient review state and explicit actions. |
| References | `ReferenceLibraryPanel` | Adds bounded manual records and inserts existing citation nodes. |
| Text checks | `TextAnalysisPanel`, `textAnalysisSnapshot` | Runs five fixed checks, invalidates stale generations, and locates passages. |
| DOCX export | `useDocxExport` | Presents Rust-owned export and source-safety results. |
| Inspector | `DocumentInspector` | Derives session metrics and maps runtime status to visible copy. |
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

Formatting, reference, and text-check panels are labeled sections controlled by
their corresponding buttons. Closed panels use `hidden`. Open panels expose
headings, plain-language controls, and live status messages.

The unsaved-changes surface is an `alertdialog`. It receives focus, contains
Tab navigation, handles Escape as Keep editing, and restores focus to the
invoking action.

## Toolbar Keyboard Contract

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

The document session stores one explicit origin: `new`, `imported_text`, or
`opened_draft`. New and imported sessions both lack a path, but the latter
shows the source filename with `Imported, unsaved`. The filename is display
metadata only. No path enters React, and only successful Save changes either
origin to native persisted DRAFT state.

## Connectivity Control

The header contains one binary session toggle. `Work offline` represents the
online state; `Go online` represents offline and uses `aria-pressed="true"`.
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

The header, toolbar controls, panels, and editor use constrained dimensions so
status text, icons, focus rings, and changing metrics do not resize the primary
layout unexpectedly.

## Visible Runtime Messages

| State | Visible message | User guidance |
| :--- | :--- | :--- |
| Checking | `Connecting to core` | Wait for desktop startup. |
| Ready | `Core v<version>` | No action required. |
| Transport unavailable | `Core unavailable` | Use the desktop app or restart it. Browser preview has no Rust core. |
| Invalid payload or response | `Core status invalid` | Restart DRAFT; report the version if it repeats. |
| `invalid_application_version` | `DRAFT received an unsupported application version.` | Install a matching DRAFT build. |
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
movement, focus visibility, or rendered landmark order.

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
- DOCX export rejects citation nodes and other unsupported content.
- PDF intake, metadata lookup, diagnostics, credentials, provider-backed
  orchestration, and background jobs remain without visible workflows.
