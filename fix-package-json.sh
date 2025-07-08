#!/bin/bash
# fix-package-json.sh - Repariert das package.json Script Problem

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🔧 Repariere package.json Scripts${NC}"
echo "====================================="

# Backup der aktuellen package.json
cp package.json package.json.backup

echo -e "${BLUE}📝 Aktualisiere Scripts-Sektion...${NC}"

# Verwende Node.js um das JSON zu aktualisieren
node << 'EOF'
const fs = require('fs');
const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));

// Aktualisiere die Scripts-Sektion
packageJson.scripts = {
  "dev": "tauri dev",
  "build": "vite build",
  "vite-build": "vite build", 
  "tauri": "tauri build",
  "tauri-build": "tauri build",
  "preview": "vite preview",
  "check": "svelte-check --tsconfig ./tsconfig.json",
  "check:watch": "svelte-check --tsconfig ./tsconfig.json --watch",
  "test": "vitest",
  "test:ui": "vitest --ui",
  "test:run": "vitest run",
  "coverage": "vitest run --coverage",
  "lint": "eslint src --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
  "lint:fix": "eslint src --ext ts,tsx --fix",
  "format": "prettier --write \"src/**/*.{ts,tsx,js,jsx,css,md}\"",
  "format:check": "prettier --check \"src/**/*.{ts,tsx,js,jsx,css,md}\"",
  "typecheck": "tsc --noEmit",
  "clean": "rimraf dist target node_modules/.cache",
  "prepare": "husky install",
  "postinstall": "tauri info",
  "build:deb": "tauri build --target x86_64-unknown-linux-gnu --bundles deb",
  "build:rpm": "tauri build --target x86_64-unknown-linux-gnu --bundles rpm",
  "build:appimage": "tauri build --target x86_64-unknown-linux-gnu --bundles appimage",
  "build:archive": "tauri build --target x86_64-unknown-linux-gnu --bundles archive",
  "build:all": "bash build-all-packages.sh",
  "package:all": "npm run build:all && bash generate-checksums.sh",
  "package:sign": "bash sign-packages.sh",
  "package:test": "bash test-packages.sh",
  "signaling-server": "cd signaling-server && npm install && node index.js",
  "signaling-server:dev": "cd signaling-server && npm install && nodemon index.js",
  "signaling-server:build": "cd signaling-server && npm install --production && tar -czf ../dist/smoldesk-signaling-server-$npm_package_version.tar.gz .",
  "security:scan": "python3 security/pentest/automated_security_scan.py",
  "docs:build": "typedoc --out docs/api src",
  "docs:serve": "serve docs",
  "release": "npm run build:all && npm run package:sign && npm run package:test"
};

// Schreibe die aktualisierte package.json zurück
fs.writeFileSync('package.json', JSON.stringify(packageJson, null, 2) + '\n');

console.log('✅ package.json wurde erfolgreich aktualisiert');
EOF

echo -e "${GREEN}✓${NC} package.json Scripts aktualisiert"

# Teste das build-Skript
echo -e "${BLUE}🧪 Teste build-Skript...${NC}"
if npm run build --silent >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} build-Skript funktioniert"
else
    echo -e "${YELLOW}⚠${NC} build-Skript hat noch Probleme (Dependencies könnten fehlen)"
fi

echo ""
echo -e "${GREEN}🎉 package.json wurde erfolgreich repariert!${NC}"
echo ""
echo "Verfügbare Scripts:"
echo "  npm run build          # Baut das Frontend"
echo "  npm run tauri-build    # Baut die gesamte App"
echo "  npm run dev            # Startet den Dev-Server"
echo ""
echo "Jetzt kannst du erneut versuchen:"
echo "  ./build-all-packages.sh"
echo "  # oder"
echo "  make package"
