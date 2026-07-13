# Current Limitations

DRAFT is pre-release software. The current workspace supports local document
editing, explicit save/open/close, manual references and citation insertion,
five local text checks, formatting review, and DOCX export on the initial macOS
Apple Silicon target.

## Documents

- There is no autosave, crash recovery, version history, or cloud sync.
- DRAFT opens and saves its version 1 document format. It can import UTF-8
  `.txt` and `.md` files as literal editable text, but it does not parse or
  preview Markdown. Unsupported or malformed input fails without changing the
  source file.
- Imported text and Markdown become unsaved DRAFT documents. They cannot be
  saved back to the source format; first Save requires a new `.draft` target
  and leaves the original source unchanged.
- DOCX, RTF, OpenDocument (`.odt`), and legacy Word (`.doc`) import are
  unavailable. DOCX is currently an export format only.
- The visible workspace manages one current document at a time.

## Desktop Interface

- Document lifecycle and DOCX export are available through matching native File
  menu and in-window controls. Other native menu groups remain limited to
  actions the current workspace can honor.
- Command grouping, responsive overflow, editor spacing, and outline layout
  remain under release-blocking desktop workflow review.
- The purple source artwork and generated bundle/header icons are integrated,
  but Finder, Dock, application-switcher, and packaged-window identity still
  require final packaged validation.

## Research And Citations

- The visible reference workflow accepts manual citekey, title, author, and
  year fields only.
- Reference editing, deletion, bulk import, synchronization, and visible
  bibliography management are unavailable.
- External metadata services and PDF intake are not exposed in the workspace.
- Citation rendering resolves saved citekeys, but there is no automatic
  citation repair.

## Text Checks

- Text checks are limited to repeated adjacent words, the 30-word sentence
  threshold, all-capital words of at least five letters, repeated substantial
  sentence openings, and mixed first-person perspective.
- Findings are fixed heuristic signals and may not fit the author's intent.
- A text snapshot is limited to 32 KiB and results to 100 findings.
- DRAFT does not generate writing, interpret arguments, evaluate ideas, check
  facts, or infer authorship.

## Formatting

- Formatting review checks heading structure and citation-style declarations
  for the supported style choices. It does not certify full compliance with a
  style manual.
- Formatting and text findings are not saved and do not apply automatic edits.
- Font formatting is limited to Arial, Avenir Next, Baskerville, Courier New,
  Georgia, Helvetica, Menlo, Palatino, Times New Roman, Trebuchet MS, and
  Verdana at whole point sizes from 8 through 72.
- Paragraph alignment, spacing, and indentation controls are not currently
  available. The underlying file and DOCX-export model does not make those
  controls a finished user workflow.

## Export

- DOCX export supports paragraphs, headings, text, hard breaks, bold, italic,
  underline, bounded font-family and font-size marks, and validated paragraph
  properties within documented resource limits.
- Unsupported content fails instead of being silently omitted.
- Citation nodes are not currently included in DOCX output.
- PDF export is currently unavailable. Its rendering policy and implementation
  boundary require separate accepted work.

## Connectivity And Services

- Offline mode is session-only. It does not detect operating-system
  connectivity, persist after restart, queue work, or retry requests.
- DRAFT does not expose provider credentials, model services, telemetry,
  diagnostics export, or support-bundle upload.

## Packaging

- The current reproducible package is an unsigned macOS Apple Silicon `.app`.
- Signing, notarization, installer creation, updates, and a published download
  remain release work.

See [Using the workspace](Workspace) for current actions and
[Troubleshooting](Troubleshooting) for recovery guidance.

Return to [Home](Home).
