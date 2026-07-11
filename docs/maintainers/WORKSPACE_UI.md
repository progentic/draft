# Workspace UI

## Status

The implemented frontend is one transient local writing workspace. It exposes
editing, formatting controls, a transient formatting review, an outline,
document statistics, and Rust runtime status. It does not expose durable
document, research, analysis, import, export, or background-job workflows.

## Component Ownership

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| Shell | `DraftWorkspace` | Owns outline visibility and composes the workspace. |
| Header | `WorkspaceHeader` | Exposes the outline toggle, application heading, document label, and unsaved status. |
| Outline | `DocumentOutline` | Derives headings from the current Tiptap state and moves the selection. |
| Editor | `DraftEditor`, `useDraftEditor` | Owns the transient Tiptap instance and initial document. |
| Formatting | `EditorToolbar` | Invokes existing Tiptap commands and reports selected state. |
| Formatting review | `FormattingReviewPanel`, `useFormattingReview` | Runs bounded checks and owns transient review state and explicit actions. |
| Inspector | `DocumentInspector` | Derives session metrics and maps runtime status to visible copy. |
| Runtime session | `useRuntimeStatus`, `startRuntimeStatusSession` | Coordinates the typed command and event wrappers without adding durable state. |
| Connectivity session | `useConnectivityMode`, `ConnectivityModeControl` | Mirrors and changes the Rust-owned online/offline session policy. |

All editor content, outline visibility, selection, and metrics are WebView state.
Reloading the WebView discards them. Rust remains authoritative for every
durable or privileged operation even though those commands are not wired to
workspace controls yet.

## Semantic Structure

The shell renders one `main` landmark named `DRAFT workspace`. The visible DRAFT
wordmark is the application-level `h1`. Outline and document panels use `h2`
headings; their internal sections use `h3`. A user-created heading inside the
editor remains document content inside the named textbox and does not replace
the shell heading.

The outline and inspector are complementary landmarks with distinct accessible
names. The hidden outline uses both `aria-hidden` and `inert`, so its controls
leave the focus order when the panel is closed.

The formatting review is a labeled section controlled by the toolbar button.
When closed it uses `hidden` and `inert`. Its style fieldset, status messages,
group headings, command labels, and finding-specific dismiss labels remain
available to assistive technology when the panel is open.

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

## Connectivity Control

The header contains one binary session toggle. `Work offline` represents the
online state; `Go online` represents offline and uses `aria-pressed="true"`.
The control remains available when the document inspector is hidden at narrow
widths. Pending changes disable repeat activation. Failed changes retain the
last confirmed visible mode and announce a bounded alert; an unreadable initial
mode offers a retry.

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

- The header always identifies an untitled, unsaved document.
- No create, open, save, close, reopen, autosave, or recovery control exists.
- Citation rendering has no insertion or library workflow.
- Rust analysis, import, and export boundaries have no controls.
- Formatting review is advisory and does not provide full style conformance,
  persistence, citation conversion, or automatic repair.
- No product worker emits progress into the workspace.
