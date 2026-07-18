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
  require_tools git rg sort wc

  check_credential_fields
  check_secret_store_contract
  check_diagnostic_snapshot_contract
  check_error_presentation_contract
  check_frontend_boundary
  check_python_boundary
  check_command_errors
  check_event_contract_coverage
  check_event_capability
  check_worker_cancellation_contract
  check_python_helper_contract
  check_text_analysis_contract
  check_formatting_contract
  check_docx_export_contract
  check_docx_import_contract
  check_phase_47_manual_gate_corrections
  check_document_envelope_contract
  check_reference_record_contract
  check_reference_store_contract
  check_citation_node_contract
  check_bibliography_consistency_contract
  check_network_client_contract
  check_connectivity_contract
  check_metadata_lookup_contract
  check_external_browser_handoff_contract
  check_pdf_import_contract
  check_background_job_contract
  check_ai_orchestration_contract
  check_v1_analysis_decision_guard
  check_adr_003_accepted_guard
  check_adr_004_accepted_guard
  check_document_registry_contract
  check_document_file_contract
  check_critical_path_contract
  check_data_migration_contract
  check_v1_usability_contract
  check_bridge_name_parity
  check_pdf_export_deferral_guard
  check_rust_network_boundary
  check_bash_runtime_boundary
  report_pdf_deferral_status

  printf 'Invariant boundary scans passed.\n'
}

check_v1_usability_contract() {
  local contract='docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  local invariant_id
  local invariant_ids=(
    INV-UX-01
    INV-UX-02
    INV-UX-03
    INV-UX-04
    INV-UX-05
    INV-UX-06
  )

  require_file "${contract}"
  require_source_pattern 'status: Accepted' "${contract}"
  require_source_pattern 'owners: [frontend, core, release]' "${contract}"
  require_source_pattern 'DRAFT v1.0.0 is not releasable unless a first-time user' \
    "${contract}"
  require_source_pattern 'Passing tests proves that DRAFT behaves as implemented.' \
    "${contract}"
  require_source_pattern 'at least five people who have not worked on the' \
    "${contract}"
  require_source_pattern 'At least 80 percent of participants' "${contract}"
  require_source_pattern 'median below 4 creates a Phase 49 finding' "${contract}"
  require_source_pattern "Any open \`UX-0\` or \`UX-1\` blocks Phase 52." "${contract}"
  require_source_pattern 'Accepted ADR-002 authorizes Phase 46 to implement local deterministic text' \
    "${contract}"

  for invariant_id in "${invariant_ids[@]}"; do
    require_source_pattern "| \`${invariant_id}\` | Accepted |" \
      docs/INVARIANTS.md
    require_source_pattern "${invariant_id}" "${contract}"
  done

  printf 'PASS INV-UX-01 through INV-UX-06 v1 usability contract\n'
}

check_credential_fields() {
  assert_no_matches "INV-01 credential fields" \
    '\b(?:publisher|institution|scholar|library)_(?:username|password)\b|\bapi_key_for_publisher\b' \
    src src-tauri/src python
}

check_secret_store_contract() {
  local store_path="src-tauri/src/secrets/store.rs"
  local test_path="src-tauri/src/secrets/store/tests.rs"
  local initializer_path="src-tauri/src/application/secret_store.rs"
  local required_tests=(
    identifiers_accept_only_bounded_normalized_service_names
    secret_values_are_nonempty_bounded_and_not_in_errors
    store_load_replace_and_delete_are_deterministic
    malformed_backend_values_fail_as_invalid_stored_secrets
    backend_failures_map_to_closed_store_errors
    keyring_failures_drop_raw_details_during_mapping
    native_store_is_safe_to_manage_without_accessing_a_credential
  )
  local test_name

  require_file "${store_path}"
  require_file "${test_path}"
  require_file "${initializer_path}"
  require_file docs/drafts/SECRET_STORAGE.md
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'keyring = "4.1.4"' src-tauri/Cargo.toml
  require_source_pattern 'zeroize = "1.9.0"' src-tauri/Cargo.toml
  require_source_pattern 'NATIVE_SERVICE_NAME: &str = "com.progentic.draft"' "${store_path}"
  require_source_pattern 'MAX_INTEGRATION_NAME_BYTES: usize = 64' "${store_path}"
  require_source_pattern 'MAX_SECRET_BYTES: usize = 4_096' "${store_path}"
  require_source_pattern 'Zeroizing<Vec<u8>>' "${store_path}"
  require_source_pattern 'bytes.zeroize();' "${store_path}"
  require_source_pattern '.set_secret(secret.expose_secret())' "${store_path}"
  require_source_pattern '.get_secret()' "${store_path}"
  require_source_pattern 'app.manage(SecretStore::native())' "${initializer_path}"
  require_source_pattern 'initialize_secret_store(app)' src-tauri/src/lib.rs

  assert_no_matches "Phase 37 secret persistence, transport, environment, or logging" \
    '#\[tauri::command\]|\bserde\b|\brusqlite\b|(?:std|tokio)::fs|\breqwest\b|\bNetworkClient\b|\bPythonHelper\b|\bstd::env\b|\benv::|\b(?:println|eprintln|dbg|trace|debug|info|warn|error)!\s*\(' \
    src-tauri/src/secrets
  assert_no_matches "Phase 37 secret value formatting, cloning, or serialization" \
    '(?s)#\[derive\([^\]]*(?:Debug|Clone|Serialize|Deserialize)[^\]]*\)\]\s*pub struct SecretValue|impl\s+(?:(?:fmt|serde)::)?(?:Debug|Display|Clone|Serialize|Deserialize)\s+for\s+SecretValue' \
    "${store_path}"
  assert_no_matches "Phase 37 secret command or event surface" \
    '\bSecret(?:Store|Value|Id|DeleteOutcome|StoreError)\b' \
    src-tauri/src/commands src-tauri/src/events
  assert_no_matches "Phase 37 frontend secret surface" \
    '(?i)\bapi[_-]?key\b|\bpassword\b|\bcredential\b|\bsecret\b' src
  assert_no_matches "Phase 37 Python secret surface" \
    '(?i)\bapi[_-]?key\b|\bpassword\b|\bcredential\b|\bsecret\b|\bkeyring\b' python
  assert_no_matches "Phase 37 ad hoc native credential access" \
    '\bkeyring::' --glob '!src-tauri/src/secrets/store.rs' \
    --glob '!src-tauri/src/secrets/store/tests.rs' \
    src-tauri/src
  assert_no_matches "Phase 37 text-password keyring API" \
    '\.(?:set|get)_password\s*\(' src-tauri/src/secrets
  printf 'PASS INV-01 Phase 37 OS-native secret store contract\n'
}

check_diagnostic_snapshot_contract() {
  local source_path='src-tauri/src/diagnostics.rs'
  local command_path='src-tauri/src/commands/diagnostic_snapshot.rs'
  local client_path='src/ipc/diagnosticSnapshot.ts'
  local client_test_path='src/ipc/diagnosticSnapshot.test.ts'
  local required_tests=(
    snapshot_schema_is_strict_versioned_and_deterministic
    serialized_snapshot_is_bounded
    snapshot_contains_no_redacted_categories
    invalid_application_versions_fail_with_closed_error
  )
  local test_name

  require_file "${source_path}"
  require_file "${command_path}"
  require_file "${client_path}"
  require_file "${client_test_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${source_path}"
  done
  require_source_pattern 'DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION: u16 = 1' "${source_path}"
  require_source_pattern 'MAX_DIAGNOSTIC_SNAPSHOT_BYTES: usize = 2 * 1_024' "${source_path}"
  require_source_pattern 'commands::diagnostic_snapshot::get_diagnostic_snapshot' \
    src-tauri/src/lib.rs
  require_source_pattern 'const COMMAND_NAME = "get_diagnostic_snapshot"' "${client_path}"

  assert_no_matches "Phase 38 diagnostics runtime authority" \
    '(?:std|tokio)::fs|\brusqlite\b|\breqwest\b|\bkeyring\b|\bSecretStore\b|\bPythonHelperRunner\b|\bAppHandle\b|\bState\s*<|\bstd::process\b|\bCommand::|\b(?:thread|tokio)::spawn\b|\b(?:println|eprintln|dbg|trace|debug|info|warn|error)!\s*\(' \
    "${source_path}" "${command_path}"
  assert_no_matches "Phase 38 diagnostics sensitive fields" \
    '(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:document_(?:title|text)|evidence|prompt|finding|secret|credential|account|path|url|request_body|response_body|environment|username|hostname|process_id|logs?)\s*:' \
    "${source_path}" "${command_path}"
  assert_no_matches "Phase 38 secret-store probing" \
    '\b(?:SecretStore|SecretId|SecretValue|NativeSecretBackend)\b|\.get_secret\s*\(|\.load\s*\(' \
    "${source_path}" "${command_path}"
  assert_no_matches "Phase 38 visible diagnostics workflow" \
    '\b(?:getDiagnosticSnapshot|DiagnosticSnapshot)\b' \
    --glob '!src/ipc/diagnosticSnapshot.ts' \
    --glob '!src/ipc/diagnosticSnapshot.test.ts' src
  printf 'PASS INV-01/02/03 Phase 38 local diagnostic snapshot contract\n'
}

check_error_presentation_contract() {
  local policy_path='src/features/error-ux/errorPresentation.ts'
  local test_path='src/features/error-ux/errorPresentation.test.ts'

  require_file "${policy_path}"
  require_file "${test_path}"
  require_file docs/maintainers/ERROR_UX.md
  require_source_pattern 'FailureDisposition = "retryable" | "actionable" | "terminal"' "${policy_path}"
  require_source_pattern 'satisfies Record<RuntimeCommandFailureCode, FailurePresentation>' "${policy_path}"
  require_source_pattern 'satisfies Record<ConnectivityCommandFailureCode, { read: string; change: string }>' "${policy_path}"
  require_source_pattern 'satisfies Record<FormattingReviewCommandErrorCode, FailurePresentation>' "${policy_path}"
  require_source_pattern 'satisfies Record<CitationNodeError["code"], FailurePresentation>' "${policy_path}"
  require_source_pattern 'satisfies Record<CitationStoreError["code"], FailurePresentation>' "${policy_path}"
  require_source_pattern 'return assertNever' "${policy_path}"
  require_source_pattern 'maps runtime command code' "${test_path}"
  require_source_pattern 'maps connectivity read failure' "${test_path}"
  require_source_pattern 'maps connectivity change failure' "${test_path}"
  require_source_pattern 'maps formatting command code' "${test_path}"
  require_source_pattern 'maps citation node error' "${test_path}"
  require_source_pattern 'maps citation client failure' "${test_path}"
  require_source_pattern 'unknown fallbacks distinct' "${test_path}"

  assert_no_matches "Phase 39 unwired error-domain presentation" \
    '\b(?:DocumentOpen|DocumentSave|WorkerCancellation|DiagnosticSnapshot|SecretStore|ExternalAccess|MetadataLookup|PdfImport|DocxExport|TextAnalysis|AiAnalysis)\b' \
    "${policy_path}"
  assert_no_matches "Phase 39 frontend authority or persistence" \
    '@tauri-apps|\binvoke\s*\(|\bfetch\s*\(|\blocalStorage\b|\bsessionStorage\b|\bindexedDB\b|\bwindow\.open\s*\(' \
    "${policy_path}"
  assert_no_matches "Phase 39 raw failure detail access" \
    '\.(?:path|secret|credential|providerPayload|payload|stack|logs?)\b' \
    "${policy_path}"
  assert_no_matches "Phase 39 unwired visible error consumer" \
    'errorPresentation' \
    --glob '!src/features/error-ux/errorPresentation.ts' \
    --glob '!src/features/error-ux/errorPresentation.test.ts' \
    --glob '!src/components/WorkspaceStatusBar.tsx' \
    --glob '!src/features/connectivity/ConnectivityModeControl.tsx' \
    --glob '!src/features/formatting-review/FormattingReviewPanel.tsx' \
    --glob '!src/editor/CitationNode.ts' src
  printf 'PASS INV-02/03 Phase 39 visible error presentation contract\n'
}

check_frontend_boundary() {
  assert_no_matches "INV-03 frontend trusted APIs" \
    'fetch\s*\(|\baxios\b|\bXMLHttpRequest\b|\bWebSocket\s*\(|\bEventSource\s*\(|navigator\.sendBeacon\s*\(|\bwindow\.open\s*\(|target\s*=\s*[\x22\x27]_blank|\bnode:fs\b|@tauri-apps/plugin-(?:dialog|fs|store|opener)|\blocalStorage\b' \
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
  # The two owned DOCX parser files return typed errors around
  # HashMap<String, String>; the nested comma otherwise matches the flat
  # Result<_, String> detector. The DOCX contract below verifies their exact
  # typed boundary, so no broader interoperability path is excluded.
  assert_no_matches "INV-02 generic Rust command errors" \
    '\banyhow::Error\b|Box\s*<\s*dyn\s+(?:std::)?error::Error|Result\s*<[^;\n]+,\s*(?:String|serde_json::Value)\s*>' \
    --glob '!src-tauri/src/interoperability/docx_import/document.rs' \
    --glob '!src-tauri/src/interoperability/docx_import/package.rs' \
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

check_python_helper_contract() {
  local protocol_path="src-tauri/src/workers/python/protocol.rs"
  local protocol_test_path="src-tauri/src/workers/python/protocol_tests.rs"
  local runner_path="src-tauri/src/workers/python/runner.rs"
  local runner_test_path="src-tauri/src/workers/python/runner_tests.rs"
  local worker_path="python/draft_helpers/worker.py"
  local python_test_path="python/tests/test_worker.py"
  local required_rust_tests=(
    contract_probe_input_is_bounded_before_request_creation
    request_serialization_is_stable_and_allowlisted
    matching_success_response_is_validated
    unknown_or_malformed_success_output_fails_closed
    response_identity_and_versions_must_match_request
    impossible_contract_probe_result_fails_validation
    helper_failure_response_is_typed_and_strict
    request_errors_do_not_include_input_content
    runtime_configuration_requires_canonical_trusted_files
    isolated_worker_round_trip_is_typed_and_unicode_safe
    helper_environment_is_cleared_before_execution
    helper_timeout_kills_and_reaps_child
    helper_cancellation_kills_and_reaps_child
    malformed_excessive_and_stderr_output_fail_closed
    nonzero_helper_failure_maps_to_closed_code
    cancellation_before_spawn_avoids_process_work
    runner_errors_do_not_expose_payload_stderr_or_paths
  )
  local required_python_tests=(
    test_package_exports_typed_protocol
    test_valid_request_returns_stable_typed_response
    test_invalid_json_and_unknown_fields_fail_typed
    test_protocol_and_helper_allowlist_fail_closed
    test_request_identity_and_input_bounds_fail_closed
    test_oversized_serialized_request_fails_closed
  )
  local test_name
  local command_path
  local command_scan_paths=()

  require_file "${protocol_path}"
  require_file "${protocol_test_path}"
  require_file "${runner_path}"
  require_file "${runner_test_path}"
  require_file "${worker_path}"
  require_file "${python_test_path}"
  require_file docs/drafts/PYTHON_HELPERS.md
  require_file docs/maintainers/PYTHON_HELPERS.md
  for test_name in "${required_rust_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src/workers/python
  done
  for test_name in "${required_python_tests[@]}"; do
    require_source_pattern "def ${test_name}" "${python_test_path}"
  done
  require_source_pattern 'PYTHON_HELPER_PROTOCOL_VERSION: u16 = 1' "${protocol_path}"
  require_source_pattern 'CONTRACT_PROBE_VERSION: u16 = 1' "${protocol_path}"
  require_source_pattern 'MAX_CONTRACT_PROBE_TEXT_BYTES: usize = 32 * 1024' "${protocol_path}"
  require_source_pattern 'MAX_PYTHON_HELPER_REQUEST_BYTES: usize = 64 * 1024' "${protocol_path}"
  require_source_pattern '#[serde(deny_unknown_fields, rename_all = "camelCase")]' "${protocol_path}"
  require_source_pattern 'MAX_PYTHON_HELPER_STDOUT_BYTES: usize = 64 * 1024' "${runner_path}"
  require_source_pattern 'MAX_PYTHON_HELPER_STDERR_BYTES: usize = 16 * 1024' "${runner_path}"
  require_source_pattern 'PYTHON_HELPER_TIMEOUT: Duration = Duration::from_secs(5)' "${runner_path}"
  require_source_pattern '.arg("-I")' "${runner_path}"
  require_source_pattern '.arg("-B")' "${runner_path}"
  require_source_pattern '.env_clear()' "${runner_path}"
  require_source_pattern '.kill_on_drop(true)' "${runner_path}"
  require_source_pattern 'cancellation.cancelled()' "${runner_path}"
  require_source_pattern 'child.start_kill()' "${runner_path}"
  require_source_pattern 'child.wait().await' "${runner_path}"
  require_source_pattern 'PROTOCOL_VERSION = 1' "${worker_path}"
  require_source_pattern 'CONTRACT_PROBE_HELPER = "contract_probe"' "${worker_path}"
  require_source_pattern 'MAX_REQUEST_BYTES = 64 * 1024' "${worker_path}"
  require_source_pattern 'dependencies = []' pyproject.toml
  require_source_pattern 'tokio = { version = "1.52.3", features = ["io-util", "macros", "process", "sync", "time"] }' src-tauri/Cargo.toml
  # The hostile worker fixture deliberately sleeps, reads PATH, writes stderr,
  # and overproduces output. Production authority scans cover only the package.
  assert_no_matches "INV-11 production Python authority" \
    '(?m)^\s*(?:from|import)\s+(?:aiohttp|glob|httpx|keyring|os|pathlib|requests|shutil|socket|sqlite3|subprocess|tempfile|urllib)(?:[.\s]|$)|\bopen\s*\(|\bos\.environ\b|\bos\.getenv\s*\(|\bos\.system\s*\(' \
    python/draft_helpers
  assert_no_matches "INV-11 Python credential fields" \
    '\b(?:api[_-]?key|authorization|bearer|credential|password|secret|token)\b' \
    python/draft_helpers "${protocol_path}"
  assert_no_matches "Phase 28 helper persistence, network, or mutation authority" \
    '\breqwest\b|\bNetworkClient\b|\brusqlite\b|\bReferenceStore\b|\bDocumentRegistry\b|(?:std|tokio)::fs|\bOpenOptions\b|\bFile::create\b' \
    "${protocol_path}" "${runner_path}"
  assert_no_matches "Phase 28 helper Tauri or detached-task authority" \
    '#\[tauri::command\]|\btauri::|(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(' \
    "${protocol_path}" "${runner_path}"
  # Phase 46 owns the sole command-level runner adapter. Every other command
  # and the application layer remain prohibited from constructing helpers.
  for command_path in src-tauri/src/commands/*.rs; do
    if [[ "${command_path}" != 'src-tauri/src/commands/text_analysis.rs' ]]; then
      command_scan_paths[${#command_scan_paths[@]}]="${command_path}"
    fi
  done
  assert_no_matches "Phase 28 application or unowned command helper authority" \
    '\bPythonHelperRunner\b|\bcontract_probe\b|draft_helpers/worker\.py' \
    src-tauri/src/application "${command_scan_paths[@]}"
  require_source_pattern 'PythonHelperRunner::new(executable, package_root)' \
    src-tauri/src/commands/text_analysis.rs
  assert_no_matches "Phase 28 frontend helper authority" \
    '\bPythonHelper\b|\bcontract_probe\b|\bContractProbe\b' src
  printf 'PASS INV-11 Phase 28 Python helper contract\n'
}

check_text_analysis_contract() {
  local protocol_path="src-tauri/src/workers/python/protocol.rs"
  local model_path="src-tauri/src/workers/python/text_analysis.rs"
  local model_test_path="src-tauri/src/workers/python/text_analysis_tests.rs"
  local runner_test_path="src-tauri/src/workers/python/runner_tests.rs"
  local worker_path="python/draft_helpers/worker.py"
  local python_test_path="python/tests/test_worker.py"
  local required_rust_tests=(
    text_analysis_request_is_versioned_and_allowlisted
    text_analysis_success_shape_is_strict_and_typed
    unknown_text_analysis_code_fails_closed
    finding_codes_map_to_fixed_explainable_policies
    unicode_ranges_must_use_utf8_character_boundaries
    empty_reversed_and_out_of_bounds_ranges_fail_closed
    duplicate_and_unsorted_findings_fail_closed
    equal_ranges_follow_lexical_wire_code_order
    excessive_finding_count_fails_closed
    finding_model_has_no_replacement_or_source_text_field
    text_analysis_round_trip_returns_explainable_non_destructive_findings
    overlapping_text_analysis_codes_keep_deterministic_wire_order
  )
  local required_python_tests=(
    test_text_analysis_returns_all_review_categories
    test_text_analysis_thresholds_are_explicit
    test_text_analysis_offsets_use_utf8_bytes
    test_text_analysis_is_deterministic_sorted_and_bounded
    test_text_analysis_false_positive_guards
  )
  local test_name
  local command_path
  local frontend_path
  local command_scan_paths=()
  local frontend_scan_paths=()

  require_file "${protocol_path}"
  require_file "${model_path}"
  require_file "${model_test_path}"
  require_file "${runner_test_path}"
  require_file "${worker_path}"
  require_file "${python_test_path}"
  require_file docs/drafts/TEXT_ANALYSIS.md
  require_file docs/maintainers/TEXT_ANALYSIS.md
  require_file src-tauri/src/commands/text_analysis.rs
  require_file src/ipc/textAnalysis.ts
  require_file src/features/text-analysis/TextAnalysisPanel.tsx
  require_file src/features/text-analysis/textAnalysisSnapshot.ts
  for test_name in "${required_rust_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src/workers/python
  done
  for test_name in "${required_python_tests[@]}"; do
    require_source_pattern "def ${test_name}" "${python_test_path}"
  done
  require_source_pattern 'TEXT_ANALYSIS_VERSION: u16 = 1' "${protocol_path}"
  require_source_pattern 'MAX_TEXT_ANALYSIS_TEXT_BYTES: usize = 32 * 1024' "${protocol_path}"
  require_source_pattern 'TextAnalysis,' "${protocol_path}"
  require_source_pattern 'MAX_TEXT_ANALYSIS_FINDINGS: usize = 100' "${model_path}"
  require_source_pattern 'text.is_char_boundary(start_byte)' "${model_path}"
  require_source_pattern 'text.is_char_boundary(end_byte)' "${model_path}"
  require_source_pattern 'previous >= current' "${model_path}"
  require_source_pattern 'title: "Repeated word"' "${model_path}"
  require_source_pattern 'title: "Long sentence"' "${model_path}"
  require_source_pattern 'title: "Extended capital emphasis"' "${model_path}"
  require_source_pattern 'title: "Repeated sentence opening"' "${model_path}"
  require_source_pattern 'title: "First-person perspective shift"' "${model_path}"
  require_source_pattern 'TEXT_ANALYSIS_HELPER = "text_analysis"' "${worker_path}"
  require_source_pattern 'TEXT_ANALYSIS_VERSION = 1' "${worker_path}"
  require_source_pattern 'MAX_FINDINGS = 100' "${worker_path}"
  require_source_pattern 'MAX_FINDINGS_PER_CHECK = 20' "${worker_path}"
  require_source_pattern 'LONG_SENTENCE_WORDS = 30' "${worker_path}"
  require_source_pattern 'MIN_ALL_CAPS_LETTERS = 5' "${worker_path}"
  require_source_pattern 'MIN_REPEATED_OPENER_LETTERS = 4' "${worker_path}"
  require_source_pattern 'def _repeated_word_findings' "${worker_path}"
  require_source_pattern 'def _long_sentence_findings' "${worker_path}"
  require_source_pattern 'def _all_caps_findings' "${worker_path}"
  require_source_pattern 'def _repeated_opener_findings' "${worker_path}"
  require_source_pattern 'def _mixed_first_person_findings' "${worker_path}"
  assert_no_matches "INV-15 replacement, scoring, or apply authority" \
    '\b(?:replacement|suggestion|apply|quality_score|readability_score)\b' \
    "${protocol_path}" "${model_path}" "${worker_path}"
  assert_no_matches "INV-15 text-analysis persistence or mutation authority" \
    '\brusqlite\b|\bReferenceStore\b|\bDocumentRegistry\b|\bDocumentEnvelope\b|(?:std|tokio)::fs|\bOpenOptions\b|\bFile::create\b' \
    "${protocol_path}" "${model_path}" "${worker_path}"
  assert_no_matches "Phase 29 Tauri, network, or detached-task authority" \
    '#\[tauri::command\]|\btauri::|\breqwest\b|\bNetworkClient\b|(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(' \
    "${protocol_path}" "${model_path}" "${worker_path}"
  for command_path in src-tauri/src/commands/*.rs; do
    case "${command_path}" in
      src-tauri/src/commands/mod.rs|src-tauri/src/commands/text_analysis.rs) ;;
      *) command_scan_paths[${#command_scan_paths[@]}]="${command_path}" ;;
    esac
  done
  while IFS= read -r frontend_path; do
    case "${frontend_path}" in
      src/ipc/textAnalysis.ts|src/ipc/textAnalysis.test.ts|src/ipc/Phase46Workflows.test.tsx|src/features/text-analysis/TextAnalysisPanel.tsx|src/features/text-analysis/textAnalysisSnapshot.ts|src/features/text-analysis/textAnalysisSnapshot.test.ts) ;;
      *) frontend_scan_paths[${#frontend_scan_paths[@]}]="${frontend_path}" ;;
    esac
  done < <(rg --files src)
  assert_no_matches "Phase 46 unowned command text-analysis authority" \
    '\bTextAnalysis\b|\btext_analysis\b' src-tauri/src/application "${command_scan_paths[@]}"
  assert_no_matches "Phase 46 unowned frontend text-analysis authority" \
    '\bTextAnalysis\b|\btext_analysis\b|\bRepeatedWord\b' "${frontend_scan_paths[@]}"
  require_source_pattern 'export const FINDING_POLICIES = {' src/ipc/textAnalysis.ts
  require_source_pattern 'repeated_word:' src/ipc/textAnalysis.ts
  require_source_pattern 'long_sentence:' src/ipc/textAnalysis.ts
  require_source_pattern 'all_caps_emphasis:' src/ipc/textAnalysis.ts
  require_source_pattern 'repeated_sentence_opener:' src/ipc/textAnalysis.ts
  require_source_pattern 'mixed_first_person:' src/ipc/textAnalysis.ts
  printf 'PASS INV-15 Phase 46 visible review-only text-analysis contract\n'
}

check_formatting_contract() {
  local source_path="src-tauri/src/formatting/checks.rs"
  local test_path="src-tauri/src/formatting/checks_tests.rs"
  local review_path="src-tauri/src/formatting/review.rs"
  local review_test_path="src-tauri/src/formatting/review_tests.rs"
  local command_path="src-tauri/src/commands/formatting_review.rs"
  local required_tests=(
    style_identifiers_are_stable_and_closed
    matching_style_and_valid_outline_are_consistent
    first_heading_below_level_one_is_reviewable
    skipped_heading_levels_are_reported_in_source_order
    siblings_and_ancestor_transitions_do_not_create_findings
    citation_style_mismatches_are_reported_for_every_selected_style
    heading_and_citation_findings_have_deterministic_target_order
    heading_validation_enforces_level_title_and_utf8_byte_bounds
    citekey_validation_reuses_the_reference_domain_rule
    snapshot_collection_bounds_fail_before_checks_run
    finding_policy_is_fixed_review_only_and_content_free
    input_errors_are_bounded_and_do_not_include_rejected_content
  )
  local test_name

  require_file "${source_path}"
  require_file "${test_path}"
  require_file "${review_path}"
  require_file "${review_test_path}"
  require_file "${command_path}"
  require_file src/ipc/formattingReview.ts
  require_file src/ipc/formattingReview.test.ts
  require_file src/features/formatting-review/FormattingReviewPanel.tsx
  require_file src/features/formatting-review/FormattingReviewPanel.test.tsx
  require_file src/features/formatting-review/formattingSnapshot.test.ts
  require_file src/features/formatting-review/useFormattingReview.test.tsx
  require_file docs/drafts/FORMATTING_CHECKS.md
  require_file docs/maintainers/FORMATTING_CHECKS.md
  require_file docs/maintainers/FORMATTING_UX.md
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_formatting_contract_markers "${source_path}"
  check_formatting_authority "${source_path}"
  check_formatting_review_contract "${review_path}" "${command_path}"
  check_text_format_contract
  printf 'PASS INV-16 Phase 34 review-only formatting contract\n'
}

check_text_format_contract() {
  local rust_path='src-tauri/src/documents/text_format.rs'
  local frontend_path='src/documents/textFormatting.ts'
  local mark_path='src/editor/TextFormattingMarks.ts'
  local control_state_path='src/editor/fontControlState.ts'
  local control_test_path='src/editor/fontControlState.test.ts'
  local rust_tests=(
    supported_family_and_size_marks_validate
    canonical_font_allowlist_maps_every_identifier_to_docx
    unsupported_font_values_fail_with_bounded_paths
    fractional_zero_negative_and_malformed_sizes_fail
    malformed_mark_shapes_fail_before_persistence
    strict_font_mark_fields_and_attrs_fail_closed
  )
  local test_name

  require_file "${rust_path}"
  require_file "${frontend_path}"
  require_file src/documents/textFormatting.test.ts
  require_file "${mark_path}"
  require_file src/editor/TextFormattingMarks.test.ts
  require_file "${control_state_path}"
  require_file "${control_test_path}"
  for test_name in "${rust_tests[@]}"; do
    require_rust_test "${test_name}" "${rust_path}"
  done
  require_source_pattern 'MIN_FONT_SIZE_POINTS: u64 = 8' "${rust_path}"
  require_source_pattern 'MAX_FONT_SIZE_POINTS: u64 = 72' "${rust_path}"
  check_font_identifier_parity "${rust_path}" "${frontend_path}"
  require_source_pattern 'export const MIN_FONT_SIZE_POINTS = 8' "${frontend_path}"
  require_source_pattern 'export const MAX_FONT_SIZE_POINTS = 72' "${frontend_path}"
  require_source_pattern 'typeof value.type !== "string"' "${frontend_path}"
  require_source_pattern 'span[data-draft-font-family]' "${mark_path}"
  require_source_pattern 'span[data-draft-font-size]' "${mark_path}"
  require_source_pattern 'FONT_FAMILIES.map((family)' src/editor/EditorToolbar.tsx
  require_source_pattern 'DOCUMENT_FONT_FAMILY: FontFamilyId = "georgia"' \
    "${control_state_path}"
  require_source_pattern 'DOCUMENT_FONT_SIZE_POINTS = 13' "${control_state_path}"
  require_source_pattern 'MIXED_FONT_VALUE = "__mixed__"' "${control_state_path}"
  require_source_pattern 'reports document and heading defaults instead of empty placeholders' \
    "${control_test_path}"
  require_source_pattern 'follows explicit formatting at the caret after a JSON round trip' \
    "${control_test_path}"
  require_source_pattern 'reports mixed family and size for a heterogeneous range' \
    "${control_test_path}"
  assert_no_matches 'empty font-control placeholders' \
    'Default font|Default size' \
    src/editor/EditorToolbar.tsx
  require_rust_test every_supported_font_family_emits_explicit_docx_run_properties \
    src-tauri/src/exports/docx_tests.rs
  require_source_pattern 'ignores pasted font CSS and accepts only canonical DRAFT attributes' \
    src/editor/TextFormattingMarks.test.ts
  assert_no_matches "arbitrary pasted font style import" \
    "parseHTML:.*(?:font-family|font-size)|getAttribute\\([\"']style" \
    "${mark_path}"
  printf 'PASS bounded persistent text-format contract\n'
}

check_font_identifier_parity() {
  local rust_path="$1"
  local frontend_path="$2"
  local expected actual_rust actual_frontend

  expected="$(printf '%s\n' arial avenir_next baskerville courier_new georgia \
    helvetica menlo palatino times_new_roman trebuchet_ms verdana | sort)"
  actual_rust="$(rg --multiline --only-matching --replace '$1' \
    'font\(\s*"([a-z_]+)"' "${rust_path}" | sort)"
  actual_frontend="$(rg --only-matching --replace '$1' \
    '\{ id: "([a-z_]+)"' "${frontend_path}" | sort)"

  if [[ "${actual_rust}" != "${expected}" || "${actual_frontend}" != "${expected}" ]]; then
    printf 'FAILED Rust and TypeScript font identifiers must match the exact 11-family contract\n' >&2
    return 1
  fi
}

require_formatting_contract_markers() {
  local source_path="$1"

  require_source_pattern 'MAX_FORMATTING_HEADINGS: usize = 512' "${source_path}"
  require_source_pattern 'MAX_FORMATTING_CITATIONS: usize = 512' "${source_path}"
  require_source_pattern 'MAX_HEADING_TITLE_BYTES: usize = 512' "${source_path}"
  require_source_pattern 'pub fn run_formatting_checks' "${source_path}"
  require_source_pattern 'Self::Apa7 => "apa7"' "${source_path}"
  require_source_pattern 'Self::Mla9 => "mla9"' "${source_path}"
  require_source_pattern 'Self::Chicago17AuthorDate => "chicago17_author_date"' "${source_path}"
  require_source_pattern 'title: "Outline starts below level 1"' "${source_path}"
  require_source_pattern 'title: "Heading level skipped"' "${source_path}"
  require_source_pattern 'title: "Citation style differs"' "${source_path}"
}

check_formatting_authority() {
  local source_path="$1"

  assert_no_matches "INV-16 formatting mutation or scoring authority" \
    '\b(?:replacement|suggestion|apply|patch|score)\b' "${source_path}"
  assert_no_matches "INV-16 formatting persistence or document authority" \
    '\brusqlite\b|\bReferenceStore\b|\bDocumentRegistry\b|\bDocumentEnvelope\b|\bCitationNodeAttributes\b|(?:std|tokio)::fs|\bOpenOptions\b|\bFile::create\b' \
    "${source_path}"
  assert_no_matches "Phase 31 network, Python, or worker authority" \
    '#\[tauri::command\]|\btauri::|\breqwest\b|\bNetworkClient\b|\bPythonHelper\b|(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(' \
    "${source_path}"
  assert_no_matches "Phase 31 Python formatting authority" \
    '\bFormattingFinding\b|\bFormattingSnapshot\b|\brun_formatting_checks\b' python
}

check_formatting_review_contract() {
  local review_path="$1"
  local command_path="$2"
  local rust_paths=("${review_path}" "${command_path}")
  local frontend_paths=(src/ipc/formattingReview.ts src/features/formatting-review)

  require_source_pattern 'pub fn run_formatting_review' "${review_path}"
  require_source_pattern '#[tauri::command]' "${command_path}"
  require_source_pattern 'ApplyHeadingLevel' "${review_path}"
  require_source_pattern 'generationRef' src/features/formatting-review/useFormattingReview.ts
  require_source_pattern 'isCurrentFormattingTarget' src/features/formatting-review/formattingSnapshot.ts
  require_source_pattern 'ignores an older run after a newer run becomes ready' \
    src/features/formatting-review/useFormattingReview.test.tsx
  require_source_pattern 'rejects a target whose captured position now addresses another node' \
    src/features/formatting-review/formattingSnapshot.test.ts
  require_source_pattern 'requires explicit review actions' \
    src/features/formatting-review/FormattingReviewPanel.test.tsx

  assert_no_matches "INV-16 review persistence, filesystem, export, or PDF authority" \
    '\brusqlite\b|(?:std|tokio)::fs|\bOpenOptions\b|\bFile::create\b|\bexport_docx\b|\bcompile_docx\b|\bPdf\b|\bPDF\b' \
    "${rust_paths[@]}"
  assert_no_matches "INV-16 review network, Python, or worker authority" \
    '\breqwest\b|\bNetworkClient\b|\bPythonHelper\b|(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(' \
    "${rust_paths[@]}"
  assert_no_matches "INV-16 frontend persistence or privileged authority" \
    '\blocalStorage\b|\bfetch\s*\(|\bopenExternalAccess\b|\bexportDocx\b|\bsaveDocument\b' \
    "${frontend_paths[@]}"
}

check_docx_export_contract() {
  local policy_path="src-tauri/src/exports/docx.rs"
  local model_path="src-tauri/src/exports/docx_model.rs"
  local package_path="src-tauri/src/exports/docx_package.rs"
  local test_path="src-tauri/src/exports/docx_tests.rs"
  local required_tests=(
    package_has_stable_safe_entries_and_reopens
    equal_documents_compile_to_equal_bytes
    every_package_xml_part_is_well_formed
    unicode_headings_breaks_and_marks_render_without_raw_markup
    mixed_font_runs_keep_distinct_docx_properties
    empty_paragraphs_and_headings_are_preserved
    unknown_fields_nodes_and_marks_fail_without_silent_omission
    malformed_nested_shapes_and_xml_controls_fail_typed
    citation_nodes_fail_instead_of_exporting_editor_markers
    source_byte_node_and_depth_limits_fail_before_parsing
    compiled_artifact_limit_fails_before_filesystem_work
    target_validation_precedes_compilation_and_write
    uppercase_docx_extension_is_accepted_for_rust_owned_targets
    atomic_export_creates_and_replaces_target_without_changing_source
    compilation_failure_preserves_prior_complete_export
    atomic_write_failures_map_to_closed_export_stages
    post_replacement_sync_failure_is_durability_uncertain
    errors_are_content_free_and_structural_paths_are_bounded
    relationships_contain_no_external_targets_or_active_content
  )
  local test_name

  require_file "${policy_path}"
  require_file "${model_path}"
  require_file "${package_path}"
  require_file "${test_path}"
  require_file docs/drafts/DOCX_EXPORT.md
  require_file docs/maintainers/DOCX_EXPORT.md
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_docx_contract_markers "${policy_path}" "${package_path}"
  check_docx_export_authority "${model_path}" "${package_path}"
  printf 'PASS INV-09 Phase 32 atomic DOCX export contract\n'
}

require_docx_contract_markers() {
  local policy_path="$1"
  local package_path="$2"

  require_source_pattern 'MAX_DOCX_SOURCE_BYTES: usize = 8 * 1024 * 1024' "${policy_path}"
  require_source_pattern 'MAX_DOCX_NODES: usize = 100_000' "${policy_path}"
  require_source_pattern 'MAX_DOCX_NESTING_DEPTH: usize = 16' "${policy_path}"
  require_source_pattern 'MAX_DOCX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024' "${policy_path}"
  require_source_pattern 'pub fn compile_docx' "${policy_path}"
  require_source_pattern 'pub fn export_docx' "${policy_path}"
  require_source_pattern 'write_document_atomically' "${policy_path}"
  require_source_pattern '[Content_Types].xml' "${package_path}"
  require_source_pattern 'word/_rels/document.xml.rels' "${package_path}"
  require_source_pattern 'SimpleFileOptions::DEFAULT' "${package_path}"
  require_source_pattern 'CompressionMethod::Stored' "${package_path}"
  require_source_pattern 'BytesText::new(value)' "${package_path}"
  require_source_pattern 'w:rFonts' "${package_path}"
  require_source_pattern 'w:sz' "${package_path}"
  require_source_pattern 'half_points()' "${package_path}"
  require_source_pattern 'quick-xml = "0.41.0"' src-tauri/Cargo.toml
  require_source_pattern 'zip = { version = "8.6.0", default-features = false, features = ["deflate"] }' src-tauri/Cargo.toml
}

check_docx_export_authority() {
  local model_path="$1"
  local package_path="$2"
  local command_path
  local frontend_path
  local command_scan_paths=()
  local frontend_scan_paths=()

  assert_no_matches "Phase 32 persistence or direct filesystem authority" \
    '\brusqlite\b|\bReferenceStore\b|\bDocumentRegistry\b|(?:std|tokio)::fs|\bOpenOptions\b|\bFile::create\b' \
    src-tauri/src/exports --glob '!docx_tests.rs'
  assert_no_matches "Phase 32 Tauri, network, Python, or worker authority" \
    '#\[tauri::command\]|\btauri::|\breqwest\b|\bNetworkClient\b|\bPythonHelper\b|(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(' \
    src-tauri/src/exports
  assert_no_matches "Phase 32 unsafe package content" \
    '\bTargetMode\b|vbaProject|macroEnabled|\.bin["\x27]' "${package_path}"
  assert_no_matches "Phase 32 manual XML interpolation" \
    'format!\s*\(' "${model_path}" "${package_path}"
  for command_path in src-tauri/src/commands/*.rs; do
    case "${command_path}" in
      src-tauri/src/commands/docx_export.rs|src-tauri/src/commands/mod.rs) ;;
      *) command_scan_paths[${#command_scan_paths[@]}]="${command_path}" ;;
    esac
  done
  while IFS= read -r frontend_path; do
    case "${frontend_path}" in
      # Phase 48 permits the closed action identifier only in its typed event,
      # shared dispatcher, visible command, and colocated tests.
      src/ipc/docxExport.ts|src/ipc/docxExport.test.ts|src/ipc/Phase46Workflows.test.tsx|src/features/docx-export/useDocxExport.ts|src/ipc/nativeMenu.ts|src/ipc/nativeMenu.test.ts|src/components/WorkspaceCommandBar.tsx|src/components/WorkspaceCommandBar.test.tsx|src/features/workspace-actions/useWorkspaceActions.ts|src/features/workspace-actions/useWorkspaceActions.test.tsx) ;;
      *) frontend_scan_paths[${#frontend_scan_paths[@]}]="${frontend_path}" ;;
    esac
  done < <(rg --files src)
  assert_no_matches "Phase 46 unowned application or command export authority" \
    '\bDocxArtifact\b|\bDocxExport\b|\bcompile_docx\b|\bexport_docx\b' \
    src-tauri/src/application "${command_scan_paths[@]}"
  assert_no_matches "Phase 46 unowned frontend DOCX authority" \
    '\bDocxArtifact\b|\bDocxExport\b|\bcompile_docx\b|\bexport_docx\b' \
    "${frontend_scan_paths[@]}"
  require_source_pattern 'export_docx(document, &target)' src-tauri/src/commands/docx_export.rs
  require_source_pattern 'export async function exportDocument' src/ipc/docxExport.ts
  assert_no_matches "Phase 32 Python DOCX authority" \
    '\bDocxArtifact\b|\bDocxExport\b|\bcompile_docx\b|\bexport_docx\b' python
}

check_docx_import_contract() {
  local policy_path='src-tauri/src/interoperability/mod.rs'
  local fidelity_path='src-tauri/src/interoperability/fidelity.rs'
  local provenance_path='src-tauri/src/interoperability/provenance.rs'
  local source_write_path='src-tauri/src/documents/external_save.rs'
  local source_write_tests='src-tauri/src/documents/external_save_tests.rs'
  local source_write_command='src-tauri/src/commands/external_document_save.rs'
  local source_write_client='src/ipc/externalDocumentSave.ts'
  local source_write_client_tests='src/ipc/externalDocumentSave.test.ts'
  local source_write_hook='src/features/external-source-save/useExternalSourceSave.ts'
  local source_write_hook_tests='src/features/external-source-save/useExternalSourceSave.test.tsx'
  local source_write_dialog='src/features/external-source-save/SaveBackToSourceDialog.tsx'
  local source_write_dialog_tests='src/features/external-source-save/SaveBackToSourceDialog.test.tsx'
  local package_path='src-tauri/src/interoperability/docx_import/package.rs'
  local document_path='src-tauri/src/interoperability/docx_import/document.rs'
  local test_path='src-tauri/src/interoperability/docx_import/tests.rs'
  local required_tests=(
    supported_paragraph_properties_map_to_canonical_data
    absent_paragraph_properties_remain_absent
    alternate_heading_name_is_canonically_normalized
    valid_unsupported_properties_require_source_preservation
    supported_run_formatting_survives_unrelated_unsupported_properties
    package_semantics_classify_valid_uneditable_behavior
    optional_relationship_and_style_parts_are_not_required
    exact_and_at_least_line_rules_are_unsupported_not_malformed
    unsupported_style_and_list_indentation_are_distinct_valid_features
    malformed_properties_fail_without_fidelity_guessing
    unrepresentable_bounds_are_lossy_and_never_clamped
    exported_supported_paragraph_data_reimports_exactly
    package_and_xml_safety_fail_with_closed_reasons
    malformed_package_contracts_are_not_reported_as_unsafe_content
    archive_compression_ratio_fails_closed_before_extraction
    package_resource_limits_have_stable_safety_reasons
    package_fixture_is_deterministic_and_hashable
  )
  local test_name

  require_file "${policy_path}"
  require_file "${fidelity_path}"
  require_file "${provenance_path}"
  require_file "${source_write_path}"
  require_file "${source_write_tests}"
  require_file "${source_write_command}"
  require_file "${source_write_client}"
  require_file "${source_write_client_tests}"
  require_file "${source_write_hook}"
  require_file "${source_write_hook_tests}"
  require_file "${source_write_dialog}"
  require_file "${source_write_dialog_tests}"
  require_file "${package_path}"
  require_file "${document_path}"
  require_file "${test_path}"
  require_file src/ipc/externalDocument.ts
  require_file src/ipc/externalDocument.test.ts
  require_file docs/maintainers/DOCX_INTEROPERABILITY.md

  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done

  require_source_pattern 'pub(crate) fn import_docx_source(' "${policy_path}"
  require_rust_test oversized_source_fails_before_package_parsing "${policy_path}"
  require_source_pattern 'pub enum ExternalFidelity' "${fidelity_path}"
  require_source_pattern 'pub enum SameFormatSaveDisposition' "${provenance_path}"
  require_source_pattern 'pub(crate) fn normalization_features(' "${provenance_path}"
  require_source_pattern 'pub(crate) async fn save_external_document(' \
    "${source_write_command}"
  require_source_pattern 'write_document_atomically' "${source_write_path}"
  require_source_pattern 'registry.commit_external_write' "${source_write_path}"
  require_source_pattern 'const COMMAND_NAME = "save_external_document"' \
    "${source_write_client}"
  require_source_pattern 'export async function saveExternalDocument(' \
    "${source_write_client}"
  require_rust_test exact_save_replaces_source_and_refreshes_provenance \
    "${source_write_tests}"
  require_rust_test normalization_requires_acceptance_and_cancel_never_writes \
    "${source_write_tests}"
  require_rust_test source_change_after_compilation_is_denied_before_replacement \
    "${source_write_tests}"
  require_rust_test imported_source_changed_after_confirmation_is_denied_without_mutation \
    "${source_write_tests}"
  require_rust_test imported_normalization_names_the_required_transformation \
    "${source_write_tests}"
  require_rust_test stale_normalized_source_returns_denial_without_obsolete_normalizations \
    "${source_write_tests}"
  require_rust_test pre_replacement_failure_preserves_source_and_registry \
    "${source_write_tests}"
  require_rust_test durability_failure_rolls_back_replacement \
    "${source_write_tests}"
  require_rust_test rollback_failure_reports_uncertain_source_state \
    "${source_write_tests}"
  require_rust_test registry_commit_failure_rolls_back_source \
    "${source_write_tests}"
  require_rust_test eligibility_inspection_never_writes_or_mutates_registry \
    "${source_write_tests}"
  require_rust_test eligibility_rejects_unrepresentable_content_without_writing \
    "${source_write_tests}"
  require_rust_test exact_and_normalized_replacements_open_in_macos_text_reader \
    "${source_write_tests}"
  require_rust_test exact_and_normalized_replacements_render_in_configured_compatible_reader \
    "${source_write_tests}"
  require_source_pattern 'normalizations: ExternalNormalizationFeature[]' \
    "${source_write_client}"
  require_source_pattern "Alternate heading style names will use DRAFT’s standard heading names." \
    "${source_write_dialog}"
  require_source_pattern '            Replace' "${source_write_dialog}"
  require_source_pattern '            Cancel' "${source_write_dialog}"
  require_source_pattern 'MAX_DOCX_IMPORT_PACKAGE_BYTES: usize = 16 * 1024 * 1024' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'MAX_DOCX_IMPORT_XML_BYTES: usize = 8 * 1024 * 1024' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'MAX_DOCX_IMPORT_ENTRIES: usize = 128' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'MAX_DOCX_IMPORT_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'MAX_DOCX_IMPORT_XML_DEPTH: usize = 64' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'MAX_DOCX_IMPORT_COMPRESSION_RATIO: u64 = 100' \
    src-tauri/src/interoperability/docx_import/mod.rs
  require_source_pattern 'c284d54886d21d2fda1d0fa51099ac2db65cbaf830ce133d8f6608c21c4bf35a' \
    "${test_path}"
  require_source_pattern 'accepts only path-free successful import summaries' \
    src/ipc/externalDocument.test.ts
  require_source_pattern 'returns path-free DOCX fidelity from the Rust import boundary' \
    src/ipc/documentOpen.test.ts
  require_source_pattern 'keeps path-free DOCX source identity separate from DRAFT Save' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'discloses unsupported DOCX behavior without exposing source authority' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'prevents stale Open responses by rejecting overlapping actions' \
    src/ipc/Phase46Workflows.test.tsx

  assert_no_matches "DOCX package parser outside its owned adapter" \
    '\bquick_xml\b|\bzip::' \
    "${policy_path}" "${fidelity_path}" "${provenance_path}"
  assert_no_matches "frontend external-document path or package authority" \
    '\bPathBuf\b|\bfilesystem\b|\bquick_xml\b|\bzip::|\brawXml\b|\bsourceBytes\b' \
    src/ipc/externalDocument.ts "${source_write_client}"
  assert_no_matches "frontend external source path or fingerprint input" \
    '\bsourcePath\b|\bsourceFingerprint\b|\btargetPath\b|\babsolutePath\b' \
    "${source_write_client}" "${source_write_client_tests}" \
    "${source_write_hook}" "${source_write_hook_tests}" \
    "${source_write_dialog}" "${source_write_dialog_tests}"
  assert_no_matches "frontend same-format save decision authority" \
    '\bsameFormatSave\b|\ballowed_exact\b|\ballowed_after_accepted_normalization\b|\bdenied_(?:unsupported_source_behavior|read_only|missing_provenance|fidelity_unknown|writer_unavailable|source_missing|source_changed)\b' \
    --glob '!**/*.test.ts' --glob '!**/*.test.tsx' \
    --glob '!src/ipc/externalDocument.ts' \
    --glob '!src/ipc/externalDocumentSave.ts' \
    --glob '!src/features/external-source-save/useExternalSourceSave.ts' \
    --glob '!src/features/external-source-save/SaveBackToSourceDialog.tsx' src
  assert_no_matches "external source-write client outside its owned workflow" \
    'externalDocumentSave|saveExternalDocument' \
    --glob '!src/features/external-source-save/useExternalSourceSave.ts' \
    --glob '!src/features/external-source-save/useExternalSourceSave.test.tsx' \
    src/app src/components src/features src/editor
  require_source_pattern 'saveExternalDocument(snapshot, "inspect")' \
    "${source_write_hook}"
  require_source_pattern 'result.status !== "cancelled"' "${source_write_hook}"
  require_source_pattern 'action="save_back_to_source"' \
    src/components/WorkspaceCommandBar.tsx
  require_source_pattern 'save_back_to_source: session.requestSaveBack' \
    src/features/workspace-actions/useWorkspaceActions.ts
  require_source_pattern '<SaveBackToSourceDialog' src/app/DraftWorkspace.tsx
  require_source_pattern 'inspects and confirms exact replacement without exposing a path' \
    "${source_write_hook_tests}"
  require_source_pattern 'requires explicit acceptance for normalized replacement' \
    "${source_write_hook_tests}"
  require_source_pattern 'cancels through the typed boundary without accepting saved state' \
    "${source_write_hook_tests}"
  require_source_pattern 'blocks stale external sources with bounded recovery' \
    "${source_write_hook_tests}"
  require_source_pattern 'blocks a source changed after confirmation without accepting saved state' \
    "${source_write_hook_tests}"
  require_source_pattern 'preserves modified state and offers bounded retry after a safe write failure' \
    "${source_write_hook_tests}"
  require_source_pattern 'blocks retry when the source final state is uncertain' \
    "${source_write_hook_tests}"
  require_source_pattern 'confirms exact Save Back and preserves the external display identity' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'routes native Save Back through the same exact confirmation workflow' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'cancels normalized Save Back without changing editor state' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'rejects an externally changed source without losing current edits' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'rejects a source changed after confirmation without losing current edits' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'preserves source identity and edits after a replacement write failure' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'gives toolbar and native Save Back the same stale-source disposition' \
    src/features/workspace-actions/useWorkspaceActions.test.tsx
  require_source_pattern 'format: TextImportFormat' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'format: "markdown" | "plain_text"' \
    src/ipc/documentOpen.ts

  require_source_pattern "| \`INV-17\` | Proposed |" docs/INVARIANTS.md
  require_source_pattern '| UX-46-011 | UX-1 | Open |' \
    docs/maintainers/V1_USABILITY_EVIDENCE.md
  require_source_pattern '| UX-46-014 | UX-1 | Open |' \
    docs/maintainers/V1_USABILITY_EVIDENCE.md
  require_source_pattern '| RC-07 | Release blocker | Open |' \
    docs/maintainers/RELEASE_CANDIDATE.md
  require_source_pattern '| GATE-47 | Roadmap gate | Open |' \
    docs/maintainers/RELEASE_CANDIDATE.md
  printf 'PASS Phase 47 bounded DOCX import and fidelity contract\n'
}

check_phase_47_manual_gate_corrections() {
  local runtime='src-tauri/src/application/runtime_status.rs'
  local application_queue='src-tauri/src/application/open_requests.rs'
  local application_command='src-tauri/src/commands/application_open.rs'
  local application_client='src/ipc/applicationOpen.ts'
  local operation_notice='src/components/WorkspaceOperationNotice.tsx'
  local workflow_tests='src/ipc/Phase46Workflows.test.tsx'
  local ledger='docs/maintainers/V1_USABILITY_EVIDENCE.md'

  require_file "${runtime}"
  require_file "${application_queue}"
  require_file "${application_command}"
  require_file "${application_client}"
  require_file "${operation_notice}"
  require_file src-tauri/Info.plist

  require_rust_test packaged_build_identity_requires_a_full_lowercase_commit \
    "${runtime}"
  require_rust_test build_profile_is_closed "${runtime}"
  require_rust_test one_file_url_is_queued_without_exposing_it \
    "${application_queue}"
  require_rust_test multiple_or_non_file_urls_fail_closed \
    "${application_queue}"
  require_rust_test dismissal_consumes_one_request_without_opening_it \
    "${application_command}"

  require_source_pattern 'DRAFT_BUILD_COMMIT' src-tauri/build.rs
  require_source_pattern 'require_clean_worktree' scripts/package-macos.sh
  require_source_pattern 'require_embedded_build_identity' scripts/package-macos.sh
  require_source_pattern 'buildCommit: string' src/ipc/runtimeStatus.ts
  require_source_pattern 'buildProfile: "debug" | "release"' \
    src/ipc/runtimeStatus.ts
  require_source_pattern 'status.buildCommit.slice(0, 8)' \
    src/components/WorkspaceStatusBar.tsx
  require_source_pattern 'about_metadata()' src-tauri/src/desktop_menu.rs
  require_source_pattern 'env!("DRAFT_BUILD_PROFILE")' src-tauri/src/desktop_menu.rs
  assert_no_matches 'build identity stays out of document inspector' \
    '\b(?:RuntimeConnectionState|runtimeStatus|buildCommit|buildProfile|Core v)\b' \
    src/components/DocumentInspector.tsx

  require_source_pattern 'const COMMAND_NAME = "open_application_document"' \
    "${application_client}"
  require_source_pattern 'const EVENT_NAME = "draft://application-open"' \
    "${application_client}"
  require_source_pattern 'opens one queued document without sending a path' \
    src/ipc/applicationOpen.test.ts
  require_source_pattern 'opens a queued macOS DRAFT document through the path-free session boundary' \
    "${workflow_tests}"
  assert_no_matches 'frontend macOS application-open path authority' \
    '\b(?:sourcePath|targetPath|absolutePath|PathBuf|fileUrl)\b|file://' \
    "${application_client}" src/features/document-session/useDocumentSession.ts

  require_source_pattern 'role="status"' "${operation_notice}"
  require_source_pattern 'aria-live="polite"' "${operation_notice}"
  require_source_pattern 'clearFeedback: () => void' \
    src/features/docx-export/useDocxExport.ts
  require_source_pattern 'docxExport.clearFeedback()' \
    src/features/workspace-actions/useWorkspaceActions.ts
  require_source_pattern 'shows a pending state before a selected DOCX resolves' \
    "${workflow_tests}"
  require_source_pattern 'exports DOCX with visible completion and source-safety feedback' \
    "${workflow_tests}"
  require_source_pattern 'presents the typed %s DOCX export failure' \
    "${workflow_tests}"
  require_source_pattern 'opens the Word fixture after clearing a settled export notice' \
    "${workflow_tests}"
  require_rust_test word_fixture_reaches_the_typed_open_response \
    src-tauri/src/commands/document_open.rs

  require_source_pattern 'com.progentic.draft.document' src-tauri/tauri.conf.json
  require_source_pattern 'com.progentic.draft.document' src-tauri/Info.plist
  require_source_pattern 'CFBundleTypeIconFile' src-tauri/Info.plist
  require_source_pattern '| UX-47-009 | UX-1 | Open - failed artifact proves identity only |' \
    "${ledger}"
  require_source_pattern '| UX-47-010 | UX-0 | Closed - artifact 2dfe312b |' \
    "${ledger}"
  require_source_pattern '| UX-47-011 | UX-0 | Closed - artifact 8e974736 |' \
    "${ledger}"
  require_source_pattern '| UX-47-012 | UX-1 | Open - manual retest pending |' \
    "${ledger}"
  require_source_pattern '| UX-47-013 | UX-0 | Open - packaged fidelity retest pending |' \
    "${ledger}"
  require_source_pattern '| UX-47-014 | UX-1 | Closed - artifact 1634d6d2 |' \
    "${ledger}"
  require_source_pattern '| UX-47-015 | UX-1 | Open - partial artifact pass |' \
    "${ledger}"
  require_source_pattern '| UX-47-016 | UX-1 | Open - packaged retest pending |' \
    "${ledger}"
  require_source_pattern '| UX-47-017 | UX-1 | Open - packaged failure; governance required |' \
    "${ledger}"
  require_source_pattern '| UX-47-018 | UX-2 | Open - future workspace scope |' \
    "${ledger}"
  require_source_pattern '| UX-47-019 | UX-2 | Open - future governed capability |' \
    "${ledger}"
  require_file src/editor/PageBreakNode.ts
  require_rust_test supported_direct_run_properties_map_to_exact_canonical_marks \
    src-tauri/src/interoperability/docx_import/tests.rs
  require_rust_test page_break_runs_become_canonical_blocks_and_export_back_to_docx \
    src-tauri/src/interoperability/docx_import/tests.rs
  require_source_pattern 'name: "pageBreak"' src/editor/PageBreakNode.ts
  require_source_pattern 'background: var(--workspace);' src/styles.css
  require_source_pattern 'renders explicit page breaks as full page-surface gaps' \
    tests/frontend/styles.test.ts
  assert_no_matches 'page-break punctuation presentation regression' \
    'border-(?:top|bottom):[^;]*dashed|content:[^;]*(?:-{3,}|Page break)' \
    src/styles.css
  require_source_pattern 'DocxBlock::PageBreak' src-tauri/src/exports/docx_package.rs
  require_source_pattern 'rendered font/paragraph/page-break import' \
    docs/maintainers/DOCX_INTEROPERABILITY.md
  require_source_pattern '| RC-07 | Release blocker | Open |' \
    docs/maintainers/RELEASE_CANDIDATE.md
  require_source_pattern '| GATE-47 | Roadmap gate | Open |' \
    docs/maintainers/RELEASE_CANDIDATE.md
  require_source_pattern "| \`INV-17\` | Proposed |" docs/INVARIANTS.md
  require_file src-tauri/src/commands/window_title.rs
  require_file src/ipc/windowTitle.ts
  require_file src/features/window-title/useWindowTitle.ts
  require_source_pattern 'commands::window_title::set_window_title' src-tauri/src/lib.rs
  require_source_pattern 'const COMMAND_NAME = "set_window_title"' src/ipc/windowTitle.ts
  require_source_pattern 'title_contract_is_bounded_and_path_free' \
    src-tauri/src/commands/window_title.rs
  require_rust_test native_document_title_formats_are_stable \
    src-tauri/src/commands/window_title.rs
  require_source_pattern 'save_dialog_suggestions_preserve_basename_identity' \
    src-tauri/src/commands/document_save.rs
  assert_no_matches 'window-title path authority' \
    '\b(?:sourcePath|targetPath|absolutePath|PathBuf|fileUrl)\b|file://' \
    src/ipc/windowTitle.ts src/features/window-title/useWindowTitle.ts

  printf 'PASS Phase 47 failed-package correction boundaries\n'
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
    valid_nested_citation_round_trips
    invalid_nested_citation_fails_with_path
    invalid_nested_font_format_fails_with_path
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

check_citation_node_contract() {
  local node_path="src-tauri/src/citations/node.rs"
  local node_test_path="src-tauri/src/citations/node_tests.rs"
  local resolution_test_path="src-tauri/src/citations/resolution_tests.rs"
  local required_node_tests=(
    valid_citation_attrs_deserialize
    citation_attrs_serialization_is_stable
    citation_attrs_round_trip_is_stable
    non_object_citation_attrs_fail
    unknown_citation_fields_fail
    missing_citation_fields_fail_predictably
    malformed_and_unsupported_citation_versions_fail
    malformed_citation_citekeys_fail
    unsupported_render_styles_fail
    nested_document_citations_validate
    document_citations_are_collected_in_order
    invalid_nested_citation_reports_path_and_cause
    unrelated_tiptap_nodes_remain_opaque
    citation_failure_shape_is_stable
  )
  local required_resolution_tests=(
    known_citation_resolves_to_disposable_marker
    invalid_citation_fails_before_store_lookup
    missing_reference_fails_explicitly
    corrupt_reference_store_failure_is_preserved
    citation_resolution_failure_shape_is_stable
  )
  local test_name

  require_citation_sources
  require_citation_schema_version "${node_path}"
  for test_name in "${required_node_tests[@]}"; do
    require_rust_test "${test_name}" "${node_test_path}"
  done
  for test_name in "${required_resolution_tests[@]}"; do
    require_rust_test "${test_name}" "${resolution_test_path}"
  done
  require_source_pattern 'marks: ""' src/editor/CitationNode.ts
  require_source_pattern 'data-citation-state' src/editor/CitationNode.ts
  require_source_pattern 'hasValidCitationNodes(value.document)' src/ipc/documentEnvelope.ts
  assert_no_matches "INV-04 embedded citation metadata" \
    '\b(?:title|contributors|issued|identifiers|csl_json)\s*:' \
    src/citations src/editor/CitationNode.ts
  printf 'PASS INV-04 citation node contract\n'
}

check_bibliography_consistency_contract() {
  local source_path="src-tauri/src/citations/bibliography.rs"
  local test_path="src-tauri/src/citations/bibliography_tests.rs"
  local required_tests=(
    matching_citations_and_records_are_consistent
    missing_citekeys_are_reported
    orphaned_citekeys_are_reported
    duplicate_bibliography_citekeys_are_reported
    repeated_in_text_citations_are_not_duplicates
    orphaned_duplicate_categories_are_independent
    consistency_results_are_sorted_and_case_sensitive
    empty_document_and_bibliography_are_consistent
  )
  local test_name

  require_file "${source_path}"
  require_file "${test_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'BTreeMap<String, usize>' "${source_path}"
  require_source_pattern 'BTreeSet<String>' "${source_path}"
  assert_no_matches "Phase 19 bibliography side effects" \
    '(?:std|tokio)::fs|\bReferenceStore\b|\brusqlite\b|\btauri::|#\[tauri::command\]' \
    "${source_path}"
  assert_no_matches "Phase 19 frontend bibliography authority" \
    '\bBibliographyConsistency\b|\bbibliography_' src
  printf 'PASS Phase 19 bibliography consistency contract\n'
}

check_network_client_contract() {
  local source_path="src-tauri/src/network/client.rs"
  local test_path="src-tauri/src/network/client_tests.rs"
  local initializer_path="src-tauri/src/application/network_client.rs"
  local required_tests=(
    current_manifest_builds_network_client
    user_agent_policy_is_deterministic
    request_and_connect_timeouts_are_explicit
    invalid_application_versions_fail
    network_client_failure_shape_is_bounded
    request_gate_enforces_per_service_interval
    request_gate_keeps_services_independent
    server_rate_limits_apply_exponential_backoff
    retry_after_seconds_are_bounded
    transport_failures_are_typed
    response_statuses_are_typed
    response_limit_rejects_oversized_body
    offline_policy_denies_before_url_or_transport_work
    online_policy_preserves_url_validation
  )
  local test_name

  require_file "${source_path}"
  require_file "${test_path}"
  require_file "${initializer_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'reqwest = { version = "0.13.4", default-features = false, features = ["rustls"] }' src-tauri/Cargo.toml
  require_source_pattern '.user_agent(&policy.user_agent)' "${source_path}"
  require_source_pattern '.connect_timeout(policy.connect_timeout)' "${source_path}"
  require_source_pattern '.timeout(policy.request_timeout)' "${source_path}"
  require_source_pattern '.https_only(true)' "${source_path}"
  require_source_pattern 'MAX_METADATA_RESPONSE_BYTES' "${source_path}"
  require_source_pattern 'MAX_RATE_LIMIT_BACKOFF' "${source_path}"
  require_source_pattern 'Arc::new(ConnectivityPolicy::default())' "${initializer_path}"
  require_source_pattern 'NetworkClient::new(Arc::clone(&connectivity))?' "${initializer_path}"
  require_source_pattern 'app.manage(connectivity)' "${initializer_path}"
  require_source_pattern 'app.manage(client)' "${initializer_path}"
  require_source_pattern '.send().await' "${source_path}"
  assert_no_matches "Phase 22 network request execution outside centralized client" \
    '\.(?:send|execute)\s*\(' --glob '!client.rs' src-tauri/src/network
  assert_no_matches "Phase 22 cookie configuration" \
    '\bcookie_store\b' src-tauri/src/network src-tauri/Cargo.toml
  printf 'PASS INV-10 centralized network client contract\n'
}

check_connectivity_contract() {
  local policy_path="src-tauri/src/network/connectivity.rs"
  local network_path="src-tauri/src/network/client.rs"
  local browser_path="src-tauri/src/research/external_access.rs"
  local command_path="src-tauri/src/commands/connectivity.rs"
  local frontend_paths=(
    src/ipc/connectivityMode.ts
    src/ipc/connectivityModeSet.ts
    src/features/connectivity
  )
  local required_tests=(
    policy_defaults_online_and_round_trips_closed_modes
    offline_policy_denies_before_url_or_transport_work
    online_policy_preserves_url_validation
    offline_policy_denies_before_validation_or_browser_launch
  )
  local test_name

  require_file "${policy_path}"
  require_file "${command_path}"
  require_file src/ipc/connectivityMode.test.ts
  require_file src/features/connectivity/useConnectivityMode.test.tsx
  require_file src/features/connectivity/ConnectivityModeControl.test.tsx
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src
  done
  require_source_pattern 'DEFAULT_CONNECTIVITY_MODE: ConnectivityMode = ConnectivityMode::Online' \
    "${policy_path}"
  require_source_pattern 'self.require_online()?;' "${network_path}"
  require_source_pattern '.require_online()' "${browser_path}"
  require_source_pattern 'const COMMAND_NAME = "get_connectivity_mode"' \
    src/ipc/connectivityMode.ts
  require_source_pattern 'const COMMAND_NAME = "set_connectivity_mode"' \
    src/ipc/connectivityModeSet.ts
  require_source_pattern 'ignores an older read after a newer refresh completes' \
    src/features/connectivity/useConnectivityMode.test.tsx
  require_source_pattern 'keeps the effective mode visible and announces a failed change' \
    src/features/connectivity/ConnectivityModeControl.test.tsx

  assert_no_matches "Phase 36 connectivity persistence, probing, or alternate transport" \
    '\brusqlite\b|(?:std|tokio)::fs|\breqwest\b|\bnavigator\.onLine\b|\bsetInterval\s*\(|\blocalStorage\b|\bfetch\s*\(' \
    "${policy_path}" "${command_path}" "${frontend_paths[@]}"
  assert_no_matches "Phase 36 formatting connectivity coupling" \
    '\bConnectivityMode\b|\bConnectivityPolicy\b|\bconnectivity_mode\b' \
    src-tauri/src/formatting src/features/formatting-review
  printf 'PASS INV-10 Phase 36 offline session policy\n'
}

check_metadata_lookup_contract() {
  local metadata_path="src-tauri/src/research/metadata.rs"
  local metadata_test_path="src-tauri/src/research/metadata_tests.rs"
  local provider_directory="src-tauri/src/research/providers"
  local required_tests=(
    doi_is_validated_and_normalized
    malformed_dois_fail_before_network_work
    contact_email_is_validated_and_normalized
    malformed_contact_emails_fail_before_network_work
    normalized_metadata_rejects_invalid_required_fields
    network_failures_map_without_raw_details
    crossref_request_uses_doi_and_polite_contact
    crossref_response_normalizes_candidate_metadata
    crossref_response_rejects_malformed_or_mismatched_data
    semantic_scholar_request_uses_doi_identifier_and_bounded_fields
    semantic_scholar_response_normalizes_candidate_metadata
    semantic_scholar_response_rejects_malformed_or_mismatched_data
    unpaywall_request_uses_doi_and_required_contact
    unpaywall_response_normalizes_candidate_metadata
    unpaywall_response_rejects_malformed_or_mismatched_data
  )
  local test_name

  require_file "${metadata_path}"
  require_file "${metadata_test_path}"
  require_file "${provider_directory}/crossref.rs"
  require_file "${provider_directory}/semantic_scholar.rs"
  require_file "${provider_directory}/unpaywall.rs"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src/research
  done
  require_source_pattern 'https://api.crossref.org/v1' "${provider_directory}/crossref.rs"
  require_source_pattern 'https://api.semanticscholar.org/graph/v1' "${provider_directory}/semantic_scholar.rs"
  require_source_pattern 'https://api.unpaywall.org/v2' "${provider_directory}/unpaywall.rs"
  assert_no_matches "Phase 22 metadata persistence or IPC authority" \
    '\bReferenceStore\b|\brusqlite\b|\btauri::|#\[tauri::command\]|(?:std|tokio)::fs' \
    src-tauri/src/research
  assert_no_matches "Phase 22 frontend metadata authority" \
    '\bCrossref\b|\bSemanticScholar\b|\bUnpaywall\b|\bMetadataRecord\b' src
  printf 'PASS Phase 22 metadata lookup contract\n'
}

check_external_browser_handoff_contract() {
  local domain_path="src-tauri/src/research/external_access.rs"
  local domain_test_path="src-tauri/src/research/external_access_tests.rs"
  local command_path="src-tauri/src/commands/external_access.rs"
  local browser_path="src-tauri/src/system_browser.rs"
  local frontend_path="src/ipc/externalAccess.ts"
  local frontend_test_path="src/ipc/externalAccess.test.ts"
  local required_tests=(
    publisher_and_institutional_urls_open_as_validated_https
    non_https_or_credentialed_urls_fail_before_browser_launch
    doi_handoff_builds_resolver_url
    google_scholar_handoff_builds_bounded_search_url
    malformed_doi_and_query_fail_before_browser_launch
    browser_launch_failures_are_bounded
    offline_policy_denies_before_validation_or_browser_launch
  )
  local test_name

  require_file "${domain_path}"
  require_file "${domain_test_path}"
  require_file "${command_path}"
  require_file "${browser_path}"
  require_file "${frontend_path}"
  require_file "${frontend_test_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${domain_test_path}"
  done
  require_source_pattern 'tauri-plugin-opener = "2.5.4"' src-tauri/Cargo.toml
  require_source_pattern 'tauri_plugin_opener::open_url(url.as_str(), None::<&str>)' "${browser_path}"
  require_source_pattern 'https://doi.org' "${domain_path}"
  require_source_pattern 'https://scholar.google.com/scholar' "${domain_path}"
  require_source_pattern 'const COMMAND_NAME = "open_external_access"' "${frontend_path}"
  # Phase 28 owns one direct Python worker launch. The exact Phase 47 test path
  # invokes macOS textutil only to prove generated DOCX packages can be opened.
  # Neither exclusion grants production browser or process authority.
  assert_no_matches "Phase 23 alternate Rust browser launch" \
    'tauri_plugin_opener::open_url|\bopen::(?:that|with)|\bwebbrowser::|Command::new' \
    --glob '!system_browser.rs' \
    --glob '!**/workers/python/runner.rs' \
    --glob '!**/workers/python/runner_tests.rs' \
    --glob '!**/documents/external_save_tests.rs' src-tauri/src
  assert_no_matches "Phase 23 frontend opener authority" \
    '@tauri-apps/plugin-opener|\bwindow\.open\s*\(|target\s*=\s*[\x22\x27]_blank' src
  assert_no_matches "Phase 23 opener plugin registration" \
    'tauri_plugin_opener::init|\.plugin\([^\n]*opener' src-tauri/src
  assert_no_matches "Phase 23 opener capability" \
    '\bopener:' src-tauri/capabilities
  assert_no_matches "Phase 23 handoff network or persistence authority" \
    '\breqwest\b|\bNetworkClient\b|\brusqlite\b|(?:std|tokio)::fs|\bReferenceStore\b' \
    "${domain_path}" "${command_path}" "${browser_path}"
  printf 'PASS Phase 23 external browser handoff contract\n'
}

check_pdf_import_contract() {
  local source_path="src-tauri/src/imports/pdf.rs"
  local test_path="src-tauri/src/imports/pdf_tests.rs"
  local required_tests=(
    explicit_pdf_enters_pending_after_validation
    explicit_import_rejects_non_pdf_and_symlink
    watched_pdf_waits_during_chunked_write
    watched_pdf_requires_debounce_and_stable_snapshot
    watched_pdf_rejects_paths_outside_root
    watched_pdf_returns_typed_file_failures
  )
  local test_name

  require_file "${source_path}"
  require_file "${test_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'pub const STABLE_WRITE_DEBOUNCE: Duration = Duration::from_secs(1);' "${source_path}"
  require_source_pattern 'const PDF_HEADER: &[u8; 5] = b"%PDF-";' "${source_path}"
  require_source_pattern 'metadata.file_type().is_symlink()' "${source_path}"
  require_source_pattern 'path.starts_with(&self.watched_root)' "${source_path}"
  assert_no_matches "Phase 24 import IPC, persistence, or network authority" \
    '\btauri::|#\[tauri::command\]|\brusqlite\b|\bReferenceStore\b|\bNetworkClient\b|\breqwest\b' \
    "${source_path}"
  assert_no_matches "Phase 24 import file mutation" \
    '\bfs::(?:write|remove_file|rename|copy)\s*\(|\bOpenOptions\b|\.write_all\s*\(' \
    "${source_path}"
  assert_no_matches "Phase 24 unmanaged watcher or worker" \
    '\bnotify::|\bRecommendedWatcher\b|(?:std::thread|tokio)::spawn\s*\(' \
    "${source_path}"
  assert_no_matches "Phase 24 frontend import authority" \
    '\bPendingPdfImport\b|\bWatchedPdfIntake\b|\bimport_pdf\b|\bwatched_folder\b|\bstable_write\b' \
    src
  printf 'PASS INV-08 Phase 24 PDF intake contract\n'
}

check_background_job_contract() {
  local model_path="src-tauri/src/jobs/pdf_import.rs"
  local store_path="src-tauri/src/jobs/store.rs"
  local test_path="src-tauri/src/jobs/store_tests.rs"
  local initialization_path="src-tauri/src/application/job_store.rs"
  local required_tests=(
    candidate_promotion_persists_pending_job
    repeated_promotion_returns_existing_job_without_reset
    separately_validated_candidates_are_not_path_deduplicated
    concurrent_promotions_return_one_durable_job
    concurrent_claims_allow_one_owner
    claim_capability_is_hashed_and_debug_redacted
    foreign_claim_cannot_checkpoint_or_finish
    checkpoint_and_typed_failure_survive_retry_and_reopen
    retry_and_reopen_require_expected_terminal_state_and_attempt
    durable_cancellation_blocks_progress_until_owner_acknowledges
    restart_invalidates_old_claim_and_reassignment_uses_new_token
    restart_turns_cancelled_in_progress_job_terminal
    terminal_resolution_is_immutable
    terminal_failures_are_bounded_and_typed
    malformed_stored_identity_fails_rehydration
  )
  local test_name

  require_file "${model_path}"
  require_file "${store_path}"
  require_file "${test_path}"
  require_file "${initialization_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'CREATE TABLE pdf_import_jobs (' "${store_path}"
  require_source_pattern 'record_id TEXT NOT NULL UNIQUE' "${store_path}"
  require_source_pattern 'TransactionBehavior::Immediate' "${store_path}"
  require_source_pattern 'claim_token_hash BLOB' "${store_path}"
  require_source_pattern 'Sha256::digest' "${store_path}"
  require_source_pattern 'Err(PdfImportJobStoreError::ClaimOwnershipLost)' "${store_path}"
  require_source_pattern 'application::job_store::initialize_job_store(app)?;' src-tauri/src/lib.rs
  require_source_pattern 'sha2 = "0.10.9"' src-tauri/Cargo.toml
  assert_no_matches "Phase 26 raw claim capability exposure" \
    'claim_token\s+TEXT|derive\([^\n]*Serialize|\bpub\s+fn\s+token\s*\(|\b(?:println|eprintln|dbg)!\s*\(|\b(?:tracing|log)::' \
    "${model_path}" "${store_path}"
  assert_no_matches "Phase 26 processing, network, or reference authority" \
    '\breqwest\b|\bNetworkClient\b|\bReferenceStore\b|\bnotify::|\bRecommendedWatcher\b|(?:std::thread|tokio)::spawn\s*\(' \
    "${model_path}" "${store_path}"
  assert_no_matches "Phase 26 source PDF mutation" \
    '\bfs::(?:write|remove_file|rename|copy)\s*\(|\bOpenOptions\b|\.write_all\s*\(' \
    "${model_path}" "${store_path}"
  assert_no_matches "Phase 26 Tauri job command" \
    '\bPdfImportJob\b|\bpromote_candidate\b|\bclaim_token_hash\b' src-tauri/src/commands
  assert_no_matches "Phase 26 frontend job authority" \
    '\bPdfImportJob\b|\bpromote_candidate\b|\bclaim_token\b|\bjob_state\b' src
  printf 'PASS INV-05 Phase 26 persistent PDF job contract\n'
}

check_ai_orchestration_contract() {
  local context_path="src-tauri/src/analysis/context.rs"
  local context_test_path="src-tauri/src/analysis/context_tests.rs"
  local orchestration_path="src-tauri/src/analysis/ai.rs"
  local orchestration_test_path="src-tauri/src/analysis/ai_tests.rs"
  local event_path="src-tauri/src/events/ai_stream.rs"
  local required_tests=(
    request_validates_and_normalizes_bounded_input
    request_rejects_instruction_and_excerpt_bounds
    request_rejects_count_and_duplicate_evidence_bounds
    evidence_identity_and_citekey_fail_closed
    context_preserves_provenance_and_class_order
    context_omits_whole_blocks_deterministically
    request_errors_do_not_include_user_content
    preparation_registers_stream_before_adapter_work
    successful_stream_is_ordered_and_generated_analysis_only
    adapter_receives_typed_provenance_not_flattened_text
    cancellation_before_run_avoids_adapter_start
    cancellation_during_read_cancels_adapter_and_emits_terminal
    adapter_start_and_stream_failures_emit_bounded_terminal_events
    invalid_or_excessive_chunks_cancel_stream_and_fail_typed
    cumulative_output_limit_is_enforced
    event_delivery_failure_stops_adapter_without_content_error
    adapter_and_preparation_errors_do_not_include_context
  )
  local test_name

  require_file "${context_path}"
  require_file "${context_test_path}"
  require_file "${orchestration_path}"
  require_file "${orchestration_test_path}"
  require_file "${event_path}"
  require_file docs/drafts/AI_ORCHESTRATION.md
  require_file docs/maintainers/AI_ORCHESTRATION.md
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" src-tauri/src/analysis
  done
  require_rust_test stream_payload_serialization_is_stable "${event_path}"
  require_source_pattern 'MAX_AI_INSTRUCTION_BYTES: usize = 4 * 1024' "${context_path}"
  require_source_pattern 'MAX_AI_EXCERPTS_PER_CLASS: usize = 64' "${context_path}"
  require_source_pattern 'MAX_AI_EXCERPT_BYTES: usize = 8 * 1024' "${context_path}"
  require_source_pattern 'MAX_AI_CONTEXT_CLASS_BYTES: usize = 32 * 1024' "${context_path}"
  require_source_pattern 'AiContextBlock::UserDocument' "${context_path}"
  require_source_pattern 'AiContextBlock::VerifiedSourceEvidence' "${context_path}"
  require_source_pattern 'is_valid_citekey(value)' "${context_path}"
  require_source_pattern 'MAX_AI_STREAM_CHUNK_BYTES: usize = 16 * 1024' "${orchestration_path}"
  require_source_pattern 'MAX_AI_STREAM_CHUNKS: u32 = 4_096' "${orchestration_path}"
  require_source_pattern 'MAX_AI_STREAM_BYTES: usize = 1024 * 1024' "${orchestration_path}"
  require_source_pattern 'run_until_cancelled(stream.next_chunk()).await' "${orchestration_path}"
  require_source_pattern 'stream.cancel();' "${orchestration_path}"
  require_source_pattern 'GeneratedAnalysis' "${event_path}"
  assert_no_matches "Phase 27 provider, network, or secret authority" \
    '\breqwest\b|\bNetworkClient\b|https?://|\b(?:api[_-]?key|authorization|bearer|credential|secret)\b|\bstd::env\b|\bdotenv\b' \
    src-tauri/src/analysis "${event_path}"
  assert_no_matches "Phase 27 persistence or mutation authority" \
    '\brusqlite\b|\bReferenceStore\b|\bDocumentRegistry\b|\bPdfImportJob\b|(?:std|tokio)::fs|\bOpenOptions\b|\.write_all\s*\(' \
    src-tauri/src/analysis "${event_path}"
  # Tests use the already-linked Tauri executor only to drive deterministic
  # futures. Product orchestration remains independent of the Tauri boundary.
  assert_no_matches "Phase 27 Tauri product authority" \
    '#\[tauri::command\]|\btauri::' \
    "${context_path}" "${orchestration_path}" "${event_path}"
  assert_no_matches "Phase 27 spawned-worker authority" \
    '(?:tokio(?:::task)?|tauri::async_runtime|std::thread)::spawn\s*\(|\bCommand::new\s*\(' \
    src-tauri/src/analysis "${event_path}"
  assert_no_matches "Phase 27 frontend analysis authority" \
    '\bAi(?:Analysis|Model|Stream)|\bGeneratedAnalysis\b|\bVerifiedSourceEvidence\b|draft://ai' src
  assert_no_matches "Phase 27 Python helper coupling" \
    '\bdraft_helpers\b|\bpython(?:3)?\b|\.py\b' src-tauri/src/analysis "${event_path}"
  printf 'PASS INV-14 Phase 27 AI orchestration contract\n'
}

check_v1_analysis_decision_guard() {
  local adr="docs/adr/002-limit-v1-analysis-to-local-text.md"
  local draft="docs/drafts/V1_LOCAL_ANALYSIS.md"
  local model_artifacts
  local status

  require_file "${adr}"
  require_file "${draft}"
  require_source_pattern 'Status: Accepted' "${adr}"
  require_source_pattern 'production analysis is limited to local deterministic text' "${adr}"
  require_source_pattern '## Analysis Layers' "${adr}"
  require_source_pattern 'permitted v1 findings are exactly' "${adr}"
  require_source_pattern 'remains open until a stable complete packaged' "${draft}"
  assert_no_matches "ADR-002 production model dependencies" \
    '(?i)^[[:space:]]*["\x27]?(?:async-openai|anthropic|candle-core|candle-transformers|genai|llama-cpp|llama-cpp-2|mistralrs|ollama-rs|openai-api-rs|ort|rig-core|tch)["\x27]?[[:space:]]*(?:=|:)' \
    src-tauri/Cargo.toml package.json pyproject.toml
  assert_no_matches "ADR-002 direct frontend HTTP or provider dependencies" \
    '(?i)"(?:@anthropic-ai/sdk|@google/generative-ai|@mistralai/mistralai|@openai/agents|axios|got|ky|openai|superagent|undici)"[[:space:]]*:' \
    package.json
  assert_no_matches "ADR-002 frontend provider SDK imports" \
    '(?i)\b(?:from|import)[[:space:]]*[\(]?[[:space:]]*["\x27](?:@anthropic-ai/sdk|@google/generative-ai|@mistralai/mistralai|@openai/agents|openai)["\x27]' \
    src
  assert_no_matches "ADR-002 provider endpoints or credential environment" \
    '(?i)\b(?:OPENAI|ANTHROPIC|COHERE|GEMINI|MISTRAL|OLLAMA|MODEL_PROVIDER)_(?:API_KEY|TOKEN|BASE_URL|ENDPOINT)\b|https?://[^[:space:]"\x27]*(?:openai|anthropic|cohere|generativelanguage|mistral|ollama)' \
    src-tauri/src/analysis src-tauri/src/commands src-tauri/src/application src
  assert_no_matches "ADR-002 runtime model download authority" \
    '(?i)\b(?:download|fetch|pull)_(?:model|weights)\b|\b(?:hf_hub|huggingface_hub|modelscope)\b|https?://huggingface\.co' \
    --glob '!check-invariants.sh' \
    src-tauri/src src scripts src-tauri/tauri.conf.json package.json pyproject.toml
  assert_no_matches "ADR-002 model-provider product authority" \
    '\b(?:OpenAi|OpenAI|Anthropic|Claude|Ollama|Llama|Mistral|ModelProvider|ProviderCredential)\b|https?://[^[:space:]"\x27]*(?:openai|anthropic|ollama)' \
    src-tauri/src/analysis src-tauri/src/commands src-tauri/src/application src
  assert_no_matches "ADR-002 direct frontend provider or secret authority" \
    '\b(?:ModelProvider|ProviderCredential|ProviderEndpoint|SecretStore|SecretValue|loadSecret|storeSecret|providerApiKey)\b' \
    src
  assert_no_matches "ADR-002 direct frontend provider transport" \
    '\bfetch[[:space:]]*\(|\bXMLHttpRequest\b|\bnew[[:space:]]+(?:WebSocket|EventSource)[[:space:]]*\(' \
    src
  assert_no_matches "ADR-002 generative analysis bridge" \
    '#\[tauri::command\][[:space:]]*(?:pub[^[:space:]]+[[:space:]]+)?(?:start|run|generate)_ai|draft://(?:ai|analysis)|\b(?:runAiAnalysis|startAiAnalysis|generateAnalysis)\b' \
    src-tauri/src/commands src-tauri/src/events src
  assert_no_matches "ADR-002 unsupported visible capability language" \
    '(?i)^(?!.*\b(?:not|no|without|unavailable|unimplemented|excluded|outside|deferred|cannot|can\x27t|doesn\x27t|does not|must not|remain absent)\b).*\b(?:AI-powered analysis|semantic analysis|semantic understanding|LLM analysis|generative feedback|originality detection|human-likeness(?: detection)?|AI detection|intelligent assessment|quality assessment|intelligence|reasoning)\b' \
    README.md CHANGELOG.md docs/user docs/wiki src

  if model_artifacts="$(git ls-files | rg '(?i)\.(?:gguf|onnx|safetensors|pt|pth|tflite)$')"; then
    printf '%s\n' "${model_artifacts}" >&2
    echo 'FAILED ADR-002 packaged model artifacts' >&2
    return 1
  else
    status=$?
  fi
  if [[ "${status}" -ne 1 ]]; then
    echo 'FAILED ADR-002 packaged model artifact scan could not run' >&2
    return "${status}"
  fi
  printf 'PASS ADR-002 packaged model artifacts\n'
  printf 'PASS ADR-002 accepted v1 local-analysis guard\n'
}

check_adr_003_accepted_guard() {
  local adr='docs/adr/003-expand-v1-document-interoperability.md'
  local contract='docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md'

  require_file "${adr}"
  require_file "${contract}"
  require_source_pattern 'Status: Accepted' "${adr}"
  require_source_pattern 'Accepted through: PR #37' "${adr}"
  require_source_pattern 'status: Accepted' "${contract}"
  require_source_pattern 'adr: ADR-003' "${contract}"
  require_source_pattern "| \`INV-UX-07\` | Proposed |" docs/INVARIANTS.md
  require_source_pattern 'Optimize documentation for human comprehension first and precision second.' \
    docs/DOCUMENTATION.md
  assert_no_matches 'ADR-003 premature documentation-readability acceptance' \
    '\| \x60INV-UX-07\x60 \| Accepted \|' \
    docs/INVARIANTS.md
  require_file src-tauri/src/interoperability/mod.rs
  require_file src-tauri/src/interoperability/fidelity.rs
  require_file src-tauri/src/interoperability/provenance.rs
  require_file src/ipc/externalDocument.ts
  require_source_pattern 'imported_external' src-tauri/src/documents/persistence.rs
  require_source_pattern 'imported_external' src/ipc/documentOpen.ts
  require_file src-tauri/src/documents/external_save.rs
  require_file src-tauri/src/commands/external_document_save.rs
  require_file src/ipc/externalDocumentSave.ts
  require_source_pattern 'commands::external_document_save::save_external_document' \
    src-tauri/src/lib.rs
  assert_no_matches 'ADR-003 unimplemented Markdown, RTF, or ODT authority' \
    '\b(?:parse_markdown|import_rtf|import_odt)\b' \
    src-tauri/src src
  require_file src-tauri/src/desktop_menu.rs
  require_file src-tauri/src/commands/native_menu.rs
  require_file src/ipc/nativeMenu.ts
  require_file src/features/workspace-actions/useWorkspaceActions.ts
  require_source_pattern 'file.new_document' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.open_document' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.close_document' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.save_document' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.save_document_as' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.save_back_to_source' src-tauri/src/desktop_menu.rs
  require_source_pattern 'file.export_docx' src-tauri/src/desktop_menu.rs
  require_source_pattern 'commands::native_menu::set_native_menu_state' src-tauri/src/lib.rs
  require_source_pattern 'listenToNativeMenuActions' \
    src/features/workspace-actions/useWorkspaceActions.ts
  require_source_pattern 'setNativeMenuState' \
    src/features/workspace-actions/useWorkspaceActions.ts
  require_source_pattern 'action="save_document_as"' \
    src/components/WorkspaceCommandBar.tsx
  require_source_pattern 'action="save_back_to_source"' \
    src/components/WorkspaceCommandBar.tsx
  require_source_pattern 'props.actions.dispatch(props.action)' \
    src/components/WorkspaceCommandBar.tsx
  require_source_pattern 'aria-label="More document actions"' \
    src/components/WorkspaceCommandBar.tsx
  require_source_pattern '<WorkspaceStatusBar' src/app/DraftWorkspace.tsx
  require_source_pattern 'className="workspace-status-bar"' \
    src/components/WorkspaceStatusBar.tsx
  assert_no_matches 'ADR-003 duplicate toolbar document authority' \
    '\b(?:saveDocument|openDocument|closeDocument|exportDocument|DocumentSession)\b' \
    src/components/WorkspaceCommandBar.tsx
  assert_no_matches 'ADR-003 primary-header status regression' \
    '\b(?:ConnectivityModeControl|connectivityState|documentStatus)\b' \
    src/components/WorkspaceHeader.tsx
  assert_no_matches 'ADR-003 frontend native-menu path authority' \
    '\b(?:path|sourcePath|targetPath|filePath)\b' \
    src/ipc/nativeMenu.ts src/features/workspace-actions/useWorkspaceActions.ts
  printf 'PASS ADR-003 accepted interoperability and implemented desktop-workflow guard\n'
}

check_adr_004_accepted_guard() {
  local adr='docs/adr/004-govern-paragraph-formatting.md'
  local contract='docs/contracts/PARAGRAPH_FORMATTING.md'

  require_file "${adr}"
  require_file "${contract}"
  require_source_pattern 'Status: Accepted' "${adr}"
  require_source_pattern 'Accepted through: PR #40' "${adr}"
  require_source_pattern 'status: Accepted' "${contract}"
  require_source_pattern "| \`INV-17\` | Proposed |" docs/INVARIANTS.md
  assert_no_matches 'ADR-004 premature invariant acceptance' \
    '\| \x60INV-17\x60 \| Accepted \|' docs/INVARIANTS.md
  require_file src-tauri/src/documents/paragraph_format.rs
  require_file src-tauri/src/documents/migration.rs
  require_file src/documents/paragraphFormatting.ts
  require_file src/editor/ParagraphFormatting.ts
  require_source_pattern 'pub const DOCUMENT_ENVELOPE_SCHEMA_VERSION: u64 = 2;' \
    src-tauri/src/documents/envelope.rs
  require_source_pattern 'pub(crate) fn from_persisted_json_value' \
    src-tauri/src/documents/envelope.rs
  require_source_pattern 'migrate_v1_to_v2' src-tauri/src/documents/migration.rs
  require_source_pattern 'DocumentEnvelope::from_persisted_json_value' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'validate_document_paragraph_styles' \
    src-tauri/src/documents/envelope.rs
  require_source_pattern 'pub(crate) const MIN_LINE_SPACING_HUNDREDTHS: u64 = 100;' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'pub(crate) const MAX_LINE_SPACING_HUNDREDTHS: u64 = 300;' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'pub(crate) const LINE_SPACING_INCREMENT: u64 = 5;' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'pub(crate) const MAX_PARAGRAPH_SPACING_TWIPS: u64 = 2_880;' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'pub(crate) const MAX_SPECIAL_INDENT_TWIPS: u64 = 1_440;' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'export const MIN_LINE_SPACING_HUNDREDTHS = 100;' \
    src/documents/paragraphFormatting.ts
  require_source_pattern 'export const MAX_LINE_SPACING_HUNDREDTHS = 300;' \
    src/documents/paragraphFormatting.ts
  require_source_pattern 'export const LINE_SPACING_INCREMENT = 5;' \
    src/documents/paragraphFormatting.ts
  require_source_pattern 'export const MAX_PARAGRAPH_SPACING_TWIPS = 2_880;' \
    src/documents/paragraphFormatting.ts
  require_source_pattern 'export const MAX_SPECIAL_INDENT_TWIPS = 1_440;' \
    src/documents/paragraphFormatting.ts
  require_source_pattern 'hasValidParagraphStyles' src/ipc/documentEnvelope.ts
  require_source_pattern 'ParagraphFormatting' src/editor/DraftEditor.tsx
  require_source_pattern 'write_paragraph_style' src-tauri/src/exports/docx_package.rs
  require_source_pattern 'legacy_document_migrates_in_memory_and_first_save_writes_version_two' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'paragraph_styles_emit_deterministic_docx_properties' \
    src-tauri/src/exports/docx_tests.rs
  require_source_pattern 'every_required_field_and_nested_field_is_enforced' \
    src-tauri/src/documents/paragraph_format.rs
  require_source_pattern 'ignores arbitrary pasted CSS and malformed DRAFT attributes' \
    src/editor/ParagraphFormatting.test.ts
  assert_no_matches 'ADR-004 paragraph controls before Phase 47' \
    '\b(?:setParagraphAlignment|setLineSpacing|setParagraphSpacing|setParagraphIndent|resetParagraphFormatting|paragraph_formatting)\b' \
    src/components src/features src/app
  printf 'PASS ADR-004 accepted decision and proposed invariant guard\n'
}

require_citation_sources() {
  require_file src-tauri/src/citations/node.rs
  require_file src-tauri/src/citations/node_tests.rs
  require_file src-tauri/src/citations/resolution.rs
  require_file src-tauri/src/citations/resolution_tests.rs
  require_file src-tauri/src/commands/citation_resolution.rs
  require_file src/citations/citationNode.test.ts
  require_file src/citations/citationNode.ts
  require_file src/editor/CitationNode.test.ts
  require_file src/editor/CitationNode.ts
  require_file src/ipc/citationResolution.test.ts
  require_file src/ipc/citationResolution.ts
}

require_citation_schema_version() {
  local source_path="$1"
  local declaration='pub const CITATION_NODE_SCHEMA_VERSION: u64 = 1;'

  if ! rg --quiet --fixed-strings "${declaration}" "${source_path}"; then
    printf 'FAILED Phase 18 citation schema version declaration\n' >&2
    return 1
  fi
}

require_envelope_schema_version() {
  local source_path="$1"
  local declaration='pub const DOCUMENT_ENVELOPE_SCHEMA_VERSION: u64 = 2;'

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
    "${source_path}"
  # Phase 38 exposes only the fixed reference_record schema-name token in its
  # bounded diagnostic contract. These exact IPC files hold no reference data
  # or reference-store authority.
  assert_no_matches "Phase 16 frontend reference authority" \
    '\bReferenceRecord\b|\breference_record\b' \
    --glob '!src/ipc/diagnosticSnapshot.ts' \
    --glob '!src/ipc/diagnosticSnapshot.test.ts' src
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

check_reference_store_contract() {
  local source_path="src-tauri/src/references/store.rs"
  local test_path="src-tauri/src/references/store_tests.rs"
  local required_tests=(
    reference_store_path_uses_app_data_directory
    new_store_initializes_schema_and_table
    schema_initialization_is_idempotent
    unsupported_store_schema_fails_explicitly
    claimed_current_schema_requires_expected_table
    claimed_current_schema_requires_expected_constraints
    store_creates_missing_parent_directory
    unavailable_parent_returns_storage_error
    directory_database_path_returns_open_error
    malformed_database_returns_schema_read_error
    conflicting_zero_version_schema_returns_migration_error
    create_read_and_reopen_preserve_record
    duplicate_identity_and_citekey_fail_explicitly
    citekey_uniqueness_is_case_sensitive
    update_replaces_payload_and_citekey
    conflicting_update_preserves_both_records
    delete_returns_record_and_removes_it
    list_is_deterministic_by_citekey
    missing_update_and_delete_fail_explicitly
    malformed_stored_json_fails_without_deleting_row
    missing_live_table_returns_read_error
    invalid_stored_record_returns_typed_cause
    mismatched_stored_indexes_fail_closed
    mismatched_stored_schema_fails_closed
    unsupported_stored_record_versions_fail_without_mutation
    concurrent_create_allows_one_record
    poisoned_store_returns_unavailable
    store_failure_shape_is_stable
  )
  local test_name

  require_file "${source_path}"
  require_file "${test_path}"
  require_reference_store_schema_version "${source_path}"
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${test_path}"
  done
  require_source_pattern 'Mutex<Connection>' "${source_path}"
  require_source_pattern 'TransactionBehavior::Immediate' "${source_path}"
  require_source_pattern 'PRAGMA user_version = 1;' "${source_path}"
  require_source_pattern ') STRICT;' "${source_path}"
  require_source_pattern 'features = ["bundled"]' src-tauri/Cargo.toml
  assert_no_matches "ad hoc SQLite access outside owned stores" \
    '\brusqlite\b|Connection::open\s*\(' \
    --glob '!src-tauri/src/references/store.rs' \
    --glob '!src-tauri/src/references/store_tests.rs' \
    --glob '!src-tauri/src/jobs/store.rs' \
    --glob '!src-tauri/src/jobs/store_tests.rs' src-tauri/src
  assert_no_matches "Phase 17 reference store Tauri surface" \
    '#\[tauri::command\]|\btauri::' "${source_path}"
  printf 'PASS Phase 17 local reference store contract\n'
}

check_data_migration_contract() {
  local document_path='src-tauri/src/documents/persistence.rs'
  local citation_test_path='src-tauri/src/citations/node_tests.rs'
  local reference_path='src-tauri/src/references/store.rs'
  local reference_test_path='src-tauri/src/references/store_tests.rs'

  require_file docs/maintainers/DATA_MIGRATION.md
  require_rust_test current_document_version_loads_without_mutation \
    "${document_path}"
  require_rust_test unsupported_document_versions_fail_without_mutation \
    "${document_path}"
  require_rust_test malformed_and_unsupported_citation_versions_fail \
    "${citation_test_path}"
  require_rust_test unsupported_stored_record_versions_fail_without_mutation \
    "${reference_test_path}"
  require_rust_test conflicting_zero_version_schema_returns_migration_error \
    "${reference_test_path}"
  require_rust_test create_read_and_reopen_preserve_record \
    "${reference_test_path}"
  require_source_pattern 'DocumentEnvelope::from_json_value(value)' "${document_path}"
  require_source_pattern 'ReferenceRecord::from_json_value(value)' "${reference_path}"
  require_source_pattern '0 => migrate_zero_to_one(connection)' "${reference_path}"
  require_source_pattern 'found => Err(ReferenceStoreError::UnsupportedStoreSchema { found })' \
    "${reference_path}"
  assert_no_matches "Phase 43 job-state mutation authority" \
    '\bPdfImportJobStore\b|\bjobs::' "${document_path}" "${reference_path}"
  printf 'PASS Phase 43 fail-closed data migration baseline\n'
}

require_reference_store_schema_version() {
  local source_path="$1"
  local declaration='pub const REFERENCE_STORE_SCHEMA_VERSION: u64 = 1;'

  if ! rg --quiet --fixed-strings "${declaration}" "${source_path}"; then
    printf 'FAILED Phase 17 store schema version declaration\n' >&2
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
    external_source_path_cannot_back_two_live_handles
    draft_save_replaces_external_authority_only_after_registry_update
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
    font_formatting_persists_through_save_close_and_reopen
    malformed_json_fails_before_registry_entry
    unsupported_document_versions_fail_without_mutation
    duplicate_load_returns_already_open
    save_uses_explicit_snapshot_and_retained_path
    save_new_snapshot_uses_rust_selected_path
    save_target_preflight_matches_registry_state
    save_target_preflight_rejects_invalid_snapshot
    cancelled_first_save_does_not_register_document
    first_save_rejects_non_draft_target_before_write
    invalid_snapshot_fails_before_path_selection
    save_does_not_reopen_dialog_for_loaded_document
    failed_first_save_does_not_register_document
    failed_attach_preserves_registry_state
    failed_existing_save_preserves_registry_snapshot
    save_rejects_source_path_owned_by_another_document
    durability_failure_advances_registry_to_complete_source
    concurrent_saves_keep_disk_and_registry_consistent
    invalid_citation_open_fails_before_registry_entry
    invalid_citation_save_fails_before_path_selection
    invalid_font_format_fails_before_path_selection
    text_import_is_unsaved_and_first_save_preserves_source
    later_import_save_reuses_only_the_chosen_draft_target
    markdown_import_uses_basename_and_remains_literal_editable_source
    malformed_and_oversized_text_imports_fail_without_mutation
    unsupported_open_extension_fails_before_read_or_registration
    docx_import_open_close_preserves_source_and_exposes_no_path
    imported_docx_saves_only_to_new_draft_target
    cancelled_docx_save_preserves_source_and_external_registration
    failed_docx_save_preserves_source_and_external_registration
    imported_docx_export_creates_copy_without_rebinding_source
    duplicate_docx_source_is_rejected_without_mutation
    failed_docx_import_preserves_source_and_registry
    docx_path_aliases_share_one_canonical_external_identity
  )
  local test_name

  require_document_file_sources
  for test_name in "${required_tests[@]}"; do
    require_rust_test "${test_name}" "${persistence_path}"
  done
  require_source_pattern 'keeps the complete canonical font allowlist stable' \
    src/documents/textFormatting.test.ts
  require_source_pattern 'keeps an imported filename display-only through first and later saves' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'keeps an imported session unsaved when its first Save is cancelled' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'rejects a non-DRAFT first-save target without changing import state' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'preserves native content, selection, title, and state when Open is cancelled' \
    src/ipc/Phase46Workflows.test.tsx
  require_source_pattern 'preserves unsaved content when a later New command fails' \
    src/ipc/Phase46Workflows.test.tsx
  require_rust_test source_path_cannot_back_two_live_handles \
    src-tauri/src/documents/registry.rs
  require_rust_test source_path_conflict_preserves_unattached_handle \
    src-tauri/src/documents/registry.rs
  require_rust_test registry_failure_shape_is_stable \
    src-tauri/src/documents/registry.rs
  require_atomic_writer_tests "${atomic_writer_path}"
  check_document_write_boundary "${atomic_writer_path}"
  check_async_native_dialog_boundary
  check_frontend_document_file_boundary
  printf 'PASS INV-03/06/09 document file contract\n'
}

check_async_native_dialog_boundary() {
  local dialog_path='src-tauri/src/documents/dialog.rs'

  assert_no_matches "blocking native dialog API" \
    '\bblocking_(?:pick|save)_file\s*\(' \
    src-tauri/src
  require_source_pattern 'pub(crate) async fn select_open_document(' "${dialog_path}"
  require_source_pattern 'pub(crate) async fn select_save_document(' "${dialog_path}"
  require_source_pattern 'pub(crate) async fn select_export_docx(' "${dialog_path}"
  require_source_pattern '.pick_file(move |selected|' "${dialog_path}"
  require_source_pattern '.save_file(move |selected|' "${dialog_path}"
  require_source_pattern 'pub(crate) async fn open_document(' \
    src-tauri/src/commands/document_open.rs
  require_source_pattern 'pub(crate) async fn save_document(' \
    src-tauri/src/commands/document_save.rs
  require_source_pattern 'select_save_document(&app_handle, &suggested_file_name)' \
    src-tauri/src/commands/document_save.rs
  assert_no_matches 'Save command avoids open dialog selector' \
    'select_open_document' \
    src-tauri/src/commands/document_save.rs
  require_source_pattern 'display_name: String' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'was_save_as: bool' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'displayName: string; wasSaveAs: boolean' \
    src/ipc/documentSave.ts
  require_source_pattern 'pub(crate) async fn export_document(' \
    src-tauri/src/commands/docx_export.rs
  printf 'PASS async Rust-owned native dialog contract\n'
}

check_critical_path_contract() {
  local source_path="src-tauri/src/critical_paths_tests.rs"

  require_file "${source_path}"
  require_source_pattern 'mod critical_paths_tests;' src-tauri/src/lib.rs
  require_rust_test critical_document_path_is_durable_citable_and_exportable \
    "${source_path}"
  require_source_pattern 'save_document(' "${source_path}"
  require_source_pattern 'open_document(' "${source_path}"
  require_source_pattern 'DocumentRegistryError::AlreadyOpen' "${source_path}"
  require_source_pattern 'resolve_citation(' "${source_path}"
  require_source_pattern 'DocxExportError::UnsupportedCitation' "${source_path}"
  require_source_pattern 'export_docx(' "${source_path}"
  require_source_pattern 'ZipArchive::new' "${source_path}"
  assert_no_matches "Phase 41 test-only product authority" \
    '#\[tauri::command\]|generate_handler|\btauri::|Command::new|\breqwest\b|\bkeyring\b' \
    "${source_path}"
  printf 'PASS Phase 41 critical-path evidence\n'
}

require_document_file_sources() {
  require_file src-tauri/src/commands/document_open.rs
  require_file src-tauri/src/commands/document_save.rs
  require_file src-tauri/src/documents/atomic_write.rs
  require_file src-tauri/src/documents/dialog.rs
  require_file src-tauri/src/documents/persistence.rs
  require_file src-tauri/src/documents/text_import.rs
  require_file src/ipc/documentEnvelope.test.ts
  require_file src/ipc/documentEnvelope.ts
  require_file src/ipc/documentErrors.ts
  require_file src/ipc/documentOpen.ts
  require_file src/ipc/documentOpen.test.ts
  require_file src/ipc/externalDocument.ts
  require_file src/ipc/externalDocument.test.ts
  require_file src/ipc/documentSave.ts
  require_file src/ipc/documentSave.test.ts
  require_source_pattern 'MAX_TEXT_IMPORT_BYTES: usize = 8 * 1024 * 1024' \
    src-tauri/src/documents/text_import.rs
  require_source_pattern 'OpenedDraft {' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'ImportedText {' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'ImportedExternal {' \
    src-tauri/src/documents/persistence.rs
  require_source_pattern 'const OPEN_DOCUMENT_EXTENSIONS: &[&str] = &["draft", "json", "txt", "md", "docx"]' \
    src-tauri/src/documents/dialog.rs
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

  # These exact test files write only temporary fixtures. The Python runner's
  # write_all targets piped child stdin, and the DOCX package writer plus its
  # colocated import and external-save fixtures target in-memory ZipWriter
  # instances. None writes a filesystem path. Production document, intake,
  # job, and export mutation remain denied by separate source scans.
  assert_no_matches "INV-09 direct document target writes" \
    '\b(?:fs::write|fs::rename|fs::copy|File::create|File::options|OpenOptions::new)\s*\(|\.write_all\s*\(' \
    --glob "!${atomic_writer_path}" \
    --glob '!src-tauri/src/imports/pdf_tests.rs' \
    --glob '!src-tauri/src/jobs/store_tests.rs' \
    --glob '!src-tauri/src/references/store_tests.rs' \
    --glob '!src-tauri/src/exports/docx_package.rs' \
    --glob '!src-tauri/src/interoperability/docx_import/tests.rs' \
    --glob '!src-tauri/src/documents/external_save_tests.rs' \
    --glob '!src-tauri/src/workers/python/runner.rs' src-tauri/src
  require_source_pattern '.tempfile_in(parent)' "${atomic_writer_path}"
  require_source_pattern '.sync_all()' "${atomic_writer_path}"
  require_source_pattern '.persist(target_path)' "${atomic_writer_path}"
  require_source_pattern 'sync_directory(parent)' "${atomic_writer_path}"
}

check_frontend_document_file_boundary() {
  local document_identity_paths=(
    src/features/document-session
    src/ipc/documentCreate.ts
  )

  assert_no_matches "frontend document path authority" \
    '\b(?:path|filePath|file_path)\s*:' \
    src/features/document-session/useDocumentSession.ts \
    src/ipc/documentCreate.ts src/ipc/documentOpen.ts src/ipc/documentSave.ts
  assert_no_matches "frontend document identity authority" \
    '\bcrypto\.randomUUID\s*\(|\b(?:uuidv[147]|nanoid|createId)\s*\(' \
    "${document_identity_paths[@]}"
  assert_no_matches "frontend document identity dependency" \
    "from\\s+['\"](?:uuid|nanoid|@paralleldrive/cuid2)['\"]" \
    "${document_identity_paths[@]}"
  require_source_pattern 'DocumentEnvelope::create_initial()' \
    src-tauri/src/commands/document_create.rs
  require_source_pattern 'type DocumentOrigin = SaveDocumentOrigin;' \
    src/features/document-session/useDocumentSession.ts
  require_source_pattern 'export type SaveDocumentOrigin =' src/ipc/documentSave.ts
  require_source_pattern 'origin: result.status' \
    src/features/document-session/useDocumentSession.ts
  require_source_pattern 'origin: "opened_draft" as const' \
    src/features/document-session/useDocumentSession.ts
  require_source_pattern 'return "Imported, unsaved"' \
    src/features/document-session/useDocumentSession.ts
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

check_pdf_export_deferral_guard() {
  require_file docs/adr/001-defer-native-pdf-export.md
  require_file docs/maintainers/PDF_EXPORT_DECISION.md
  require_file docs/drafts/FORMATTING_UX.md

  assert_no_matches "Phase 33 PDF export symbols" \
    '\b(?:PdfExport|PdfArtifact|PdfRenderer|export_pdf|compile_pdf|render_pdf|print_to_pdf)\b' \
    python/draft_helpers src-tauri/src src
  assert_no_matches "Phase 33 PDF conversion runtime" \
    '(?i)\b(?:wkhtmltopdf|weasyprint|libreoffice|soffice|pdfium|chromiumoxide|headless_chrome)\b' \
    python/draft_helpers src-tauri/src src src-tauri/tauri.conf.json
  assert_no_matches "Phase 33 PDF renderer dependencies" \
    '(?i)^[[:space:]]*["\x27]?(?:printpdf|lopdf|pdf-writer|pdfium-render|chromiumoxide|headless_chrome|wkhtmltopdf|weasyprint)["\x27]?[[:space:]]*(?:=|:)' \
    src-tauri/Cargo.toml package.json pyproject.toml
  assert_no_matches "Phase 33 frontend PDF claims" \
    '(?i)\b(?:export(?:[[:space:]]+to)?|download|save[[:space:]]+as)[[:space:]]+pdf\b|\bpdf[[:space:]]+export\b' \
    src
  printf 'PASS Phase 33 PDF export deferral guard\n'
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
    '\breqwest\b|\bureq\b|hyper::Client' \
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

report_pdf_deferral_status() {
  printf '%s\n' \
    'INFO PDF export remains absent under accepted ADR-001; no PDF runtime path is active.'
}

main "$@"
