#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
if [ ! -f "target/release/bkg-llmserver" ]; then
  cargo build --release
fi
exec ./target/release/bkg-llmserver
