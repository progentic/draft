# Current Limitations

DRAFT is pre-release software. The current workspace supports local document
editing, explicit save/open/close, manual references and citation insertion,
five local text checks, formatting review, and DOCX export on the initial macOS
Apple Silicon target.

## Documents

- There is no autosave, crash recovery, version history, or cloud sync.
- DRAFT opens and saves its version 1 document format only. Unsupported or
  malformed versions fail without changing the source file.
- The visible workspace manages one current document at a time.

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

## Export

- DOCX export supports paragraphs, headings, text, hard breaks, and bold,
  italic, or underline marks within documented resource limits.
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
