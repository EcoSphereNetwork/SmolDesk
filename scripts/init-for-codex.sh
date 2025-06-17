#!/bin/bash
# Initial setup for Codex automation
set -e

echo "[Codex] Installing node and rust dependencies"
npm install
cd src-tauri && cargo fetch && cd ..

mkdir -p docs/docs/components docs/docs/api docs/docs/testing docs/docs/agents docs/docs/development

echo "[Codex] Setup complete"
