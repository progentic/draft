# Packaging And Application Icons

## Status

DRAFT has approved application artwork and a verified macOS `.app` icon
contract. Packaging remains inactive in normal configuration. This is release
groundwork only and does not complete Phase 42, produce an installer, define
signing or notarization, or publish a release.

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

`bundle.active` remains `false`. Supplying `--bundles app` during verification
is an explicit one-command override and is not a release configuration change.

## Platform Assumptions

The intended initial release platform is macOS on Apple Silicon. The repository
also retains standard Windows and Linux desktop assets, but no reproducible
installer, signing identity, notarization policy, update channel, package test,
or release automation exists yet.

Mobile icon derivatives are generator outputs only. DRAFT remains a desktop
application and has no iOS or Android product target.

## Verification

Inspect the environment and build an unsigned application bundle:

```bash
npm run tauri -- info
npm run tauri -- build --bundles app --no-sign
```

Then verify:

- every generated PNG is square and RGBA;
- `.icns` and `.ico` expose alpha-capable images;
- `Info.plist` contains `CFBundleIconFile = icon.icns`;
- the embedded and tracked `.icns` SHA-256 values match;
- the five desktop icon paths remain exact; and
- no product, UI, roadmap, architecture, invariant, PDF, or phase file entered
  the packaging diff.

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

Do not commit files from `src-tauri/target/`. Bundles are verification output,
not release artifacts.

## Configuration Index

Bundle activation, desktop icon paths, window defaults, and platform assumptions
are indexed in `docs/maintainers/CONFIGURATION.md`.
