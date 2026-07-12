# DRAFT Workspace

## Create And Save Documents

Use the document action bar to choose **New**, **Open**, **Save**, **Export
DOCX**, or **Close**. DRAFT shows the current document name and a visible state
such as Not saved, Unsaved changes, Saving, or Saved.

New, Open, and Close protect edited text. When changes have not been saved,
DRAFT asks whether to save and continue, discard the changes, or keep editing.
There is no autosave or crash recovery, so save important work explicitly.

## Write And Format

The editor supports undo and redo, bold, italic, strikethrough, first- and
second-level headings, bulleted and numbered lists, and block quotes. Formatting
applies to the current selection.

Choose a font family from Arial, Georgia, Times New Roman, or Courier New, or
choose **Default font** to remove the family from the selected text. Choose a
whole point size from 8 through 72, or **Default size** to remove the size.
These choices are saved with the document and included in DOCX export.

The outline lists document headings and moves the cursor to the selected
heading. The Document panel shows live word, character, and heading counts.

## Review Formatting

Choose **Formatting review** in the formatting toolbar. Select APA 7, MLA 9, or
Chicago 17 author-date, then choose **Check formatting**. Use **Inspect** to
locate a finding. Dismiss hides it for the current result. A permitted heading
change runs only after you choose it and only while the checked target remains
current.

Formatting review checks heading structure and citation-style declarations. It
does not certify complete style-manual compliance or repair a document
automatically.

## Add A Reference And Citation

Choose **References**. Enter a unique citekey, title, author, and four-digit
year, then choose **Add reference**. Saved references appear in the same panel.
Place the editor cursor where the citation belongs and choose **Insert
citation** for that source.

The current workflow adds manual references only. It does not search metadata
services, import a library, edit or delete stored references, or build a visible
bibliography.

## Run Text Checks

Choose **Text checks**, then **Check document**. DRAFT runs five local fixed
checks:

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

## Export DOCX

Choose **Export DOCX**, select a destination in the system dialog, and wait for
the completion message. Export does not change the DRAFT source document.

DOCX export supports the documented basic writing subset, including the four
font families and whole point sizes from 8 through 72. Unsupported content
fails instead of disappearing. Citation nodes are not currently included in
DOCX output; remove them before exporting when that limitation applies. PDF
export remains unavailable pending its separate rendering policy and
implementation work.

## Work Offline

Choose **Online** in the header to work offline for the current session. DRAFT
blocks new metadata requests and research links before external work begins.
Editing, formatting review, manual references, local text checks, saving, and
DOCX export remain local.

The setting resets to online when DRAFT restarts. It does not indicate whether
the operating system has a connection and does not retry or queue requests.

## Keyboard And Status

Press Tab to reach the document actions and formatting toolbar. In the
formatting toolbar, use Left Arrow and Right Arrow to move, Home for the first
enabled control, and End for the last. Disabled controls are skipped.

Panels and operations announce pending, completed, empty, and failed states.
`Core v<version>` means the desktop interface reached the Rust runtime. A
browser preview has no desktop core and reports it as unavailable.
