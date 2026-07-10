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
  check_python_helper_contract
  check_document_envelope_contract
  check_reference_record_contract
  check_reference_store_contract
  check_citation_node_contract
  check_bibliography_consistency_contract
  check_network_client_contract
  check_metadata_lookup_contract
  check_external_browser_handoff_contract
  check_pdf_import_contract
  check_background_job_contract
  check_ai_orchestration_contract
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
  require_source_pattern 'tokio = { version = "1.52.3", features = ["io-util", "macros", "process", "time"] }' src-tauri/Cargo.toml
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
  assert_no_matches "Phase 28 application or command helper authority" \
    '\bPythonHelperRunner\b|\bcontract_probe\b|draft_helpers/worker\.py' \
    src-tauri/src/application src-tauri/src/commands
  assert_no_matches "Phase 28 frontend helper authority" \
    '\bPythonHelper\b|\bcontract_probe\b|\bContractProbe\b' src
  printf 'PASS INV-11 Phase 28 Python helper contract\n'
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
  require_source_pattern 'app.manage(NetworkClient::new()?)' "${initializer_path}"
  require_source_pattern '.send().await' "${source_path}"
  assert_no_matches "Phase 22 network request execution outside centralized client" \
    '\.(?:send|execute)\s*\(' --glob '!client.rs' src-tauri/src/network
  assert_no_matches "Phase 22 cookie configuration" \
    '\bcookie_store\b' src-tauri/src/network src-tauri/Cargo.toml
  printf 'PASS INV-10 centralized network client contract\n'
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
  # Phase 28 owns one direct Python process launch under the worker boundary;
  # it is not a browser launch path and remains governed by its own scans.
  assert_no_matches "Phase 23 alternate Rust browser launch" \
    'tauri_plugin_opener::open_url|\bopen::(?:that|with)|\bwebbrowser::|Command::new' \
    --glob '!system_browser.rs' \
    --glob '!**/workers/python/runner.rs' \
    --glob '!**/workers/python/runner_tests.rs' src-tauri/src
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
    "${source_path}"
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
    invalid_citation_open_fails_before_registry_entry
    invalid_citation_save_fails_before_path_selection
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

  # These exact test files write only temporary fixtures. The Python runner's
  # write_all targets piped child stdin, not a filesystem path. Production
  # document, intake, and job mutation remain denied by separate source scans.
  assert_no_matches "INV-09 direct document target writes" \
    '\b(?:fs::write|fs::rename|fs::copy|File::create|File::options|OpenOptions::new)\s*\(|\.write_all\s*\(' \
    --glob "!${atomic_writer_path}" \
    --glob '!src-tauri/src/imports/pdf_tests.rs' \
    --glob '!src-tauri/src/jobs/store_tests.rs' \
    --glob '!src-tauri/src/references/store_tests.rs' \
    --glob '!src-tauri/src/workers/python/runner.rs' src-tauri/src
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
  assert_no_matches "Phase 29 text-analysis behavior" \
    '\b(?:TextAnalysis|GrammarFinding|ClarityFinding|ToneFinding|CohesionFinding|VoiceFinding|analyze_text|run_text_analysis)\b' \
    python/draft_helpers src-tauri/src/workers/python src
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

report_deferred_behavior_checks() {
  printf '%s\n' \
    'INFO The remaining future feature absence gate stays active until its owning phase adds behavioral checks.'
}

main "$@"
