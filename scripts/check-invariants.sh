#!/usr/bin/env bash
set -euo pipefail

# Enforces invariant boundaries that exist in the current source tree. Feature
# invariants remain explicit future checks until their owning modules exist.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools rg wc

  check_credential_fields
  check_frontend_boundary
  check_python_boundary
  check_command_errors
  check_rust_network_boundary
  check_bash_runtime_boundary
  report_deferred_feature_invariants

  printf 'Invariant boundary scans passed.\n'
}

check_credential_fields() {
  assert_no_matches "INV-01 credential fields" \
    '\b(?:publisher|institution|scholar|library)_(?:username|password)\b|\bapi_key_for_publisher\b' \
    src src-tauri/src python
}

check_frontend_boundary() {
  assert_no_matches "INV-03 frontend trusted APIs" \
    'fetch\s*\(|\baxios\b|\bXMLHttpRequest\b|\bWebSocket\s*\(|\bEventSource\s*\(|navigator\.sendBeacon\s*\(|\bnode:fs\b|@tauri-apps/plugin-fs|@tauri-apps/plugin-store|\blocalStorage\b' \
    src
}

check_python_boundary() {
  assert_no_matches "INV-10/11 Python network or process APIs" \
    '(?m)^\s*(?:from|import)\s+(?:requests|httpx|urllib|aiohttp|socket|subprocess|keyring|sqlite3)(?:[.\s]|$)|\bos\.system\s*\(|\bshell\s*=\s*True\b' \
    python
}

check_command_errors() {
  assert_no_matches "INV-02 generic Rust command errors" \
    '\banyhow::Error\b|Box\s*<\s*dyn\s+(?:std::)?error::Error|Result\s*<[^;\n]+,\s*(?:String|serde_json::Value)\s*>' \
    src-tauri/src

  check_command_contract_coverage
  report_command_surface
}

check_command_contract_coverage() {
  local command_count

  command_count="$(count_tauri_commands)"
  check_command_registrations "${command_count}"
  check_command_signature_tests "${command_count}"
  check_command_request_tests "${command_count}"
  check_command_response_tests "${command_count}"
  check_command_error_tests "${command_count}"
  printf 'PASS INV-02 typed command contract coverage\n'
}

count_tauri_commands() {
  count_pattern_matches '#\[tauri::command\]' src-tauri/src
}

check_command_registrations() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'commands::[[:alnum:]_]+::[[:alnum:]_]+' src-tauri/src/lib.rs)"
  require_matching_count "registered commands" "${expected_count}" "${actual_count}"
}

check_command_signature_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn command_signature_is_typed' src-tauri/src)"
  require_matching_count "typed command signature tests" "${expected_count}" "${actual_count}"
}

check_command_request_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn request_deserialization_is_stable' src-tauri/src)"
  require_matching_count "command request tests" "${expected_count}" "${actual_count}"
}

check_command_response_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn response_serialization_is_stable' src-tauri/src)"
  require_matching_count "command response tests" "${expected_count}" "${actual_count}"
}

check_command_error_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn error_serialization_is_stable' src-tauri/src)"
  require_matching_count "command error tests" "${expected_count}" "${actual_count}"
}

count_pattern_matches() {
  local pattern="$1"
  shift

  rg --only-matching --no-filename --pcre2 "${pattern}" "$@" | wc -l
}

require_matching_count() {
  local label="$1"
  local expected_count="$2"
  local actual_count="$3"

  if [[ "${actual_count}" -ne "${expected_count}" ]]; then
    printf 'FAILED INV-02 %s: expected %s, found %s\n' \
      "${label}" "${expected_count}" "${actual_count}" >&2
    return 1
  fi
}

check_rust_network_boundary() {
  assert_no_matches "INV-10 ad hoc Rust network clients" \
    'reqwest::Client::new\s*\(|reqwest::get\s*\(|ureq::|hyper::Client' \
    --glob '!network/**' --glob '!src-tauri/src/network/**' src-tauri/src
}

check_bash_runtime_boundary() {
  assert_no_matches "INV-12 Bash product runtime" \
    'Command::new\s*\(\s*"(?:/bin/)?bash"' \
    src-tauri/src
}

assert_no_matches() {
  local label="$1"
  local pattern="$2"
  local matches
  local status
  shift 2

  if matches="$(rg --line-number --pcre2 "${pattern}" "$@")"; then
    printf '%s\n' "${matches}" >&2
    echo "FAILED ${label}" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "FAILED ${label}: scan could not run" >&2
    return "${status}"
  fi

  printf 'PASS %s\n' "${label}"
}

report_command_surface() {
  local status

  if rg --quiet '#\[tauri::command\]' src-tauri/src; then
    printf 'INFO Tauri commands found; generic error scan applied.\n'
    return
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo "Tauri command surface scan could not run" >&2
    return "${status}"
  fi

  printf 'INFO No Tauri commands exist yet; command-specific tests begin in Phase 6.\n'
}

report_deferred_feature_invariants() {
  printf '%s\n' \
    'INFO Citation, job, document-handle, cancellation, import, and atomic-save checks are deferred until their owning phases.'
}

main "$@"
