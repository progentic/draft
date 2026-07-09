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
  check_event_contract_coverage
  check_event_capability
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

  check_frontend_ipc_boundary
  check_frontend_event_boundary
}

check_frontend_ipc_boundary() {
  require_file src/ipc/client.ts
  require_file src/ipc/runtimeStatus.ts
  assert_no_matches "INV-03 untyped Tauri IPC outside src/ipc" \
    '@tauri-apps/api/core|\binvoke\s*\(|\binvokeCommand\s*\(' \
    --glob '!src/ipc/**' src
}

check_frontend_event_boundary() {
  require_file src/ipc/eventClient.ts
  require_file src/ipc/runtimeStatusEvents.ts
  assert_no_matches "raw or generic Tauri events outside src/ipc" \
    '@tauri-apps/api/event|\blisten\s*\(|\blistenToEvent\s*\(' \
    --glob '!src/ipc/**' src
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
  require_matching_count "INV-02" "registered commands" "${expected_count}" "${actual_count}"
}

check_command_signature_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn command_signature_is_typed' src-tauri/src)"
  require_matching_count "INV-02" "typed command signature tests" "${expected_count}" "${actual_count}"
}

check_command_request_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn request_deserialization_is_stable' src-tauri/src)"
  require_matching_count "INV-02" "command request tests" "${expected_count}" "${actual_count}"
}

check_command_response_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn response_serialization_is_stable' src-tauri/src)"
  require_matching_count "INV-02" "command response tests" "${expected_count}" "${actual_count}"
}

check_command_error_tests() {
  local expected_count="$1"
  local actual_count
  actual_count="$(count_pattern_matches 'fn error_serialization_is_stable' src-tauri/src)"
  require_matching_count "INV-02" "command error tests" "${expected_count}" "${actual_count}"
}

check_event_contract_coverage() {
  local event_count
  local event_name_test_count
  local event_payload_test_count

  require_file src-tauri/src/events/runtime_status.rs
  event_count="$(count_pattern_matches 'pub\(crate\) enum [[:alnum:]_]+Event' src-tauri/src/events)"
  event_name_test_count="$(count_pattern_matches 'fn event_name_is_stable' src-tauri/src/events)"
  event_payload_test_count="$(count_pattern_matches 'fn event_payload_serialization_is_stable' src-tauri/src/events)"

  require_matching_count "Phase 8" "event name tests" "${event_count}" "${event_name_test_count}"
  require_matching_count "Phase 8" "event payload tests" "${event_count}" "${event_payload_test_count}"
  printf 'PASS typed event contract coverage\n'
}

check_event_capability() {
  local capability_path="src-tauri/capabilities/main.json"

  require_file "${capability_path}"
  require_capability_permission "core:event:allow-listen" "${capability_path}"
  require_capability_permission "core:event:allow-unlisten" "${capability_path}"
  assert_no_matches "frontend event emission permissions" \
    'core:event:(?:default|allow-emit)' "${capability_path}"
  printf 'PASS Phase 8 event listener capability\n'
}

require_capability_permission() {
  local permission="$1"
  local capability_path="$2"

  if ! rg --quiet --fixed-strings "${permission}" "${capability_path}"; then
    printf 'FAILED missing capability permission: %s\n' "${permission}" >&2
    return 1
  fi
}

count_pattern_matches() {
  local pattern="$1"
  shift

  rg --only-matching --no-filename --pcre2 "${pattern}" "$@" | wc -l
}

require_matching_count() {
  local rule="$1"
  local label="$2"
  local expected_count="$3"
  local actual_count="$4"

  if [[ "${actual_count}" -ne "${expected_count}" ]]; then
    printf 'FAILED %s %s: expected %s, found %s\n' \
      "${rule}" "${label}" "${expected_count}" "${actual_count}" >&2
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
