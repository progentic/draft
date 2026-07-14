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

## Application Build Could Not Be Verified

`DRAFT could not verify this application build.` means the desktop runtime did
not contain valid build identity. Replace it with one complete DRAFT package.
If the message repeats, report the visible version, build commit, and profile.
Do not use that package as manual validation evidence.

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

If a document cannot be opened, choose a valid DRAFT document, a UTF-8 `.txt`
or `.md` file no larger than 8 MiB, or a `.docx` file within the documented
package limits. Other extensions, invalid UTF-8, malformed DRAFT versions, and
invalid DOCX packages fail without changing that file. Imported content is
not a native DRAFT document; choose **Save** to select a new `.draft`
destination. DRAFT has no autosave or crash recovery, so discarded unsaved
changes cannot be restored.

Open always reports its pending and final disposition in the temporary notice
below the document controls. If selecting a DOCX returns to the editor without
a success, cancellation, limitation, or failure message, record the visible
build commit and do not treat the operation as successful.

If a DOCX message says the file is malformed or unsafe, keep the original file
unchanged and open a trusted, valid copy. If DRAFT reports an unsupported or
lossy feature, preserve the original and edit through a supported format rather
than assuming DRAFT can round-trip it. A source-preservation notice means DRAFT
opened the supported content but cannot safely save changes back to that DOCX.
Save to `.draft` or export a separate DOCX copy instead.

**Save Back to Source** always shows a confirmation before replacing a DOCX.
Choose **Keep source** to cancel without accepting saved state. If DRAFT says
the source changed or is missing, reopen the source before trying again. If it
says replacement is unavailable, use **Save As…** for a `.draft` document or
export a new DOCX. If DRAFT cannot confirm the source's final state, reopen it
before continuing; no message exposes a path, fingerprint, or document XML.

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

Export always reports pending and final state in the temporary notice below the
document controls. A completed export names success; cancellation and every
typed failure show a separate disposition. If no disposition appears, record
the visible build commit and preserve the DRAFT source.

## A DRAFT File Opens In Another Application

`.draft` is a structured DRAFT source file, so seeing JSON in a text editor does
not mean the document serialization is damaged. Install one complete DRAFT
application bundle, then open the file with DRAFT. The package declares DRAFT
as the owner of `com.progentic.draft.document`; macOS may need to refresh its
application registration after an older build is replaced.

Return to [Home](Home).
## Native Menu Is Unavailable

If DRAFT reports that it could not read or update the native menu, use the
matching action in the visible document toolbar. Finish or cancel any pending
save, open, close, or export operation before trying again. Restart DRAFT if
the menu remains out of sync with the toolbar.
