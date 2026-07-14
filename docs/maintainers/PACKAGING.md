# Packaging And Application Icons

## Status

Phase 42 establishes one supported package-build path for an unsigned macOS
Apple Silicon `.app`. It uses the approved application artwork and verified
bundle contract. It does not produce a signed installer, define notarization,
create a DMG, configure updates, or publish a release.

## Source And Generated Assets

One approved square PNG at `assets/DRAFT_Logo.png` is the source for icon generation. Do not derive one
platform from another generated platform asset. Regenerate the complete set
with the pinned Tauri CLI:

| Source property | Approved value |
| :--- | :--- |
| Filename | `DRAFT_Logo.png` |
| Geometry | 1,254 by 1,254 pixels |
| Color | RGB PNG without source alpha |
| SHA-256 | `ce7cc5a5df592ac11873ff0f49d9c150e5a3a64e0c0ef9ffd1e05162da5fb043` |

```bash
npm run tauri -- icon assets/DRAFT_Logo.png
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

`scripts/check-packaging.sh` pins the canonical source hash and the stable
desktop PNG/ICO derivatives. The `.icns` container is structurally checked and
the supported-host package command requires its embedded copy to match the
tracked container byte-for-byte. The visible workspace mark uses the generated
`32x32.png`, so in-window and packaged identity derive from the same source.

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

The bundle also registers `.draft` as the owned `DRAFT Document` type with the
exported identifier `com.progentic.draft.document`. The document type uses the
same `icon.icns` artwork as the application. macOS can therefore route a
double-clicked `.draft` file to DRAFT without making the WebView aware of its
path. The Rust run-event queue retains the path until the typed document-open
boundary consumes it.

Product SemVer remains governed separately from validation-artifact identity.
`scripts/package-macos.sh` requires a clean worktree, reads the exact
40-character Git commit, and supplies it to the Rust build as
`DRAFT_BUILD_COMMIT`. Cargo also embeds the `release` profile. The running
workspace shows the product version, short commit, and profile; the complete
commit remains embedded for mechanical comparison. No timestamp is embedded,
so this mechanism does not introduce a reproducibility exception.

`bundle.active` is `true`, and `bundle.targets` contains only `app`. Other Tauri
bundle targets are not part of the Phase 42 package contract.

## Platform Assumptions

The supported Phase 42 package host is macOS on Apple Silicon. The repository
retains standard Windows and Linux desktop assets, but no Windows, Linux, Intel
macOS, or mobile package path is verified. A signing identity, notarization
policy, update channel, installer image, and release automation do not exist
yet. Those distribution and CSP gates are tracked in
`docs/maintainers/RELEASE_CANDIDATE.md`; this unsigned package is supported-host
evidence, not final-candidate distribution.

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
- `Info.plist` declares the owned `.draft` document type and exported UTI;
- the executable is a native Apple Silicon Mach-O binary;
- the executable contains the exact clean-worktree Git commit used for the
  package;
- the embedded and tracked `.icns` files match byte-for-byte; and
- the packaged `python/draft_helpers` files are present;
- `/usr/bin/python3` is available on the supported host; and
- the embedded `text_analysis` helper returns the expected typed finding under
  an isolated environment; and
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

If the packaged helper probe fails, confirm the bundle contains the complete
`Resources/python/draft_helpers` package and that `/usr/bin/python3` satisfies
the documented Python 3.9 minimum. Do not add a downloaded runtime or restore
the invoking shell environment.

If Finder opens a `.draft` file in another application, first inspect the built
bundle's `CFBundleDocumentTypes` and `UTExportedTypeDeclarations`. A correct
bundle can still require Launch Services to refresh after replacing an older
copy. Do not work around registration by returning the selected path to React.

If the visible short commit does not match the package under review, discard
that validation session. The executable SHA-256 remains the evidence artifact
identity, while the embedded commit proves which clean repository revision
produced it.

Do not commit files from `src-tauri/target/`. The unsigned app is reproducible
build output, not a published release artifact.

## Configuration Index

Bundle activation, app target, canonical command, desktop icon paths, window
defaults, and platform assumptions are indexed in
`docs/maintainers/CONFIGURATION.md`.
