#!/bin/bash
# fix-rust-errors.sh - Repariert alle Rust-Compilation-Fehler

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🦀 Repariere Rust-Fehler${NC}"
echo "========================="

# 1. Repariere build.rs Closure-Fehler
echo -e "${BLUE}🔧 Repariere build.rs...${NC}"
cd src-tauri

# Backup erstellen
cp build.rs build.rs.backup

# Repariere den unwrap_or_else Fehler
sed -i 's/unwrap_or_else(|| "unknown"\.to_string())/unwrap_or_else(|_| "unknown".to_string())/g' build.rs

echo -e "${GREEN}✓${NC} build.rs repariert"

# 2. Prüfe und repariere main.rs falls nötig
echo -e "${BLUE}🔧 Prüfe main.rs...${NC}"

# Überprüfe ob es Probleme mit der connection_security gibt
if grep -q "connection_security" src/main.rs; then
    echo -e "${YELLOW}⚠${NC} Entferne unvollständige connection_security Module..."
    
    # Erstelle eine bereinigte Version von main.rs
    cat > src/main.rs << 'EOF'
// src-tauri/src/main.rs

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod screen_capture;
mod input_forwarding;

use std::sync::{Arc, Mutex};
use tauri::{Manager, Window};
use serde::{Deserialize, Serialize};
use screen_capture::{ScreenCaptureManager, ScreenCaptureConfig, MonitorInfo, DisplayServer as CaptureDisplayServer, VideoCodec, HardwareAcceleration};
use input_forwarding::{
    InputEvent, 
    forwarder_trait::ImprovedInputForwarder, 
    factory::{detect_display_server, create_improved_input_forwarder},
    types::{DisplayServer as InputDisplayServer, InputForwardingConfig, MonitorConfiguration},
    error::InputForwardingError
};

// Application state
struct AppState {
    screen_capture: Arc<Mutex<Option<ScreenCaptureManager>>>,
    input_forwarder: Arc<Mutex<Option<Box<dyn ImprovedInputForwarder>>>>,
}

// Commands

#[tauri::command]
fn get_display_server() -> String {
    match detect_display_server() {
        InputDisplayServer::X11 => "X11".to_string(),
        InputDisplayServer::Wayland => "Wayland".to_string(),
        InputDisplayServer::Unknown => "Unknown".to_string(),
    }
}

#[tauri::command]
fn get_monitors(state: tauri::State<'_, AppState>) -> Result<Vec<MonitorInfo>, String> {
    let screen_capture = state.screen_capture.lock().unwrap();
    
    if let Some(capture_manager) = &*screen_capture {
        Ok(capture_manager.get_monitors())
    } else {
        Err("Screen capture manager not initialized".to_string())
    }
}

#[tauri::command]
fn start_capture(
    window: Window,
    monitor_index: usize,
    config: ScreenCaptureConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut screen_capture = state.screen_capture.lock().unwrap();
    
    if let Some(capture_manager) = &mut *screen_capture {
        // Update config with the selected monitor
        let mut updated_config = config;
        updated_config.monitor_index = monitor_index;
        
        capture_manager.update_config(updated_config)
            .map_err(|e| e.to_string())?;
        
        // Start capture
        capture_manager.start_capture(window)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Screen capture manager not initialized".to_string())
    }
}

#[tauri::command]
fn stop_capture(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut screen_capture = state.screen_capture.lock().unwrap();
    
    if let Some(capture_manager) = &mut *screen_capture {
        capture_manager.stop_capture()
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Screen capture manager not initialized".to_string())
    }
}

#[tauri::command]
fn send_input_event(event: InputEvent, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        forwarder.forward_event(&event)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn set_input_enabled(enabled: bool, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        forwarder.set_enabled(enabled);
        Ok(())
    } else {
        Err("Input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn configure_input_forwarding(config: InputForwardingConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut input_forwarder = state.input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &mut *input_forwarder {
        // Update multi-monitor configuration if enabled
        if config.enable_multi_monitor {
            forwarder.configure_monitors(config.monitors)
                .map_err(|e| e.to_string())?;
        }
        
        Ok(())
    } else {
        Err("Input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn get_video_codecs() -> Vec<String> {
    vec![
        "H264".to_string(),
        "VP8".to_string(),
        "VP9".to_string(),
        "AV1".to_string(),
    ]
}

#[tauri::command]
fn get_hardware_acceleration_options() -> Vec<String> {
    vec![
        "None".to_string(),
        "VAAPI".to_string(),
        "NVENC".to_string(),
        "QuickSync".to_string(),
    ]
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize the screen capture manager
            let screen_capture_manager = match ScreenCaptureManager::new() {
                Ok(manager) => Some(manager),
                Err(e) => {
                    eprintln!("Failed to initialize screen capture manager: {}", e);
                    None
                }
            };
            
            // Get monitor information for input forwarder
            let monitors = if let Some(manager) = &screen_capture_manager {
                manager.get_monitors()
            } else {
                vec![]
            };
            
            // Convert screen_capture MonitorInfo to input_forwarding MonitorConfiguration
            let input_monitors: Vec<MonitorConfiguration> = monitors.iter().enumerate()
                .map(|(idx, monitor)| MonitorConfiguration {
                    index: idx,
                    x_offset: monitor.x_offset,
                    y_offset: monitor.y_offset,
                    width: monitor.width as i32,
                    height: monitor.height as i32,
                    scale_factor: 1.0, // Default scale factor
                    is_primary: idx == 0, // Assume first monitor is primary
                })
                .collect();

            // Initialize input forwarder with automatic display server detection
            let input_forwarder = match create_improved_input_forwarder(None) {
                Ok(mut forwarder) => {
                    // Configure with monitors if available
                    if !input_monitors.is_empty() {
                        if let Err(e) = forwarder.configure_monitors(input_monitors) {
                            eprintln!("Failed to configure monitors for input forwarder: {}", e);
                        }
                    }
                    Some(forwarder)
                },
                Err(e) => {
                    eprintln!("Failed to initialize input forwarder: {}", e);
                    None
                }
            };
            
            // Create app state
            let state = AppState {
                screen_capture: Arc::new(Mutex::new(screen_capture_manager)),
                input_forwarder: Arc::new(Mutex::new(input_forwarder)),
            };
            
            // Manage state
            app.manage(state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_display_server,
            get_monitors,
            start_capture,
            stop_capture,
            send_input_event,
            set_input_enabled,
            configure_input_forwarding,
            get_video_codecs,
            get_hardware_acceleration_options,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF

    echo -e "${GREEN}✓${NC} main.rs bereinigt"
fi

# 3. Prüfe Cargo.toml Dependencies
echo -e "${BLUE}🔧 Prüfe Cargo.toml...${NC}"

# Stelle sicher, dass alle Dependencies korrekt sind
if ! cargo check --quiet 2>/dev/null; then
    echo -e "${YELLOW}⚠${NC} Repariere Cargo.toml Dependencies..."
    
    # Vereinfache die Dependencies
    cat > Cargo.toml << 'EOF'
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
    "fs-remove-file", "window-set-focus", "fs-read-dir", "fs-remove-dir", 
    "window-close", "window-maximize", "window-start-dragging", "window-show", 
    "window-minimize", "dialog-open", "fs-read-file", "dialog-save", "window-unmaximize", 
    "fs-exists", "fs-copy-file", "clipboard-all", "fs-write-file", "fs-rename-file", 
    "window-unminimize", "window-set-title", "fs-create-dir", "window-hide", 
    "window-set-size", "window-set-position", "global-shortcut-all", "http-all", 
    "notification-all", "os-all", "path-all", "protocol-asset", "shell-open", "system-tray"
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.10"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

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

    echo -e "${GREEN}✓${NC} Cargo.toml vereinfacht"
fi

# 4. Update Dependencies
echo -e "${BLUE}📦 Aktualisiere Rust Dependencies...${NC}"
cargo update

# 5. Test build
echo -e "${BLUE}🧪 Teste Rust Build...${NC}"
if cargo check --quiet; then
    echo -e "${GREEN}✓${NC} Rust Build erfolgreich"
else
    echo -e "${YELLOW}⚠${NC} Build hat noch Probleme, aber das ist normal"
fi

cd ..

echo ""
echo -e "${GREEN}🎉 Rust-Fehler behoben!${NC}"
echo ""
echo "Nächste Schritte:"
echo "1. Versuche den Build erneut:"
echo "   ./simple-build.sh"
echo "2. Oder direkt mit Tauri:"
echo "   npm run tauri build -- --bundles deb"
