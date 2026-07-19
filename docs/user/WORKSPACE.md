# DRAFT Workspace

## Create And Save Documents

Use the compact document controls at the top of the workspace for common
actions. Additional commands, including Save As, Save Back to Source when
available, References, and Text checks, are available from the **More**
(`…`) menu. The same document actions are also available from the macOS
**File** menu.

DRAFT shows the current document name and `Unsaved` state near the top of the
workspace. Detailed document state, connectivity, active operations, and a
compact build identity appear in the status bar at the bottom. A temporary
notice directly below the document controls reports the
pending, completed, cancelled, or failed result of Open, Save, and Save As.

When New, Open, or Close would replace or discard unsaved work, DRAFT asks
whether to save, discard the changes, or keep editing. There is no autosave or
crash recovery, so save important work explicitly.

**New Document** opens a blank page with the cursor ready. **Open…** loads a DRAFT
document, imports a UTF-8 `.txt` or `.md` file as editable text, or reads the
supported paragraph subset from a `.docx` file. Text and Markdown imports are
shown as imported and unsaved; their filename is for orientation only. Their
first Save asks for a new `.draft` destination. Markdown punctuation remains
literal text rather than a preview, and DRAFT states this when the import
opens. The imported `.txt` or `.md` source is never overwritten.

A `.draft` file is DRAFT's structured editable source, not a plain-text
document. It stores document identity and formatting alongside the writing.
The macOS package associates this file type with DRAFT so a double-click uses
the same safe Open workflow. The interface never displays the file's full
path.

An opened DOCX remains associated with its Rust-owned source identity. Ordinary
**Save** creates a `.draft` document. **Save As…** can create a new DRAFT
document, a separate Word copy, or a separate plain-text copy. **Save Back to Source**
is available only for modified DOCX content that
DRAFT can replace safely. DRAFT checks the current source before showing an
overwrite warning. Exact replacement and accepted normalization both require
confirmation. A normalized replacement lists the exact supported change before
you choose **Replace** or **Cancel**. Unsupported, lossy, missing, or externally
changed sources stay unavailable and are not overwritten.

DOCX import retains supported explicit font family, whole-point size, bold,
italic, underline, paragraph alignment, spacing, indentation, heading styles,
and page breaks. Source behavior outside that subset stays in the unchanged
original and is disclosed when the imported copy opens. Inline Word tabs use
readable spacing but do not retain exact tab-stop placement. Common Word
proofing metadata, custom style names, layout markers, and hyperlink wrappers
retain their visible text, but unsupported behavior remains only in the
unchanged source. A DOCX that depends on tables or footnotes cannot currently
open because DRAFT cannot preserve those structures safely.

An explicit page break appears as a gap between separate page surfaces. DRAFT
does not automatically calculate page boundaries from text flow, margins,
fonts, or printer settings, so content without an explicit break remains on
one continuous surface.

Use **Save As…** and choose **DRAFT document**, **Word document**, or **Plain
text**. A DRAFT result becomes the active document only after the atomic write
succeeds; later Save operations use it. Word and plain-text results are copies:
they do not change the active filename or clear unsaved edits. Cancelling or
failing Save As leaves the current document unchanged. Rust suggests the
current basename with the selected extension, an imported source basename, or
a bounded `Untitled` name.

## Write And Format

The editor supports undo and redo, bold, italic, strikethrough, first-, second-,
and third-level headings, bulleted and numbered lists, and block quotes.
Formatting applies to selected text. With no selection, it applies at the
current cursor position or to text entered next, depending on the command.

Choose Arial, Avenir Next, Baskerville, Courier New, Georgia, Helvetica, Menlo,
Palatino, Times New Roman, Trebuchet MS, or Verdana. Choose a whole point size
from 8 through 72. The controls show effective values at the caret and a mixed
state for selections with different formatting. Choose **Reset to document font**
or **Reset to document size** in the relevant control to remove an explicit
override. These choices are saved with the document and included in DOCX
export.

The outline lists document headings and moves the cursor to the selected
heading. The Document panel shows live word, character, and heading counts.

## Review Formatting

Open **Formatting review**, select APA 7, MLA 9, or Chicago 17 author-date, then
choose **Check formatting**. Use **Inspect** to locate a finding. Dismiss hides
it for the current result. A permitted heading change runs only after you
choose it and only while the checked target remains current.

Formatting review checks heading structure and citation-style declarations. It
does not certify complete style-manual compliance or repair a document
automatically.

## Add A Reference And Citation

Open **References** from the **More** (`…`) menu. Enter a unique citekey, title,
author, and four-digit year, then choose **Add reference**. Saved references
appear in the same panel. Place the editor cursor where the citation belongs
and choose **Insert citation** for that source.

The current workflow adds manual references only. It does not search metadata
services, import a library, edit or delete stored references, or build a visible
bibliography.

## Run Text Checks

Open **Text checks** from the **More** (`…`) menu, then choose **Check
document**. DRAFT runs five local fixed checks:

- repeated adjacent words;
- sentences longer than 30 words;
- all-capital words with at least five letters;
- consecutive sentences beginning with the same substantial word; and
- mixed singular and plural first-person perspective.

These findings are suggestions for review, not conclusions. Each finding names
the pattern, explains it, shows the flagged passage, and offers **Show in
document**. DRAFT does not change the passage. If the document changes while a
check runs, run the check again.

Text checks run locally and do not use a provider, credentials, or network
transmission. They do not generate text, evaluate ideas, or determine author
intent.

## Save As And Converted Copies

Choose **Save As…**, select **Word document**, and choose a destination. Wait
for the completion message. The Word copy does not change the DRAFT source or
the active document identity. Choose **Plain text** for deterministic UTF-8
text with visible paragraphs, list markers, block quotes, page breaks, and
citation keys. Formatting is intentionally absent from the text copy.

Word Save As supports the documented basic writing subset, including the eleven
font families and whole point sizes from 8 through 72. DRAFT does not silently
discard unsupported content. Conversion either preserves supported content or
reports a clear failure or limitation. Citation nodes are not currently
included in DOCX output. Documents containing citation nodes must be reviewed
before conversion, and DRAFT may reject the copy rather than omit them silently.
PDF export remains unavailable pending its separate rendering policy and
implementation work.

## Work Offline

Use the connectivity control in the bottom status bar to switch the current
session between online and offline modes. DRAFT blocks new metadata requests
and research links before external work begins. Editing, formatting review,
manual references, local text checks, saving, and converted copies remain local.

The setting resets to online when DRAFT restarts. It does not indicate whether
the operating system has a connection and does not retry or queue requests.

## Keyboard And Status

The File menu uses Command-N for New Document, Command-O for Open, Command-W
for Close, Command-S for Save, and Shift-Command-S for Save As. Save Back to
Source has no shortcut. Unavailable actions are
disabled while another document or save operation is pending.

Press Tab to move through the document controls, formatting controls, editor,
panels, and bottom status bar. Icon-only controls expose accessible names and
tooltips. Open the **More** menu with Enter, Space, or Down Arrow. Within that
menu, use Up Arrow and Down Arrow to move, Home or End to reach the first or
last enabled action, and Escape to return focus to **More**.

In the formatting toolbar, use Left Arrow and Right Arrow to move, Home for the
first enabled control, and End for the last. Disabled controls are skipped.

The bottom status bar reports document, connectivity, background-operation,
and compact `v<version> · <commit>` build state. About DRAFT shows the version,
short commit, and build profile. The short commit identifies the clean package
revision without changing the product version. A browser preview has no
desktop core and reports it as unavailable.
