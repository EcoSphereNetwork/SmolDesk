#!/bin/bash
# fix-configs.sh - Korrigiert Tauri Konfigurationsfehler

set -e

echo "🔧 Korrigiere Tauri Konfigurationsfehler..."

# 1. Package Manager Konflikt lösen
echo "📦 Löse Package Manager Konflikt..."
if [ -f "yarn.lock" ]; then
    echo "⚠️  Entferne yarn.lock (verwende npm)"
    rm yarn.lock
fi

# 2. Korrigiere src-tauri/Cargo.toml
echo "🦀 Korrigiere Cargo.toml..."

cat > src-tauri/Cargo.toml << 'EOF'
[package]
name = "smoldesk"
version = "1.0.0"
description = "WebRTC-basiertes Remote-Desktop-Tool für Linux mit niedrigen Latenzen und nativer Unterstützung für X11 und Wayland"
authors = ["SmolDesk Team <dev@ecospherenetwork.org>"]
license = "MIT"
repository = "https://github.com/EcoSphereNetwork/SmolDesk.git"
homepage = "https://github.com/EcoSphereNetwork/SmolDesk"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = [
    "api-all",
    "cli",
    "dialog-all",
    "fs-all",
    "global-shortcut-all",
    "http-all",
    "notification-all",
    "os-all",
    "path-all",
    "protocol-asset",
    "shell-open",
    "system-tray",
    "updater",
    "window-all"
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
base64 = "0.21"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
hmac = "0.12"
jsonwebtoken = "9.2"
rand = "0.8"
regex = "1.10"
urlencoding = "2.1"

# Async and concurrency
futures = "0.3"
async-trait = "0.1"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.10"

# System integration
nix = "0.27"

# Image processing (for screen capture)
image = "0.24"

# Configuration management
config = "0.13"
toml = "0.8"

# Platform-specific dependencies - korrekt als optionale Features
x11 = { version = "2.21", optional = true }
wayland-client = { version = "0.31", optional = true }
wayland-protocols = { version = "0.31", optional = true }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

# Platform-specific features - jetzt korrekt definiert
x11-support = ["dep:x11"]
wayland-support = ["dep:wayland-client", "dep:wayland-protocols"]

[profile.dev]
incremental = true

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
EOF

# 3. Korrigiere src-tauri/tauri.conf.json
echo "⚙️  Korrigiere tauri.conf.json..."

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
        "scope": ["https://**", "http://localhost:*"]
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
          "libpipewire-0.3-0"
        ],
        "section": "net",
        "priority": "optional",
        "changelog": "debian/changelog",
        "files": {
          "/usr/share/applications/smoldesk.desktop": "packaging/smoldesk.desktop",
          "/usr/share/pixmaps/smoldesk.png": "docs/static/img/logo.png",
          "/etc/smoldesk/": "packaging/config/"
        }
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
        "epoch": 0,
        "files": {
          "/usr/share/applications/smoldesk.desktop": "packaging/smoldesk.desktop",
          "/usr/share/pixmaps/smoldesk.png": "docs/static/img/logo.png",
          "/etc/smoldesk/": "packaging/config/"
        }
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

# 4. Erstelle fehlende Verzeichnisse
echo "📁 Erstelle fehlende Verzeichnisse..."
mkdir -p packaging/config

# 5. Bereinige Cache und Dependencies
echo "🧹 Bereinige Cache..."
rm -rf node_modules/.cache
rm -rf src-tauri/target/debug
rm -rf dist/*

echo "✅ Konfigurationsfehler korrigiert!"
echo ""
echo "🚀 Jetzt Dependencies neu installieren:"
echo "   npm install"
echo "   cd src-tauri && cargo fetch && cd .."
echo "   make setup"
echo "   make package"
EOF

chmod +x fix-configs.sh
