#!/usr/bin/env bash
set -euo pipefail

# Performs offline documentation sanity checks without validating external URLs.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools find rg sort tail wc

  check_required_documents
  report_local_agent_instructions
  check_docs_have_headings
  check_heading_parser_contract
  check_machine_specific_links
  check_changelog_shape
  check_phase_checkpoint
  check_coverage_matrix
  check_coverage_symbols
  check_configuration_index
  check_configuration_backlinks
  check_wiki_sources
  check_visible_error_recovery
  check_formatting_export_alignment
  check_offline_mode_documentation
  check_secret_storage_documentation
  check_diagnostic_snapshot_documentation
  check_error_ux_documentation
  check_critical_path_documentation
  check_packaging_documentation
  check_data_migration_documentation
  check_release_candidate_documentation
  check_v1_analysis_decision_state
  check_v1_usability_documentation
  check_adr_003_accepted_state
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
    docs/contracts/V1_USABILITY_ACCEPTANCE.md
    docs/DOCUMENTATION.md
    docs/drafts/AI_ORCHESTRATION.md
    docs/drafts/AUDIT_DIAGNOSTICS.md
    docs/drafts/BACKGROUND_JOBS.md
    docs/drafts/BIBLIOGRAPHY_CONSISTENCY.md
    docs/drafts/DOCUMENT_ENVELOPE.md
    docs/drafts/DOCX_EXPORT.md
    docs/drafts/ERROR_UX.md
    docs/drafts/EXTERNAL_BROWSER_HANDOFF.md
    docs/drafts/FORMATTING_CHECKS.md
    docs/drafts/FORMATTING_UX.md
    docs/drafts/CITATION_NODE.md
    docs/drafts/NETWORK_CLIENT.md
    docs/drafts/OFFLINE_MODE.md
    docs/drafts/PDF_IMPORT.md
    docs/drafts/PDF_EXPORT_DECISION.md
    docs/drafts/PYTHON_HELPERS.md
    docs/drafts/TEXT_ANALYSIS.md
    docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md
    docs/drafts/V1_LOCAL_ANALYSIS.md
    docs/drafts/METADATA_LOOKUP.md
    docs/drafts/REFERENCE_RECORD.md
    docs/drafts/REFERENCE_STORE.md
    docs/drafts/SECRET_STORAGE.md
    docs/GOVERNANCE.md
    docs/INVARIANTS.md
    docs/adr/003-expand-v1-document-interoperability.md
    docs/PHASEMAP.md
    docs/ROADMAP.md
    docs/maintainers/AI_ORCHESTRATION.md
    docs/maintainers/AUDIT_DIAGNOSTICS.md
    docs/maintainers/CANCELLATION_BOUNDARY.md
    docs/maintainers/BACKGROUND_JOBS.md
    docs/maintainers/BIBLIOGRAPHY_CONSISTENCY.md
    docs/maintainers/CITATION_NODE.md
    docs/maintainers/COMMAND_BOUNDARY.md
    docs/maintainers/CONFIGURATION.md
    docs/maintainers/CRITICAL_PATHS.md
    docs/maintainers/DATA_MIGRATION.md
    docs/maintainers/DOCUMENTATION_COVERAGE.md
    docs/maintainers/DOCUMENT_ENVELOPE.md
    docs/maintainers/DOCUMENT_REGISTRY.md
    docs/maintainers/DOCUMENT_SAVE_LOAD.md
    docs/maintainers/DOCX_EXPORT.md
    docs/maintainers/ERROR_MESSAGES.md
    docs/maintainers/ERROR_UX.md
    docs/maintainers/EVENT_BOUNDARY.md
    docs/maintainers/EXTERNAL_BROWSER_HANDOFF.md
    docs/maintainers/FRONTEND_COMMAND_CLIENT.md
    docs/maintainers/FORMATTING_CHECKS.md
    docs/maintainers/FORMATTING_UX.md
    docs/maintainers/NETWORK_CLIENT.md
    docs/maintainers/OFFLINE_MODE.md
    docs/maintainers/PDF_IMPORT.md
    docs/maintainers/PACKAGING.md
    docs/maintainers/PERFORMANCE_MEASUREMENT.md
    docs/maintainers/PHASE46_WORKFLOWS.md
    docs/maintainers/PYTHON_HELPERS.md
    docs/maintainers/TEXT_ANALYSIS.md
    docs/maintainers/METADATA_LOOKUP.md
    docs/maintainers/REFERENCE_RECORD.md
    docs/maintainers/REFERENCE_STORE.md
    docs/maintainers/RELEASE_CANDIDATE.md
    docs/maintainers/REALIGNMENT.md
    docs/maintainers/SECRET_STORAGE.md
    docs/maintainers/TOOLCHAIN.md
    docs/maintainers/V1_USABILITY_EVIDENCE.md
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

check_v1_usability_documentation() {
  local contract='docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  local ledger='docs/maintainers/V1_USABILITY_EVIDENCE.md'

  require_document_text "${contract}" 'status: Accepted'
  require_document_text "${contract}" '## Required Qualities'
  require_document_text "${contract}" '## Supported v1 Workflow'
  require_document_text "${contract}" '## First-Time-User Task Validation'
  require_document_text "${contract}" '## Measurable Release Thresholds'
  require_document_text "${contract}" '## Phase 49 - Usability And Performance Validation'
  require_document_text "${contract}" '## Phase 51 - Secure Usability'
  require_document_text "${contract}" '## Phase 52 - Packaged Release-Candidate Gate'
  require_document_text "${contract}" '## Phase 53 - Release Entry Point'
  require_document_text "${contract}" 'Accepted ADR-002 authorizes Phase 46 to implement local deterministic text'
  require_document_text docs/ROADMAP.md 'docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  require_document_text docs/PHASEMAP.md 'docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  require_document_text docs/DOCUMENTATION.md 'docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  require_document_text docs/maintainers/RELEASE_CANDIDATE.md \
    'docs/contracts/V1_USABILITY_ACCEPTANCE.md'
  require_document_text docs/maintainers/DOCUMENTATION_COVERAGE.md \
    'v1 usability acceptance'
  require_document_text "${ledger}" '154c34c96183ff67d4ecd6acd790b0410403dd58'
  require_document_text "${ledger}" '68aa08d8a0577ec32a128cd3368ea830be7f91f5'
  require_document_text "${ledger}" 'ae66d3dae64fbe738fcd371b776b27d022bea3182eb9920c89773498dcf289f9'
  require_document_text "${ledger}" 'a0f1ab8d5cc0def97fe98d501324633e341bef74'
  require_document_text "${ledger}" '3b4e996091d9a6618d62570070fcc3d412b394690b855b502114d4f2cc1e7dd0'
  require_document_text "${ledger}" '228bce73e9ea210e9f6f842d8bb2683b70031de4'
  require_document_text "${ledger}" '8870dcc412dfb04ada5ae0ba28ea630eb37925bc94b55b4f182e423b5afd9eb4'
  require_document_text "${ledger}" '## Phase 46'
  require_document_text "${ledger}" '### Automated Evidence'
  require_document_text "${ledger}" '### Findings And Dispositions'
  require_document_text "${ledger}" '| UX-46-001 | UX-0 | Closed |'
  require_document_text "${ledger}" '| UX-46-002 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-003 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-004 | UX-2 | Closed |'
  require_document_text "${ledger}" '| UX-46-005 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-006 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-007 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-008 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-009 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-010 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-011 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-012 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-013 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-014 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-015 | UX-2 | Open |'
  require_document_text "${ledger}" '| UX-46-016 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-017 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-018 | UX-1 | Closed |'
  require_document_text "${ledger}" '| UX-46-019 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-020 | UX-1 | Open |'
  require_document_text "${ledger}" '| UX-46-021 | UX-1 | Open |'
  require_document_text docs/maintainers/RELEASE_CANDIDATE.md \
    "font-control finding \`UX-46-019\`"
  require_document_text docs/maintainers/CONFIGURATION.md \
    "\`arial\`, \`avenir_next\`, \`baskerville\`, \`courier_new\`, \`georgia\`, \`helvetica\`, \`menlo\`, \`palatino\`, \`times_new_roman\`, \`trebuchet_ms\`, \`verdana\`"
  require_document_text docs/maintainers/CONFIGURATION.md \
    "\`MAX_TEXT_IMPORT_BYTES\` | 8 MiB"
  require_document_text docs/maintainers/CONFIGURATION.md \
    "| Document font family | \`georgia\` |"
  require_document_text docs/maintainers/CONFIGURATION.md \
    '| Document body font size | 13 points |'
  require_document_text docs/maintainers/PHASE46_WORKFLOWS.md \
    'whole point sizes from 8 through 72 in one-point'
  require_document_text docs/maintainers/PHASE46_WORKFLOWS.md \
    "\`data-draft-font-family\` and \`data-draft-font-size\`"
  require_document_text docs/maintainers/PHASE46_WORKFLOWS.md \
    "\`invalid_envelope\` before a dialog or filesystem operation begins"
  require_document_text docs/maintainers/DOCX_EXPORT.md "\`w:rFonts\`"
  require_document_text docs/maintainers/DOCX_EXPORT.md "\`w:sz\` and \`w:szCs\`"
  require_document_text docs/wiki/Workspace.md \
    'Arial, Avenir Next, Baskerville, Courier New'
  require_document_text docs/wiki/Workspace.md \
    'Imported, unsaved'
  require_document_text docs/maintainers/DOCUMENT_SAVE_LOAD.md \
    "\`opened_draft\` means Rust retained a native"
  require_document_text docs/maintainers/DOCUMENT_SAVE_LOAD.md \
    "\`invalid_target\` when a new target does not end in \`.draft\`"
  require_document_text docs/wiki/Current-Limitations.md \
    'whole point sizes from 8 through 72'
  require_document_text docs/wiki/Current-Limitations.md \
    "DOCX, RTF, OpenDocument (\`.odt\`), and legacy Word (\`.doc\`) import are"
}

check_adr_003_accepted_state() {
  local adr='docs/adr/003-expand-v1-document-interoperability.md'
  local contract='docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md'
  local release='docs/maintainers/RELEASE_CANDIDATE.md'

  require_document_text "${adr}" 'Status: Accepted'
  require_document_text "${adr}" 'Accepted through: PR #37'
  require_document_text "${adr}" '| 47 | Document interoperability |'
  require_document_text "${adr}" '| 50 | Documentation and drift realignment |'
  require_document_text "${adr}" '| 53 | v1.0.0 release |'
  require_document_text "${adr}" 'Phase 47 implementation may begin only after Phase'
  require_document_text "${adr}" '### Documentation comprehension'
  require_document_text "${contract}" 'status: Accepted'
  require_document_text "${contract}" 'adr: ADR-003'
  require_document_text "${contract}" '## Successor Gate Chain'
  require_document_text "${contract}" "| \`RC-07\`, \`GATE-47\` | Phase 47 |"
  require_document_text "${contract}" "| \`RC-08\`, \`GATE-48\` | Phase 48 |"
  require_document_text "${contract}" "| \`RC-05\`, \`GATE-51\` | Phase 51 |"
  require_document_text "${release}" '## Accepted ADR-003 Gate Chain'
  require_document_text "${release}" 'Every new or remapped'
  require_document_text docs/ROADMAP.md \
    'The authoritative successor sequence is:'
  require_document_text docs/PHASEMAP.md \
    '| 47 | Document interoperability |'
  require_document_text docs/PHASEMAP.md \
    '| 53 | v1.0.0 release |'
  require_document_text docs/ARCHITECTURE.md \
    'Accepted ADR-003 adds a Rust-owned external-document lifecycle'
  require_document_text docs/INVARIANTS.md \
    "| \`INV-UX-07\` | Proposed |"
  require_document_text docs/DOCUMENTATION.md \
    'docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md'
  require_document_text docs/DOCUMENTATION.md \
    'Optimize documentation for human comprehension first and precision second.'
  require_document_text docs/DOCUMENTATION.md '### 2.1 Plain Language Requirement'
  require_major_maintainer_section_policy
  require_document_text "${contract}" \
    'Phase 49 also reviews maintainer documentation as a teaching surface.'
  require_document_text "${contract}" \
    'Only then may a separate governed change mark'
  require_document_text "${release}" \
    'maintainer-documentation comprehension'
  require_document_text docs/maintainers/DOCUMENTATION_COVERAGE.md \
    '| Document interoperability and desktop workflows |'
  require_adr_003_coverage_areas

  if [[ -e docs/drafts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md ]]; then
    echo 'Accepted ADR-003 contract must not retain a live draft copy' >&2
    return 1
  fi
  reject_document_pattern \
    'Proposed ADR-003|ADR-003 remains Proposed|Proposed and non-binding' \
    'accepted ADR-003 surfaces cannot retain proposal-state language' \
    "${adr}" "${contract}" docs/ARCHITECTURE.md docs/DOCUMENTATION.md \
    docs/ROADMAP.md docs/PHASEMAP.md docs/maintainers/DOCUMENTATION_COVERAGE.md \
    docs/maintainers/RELEASE_CANDIDATE.md
  reject_document_pattern \
    '\| \x60INV-UX-07\x60 \| Accepted \|' \
    'INV-UX-07 must remain proposed until Phase 50 evidence exists' \
    docs/INVARIANTS.md
}

require_major_maintainer_section_policy() {
  local policy='docs/DOCUMENTATION.md'
  local sections=(
    Purpose
    Problem
    Solution
    Trade-offs
    'Technical Contract'
    'Implementation Notes'
    'Failure Modes'
    Tests
    'Related Documents'
  )
  local section

  for section in "${sections[@]}"; do
    require_document_text "${policy}" "${section}"
  done
}

require_adr_003_coverage_areas() {
  local matrix='docs/maintainers/DOCUMENTATION_COVERAGE.md'
  local suffixes=(
    INTEROP
    ROUNDTRIP
    FIDELITY
    NATIVE-MENU
    DESKTOP-UI
    USABILITY-PERF
    REALIGNMENT
    GATE-REMAP
  )
  local suffix
  local identifier
  local match_count

  for suffix in "${suffixes[@]}"; do
    identifier="ADR003-COV-${suffix}"
    require_document_text "${matrix}" "| \`${identifier}\` |"
    match_count="$(rg --only-matching --fixed-strings \
      "${identifier}" "${matrix}" | wc -l)"
    if [[ "${match_count}" -ne 1 ]]; then
      printf 'ADR-003 proposal coverage identifier %s expected once, found %s\n' \
        "${identifier}" "${match_count}" >&2
      return 1
    fi
    if rg --quiet --fixed-strings \
      --glob '!**/DOCUMENTATION_COVERAGE.md' \
      "${identifier}" docs; then
      printf 'ADR-003 proposal coverage identifier escaped its owning surface: %s\n' \
        "${identifier}" >&2
      return 1
    fi
    if rg --quiet --fixed-strings "${identifier}" src src-tauri/src; then
      printf 'ADR-003 proposal coverage identifier entered product source: %s\n' \
        "${identifier}" >&2
      return 1
    fi
  done
}

check_data_migration_documentation() {
  local migration_doc='docs/maintainers/DATA_MIGRATION.md'

  require_document_text "${migration_doc}" 'DRAFT has no released schema'
  require_document_text "${migration_doc}" 'Future versions are never downgraded.'
  require_document_text "${migration_doc}" \
    'preserve document source bytes until an atomic replacement succeeds'
  require_document_text "${migration_doc}" 'one immediate SQLite transaction'
  require_document_text docs/maintainers/DOCUMENT_ENVELOPE.md 'DATA_MIGRATION.md'
  require_document_text docs/maintainers/REFERENCE_STORE.md 'DATA_MIGRATION.md'
  require_document_text docs/maintainers/DOCUMENTATION_COVERAGE.md \
    'Data migration baseline'
  printf 'PASS Phase 43 data migration documentation\n'
}

check_release_candidate_documentation() {
  local candidate_doc='docs/maintainers/RELEASE_CANDIDATE.md'

  require_document_text "${candidate_doc}" \
    'not a release-candidate declaration'
  require_document_text "${candidate_doc}" \
    'An open blocker must name evidence, an owner, a closure phase'
  require_document_text "${candidate_doc}" 'Phase 52 entry is stricter'
  require_document_text "${candidate_doc}" \
    '| GATE-45 | Roadmap gate | Closed |'
  require_document_text docs/ROADMAP.md \
    'DRAFT is not ready for v1.0.0 unless a user can identify the primary controls'
  require_document_text docs/PHASEMAP.md \
    'DRAFT is not ready for v1.0.0 unless a user can identify the primary controls'
  require_document_text docs/maintainers/DOCUMENTATION_COVERAGE.md \
    'Release-candidate hardening'
  require_document_text docs/maintainers/PACKAGING.md 'RELEASE_CANDIDATE.md'
  require_document_text docs/maintainers/REALIGNMENT.md \
    '## Phase 45 - 2026-07-11'
  printf 'PASS Phase 44/45 release-candidate documentation\n'
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

  while IFS= read -r document_path; do
    if [[ ! -s "${document_path}" ]]; then
      echo "Documentation file is empty: ${document_path}" >&2
      return 1
    fi

    check_document_heading "${document_path}"
  done < <(find docs -type f -name "*.md" -print | sort)
}

check_document_heading() {
  local document_path="$1"
  local first_line

  IFS= read -r first_line <"${document_path}"
  if [[ "${first_line}" != '---' ]]; then
    require_top_level_heading "${document_path}" "${first_line}"
    return
  fi

  check_frontmatter_heading "${document_path}"
}

check_frontmatter_heading() {
  local document_path="$1"
  local line
  local has_field=false
  local is_closed=false

  while IFS= read -r line; do
    if [[ "${is_closed}" == false ]]; then
      if [[ "${line}" == '---' ]]; then
        if [[ "${has_field}" == false ]]; then
          echo "Documentation has empty frontmatter: ${document_path}" >&2
          return 1
        fi
        is_closed=true
      elif [[ -n "${line}" ]]; then
        if [[ ! "${line}" =~ ^[A-Za-z][A-Za-z0-9_-]*:[[:space:]]*[^[:space:]].*$ ]]; then
          echo "Documentation has malformed frontmatter: ${document_path}" >&2
          return 1
        fi
        has_field=true
      fi
      continue
    fi
    [[ -z "${line}" ]] && continue
    require_top_level_heading "${document_path}" "${line}"
    return
  done < <(tail -n +2 "${document_path}")

  if [[ "${is_closed}" == false ]]; then
    echo "Documentation has unterminated frontmatter: ${document_path}" >&2
  else
    echo "Documentation needs content after frontmatter: ${document_path}" >&2
  fi
  return 1
}

require_top_level_heading() {
  local document_path="$1"
  local content_line="$2"

  if [[ "${content_line}" != \#* ]]; then
    echo "Documentation file needs a top-level heading: ${document_path}" >&2
    return 1
  fi
}

check_heading_parser_contract() {
  local fixture_root='scripts/fixtures/docs'
  local invalid_fixture
  local invalid_fixtures=(
    blank-leading.fixture
    empty-frontmatter.fixture
    leading-content.fixture
    malformed-frontmatter.fixture
    repeated-frontmatter.fixture
    unterminated-frontmatter.fixture
  )

  require_file "${fixture_root}/heading-only.fixture"
  require_file "${fixture_root}/valid-frontmatter.fixture"
  check_document_heading "${fixture_root}/heading-only.fixture"
  check_document_heading "${fixture_root}/valid-frontmatter.fixture"
  for invalid_fixture in "${invalid_fixtures[@]}"; do
    require_file "${fixture_root}/${invalid_fixture}"
    if check_document_heading "${fixture_root}/${invalid_fixture}" \
      >/dev/null 2>&1; then
      printf 'Heading parser accepted invalid fixture: %s\n' "${invalid_fixture}" >&2
      return 1
    fi
  done
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
  local checkpoint='Phases 0 through 45 are complete'

  if ! rg --quiet --fixed-strings "${checkpoint}" docs/ROADMAP.md || \
    ! rg --quiet --fixed-strings "${checkpoint}" docs/PHASEMAP.md || \
    ! rg --quiet --fixed-strings "${checkpoint}" docs/maintainers/TOOLCHAIN.md; then
    echo "Roadmap, phasemap, and toolchain must agree on the completed phase checkpoint" >&2
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
  require_document_text "${matrix}" '1bddd52'
  require_document_text docs/DOCUMENTATION.md '1bddd52'
  require_document_text docs/maintainers/REALIGNMENT.md '## Phase 40 - 2026-07-11'
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
    'Document create, open, import, save, close, and atomic replacement'
    'Critical-flow evidence'
    'Reference record'
    'Reference store'
    'Citation node and resolution'
    'Bibliography consistency'
    'Central network client'
    'Offline session policy'
    'OS-native secret storage'
    'Local diagnostic snapshot'
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
    'Document interoperability and desktop workflows'
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
    'src-tauri/src/network/connectivity.rs|ConnectivityPolicy'
    'src-tauri/src/secrets/store.rs|SecretStore'
    'src-tauri/src/secrets/store.rs|SecretValue'
    'src-tauri/src/diagnostics.rs|DiagnosticSnapshot'
    'src-tauri/src/commands/diagnostic_snapshot.rs|get_diagnostic_snapshot'
    'src/ipc/diagnosticSnapshot.ts|getDiagnosticSnapshot'
    'src-tauri/src/commands/connectivity.rs|get_connectivity_mode'
    'src/features/connectivity/useConnectivityMode.ts|useConnectivityMode'
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
    'src-tauri/src/commands/document_close.rs|close_document'
    'src-tauri/src/commands/document_create.rs|create_document'
    'src-tauri/src/commands/reference_library.rs|add_reference'
    'src-tauri/src/commands/text_analysis.rs|run_text_analysis'
    'src-tauri/src/commands/docx_export.rs|export_document'
    'src/features/document-session/useDocumentSession.ts|useDocumentSession'
    'src/features/text-analysis/TextAnalysisPanel.tsx|TextAnalysisPanel'
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
    NATIVE_SERVICE_NAME
    API_KEY_ACCOUNT_PREFIX
    MAX_INTEGRATION_NAME_BYTES
    MAX_SECRET_BYTES
    DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION
    MAX_DIAGNOSTIC_SNAPSHOT_BYTES
    MAX_APPLICATION_VERSION_BYTES
    NETWORK_CONNECT_TIMEOUT
    DEFAULT_CONNECTIVITY_MODE
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
    AUDIT_DIAGNOSTICS.md
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
    OFFLINE_MODE.md
    PACKAGING.md
    PDF_IMPORT.md
    PYTHON_HELPERS.md
    REFERENCE_RECORD.md
    REFERENCE_STORE.md
    SECRET_STORAGE.md
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
  local presentation='src/features/error-ux/errorPresentation.ts'
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

  require_document_text "${presentation}" 'DRAFT received an invalid formatting response. Check again.'
  require_document_text "${presentation}" 'Formatting review could not reach the DRAFT core. Restart DRAFT, then check again.'
  require_document_text "${recovery}" 'DRAFT received an invalid formatting response. Check again.'
  require_document_text "${recovery}" 'Formatting review could not reach the DRAFT core. Restart DRAFT, then check'
  require_document_text "${presentation}" 'DRAFT received an invalid connectivity response.'
  require_document_text "${presentation}" 'DRAFT could not resolve this citation. Keep it unchanged.'
  require_document_text "${recovery}" '## Citation Cannot Be Resolved'
}

check_formatting_export_alignment() {
  local checks='docs/maintainers/FORMATTING_CHECKS.md'
  local command='docs/maintainers/COMMAND_BOUNDARY.md'
  local client='docs/maintainers/FRONTEND_COMMAND_CLIENT.md'
  local review='docs/maintainers/FORMATTING_UX.md'
  local docx='docs/maintainers/DOCX_EXPORT.md'
  local pdf='docs/maintainers/PDF_EXPORT_DECISION.md'
  local offline='docs/drafts/OFFLINE_MODE.md'

  require_document_text "${checks}" 'FORMATTING_UX.md'
  require_document_text "${command}" 'run_formatting_review'
  require_document_text "${client}" 'runFormattingReview'
  require_document_text "${review}" 'stale result cannot'
  require_document_text "${docx}" 'frontend wrapper, and visible control'
  require_document_text "${pdf}" 'PDF export remains mechanically absent'
  require_document_text docs/wiki/Workspace.md 'not certify complete style-manual compliance'
  require_document_text docs/wiki/Current-Limitations.md 'citation-style declarations'
  require_document_text "${offline}" 'Rust-owned session policy'
  require_document_text "${offline}" 'does not add operating-system reachability monitoring'

  reject_document_pattern \
    'No command or visible workflow can invoke it|Findings are not persisted or visible|DOCX-export absence gate remains active' \
    'Phase 35 formatting documentation must describe the implemented review workflow' \
    "${checks}"
}

check_offline_mode_documentation() {
  local guide='docs/maintainers/OFFLINE_MODE.md'
  local draft='docs/drafts/OFFLINE_MODE.md'
  local workspace='docs/wiki/Workspace.md'
  local recovery='docs/wiki/Troubleshooting.md'

  require_document_text "${guide}" 'DEFAULT_CONNECTIVITY_MODE'
  require_document_text "${guide}" 'get_connectivity_mode'
  require_document_text "${guide}" 'set_connectivity_mode'
  require_document_text "${guide}" 'does not monitor the operating system'
  require_document_text "${draft}" 'non-binding requirements draft for Phase 36'
  require_document_text "${workspace}" '## Work Offline'
  require_document_text "${workspace}" 'setting resets to'
  require_document_text "${workspace}" 'online when DRAFT restarts'
  require_document_text "${recovery}" '## Connectivity Mode Unavailable'
  require_document_text "${recovery}" 'Online - change failed'
  require_document_text docs/wiki/Current-Limitations.md 'does not detect operating-system'
  require_document_text docs/INVARIANTS.md "Phase 36 adds one shared \`ConnectivityPolicy\`"
}

check_secret_storage_documentation() {
  local guide='docs/maintainers/SECRET_STORAGE.md'
  local draft='docs/drafts/SECRET_STORAGE.md'
  local next_draft='docs/drafts/AUDIT_DIAGNOSTICS.md'

  require_document_text "${guide}" "\`keyring\` 4.1.4"
  require_document_text "${guide}" "\`zeroize\` 1.9.0"
  require_document_text "${guide}" 'no Tauri command'
  require_document_text "${guide}" 'never access a real'
  require_document_text "${draft}" 'non-binding requirements draft for Phase 37'
  require_document_text "${next_draft}" 'non-binding requirements draft for Phase 38'
  require_document_text "${next_draft}" 'secret presence and secret-store operations are never queried'
  require_document_text docs/wiki/Current-Limitations.md 'does not expose provider credentials'
  require_document_text docs/wiki/Current-Limitations.md 'diagnostics export'
  require_document_text docs/INVARIANTS.md "Phase 37 adds one lazy \`SecretStore\`"
}

check_diagnostic_snapshot_documentation() {
  local guide='docs/maintainers/AUDIT_DIAGNOSTICS.md'
  local draft='docs/drafts/AUDIT_DIAGNOSTICS.md'
  local next_draft='docs/drafts/ERROR_UX.md'

  require_document_text "${guide}" 'get_diagnostic_snapshot'
  require_document_text "${guide}" 'MAX_DIAGNOSTIC_SNAPSHOT_BYTES'
  require_document_text "${guide}" 'Native credential storage is omitted entirely'
  require_document_text "${guide}" "\`SecretStore\`"
  require_document_text "${guide}" 'No React component or hook imports the wrapper'
  require_document_text "${draft}" 'non-binding requirements draft for Phase 38'
  require_document_text "${next_draft}" 'non-binding requirements draft for Phase 39'
  require_document_text "${next_draft}" 'typed but unwired backend failure'
  require_document_text docs/wiki/Current-Limitations.md 'diagnostics export'
  require_document_text docs/INVARIANTS.md 'Phase 38 diagnostics omit the secret-storage subsystem'
}

check_error_ux_documentation() {
  local guide='docs/maintainers/ERROR_UX.md'
  local draft='docs/drafts/ERROR_UX.md'
  local inventory='docs/maintainers/ERROR_MESSAGES.md'

  require_document_text "${guide}" 'four visible surfaces'
  require_document_text "${guide}" "\`retryable\`"
  require_document_text "${guide}" "\`actionable\`"
  require_document_text "${guide}" "\`terminal\`"
  require_document_text "${guide}" 'Labels are allowed only when an already-visible control can honor them.'
  require_document_text "${guide}" 'outer fallbacks'
  require_document_text "${guide}" 'unwired'
  require_document_text "${inventory}" 'Typed but unwired errors remain'
  require_document_text "${draft}" 'non-binding requirements draft for Phase 39'
  require_document_text docs/wiki/Troubleshooting.md 'same **Check document** control to retry'
  require_document_text docs/wiki/Troubleshooting.md '## Citation Cannot Be Resolved'
  require_document_text docs/user/WORKSPACE.md '## Add A Reference And Citation'
  require_document_text docs/user/WORKSPACE.md 'setting resets to online when DRAFT restarts'
  require_document_text docs/INVARIANTS.md 'Phase 39 retains each visible typed failure through the presentation boundary.'
}

check_critical_path_documentation() {
  local guide='docs/maintainers/CRITICAL_PATHS.md'

  require_document_text "${guide}" "registered only under \`cfg(test)\`"
  require_document_text "${guide}" 'reopen restores the last committed envelope'
  require_document_text "${guide}" "\`UnsupportedCitation\`"
  require_document_text "${guide}" 'package reopens'
  require_document_text "${guide}" 'adds no application command'
  require_document_text docs/ARCHITECTURE.md 'implemented application through Phase 46'
  require_document_text docs/user/WORKSPACE.md '## Export DOCX'
}

check_packaging_documentation() {
  local guide='docs/maintainers/PACKAGING.md'
  local configuration='docs/maintainers/CONFIGURATION.md'

  require_document_text "${guide}" 'Phase 42 establishes one supported package-build path'
  require_document_text "${guide}" 'npm run package:macos'
  require_document_text "${guide}" 'Darwin arm64'
  require_document_text "${guide}" 'CFBundleIdentifier = com.progentic.draft'
  require_document_text "${guide}" 'It does not produce a signed installer'
  require_document_text "${configuration}" "| Bundle activation | \`true\` |"
  require_document_text "${configuration}" "| Bundle targets | \`app\` only |"
  require_document_text docs/wiki/Current-Limitations.md 'a published download'
  require_document_text README.md 'Versioned downloads will be published on the'
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

check_v1_analysis_decision_state() {
  local adr='docs/adr/002-limit-v1-analysis-to-local-text.md'
  local draft='docs/drafts/V1_LOCAL_ANALYSIS.md'
  local release_contract='docs/maintainers/RELEASE_CANDIDATE.md'
  local decision_files=(
    docs/ARCHITECTURE.md
    docs/contracts/V1_USABILITY_ACCEPTANCE.md
    docs/INVARIANTS.md
    docs/PHASEMAP.md
    docs/ROADMAP.md
    docs/maintainers/AI_ORCHESTRATION.md
    docs/maintainers/DOCUMENTATION_COVERAGE.md
    docs/maintainers/RELEASE_CANDIDATE.md
    docs/maintainers/TEXT_ANALYSIS.md
    docs/maintainers/TOOLCHAIN.md
  )

  require_document_text "${adr}" 'Status: Accepted'
  require_document_text "${adr}" '**Owner Decision: v1.0.0 Analysis Boundary**'
  require_document_text "${adr}" 'For DRAFT v1.0.0, production analysis is limited to local deterministic text'
  require_document_text "${adr}" '## Analysis Layers'
  require_document_text "${adr}" 'permitted v1 findings are exactly:'
  require_document_text "${draft}" '**Status:** Accepted implementation contract'
  require_document_text "${draft}" '**Decision dependency:** Accepted ADR-002'
  require_document_text "${draft}" '## RC-03 Closure Contract'
  require_document_text "${draft}" 'The five permitted user-visible analyses are'
  require_document_text "${draft}" 'remains open until a stable complete packaged'
  require_document_text "${release_contract}" '| RC-03 | Release blocker | Open |'
  for decision_file in "${decision_files[@]}"; do
    require_document_text "${decision_file}" 'ADR-002'
  done
  reject_document_pattern \
    'Status: Proposed|Proposed ADR-002|ADR-002 is under architecture review|\| RC-03 \| Release blocker \| Closed \|' \
    'ADR-002 must remain accepted and RC-03 open after the governed merge' \
    "${adr}" "${draft}" "${decision_files[@]}"
  reject_public_analysis_claims
}

reject_public_analysis_claims() {
  local pattern='(?i)^(?!.*\b(?:not|no|without|unavailable|unimplemented|excluded|outside|deferred|cannot|can\x27t|doesn\x27t|does not|must not|remain absent)\b).*\b(?:AI-powered analysis|semantic analysis|semantic understanding|LLM analysis|generative feedback|originality detection|human-likeness(?: detection)?|AI detection|intelligent assessment|quality assessment|intelligence|reasoning)\b'
  local status

  if rg --line-number --pcre2 "${pattern}" README.md CHANGELOG.md docs/user docs/wiki; then
    echo 'Public documentation claims unsupported model-backed v1 analysis' >&2
    return 1
  else
    status=$?
  fi
  if [[ "${status}" -ne 1 ]]; then
    echo 'Public analysis capability-language scan could not run' >&2
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
  require_document_text "${user_workspace}" 'export remains unavailable pending'
  require_document_text "${user_limits}" 'PDF export is currently unavailable'
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
