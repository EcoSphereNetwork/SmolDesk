// screen_capture/types.rs - Common types and interfaces

use serde::{Deserialize, Serialize};

/// Display server type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

/// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

/// Hardware acceleration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    None,
    VAAPI,
    NVENC,
    QuickSync,
}

/// Latency optimization modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LatencyMode {
    UltraLow,  // Minimal latency, possibly at the expense of quality
    Balanced,  // Balanced ratio between latency and quality
    Quality,   // Higher quality, possibly at the expense of latency
}

/// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: Option<f64>,
    pub primary: bool,
    pub x_offset: i32,
    pub y_offset: i32,
}

/// Statistics for screen capturing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStats {
    pub fps: f64,
    pub bitrate: u64,
    pub encode_time: f64,
    pub frame_size: u64,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub buffer_level: usize,    // Buffer fill level
    pub latency_estimate: f64,  // Estimated latency in ms
}

/// Frame data containing video frame and metadata
#[derive(Debug, Clone)]
pub struct FrameData {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub keyframe: bool,
    pub width: u32,
    pub height: u32,
    pub format: String, // e.g., "h264", "vp8"
}

/// Monitor detection interface
pub trait MonitorDetector {
    fn detect_monitors(&self) -> Result<Vec<MonitorInfo>, crate::screen_capture::error::ScreenCaptureError>;
}

/// Screen capture interface
pub trait ScreenCapturer {
    fn start_capture(&mut self) -> Result<(), crate::screen_capture::error::ScreenCaptureError>;
    fn stop_capture(&mut self) -> Result<(), crate::screen_capture::error::ScreenCaptureError>;
    fn get_next_frame(&mut self) -> Option<FrameData>;
    fn get_stats(&self) -> CaptureStats;
}
