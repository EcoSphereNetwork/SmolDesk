// src-tauri/src/screen_capture.rs - Screen Capture Module

use serde::{Deserialize, Serialize};

pub mod types;
pub mod error;
pub mod config;
pub mod manager;
pub mod buffer;
pub mod quality;
pub mod x11;
pub mod wayland;
pub mod utils;

// Re-export the main components
pub use types::{
    DisplayServer, VideoCodec, HardwareAcceleration, LatencyMode,
    MonitorInfo, CaptureStats
};
pub use config::ScreenCaptureConfig;
pub use error::ScreenCaptureError;
pub use manager::ScreenCaptureManager;

// Legacy compatibility - only keep this struct, remove the duplicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyScreenCaptureConfig {
    pub fps: u32,
    pub quality: u8,
    pub codec: String,
    pub hardware_acceleration: String,
    pub capture_cursor: bool,
    pub capture_audio: bool,
    pub monitor_index: usize,
}

impl From<LegacyScreenCaptureConfig> for ScreenCaptureConfig {
    fn from(legacy: LegacyScreenCaptureConfig) -> Self {
        let codec = match legacy.codec.as_str() {
            "VP8" => VideoCodec::VP8,
            "VP9" => VideoCodec::VP9,
            "AV1" => VideoCodec::AV1,
            _ => VideoCodec::H264,
        };

        let hardware_acceleration = match legacy.hardware_acceleration.as_str() {
            "VAAPI" => HardwareAcceleration::VAAPI,
            "NVENC" => HardwareAcceleration::NVENC,
            "QuickSync" => HardwareAcceleration::QuickSync,
            _ => HardwareAcceleration::None,
        };

        ScreenCaptureConfig {
            monitor_index: legacy.monitor_index,
            fps: legacy.fps,
            quality: legacy.quality as u32,
            codec,
            hardware_acceleration,
            capture_cursor: legacy.capture_cursor,
            capture_audio: legacy.capture_audio,
            keyframe_interval: 30,
            bitrate: None,
            latency_mode: LatencyMode::Balanced,
            advanced_options: None,
        }
    }
}
