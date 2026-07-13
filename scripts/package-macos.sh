#!/usr/bin/env bash
set -euo pipefail

# Builds and validates the supported unsigned macOS Apple Silicon application
# package. Signing, notarization, and release publication are separate gates.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

readonly APP_BUNDLE='src-tauri/target/release/bundle/macos/DRAFT.app'
readonly APP_PLIST="${APP_BUNDLE}/Contents/Info.plist"
readonly APP_EXECUTABLE="${APP_BUNDLE}/Contents/MacOS/draft"
readonly APP_ICON="${APP_BUNDLE}/Contents/Resources/icon.icns"
readonly APP_HELPER_ROOT="${APP_BUNDLE}/Contents/Resources/python/draft_helpers"
readonly APP_HELPER_INIT="${APP_HELPER_ROOT}/__init__.py"
readonly APP_HELPER_WORKER="${APP_HELPER_ROOT}/worker.py"
readonly SOURCE_ICON='src-tauri/icons/icon.icns'
readonly SYSTEM_PYTHON='/usr/bin/python3'

main() {
  cd "$(repository_root)"
  require_tools cmp env file npm plutil rg uname
  require_supported_host
  require_file package-lock.json
  require_file src-tauri/Cargo.lock
  run_step "Packaging contract" bash scripts/check-packaging.sh
  clear_previous_bundle
  run_step "Unsigned macOS application package" build_application_bundle
  verify_application_bundle

  printf '\nDRAFT package ready: %s\n' "${APP_BUNDLE}"
}

require_supported_host() {
  local system_name
  local machine_architecture
  system_name="$(uname -s)"
  machine_architecture="$(uname -m)"

  if [[ "${system_name}" != 'Darwin' || "${machine_architecture}" != 'arm64' ]]; then
    printf 'Phase 42 packaging requires macOS Apple Silicon; found %s %s\n' \
      "${system_name}" "${machine_architecture}" >&2
    return 1
  fi
}

clear_previous_bundle() {
  rm -rf -- "${APP_BUNDLE}"
}

build_application_bundle() {
  npm run tauri -- build --bundles app --no-sign
}

verify_application_bundle() {
  require_file "${APP_PLIST}"
  require_file "${APP_EXECUTABLE}"
  require_file "${APP_ICON}"
  require_file "${APP_HELPER_INIT}"
  require_file "${APP_HELPER_WORKER}"
  require_file "${SYSTEM_PYTHON}"
  require_plist_value CFBundleIdentifier com.progentic.draft
  require_plist_value CFBundleExecutable draft
  require_plist_value CFBundleIconFile icon.icns
  require_apple_silicon_executable
  require_embedded_icon_match
  require_embedded_helper_execution
}

require_plist_value() {
  local key="$1"
  local expected="$2"
  local actual
  actual="$(plutil -extract "${key}" raw -o - "${APP_PLIST}")"

  if [[ "${actual}" != "${expected}" ]]; then
    printf 'Unexpected %s: expected %s, found %s\n' \
      "${key}" "${expected}" "${actual}" >&2
    return 1
  fi
}

require_apple_silicon_executable() {
  if ! file "${APP_EXECUTABLE}" | rg --quiet 'Mach-O 64-bit executable arm64'; then
    echo 'Packaged executable is not a native Apple Silicon Mach-O binary' >&2
    return 1
  fi
}

require_embedded_icon_match() {
  if ! cmp -s "${SOURCE_ICON}" "${APP_ICON}"; then
    echo 'Packaged icon does not match the tracked DRAFT icon' >&2
    return 1
  fi
}

require_embedded_helper_execution() {
  local request
  local response
  request='{"protocolVersion":1,"requestId":"00000000-0000-4000-8000-000000000000","helper":"text_analysis","helperVersion":1,"input":{"text":"Word word.","locale":"en-US"}}'
  response="$(printf '%s' "${request}" | env -i TMPDIR=/tmp "${SYSTEM_PYTHON}" -I -B "${APP_HELPER_WORKER}")"

  if ! rg --quiet --fixed-strings '"status":"ok"' <<<"${response}" ||
    ! rg --quiet --fixed-strings '"code":"repeated_word"' <<<"${response}"; then
    echo 'Packaged deterministic text helper did not return its typed result' >&2
    return 1
  fi
}

main "$@"
