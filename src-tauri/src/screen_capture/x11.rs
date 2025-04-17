// screen_capture/x11.rs - X11-specific screen capture implementation

use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use tauri::Window;

use crate::screen_capture::types::{MonitorInfo, CaptureStats, ScreenCapturer, MonitorDetector, FrameData};
use crate::screen_capture::error::{ScreenCaptureError, to_capture_error, to_ffmpeg_error};
use crate::screen_capture::config::ScreenCaptureConfig;
use crate::screen_capture::buffer::StreamBuffer;
use crate::screen_capture::quality::AdaptiveQualityController;
use crate::screen_capture::utils;

/// X11-specific monitor detector implementation
pub struct X11MonitorDetector;

impl MonitorDetector for X11MonitorDetector {
    fn detect_monitors(&self) -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
        get_x11_monitors()
    }
}

/// X11-specific screen capture implementation
pub struct X11ScreenCapturer {
    // Configuration
    config: Arc<Mutex<ScreenCaptureConfig>>,
    
    // Capture state
    running: Arc<Mutex<bool>>,
    
    // FFmpeg process
    capture_process: Arc<Mutex<Option<Child>>>,
    
    // Monitor info
    monitor: MonitorInfo,
    
    // Stream buffer
    stream_buffer: Arc<Mutex<StreamBuffer>>,
    
    // Quality controller
    quality_controller: Arc<Mutex<AdaptiveQualityController>>,
    
    // Stats
    stats: Arc<Mutex<CaptureStats>>,
    
    // Capture thread
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl X11ScreenCapturer {
    /// Create a new X11 screen capturer
    pub fn new(
        config: Arc<Mutex<ScreenCaptureConfig>>,
        monitor: MonitorInfo,
        stream_buffer: Arc<Mutex<StreamBuffer>>,
        quality_controller: Arc<Mutex<AdaptiveQualityController>>,
        stats: Arc<Mutex<CaptureStats>>
    ) -> Result<Self, ScreenCaptureError> {
        Ok(X11ScreenCapturer {
            config,
            running: Arc::new(Mutex::new(false)),
            capture_process: Arc::new(Mutex::new(None)),
            monitor,
            stream_buffer,
            quality_controller,
            stats,
            capture_thread: None,
        })
    }

    /// Start FFmpeg process for X11 screen capture
    fn start_ffmpeg_process(&self) -> Result<Child, ScreenCaptureError> {
        let config_guard = self.config.lock().unwrap();
        
        // Create FFmpeg command for continuous stream
        let mut cmd = Command::new("ffmpeg");
        
        // Input configuration
        cmd.arg("-f").arg("x11grab")
           .arg("-video_size").arg(format!("{}x{}", self.monitor.width, self.monitor.height))
           .arg("-i").arg(format!(":0.0+{},{}", self.monitor.x_offset, self.monitor.y_offset));
        
        // Framerate
        cmd.arg("-framerate").arg(config_guard.fps.to_string());
        
        // Mouse cursor capture
        if config_guard.capture_cursor {
            cmd.arg("-draw_mouse").arg("1");
        } else {
            cmd.arg("-draw_mouse").arg("0");
        }
        
        // Hardware acceleration
        match config_guard.hardware_acceleration {
            crate::screen_capture::types::HardwareAcceleration::VAAPI => {
                cmd.arg("-hwaccel").arg("vaapi")
                   .arg("-hwaccel_device").arg("/dev/dri/renderD128")
                   .arg("-hwaccel_output_format").arg("vaapi");
                
                // Codec-specific optimizations for VAAPI
                match config_guard.codec {
                    crate::screen_capture::types::VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_vaapi")
                           .arg("-qp").arg("23")
                           .arg("-quality").arg("speed");
                    },
                    crate::screen_capture::types::VideoCodec::VP8 => {
                        // VP8 with VAAPI not always available
                        cmd.arg("-c:v").arg("vp8_vaapi");
                    },
                    crate::screen_capture::types::VideoCodec::VP9 => {
                        // VP9 with VAAPI
