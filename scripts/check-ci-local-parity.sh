#!/usr/bin/env bash
set -euo pipefail

# Verifies that GitHub Actions delegates to the same bootstrap and verification
# scripts used locally and does not add privileged or publishing behavior.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

readonly WORKFLOW_PATH=".github/workflows/verify.yml"

main() {
  cd "$(repository_root)"
  require_tools rg
  require_file "${WORKFLOW_PATH}"

  check_workflow_contract
  check_main_branch_triggers
  check_single_verification_entry_point
  check_lockfile_bootstrap
  check_forbidden_workflow_behavior

  printf 'CI/local parity checks passed.\n'
}

check_workflow_contract() {
  local required_patterns=(
    '^name: Verify$'
    '^on:$'
    '^  push:$'
    '^  pull_request:$'
    '^  contents: read$'
    '^  verify:$'
    'runs-on: ubuntu-24.04'
    'uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0'
    'persist-credentials: false'
    'uses: actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e'
    'node-version: 24'
    'cache: npm'
    'uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1'
    'python-version: "3.12"'
    'uses: dtolnay/rust-toolchain@c0e9df88980754dd93e5833b8dcb1b304c1fe173'
    'components: clippy, rustfmt'
    'build-essential'
    'file'
    'libayatana-appindicator3-dev'
    'librsvg2-dev'
    'libssl-dev'
    'libwebkit2gtk-4.1-dev'
    'libxdo-dev'
    'run: bash scripts/bootstrap.sh'
    'run: bash scripts/verify.sh'
  )
  local pattern

  for pattern in "${required_patterns[@]}"; do
    require_pattern "${pattern}" "${WORKFLOW_PATH}"
  done
}

check_main_branch_triggers() {
  local main_branch_count
  main_branch_count="$(rg --count '^      - main$' "${WORKFLOW_PATH}")"

  if [[ "${main_branch_count}" -ne 2 ]]; then
    echo "Verify workflow must target main for push and pull_request" >&2
    return 1
  fi
}

check_single_verification_entry_point() {
  local verify_command_count
  verify_command_count="$(rg --count 'run: bash scripts/verify.sh' "${WORKFLOW_PATH}")"

  if [[ "${verify_command_count}" -ne 1 ]]; then
    echo "Verify workflow must call scripts/verify.sh exactly once" >&2
    return 1
  fi
}

check_lockfile_bootstrap() {
  require_pattern 'npm ci --ignore-scripts' scripts/bootstrap.sh
  require_pattern 'cargo fetch --locked' scripts/bootstrap.sh
}

check_forbidden_workflow_behavior() {
  assert_no_pattern \
    'continue-on-error|pull_request_target|actions/upload-artifact|\bsecrets\.|\brelease\b|\bdeploy(?:ment)?\b|permissions:[[:space:]]*write' \
    "${WORKFLOW_PATH}"
}

require_pattern() {
  local pattern="$1"
  local file_path="$2"

  if ! rg --quiet --pcre2 "${pattern}" "${file_path}"; then
    echo "Missing required CI pattern in ${file_path}: ${pattern}" >&2
    return 1
  fi
}

assert_no_pattern() {
  local pattern="$1"
  local file_path="$2"
  local matches
  local status

  if matches="$(rg --line-number --pcre2 "${pattern}" "${file_path}")"; then
    printf '%s\n' "${matches}" >&2
    echo "Forbidden CI behavior found in ${file_path}" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "CI policy scan could not run" >&2
    return "${status}"
  fi
}

main "$@"
