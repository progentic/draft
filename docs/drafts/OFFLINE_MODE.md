# Offline Mode Requirements Draft

## Status

This is the non-binding requirements draft for Phase 36. It defines the next
implementation boundary after the Phase 35 documentation realignment. It is not
an accepted contract under `docs/GOVERNANCE.md`.

## Purpose

DRAFT needs a deterministic session mode that prevents external work while
keeping implemented local workflows available. Offline behavior must be a
Rust-enforced policy, not a frontend convention or a guess based on one failed
request.

## Scope

Phase 36 introduces one closed Rust-owned session policy with `online` and
`offline` states. The frontend may request a mode change and display the
current state, but Rust validates, stores, and enforces the active value.

Offline mode must reject new DRAFT-owned metadata requests and system-browser
handoffs before transport or opener work begins. Existing typed boundaries must
return a distinct offline result that presentation code can identify without
raw transport details.

## Degraded Behavior

Offline mode must leave these implemented local boundaries available:

- transient document editing and toolbar formatting;
- the Phase 34 formatting review and explicit current-target heading actions;
- local document validation, registry, and save/export foundations;
- the local reference store and citation resolution from stored records;
- PDF intake validation and durable import-job state transitions that do not
  start processing; and
- deterministic local helper and text-analysis boundaries when their runtime is
  available.

This list describes internal capabilities as well as visible ones. It does not
claim that an unwired backend boundary has gained a user workflow.

## State And Command Boundary

The session starts in the documented default mode. A bounded typed command may
set one closed value and return the effective Rust-owned state. A separate read
command is optional if the existing response and startup state provide one
authoritative value without duplication.

No command accepts a URL, credential, proxy, retry instruction, arbitrary
provider, or network-client configuration. The frontend cannot bypass the gate
by calling Tauri, opener, browser, or HTTP APIs directly.

## Enforcement

The offline check must happen before rate-limit reservation, request creation,
socket work, or system-browser launch. Metadata providers preserve their typed
`Offline` mapping. External handoff adds a typed offline distinction rather
than collapsing policy denial into browser unavailability.

Switching to offline mode blocks new external operations. Phase 36 does not
promise cancellation of an operation that was already dispatched before the
mode change; that requires a separately bounded lifecycle if a visible
long-running network workflow is later introduced.

## User Experience

The workspace must expose the current session mode with a labeled binary
control and a concise status. Keyboard and assistive-technology users must be
able to read and change it. A failed mode change leaves the prior effective
state visible.

Offline copy must explain that local work remains available and external
metadata or browser access is paused for the session. It must not imply that
the operating system has no network connection.

## Acceptance Tests

Phase 36 must prove:

- the closed state and command serialization are stable;
- unknown request fields and mode values fail closed;
- offline policy denies metadata and browser handoff before adapter work;
- existing online request and handoff behavior remains unchanged;
- metadata denial remains the typed `Offline` result;
- browser denial has a distinct typed offline code;
- local formatting review remains operational in offline mode;
- the frontend control is keyboard accessible and reflects Rust-owned state;
- no connectivity probe, automatic retry, queued request, persistence,
  telemetry, credential, proxy, or alternate network path is added; and
- local and GitHub Actions verification run the same focused tests and scans.

## Non-Goals

Phase 36 does not add operating-system reachability monitoring, captive-portal
detection, background retries, request queues, sync, conflict resolution,
persisted preferences, account state, API keys, proxy settings, secret storage,
telemetry, update checks, service workers, browser storage, or a new research
workflow. Phase 37 secret storage remains a separate architecture boundary.
