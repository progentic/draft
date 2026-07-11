#!/usr/bin/env bash
set -euo pipefail

# Checks the Phase 44 hardening baseline. Passing means findings are classified;
# it does not mean DRAFT is ready for a final release candidate.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools git node rg

  check_inventory_structure
  check_pre_release_state
  check_live_blocker_evidence
  check_generated_release_artifacts
  check_phase45_release_rule

  printf 'INFO Phase 49 remains blocked by RC-01 through RC-06 and GATE-46 through GATE-48.\n'
  printf 'Release-candidate hardening baseline passed.\n'
}

check_inventory_structure() {
  local contract='docs/maintainers/RELEASE_CANDIDATE.md'

  require_file "${contract}"
  require_inventory_group "${contract}" 'RC-' 'Release blocker' 'Open' 6
  require_inventory_group "${contract}" 'GATE-' 'Must close before Phase 49' 'Open' 3
  require_inventory_group "${contract}" 'GATE-' 'Must close before Phase 49' 'Closed' 1
  require_inventory_group "${contract}" 'LIMIT-' 'Accepted v1 limitation' 'Accepted' 5
  require_inventory_group "${contract}" 'MAINT-' 'P2 maintenance backlog' 'Backlog' 3
  require_inventory_group "${contract}" 'POST-' 'Post-v1 work' 'Deferred' 4
}

check_phase45_release_rule() {
  local release_rule='DRAFT is not ready for v1.0.0 unless a user can identify the primary controls'

  require_literal "${release_rule}" docs/ROADMAP.md
  require_literal "${release_rule}" docs/PHASEMAP.md
  require_literal "${release_rule}" docs/maintainers/RELEASE_CANDIDATE.md
  require_literal '| RC-01 | Release blocker | Open |' \
    docs/maintainers/RELEASE_CANDIDATE.md
  require_literal '| GATE-45 | Must close before Phase 49 | Closed |' \
    docs/maintainers/RELEASE_CANDIDATE.md
}

require_inventory_group() {
  local contract="$1"
  local identifier_prefix="$2"
  local classification="$3"
  local status="$4"
  local expected_count="$5"
  local pattern="^\\| ${identifier_prefix}[^ ]* \\| ${classification} \\| ${status} \\| [^|]+ \\| [^|]+ \\| [^|]+ \\| [^|]+ \\|$"
  local actual_count

  actual_count="$(rg --count --regexp "${pattern}" "${contract}")"
  if [[ "${actual_count}" != "${expected_count}" ]]; then
    printf 'Release inventory group %s expected %s rows, found %s\n' \
      "${identifier_prefix}" "${expected_count}" "${actual_count}" >&2
    return 1
  fi
}

check_pre_release_state() {
  node -e '
    const fs = require("node:fs");
    const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
    const tauri = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
    if (pkg.version !== "0.1.0" || tauri.version !== "0.1.0") process.exit(1);
  '
  require_literal 'version = "0.1.0"' src-tauri/Cargo.toml
  require_literal 'No versioned DRAFT release has been published yet.' CHANGELOG.md
  if git rev-parse --quiet --verify refs/tags/v1.0.0 >/dev/null; then
    echo 'v1.0.0 tag exists before the final release gate' >&2
    return 1
  fi
}

check_live_blocker_evidence() {
  node -e '
    const fs = require("node:fs");
    const config = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
    if (config.app?.security?.csp !== null) process.exit(1);
  '
  require_literal '--no-sign' scripts/package-macos.sh
  require_literal 'You cannot create, open, save, close, or reopen a document file' \
    docs/wiki/Current-Limitations.md
  require_literal 'There is no visible reference library.' docs/wiki/Current-Limitations.md
  require_literal 'Automated source analysis is currently unavailable.' \
    docs/wiki/Current-Limitations.md
  require_literal 'You cannot export a DOCX file from the workspace.' \
    docs/wiki/Current-Limitations.md
  require_literal '"csp": null' src-tauri/tauri.conf.json

  if rg --quiet '\b(openDocument|saveDocument|exportDocx|runAiAnalysis|openExternalAccess)\b' \
    --glob '!src/ipc/**' --glob '!*.test.*' src; then
    echo 'A blocker-owned visible workflow changed without RC inventory reassessment' >&2
    return 1
  fi
}

require_literal() {
  local literal="$1"
  local file_path="$2"

  if ! rg --quiet --fixed-strings -- "${literal}" "${file_path}"; then
    printf 'Missing release-candidate evidence: %s: %s\n' "${file_path}" "${literal}" >&2
    return 1
  fi
}

check_generated_release_artifacts() {
  local tracked_artifacts
  tracked_artifacts="$(git ls-files \
    'src-tauri/target/**' 'dist/**' '*.app/**' '*.dmg' '*.pkg' '*.zip')"
  if [[ -n "${tracked_artifacts}" ]]; then
    printf '%s\n' "${tracked_artifacts}" >&2
    echo 'Generated package or release artifacts must remain untracked' >&2
    return 1
  fi
}

main "$@"
