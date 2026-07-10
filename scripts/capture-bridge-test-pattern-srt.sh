#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/capture-bridge"

cargo run --quiet -p capture-cli -- run --profile program-test-pattern "$@"
