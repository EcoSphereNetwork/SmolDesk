#!/bin/bash
# Initial setup for Codex automation
set -e

echo "[Codex] Installing node and rust dependencies"
# prefer cached packages; fall back to offline mode so Codex works without network
npm install --prefer-offline || npm install --offline || true
cd src-tauri && cargo fetch || true
# try to vendor crates for completely offline builds (requires cargo-vendor)
(cargo vendor > /dev/null && echo "[Codex] cargo vendor complete") || echo "[Codex] cargo vendor skipped"
# place any pre-downloaded *.crate or *.tgz archives in a local vendor/ directory
# to run completely offline
cd ..

mkdir -p docs/docs/components docs/docs/api docs/docs/testing docs/docs/agents docs/docs/development

echo "[Codex] Setup complete"
