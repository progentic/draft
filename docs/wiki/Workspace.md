# Using The Workspace

## Create And Save A Document

Use the compact document controls at the top of the workspace for common
actions. Additional commands, including Save As, Save Back to Source when
available, export, References, and Text checks, are available from the **More**
(`…`) menu. The same document actions are also available from the macOS
**File** menu.

DRAFT shows the document name near the top of the workspace. Save state,
import state, connectivity, active operations, and concise feedback appear in
the status bar at the bottom.

When New, Open, or Close would replace or discard unsaved work, choose **Save
and continue**, **Discard changes**, or **Keep editing**. DRAFT does not
autosave and cannot recover unsaved work after a crash, so save important
changes explicitly.

**New Document** opens a blank page with the cursor ready. **Open…** loads a DRAFT
document, imports a UTF-8 `.txt` or `.md` file as editable text, or reads the
supported paragraph subset from a `.docx` file. Text and Markdown imports show
their source filename and `Imported, unsaved`; the filename does not become a
save location. Their first Save asks for a new `.draft` destination. After Save
succeeds, the header shows the selected `.draft` filename and later saves reuse
that target. Markdown syntax is kept as literal text, not parsed or previewed,
and the original `.txt` or `.md` source is never overwritten.

An opened DOCX remains associated with its Rust-owned source identity. Ordinary
**Save** creates a `.draft` document, and **Export DOCX…** creates a separate
copy. **Save Back to Source** appears only for a DOCX source. It checks whether
the current content and source can be replaced safely, shows an overwrite or
normalization warning, lists each known normalization, and requires Replace or
Cancel confirmation. Unsupported, lossy, missing, or externally changed
sources remain unavailable.

**Save As…** chooses a new `.draft` target while preserving the previous file.
After it succeeds, later Save operations use the new target. Cancellation or a
failed write leaves the current filename and file unchanged.

## Write And Format

The editor supports undo and redo, bold, italic, strikethrough, first-, second-,
and third-level headings, bulleted and numbered lists, and block quotes.
Formatting applies to selected text. With no selection, it applies at the
current cursor position or to text entered next, depending on the command.

Use **Font family** to choose Arial, Avenir Next, Baskerville, Courier New,
Georgia, Helvetica, Menlo, Palatino, Times New Roman, Trebuchet MS, or Verdana.
Use **Font size in points** to choose a whole size from 8 through 72. The
controls show the effective formatting at the caret, or a mixed state when a
selection contains different values. Choose the document-default option in the
font or size control to remove an explicit override. Family and size are saved
with the DRAFT document and included in DOCX export.

Press Tab to enter the formatting toolbar. Use Left Arrow and Right Arrow to
move between enabled controls. Home moves to the first and End to the last.

## Navigate The Document

The Outline panel lists headings. Choose one to move the editor cursor to that
heading. The Document panel shows live word, character, and heading counts.

## Review Formatting

Open **Formatting review**, select APA 7, MLA 9, or Chicago 17 author-date, then
choose **Check formatting**. Use **Inspect** to locate a finding. Dismiss hides
it for the current check. A heading-level action runs only when you choose it
and its target is still current.

These checks cover heading structure and citation-style declarations. They do
not certify complete style-manual compliance or repair text automatically.

## Add A Reference And Citation

Open **References** from the **More** (`…`) menu. Enter a unique citekey, title,
author, and four-digit year, then choose **Add reference**. Put the cursor in
the document and choose **Insert citation** beside the saved source.

This workflow supports manual references. Metadata search, reference editing or
deletion, library import, synchronization, and a visible bibliography remain
unavailable.

## Run Local Text Checks

Open **Text checks** from the **More** (`…`) menu, then choose **Check
document**. DRAFT checks for repeated adjacent words, sentences longer than 30
words, extended all-capital emphasis, repeated consecutive sentence openings,
and mixed singular/plural first-person perspective.

Findings are suggestions for review, not conclusions. Each one explains its
fixed pattern and shows the flagged passage. Choose **Show in document** to
select that passage. DRAFT never applies an edit from a text finding.

The checks run locally without a provider, credentials, or network
transmission. If the document changes during a run, check it again.

## Export DOCX

Choose **Export DOCX…** and select a destination. Wait for the completion
message. Export does not change the DRAFT source.

The supported subset includes the eleven named font families and whole point
sizes from 8 through 72. DRAFT does not silently discard unsupported content.
Export either preserves supported content or reports a clear failure or
limitation. Citation nodes are not currently included in DOCX output. Documents
containing citation nodes must be reviewed before export, and DRAFT may reject
the export rather than omit them silently. PDF export remains unavailable
pending its separate review and implementation path.

## Work Offline

Use the connectivity control in the bottom status bar to switch the current
session between online and offline modes. New metadata requests and research
links are blocked before external work begins. Local editing, review,
references, saving, and export remain available.

The setting resets to online when DRAFT restarts and is not an operating-system
network indicator.

## File Menu Shortcuts

- Command-N: New Document
- Command-O: Open
- Command-W: Close
- Command-S: Save
- Shift-Command-S: Save As
- Save Back to Source: no shortcut
- Shift-Command-E: Export DOCX

Actions that cannot run in the current document state are disabled. While a
document or export operation is pending, competing actions remain unavailable.

Press Tab to move through the document controls, formatting controls, editor,
panels, and bottom status bar. Icon-only controls expose accessible names and
tooltips. Open the **More** menu with Enter, Space, or Down Arrow. Within that
menu, use Up Arrow and Down Arrow to move, Home or End to reach the first or
last enabled action, and Escape to return focus to **More**.

In the formatting toolbar, use Left Arrow and Right Arrow to move, Home for the
first enabled control, and End for the last. Disabled controls are skipped.
Panels and the bottom status bar announce pending, completed, empty, failed,
and recovery states.

See [Troubleshooting](Troubleshooting) for message-specific recovery and
[Current limitations](Current-Limitations) for the complete current boundary.

Return to [Home](Home).
