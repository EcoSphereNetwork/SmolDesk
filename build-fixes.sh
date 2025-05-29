#!/bin/bash
# build-fixes.sh - Erstellt fehlende Dateien für SmolDesk Build

set -e

echo "🔧 Erstelle fehlende Dateien für SmolDesk Build..."

# 1. Frontend Entry Points erstellen
echo "📝 Erstelle Frontend Entry Points..."

cat > src/main.tsx << 'EOF'
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
EOF

cat > index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SmolDesk</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
EOF

cat > vite.config.ts << 'EOF'
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: ["es2021", "chrome100", "safari13"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
}));
EOF

cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
EOF

cat > tsconfig.node.json << 'EOF'
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
EOF

# 2. Basis CSS
cat > src/styles.css << 'EOF'
:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color-scheme: light dark;
  color: rgba(255, 255, 255, 0.87);
  background-color: #242424;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  display: flex;
  place-items: center;
  min-width: 320px;
  min-height: 100vh;
}

#root {
  max-width: 1280px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
}

.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background: #1a1a1a;
  padding: 1rem;
  border-bottom: 1px solid #333;
}

.app-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.sidebar {
  width: 250px;
  background: #2a2a2a;
  padding: 1rem;
  transition: transform 0.3s ease;
}

.sidebar.hidden {
  transform: translateX(-100%);
}

.main-content {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
}

button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  background-color: #1a1a1a;
  color: white;
  cursor: pointer;
  transition: border-color 0.25s;
}

button:hover {
  border-color: #646cff;
}

button:focus,
button:focus-visible {
  outline: 4px auto -webkit-focus-ring-color;
}

@media (prefers-color-scheme: light) {
  :root {
    color: #213547;
    background-color: #ffffff;
  }
  
  button {
    background-color: #f9f9f9;
    color: #213547;
  }
}
EOF

# 3. Rust Backend Module Stubs erstellen
echo "🦀 Erstelle Rust Backend Module..."

mkdir -p src-tauri/src

cat > src-tauri/src/screen_capture.rs << 'EOF'
// src-tauri/src/screen_capture.rs - Screen Capture Module

use serde::{Deserialize, Serialize};
use tauri::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    pub fps: u32,
    pub quality: u8,
    pub codec: String,
    pub hardware_acceleration: String,
    pub capture_cursor: bool,
    pub capture_audio: bool,
    pub monitor_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub refresh_rate: Option<u32>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    None,
    VAAPI,
    NVENC,
    QuickSync,
}

pub struct ScreenCaptureManager {
    config: Option<ScreenCaptureConfig>,
    is_capturing: bool,
}

impl ScreenCaptureManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: None,
            is_capturing: false,
        })
    }

    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        // Stub implementation - würde echte Monitor-Erkennung implementieren
        vec![
            MonitorInfo {
                index: 0,
                name: "Primary Monitor".to_string(),
                width: 1920,
                height: 1080,
                x_offset: 0,
                y_offset: 0,
                refresh_rate: Some(60),
                primary: true,
            }
        ]
    }

    pub fn update_config(&mut self, config: ScreenCaptureConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Some(config);
        Ok(())
    }

    pub fn start_capture(&mut self, _window: Window) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_none() {
            return Err("No configuration set".into());
        }
        
        self.is_capturing = true;
        println!("Screen capture started (stub implementation)");
        Ok(())
    }

    pub fn stop_capture(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_capturing = false;
        println!("Screen capture stopped");
        Ok(())
    }
}
EOF

cat > src-tauri/src/input_forwarding.rs << 'EOF'
// src-tauri/src/input_forwarding.rs - Input Forwarding Module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub button: Option<MouseButton>,
    pub key_code: Option<u32>,
    pub modifiers: Option<Vec<String>>,
    pub is_pressed: Option<bool>,
    pub delta_x: Option<f64>,
    pub delta_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
}

pub mod forwarder_trait {
    use super::*;

    pub trait ImprovedInputForwarder: Send + Sync {
        fn forward_event(&self, event: &InputEvent) -> Result<(), Box<dyn std::error::Error>>;
        fn set_enabled(&self, enabled: bool);
        fn configure_monitors(&mut self, monitors: Vec<types::MonitorConfiguration>) -> Result<(), error::InputForwardingError>;
    }
}

pub mod factory {
    use super::*;
    use super::forwarder_trait::ImprovedInputForwarder;

    #[derive(Debug, Clone)]
    pub enum DisplayServer {
        X11,
        Wayland,
        Unknown,
    }

    pub fn detect_display_server() -> DisplayServer {
        // Stub implementation
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            DisplayServer::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            DisplayServer::X11
        } else {
            DisplayServer::Unknown
        }
    }

    pub fn create_improved_input_forwarder(
        _display_server: Option<DisplayServer>
    ) -> Result<Box<dyn ImprovedInputForwarder>, error::InputForwardingError> {
        Ok(Box::new(StubInputForwarder::new()))
    }

    struct StubInputForwarder {
        enabled: bool,
    }

    impl StubInputForwarder {
        fn new() -> Self {
            Self { enabled: false }
        }
    }

    impl ImprovedInputForwarder for StubInputForwarder {
        fn forward_event(&self, event: &InputEvent) -> Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                println!("Input event: {:?}", event);
            }
            Ok(())
        }

        fn set_enabled(&self, _enabled: bool) {
            // Stub implementation
        }

        fn configure_monitors(&mut self, _monitors: Vec<types::MonitorConfiguration>) -> Result<(), error::InputForwardingError> {
            Ok(())
        }
    }
}

pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone)]
    pub enum DisplayServer {
        X11,
        Wayland,
        Unknown,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InputForwardingConfig {
        pub enable_multi_monitor: bool,
        pub monitors: Vec<MonitorConfiguration>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MonitorConfiguration {
        pub index: usize,
        pub x_offset: i32,
        pub y_offset: i32,
        pub width: i32,
        pub height: i32,
        pub scale_factor: f64,
        pub is_primary: bool,
    }
}

pub mod error {
    use std::fmt;

    #[derive(Debug)]
    pub enum InputForwardingError {
        InitializationFailed(String),
        ConfigurationError(String),
        SendError(String),
    }

    impl fmt::Display for InputForwardingError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                InputForwardingError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
                InputForwardingError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
                InputForwardingError::SendError(msg) => write!(f, "Send error: {}", msg),
            }
        }
    }

    impl std::error::Error for InputForwardingError {}
}
EOF

# 4. Icons erstellen (einfache SVG-basierte PNGs)
echo "🎨 Erstelle Icons..."

mkdir -p icons docs/static/img

# Erstelle ein einfaches SVG Icon
cat > /tmp/smoldesk-icon.svg << 'EOF'
<svg width="128" height="128" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
  <rect width="128" height="128" rx="20" fill="#2563eb"/>
  <rect x="20" y="30" width="88" height="58" rx="8" fill="white"/>
  <rect x="28" y="38" width="72" height="42" fill="#1e40af"/>
  <circle cx="64" cy="100" r="8" fill="white"/>
  <rect x="56" y="88" width="16" height="12" fill="white"/>
  <text x="64" y="68" text-anchor="middle" font-family="Arial" font-size="24" font-weight="bold" fill="white">SD</text>
</svg>
EOF

# Konvertiere SVG zu PNG (falls ImageMagick/convert verfügbar ist)
if command -v convert >/dev/null 2>&1; then
    echo "📷 Konvertiere SVG zu PNG Icons..."
    convert /tmp/smoldesk-icon.svg -resize 32x32 icons/32x32.png
    convert /tmp/smoldesk-icon.svg -resize 128x128 icons/128x128.png
    convert /tmp/smoldesk-icon.svg -resize 256x256 icons/128x128@2x.png
    cp icons/128x128.png docs/static/img/logo.png
    
    # ICO erstellen (falls möglich)
    if command -v magick >/dev/null 2>&1; then
        magick /tmp/smoldesk-icon.svg -resize 32x32 icons/icon.ico
    else
        cp icons/32x32.png icons/icon.ico 2>/dev/null || echo "⚠️  ICO-Erstellung übersprungen"
    fi
    
    # ICNS für macOS (falls möglich)
    if command -v png2icns >/dev/null 2>&1; then
        png2icns icons/icon.icns icons/128x128.png
    else
        cp icons/128x128.png icons/icon.icns 2>/dev/null || echo "⚠️  ICNS-Erstellung übersprungen"
    fi
else
    echo "⚠️  ImageMagick nicht gefunden. Erstelle Placeholder-Dateien..."
    # Erstelle leere Placeholder-Dateien
    touch icons/32x32.png icons/128x128.png icons/128x128@2x.png icons/icon.ico icons/icon.icns
    touch docs/static/img/logo.png
fi

# 5. Desktop-Datei erstellen
mkdir -p packaging

cat > packaging/smoldesk.desktop << 'EOF'
[Desktop Entry]
Name=SmolDesk
Comment=WebRTC Remote Desktop
GenericName=Remote Desktop
Exec=smoldesk
Icon=smoldesk
Terminal=false
Type=Application
Categories=Network;RemoteAccess;
MimeType=x-scheme-handler/smoldesk;
Keywords=remote;desktop;screen;sharing;webrtc;
StartupNotify=true
StartupWMClass=SmolDesk
EOF

# 6. Fehlende Verzeichnisse erstellen
mkdir -p dist
mkdir -p target/release/bundle

echo "✅ Fehlende Dateien erstellt!"
echo ""
echo "🚀 Jetzt kannst du den Build starten:"
echo "   make setup"
echo "   make build"
echo "   make package"
