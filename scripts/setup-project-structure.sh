#!/bin/bash

# Skript zur Erstellung der SmolDesk-Projektstruktur
# Erstellt Ordner und leere Dateien gemäß der spezifizierten Struktur

# Hauptverzeichnis erstellen (falls es noch nicht existiert)
mkdir -p SmolDesk

# In das Hauptverzeichnis wechseln
cd SmolDesk

# GitHub-Konfigurationen und Workflows
mkdir -p .github/workflows
touch .github/workflows/ci.yml
touch .github/workflows/release.yml
touch .github/ISSUE_TEMPLATE.md
touch .github/PULL_REQUEST_TEMPLATE.md

# Projektdokumentation
mkdir -p docs/api docs/user docs/technical docs/static/img
touch docs/README.md
touch docs/api/README.md
touch docs/user/README.md
touch docs/technical/README.md
# Erstellen eines Platzhalters für das Logo
touch docs/static/img/logo.png

# Frontend-Quellcode (React)
mkdir -p src/components src/hooks src/utils src/contexts
touch src/main.tsx
touch src/App.tsx
touch src/styles.css
touch src/components/ConnectionManager.tsx
touch src/components/RemoteScreen.tsx
touch src/hooks/useWebRTC.ts
touch src/utils/webrtc.ts
touch src/contexts/ConnectionContext.tsx

# Backend-Quellcode (Rust)
mkdir -p src-tauri/src
touch src-tauri/Cargo.toml
touch src-tauri/tauri.conf.json
touch src-tauri/build.rs
touch src-tauri/src/main.rs
touch src-tauri/src/screen_capture.rs
touch src-tauri/src/input_forwarding.rs

# WebRTC Signaling-Server
mkdir -p signaling-server
touch signaling-server/index.js
touch signaling-server/package.json
touch signaling-server/Dockerfile

# Testsuite
mkdir -p tests/unit tests/integration tests/e2e
touch tests/unit/webrtc.test.ts
touch tests/unit/screen-capture.test.ts
touch tests/integration/connection.test.ts
touch tests/e2e/remote-control.test.ts

# Root-Dateien
touch package.json
touch README.md
touch .gitignore
touch LICENSE
touch CONTRIBUTING.md

echo "Die SmolDesk-Projektstruktur wurde erfolgreich erstellt!"
echo "Strukturübersicht:"
find . -type d | sort | sed -e "s/[^-][^\/]*\// |  /g" -e "s/|\([^ ]\)/| - \1/"

echo "Sie können jetzt mit der Entwicklung beginnen."
