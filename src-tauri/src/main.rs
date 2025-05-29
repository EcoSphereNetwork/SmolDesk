// src-tauri/src/main.rs

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod screen_capture;
mod input_forwarding;
mod clipboard;
mod connection_security;
mod file_transfer;

use std::sync::{Arc, Mutex};
use tauri::{Manager, Window};
use serde::{Deserialize, Serialize};

use screen_capture::{ScreenCaptureManager, ScreenCaptureConfig, MonitorInfo};
use input_forwarding::{
    InputEvent, 
    forwarder_trait::ImprovedInputForwarder, 
    factory::{detect_display_server, create_improved_input_forwarder},
    types::{InputForwardingConfig, MonitorConfiguration},
    error::InputForwardingError
};
use clipboard::ClipboardManager;
use connection_security::ConnectionSecurityManager;

// Application state
struct AppState {
    screen_capture: Arc<Mutex<Option<ScreenCaptureManager>>>,
    input_forwarder: Arc<Mutex<Option<Box<dyn ImprovedInputForwarder>>>>,
    clipboard_manager: Arc<Mutex<Option<ClipboardManager>>>,
    security_manager: Arc<Mutex<Option<ConnectionSecurityManager>>>,
}

// Commands

#[tauri::command]
fn get_display_server() -> String {
    match detect_display_server() {
        input_forwarding::types::DisplayServer::X11 => "X11".to_string(),
        input_forwarding::types::DisplayServer::Wayland => "Wayland".to_string(),
        input_forwarding::types::DisplayServer::Unknown => "Unknown".to_string(),
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

#[tauri::command]
fn get_clipboard_text(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut clipboard = state.clipboard_manager.lock().unwrap();
    
    if let Some(clipboard_manager) = &mut *clipboard {
        clipboard_manager.get_text()
            .map_err(|e| e.to_string())
    } else {
        Err("Clipboard manager not initialized".to_string())
    }
}

#[tauri::command]
fn set_clipboard_text(text: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut clipboard = state.clipboard_manager.lock().unwrap();
    
    if let Some(clipboard_manager) = &mut *clipboard {
        clipboard_manager.set_text(&text)
            .map_err(|e| e.to_string())
    } else {
        Err("Clipboard manager not initialized".to_string())
    }
}

#[tauri::command]
fn initialize_security(secret_key: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let security_config = connection_security::ConnectionSecurityConfig::default();
    let security_manager = ConnectionSecurityManager::new(&secret_key, security_config);
    
    let mut app_security = state.security_manager.lock().unwrap();
    *app_security = Some(security_manager);
    
    Ok(())
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

            // Initialize clipboard manager
            let clipboard_manager = match detect_display_server() {
                input_forwarding::types::DisplayServer::X11 => {
                    match ClipboardManager::new(screen_capture::types::DisplayServer::X11) {
                        Ok(manager) => Some(manager),
                        Err(e) => {
                            eprintln!("Failed to initialize clipboard manager: {}", e);
                            None
                        }
                    }
                },
                input_forwarding::types::DisplayServer::Wayland => {
                    match ClipboardManager::new(screen_capture::types::DisplayServer::Wayland) {
                        Ok(manager) => Some(manager),
                        Err(e) => {
                            eprintln!("Failed to initialize clipboard manager: {}", e);
                            None
                        }
                    }
                },
                _ => None,
            };
            
            // Create app state
            let state = AppState {
                screen_capture: Arc::new(Mutex::new(screen_capture_manager)),
                input_forwarder: Arc::new(Mutex::new(input_forwarder)),
                clipboard_manager: Arc::new(Mutex::new(clipboard_manager)),
                security_manager: Arc::new(Mutex::new(None)),
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
            get_clipboard_text,
            set_clipboard_text,
            initialize_security,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
