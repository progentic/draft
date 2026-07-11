#!/usr/bin/env bash
set -euo pipefail

# Runs the complete local health check. It writes only ignored compiler
# and test output inside the repository plus standard tool-managed caches.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools bash cargo find git grep node npm python3 rg
  require_bootstrapped_dependencies

  run_step "Frontend dependency tree" npm ls --depth=0
  run_step "Frontend tests" npm test
  run_step "Rust formatting" cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
  run_step "Rust lint" cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
  run_step "Locked application builds" bash scripts/build.sh
  run_step "Rust tests" cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
  run_step "Python tests" run_python_tests
  run_step "Bash syntax" find scripts -type f -name "*.sh" -exec bash -n "{}" +
  run_optional_checks
  run_step "Packaging contract" bash scripts/check-packaging.sh
  run_step "Release-candidate hardening" bash scripts/check-release-candidate.sh
  run_step "Invariant boundaries" bash scripts/check-invariants.sh
  run_step "CI/local parity" bash scripts/check-ci-local-parity.sh
  run_step "Documentation sanity" bash scripts/check-docs.sh
  run_step "Repository hygiene" bash scripts/check-repository.sh

  printf '\nDRAFT local verification passed.\n'
}

require_bootstrapped_dependencies() {
  require_file package-lock.json
  require_file src-tauri/Cargo.lock

  if [[ ! -x node_modules/.bin/tsc || ! -x node_modules/.bin/vite ]]; then
    echo "Frontend dependencies are missing. Run: bash scripts/bootstrap.sh" >&2
    return 1
  fi
}

run_python_tests() {
  PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=python \
    python3 -m unittest discover -s python/tests -v
}

run_optional_checks() {
  local script_files=(scripts/*.sh scripts/lib/*.sh)

  run_optional_shellcheck "${script_files[@]}"
  run_optional_shfmt "${script_files[@]}"
  run_optional_ruff
  report_optional_just
  report_skip "frontend formatter/linter" "not configured; tests, TypeScript, and production builds are required"
}

run_optional_shellcheck() {
  if command -v shellcheck >/dev/null 2>&1; then
    run_step "Optional ShellCheck" shellcheck "$@"
    return
  fi

  report_skip shellcheck "optional tool is not installed"
}

run_optional_shfmt() {
  if command -v shfmt >/dev/null 2>&1; then
    run_step "Optional shfmt check" shfmt -d "$@"
    return
  fi

  report_skip shfmt "optional tool is not installed"
}

run_optional_ruff() {
  if command -v ruff >/dev/null 2>&1; then
    run_step "Optional Ruff format" ruff format --check python
    run_step "Optional Ruff lint" ruff check python
    return
  fi

  report_skip ruff "optional tool is not installed"
}

report_optional_just() {
  if command -v just >/dev/null 2>&1; then
    printf '==> Optional task runner available: just\n'
    return
  fi

  report_skip just "use bash scripts/verify.sh as the equivalent entry point"
}

main "$@"
