# Offline Session Mode

## Status

This guide records implemented Phase 36 behavior. The requirements in
`docs/drafts/OFFLINE_MODE.md` remain non-binding under `docs/GOVERNANCE.md`.

## Scope

Phase 36 adds one process-local `ConnectivityPolicy` with exactly `Online` and
`Offline` modes. Every new application session starts online through
`DEFAULT_CONNECTIVITY_MODE`. Rust stores the effective value behind a mutex and
shares the same policy with the centralized metadata client, browser handoff,
and typed connectivity commands.

The mode is explicit session policy. It is not operating-system reachability,
does not change automatically after a transport error, and is not persisted.

## Ownership Flow

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| High | `initialize_network_client` | Creates and manages one shared policy and network client. |
| Mid | `ConnectivityPolicy` | Stores, returns, changes, and enforces the closed session mode. |
| Mid | `get_connectivity_mode`, `set_connectivity_mode` | Expose only the effective closed Rust-owned value. |
| Mid | `NetworkClient::get_metadata`, `open_in_system_browser` | Require online policy before external work. |
| Mid | `useConnectivityMode` | Mirrors typed state and ignores stale frontend requests. |
| Low | `ConnectivityModeControl` | Presents one labeled session toggle and bounded failures. |

## External Denial

`NetworkClient::get_metadata` checks the policy before provider rate
reservation, URL validation, request construction, or socket work. Offline
mode returns `NetworkRequestError::Offline`; unavailable policy state returns
`ClientUnavailable`. Existing online validation and request policy are
unchanged.

`open_in_system_browser` checks the same policy before destination validation
or opener invocation. Its command now distinguishes `offline` and
`connectivity_unavailable` from `browser_unavailable`.

Switching offline blocks new operations. It does not cancel work already
dispatched before the mode change.

## Command And Frontend Contract

`get_connectivity_mode` accepts an empty request. `set_connectivity_mode`
accepts only `online` or `offline`; unknown values and fields fail
deserialization. Both return `{ "mode": "..." }` and expose only
`connectivity_unavailable` as a command failure.

The TypeScript clients validate exact responses and require the set response to
match the requested value. The hook retains the last confirmed mode while a
change is pending or fails and ignores an older read after a newer request.

The header toggle uses native button and toggle semantics. It exposes `Work
offline` or `Go online`, remains reachable at minimum width, and announces a
failed change while preserving the last confirmed mode. A browser-only preview
offers a retry because no Tauri core is present.

Phase 39 keeps command, invalid-response, and transport copy distinct. An
unreadable mode reuses the existing retry control; a failed change reuses the
existing mode toggle. No failure presentation creates another connectivity
action. See `docs/maintainers/ERROR_UX.md`.

## Local Behavior

Connectivity policy is not imported by the formatting domain or formatting
review feature. Editing, formatting review, local citation resolution,
document validation/save foundations, DOCX compilation, local stores, PDF
intake/job state, and deterministic helper boundaries remain independent. This
does not add visible controls for internal-only workflows.

## Verification

Rust tests prove the closed default/state, typed command contracts, offline
denial before invalid URL or opener work, and unchanged online URL validation.
Frontend tests cover exact IPC, closed failures, stale reads, pending and failed
state, toggle semantics, retry, and workspace integration.

`scripts/check-invariants.sh` requires those tests and markers. It denies
persistence, filesystem work, `reqwest`, browser storage, fetch, reachability
probing, timers, and formatting-policy coupling in the Phase 36 surface. The
complete configuration index is `docs/maintainers/CONFIGURATION.md`.

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline network::
npm test -- --run src/ipc/connectivityMode.test.ts src/features/connectivity src/App.test.tsx
bash scripts/check-invariants.sh
```

## Current Limits

The mode resets online on restart. DRAFT does not monitor the operating system,
detect captive portals, persist the preference, retry or queue requests,
cancel already dispatched work, or add proxy settings or credential controls.
The implemented Rust secret store remains a separate internal boundary with no
connectivity or frontend integration.
