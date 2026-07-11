# Packaging And Application Icons

## Status

Phase 42 establishes one supported package-build path for an unsigned macOS
Apple Silicon `.app`. It uses the approved application artwork and verified
bundle contract. It does not produce a signed installer, define notarization,
create a DMG, configure updates, or publish a release.

## Source And Generated Assets

One approved square PNG is the source for icon generation. Do not derive one
platform from another generated platform asset. Regenerate the complete set
with the pinned Tauri CLI:

| Source property | Approved value |
| :--- | :--- |
| Filename | `DRAFT_Logo.png` |
| Geometry | 1,254 by 1,254 pixels |
| Color | RGB PNG without source alpha |
| SHA-256 | `ce7cc5a5df592ac11873ff0f49d9c150e5a3a64e0c0ef9ffd1e05162da5fb043` |

```bash
npm run tauri -- icon path/to/DRAFT_Logo.png
```

Tauri 2.11.4 preserves the square source geometry and currently generates 52
files under `src-tauri/icons/`:

- 48 square RGBA PNG files for desktop, iOS, and Android conventions;
- one macOS `icon.icns` container;
- one Windows `icon.ico` container; and
- two Android adaptive-icon XML files.

The complete standard output is tracked so a clean checkout has the same asset
set used in review. Generated iOS and Android assets do not add a supported
mobile target or enter the desktop bundle list.

## Desktop Bundle Contract

`src-tauri/tauri.conf.json` explicitly lists only these desktop paths:

```text
icons/32x32.png
icons/128x128.png
icons/128x128@2x.png
icons/icon.icns
icons/icon.ico
```

The explicit list is required. A bundle built with an empty icon list completed
without a `CFBundleIconFile` or embedded `.icns`; conventional discovery did
not establish the contract. The corrected unsigned macOS build declares
`CFBundleIconFile = icon.icns`, and the packaged resource is byte-for-byte
identical to `src-tauri/icons/icon.icns`.

`bundle.active` is `true`, and `bundle.targets` contains only `app`. Other Tauri
bundle targets are not part of the Phase 42 package contract.

## Platform Assumptions

The supported Phase 42 package host is macOS on Apple Silicon. The repository
retains standard Windows and Linux desktop assets, but no Windows, Linux, Intel
macOS, or mobile package path is verified. A signing identity, notarization
policy, update channel, installer image, and release automation do not exist
yet.

Mobile icon derivatives are generator outputs only. DRAFT remains a desktop
application and has no iOS or Android product target.

## Verification

Build and validate the unsigned application bundle from the repository root:

```bash
npm run package:macos
```

The command rejects any host other than `Darwin arm64`, removes only the prior
ignored `DRAFT.app`, runs the pinned Tauri build, and verifies:

- `CFBundleIdentifier = com.progentic.draft`;
- `CFBundleExecutable = draft`;
- `Info.plist` contains `CFBundleIconFile = icon.icns`;
- the executable is a native Apple Silicon Mach-O binary;
- the embedded and tracked `.icns` files match byte-for-byte; and
- the application exists at
  `src-tauri/target/release/bundle/macos/DRAFT.app`.

`bash scripts/check-packaging.sh` separately parses Tauri and npm configuration,
requires the exact app target and five desktop icons, and checks the owned build
script contract on every local and hosted verification run.

The full repository verifier remains required:

```bash
bash scripts/verify.sh
```

## Troubleshooting

If an `.app` builds without the DRAFT icon, check the explicit `bundle.icon`
array before regenerating assets. Regeneration cannot fix a missing bundle
reference.

If Tauri rejects an image, confirm that the source is square and that generated
PNG outputs are RGBA. Do not hand-edit one derivative; fix the approved source
or generation command and regenerate the complete set.

Do not commit files from `src-tauri/target/`. The unsigned app is reproducible
build output, not a published release artifact.

## Configuration Index

Bundle activation, app target, canonical command, desktop icon paths, window
defaults, and platform assumptions are indexed in
`docs/maintainers/CONFIGURATION.md`.
