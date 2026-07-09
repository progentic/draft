#!/usr/bin/env bash
set -euo pipefail

# Builds the frontend and checks the Rust desktop crate without network access.

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/common.sh
source "${SCRIPT_DIRECTORY}/lib/common.sh"

main() {
  cd "$(repository_root)"
  require_tools cargo npm
  require_file package-lock.json
  require_file src-tauri/Cargo.lock

  run_step "TypeScript typecheck" npm run typecheck
  run_step "Frontend production build" npm run build:frontend
  run_step "Rust compile check" cargo check --locked --offline --manifest-path src-tauri/Cargo.toml
}

main "$@"
