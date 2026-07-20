# Native Desktop Workflow

## Purpose

DRAFT gives macOS users the document commands they expect in the File menu and
keeps those commands consistent with the visible toolbar. This guide explains
how the two surfaces stay synchronized without moving file authority into the
frontend.

## Problem

A toolbar alone does not provide a conventional desktop document workflow.
Users expect New, Open, Close, Save, Save As, and source replacement in
the macOS menu bar with familiar shortcuts and accurate disabled states.
Separate menu and toolbar implementations would eventually behave differently
and could bypass the Rust-owned document lifecycle.

## Solution

Rust creates the native menu and emits one closed action identifier when a user
chooses an item. React validates that event and sends it through the same
workspace action dispatcher used by the toolbar. React sends only transient
booleans back to Rust so native items reflect current availability. Paths,
document contents, and persistence decisions never enter that state message.

Save As uses the existing typed save command with `mode: save_as` and one
closed DRAFT, Word, or plain-text format. Rust opens a format-constrained save
panel. DRAFT output writes atomically and rebinds only after success. Word and
plain-text output are converted copies that preserve document identity and
dirty state. The frontend receives basename-level result data, but no path.

Rust also chooses the save-panel filename suggestion. The typed request carries
the basename display name and closed lifecycle origin that Rust originally
returned. Existing DRAFT documents keep their basename, DOCX and text imports
replace their source extension with `.draft`, and new documents use
`Untitled.draft`. A suggestion becomes a target only after native confirmation.

Save Back to Source uses the separate typed external-source command. Rust
inspects current fidelity and source identity before React presents a warning.
Only a confirmed exact or accepted-normalized replacement can write; ordinary
Save and converted Save As copies retain their existing meanings.

## Trade-offs

The WebView remains responsible for transient availability because it knows the
active editor and pending operation state. Rust remains responsible for the
native menu objects and all trusted work. A failed availability update can
temporarily leave a native item stale, so the dispatcher checks availability
again before invoking any action and shows guidance to use the toolbar.

The first Phase 48 increment covers the File menu. Other accepted desktop menu
groups must be added only when their shipped actions can use the same
dispatcher and state rules.

The visible command bar stays intentionally compact. New remains a short
icon-and-text action, Open, Save, and Close use familiar icons with accessible
names, and Save As and Save Back to Source when applicable,
References, and Text checks live in one labeled overflow menu. Document,
connectivity, operation, and recovery state appear in the bottom status bar
instead of competing with the document name in the header.

The native window title and in-window header use the same path-free identity.
A clean saved document shows `name.draft — DRAFT`; a modified or imported
document includes `— Unsaved —`; and a new document shows
`Untitled document — Unsaved — DRAFT`. `set_window_title` accepts only a
validated basename and transient Unsaved boolean. It performs no file or
registry work.

About DRAFT shows the product version, exact package's short Git commit, and
closed build profile. The bottom-right status item shows the compact version
and short commit. The document inspector contains document metrics only.

## Technical Contract

The File menu order is:

1. New Document - Command-N
2. Open… - Command-O
3. Close - Command-W
4. separator
5. Save - Command-S
6. Save As… - Shift-Command-S
7. Save Back to Source - no shortcut

`src-tauri/src/desktop_menu.rs` owns menu construction, stable identifiers,
initial enablement, shortcuts, and typed event emission.
`set_native_menu_state` accepts exactly six booleans. It cannot receive a path,
document ID, source text, or arbitrary menu identifier.

`useWorkspaceActions` is the only frontend policy layer that maps File menu and
toolbar actions to document-session operations. It rejects disabled
and stale actions before they reach a typed command. `WorkspaceCommandBar`
contains no direct save, open, close, conversion, registry, or path authority.

Save and Save As are distinct requests. Save reuses the Rust-owned DRAFT target
when one exists. Save As requests an explicit output format. DRAFT output
preserves the old file and becomes authoritative only after successful atomic
replacement. DOCX and plain-text output remain non-authoritative copies and do
not clear unsaved work. Cancellation and failure leave the current target and
visible identity unchanged. PDF is absent under ADR-001.

Save Back is distinct from both. It is available only for a modified external
DOCX whose typed source state may be writable. It first runs non-mutating
eligibility inspection, then requires confirmation. Success preserves the
external source identity and display name. Cancellation, stale fingerprints,
and failure preserve the editor and visible identity.

The canonical icon source is `assets/DRAFT_Logo.png`. Generated Tauri assets
live under `src-tauri/icons/`; the in-window mark uses the generated 32-pixel
asset. The package must embed `icon.icns` and declare it through
`CFBundleIconFile`.

The same package declares `.draft` as the owned `DRAFT Document` type with the
exported UTI `com.progentic.draft.document`. macOS document activation enters a
Rust-owned queue and then the normal typed Open lifecycle. Native activation
never sends a path to the shared frontend dispatcher.

## Implementation Notes

Native menu events use `draft://native-menu-action`. The typed frontend wrapper
accepts only the seven File actions. `NativeMenuItems` starts with New and Open
enabled and every document-dependent action disabled until the frontend sends
current state.

The shared dispatcher also groups the existing References and Text checks
actions separately from document lifecycle and export actions. Those panel
actions are available from the overflow menu and are not added to the native
File menu. The overflow menu supports disabled-state skipping, Arrow, Home,
End, and Escape behavior, while every icon-only action retains an accessible
name and tooltip.

## Failure Modes

- An invalid native event or listener setup failure is contained and the
  toolbar remains available.
- A menu-state command failure shows bounded recovery guidance without raw
  platform or path details.
- A disabled or stale action does nothing when received.
- Save As cancellation keeps the current filename, dirty state, and Rust target.
- A failed window-title update leaves document and path authority unchanged and
  reports bounded recovery guidance.
- A failed Save As write preserves both the prior file and registry authority.
- Save Back cancellation and denied eligibility preserve the current editor and
  source identity.
- A missing or externally changed source requires reopening before Save Back.
- Overflow and native Save Back events use the same dispatcher, eligibility
  state, stale-source reason, and busy rejection. A stale event cannot bypass a
  disabled action after the menu state changes.
- A stale Finder or Dock cache can display an old icon even when the package is
  correct; inspect the bundle resource before clearing macOS caches.

## Tests

Rust tests pin menu order, About identity, labels, shortcuts, identifiers,
initial state, typed title and save requests, bounded filename suggestions,
Save As source preservation, target rebinding, and cancellation. Frontend
tests validate event payloads, title transitions, state responses, shared
dispatch, stale-action rejection, busy-state behavior, visible label parity,
overflow keyboard behavior, status placement, and the distinct Save, Save As,
and Save Back requests. Save As tests separately cover authoritative DRAFT and
non-authoritative DOCX/plain-text outcomes.

`scripts/check-invariants.sh` enforces the menu files, action set, shared
dispatcher, path-free frontend boundary, and absence of direct toolbar document
authority. `scripts/check-packaging.sh` pins the canonical source hash, stable
desktop derivatives, explicit bundle paths, and macOS icon container.

The unsigned package command verifies `Info.plist`, the embedded icon, and the
Apple Silicon executable. Direct review of exact artifact `75373ffb` passed the
Finder, Dock, application-switcher, native-menu, window-branding, dispatcher,
Save As, disabled-state, narrow-window, and keyboard checklist. This validates
the bounded Phase 48 implementation, not a final UI design. `RC-08`, `GATE-48`,
and paragraph controls plus their packaged evidence remain open under the
accepted ADR-004 contract.

## Related Documents

- `docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`
- `docs/adr/003-expand-v1-document-interoperability.md`
- `docs/maintainers/DOCUMENT_SAVE_LOAD.md`
- `docs/maintainers/PACKAGING.md`
- `docs/maintainers/WORKSPACE_UI.md`
- `docs/maintainers/RELEASE_CANDIDATE.md`
