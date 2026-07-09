#!/usr/bin/env bash
set -euo pipefail

repository_root() {
  local common_directory
  common_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  cd -- "${common_directory}/../.." && pwd
}

require_tools() {
  local tool_name

  for tool_name in "$@"; do
    if ! command -v "${tool_name}" >/dev/null 2>&1; then
      echo "Missing required tool: ${tool_name}" >&2
      return 1
    fi
  done
}

require_file() {
  local file_path="$1"

  if [[ ! -f "${file_path}" ]]; then
    echo "Missing required file: ${file_path}" >&2
    return 1
  fi
}

run_step() {
  local label="$1"
  local status
  shift

  printf '\n==> %s\n' "${label}"
  if "$@"; then
    return
  else
    status=$?
  fi

  report_step_failure "${label}" "${status}"
  return "${status}"
}

report_step_failure() {
  local label="$1"
  local status="$2"

  printf 'FAILED %s (exit %s)\n' "${label}" "${status}" >&2
  if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
    printf '::error title=Verification failed::%s failed with exit code %s\n' \
      "${label}" "${status}"
  fi
}

report_skip() {
  local tool_name="$1"
  local reason="$2"

  printf '==> SKIP %s: %s\n' "${tool_name}" "${reason}"
}
