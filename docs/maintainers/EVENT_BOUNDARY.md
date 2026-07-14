# Typed Event Boundary

**Status:** Implemented checkpoint guide; not an accepted contract under
`GOVERNANCE.md` §7.

## Purpose

Phase 8 establishes the Rust-to-frontend event pattern used for ongoing or
Rust-initiated updates. Event transport remains separate from command
request/response transport.

The current runtime-status event is deliberately finite. The existing command
emits one typed `ready` update and returns. It does not start a worker, loop, or
background task.

Phase 48 adds a second finite event for native File menu selections. It carries
one closed action and starts no work until the frontend validates and dispatches
it through current application state.

Phase 47 adds a third finite event for macOS document activation. It announces
only that a Rust-owned request is queued. The file URL never enters the event
payload or frontend state.

## Runtime status event

The stable event name is:

```text
draft://runtime-status
```

The only accepted payload is:

```json
{
  "buildCommit": "0123456789abcdef0123456789abcdef01234567",
  "buildProfile": "release",
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

## Native menu action event

The stable event name is `draft://native-menu-action`. Its payload contains
exactly one `action` field with one of: `new_document`, `open_document`,
`close_document`, `save_document`, `save_document_as`,
`save_back_to_source`, or `export_docx`.

Rust creates the native item and emits the selected closed action. The frontend
validates the payload as unknown input, rejects extra fields, and passes valid
actions to `useWorkspaceActions`. The dispatcher checks live availability
before invoking the same document or export operation used by the toolbar.

## macOS application-open event

The stable event name is `draft://application-open`. Its payload is exactly
`{ "type": "available" }` or `{ "type": "queue_unavailable" }`. It carries no
URL, path, filename, content, or document identity.

The Rust run loop queues one macOS file URL before emitting `available`. React
validates the event, applies current busy and unsaved-work policy, and invokes
the path-free `open_application_document` command with `open` or `dismiss`.
Rust alone consumes and converts the queued URL.

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
| Low | `emit_native_menu_action` | Emits one selected closed File action to the main window. |
| Low | `emit_application_open_event` | Announces path-free queued application-open state. |
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
- Invalid native menu payloads leave the toolbar available and show bounded
  recovery guidance.
- Invalid application-open payloads preserve the current document and direct
  the user to the visible Open action.

## Cancellation boundary

`draft://runtime-status` announces the result of one short, synchronous command.
No work continues after the command returns, so `INV-07` cancellation does not
apply.

Any future event producer that continues after its initiating command returns
must follow the Phase 9 cancellation pattern before it is considered complete.

## Internal Stream Types

Phase 27 defines `AiStreamEvent` and an `AiEventSink` for internal Rust
orchestration. Those values are delivered only to a caller-supplied Rust sink.
They have no stable `draft://` event name, Tauri emitter, registered command,
WebView capability, frontend listener, or visible analysis workflow.

The Python helper runner returns one terminal Rust result and emits no progress
event. Runtime status, native File actions, and path-free macOS application-open
availability are the only implemented Rust-to-frontend Tauri events at this
checkpoint.

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
