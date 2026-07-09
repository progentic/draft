# DRAFT Workspace

## Current workspace

DRAFT currently opens a local writing workspace with one editable document.
The workspace contains a document outline, a formatting toolbar, the writing
surface, and live document statistics.

This is a pre-release workspace. The header and document panel show `Not saved`
and `Unsaved` because the current UI does not expose the new Rust document file
commands yet. Autosave and recovery are not available at this checkpoint.

## Editing

The editor supports the current Tiptap formatting set:

- undo and redo
- bold, italic, and strikethrough text
- first- and second-level headings
- bulleted and numbered lists
- block quotes

Formatting controls operate on the active selection. Their selected state
changes with the current cursor position.

## Outline and document details

The outline lists headings from the current editor content. Selecting an
outline entry moves the editor cursor to that heading. The outline can be
closed on larger screens to leave more room for writing.

The document panel reports the current word, character, and heading counts.
These values are session information only.

The session area also reports the desktop core connection. `Core v<version>`
means the workspace reached the trusted Rust runtime and validated its status
event. `Core unavailable` means the desktop transport was not available. `Core
event failed` means Rust could not deliver the status update. `Core status
invalid` means the app rejected an unexpected response or event payload. A
standalone browser preview has no Tauri core and shows the unavailable state;
the desktop application provides the command and event connection.

## Saving and reopening

Do not use the current workspace for document storage. Reloading or closing the
application discards edits. DRAFT does not yet let users create, open, save, or
reopen document files from the workspace controls.

The Rust core now has a validated envelope, single-live-handle registry, native
file dialog commands, explicit snapshot save path, and hardened atomic
replacement path. The backend save path is tested against interruption and
concurrent saves, but these boundaries are not presented as a user workflow
until workspace file controls are integrated.

## Local behavior

The visible workspace does not call external services, read local files, or
write application data. Editor state remains transient inside the application
WebView until file controls are integrated.
