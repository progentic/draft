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
  check_coverage_matrix
  check_coverage_symbols
  check_configuration_index
  check_configuration_backlinks
  check_wiki_sources
  check_visible_error_recovery
  check_readme_scope
  check_pdf_decision_state

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
    docs/drafts/AI_ORCHESTRATION.md
    docs/drafts/BACKGROUND_JOBS.md
    docs/drafts/BIBLIOGRAPHY_CONSISTENCY.md
    docs/drafts/DOCUMENT_ENVELOPE.md
    docs/drafts/DOCX_EXPORT.md
    docs/drafts/EXTERNAL_BROWSER_HANDOFF.md
    docs/drafts/FORMATTING_CHECKS.md
    docs/drafts/FORMATTING_UX.md
    docs/drafts/CITATION_NODE.md
    docs/drafts/NETWORK_CLIENT.md
    docs/drafts/PDF_IMPORT.md
    docs/drafts/PDF_EXPORT_DECISION.md
    docs/drafts/PYTHON_HELPERS.md
    docs/drafts/TEXT_ANALYSIS.md
    docs/drafts/METADATA_LOOKUP.md
    docs/drafts/REFERENCE_RECORD.md
    docs/drafts/REFERENCE_STORE.md
    docs/GOVERNANCE.md
    docs/INVARIANTS.md
    docs/PHASEMAP.md
    docs/ROADMAP.md
    docs/maintainers/AI_ORCHESTRATION.md
    docs/maintainers/CANCELLATION_BOUNDARY.md
    docs/maintainers/BACKGROUND_JOBS.md
    docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md
    docs/maintainers/CITATION_NODE.md
    docs/maintainers/COMMAND_BOUNDARY.md
    docs/maintainers/CONFIGURATION.md
    docs/maintainers/DOCUMENTATION_COVERAGE.md
    docs/maintainers/DOCUMENT_ENVELOPE.md
    docs/maintainers/DOCUMENT_REGISTRY.md
    docs/maintainers/DOCUMENT_SAVE_LOAD.md
    docs/maintainers/DOCX_EXPORT.md
    docs/maintainers/ERROR_MESSAGES.md
    docs/maintainers/EVENT_BOUNDARY.md
    docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md
    docs/maintainers/FRONTEND_COMMAND_CLIENT.md
    docs/maintainers/FORMATTING_CHECKS.md
    docs/maintainers/FORMATTING_UX.md
    docs/maintainers/NETWORK_CLIENT.md
    docs/maintainers/PDF_IMPORT.md
    docs/maintainers/PACKAGING.md
    docs/maintainers/PERFORMANCE_MEASUREMENT.md
    docs/maintainers/PYTHON_HELPERS.md
    docs/maintainers/TEXT_ANALYSIS.md
    docs/maintainers/METADATA_LOOKUP.md
    docs/maintainers/REFERENCE_RECORD.md
    docs/maintainers/REFERENCE_STORE.md
    docs/maintainers/REALIGNMENT.md
    docs/maintainers/TOOLCHAIN.md
    docs/maintainers/WORKSPACE_UI.md
    docs/user/WORKSPACE.md
    docs/wiki/Current-Limitations.md
    docs/wiki/Home.md
    docs/wiki/Troubleshooting.md
    docs/wiki/Workspace.md
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
  local checkpoint='Phases 0 through 34 are complete'

  if ! rg --quiet --fixed-strings "${checkpoint}" docs/ROADMAP.md || \
    ! rg --quiet --fixed-strings "${checkpoint}" docs/PHASEMAP.md; then
    echo "ROADMAP.md and PHASEMAP.md must agree on the completed phase checkpoint" >&2
    return 1
  fi
}

check_coverage_matrix() {
  local matrix='docs/maintainers/DOCUMENTATION_COVERAGE.md'
  local header='| Subsystem | Code Surface | Maintainer Doc | User Doc | ADR | Invariant | Tests | Gap |'

  require_document_text "${matrix}" "${header}"
  check_matrix_subsystems "${matrix}"
  require_document_text "${matrix}" '457 granular lint findings remain'
  require_document_text "${matrix}" 'Live Wiki publication verified'
}

check_matrix_subsystems() {
  local matrix="$1"
  local subsystems=(
    'Desktop runtime and managed state'
    'Workspace shell and editor'
    'Runtime status and visible failures'
    'Typed Tauri command client'
    'Transient worker cancellation'
    'Document envelope'
    'Document registry'
    'Document open, save, and atomic replacement'
    'Reference record'
    'Reference store'
    'Citation node and resolution'
    'Bibliography consistency'
    'Central network client'
    'Metadata providers'
    'External browser handoff'
    'PDF intake candidate'
    'Durable PDF import jobs'
    'AI orchestration'
    'Python helper process'
    'Text-analysis findings'
    'Formatting review'
    'DOCX export'
    'Error presentation'
    'Verification and repository tooling'
    'Packaging and application icons'
    'PDF export decision'
  )
  local subsystem

  for subsystem in "${subsystems[@]}"; do
    require_document_text "${matrix}" "| ${subsystem} |"
  done
}

check_coverage_symbols() {
  local matrix='docs/maintainers/DOCUMENTATION_COVERAGE.md'
  local entries=(
    'src-tauri/src/lib.rs|run'
    'src/app/DraftWorkspace.tsx|DraftWorkspace'
    'src-tauri/src/commands/runtime_status.rs|get_runtime_status'
    'src-tauri/src/workers/cancellation.rs|WorkerCancellationRegistry'
    'src-tauri/src/documents/envelope.rs|DocumentEnvelope'
    'src-tauri/src/documents/registry.rs|DocumentRegistry'
    'src-tauri/src/references/record.rs|ReferenceRecord'
    'src-tauri/src/references/store.rs|ReferenceStore'
    'src-tauri/src/citations/bibliography.rs|check_bibliography_consistency'
    'src-tauri/src/network/client.rs|NetworkClient'
    'src-tauri/src/research/providers/crossref.rs|lookup_crossref'
    'src-tauri/src/imports/pdf.rs|prepare_explicit_pdf'
    'src-tauri/src/jobs/store.rs|PdfImportJobStore'
    'src-tauri/src/analysis/context.rs|assemble_model_request'
    'src-tauri/src/workers/python/runner.rs|PythonHelperRunner'
    'src-tauri/src/workers/python/text_analysis.rs|TextAnalysisInput'
    'src-tauri/src/formatting/checks.rs|run_formatting_checks'
    'src-tauri/src/commands/formatting_review.rs|run_formatting_review'
    'src/features/formatting-review/useFormattingReview.ts|useFormattingReview'
    'src-tauri/src/exports/docx.rs|compile_docx'
  )
  local entry
  local source_path
  local symbol

  for entry in "${entries[@]}"; do
    IFS='|' read -r source_path symbol <<<"${entry}"
    require_document_text "${source_path}" "${symbol}"
    require_document_text "${matrix}" "${symbol}"
  done
}

check_configuration_index() {
  local index='docs/maintainers/CONFIGURATION.md'
  local symbols=(
    DOCUMENT_ENVELOPE_SCHEMA_VERSION
    CITATION_NODE_SCHEMA_VERSION
    REFERENCE_RECORD_SCHEMA_VERSION
    REFERENCE_STORE_SCHEMA_VERSION
    REFERENCE_STORE_FILENAME
    REFERENCE_STORE_BUSY_TIMEOUT
    JOB_STORE_SCHEMA_VERSION
    JOB_STORE_FILENAME
    JOB_STORE_BUSY_TIMEOUT
    MAX_JOB_FAILURE_MESSAGE_BYTES
    NETWORK_CONNECT_TIMEOUT
    NETWORK_REQUEST_TIMEOUT
    PROVIDER_REQUEST_INTERVAL
    MAX_METADATA_RESPONSE_BYTES
    MAX_RATE_LIMIT_BACKOFF
    STABLE_WRITE_DEBOUNCE
    MAX_EXTERNAL_URL_LENGTH
    MAX_SCHOLAR_QUERY_LENGTH
    MAX_DOI_LENGTH
    MAX_CONTACT_EMAIL_LENGTH
    DOI_RESOLVER_BASE_URL
    GOOGLE_SCHOLAR_BASE_URL
    CROSSREF_BASE_URL
    SEMANTIC_SCHOLAR_BASE_URL
    UNPAYWALL_BASE_URL
    MAX_AI_INSTRUCTION_BYTES
    MAX_AI_EXCERPTS_PER_CLASS
    MAX_AI_EXCERPT_BYTES
    MAX_AI_CONTEXT_CLASS_BYTES
    MAX_EVIDENCE_ID_BYTES
    MAX_CITEKEY_BYTES
    MAX_AI_STREAM_CHUNK_BYTES
    MAX_AI_STREAM_CHUNKS
    MAX_AI_STREAM_BYTES
    PYTHON_HELPER_PROTOCOL_VERSION
    CONTRACT_PROBE_VERSION
    TEXT_ANALYSIS_VERSION
    MAX_CONTRACT_PROBE_TEXT_BYTES
    MAX_TEXT_ANALYSIS_TEXT_BYTES
    MAX_PYTHON_HELPER_REQUEST_BYTES
    MAX_PYTHON_HELPER_STDOUT_BYTES
    MAX_PYTHON_HELPER_STDERR_BYTES
    PYTHON_HELPER_TIMEOUT
    MAX_TEXT_ANALYSIS_FINDINGS
    SUPPORTED_LOCALE
    MAX_FINDINGS_PER_CHECK
    LONG_SENTENCE_WORDS
    MIN_ALL_CAPS_LETTERS
    MIN_REPEATED_OPENER_LETTERS
    MAX_FORMATTING_HEADINGS
    MAX_FORMATTING_CITATIONS
    MAX_HEADING_TITLE_BYTES
    DEFAULT_FORMATTING_STYLE
    MAX_DOCX_SOURCE_BYTES
    MAX_DOCX_NODES
    MAX_DOCX_NESTING_DEPTH
    MAX_DOCX_ARTIFACT_BYTES
    DOCUMENT_EXTENSIONS
    DEFAULT_DOCUMENT_FILE_NAME
  )
  local symbol

  for symbol in "${symbols[@]}"; do
    require_document_text "${index}" "${symbol}"
    require_source_symbol "${symbol}"
  done
}

require_source_symbol() {
  local symbol="$1"

  if ! rg --quiet --fixed-strings \
    "${symbol}" src-tauri/src src python; then
    echo "Configuration index names a missing source symbol: ${symbol}" >&2
    return 1
  fi
}

check_configuration_backlinks() {
  local guides=(
    AI_ORCHESTRATION.md
    BACKGROUND_JOBS.md
    CITATION_NODE.md
    DOCUMENT_ENVELOPE.md
    DOCUMENT_SAVE_LOAD.md
    DOCX_EXPORT.md
    EXTERNAL_BROWSER_HANDOFF.md
    FORMATTING_CHECKS.md
    FORMATTING_UX.md
    METADATA_LOOKUP.md
    NETWORK_CLIENT.md
    PACKAGING.md
    PDF_IMPORT.md
    PYTHON_HELPERS.md
    REFERENCE_RECORD.md
    REFERENCE_STORE.md
    TEXT_ANALYSIS.md
  )
  local guide

  for guide in "${guides[@]}"; do
    require_document_text \
      "docs/maintainers/${guide}" \
      'docs/maintainers/CONFIGURATION.md'
  done
}

check_wiki_sources() {
  local home='docs/wiki/Home.md'
  local workspace='docs/wiki/Workspace.md'

  require_document_text "${home}" '(Workspace)'
  require_document_text "${home}" '(Troubleshooting)'
  require_document_text "${home}" '(Current-Limitations)'
  require_document_text "${workspace}" '(Troubleshooting)'
  require_document_text "${workspace}" '(Current-Limitations)'
  require_document_text docs/wiki/Current-Limitations.md '(Home)'
  require_document_text docs/wiki/Troubleshooting.md '(Home)'
  require_document_text docs/wiki/Workspace.md '(Home)'
  require_document_text docs/DOCUMENTATION.md 'canonical source for the public GitHub Wiki'
  reject_document_pattern \
    '\]\([^)]*\.md\)' \
    'Wiki links must use extensionless GitHub Wiki page names' \
    docs/wiki/Current-Limitations.md \
    docs/wiki/Home.md \
    docs/wiki/Troubleshooting.md \
    docs/wiki/Workspace.md
  reject_document_pattern \
    'Welcome to the draft wiki!' \
    'Wiki sources must not contain the initialization placeholder' \
    docs/wiki/Current-Limitations.md \
    docs/wiki/Home.md \
    docs/wiki/Troubleshooting.md \
    docs/wiki/Workspace.md
}

check_visible_error_recovery() {
  local presentation='src/components/DocumentInspector.tsx'
  local formatting_presentation='src/features/formatting-review/FormattingReviewPanel.tsx'
  local recovery='docs/wiki/Troubleshooting.md'
  local messages=(
    'DRAFT received an unsupported application version.'
    'DRAFT could not deliver the core status event.'
    'DRAFT could not read the core status.'
    'Core status invalid'
    'Core unavailable'
  )
  local message

  for message in "${messages[@]}"; do
    require_document_text "${presentation}" "${message}"
    require_document_text "${recovery}" "${message}"
  done

  require_document_text "${formatting_presentation}" 'DRAFT received an invalid formatting response.'
  require_document_text "${formatting_presentation}" 'Formatting review could not reach the DRAFT core.'
  require_document_text "${recovery}" 'DRAFT received an invalid formatting response.'
  require_document_text "${recovery}" 'Formatting review could not reach the DRAFT core.'
}

check_readme_scope() {
  local forbidden_heading='^## (Architecture|Build|Contributing|Development|Governance|Implementation|Repository Layout|Roadmap|Testing)($|[[:space:]])'
  local status

  if rg --quiet "${forbidden_heading}" README.md; then
    echo 'README.md contains a maintainer or repository heading' >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo 'README scope scan could not run' >&2
    return "${status}"
  fi
}

check_pdf_decision_state() {
  local adr='docs/adr/001-defer-native-pdf-export.md'
  local decision_record='docs/maintainers/PDF_EXPORT_DECISION.md'
  local phase34_draft='docs/drafts/FORMATTING_UX.md'
  local user_workspace='docs/user/WORKSPACE.md'
  local user_limits='docs/wiki/Current-Limitations.md'
  local living_files=(
    docs/ARCHITECTURE.md
    docs/INVARIANTS.md
    docs/PHASEMAP.md
    docs/ROADMAP.md
    docs/maintainers/DOCUMENTATION_COVERAGE.md
    docs/maintainers/PDF_EXPORT_DECISION.md
    docs/maintainers/TOOLCHAIN.md
  )

  [[ -f "${adr}" ]] || return 0
  require_document_text "${adr}" 'Status: Accepted'
  require_document_text "${decision_record}" '**One-time owner override**'
  require_document_text "${decision_record}" "It does not change \`GOVERNANCE.md\`"
  require_document_text "${phase34_draft}" 'bounded Phase 34'
  require_document_text "${phase34_draft}" 'ADR-001 is accepted'
  require_document_text "${user_workspace}" 'DRAFT has deferred that work'
  require_document_text "${user_limits}" 'DRAFT has deferred that work'
  reject_document_pattern \
    'ADR-001 proposes|Proposed ADR-001|ADR-001 is proposed|Phase 33 is not complete|Phase 33 is under architecture review|Status: Proposed' \
    'Phase 33 and ADR-001 must remain accepted after the governed merge' \
    "${living_files[@]}" "${adr}"
}

require_document_text() {
  local document_path="$1"
  local required_text="$2"

  if ! rg --quiet --fixed-strings "${required_text}" "${document_path}"; then
    echo "Documentation is missing required coverage: ${document_path}: ${required_text}" >&2
    return 1
  fi
}

reject_document_pattern() {
  local pattern="$1"
  local message="$2"
  shift 2
  local documents=("$@")
  local status

  if rg --line-number "${pattern}" "${documents[@]}"; then
    echo "Documentation contains forbidden content: ${message}" >&2
    return 1
  else
    status=$?
  fi

  if [[ "${status}" -ne 1 ]]; then
    echo 'Documentation exclusion scan could not run' >&2
    return "${status}"
  fi
}

main "$@"
