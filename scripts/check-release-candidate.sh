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
  check_v1_usability_acceptance

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
  require_literal 'Status: Proposed' \
    docs/adr/002-limit-v1-analysis-to-local-text.md
  require_literal '| RC-03 | Release blocker | Open | Proposed ADR-002 is under review. The local deterministic analysis path' \
    docs/maintainers/RELEASE_CANDIDATE.md
}

check_v1_usability_acceptance() {
  local contract='docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  local ledger='docs/maintainers/V1_USABILITY_EVIDENCE.md'

  require_file "${contract}"
  require_literal 'status: Accepted' "${contract}"
  require_literal '## Supported v1 Workflow' "${contract}"
  require_literal '## First-Time-User Task Validation' "${contract}"
  require_literal '## Measurable Release Thresholds' "${contract}"
  require_literal '## Phase 48 - Secure Usability' "${contract}"
  require_literal '## Phase 49 - Packaged Release-Candidate Gate' "${contract}"
  require_literal '## Phase 50 - Release Entry Point' "${contract}"
  require_gate_usability_row GATE-46
  require_gate_usability_row GATE-47
  require_gate_usability_row GATE-48

  require_usability_evidence_if_closed GATE-46 '## Phase 46' "${ledger}"
  require_usability_evidence_if_closed GATE-47 '## Phase 47' "${ledger}"
  require_usability_evidence_if_closed GATE-48 '## Phase 48' "${ledger}"
  require_candidate_usability_evidence_if_closed "${ledger}"
}

require_gate_usability_row() {
  local gate_id="$1"

  if ! rg --quiet --regexp \
    "^\\| ${gate_id} \\| Must close before Phase 49 \\| (Open|Closed) \\|" \
    docs/maintainers/RELEASE_CANDIDATE.md; then
    printf 'Missing v1 usability gate row: %s\n' "${gate_id}" >&2
    return 1
  fi
}

require_usability_evidence_if_closed() {
  local gate_id="$1"
  local heading="$2"
  local ledger="$3"

  if ! rg --quiet --fixed-strings \
    "| ${gate_id} | Must close before Phase 49 | Closed |" \
    docs/maintainers/RELEASE_CANDIDATE.md; then
    return
  fi

  require_file "${ledger}"
  require_literal "${heading}" "${ledger}"
  require_literal '### Automated Evidence' "${ledger}"
  require_literal '### Findings And Dispositions' "${ledger}"
}

require_candidate_usability_evidence_if_closed() {
  local ledger="$1"

  if ! rg --quiet --fixed-strings '| RC-06 | Release blocker | Closed |' \
    docs/maintainers/RELEASE_CANDIDATE.md; then
    return
  fi

  require_file "${ledger}"
  require_literal '## Phase 49' "${ledger}"
  require_literal '### Packaged Workflow Evidence' "${ledger}"
  if rg --quiet --regexp '^\| UX-[^|]+ \| UX-[01] \| Open \|' "${ledger}"; then
    echo 'Phase 49 cannot close with an open UX-0 or UX-1 finding' >&2
    return 1
  fi
  if rg --quiet --regexp '^\| UX-[^|]+ \| UX-2 \| Open \|' "${ledger}"; then
    echo 'Phase 49 cannot close with an undispositioned UX-2 finding' >&2
    return 1
  fi
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
