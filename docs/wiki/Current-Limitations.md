# Current Limitations

DRAFT is pre-release software. The current workspace supports local document
editing, explicit save/open/close, manual references and citation insertion,
five local text checks, formatting review, and DOCX export on the initial macOS
Apple Silicon target.

## Documents

- There is no autosave, crash recovery, version history, or cloud sync.
- DRAFT opens and saves its version 2 document format. It migrates valid version
  1 DRAFT documents in memory and writes version 2 only after an explicit save.
- `.draft` is structured JSON, not plain prose. The macOS bundle declares DRAFT
  as the owner of that document type and supplies its icon, but double-click and
  Launch Services behavior still require replacement-package validation.
- DRAFT imports UTF-8 `.txt` and `.md` files as literal editable text, but it
  does not parse or preview Markdown. Unsupported or malformed input fails
  without changing the source file.
- DRAFT can read a bounded DOCX paragraph subset, including supported headings,
  alignment, line spacing, paragraph spacing, and indentation. Valid DOCX
  features outside that subset are disclosed as requiring source preservation;
  malformed, unsafe, unsupported, or lossy input fails without changing the
  source file.
- Inline Word tabs import as readable spacing, but exact tab-stop placement is
  retained only in the unchanged source. Common proofing metadata, custom style
  names, layout markers, and hyperlink wrappers retain visible text without
  claiming full behavior. DOCX tables and footnote references are not imported
  because DRAFT does not yet have editable models that can preserve their
  structure safely. They are not flattened or silently removed.
- Imported text and Markdown become unsaved DRAFT documents and cannot be saved
  back to the source format. First Save requires a new `.draft` target.
- A supported DOCX source can be replaced only when DRAFT reports an exact or
  accepted-normalized disposition and the user confirms the warning. Lossy,
  uncertain, unsupported, missing, or externally changed sources are not
  replaced. Save creates a `.draft` document, while Export creates a separate
  DOCX copy.
- RTF, OpenDocument (`.odt`), and legacy Word (`.doc`) import are unavailable.
  Complete same-format DOCX round-trip fidelity is not currently supported.
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
- Spelling highlights, suggestions, ignore rules, and correction controls are
  not currently available as a DRAFT-owned workflow.
- Only explicit page-break nodes create separate page surfaces. DRAFT does not
  automatically paginate from content flow, fonts, margins, or printer geometry.

## Export

- DOCX export supports paragraphs, headings, text, hard and page breaks, bold, italic,
  underline, bounded font-family and font-size marks, and validated paragraph
  properties within documented resource limits.
- Unsupported content fails instead of being silently omitted.
- Citation nodes are not currently included in DOCX output.
- Save As offers DRAFT, Word, and plain-text output. Only DRAFT output becomes
  the active authoritative document; Word and text are converted copies that
  keep the current identity and unsaved state. Packaged validation of this
  selector remains open.
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
