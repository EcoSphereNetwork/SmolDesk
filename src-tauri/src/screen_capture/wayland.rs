// screen_capture/wayland.rs - Wayland-specific screen capture implementation

use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use tauri::Window;

use crate::screen_capture::types::{MonitorInfo, CaptureStats, ScreenCapturer, MonitorDetector, FrameData, DisplayServer, VideoCodec, HardwareAcceleration};
use crate::screen_capture::error::{ScreenCaptureError, to_capture_error, to_ffmpeg_error};
use crate::screen_capture::config::ScreenCaptureConfig;
use crate::screen_capture::buffer::{StreamBuffer, DropMode};
use crate::screen_capture::quality::AdaptiveQualityController;
use crate::screen_capture::utils;

/// Wayland-specific monitor detector implementation
pub struct WaylandMonitorDetector;

impl MonitorDetector for WaylandMonitorDetector {
    fn detect_monitors(&self) -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
        get_wayland_monitors()
    }
}

/// Wayland-specific screen capture implementation
pub struct WaylandScreenCapturer {
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

impl WaylandScreenCapturer {
    /// Create a new Wayland screen capturer
    pub fn new(
        config: Arc<Mutex<ScreenCaptureConfig>>,
        monitor: MonitorInfo,
        stream_buffer: Arc<Mutex<StreamBuffer>>,
        quality_controller: Arc<Mutex<AdaptiveQualityController>>,
        stats: Arc<Mutex<CaptureStats>>
    ) -> Result<Self, ScreenCaptureError> {
        Ok(WaylandScreenCapturer {
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

    /// Start PipeWire process for Wayland screen capture
    fn start_pipewire_process(&self) -> Result<Child, ScreenCaptureError> {
        let config_guard = self.config.lock().unwrap();
        
        // Create FFmpeg command for continuous stream using PipeWire
        let mut cmd = Command::new("ffmpeg");
        
        // Input configuration
        // Use pipewire to capture Wayland screens
        cmd.arg("-f").arg("pipewire")
           .arg("-framerate").arg(config_guard.fps.to_string());
           
        // Select specific monitor if needed
        if self.monitor.name != "Wayland-0" {
            cmd.arg("-i").arg(format!("{}:{}", "pipewire", self.monitor.index));
        } else {
            cmd.arg("-i").arg("0"); // Default screen
        }
        
        // Hardware acceleration
        match config_guard.hardware_acceleration {
            HardwareAcceleration::VAAPI => {
                cmd.arg("-hwaccel").arg("vaapi")
                   .arg("-hwaccel_device").arg("/dev/dri/renderD128")
                   .arg("-hwaccel_output_format").arg("vaapi");
                
                // Codec-specific optimizations for VAAPI
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_vaapi")
                           .arg("-qp").arg("23")
                           .arg("-quality").arg("speed");
                    },
                    VideoCodec::VP8 => {
                        // VP8 with VAAPI not always available
                        cmd.arg("-c:v").arg("vp8_vaapi");
                    },
                    VideoCodec::VP9 => {
                        // VP9 with VAAPI
                        cmd.arg("-c:v").arg("vp9_vaapi");
                    },
                    VideoCodec::AV1 => {
                        // AV1 might not be available with VAAPI, fallback to software
                        cmd.arg("-c:v").arg("libaom-av1");
                    }
                }
            },
            HardwareAcceleration::NVENC => {
                cmd.arg("-hwaccel").arg("cuda")
                   .arg("-hwaccel_output_format").arg("cuda");
                
                // Codec-specific optimizations for NVENC
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_nvenc")
                           .arg("-preset").arg("llhp")  // Low latency high performance
                           .arg("-zerolatency").arg("1");
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 => {
                        // NVENC doesn't support VP8/VP9, fallback to software
                        match config_guard.codec {
                            VideoCodec::VP8 => cmd.arg("-c:v").arg("libvpx"),
                            VideoCodec::VP9 => cmd.arg("-c:v").arg("libvpx-vp9"),
                            _ => {}
                        }
                    },
                    VideoCodec::AV1 => {
                        // Check if we have NVENC AV1 support, otherwise fallback
                        cmd.arg("-c:v").arg("av1_nvenc");
                    }
                }
            },
            HardwareAcceleration::QuickSync => {
                cmd.arg("-hwaccel").arg("qsv")
                   .arg("-hwaccel_output_format").arg("qsv");
                
                // Codec-specific optimizations for QuickSync
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_qsv")
                           .arg("-preset").arg("veryfast")
                           .arg("-low_power").arg("1");  // Low power mode for better battery life
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 | VideoCodec::AV1 => {
                        // QSV typically doesn't support these codecs well, fallback to software
                        match config_guard.codec {
                            VideoCodec::VP8 => cmd.arg("-c:v").arg("libvpx"),
                            VideoCodec::VP9 => cmd.arg("-c:v").arg("libvpx-vp9"),
                            VideoCodec::AV1 => cmd.arg("-c:v").arg("libaom-av1"),
                            _ => {}
                        }
                    }
                }
            },
            HardwareAcceleration::None => {
                // Software encoding
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("libx264")
                           .arg("-preset").arg("ultrafast")
                           .arg("-tune").arg("zerolatency");
                    },
                    VideoCodec::VP8 => {
                        cmd.arg("-c:v").arg("libvpx")
                           .arg("-deadline").arg("realtime")
                           .arg("-cpu-used").arg("8");
                    },
                    VideoCodec::VP9 => {
                        cmd.arg("-c:v").arg("libvpx-vp9")
                           .arg("-deadline").arg("realtime")
                           .arg("-cpu-used").arg("8");
                    },
                    VideoCodec::AV1 => {
                        cmd.arg("-c:v").arg("libaom-av1")
                           .arg("-cpu-used").arg("8");
                    }
                }
            }
        }
        
        // Get quality-based parameters from quality controller
        let quality_controller = self.quality_controller.lock().unwrap();
        let quality_params = quality_controller.generate_ffmpeg_params(&config_guard);
        
        // Add quality parameters
        for param in quality_params {
            cmd.arg(&param);
        }
        
        // Keyframe interval
        cmd.arg("-g").arg(config_guard.keyframe_interval.to_string());
        
        // Low-latency optimizations based on latency mode
        match config_guard.latency_mode {
            crate::screen_capture::types::LatencyMode::UltraLow => {
                cmd.arg("-tune").arg("zerolatency")
                   .arg("-probesize").arg("32")
                   .arg("-flush_packets").arg("1");
            },
            crate::screen_capture::types::LatencyMode::Balanced => {
                cmd.arg("-tune").arg("zerolatency");
            },
            crate::screen_capture::types::LatencyMode::Quality => {
                // No special low-latency flags as we prioritize quality
            }
        }
        
        // Output format for streaming - use matroska for container
        cmd.arg("-f").arg("matroska")
           .arg("-movflags").arg("faststart")  // Fast start for streaming
           .arg("-");  // Output to stdout
        
        // Redirect stderr and make stdout available for reading
        cmd.stderr(Stdio::null())
           .stdout(Stdio::piped());
        
        // Start the ffmpeg process
        let process = cmd.spawn()
            .map_err(|e| to_ffmpeg_error(e, "Failed to start FFmpeg process with PipeWire"))?;
        
        Ok(process)
    }
    
    /// Wayland capture loop
    fn capture_loop(
        config: Arc<Mutex<ScreenCaptureConfig>>,
        running: Arc<Mutex<bool>>,
        stats: Arc<Mutex<CaptureStats>>,
        window: Option<Window>,
        monitor: MonitorInfo,
        stream_buffer: Arc<Mutex<StreamBuffer>>,
        quality_controller: Arc<Mutex<AdaptiveQualityController>>,
        capture_process: Arc<Mutex<Option<Child>>>,
    ) {
        // Get initial CPU usage
        let initial_cpu_usage = utils::get_cpu_usage().unwrap_or(0.0);
        
        let mut last_frame_time = Instant::now();
        let mut frame_count: u64 = 0;
        let mut dropped_frames: u64 = 0;
        let start_time = Instant::now();
        
        // Start the PipeWire process for continuous capture
        let mut process = match Self::start_pipewire_process_static(&config, &monitor, &quality_controller) {
            Ok(process) => process,
            Err(e) => {
                eprintln!("Failed to start PipeWire process: {}", e);
                return;
            }
        };
        
        // Store the process in shared variable
        {
            let mut process_guard = capture_process.lock().unwrap();
            *process_guard = Some(process.try_clone().unwrap_or(process));
        }
        
        // Get stdout for reading video data
        let mut stdout = process.stdout.take().expect("Failed to take stdout from FFmpeg process");
        
        // Buffer for reading output
        let mut buffer = Vec::new();
        let mut read_buffer = vec![0u8; 65536]; // 64KB buffer for reading
        
        // Main loop for capturing and processing frames
        let mut last_stats_update = Instant::now();
        
        while *running.lock().unwrap() {
            let now = Instant::now();
            
            // Check if the process is still running
            match process.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("FFmpeg/PipeWire process exited with status: {}", status);
                    break;
                }
                Ok(None) => {},
                Err(e) => {
                    eprintln!("Error checking FFmpeg/PipeWire process: {}", e);
                    break;
                }
            }
            
            // Read data from the FFmpeg process
            match stdout.read(&mut read_buffer) {
                Ok(n) if n > 0 => {
                    // Data was read, add it to the buffer
                    buffer.extend_from_slice(&read_buffer[0..n]);
                    
                    // For matroska/webm streams, we need to detect frame boundaries
                    // Here's a simple heuristic: look for keyframe markers (0x87)
                    // A more robust approach would involve parsing the matroska container
                    
                    let mut frame_start_index = 0;
                    for i in 0..buffer.len().saturating_sub(4) {
                        // Look for likely keyframe marker (simple heuristic)
                        if buffer[i] == 0x87 && buffer[i+1] == 0x00 {
                            // Found a potential keyframe, treat everything before this as one frame
                            if i > frame_start_index {
                                // Extract the frame data
                                let frame_data = buffer[frame_start_index..i].to_vec();
                                
                                if !frame_data.is_empty() {
                                    // Create frame data
                                    let frame = FrameData {
                                        data: frame_data,
                                        timestamp: now.elapsed().as_millis() as u64,
                                        keyframe: true, // Assuming keyframes for simplicity
                                        width: monitor.width,
                                        height: monitor.height,
                                        format: "matroska".to_string(),
                                    };
                                    
                                    // Add to buffer
                                    {
                                        let mut stream_buf = stream_buffer.lock().unwrap();
                                        if let Err(e) = stream_buf.push_frame(frame) {
                                            eprintln!("Error adding frame to buffer: {}", e);
                                            dropped_frames += 1;
                                        }
                                    }
                                    
                                    frame_count += 1;
                                }
                                
                                frame_start_index = i;
                            }
                        }
                    }
                    
                    // Remove processed data from buffer, keeping potential partial frame
                    if frame_start_index > 0 {
                        buffer.drain(0..frame_start_index);
                    }
                    
                    // If buffer is too big, reset it (something went wrong)
                    if buffer.len() > 10 * 1024 * 1024 { // 10MB limit
                        buffer.clear();
                        eprintln!("Buffer overflow, clearing");
                    }
                    
                    // Send frame data to frontend if window is provided
                    if let Some(ref window) = window {
                        // Get the first frame from buffer without removing it
                        let frame_preview = {
                            let stream_buf = stream_buffer.lock().unwrap();
                            stream_buf.peek_next_frame().map(|f| f.data.clone())
                        };
                        
                        if let Some(frame_data) = frame_preview {
                            // Send as binary data or base64 depending on frontend needs
                            let _ = window.emit("frame_data", utils::frame_to_base64(&frame_data));
                        }
                    }
                    
                    // Update stats periodically
                    if now.duration_since(last_stats_update) > Duration::from_millis(500) {
                        last_stats_update = now;
                        
                        // Capture current statistics
                        let current_cpu_usage = utils::get_cpu_usage().unwrap_or(initial_cpu_usage);
                        let buffer_stats = stream_buffer.lock().unwrap().get_stats();
                        let buffer_fill_ratio = buffer_stats.fill_ratio;
                        
                        // Calculate frame rate and bitrate
                        let elapsed_secs = start_time.elapsed().as_secs_f64();
                        let fps = if elapsed_secs > 0.0 { frame_count as f64 / elapsed_secs } else { 0.0 };
                        let bitrate = if elapsed_secs > 0.0 { 
                            (buffer.len() as f64 * 8.0 / elapsed_secs) as u64
                        } else { 
                            0 
                        };
                        
                        // Update quality controller with new metrics
                        {
                            let mut quality_ctrl = quality_controller.lock().unwrap();
                            quality_ctrl.update_metrics(
                                current_cpu_usage,
                                (bitrate / 1000) as u32, // kbps
                                if frame_count > 0 { dropped_frames as f32 / frame_count as f32 } else { 0.0 },
                                buffer_stats.latency_ms as u32
                            );
                            
                            // Apply quality adjustments if needed
                            let _ = quality_ctrl.adjust_quality();
                        }
                        
                        // Update capture statistics
                        {
                            let mut stats_guard = stats.lock().unwrap();
                            stats_guard.fps = fps;
                            stats_guard.bitrate = bitrate;
                            stats_guard.frame_count = frame_count;
                            stats_guard.dropped_frames = dropped_frames;
                            stats_guard.buffer_level = buffer_stats.frame_count;
                            stats_guard.latency_estimate = buffer_stats.latency_ms;
                            
                            // Send stats to frontend
                            if let Some(ref window) = window {
                                let _ = window.emit("capture_stats", stats_guard.clone());
                            }
                        }
                    }
                    
                    last_frame_time = now;
                },
                Ok(_) => {
                    // No data available, wait a bit
                    thread::sleep(Duration::from_millis(1));
                },
                Err(e) => {
                    eprintln!("Error reading from FFmpeg/PipeWire: {}", e);
                    dropped_frames += 1;
                    
                    // Update stats
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.dropped_frames = dropped_frames;
                    
                    // Check if process is still alive
                    if let Err(e) = process.try_wait() {
                        eprintln!("Error checking FFmpeg/PipeWire process: {}", e);
                        break;
                    }
                }
            }
        }
        
        // Clean up when the loop ends
        if let Err(e) = process.kill() {
            eprintln!("Error killing FFmpeg/PipeWire process: {}", e);
        }
    }
    
    /// Static version of start_pipewire_process for use in capture_loop
    fn start_pipewire_process_static(
        config: &Arc<Mutex<ScreenCaptureConfig>>,
        monitor: &MonitorInfo,
        quality_controller: &Arc<Mutex<AdaptiveQualityController>>
    ) -> Result<Child, ScreenCaptureError> {
        let config_guard = config.lock().unwrap();
        
        // Create FFmpeg command for continuous stream using PipeWire
        let mut cmd = Command::new("ffmpeg");
        
        // Input configuration
        // Use pipewire to capture Wayland screens
        cmd.arg("-f").arg("pipewire")
           .arg("-framerate").arg(config_guard.fps.to_string());
           
        // Select specific monitor if needed
        if monitor.name != "Wayland-0" {
            cmd.arg("-i").arg(format!("{}:{}", "pipewire", monitor.index));
        } else {
            cmd.arg("-i").arg("0"); // Default screen
        }
        
        // Hardware acceleration
        match config_guard.hardware_acceleration {
            HardwareAcceleration::VAAPI => {
                cmd.arg("-hwaccel").arg("vaapi")
                   .arg("-hwaccel_device").arg("/dev/dri/renderD128")
                   .arg("-hwaccel_output_format").arg("vaapi");
                
                // Codec-specific optimizations for VAAPI
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_vaapi")
                           .arg("-qp").arg("23")
                           .arg("-quality").arg("speed");
                    },
                    VideoCodec::VP8 => {
                        // VP8 with VAAPI not always available
                        cmd.arg("-c:v").arg("vp8_vaapi");
                    },
                    VideoCodec::VP9 => {
                        // VP9 with VAAPI
                        cmd.arg("-c:v").arg("vp9_vaapi");
                    },
                    VideoCodec::AV1 => {
                        // AV1 might not be available with VAAPI, fallback to software
                        cmd.arg("-c:v").arg("libaom-av1");
                    }
                }
            },
            HardwareAcceleration::NVENC => {
                cmd.arg("-hwaccel").arg("cuda")
                   .arg("-hwaccel_output_format").arg("cuda");
                
                // Codec-specific optimizations for NVENC
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_nvenc")
                           .arg("-preset").arg("llhp")  // Low latency high performance
                           .arg("-zerolatency").arg("1");
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 => {
                        // NVENC doesn't support VP8/VP9, fallback to software
                        match config_guard.codec {
                            VideoCodec::VP8 => cmd.arg("-c:v").arg("libvpx"),
                            VideoCodec::VP9 => cmd.arg("-c:v").arg("libvpx-vp9"),
                            _ => {}
                        }
                    },
                    VideoCodec::AV1 => {
                        // Check if we have NVENC AV1 support, otherwise fallback
                        cmd.arg("-c:v").arg("av1_nvenc");
                    }
                }
            },
            HardwareAcceleration::QuickSync => {
                cmd.arg("-hwaccel").arg("qsv")
                   .arg("-hwaccel_output_format").arg("qsv");
                
                // Codec-specific optimizations for QuickSync
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_qsv")
                           .arg("-preset").arg("veryfast")
                           .arg("-low_power").arg("1");  // Low power mode for better battery life
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 | VideoCodec::AV1 => {
                        // QSV typically doesn't support these codecs well, fallback to software
                        match config_guard.codec {
                            VideoCodec::VP8 => cmd.arg("-c:v").arg("libvpx"),
                            VideoCodec::VP9 => cmd.arg("-c:v").arg("libvpx-vp9"),
                            VideoCodec::AV1 => cmd.arg("-c:v").arg("libaom-av1"),
                            _ => {}
                        }
                    }
                }
            },
            HardwareAcceleration::None => {
                // Software encoding
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("libx264")
                           .arg("-preset").arg("ultrafast")
                           .arg("-tune").arg("zerolatency");
                    },
                    VideoCodec::VP8 => {
                        cmd.arg("-c:v").arg("libvpx")
                           .arg("-deadline").arg("realtime")
                           .arg("-cpu-used").arg("8");
                    },
                    VideoCodec::VP9 => {
                        cmd.arg("-c:v").arg("libvpx-vp9")
                           .arg("-deadline").arg("realtime")
                           .arg("-cpu-used").arg("8");
                    },
                    VideoCodec::AV1 => {
                        cmd.arg("-c:v").arg("libaom-av1")
                           .arg("-cpu-used").arg("8");
                    }
                }
            }
        }
        
        // Get quality-based parameters from quality controller
        let quality_controller_guard = quality_controller.lock().unwrap();
        let quality_params = quality_controller_guard.generate_ffmpeg_params(&config_guard);
        
        // Add quality parameters
        for param in quality_params {
            cmd.arg(&param);
        }
        
        // Keyframe interval
        cmd.arg("-g").arg(config_guard.keyframe_interval.to_string());
        
        // Output format for streaming - use matroska for container
        cmd.arg("-f").arg("matroska")
           .arg("-movflags").arg("faststart")  // Fast start for streaming
           .arg("-");  // Output to stdout
        
        // Redirect stderr and make stdout available for reading
        cmd.stderr(Stdio::null())
           .stdout(Stdio::piped());
        
        // Start the ffmpeg process
        let process = cmd.spawn()
            .map_err(|e| to_ffmpeg_error(e, "Failed to start FFmpeg process with PipeWire"))?;
        
        Ok(process)
    }
}

impl ScreenCapturer for WaylandScreenCapturer {
    fn start_capture(&mut self) -> Result<(), ScreenCaptureError> {
        // Check if already running
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Clear stream buffer
        {
            let mut buffer = self.stream_buffer.lock().unwrap();
            buffer.clear();
        }

        // Clone necessary data for the capture thread
        let config = self.config.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();
        let monitor = self.monitor.clone();
        let stream_buffer = self.stream_buffer.clone();
        let quality_controller = self.quality_controller.clone();
        let capture_process = self.capture_process.clone();

        // Create the capture thread
        self.capture_thread = Some(thread::spawn(move || {
            Self::capture_loop(
                config,
                running,
                stats,
                None, // No window for direct UI updates in the module
                monitor,
                stream_buffer,
                quality_controller,
                capture_process
            );
        }));

        Ok(())
    }

    fn stop_capture(&mut self) -> Result<(), ScreenCaptureError> {
        // Set running flag to false to signal the capture thread to stop
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        // Kill the FFmpeg process if it's running
        {
            let mut process = self.capture_process.lock().unwrap();
            if let Some(ref mut child) = *process {
                let _ = child.kill();
            }
            *process = None;
        }

        // Wait for the capture thread to finish
        if let Some(handle) = self.capture_thread.take() {
            match handle.join() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error joining capture thread: {:?}", e);
                }
            }
        }

        Ok(())
    }

    fn get_next_frame(&mut self) -> Option<FrameData> {
        let mut buffer = self.stream_buffer.lock().unwrap();
        buffer.get_next_frame()
    }

    fn get_stats(&self) -> CaptureStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Get monitor information for Wayland
pub fn get_wayland_monitors() -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
    // For Wayland, we can use wlr-randr for wlroots-based compositors
    // or try to use swaymsg for Sway
    
    // First try wlr-randr
    let output = Command::new("wlr-randr")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            return parse_wlr_randr_output(&output.stdout);
        }
    }
    
    // Fallback to swaymsg for Sway
    let output = Command::new("swaymsg")
        .arg("-t")
        .arg("get_outputs")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            return parse_swaymsg_output(&output.stdout);
        }
    }
    
    // If both fail, try to use most basic detection with Wayland-specific tools
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo $WAYLAND_DISPLAY")
        .output();
    
    // If we're running on Wayland, at least provide a generic monitor
    if let Ok(output) = output {
        if output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
            // For now, just return a single monitor that represents the whole screen
            let monitors = vec![MonitorInfo {
                index: 0,
                name: "Wayland-0".to_string(),
                width: 1920, // Default assumption
                height: 1080, // Default assumption
                refresh_rate: Some(60.0), // Default assumption
                primary: true,
                x_offset: 0,
                y_offset: 0,
            }];
            
            return Ok(monitors);
        }
    }
    
    Err(ScreenCaptureError::DisplayServerError(
        "Failed to detect Wayland monitors".to_string()
    ))
}

/// Parse wlr-randr output to get monitor information
fn parse_wlr_randr_output(output: &[u8]) -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
    let output_str = String::from_utf8_lossy(output);
    let mut monitors = Vec::new();
    let mut current_monitor: Option<MonitorInfo> = None;
    let mut index = 0;
    
    for line in output_str.lines() {
        if !line.starts_with(' ') {
            // This is a monitor name line
            if let Some(monitor) = current_monitor.take() {
                monitors.push(monitor);
            }
            
            let name = line.trim().to_string();
            current_monitor = Some(MonitorInfo {
                index,
                name,
                width: 0,
                height: 0,
                refresh_rate: None,
                primary: false,
                x_offset: 0,
                y_offset: 0,
            });
            
            index += 1;
        } else if line.contains("current") {
            // This line contains resolution
            if let Some(ref mut monitor) = current_monitor {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "current" {
                    let resolution = parts[1];
                    let dimensions: Vec<&str> = resolution.split('x').collect();
                    if dimensions.len() >= 2 {
                        if let (Ok(width), Ok(height)) = (
                            dimensions[0].parse::<u32>(),
                            dimensions[1].parse::<u32>(),
                        ) {
                            monitor.width = width;
                            monitor.height = height;
                        }
                    }
                }
            }
        } else if line.contains("Hz") {
            // This line might contain refresh rate
            if let Some(ref mut monitor) = current_monitor {
                if let Some(hz_part) = line.split_whitespace().find(|p| p.ends_with("Hz")) {
                    if let Ok(rate) = hz_part.trim_end_matches("Hz").parse::<f64>() {
                        monitor.refresh_rate = Some(rate);
                    }
                }
            }
        } else if line.contains("position") {
            // This line contains position information
            if let Some(ref mut monitor) = current_monitor {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "position" {
                    let position = parts[1];
                    let pos_parts: Vec<&str> = position.split(',').collect();
                    if pos_parts.len() >= 2 {
                        monitor.x_offset = pos_parts[0].parse().unwrap_or(0);
                        monitor.y_offset = pos_parts[1].parse().unwrap_or(0);
                    }
                }
            }
        } else if line.contains("primary") {
            // This line indicates primary display
            if let Some(ref mut monitor) = current_monitor {
                monitor.primary = true;
            }
        }
    }
    
    // Don't forget the last monitor
    if let Some(monitor) = current_monitor {
        monitors.push(monitor);
    }
    
    if monitors.is_empty() {
        return Err(ScreenCaptureError::DisplayServerError(
            "No monitors detected from wlr-randr".to_string(),
        ));
    }
    
    Ok(monitors)
}

/// Parse swaymsg output (JSON)
fn parse_swaymsg_output(output: &[u8]) -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
    let output_str = String::from_utf8_lossy(output);
    let mut monitors = Vec::new();
    
    // Parse JSON output
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output_str);
    
    match parsed {
        Ok(json) => {
            if let Some(outputs) = json.as_array() {
                for (index, output) in outputs.iter().enumerate() {
                    if let (Some(name), Some(active)) = (
                        output.get("name").and_then(|v| v.as_str()),
                        output.get("active").and_then(|v| v.as_bool()),
                    ) {
                        // Skip inactive outputs
                        if !active {
                            continue;
                        }
                        
                        let primary = output.get("primary")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        if let Some(rect) = output.get("rect").and_then(|v| v.as_object()) {
                            let width = rect.get("width")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            
                            let height = rect.get("height")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            
                            let x_offset = rect.get("x")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;
                            
                            let y_offset = rect.get("y")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;
                            
                            let refresh_rate = output.get("refresh")
                                .and_then(|v| v.as_f64());
                            
                            monitors.push(MonitorInfo {
                                index,
                                name: name.to_string(),
                                width,
                                height,
                                refresh_rate,
                                primary,
                                x_offset,
                                y_offset,
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(ScreenCaptureError::DisplayServerError(
                format!("Failed to parse swaymsg output: {}", e)
            ));
        }
    }
    
    if monitors.is_empty() {
        return Err(ScreenCaptureError::DisplayServerError(
            "No active monitors detected from swaymsg".to_string(),
        ));
    }
    
    Ok(monitors)
}
