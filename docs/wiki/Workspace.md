# Using The Workspace

## Create And Save A Document

The document action bar contains **New**, **Open**, **Save**, **References**,
**Text checks**, **Export DOCX**, and **Close**. The header shows the document
name and whether it is not saved, has unsaved changes, is saving, or is saved.

When edited text would be replaced or closed, choose **Save and continue**,
**Discard changes**, or **Keep editing**. DRAFT does not autosave and cannot
recover unsaved work after a crash, so save important changes explicitly.

**New** opens a blank page with the cursor ready. **Open** loads a DRAFT
document or imports a UTF-8 `.txt` or `.md` file as editable text. Imported
content shows its source filename and `Imported, unsaved`; the filename does
not become a save location. The first Save asks for a new `.draft` destination,
and the original text or Markdown file remains unchanged. Markdown syntax is
kept as literal text.

## Write And Format

The formatting toolbar supports undo and redo, bold, italic, strikethrough,
first- and second-level headings, bulleted and numbered lists, and block quotes.
Formatting applies to the current selection.

Use **Font family** to choose Arial, Avenir Next, Baskerville, Courier New,
Georgia, Helvetica, Menlo, Palatino, Times New Roman, Trebuchet MS, or Verdana.
Use **Font size in points** to choose a whole size from 8 through 72. The
Default choices remove that font setting from the selected text. Family and
size are saved with the DRAFT document and included in DOCX export.

Press Tab to enter the formatting toolbar. Use Left Arrow and Right Arrow to
move between enabled controls. Home moves to the first and End to the last.

## Navigate The Document

The Outline panel lists headings. Choose one to move the editor cursor to that
heading. The Document panel shows live word, character, and heading counts.

## Review Formatting

Choose **Formatting review** in the formatting toolbar. Select APA 7, MLA 9, or
Chicago 17 author-date, then choose **Check formatting**. Use **Inspect** to
locate a finding. Dismiss hides it for the current check. A heading-level action
runs only when you choose it and its target is still current.

These checks cover heading structure and citation-style declarations. They do
not certify complete style-manual compliance or repair text automatically.

## Add A Reference And Citation

Choose **References**. Enter a unique citekey, title, author, and four-digit
year, then choose **Add reference**. Put the cursor in the document and choose
**Insert citation** beside the saved source.

This workflow supports manual references. Metadata search, reference editing or
deletion, library import, synchronization, and a visible bibliography remain
unavailable.

## Run Local Text Checks

Choose **Text checks**, then **Check document**. DRAFT checks for repeated
adjacent words, sentences longer than 30 words, extended all-capital emphasis,
repeated consecutive sentence openings, and mixed singular/plural first-person
perspective.

Findings are suggestions for review, not conclusions. Each one explains its
fixed pattern and shows the flagged passage. Choose **Show in document** to
select that passage. DRAFT never applies an edit from a text finding.

The checks run locally without a provider, credentials, or network
transmission. If the document changes during a run, check it again.

## Export DOCX

Choose **Export DOCX** and select a destination. Wait for the completion
message. Export does not change the DRAFT source.

The supported subset includes the eleven named font families and whole point
sizes from 8 through 72. Other unsupported content fails rather than
disappearing. Citation nodes are not currently included in DOCX output. PDF
export remains unavailable pending its separate review and implementation path.

## Work Offline

Choose **Online** in the header to work offline for the current session. New
metadata requests and research links are blocked before external work begins.
Local editing, review, references, saving, and export remain available.

The setting resets to online when DRAFT restarts and is not an operating-system
network indicator.

See [Troubleshooting](Troubleshooting) for message-specific recovery and
[Current limitations](Current-Limitations) for the complete current boundary.

Return to [Home](Home).
