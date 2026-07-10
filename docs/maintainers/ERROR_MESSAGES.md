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
| Runtime status | `invalid_application_version`, `event_delivery_failed` | Visible in the document inspector. Known command failures have distinct messages. |
| Worker cancellation | `invalid_worker_id`, `worker_not_found`, `registry_unavailable` | No visible workflow currently consumes this wrapper. |
| Document open | `unsupported_file_location`, `file_not_found`, `read_failed`, `malformed_json`, `invalid_envelope`, `registry` | No visible workflow currently consumes this wrapper. |
| Document save | `unsupported_file_location`, `serialization_failed`, `durability_uncertain`, `write_failed`, `invalid_envelope`, `registry` | No visible workflow currently consumes this wrapper. |
| Citation resolution | `invalid_citation`, `reference_not_found`, `reference_store` | The citation node renders a bounded unavailable state; no product-wide message mapping exists. |
| External access | `invalid_url`, `invalid_doi`, `invalid_search_query`, `browser_unavailable` | No visible workflow currently consumes this wrapper. |
| Formatting review | `too_many_headings`, `too_many_citations`, `invalid_heading_level`, `empty_heading_title`, `heading_title_too_long`, `invalid_citekey` | Visible in the formatting review band with code-specific bounded messages. |

Every command wrapper also distinguishes an invalid response from a transport
failure. Runtime events add an invalid-payload failure. Document, citation, and
store errors retain bounded nested causes where the caller needs a stable
distinction; raw paths, runtime details, and internal error strings do not
enter presentation state.

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

Maintainer copy, rendered copy, and the Wiki recovery page must change together.
Typed errors for commands with no visible workflow stay in the inventory only;
the project does not invent user instructions before a real control owns the
action.

## Deferred work

Future visible workflows should map their existing typed failures when those
workflows are introduced. This inventory does not add commands, controls, or
frontend authority, and it does not define a complete error-experience policy.
