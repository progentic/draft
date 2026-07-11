# OS-Native Secret Storage

## Status

This guide records implemented Phase 37 behavior. The original requirements in
`docs/drafts/SECRET_STORAGE.md` remain non-binding under
`docs/GOVERNANCE.md`.

## Scope

Phase 37 adds one Rust-owned `SecretStore` for service API keys. Tauri manages
the store during application setup, but construction is lazy and does not read,
write, delete, or probe a credential.

The production adapter uses `keyring` 4.1.4 with its native desktop stores:
Keychain on macOS, Credential Manager on Windows, and Secret Service on
supported Unix desktops. `zeroize` 1.9.0 clears owned secret bytes when Rust
releases them.

Publisher, institutional, library, and research-database login credentials
remain outside DRAFT. This boundary stores no credential until a later
Rust-owned integration explicitly calls it.

## Ownership

| Layer | Surface | Responsibility |
| :--- | :--- | :--- |
| High | `initialize_secret_store` | Registers lazy Rust-managed application state. |
| Mid | `SecretStore::{store,load,delete}` | Coordinates closed secret operations and missing-entry behavior. |
| Mid | `SecretId::service_api_key` | Builds one bounded normalized internal account slot. |
| Mid | `SecretValue` | Validates and zeroizes owned secret bytes. |
| Low | `NativeSecretBackend` | Calls binary keyring operations and drops raw native failures. |

## Data Contract

The native service namespace is fixed as `com.progentic.draft`. Account slots
use the fixed `service-api-key/` prefix plus a lowercase ASCII integration name.
Names are limited by `MAX_INTEGRATION_NAME_BYTES`; callers cannot replace the
native service name or account prefix.

`SecretValue` accepts 1 through `MAX_SECRET_BYTES` bytes. It owns a
`Zeroizing<Vec<u8>>` and does not implement `Clone`, `Debug`, `Display`, Serde,
or another text conversion. Rust consumers can borrow the bytes only through
`expose_secret`; no current consumer exists.

Store replaces the existing value for one identifier. Load returns `None` for
a missing entry. Delete returns `Deleted` or `NotFound`, making repeated
deletion deterministic.

The complete limits index is `docs/maintainers/CONFIGURATION.md`.

## Failure Contract

`SecretStoreError` contains only closed categories:

- `InvalidIdentifier`
- `EmptySecret`
- `SecretTooLong`
- `AccessDenied`
- `StoreUnavailable`
- `AmbiguousEntry`
- `InvalidStoredSecret`
- `Unsupported`

Native platform errors, keyring metadata, account values, and secret bytes are
discarded during mapping. Attached malformed-data buffers and text details are
zeroized before return. No error retains a source error or user value.

## Authority Boundary

Phase 37 adds no Tauri command, event, capability, TypeScript client, React
state, credential prompt, settings control, provider request, SQLite column,
config field, environment-variable read, filesystem fallback, Python payload,
Bash runtime path, telemetry, or log field.

The frontend cannot set or retrieve a secret. A later user-visible workflow
must preserve the Rust-only value boundary and add its own typed, non-returning
command contract.

## Verification

Seven focused Rust tests use an injected in-memory backend. They cover
identifier and value bounds, replacement, missing load, idempotent deletion,
malformed stored values, every closed backend mapping, raw keyring error
mapping, and managed-state thread safety. They never access a real operating-
system credential manager.

`scripts/check-invariants.sh` requires the pinned dependencies, constants,
binary keyring methods, zeroizing storage, application registration, tests,
and absence of forbidden authority. Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline secrets::
bash scripts/check-invariants.sh
```

## Current Limits

No user-facing API-key workflow exists. DRAFT does not currently select a
provider, request a credential, store a credential automatically, test a
credential against a service, rotate it, synchronize it, import or export it,
or fall back to application-owned encryption. Phase 38 diagnostics remains a
separate boundary and must not reveal whether a particular secret exists.
