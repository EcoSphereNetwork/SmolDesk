#!/bin/bash
# simple-build.sh - Einfacher Build mit Tauri direkt

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 SmolDesk Einfacher Build${NC}"
echo "==========================="

# 1. Repariere package.json falls nötig
if ! grep -q '"build":' package.json; then
    echo -e "${BLUE}🔧 Repariere package.json...${NC}"
    chmod +x fix-package-json.sh
    ./fix-package-json.sh
fi

# 2. Installiere Dependencies
echo -e "${BLUE}📦 Installiere Dependencies...${NC}"
npm install

# 3. Baue mit Tauri (das macht automatisch auch das Frontend)
echo -e "${BLUE}🏗️  Starte Tauri Build...${NC}"
echo "Dies kann einige Minuten dauern..."

# Tauri Build mit allen Bundles
npm run tauri-build -- --bundles deb,rpm,appimage

# Prüfe ob Build erfolgreich war
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 Build erfolgreich abgeschlossen!${NC}"
    echo ""
    echo "📦 Erstellte Pakete:"
    find src-tauri/target/release/bundle -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.tar.gz" 2>/dev/null | while read file; do
        echo "  ✓ $file"
    done
    
    # Kopiere Pakete nach dist/
    echo ""
    echo -e "${BLUE}📂 Kopiere Pakete nach dist/...${NC}"
    mkdir -p dist
    find src-tauri/target/release/bundle -name "*.deb" -exec cp {} dist/ \; 2>/dev/null || true
    find src-tauri/target/release/bundle -name "*.rpm" -exec cp {} dist/ \; 2>/dev/null || true
    find src-tauri/target/release/bundle -name "*.AppImage" -exec cp {} dist/ \; 2>/dev/null || true
    find src-tauri/target/release/bundle -name "*.tar.gz" -exec cp {} dist/ \; 2>/dev/null || true
    
    echo ""
    echo -e "${GREEN}📦 Pakete in dist/:${NC}"
    ls -la dist/ 2>/dev/null || echo "Keine Pakete gefunden"
    
    echo ""
    echo -e "${BLUE}💡 Nächste Schritte:${NC}"
    echo "1. Installiere ein Paket:"
    echo "   sudo dpkg -i dist/*.deb"
    echo "   # oder"
    echo "   sudo rpm -i dist/*.rpm"
    echo "   # oder führe die AppImage direkt aus"
    echo ""
    echo "2. Starte SmolDesk:"
    echo "   smoldesk"
    
else
    echo ""
    echo -e "${YELLOW}⚠️  Build hatte Probleme${NC}"
    echo ""
    echo "Mögliche Lösungen:"
    echo "1. Prüfe die Fehlerausgabe oben"
    echo "2. Installiere fehlende Dependencies:"
    echo "   ./install-deps.sh"
    echo "3. Versuche einen einfacheren Build:"
    echo "   npm run tauri-build -- --bundles deb"
fi
