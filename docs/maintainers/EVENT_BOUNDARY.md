# Typed Event Boundary

## Purpose

Phase 8 establishes the Rust-to-frontend event pattern used for ongoing or
Rust-initiated updates. Event transport remains separate from command
request/response transport.

The current runtime-status event is deliberately finite. The existing command
emits one typed `ready` update and returns. It does not start a worker, loop, or
background task.

## Runtime status event

The stable event name is:

```text
draft://runtime-status
```

The only accepted payload is:

```json
{
  "type": "ready",
  "version": "0.1.0"
}
```

Rust owns the event name and payload construction. The frontend receives the
payload as `unknown`, rejects extra or invalid fields, and exposes only the
typed `ready` variant to feature code.

If Rust cannot deliver the event, `get_runtime_status` returns its
command-specific error:

```json
{
  "code": "event_delivery_failed"
}
```

## Lifecycle

The runtime-status session follows this order:

1. React starts the transient runtime-status session.
2. `listenToRuntimeStatus` registers the typed event listener.
3. The session invokes `get_runtime_status` through its typed command wrapper.
4. Rust builds the status response and emits `draft://runtime-status`.
5. The frontend validates the event and moves to `ready` from the event.
6. Rust returns the command response, which is still validated for contract
   integrity but does not duplicate the ready-state transition.
7. React removes the listener when the owning hook unmounts.

Registering before invoking prevents the finite event from racing ahead of the
listener. Listener cleanup prevents stale callbacks and duplicate updates after
component remounts.

## Ownership layers

| Layer | Item | Responsibility |
| :--- | :--- | :--- |
| High | `startRuntimeStatusSession` | Registers, invokes, maps failures, and returns cleanup. |
| Mid | `get_runtime_status` | Builds status, emits its documented event, and returns a typed result. |
| Mid | `listenToRuntimeStatus` | Validates unknown payloads and exposes the typed event. |
| Low | `emit_runtime_status` | Calls the raw Tauri Rust emitter. |
| Low | `listenToEvent` | Calls the raw Tauri TypeScript listener and extracts unknown payload data. |

Raw TypeScript event APIs are isolated in `src/ipc/eventClient.ts`. Raw Rust
emission stays under `src-tauri/src/events/`. Commands and feature hooks call
typed event functions rather than framework APIs.

## Desktop capability

Tauri 2 requires an explicit capability before a WebView may call core event
listener APIs. `src-tauri/capabilities/main.json` binds the capability to the
window labeled `main` and grants only:

```text
core:event:allow-listen
core:event:allow-unlisten
```

The capability does not grant frontend event emission. Rust remains the event
producer, and the WebView can only register or remove listeners.

## Failure behavior

- Invalid event payloads resolve the UI to a bounded `unavailable` state.
- Event-listener setup failure maps to `transport` without exposing raw error
  text.
- Command failure after listener setup maps through the existing typed command
  result and keeps the listener cleanup function available.
- Rust event delivery failure is observable as `event_delivery_failed`.

## Cancellation boundary

`draft://runtime-status` announces the result of one short, synchronous command.
No work continues after the command returns, so `INV-07` cancellation does not
apply.

Any future event producer that continues after its initiating command returns
must follow the Phase 9 cancellation pattern before it is considered complete.

## Enforcement

Rust tests pin the event name, payload serialization, expanded command error
serialization, and command signature. Frontend tests pin listener ordering,
payload validation, setup failure, command failure, event-driven state, and
unmount cleanup.

`scripts/check-invariants.sh`:

- rejects raw or generic Tauri event listening outside `src/ipc/`
- requires least-privilege listen and unlisten permissions for the main window
- rejects default or frontend event-emission permissions
- requires one stable event-name test per Rust event type
- requires one stable payload-serialization test per Rust event type

Run the focused evidence with:

```bash
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
npm test
bash scripts/check-invariants.sh
```
