#!/bin/bash
# simple environment validation for local Codex runs
set -e

check() {
  command -v "$1" >/dev/null 2>&1 || { echo "[error] $1 not found"; exit 1; }
}

check node
check cargo
check make

pkg-config --exists glib-2.0 || { echo "[error] glib-2.0 not installed"; exit 1; }

if ! command -v vitest >/dev/null 2>&1; then
  echo "[warn] vitest not installed; run scripts/install-vitest.sh"
fi

if ! command -v tauri >/dev/null 2>&1; then
  echo "[warn] tauri CLI missing; install with npm install -g @tauri-apps/cli"
fi

echo "Environment looks OK"
