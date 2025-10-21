#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
if [ ! -f "target/release/brainml" ]; then
  cargo build --release
fi
PLUGIN_PORT="${PLUGIN_PORT:-}" RUST_LOG="${RUST_LOG:-info}" ./target/release/brainml
