# Error Message Inventory

**Status:** Maintenance inventory of current typed command boundaries and
visible messages. This is not a complete product-wide error review.

## Purpose

DRAFT keeps raw command and transport details out of the interface. Rust
commands serialize bounded failures, and TypeScript wrappers validate those
same codes before exposing them to presentation state. A visible workflow can
then offer a specific, useful message without exposing implementation details.

## Current inventory

| Surface | Rust and TypeScript failure codes | Current presentation |
| :--- | :--- | :--- |
| Runtime status | `invalid_application_version`, `event_delivery_failed` | Visible in the document inspector through the Phase 39 presentation policy. |
| Worker cancellation | `invalid_worker_id`, `worker_not_found`, `registry_unavailable` | No visible workflow currently consumes this wrapper. |
| Document open | `unsupported_file_location`, `file_not_found`, `read_failed`, `malformed_json`, `invalid_envelope`, `registry` | No visible workflow currently consumes this wrapper. |
| Document save | `unsupported_file_location`, `serialization_failed`, `durability_uncertain`, `write_failed`, `invalid_envelope`, `registry` | No visible workflow currently consumes this wrapper. |
| Citation resolution | `invalid_citation`, `reference_not_found`, `reference_store` | The citation node renders bounded invalid, unavailable, or failed copy. No citation-management workflow exists. |
| External access | `invalid_url`, `invalid_doi`, `invalid_search_query`, `offline`, `connectivity_unavailable`, `browser_unavailable` | No visible research workflow currently consumes this wrapper. |
| Connectivity mode | `connectivity_unavailable` | Command, invalid-response, and transport failures remain distinct. The header retains the last confirmed mode or reuses its retry control. |
| Formatting review | `too_many_headings`, `too_many_citations`, `invalid_heading_level`, `empty_heading_title`, `heading_title_too_long`, `invalid_citekey` | Every code has actionable copy. Invalid responses and transport failures reuse the existing formatting check. |
| Native secret storage | `InvalidIdentifier`, `EmptySecret`, `SecretTooLong`, `AccessDenied`, `StoreUnavailable`, `AmbiguousEntry`, `InvalidStoredSecret`, `Unsupported` | Rust-only internal errors; no command or visible credential workflow exists. |
| Local diagnostic snapshot | `invalid_application_version`, `snapshot_serialization_failed`, `snapshot_too_large` | Typed command/client boundary only; no component or visible support workflow consumes it. |

Every command wrapper also distinguishes an invalid response from a transport
failure. Runtime events add an invalid-payload failure. Document, citation, and
store errors retain bounded nested causes where the caller needs a stable
distinction; raw paths, runtime details, and internal error strings do not
enter presentation state.

## Phase 39 Presentation Policy

`docs/maintainers/ERROR_UX.md` owns the frontend policy for runtime status,
connectivity, formatting review, and citation rendering. Every mapped failure
is `retryable`, `actionable`, or `terminal`. A label is allowed only when an
existing control can honor it; the policy creates no new action.

The four visible unions are exhaustive at compile time. The runtime command
mapper retains one outer fallback for malformed or future command input.
Unknown IPC input at the other surfaces is already reduced to invalid-response
or transport before presentation. Typed but unwired errors remain in this
inventory and receive no speculative visible copy.

## Runtime status messages

The runtime-status session preserves the complete typed error through its
transient state. The document inspector maps the known command codes as
follows:

| Error | Message |
| :--- | :--- |
| `invalid_application_version` | DRAFT received an unsupported application version. |
| `event_delivery_failed` | DRAFT could not deliver the core status event. |
| Unknown command failure | DRAFT could not read the core status. |
| Invalid response or event payload | Core status invalid |
| Transport failure | Core unavailable |

The unknown-command message is a defensive fallback. The current command
wrapper accepts only the two documented runtime-status command codes.

## Recovery Guidance

Visible copy is incomplete unless the user has a next action. Canonical user
guidance lives in `docs/wiki/Troubleshooting.md`:

| Visible failure | Recovery action |
| :--- | :--- |
| Unsupported application version | Install one complete matching DRAFT build, restart, and report the source/version if it repeats. |
| Core status event delivery failure | Restart DRAFT and report the version and exact message if it repeats. |
| Unknown core command failure | Restart DRAFT and report the version and exact message if it repeats. |
| Invalid response or event payload | Restart DRAFT and report the version if the status remains invalid. |
| Transport failure | Use the desktop app rather than browser preview, restart, and report the operating-system and DRAFT versions if it repeats. |
| Formatting input cannot be checked | Correct the identified heading or citation when possible, then run the formatting check again. Split an unusually large document when the heading or citation count is the stated limit. |
| Formatting response is invalid or core is unavailable | Run the check again. Restart DRAFT and report the version and exact message if the failure repeats. |
| Connectivity mode unavailable | Retry from the header. Restart DRAFT and report the version and exact visible message if it repeats. |
| Connectivity mode change failed | The visible prior mode remains effective. Retry the change or continue local work in that mode. |
| Citation input is invalid | Keep the citation unchanged. The workspace has no citation-repair control. |
| Citation cannot be resolved or read | Keep the citation unchanged. Restart DRAFT only when the visible message directs it. |

Maintainer copy, rendered copy, and the Wiki recovery page must change together.
Typed errors for commands with no visible workflow stay in the inventory only;
the project does not invent user instructions before a real control owns the
action.

## Excluded Boundaries

Document files, workers, external access, metadata, secrets, diagnostics,
imports, exports, and other unwired boundaries remain inventory-only. Phase 39
does not add commands, controls, recovery workflows, or frontend authority for
them, and it does not define a generalized application-wide error framework.
