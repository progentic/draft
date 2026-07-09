#!/usr/bin/env bash
set -euo pipefail

# Performs offline documentation sanity checks without validating external URLs.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools find rg sort

  check_required_documents
  report_local_agent_instructions
  check_docs_have_headings
  check_machine_specific_links
  check_changelog_shape
  check_phase_checkpoint

  printf 'INFO External URLs and Markdown anchor targets are not checked.\n'
  printf 'Documentation sanity checks passed.\n'
}

check_required_documents() {
  local required_documents=(
    CHANGELOG.md
    LICENSE
    README.md
    docs/ARCHITECTURE.md
    docs/CODING_STYLE.md
    docs/DOCUMENTATION.md
    docs/drafts/DOCUMENT_ENVELOPE.md
    docs/GOVERNANCE.md
    docs/INVARIANTS.md
    docs/PHASEMAP.md
    docs/ROADMAP.md
    docs/maintainers/CANCELLATION_BOUNDARY.md
    docs/maintainers/COMMAND_BOUNDARY.md
    docs/maintainers/DOCUMENT_ENVELOPE.md
    docs/maintainers/DOCUMENT_REGISTRY.md
    docs/maintainers/DOCUMENT_SAVE_LOAD.md
    docs/maintainers/EVENT_BOUNDARY.md
    docs/maintainers/FRONTEND_COMMAND_CLIENT.md
    docs/maintainers/REALIGNMENT.md
    docs/maintainers/TOOLCHAIN.md
    docs/user/WORKSPACE.md
  )
  local document_path

  for document_path in "${required_documents[@]}"; do
    require_file "${document_path}"
  done
}

report_local_agent_instructions() {
  if [[ -f AGENTS.md ]]; then
    printf 'INFO Local AGENTS.md is present.\n'
    return
  fi

  printf 'INFO Local AGENTS.md is absent; it is intentionally ignored and is not required in a clean checkout.\n'
}

check_docs_have_headings() {
  local document_path
  local first_line

  while IFS= read -r document_path; do
    if ! IFS= read -r first_line <"${document_path}"; then
      echo "Documentation file is empty: ${document_path}" >&2
      return 1
    fi

    if [[ "${first_line}" != \#* ]]; then
      echo "Documentation file needs a top-level heading: ${document_path}" >&2
      return 1
    fi
  done < <(find docs -type f -name "*.md" -print | sort)
}

check_machine_specific_links() {
  local matches
  local status

  if matches="$(rg --line-number --glob '*.md' \
    "file://|/Users/[^/[:space:]]+/|[A-Za-z]:\\\\Users\\\\" .)"; then
    printf '%s\n' "${matches}" >&2
    echo "Documentation contains a machine-specific path" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "Documentation path scan could not run" >&2
    return "${status}"
  fi
}

check_changelog_shape() {
  local status

  if rg --quiet '^## \[Unreleased\]' CHANGELOG.md; then
    echo "CHANGELOG.md must not contain an Unreleased section" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "Changelog shape scan could not run" >&2
    return "${status}"
  fi

  if rg --quiet 'YYYY-MM-DD|github\.com/progentic/changelog' CHANGELOG.md; then
    echo "CHANGELOG.md contains placeholder or foreign-repository release data" >&2
    return 1
  fi
}

check_phase_checkpoint() {
  local checkpoint='Phases 0 through 13 are complete'

  if ! rg --quiet --fixed-strings "${checkpoint}" docs/ROADMAP.md || \
    ! rg --quiet --fixed-strings "${checkpoint}" docs/PHASEMAP.md; then
    echo "ROADMAP.md and PHASEMAP.md must agree on the completed phase checkpoint" >&2
    return 1
  fi
}

main "$@"
