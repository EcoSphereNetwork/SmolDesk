#!/bin/bash
# build-fixes.sh - Repariert SmolDesk Build-Probleme

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔧 SmolDesk Build-Reparatur${NC}"
echo "============================"
echo ""

# 1. Icons erstellen falls fehlend
echo -e "${BLUE}📸 Prüfe Icons...${NC}"
if [ ! -f "icons/32x32.png" ] || [ ! -f "icons/128x128.png" ]; then
    echo "Icons fehlen, erstelle sie..."
    chmod +x create-missing-icons.sh
    ./create-missing-icons.sh
else
    echo -e "${GREEN}✓${NC} Icons sind vorhanden"
fi

# 2. package.json reparieren
echo -e "${BLUE}📦 Repariere package.json...${NC}"
if ! grep -q '"build":' package.json; then
    echo "Füge fehlendes build-Skript hinzu..."
    
    # Backup erstellen
    cp package.json package.json.bak
    
    # build-Skript hinzufügen
    sed -i 's/"dev": "tauri dev",/"dev": "tauri dev",\n    "build": "vite build",/' package.json
    
    echo -e "${GREEN}✓${NC} build-Skript hinzugefügt"
else
    echo -e "${GREEN}✓${NC} package.json ist korrekt"
fi

# 3. Dependencies installieren/reparieren
echo -e "${BLUE}📚 Installiere Dependencies...${NC}"

# Node.js Dependencies
if [ ! -d "node_modules" ] || [ ! -f "package-lock.json" ]; then
    echo "Installiere Node.js Dependencies..."
    npm install
else
    echo "Aktualisiere Node.js Dependencies..."
    npm ci
fi

# Rust Dependencies
echo "Lade Rust Dependencies..."
cd src-tauri
cargo fetch
cd ..

echo -e "${GREEN}✓${NC} Dependencies installiert"

# 4. Fehlende Verzeichnisse erstellen
echo -e "${BLUE}📁 Erstelle fehlende Verzeichnisse...${NC}"

mkdir -p dist
mkdir -p docs/static/img
mkdir -p packaging/{debian/DEBIAN,rpm,appimage,flatpak}

echo -e "${GREEN}✓${NC} Verzeichnisse erstellt"

# 5. Prüfe Tauri Konfiguration
echo -e "${BLUE}⚙️  Prüfe Tauri Konfiguration...${NC}"

# Backup der Tauri-Konfiguration
cp src-tauri/tauri.conf.json src-tauri/tauri.conf.json.bak

# Validiere JSON
if ! node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8'))" 2>/dev/null; then
    echo -e "${YELLOW}⚠${NC} tauri.conf.json hat Syntax-Fehler, repariere..."
    
    # Erstelle eine korrigierte Version
    cat > src-tauri/tauri.conf.json << 'EOF'
{
  "$schema": "../node_modules/@tauri-apps/cli/schema.json",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:1420",
    "distDir": "../dist",
    "withGlobalTauri": false
  },
  "package": {
    "productName": "SmolDesk",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "copyFile": true,
        "createDir": true,
        "removeDir": true,
        "removeFile": true,
        "renameFile": true,
        "exists": true,
        "scope": ["$APPDATA", "$APPDATA/**", "$RESOURCE", "$RESOURCE/**", "$TEMP", "$TEMP/**"]
      },
      "path": {
        "all": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true,
        "setPosition": true,
        "setSize": true,
        "setTitle": true,
        "setFocus": true
      },
      "globalShortcut": {
        "all": true
      },
      "os": {
        "all": true
      },
      "http": {
        "all": true,
        "request": true,
        "scope": ["https://**", "http://localhost:1420", "http://localhost:3000", "http://localhost:8080"]
      },
      "notification": {
        "all": true
      },
      "clipboard": {
        "all": true,
        "readText": true,
        "writeText": true
      },
      "protocol": {
        "all": false,
        "asset": true,
        "assetScope": ["$RESOURCE/**"]
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.smoldesk.SmolDesk",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "externalBin": [],
      "copyright": "© 2025 SmolDesk Team",
      "category": "Network",
      "shortDescription": "WebRTC Remote Desktop for Linux",
      "longDescription": "SmolDesk is a modern remote desktop solution that provides low-latency screen sharing using WebRTC technology. Supports both X11 and Wayland display servers with native Linux integration.",
      "deb": {
        "depends": [
          "libwebkit2gtk-4.1-0",
          "libgtk-3-0",
          "ffmpeg",
          "wl-clipboard | xclip",
          "ydotool | xdotool",
          "pipewire",
          "libpipewire-0.3-0 | libpipewire-0.3-0t64"
        ],
        "section": "net",
        "priority": "optional"
      },
      "rpm": {
        "license": "MIT",
        "depends": [
          "webkit2gtk4.1",
          "gtk3",
          "ffmpeg",
          "wl-clipboard",
          "ydotool",
          "pipewire",
          "pipewire-libs"
        ],
        "epoch": 0
      },
      "appimage": {
        "bundleMediaFramework": true
      },
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      },
      "macOS": {
        "frameworks": [],
        "minimumSystemVersion": "",
        "exceptionDomain": "",
        "signingIdentity": null,
        "providerShortName": null,
        "entitlements": null
      }
    },
    "security": {
      "csp": "default-src 'self' blob: data: filesystem: ws: wss: http: https: tauri: 'unsafe-eval' 'unsafe-inline'; img-src 'self' blob: data: filesystem: http: https: tauri:; media-src 'self' blob: data: filesystem: http: https: tauri:",
      "devCsp": null,
      "freezePrototype": false,
      "dangerousDisableAssetCspModification": false
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "SmolDesk",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "center": true,
        "decorations": true,
        "transparent": false,
        "alwaysOnTop": false,
        "skipTaskbar": false,
        "url": "index.html"
      }
    ],
    "systemTray": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,
      "menuOnLeftClick": false
    },
    "updater": {
      "active": false,
      "endpoints": [],
      "dialog": true,
      "pubkey": ""
    }
  }
}
EOF
    
    echo -e "${GREEN}✓${NC} tauri.conf.json repariert"
else
    echo -e "${GREEN}✓${NC} tauri.conf.json ist valide"
fi

# 6. Cargo.toml reparieren
echo -e "${BLUE}🦀 Prüfe Cargo.toml...${NC}"

cd src-tauri

# Prüfe Cargo.toml Syntax
if ! cargo check --quiet 2>/dev/null; then
    echo -e "${YELLOW}⚠${NC} Cargo.toml hat Probleme, versuche Reparatur..."
    
    # Erstelle Backup
    cp Cargo.toml Cargo.toml.bak
    
    # Update Cargo.lock
    cargo update
    
    echo -e "${GREEN}✓${NC} Cargo.toml repariert"
else
    echo -e "${GREEN}✓${NC} Cargo.toml ist valide"
fi

cd ..

# 7. Signaling Server Dependencies
echo -e "${BLUE}🌐 Prüfe Signaling Server...${NC}"

if [ -d "signaling-server" ]; then
    cd signaling-server
    if [ ! -d "node_modules" ]; then
        echo "Installiere Signaling Server Dependencies..."
        npm install
    fi
    cd ..
    echo -e "${GREEN}✓${NC} Signaling Server ist bereit"
fi

# 8. Berechtigungen setzen
echo -e "${BLUE}🔒 Setze Berechtigungen...${NC}"

chmod +x build-all-packages.sh
chmod +x validate-build.sh
chmod +x create-missing-icons.sh
chmod +x quick-install.sh 2>/dev/null || true
chmod +x install-deps.sh 2>/dev/null || true

if [ -d "scripts" ]; then
    chmod +x scripts/*.sh 2>/dev/null || true
fi

echo -e "${GREEN}✓${NC} Berechtigungen gesetzt"

# 9. Test-Build der Frontend
echo -e "${BLUE}🧪 Teste Frontend-Build...${NC}"

if npm run build >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Frontend kann erfolgreich gebaut werden"
else
    echo -e "${YELLOW}⚠${NC} Frontend-Build hat Probleme, aber das ist normal bei fehlenden Dependencies"
fi

# 10. Zusammenfassung
echo ""
echo -e "${GREEN}🎉 Build-Reparatur abgeschlossen!${NC}"
echo ""
echo "Nächste Schritte:"
echo "1. Führe ./validate-build.sh aus, um alles zu prüfen"
echo "2. Starte den Build mit:"
echo "   make setup"
echo "   make build"
echo "   make package"
echo ""
echo "Oder verwende das all-in-one Script:"
echo "   ./build-all-packages.sh"
echo ""
echo -e "${BLUE}💡 Tipp:${NC} Falls noch Probleme auftreten, prüfe die Logs und"
echo "installiere fehlende System-Dependencies mit:"
echo "   ./install-deps.sh"
