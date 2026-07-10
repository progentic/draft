#!/usr/bin/env bash
set -euo pipefail

# Enforces implemented invariant boundaries and blocks future feature surfaces
# until their owning phases replace absence gates with behavioral checks.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools rg sort wc

  check_credential_fields
  check_frontend_boundary
  check_python_boundary
  check_command_errors
  check_event_contract_coverage
  check_event_capability
  check_worker_cancellation_contract
  check_document_envelope_contract
  check_reference_record_contract
  check_document_registry_contract
  check_document_file_contract
  check_bridge_name_parity
  check_future_feature_absence_gates
  check_rust_network_boundary
  check_bash_runtime_boundary
  report_deferred_behavior_checks

  printf 'Invariant boundary scans passed.\n'
}

check_credential_fields() {
  assert_no_matches "INV-01 credential fields" \
    '\b(?:publisher|institution|scholar|library)_(?:username|password)\b|\bapi_key_for_publisher\b' \
    src src-tauri/src python
}

check_frontend_boundary() {
  assert_no_matches "INV-03 frontend trusted APIs" \
    'fetch\s*\(|\baxios\b|\bXMLHttpRequest\b|\bWebSocket\s*\(|\bEventSource\s*\(|navigator\.sendBeacon\s*\(|\bnode:fs\b|@tauri-apps/plugin-(?:dialog|fs|store)|\blocalStorage\b' \
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

check_worker_cancellation_contract() {
  local required_tests=(
    cancellation_requests_active_worker
    repeated_cancellation_is_idempotent
    cancellation_of_ended_worker_is_idempotent
    cancellation_of_unknown_worker_returns_error
  )
  local test_name

  require_file src-tauri/src/workers/cancellation.rs
  require_file src-tauri/src/commands/worker_cancellation.rs
  require_file src/ipc/workerCancellation.ts
  require_file src/ipc/workerCancellation.test.ts

  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src/commands/worker_cancellation.rs
  done

  assert_no_matches "INV-07 unmanaged Rust worker spawn" \
    '(?:tokio(?:::task)?|tauri::async_runtime)::spawn\s*\(' \
    --glob '!src-tauri/src/workers/**' src-tauri/src
  printf 'PASS INV-07 worker cancellation contract\n'
}

check_document_envelope_contract() {
  local source_path="src-tauri/src/documents/envelope.rs"
  local required_tests=(
    minimal_envelope_deserializes
    envelope_serialization_is_stable
    envelope_round_trip_is_stable
    missing_required_fields_fail_predictably
    non_object_envelope_fails
    unknown_top_level_fields_fail
    unsupported_schema_versions_fail
    malformed_schema_versions_fail
    malformed_document_id_fails
    blank_title_fails
    invalid_document_root_fails
    invalid_document_content_fails
    unicode_and_nested_tiptap_json_round_trip
    envelope_failure_shape_is_stable
  )
  local test_name

  require_file "${source_path}"
  require_envelope_schema_version "${source_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${source_path}"
  done
  assert_no_matches "Phase 11 envelope runtime I/O" \
    '(?:std|tokio)::fs|\b(?:File|OpenOptions|PathBuf)\b|#\[tauri::command\]' \
    "${source_path}"
  printf 'PASS Phase 11 document envelope contract\n'
}

require_envelope_schema_version() {
  local source_path="$1"
  local declaration='pub const DOCUMENT_ENVELOPE_SCHEMA_VERSION: u64 = 1;'

  if ! rg --quiet --fixed-strings "${declaration}" "${source_path}"; then
    printf 'FAILED Phase 11 schema version declaration\n' >&2
    return 1
  fi
}

check_reference_record_contract() {
  local source_path="src-tauri/src/references/record.rs"
  local required_tests=(
    minimal_reference_deserializes
    reference_serialization_is_stable
    reference_round_trip_is_stable
    person_and_organization_contributors_round_trip
    partial_and_absent_issued_dates_round_trip
    unicode_bibliographic_text_round_trip
    supported_reference_kinds_round_trip
    supported_contributor_roles_and_partial_names_round_trip
    supported_resolution_states_round_trip
    supported_provenance_sources_and_overrides_round_trip
    nullable_bibliographic_fields_round_trip
    missing_required_fields_fail_predictably
    non_object_reference_fails
    unknown_top_level_and_nested_fields_fail
    malformed_and_unsupported_schema_versions_fail
    malformed_identity_and_citekey_fail
    unsupported_reference_kinds_fail
    blank_titles_fail
    malformed_contributors_fail
    malformed_issued_dates_fail
    malformed_optional_bibliographic_fields_fail
    malformed_identifiers_fail
    malformed_resolution_and_provenance_fail
    reference_failure_shape_is_stable
  )
  local test_name

  require_file "${source_path}"
  require_reference_schema_version "${source_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${source_path}"
  done
  assert_no_matches "Phase 16 reference runtime authority" \
    '(?:std|tokio)::fs|\b(?:File|OpenOptions|PathBuf|Mutex|RwLock|HashMap)\b|#\[tauri::command\]|\b(?:rusqlite|sqlx|diesel)\b' \
    src-tauri/src/references
  assert_no_matches "Phase 16 frontend reference authority" \
    '\bReferenceRecord\b|\breference_record\b' src
  printf 'PASS Phase 16 reference record contract\n'
}

require_reference_schema_version() {
  local source_path="$1"
  local declaration='pub const REFERENCE_RECORD_SCHEMA_VERSION: u64 = 1;'

  if ! rg --quiet --fixed-strings "${declaration}" "${source_path}"; then
    printf 'FAILED Phase 16 schema version declaration\n' >&2
    return 1
  fi
}

check_document_registry_contract() {
  local source_path="src-tauri/src/documents/registry.rs"
  local required_tests=(
    open_document_twice_returns_already_open
    rejected_duplicate_does_not_replace_live_document
    closing_document_releases_live_handle
    closing_unknown_document_returns_not_open
    distinct_documents_open_independently
    concurrent_open_allows_one_live_handle
    poisoned_registry_returns_unavailable
  )
  local test_name

  require_file "${source_path}"
  require_document_registry_state
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${source_path}"
  done
  assert_no_matches "Phase 12 registry runtime I/O" \
    '(?:std|tokio)::fs|\b(?:File|OpenOptions)\b|#\[tauri::command\]' \
    "${source_path}"
  printf 'PASS INV-06 document registry contract\n'
}

check_document_file_contract() {
  local persistence_path="src-tauri/src/documents/persistence.rs"
  local atomic_writer_path="src-tauri/src/documents/atomic_write.rs"
  local required_tests=(
    document_round_trip_preserves_updated_snapshot
    malformed_json_fails_before_registry_entry
    unsupported_schema_version_fails_explicitly
    duplicate_load_returns_already_open
    save_uses_explicit_snapshot_and_retained_path
    save_new_snapshot_uses_rust_selected_path
    cancelled_first_save_does_not_register_document
    invalid_snapshot_fails_before_path_selection
    save_does_not_reopen_dialog_for_loaded_document
    failed_first_save_does_not_register_document
    failed_attach_preserves_registry_state
    failed_existing_save_preserves_registry_snapshot
    save_rejects_source_path_owned_by_another_document
    durability_failure_advances_registry_to_complete_source
    concurrent_saves_keep_disk_and_registry_consistent
  )
  local test_name

  require_document_file_sources
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${persistence_path}"
  done
  require_rust_test source_path_cannot_back_two_live_handles \
    src-tauri/src/documents/registry.rs
  require_rust_test source_path_conflict_preserves_unattached_handle \
    src-tauri/src/documents/registry.rs
  require_rust_test registry_failure_shape_is_stable \
    src-tauri/src/documents/registry.rs
  require_atomic_writer_tests "${atomic_writer_path}"
  check_document_write_boundary "${atomic_writer_path}"
  check_frontend_document_file_boundary
  printf 'PASS INV-03/06/09 document file contract\n'
}

require_document_file_sources() {
  require_file src-tauri/src/commands/document_open.rs
  require_file src-tauri/src/commands/document_save.rs
  require_file src-tauri/src/documents/atomic_write.rs
  require_file src-tauri/src/documents/dialog.rs
  require_file src-tauri/src/documents/persistence.rs
  require_file src/ipc/documentEnvelope.test.ts
  require_file src/ipc/documentEnvelope.ts
  require_file src/ipc/documentErrors.ts
  require_file src/ipc/documentOpen.ts
  require_file src/ipc/documentOpen.test.ts
  require_file src/ipc/documentSave.ts
  require_file src/ipc/documentSave.test.ts
}

require_atomic_writer_tests() {
  local source_path="$1"
  local required_tests=(
    atomic_writer_creates_complete_document
    platform_replacement_preserves_complete_document
    atomic_writer_rejects_missing_parent
    interrupted_save_preserves_complete_source
    interrupted_save_cleans_temporary_file
    failed_replacement_cleans_temporary_file
    parent_sync_failure_leaves_new_complete_source
    atomic_write_failure_shape_is_stable
  )
  local test_name

  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${source_path}"
  done
}

check_document_write_boundary() {
  local atomic_writer_path="$1"

  assert_no_matches "INV-09 direct document target writes" \
    '\b(?:fs::write|fs::rename|fs::copy|File::create|File::options|OpenOptions::new)\s*\(|\.write_all\s*\(' \
    --glob "!${atomic_writer_path}" src-tauri/src
  require_source_pattern '.tempfile_in(parent)' "${atomic_writer_path}"
  require_source_pattern '.sync_all()' "${atomic_writer_path}"
  require_source_pattern '.persist(target_path)' "${atomic_writer_path}"
  require_source_pattern 'sync_directory(parent)' "${atomic_writer_path}"
}

check_frontend_document_file_boundary() {
  assert_no_matches "frontend document path authority" \
    '\b(?:path|filePath|file_path)\s*:' \
    src/ipc/documentOpen.ts src/ipc/documentSave.ts
}

require_source_pattern() {
  local pattern="$1"
  local source_path="$2"

  if ! rg --quiet --fixed-strings "${pattern}" "${source_path}"; then
    printf 'FAILED missing source pattern %s in %s\n' "${pattern}" "${source_path}" >&2
    return 1
  fi
}

require_document_registry_state() {
  local registration='.manage(documents::registry::DocumentRegistry::new())'

  if ! rg --quiet --fixed-strings "${registration}" src-tauri/src/lib.rs; then
    printf 'FAILED Phase 12 document registry runtime state\n' >&2
    return 1
  fi
}

require_rust_test() {
  local test_name="$1"
  local source_path="$2"

  if ! rg --quiet --fixed-strings "fn ${test_name}" "${source_path}"; then
    printf 'FAILED missing Rust test: %s\n' "${test_name}" >&2
    return 1
  fi
}

check_bridge_name_parity() {
  local rust_commands
  local frontend_commands
  local rust_events
  local frontend_events

  rust_commands="$(extract_values \
    'commands::[[:alnum:]_]+::([[:alnum:]_]+)' src-tauri/src/lib.rs)"
  frontend_commands="$(extract_values \
    'const COMMAND_NAME = "([^"]+)"' --glob '!*.test.ts' src/ipc)"
  rust_events="$(extract_values \
    'const [A-Z_]*EVENT_NAME: &str = "([^"]+)"' src-tauri/src/events)"
  frontend_events="$(extract_values \
    'const EVENT_NAME = "([^"]+)"' --glob '!*.test.ts' src/ipc)"

  require_matching_values "command names" "${rust_commands}" "${frontend_commands}"
  require_matching_values "event names" "${rust_events}" "${frontend_events}"
  require_documented_values "${rust_commands}"
  require_documented_values "${rust_events}"
  printf 'PASS Phase 10 bridge name parity\n'
}

extract_values() {
  local pattern="$1"
  shift

  rg --only-matching --no-filename --replace '$1' "${pattern}" "$@" | sort
}

require_matching_values() {
  local label="$1"
  local rust_values="$2"
  local frontend_values="$3"

  if [[ -z "${rust_values}" || "${rust_values}" != "${frontend_values}" ]]; then
    printf 'FAILED Phase 10 %s parity\nRust:\n%s\nFrontend:\n%s\n' \
      "${label}" "${rust_values}" "${frontend_values}" >&2
    return 1
  fi
}

require_documented_values() {
  local values="$1"
  local value

  while IFS= read -r value; do
    if [[ -n "${value}" ]] && ! rg --quiet --fixed-strings \
      "${value}" docs/maintainers; then
      printf 'FAILED undocumented bridge name: %s\n' "${value}" >&2
      return 1
    fi
  done <<<"${values}"
}

check_future_feature_absence_gates() {
  assert_no_matches "INV-04 citation surface before Phase 18" \
    '\bCitationNode\b|\bcitation_node\b|\binsert_citation\b' src src-tauri/src
  assert_no_matches "INV-05 persistent job surface before Phase 26" \
    '\bBackgroundJob\b|\bPersistentJob\b|\bbackground_job\b|\bjob_state\b' \
    src src-tauri/src
  assert_no_matches "INV-08 watched import before Phase 24" \
    '\bimport_pdf\b|\bwatched_folder\b|\bstable_write\b' src src-tauri/src
  assert_no_matches "Phase 17 reference store before persistence phase" \
    '\bReference(?:Store|Repository)\b|\breference_(?:store|repository|records)\b' \
    src src-tauri/src
  assert_no_matches "INV-11 helper protocol before Phase 28" \
    '(?m)^\s*(?:def|class)\s+(?:run_|analyze|format|[[:alnum:]_]+Request\b|[[:alnum:]_]+Response\b)' \
    python/draft_helpers
  printf 'PASS future feature absence gates\n'
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

  printf 'INFO No Tauri commands exist; command contract checks have no active surface.\n'
}

report_deferred_behavior_checks() {
  printf '%s\n' \
    'INFO Future feature absence gates are active; behavioral checks replace them in each owning phase.'
}

main "$@"
