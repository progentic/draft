# Troubleshooting

## Connecting To Core

`Connecting to core` is a temporary startup state. Wait briefly for the desktop
application to finish loading.

## Core Unavailable

`Core unavailable` means the workspace could not reach the desktop core. A
browser preview always shows this state because it does not include the desktop
runtime.

Close DRAFT and open the desktop application again. If the message repeats,
record the DRAFT version and operating-system version when reporting the issue.

## Core Status Invalid

`Core status invalid` means DRAFT rejected an unexpected status response or
update. Restart DRAFT. If the message returns, report the DRAFT version and the
exact visible message.

## Unsupported Application Version

`DRAFT received an unsupported application version.` means the interface and
desktop core do not agree on the application version.

Install one complete DRAFT build rather than combining files from different
builds. Restart the application after installation. If the message remains,
report the build source and version.

## Core Status Event Could Not Be Delivered

`DRAFT could not deliver the core status event.` means startup validation ran,
but the status update did not reach the workspace.

Restart DRAFT. If the failure repeats, report the DRAFT version and exact
message. Do not treat the workspace as connected while this message is shown.

## Core Status Could Not Be Read

`DRAFT could not read the core status.` is the fallback for an unknown desktop
status failure.

Restart DRAFT and report the DRAFT version and exact message if it repeats.

## Document Could Not Be Saved Or Opened

If save is cancelled, the document remains open with unsaved changes. Choose
**Save** again and select a writable location. If DRAFT cannot save, it keeps
the current open document and does not promote a partial replacement.
`Choose a .draft file name.` means the selected first-save name used another
extension; choose **Save** again and use `.draft`.

If a document cannot be opened, choose a valid DRAFT document or a UTF-8
`.txt` or `.md` file no larger than 8 MiB. Other extensions, invalid UTF-8, and
malformed DRAFT versions fail without changing that file. Imported text is
unsaved; choose **Save** to select a new `.draft` destination. DRAFT never
overwrites the imported source. DRAFT has no autosave or crash recovery, so
discarded unsaved changes cannot be restored.

If DRAFT cannot close a saved document, keep it open and retry **Close** before
switching documents.

## Formatting Check Needs To Run Again

`The document changed. Run the formatting check again.` means the checked
snapshot or its target is no longer current. Run the check again before using
Inspect or a heading-level action.

If DRAFT says the document has too many headings or citations, split the work
into a smaller document before checking it. For another heading or citation
validation message, correct that item when possible and rerun the check.

`DRAFT received an invalid formatting response. Check again.` means the core
returned a result the workspace could not use. Choose **Check again**.

`Formatting review could not reach the DRAFT core. Restart DRAFT, then check
again.` means the operation did not reach the desktop core. Restart DRAFT, then
choose **Check again**. Report the version and exact message if it repeats.

## Connectivity Mode Unavailable

`Mode unavailable` means the workspace could not read the Rust-owned session
mode. The alert distinguishes a command failure, an invalid response, and an
unreachable core. Choose the same control again to retry. Restart DRAFT and
report the version and exact visible message if it repeats.

`Online - change failed` or `Offline - change failed` means DRAFT kept the
visible prior mode because the requested change failed. Retry the change or
continue local work in the displayed mode.

Offline mode is a DRAFT session policy, not a network-status indicator. It
resets to online when DRAFT restarts.

## Citation Cannot Be Resolved

A citation can show invalid, unavailable, or failed copy inside the document.
Keep invalid citation input unchanged. The current workspace does not repair a
citation automatically.

For a read or transport failure, restart DRAFT only when the visible message
directs it. Confirm that the citekey exists in **References**.

## Reference Could Not Be Added

`That citekey is already in use.` means a saved reference has the same
case-sensitive citekey. Choose a different citekey.

`The reference details are not valid.` means at least one field failed the
bounded manual-reference rules. Enter a nonblank citekey, title, and author plus
a four-digit year, then try again.

For a library update failure, keep the form values visible and retry. Restart
DRAFT if the failure repeats.

## Text Checks Did Not Finish

`The document changed. Run text checks again.` means a pending or completed
result no longer addresses the current text. Choose **Check document** again.

If the document is too large, reduce it below the current 32 KiB text limit and
retry. If text checks are unavailable in this installation, restart DRAFT. The
workflow has no provider sign-in or credential recovery because the five checks
run locally.

An invalid response or unfinished check leaves the document unchanged. Use the
same **Check document** control to retry.

## DOCX Export Failed

If the file location is invalid, choose **Export DOCX** again and select a
writable `.docx` destination.

Unsupported document content and resource-limit failures require editing the
document before retrying. Citation nodes are not currently included in DOCX
output; remove them before export when that message appears. Export failure
does not change the DRAFT source document.

Return to [Home](Home).
