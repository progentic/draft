#!/usr/bin/env bash
set -euo pipefail

# Checks repository-root execution, required source visibility, whitespace, and
# generated-file hygiene. It does not inspect unrelated filesystem locations.

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools git grep rg

  check_repository_root
  check_required_sources
  check_generated_files_untracked
  check_source_whitespace
  run_step "Git whitespace errors" git diff --check

  printf 'INFO External filesystem locations are not scanned; verification writes only ignored build output and tool caches.\n'
  printf 'Repository hygiene checks passed.\n'
}

check_repository_root() {
  local expected_root
  local git_root
  expected_root="$(repository_root)"
  git_root="$(git rev-parse --show-toplevel)"

  if [[ "${git_root}" != "${expected_root}" ]]; then
    echo "Verification is not running in the DRAFT repository root" >&2
    return 1
  fi
}

check_required_sources() {
  local required_paths=(
    .github/workflows/verify.yml
    index.html
    package-lock.json
    src-tauri/Cargo.lock
    src-tauri/icons/icon.png
  )
  local file_path

  for file_path in "${required_paths[@]}"; do
    ensure_trackable "${file_path}"
  done
}

ensure_trackable() {
  local file_path="$1"

  require_file "${file_path}"
  if ! git ls-files --cached --others --exclude-standard -- "${file_path}" | grep -Fqx "${file_path}"; then
    echo "Required source is ignored or outside Git visibility: ${file_path}" >&2
    return 1
  fi
}

check_generated_files_untracked() {
  local tracked_generated
  tracked_generated="$(git ls-files -- \
    '.DS_Store' '**/.DS_Store' '.tmp/**' 'dist/**' 'node_modules/**' \
    'src-tauri/gen/**' 'src-tauri/target/**' '*.tsbuildinfo' \
    '**/__pycache__/**' '**/*.pyc')"

  if [[ -n "${tracked_generated}" ]]; then
    printf '%s\n' "${tracked_generated}" >&2
    echo "Generated or machine-local files are tracked" >&2
    return 1
  fi
}

check_source_whitespace() {
  local whitespace_matches
  local status

  if whitespace_matches="$(rg --line-number '[[:blank:]]+$' \
    --glob '*.{css,json,py,rs,sh,toml,ts,tsx,yaml,yml}' .)"; then
    printf '%s\n' "${whitespace_matches}" >&2
    echo "Source files contain trailing whitespace" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "Source whitespace scan could not run" >&2
    return "${status}"
  fi
}

main "$@"
