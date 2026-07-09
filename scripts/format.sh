#!/usr/bin/env bash
set -euo pipefail

# Formats supported source files. Optional formatters are reported when absent.

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools cargo

  run_step "Rust format" cargo fmt --all --manifest-path src-tauri/Cargo.toml
  format_bash_if_available
  format_python_if_available
  report_skip "frontend formatter" "not configured in Phase 2"
}

format_bash_if_available() {
  if command -v shfmt >/dev/null 2>&1; then
    run_step "Optional Bash format" shfmt -w scripts/*.sh scripts/lib/*.sh
    return
  fi

  report_skip shfmt "optional tool is not installed"
}

format_python_if_available() {
  if command -v ruff >/dev/null 2>&1; then
    run_step "Optional Python format" ruff format python
    return
  fi

  report_skip ruff "optional tool is not installed"
}

main "$@"
