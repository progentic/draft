# Native Desktop Workflow

## Purpose

DRAFT gives macOS users the document commands they expect in the File menu and
keeps those commands consistent with the visible toolbar. This guide explains
how the two surfaces stay synchronized without moving file authority into the
frontend.

## Problem

A toolbar alone does not provide a conventional desktop document workflow.
Users expect New, Open, Close, Save, Save As, and Export in the macOS menu bar
with familiar shortcuts and accurate disabled states. Separate menu and toolbar
implementations would eventually behave differently and could bypass the
Rust-owned document lifecycle.

## Solution

Rust creates the native menu and emits one closed action identifier when a user
chooses an item. React validates that event and sends it through the same
workspace action dispatcher used by the toolbar. React sends only transient
booleans back to Rust so native items reflect current availability. Paths,
document contents, and persistence decisions never enter that state message.

Save As uses the existing typed save command with `mode: save_as`. Rust opens
the save panel, writes atomically, preserves the prior file, and rebinds the
registry only after the replacement target is complete. The frontend receives
the document ID, basename display name, and Save As flag, but no path.

## Trade-offs

The WebView remains responsible for transient availability because it knows the
active editor and pending operation state. Rust remains responsible for the
native menu objects and all trusted work. A failed availability update can
temporarily leave a native item stale, so the dispatcher checks availability
again before invoking any action and shows guidance to use the toolbar.

The first Phase 48 increment covers the File menu. Other accepted desktop menu
groups must be added only when their shipped actions can use the same
dispatcher and state rules.

## Technical Contract

The File menu order is:

1. New Document - Command-N
2. Open… - Command-O
3. Close - Command-W
4. separator
5. Save - Command-S
6. Save As… - Shift-Command-S
7. separator
8. Export DOCX… - Shift-Command-E

`src-tauri/src/desktop_menu.rs` owns menu construction, stable identifiers,
initial enablement, shortcuts, and typed event emission.
`set_native_menu_state` accepts exactly six booleans. It cannot receive a path,
document ID, source text, or arbitrary menu identifier.

`useWorkspaceActions` is the only frontend policy layer that maps File menu and
toolbar actions to document-session or export operations. It rejects disabled
and stale actions before they reach a typed command. `WorkspaceCommandBar`
contains no direct save, open, close, export, registry, or path authority.

Save and Save As are distinct requests. Save reuses the Rust-owned target when
one exists. Save As always requests a new `.draft` target, preserves the old
file, and makes the new target authoritative only after a successful atomic
write. Cancellation and pre-replacement failure leave the current target and
visible identity unchanged.

The canonical icon source is `assets/DRAFT_Logo.png`. Generated Tauri assets
live under `src-tauri/icons/`; the in-window mark uses the generated 32-pixel
asset. The package must embed `icon.icns` and declare it through
`CFBundleIconFile`.

## Implementation Notes

Native menu events use `draft://native-menu-action`. The typed frontend wrapper
accepts only the six File actions. `NativeMenuItems` starts with New and Open
enabled and every document-dependent action disabled until the frontend sends
current state.

The shared dispatcher also groups the existing References and Text checks
toolbar actions separately from document lifecycle and export actions. Those
panel actions are not added to the native File menu.

## Failure Modes

- An invalid native event or listener setup failure is contained and the
  toolbar remains available.
- A menu-state command failure shows bounded recovery guidance without raw
  platform or path details.
- A disabled or stale action does nothing when received.
- Save As cancellation keeps the current filename, dirty state, and Rust target.
- A failed Save As write preserves both the prior file and registry authority.
- A stale Finder or Dock cache can display an old icon even when the package is
  correct; inspect the bundle resource before clearing macOS caches.

## Tests

Rust tests pin menu order, labels, shortcuts, identifiers, initial state, typed
command serialization, Save As source preservation, target rebinding, and
cancellation. Frontend tests validate event payloads, state responses, shared
dispatch, stale-action rejection, busy-state behavior, visible label parity,
and Save versus Save As requests.

`scripts/check-invariants.sh` enforces the menu files, action set, shared
dispatcher, path-free frontend boundary, and absence of direct toolbar document
authority. `scripts/check-packaging.sh` pins the canonical source hash, stable
desktop derivatives, explicit bundle paths, and macOS icon container.

The unsigned package command verifies `Info.plist`, the embedded icon, and the
Apple Silicon executable. Finder, Dock, application switcher, native menu, and
window branding still require manual packaged validation. `RC-08` and
`GATE-48` remain open until that evidence exists.

## Related Documents

- `docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/adr/003-expand-v1-document-interoperability.md`
- `docs/maintainers/DOCUMENT_SAVE_LOAD.md`
- `docs/maintainers/PACKAGING.md`
- `docs/maintainers/WORKSPACE_UI.md`
- `docs/maintainers/RELEASE_CANDIDATE.md`
