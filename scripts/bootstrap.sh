#!/usr/bin/env bash
set -euo pipefail

# Installs locked frontend dependencies, fetches locked Rust crates, and checks
# the Python package. It is safe to run locally and writes no project source.

SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  local root
  root="$(repository_root)"

  require_tools cargo npm python3

  install_frontend_dependencies "${root}"
  fetch_rust_dependencies "${root}"
  check_python_surface "${root}"
}

install_frontend_dependencies() {
  local repository_root="$1"

  npm ci --ignore-scripts --prefix "${repository_root}"
}

fetch_rust_dependencies() {
  local repository_root="$1"

  cargo fetch --locked --manifest-path "${repository_root}/src-tauri/Cargo.toml"
}

check_python_surface() {
  local repository_root="$1"

  PYTHONDONTWRITEBYTECODE=1 PYTHONPATH="${repository_root}/python" \
    python3 -c "import draft_helpers"
}

main "$@"
