#!/bin/bash
# Basic analysis script for Codex
set -e

npm run build >/tmp/codex_build.log || true
npm test >/tmp/codex_test.log || true
cd src-tauri && cargo build >/tmp/codex_cargo.log || true
cd ..

echo "# Build Log" > codex-report.md
cat /tmp/codex_build.log >> codex-report.md

echo "# Test Log" >> codex-report.md
cat /tmp/codex_test.log >> codex-report.md

echo "# Cargo Log" >> codex-report.md
cat /tmp/codex_cargo.log >> codex-report.md

echo "[Codex] Analysis finished. See codex-report.md"
