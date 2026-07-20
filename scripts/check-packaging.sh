#!/usr/bin/env bash
set -euo pipefail

# Verifies the portable Phase 42 package configuration without building a
# platform-specific artifact.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools node rg
  require_packaging_sources
  check_structured_configuration
  check_package_script_contract
  check_visible_build_identity_contract

  printf 'Packaging contract checks passed.\n'
}

require_packaging_sources() {
  local source_path
  local required_sources=(
    assets/DRAFT_Logo.png
    package.json
    scripts/package-macos.sh
    src-tauri/Info.plist
    src-tauri/src/desktop_menu.rs
    src-tauri/tauri.conf.json
    src-tauri/icons/32x32.png
    src-tauri/icons/128x128.png
    src-tauri/icons/128x128@2x.png
    src-tauri/icons/icon.icns
    src-tauri/icons/icon.ico
  )

  for source_path in "${required_sources[@]}"; do
    require_file "${source_path}"
  done
}

check_structured_configuration() {
  node --input-type=module -e '
    import crypto from "node:crypto";
    import fs from "node:fs";
    const config = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
    const manifest = JSON.parse(fs.readFileSync("package.json", "utf8"));
    const expectedIcons = [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ];
    const expectedFileAssociations = [{
      ext: ["draft"],
      name: "DRAFT Document",
      role: "Editor",
      rank: "Owner",
      mimeType: "application/vnd.progentic.draft+json",
      exportedType: {
        identifier: "com.progentic.draft.document",
        conformsTo: ["public.json", "public.data"]
      }
    }];
    const expectedHashes = new Map([
      ["assets/DRAFT_Logo.png", "ce7cc5a5df592ac11873ff0f49d9c150e5a3a64e0c0ef9ffd1e05162da5fb043"],
      ["src-tauri/icons/32x32.png", "111d44f795b2945b61627faf3c2508b8280b9ecd4508abfcb13b3befef4fe98d"],
      ["src-tauri/icons/128x128.png", "1d245717e19b250131fed1c51e2684810892216f3864b7639a12f05a250fbf84"],
      ["src-tauri/icons/128x128@2x.png", "fcb5d936aef3c07be472812d189dd46a586d6d26cdd918ba6986bdcd22edf4eb"],
      ["src-tauri/icons/icon.ico", "d70beb61bf0b4a2a5fa1d031201449f6cc9381f6192eeec966ef4277c7a65db3"]
    ]);
    if (config.bundle?.active !== true) throw new Error("bundle.active must be true");
    if (JSON.stringify(config.bundle.targets) !== JSON.stringify(["app"])) {
      throw new Error("bundle.targets must contain only app");
    }
    if (JSON.stringify(config.bundle.icon) !== JSON.stringify(expectedIcons)) {
      throw new Error("desktop icon paths do not match the package contract");
    }
    if (JSON.stringify(config.bundle.fileAssociations) !== JSON.stringify(expectedFileAssociations)) {
      throw new Error("DRAFT document association does not match the package contract");
    }
    if (manifest.scripts?.["package:macos"] !== "bash scripts/package-macos.sh") {
      throw new Error("package:macos must invoke the owned package script");
    }
    for (const [path, expected] of expectedHashes) {
      const actual = crypto.createHash("sha256").update(fs.readFileSync(path)).digest("hex");
      if (actual !== expected) throw new Error("icon asset hash drifted: " + path);
    }
    if (fs.readFileSync("src-tauri/icons/icon.icns").subarray(0, 4).toString() !== "icns") {
      throw new Error("macOS icon container is invalid");
    }
  '
}

check_package_script_contract() {
  local script_path='scripts/package-macos.sh'
  local marker
  local required_markers=(
    'Darwin'
    'arm64'
    'npm run tauri -- build --bundles app --no-sign'
    'DRAFT_BUILD_COMMIT'
    'git rev-parse HEAD'
    'CFBundleDocumentTypes.0.CFBundleTypeIconFile'
    'UTExportedTypeDeclarations.0.UTTypeIdentifier'
    'CFBundleIdentifier'
    'CFBundleIconFile'
    'Contents/MacOS/draft'
    'Contents/Resources/icon.icns'
    'require_embedded_build_identity'
  )

  for marker in "${required_markers[@]}"; do
    if ! rg --quiet --fixed-strings "${marker}" "${script_path}"; then
      printf 'Missing packaging marker %s in %s\n' "${marker}" "${script_path}" >&2
      return 1
    fi
  done
}

check_visible_build_identity_contract() {
  local about='src-tauri/src/desktop_menu.rs'
  local status='src/components/WorkspaceStatusBar.tsx'

  require_file "${status}"
  for marker in \
    '.about(Some(about_metadata()))' \
    'env!("DRAFT_BUILD_COMMIT")' \
    'env!("DRAFT_BUILD_PROFILE")'; do
    if ! rg --quiet --fixed-strings "${marker}" "${about}"; then
      printf 'Missing native About build marker %s\n' "${marker}" >&2
      return 1
    fi
  done
  if ! rg --quiet --fixed-strings 'status.buildCommit.slice(0, 8)' "${status}"; then
    echo 'Bottom status bar must expose the short packaged commit' >&2
    return 1
  fi
}

main "$@"
