#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GUI_TAURI_DIR="$ROOT_DIR/capture-bridge/apps/capture-gui/src-tauri"
GUI_FRONTEND_DIR="$ROOT_DIR/capture-bridge/apps/capture-gui/frontend"

install_tools=false
if [[ "${1:-}" == "--install-tools" ]]; then
  install_tools=true
  shift
fi

missing=()

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo is required to run Campus Connect Capture Bridge."
  echo "Install Rust from https://rustup.rs, then run this script again."
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "Campus Connect Capture Bridge desktop GUI needs rustup for the WASM frontend target."
  echo
  echo "Install Rust with rustup:"
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo
  echo "Then run:"
  echo "  ./scripts/capture-bridge-gui.sh --install-tools"
  exit 1
fi

if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  if [[ "$install_tools" == true ]]; then
    rustup target add wasm32-unknown-unknown
  else
    missing+=("wasm32-unknown-unknown target")
  fi
fi

if ! command -v trunk >/dev/null 2>&1; then
  if [[ "$install_tools" == true ]]; then
    cargo install trunk --locked
  else
    missing+=("trunk")
  fi
fi

if ! command -v cargo-tauri >/dev/null 2>&1; then
  if [[ "$install_tools" == true ]]; then
    cargo install tauri-cli --locked
  else
    missing+=("tauri-cli")
  fi
fi

if ((${#missing[@]})); then
  echo "Campus Connect Capture Bridge desktop GUI is implemented, but these dev tools are missing:"
  printf '  - %s\n' "${missing[@]}"
  echo
  echo "Install them automatically with:"
  echo "  ./scripts/capture-bridge-gui.sh --install-tools"
  echo
  echo "Then start the desktop app with:"
  echo "  ./scripts/capture-bridge-gui.sh"
  exit 1
fi

cd "$GUI_TAURI_DIR"
export CAPTURE_BRIDGE_FRONTEND_DIR="$GUI_FRONTEND_DIR"
export NO_COLOR=true
exec cargo tauri dev "$@"
