#!/bin/bash
# fix-http-scope.sh - Korrigiert HTTP Scope Problem in tauri.conf.json

set -e

echo "🔧 Korrigiere HTTP Scope Problem..."

# Korrigiere src-tauri/tauri.conf.json mit richtigem HTTP scope
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
          "libpipewire-0.3-0"
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

echo "✅ HTTP Scope Problem behoben!"
echo "🚀 Jetzt nochmal versuchen: npm install"
