# Frontend Command Client

## Purpose

Phase 7 establishes the only frontend path to trusted Rust commands. React
components and feature hooks use typed command wrappers. They never import or
call Tauri `invoke` directly.

This boundary keeps command names, payload envelopes, runtime validation, and
transport failures out of presentation components.

## Layering

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `DraftWorkspace` | Coordinates transient editor and runtime-status UI state. |
| Mid | `useRuntimeStatus` | Resolves the command result into transient connection state. |
| Mid | `getRuntimeStatus` | Owns the command name, request envelope, response validation, and error classification. |
| Low | `invokeCommand` | Calls the raw Tauri `invoke` API and returns unknown IPC data to its wrapper. |

Raw Tauri access is isolated in `src/ipc/client.ts`. Command-specific wrappers
live beside it under `src/ipc/`. React hooks live under their feature folder and
receive typed wrapper results.

## Runtime status wrapper

`getRuntimeStatus` sends this Tauri argument object:

```json
{
  "request": {}
}
```

It invokes `get_runtime_status` and validates the response before exposing it
to React. The accepted success shape is:

```json
{
  "version": "0.1.0"
}
```

The wrapper returns a discriminated result instead of throwing:

```text
ready(version)
error(command: invalid_application_version)
error(command: event_delivery_failed)
error(invalid-response)
error(transport)
```

Unknown transport failures are classified without retaining or displaying raw
error details. This prevents implementation or environment details from
leaking into the UI.

## UI behavior

Phase 8 registers the typed runtime-status event listener before invoking this
wrapper. A successful command response confirms contract integrity, while the
validated event drives the ready-state transition. `useRuntimeStatus` exposes
one of three transient states:

- `checking`
- `ready` with the Rust application version
- `unavailable` with a bounded reason category

The document inspector displays `Connecting to core`, `Core v<version>`, or
`Core unavailable`. This state is not persisted and does not make the frontend
authoritative for runtime metadata.

A standalone Vite browser does not have a Tauri runtime and therefore reports
the core as unavailable. The desktop application resolves the registered Rust
command.

## Enforcement

`scripts/check-invariants.sh` rejects `@tauri-apps/api/core` imports, raw
`invoke(...)` calls, and generic `invokeCommand(...)` calls outside `src/ipc/`.
`scripts/check-repository.sh` requires the low-level client, runtime-status
wrapper, and wrapper tests to remain tracked.

Frontend tests prove:

- exact command name and request arguments
- valid response mapping
- invalid response rejection
- command-specific error preservation
- unknown transport error classification without detail leakage
- workspace rendering of the connected Rust version

Run the focused evidence with:

```bash
npm test
bash scripts/check-invariants.sh
```

## Adding another command wrapper

1. Define explicit request, response, result, and client-error types.
2. Keep the command name and Tauri argument envelope inside `src/ipc/`.
3. Receive the low-level response as `unknown` and validate it.
4. Classify command and transport failures without exposing raw details.
5. Return a discriminated result to the feature hook.
6. Add exact request, success, invalid-response, command-error, and transport
   tests.
7. Let React own display state only; durable decisions remain in Rust.

Phase 8 event transport is documented in
`docs/maintainers/EVENT_BOUNDARY.md`. It remains separate from this
request/response abstraction.
