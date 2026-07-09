# DRAFT Workspace

## Current workspace

DRAFT currently opens a local writing workspace with one editable document.
The workspace contains a document outline, a formatting toolbar, the writing
surface, and live document statistics.

This is a pre-release workspace. The header and document panel show `Not saved`
and `Unsaved` because file creation, save, open, and recovery are not available
at this checkpoint.

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
application discards edits. DRAFT does not yet create, open, save, or reopen
document files.

The durable document model, Rust-owned document registry, save/load path, and
atomic save protection are implemented in later document-core phases. The UI
does not present inactive file commands before those protections exist.

## Local behavior

The current workspace does not call external services, read local files, or
write application data. Editor state remains transient inside the application
WebView.
