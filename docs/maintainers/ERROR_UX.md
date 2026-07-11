# Error UX

## Status

This guide records implemented Phase 39 behavior. The requirements in
`docs/drafts/ERROR_UX.md` remain non-binding historical input.

## Scope

Phase 39 maps failures that already reach four visible surfaces:

- runtime status;
- connectivity mode;
- formatting review; and
- citation rendering.

It adds no command, navigation path, backend capability, diagnostic control,
or product workflow. Typed errors for document files, workers, external
access, metadata, secrets, diagnostics, imports, exports, and other unwired
boundaries remain inventory-only in `ERROR_MESSAGES.md`.

## Presentation Contract

`src/features/error-ux/errorPresentation.ts` owns stable frontend copy and one
closed recovery disposition for every visible typed failure:

| Disposition | Meaning |
| :--- | :--- |
| `retryable` | Repeating the current operation or restarting DRAFT can reasonably retry the same operation. |
| `actionable` | The user can change visible input or application state before trying again. |
| `terminal` | The current visible workflow offers no recovery action. State remains explicit and unchanged. |

Each presentation has a short title, a bounded message, and a disposition.
Labels are allowed only when an already-visible control can honor them. The
current retry labels refer to the connectivity and formatting controls; no
action label is emitted. A disposition does not create a new control.

## Visible Mappings

### Runtime status

The existing `invalid_application_version` and `event_delivery_failed` codes
retain distinct messages. Invalid payload/response, transport, and unknown
command fallbacks remain separate. No new runtime-status action is added.

### Connectivity

`connectivity_unavailable`, invalid-response, and transport failures now have
different read/change messages. Failed changes state which confirmed mode
remains effective. An unreadable mode reuses the existing retry button; a
failed change reuses the existing online/offline toggle.

### Formatting review

All six command codes retain explicit mappings. Input failures tell the user
what visible document input must change. Invalid response and transport
failures are retryable because the existing formatting check can be run again.
The existing command reads `Check again` after any prior run.

### Citation rendering

Every citation-node cause, missing-reference result, reference-read category,
invalid response, and transport failure maps explicitly. Messages distinguish
input problems from resolution failures without naming a citation-library
workflow or suggesting that reference management is visible. Invalid and
terminal states preserve the citation unchanged.

## Accessibility And State

Runtime and formatting state replace one existing message rather than adding
notices. Connectivity renders at most one atomic alert. Citation node views
update one atomic polite live region per node. Repeated renders do not append
messages or move focus.

The existing connectivity retry, formatting check, and finding-dismiss
controls remain native `button` elements with stable accessible names. Phase
39 adds no modal, toast stack, focus trap, or automatic focus movement.

## Redaction And Authority

Presentation functions receive only validated frontend error unions. Copy
contains no raw runtime text, source content, titles, citekeys, paths, URLs,
credentials, provider payloads, logs, stack traces, or native error details.
Unknown raw IPC failures are reduced to existing transport fallbacks before
presentation.

The frontend remains presentation-only. No mapper reads Rust state, accesses a
trusted API, persists a notice, or changes document/citation authority.

## Enforcement

Compile-time `Record` mappings and exhaustive switches fail when a visible
typed variant is added without a presentation. Focused tests cover every known
code, all three dispositions, outer fallbacks, retained connectivity state,
single announcements, focus retention, and button semantics.

`scripts/check-invariants.sh` requires those mappings/tests, rejects imports
from unwired error domains, and denies raw detail categories or privileged
frontend APIs in the Phase 39 module.

Run:

```bash
npm test -- src/features/error-ux/errorPresentation.test.ts \
  src/features/connectivity/ConnectivityModeControl.test.tsx \
  src/features/formatting-review/FormattingReviewPanel.test.tsx \
  src/editor/CitationNode.test.ts src/App.test.tsx
npm run typecheck
bash scripts/check-invariants.sh
```

## Ownership Layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | visible workspace surfaces | Present one current failure and existing recovery control. |
| Mid | `errorPresentation.ts` | Maps validated visible errors to stable copy and recovery disposition. |
| Low | IPC guards and component state | Reduce unknown input and replace one bounded visible state. |
