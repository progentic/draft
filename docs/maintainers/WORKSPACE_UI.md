# Workspace UI

## Status

The implemented frontend is one transient local writing workspace. It exposes
editing, formatting controls, an outline, document statistics, and Rust runtime
status. It does not expose durable document, research, analysis, formatting-
check, import, export, or background-job workflows.

## Component Ownership

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| Shell | `DraftWorkspace` | Owns outline visibility and composes the workspace. |
| Header | `WorkspaceHeader` | Exposes the outline toggle, application heading, document label, and unsaved status. |
| Outline | `DocumentOutline` | Derives headings from the current Tiptap state and moves the selection. |
| Editor | `DraftEditor`, `useDraftEditor` | Owns the transient Tiptap instance and initial document. |
| Formatting | `EditorToolbar` | Invokes existing Tiptap commands and reports selected state. |
| Inspector | `DocumentInspector` | Derives session metrics and maps runtime status to visible copy. |
| Runtime session | `useRuntimeStatus`, `startRuntimeStatusSession` | Coordinates the typed command and event wrappers without adding durable state. |

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

## Toolbar Keyboard Contract

The formatting toolbar is one named toolbar with grouped history, inline, and
structure controls. Exactly one enabled control participates in ordinary Tab
navigation. Within the toolbar:

- Left and Right Arrow move between enabled controls and wrap at each end.
- Home moves to the first enabled control.
- End moves to the last enabled control.
- disabled controls are skipped.

Formatting buttons use toggle semantics only when they represent persistent
selection state. Undo and Redo are commands and do not expose `aria-pressed`.
All icon-only controls have visible focus treatment, accessible names, and
tooltips.

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

## Verification

`src/App.test.tsx` covers shell landmarks, heading order, outline state,
editable content, toolbar semantics, keyboard navigation, disabled-control
skipping, runtime labels, and error distinctions. Component and hook suites
cover Tiptap, citation rendering, runtime sessions, and typed wrappers.

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
- Rust formatting, analysis, import, and export boundaries have no controls.
- No product worker emits progress into the workspace.
