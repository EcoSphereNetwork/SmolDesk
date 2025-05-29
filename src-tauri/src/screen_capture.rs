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
