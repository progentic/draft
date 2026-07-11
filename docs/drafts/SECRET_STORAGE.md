# Secret Storage Requirements Draft

## Status

This is the non-binding requirements draft for Phase 37. It defines the
implementation boundary after the verified Phase 36 offline-mode work. It is
not an accepted contract under `docs/GOVERNANCE.md`.

## Purpose

DRAFT needs one Rust-owned path for service API keys that uses the operating
system credential manager instead of application files, SQLite, browser
storage, environment variables, or logs.

Publisher, institutional, library, and scholarly-database login credentials
remain permanently outside DRAFT. Phase 37 does not weaken `INV-01` or turn the
system-browser handoff into an authenticated application flow.

## Scope

Phase 37 adds one managed Rust secret store backed by the platform-native
credential manager:

- Keychain on macOS;
- Credential Manager on Windows; and
- Secret Service on supported Unix desktops.

The store supports bounded service API-key identifiers and three operations:
store, load, and delete. Native access is lazy so application startup does not
prompt for or probe the credential manager.

No Tauri command, event, TypeScript wrapper, frontend state, Python payload, or
Bash runtime path may carry a secret value in this phase.

## Secret Value Contract

Secret values must be non-empty and bounded. Rust owns their memory, clears it
on drop, and exposes bytes only to Rust code that already owns the trusted
operation. The value type must not implement serialization, cloning, display,
or debug formatting.

Errors may identify the failed operation and a closed recovery category. They
must not retain a secret, account value, platform error, credential-manager
payload, or raw backend detail.

## Identifier Contract

Secret identifiers are internal Rust values for service API keys. They use a
bounded normalized integration name and a fixed DRAFT service namespace.
Callers cannot supply a credential-manager service name, account prefix,
publisher login, institutional login, filesystem path, URL, or arbitrary
metadata field.

## Native Adapter

The production adapter uses the maintained Rust `keyring` crate with its native
desktop store feature set. The dependency remains pinned in `Cargo.lock` and
must support the repository Rust version and license policy.

Backend errors map immediately into closed DRAFT errors. Missing entries map to
an absent result for load and an idempotent absent result for delete. Ambiguous,
locked, unavailable, unsupported, malformed, and invalid store outcomes remain
distinct where recovery differs.

## Verification

Tests use an injected in-memory backend. They must not create, read, update, or
delete a real credential in a developer or hosted CI credential manager.

Phase 37 must prove:

- identifier and value bounds fail closed;
- store, load, replacement, and idempotent deletion are deterministic;
- missing entries are not generic failures;
- every native backend error maps to a bounded DRAFT category;
- error and diagnostic text cannot include secret bytes;
- the production store is registered as Rust-managed application state;
- no secret command, event, frontend state, browser storage, config field,
  SQLite column, Python input, environment-variable path, or log field exists;
  and
- local and GitHub Actions verification run the same focused tests and scans.

## Non-Goals

Phase 37 does not add an account system, settings screen, credential prompt,
provider selection, API call, model integration, metadata-provider key,
institutional login, publisher login, browser automation, credential import or
export, secret synchronization, rotation scheduler, telemetry, or fallback
application-owned encryption store.

The first user-visible secret workflow requires its own bounded phase and must
preserve the Rust-only value boundary established here.
