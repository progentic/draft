# Rust Command Boundary

## Purpose

Phase 6 establishes the Tauri request/response boundary used by trusted Rust
work. It provides one registered command and the enforcement pattern every
later command must follow.

Phase 6 did not add a frontend command client. The current typed wrapper is
documented in `docs/maintainers/FRONTEND_COMMAND_CLIENT.md`.

## Current command

`get_runtime_status` confirms that the Rust process has valid application
metadata compiled into it.

The command accepts a bounded request envelope with no fields:

```json
{
  "request": {}
}
```

Unknown request fields are rejected before domain work begins. The command
returns:

```json
{
  "version": "0.1.0"
}
```

The response version comes from Rust package metadata. The frontend does not
supply or decide it.

The command-specific error codes are:

```json
{ "code": "invalid_application_version" }
{ "code": "event_delivery_failed" }
```

The delivery error is returned when the documented Phase 8 runtime-status
event cannot reach current frontend listeners.

No generic string, `anyhow::Error`, `serde_json::Value`, or boxed error crosses
the IPC boundary.

## Ownership layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `run` | Registers commands and starts the Tauri runtime. |
| Mid | `get_runtime_status` | Coordinates domain status construction and maps the result to command DTOs. |
| Mid | `current_runtime_status` | Builds Rust-owned application status from compiled metadata. |
| Low | `validated_version` | Normalizes and rejects an empty package version. |

The command module owns only Tauri-facing DTOs and error mapping. The
application module owns status construction. This keeps Tauri mechanics out of
domain logic and prevents React from becoming authoritative for runtime state.

## Required command shape

Every Tauri command added after this checkpoint must have:

- a bounded request DTO that rejects unknown fields
- a concrete serialized response type
- a command-specific serialized error enum
- a thin handler that calls Rust-owned domain logic
- explicit registration in `tauri::generate_handler!`
- a compile-time signature test named `command_signature_is_typed`
- a request test named `request_deserialization_is_stable`
- a response test named `response_serialization_is_stable`
- an error test named `error_serialization_is_stable`

Commands must not hide durable side effects. A response or documented event
must make the result observable to the frontend.

`get_runtime_status` emits the finite event documented in
`docs/maintainers/EVENT_BOUNDARY.md` before returning its response.

## Enforcement

Rust tests cover valid and blank version inputs, the exact command signature,
bounded request deserialization, and stable JSON for both success and error
values.

`scripts/check-invariants.sh` rejects generic Rust error patterns and compares
the number of Tauri commands with registered handlers, typed signature tests,
request deserialization tests, response serialization tests, and error
serialization tests. The aggregate local verifier and GitHub Actions run that
same scan.

Run the focused Rust evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
bash scripts/check-invariants.sh
```

## Deferred boundary work

- Phase 7 adds the TypeScript wrapper described in
  `docs/maintainers/FRONTEND_COMMAND_CLIENT.md`.
- Phase 8 adds the typed event path described in
  `docs/maintainers/EVENT_BOUNDARY.md`.
- Phase 9 adds cancellation behavior for long-running user-initiated work.
- Product commands are introduced only in their owning phases with their
  domain models and negative-path tests.
