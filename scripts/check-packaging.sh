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

  printf 'Packaging contract checks passed.\n'
}

require_packaging_sources() {
  local source_path
  local required_sources=(
    package.json
    scripts/package-macos.sh
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
    if (config.bundle?.active !== true) throw new Error("bundle.active must be true");
    if (JSON.stringify(config.bundle.targets) !== JSON.stringify(["app"])) {
      throw new Error("bundle.targets must contain only app");
    }
    if (JSON.stringify(config.bundle.icon) !== JSON.stringify(expectedIcons)) {
      throw new Error("desktop icon paths do not match the package contract");
    }
    if (manifest.scripts?.["package:macos"] !== "bash scripts/package-macos.sh") {
      throw new Error("package:macos must invoke the owned package script");
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
    'CFBundleIdentifier'
    'CFBundleIconFile'
    'Contents/MacOS/draft'
    'Contents/Resources/icon.icns'
  )

  for marker in "${required_markers[@]}"; do
    if ! rg --quiet --fixed-strings "${marker}" "${script_path}"; then
      printf 'Missing packaging marker %s in %s\n' "${marker}" "${script_path}" >&2
      return 1
    fi
  done
}

main "$@"
