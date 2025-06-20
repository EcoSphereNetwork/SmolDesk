// screen_capture/config.rs - Configuration structures

use serde::{Deserialize, Serialize};
use crate::screen_capture::types::{VideoCodec, HardwareAcceleration, LatencyMode};

/// Screen capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    /// Index of the monitor to capture
    pub monitor_index: usize,
    
    /// Target frames per second
    pub fps: u32,
    
    /// Quality setting (0-100)
    pub quality: u32,
    
    /// Video codec to use
    pub codec: VideoCodec,
    
    /// Hardware acceleration method
    pub hardware_acceleration: HardwareAcceleration,
    
    /// Whether to capture the cursor
    pub capture_cursor: bool,
    
    /// Whether to capture audio
    pub capture_audio: bool,
    
    /// Keyframe interval for better compression
    pub keyframe_interval: u32,
    
    /// Optional bitrate in Kbps (overrides quality-based bitrate estimation)
    pub bitrate: Option<u32>,
    
    /// Latency optimization mode
    pub latency_mode: LatencyMode,
    
    /// Advanced FFmpeg options (optional)
    pub advanced_options: Option<AdvancedEncodingOptions>,
}

/// Advanced encoding options for FFmpeg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedEncodingOptions {
    /// Pixel format (e.g., "yuv420p")
    pub pixel_format: String,
    
    /// Preset for x264/x265 encoders (e.g., "ultrafast", "medium", "slow")
    pub preset: Option<String>,
    
    /// Tune parameter for x264/x265 encoders (e.g., "zerolatency", "film")
    pub tune: Option<String>,
    
    /// Profile for encoders (e.g., "baseline", "main", "high")
    pub profile: Option<String>,
    
    /// Rate control mode (e.g., "crf", "vbr", "cbr")
    pub rate_control: RateControlMode,
    
    /// Additional FFmpeg parameters as key-value pairs
    pub extra_params: Vec<(String, String)>,
}

/// Rate control modes for video encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateControlMode {
    /// Constant Rate Factor (quality-based)
    CRF(u32),
    
    /// Variable Bitrate
    VBR {
        target_bitrate: u32,
        max_bitrate: u32,
    },
    
    /// Constant Bitrate
    CBR(u32),
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        ScreenCaptureConfig {
            monitor_index: 0,
            fps: 30,
            quality: 80,
            codec: VideoCodec::H264,
            hardware_acceleration: HardwareAcceleration::None,
            capture_cursor: true,
            capture_audio: false,
            keyframe_interval: 30,   // One keyframe per second at 30 FPS
            bitrate: None,           // Auto bitrate based on quality
            latency_mode: LatencyMode::Balanced,
            advanced_options: None,
        }
    }
}

impl Default for AdvancedEncodingOptions {
    fn default() -> Self {
        AdvancedEncodingOptions {
            pixel_format: "yuv420p".to_string(),
            preset: Some("ultrafast".to_string()),
            tune: Some("zerolatency".to_string()),
            profile: Some("baseline".to_string()),
            rate_control: RateControlMode::CRF(23),
            extra_params: Vec::new(),
        }
    }
}

/// Builder pattern for ScreenCaptureConfig
pub struct ScreenCaptureConfigBuilder {
    config: ScreenCaptureConfig,
}

impl ScreenCaptureConfigBuilder {
    pub fn new() -> Self {
        ScreenCaptureConfigBuilder {
            config: ScreenCaptureConfig::default(),
        }
    }
    
    pub fn monitor_index(mut self, index: usize) -> Self {
        self.config.monitor_index = index;
        self
    }
    
    pub fn fps(mut self, fps: u32) -> Self {
        self.config.fps = fps;
        self
    }
    
    pub fn quality(mut self, quality: u32) -> Self {
        self.config.quality = quality.min(100).max(1);
        self
    }
    
    pub fn codec(mut self, codec: VideoCodec) -> Self {
        self.config.codec = codec;
        self
    }
    
    pub fn hardware_acceleration(mut self, accel: HardwareAcceleration) -> Self {
        self.config.hardware_acceleration = accel;
        self
    }
    
    pub fn capture_cursor(mut self, capture: bool) -> Self {
        self.config.capture_cursor = capture;
        self
    }
    
    pub fn capture_audio(mut self, capture: bool) -> Self {
        self.config.capture_audio = capture;
        self
    }
    
    pub fn keyframe_interval(mut self, interval: u32) -> Self {
        self.config.keyframe_interval = interval;
        self
    }
    
    pub fn bitrate(mut self, bitrate: Option<u32>) -> Self {
        self.config.bitrate = bitrate;
        self
    }
    
    pub fn latency_mode(mut self, mode: LatencyMode) -> Self {
        self.config.latency_mode = mode;
        self
    }
    
    pub fn advanced_options(mut self, options: AdvancedEncodingOptions) -> Self {
        self.config.advanced_options = Some(options);
        self
    }
    
    pub fn build(self) -> ScreenCaptureConfig {
        self.config
    }
}
