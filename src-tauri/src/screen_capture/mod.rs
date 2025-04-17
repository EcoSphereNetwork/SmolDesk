// screen_capture/mod.rs - Main module that exports public APIs

pub mod types;
pub mod error;
pub mod config;
pub mod manager;
pub mod buffer;
pub mod quality;
pub mod x11;
pub mod wayland;
pub mod utils;

// Re-export the main components for easier access
pub use types::{
    DisplayServer, VideoCodec, HardwareAcceleration, LatencyMode,
    MonitorInfo, CaptureStats
};
pub use config::ScreenCaptureConfig;
pub use error::ScreenCaptureError;
pub use manager::ScreenCaptureManager;

// This allows the main components to be imported directly:
// use crate::screen_capture::{ScreenCaptureManager, ScreenCaptureConfig, ...};
