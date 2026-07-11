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

## Formatting review

Choose **Formatting review** in the toolbar to open the review band. Select APA
7, MLA 9, or Chicago 17 author-date, then choose **Check formatting**. DRAFT
groups current findings under Structure and Citations.

Use **Inspect** to move to the affected current heading or citation. Some
heading findings offer an explicit level change. DRAFT applies that change only
after you choose it and only if the document has not changed since the check.
**Dismiss** hides a finding for the current result only.

These checks cover heading structure and citation-style declarations. They do
not certify complete compliance with a style manual, reformat citations, save
findings, or change the document automatically.

After a failed check, the existing button reads **Check again**. Validation
messages identify input that must change. Invalid responses and transport
failures tell you to retry the same check or restart DRAFT.

## Citation messages

An existing citation can report that its data is invalid, unavailable, or
could not be rendered. DRAFT keeps the document unchanged when no recovery is
available. Citation insertion, management, and repair controls are not yet
available in the workspace.

## Outline and document details

The outline lists headings from the current editor content. Selecting an
outline entry moves the editor cursor to that heading. The outline can be
closed on larger screens to leave more room for writing.

The document panel reports the current word, character, and heading counts.
These values are session information only.

The session area also reports the desktop core connection. `Core v<version>`
means the workspace reached the trusted Rust runtime and validated its status
event. `Core unavailable` means the desktop transport was not available.
`DRAFT could not deliver the core status event.` means Rust could not deliver
the status update. An unsupported application version means the workspace and
desktop core do not share the expected contract. `Core status invalid` means
the app rejected an unexpected response or event payload. Any unknown status
failure uses a bounded fallback without exposing internal details. A standalone
browser preview has no Tauri core and shows the unavailable state; the desktop
application provides the command and event connection.

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

Use the **Online** control in the header to work offline for the current
session. When offline, DRAFT blocks new metadata requests and research links
before external work begins. Local editing and formatting review remain
available. Choose **Go online** to allow those external actions again.

The mode resets to online when DRAFT restarts. It does not report whether the
operating system has a connection and does not retry or queue requests. When a
mode change fails, the prior confirmed mode remains visible and the same
control can retry the change.

## Exporting

The current workspace has no export controls. DRAFT has an internal DOCX
foundation, but users cannot start that export from the workspace yet. PDF
export is not currently available. DRAFT has deferred that work until its
rendering policy and implementation boundary are defined and verified.

DRAFT must define reliable rules for fonts, page layout, accessibility, and
consistent output across supported platforms before PDF work can begin. No PDF
converter or hidden printing process runs in the application.
