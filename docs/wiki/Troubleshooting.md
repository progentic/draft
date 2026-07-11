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

## Edits Disappeared

The current interface does not save documents. Reloading the workspace or
closing DRAFT discards edits. There is no recovery path for a discarded
pre-release editor session. Keep important work in another saved document.

## Formatting Check Needs To Run Again

`The document changed. Run the formatting check again.` means the checked
snapshot or its target is no longer current. Run the check again before using
Inspect or a heading-level action.

If DRAFT says the document has too many headings or citations, split the work
into a smaller document before checking it. For another heading or citation
validation message, correct that item when possible and rerun the check.

`DRAFT received an invalid formatting response.` or
`Formatting review could not reach the DRAFT core.` means the review did not
receive a usable result.
Run the check again. Restart DRAFT and report the version and exact message if
the failure repeats.

## Connectivity Mode Unavailable

`Mode unavailable` means the workspace could not read the Rust-owned session
mode. Choose the control again to retry. Restart DRAFT and report the version
and exact visible message if it repeats.

`Online - change failed` or `Offline - change failed` means DRAFT kept the
visible prior mode because the requested change failed. Retry the change or
continue local work in the displayed mode.

Offline mode is a DRAFT session policy, not a network-status indicator. It
resets to online when DRAFT restarts.

Return to [Home](Home).
