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
use input_forwarding::{InputEvent, InputForwarder, detect_display_server, create_input_forwarder, DisplayServer as InputDisplayServer};

// Application state
struct AppState {
    screen_capture: Arc<Mutex<Option<ScreenCaptureManager>>>,
    input_forwarder: Arc<Mutex<Option<Box<dyn InputForwarder>>>>,
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
            
            // Get screen dimensions from first monitor for input forwarder
            let (screen_width, screen_height) = if let Some(manager) = &screen_capture_manager {
                let monitors = manager.get_monitors();
                if !monitors.is_empty() {
                    (monitors[0].width as i32, monitors[0].height as i32)
                } else {
                    (1920, 1080) // Fallback dimensions
                }
            } else {
                (1920, 1080) // Fallback dimensions
            };
            
            // Initialize input forwarder
            let display_server = detect_display_server();
            let input_display_server = match display_server {
                InputDisplayServer::X11 => CaptureDisplayServer::X11,
                InputDisplayServer::Wayland => CaptureDisplayServer::Wayland,
                InputDisplayServer::Unknown => CaptureDisplayServer::Unknown,
            };
            
            let input_forwarder = match create_input_forwarder(display_server, screen_width, screen_height) {
                Ok(forwarder) => Some(forwarder),
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
            get_video_codecs,
            get_hardware_acceleration_options,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
