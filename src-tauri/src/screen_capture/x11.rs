// screen_capture/x11.rs - X11-specific screen capture implementation

use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::io::Read;

use crate::screen_capture::types::{MonitorInfo, CaptureStats, ScreenCapturer, MonitorDetector, FrameData, VideoCodec, HardwareAcceleration};
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
    fn start_ffmpeg_process_static(
        config: &Arc<Mutex<ScreenCaptureConfig>>,
        monitor: &MonitorInfo,
        quality_controller: &Arc<Mutex<AdaptiveQualityController>>
    ) -> Result<Child, ScreenCaptureError> {
        let config_guard = config.lock().unwrap();
        
        // Create FFmpeg command for continuous stream
        let mut cmd = Command::new("ffmpeg");
        
        // Input configuration
        cmd.arg("-f").arg("x11grab")
           .arg("-video_size").arg(format!("{}x{}", monitor.width, monitor.height))
           .arg("-i").arg(format!(":0.0+{},{}", monitor.x_offset, monitor.y_offset));
        
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
            HardwareAcceleration::VAAPI => {
                cmd.arg("-hwaccel").arg("vaapi")
                   .arg("-hwaccel_device").arg("/dev/dri/renderD128")
                   .arg("-hwaccel_output_format").arg("vaapi");
                
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_vaapi")
                           .arg("-qp").arg("23")
                           .arg("-quality").arg("speed");
                    },
                    VideoCodec::VP8 => {
                        cmd.arg("-c:v").arg("vp8_vaapi");
                    },
                    VideoCodec::VP9 => {
                        cmd.arg("-c:v").arg("vp9_vaapi");
                    },
                    VideoCodec::AV1 => {
                        cmd.arg("-c:v").arg("libaom-av1");
                    }
                }
            },
            HardwareAcceleration::NVENC => {
                cmd.arg("-hwaccel").arg("cuda")
                   .arg("-hwaccel_output_format").arg("cuda");
                
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_nvenc")
                           .arg("-preset").arg("llhp")
                           .arg("-zerolatency").arg("1");
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 => {
                        match config_guard.codec {
                            VideoCodec::VP8 => cmd.arg("-c:v").arg("libvpx"),
                            VideoCodec::VP9 => cmd.arg("-c:v").arg("libvpx-vp9"),
                            _ => {}
                        }
                    },
                    VideoCodec::AV1 => {
                        cmd.arg("-c:v").arg("av1_nvenc");
                    }
                }
            },
            HardwareAcceleration::QuickSync => {
                cmd.arg("-hwaccel").arg("qsv")
                   .arg("-hwaccel_output_format").arg("qsv");
                
                match config_guard.codec {
                    VideoCodec::H264 => {
                        cmd.arg("-c:v").arg("h264_qsv")
                           .arg("-preset").arg("veryfast")
                           .arg("-low_power").arg("1");
                    },
                    VideoCodec::VP8 | VideoCodec::VP9 | VideoCodec::AV1 => {
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
           .arg("-movflags").arg("faststart")
           .arg("-");
        
        // Redirect stderr and make stdout available for reading
        cmd.stderr(Stdio::null())
           .stdout(Stdio::piped());
        
        // Start the ffmpeg process
        let process = cmd.spawn()
            .map_err(|e| to_ffmpeg_error(e, "Failed to start FFmpeg process"))?;
        
        Ok(process)
    }
    
    /// X11 capture loop
    fn capture_loop(
        config: Arc<Mutex<ScreenCaptureConfig>>,
        running: Arc<Mutex<bool>>,
        stats: Arc<Mutex<CaptureStats>>,
        monitor: MonitorInfo,
        stream_buffer: Arc<Mutex<StreamBuffer>>,
        quality_controller: Arc<Mutex<AdaptiveQualityController>>,
        capture_process: Arc<Mutex<Option<Child>>>,
    ) {
        let mut frame_count: u64 = 0;
        let mut dropped_frames: u64 = 0;
        let start_time = Instant::now();
        
        // Start the FFmpeg process for continuous capture
        let mut process = match Self::start_ffmpeg_process_static(&config, &monitor, &quality_controller) {
            Ok(process) => process,
            Err(e) => {
                eprintln!("Failed to start FFmpeg process: {}", e);
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
                    eprintln!("FFmpeg process exited with status: {}", status);
                    break;
                }
                Ok(None) => {},
                Err(e) => {
                    eprintln!("Error checking FFmpeg process: {}", e);
                    break;
                }
            }
            
            // Read data from the FFmpeg process
            match stdout.read(&mut read_buffer) {
                Ok(n) if n > 0 => {
                    // Data was read, add it to the buffer
                    buffer.extend_from_slice(&read_buffer[0..n]);
                    
                    // For matroska/webm streams, we need to detect frame boundaries
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
                                        keyframe: true,
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
                    
                    // Update stats periodically
                    if now.duration_since(last_stats_update) > Duration::from_millis(500) {
                        last_stats_update = now;
                        
                        // Capture current statistics
                        let current_cpu_usage = utils::get_cpu_usage().unwrap_or(0.0);
                        let buffer_stats = stream_buffer.lock().unwrap().get_stats();
                        
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
                                (bitrate / 1000) as u32,
                                if frame_count > 0 { dropped_frames as f32 / frame_count as f32 } else { 0.0 },
                                buffer_stats.latency_ms as u32
                            );
                            
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
                        }
                    }
                },
                Ok(_) => {
                    // No data available, wait a bit
                    thread::sleep(Duration::from_millis(1));
                },
                Err(e) => {
                    eprintln!("Error reading from FFmpeg: {}", e);
                    dropped_frames += 1;
                    
                    // Update stats
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.dropped_frames = dropped_frames;
                    
                    // Check if process is still alive
                    if let Err(e) = process.try_wait() {
                        eprintln!("Error checking FFmpeg process: {}", e);
                        break;
                    }
                }
            }
        }
        
        // Clean up when the loop ends
        if let Err(e) = process.kill() {
            eprintln!("Error killing FFmpeg process: {}", e);
        }
    }
}

impl ScreenCapturer for X11ScreenCapturer {
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

/// Get monitor information for X11
pub fn get_x11_monitors() -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
    // Use xrandr to get monitor information
    let output = Command::new("xrandr")
        .arg("--listmonitors")
        .output()
        .map_err(|e| {
            ScreenCaptureError::DisplayServerError(format!("Failed to execute xrandr: {}", e))
        })?;
    
    if !output.status.success() {
        return Err(ScreenCaptureError::DisplayServerError(
            "xrandr returned an error".to_string(),
        ));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    
    // Parse xrandr output to get monitor information
    for (index, line) in output_str.lines().enumerate().skip(1) {
        // Skip header line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let monitor_index = index - 1; // Adjust index to start from 0
            let mut name = parts[1].to_string();
            
            // Remove trailing colon if present
            if name.ends_with(':') {
                name = name[0..name.len()-1].to_string();
            }
            
            // Parse dimensions
            let dimensions_part = parts[2];
            let dimensions: Vec<&str> = dimensions_part.split('/').collect();
            if dimensions.len() >= 1 {
                let resolution: Vec<&str> = dimensions[0].split('x').collect();
                if resolution.len() >= 2 {
                    if let (Ok(width), Ok(height)) = (
                        resolution[0].parse::<u32>(),
                        resolution[1].parse::<u32>(),
                    ) {
                        // Parse position
                        let mut x_offset = 0;
                        let mut y_offset = 0;
                        
                        if parts.len() >= 5 {
                            let position = parts[3];
                            if position.contains('+') {
                                let pos_parts: Vec<&str> = position.split('+').collect();
                                if pos_parts.len() >= 3 {
                                    x_offset = pos_parts[1].parse().unwrap_or(0);
                                    y_offset = pos_parts[2].parse().unwrap_or(0);
                                }
                            }
                        }
                        
                        monitors.push(MonitorInfo {
                            index: monitor_index,
                            name,
                            width,
                            height,
                            refresh_rate: None,
                            primary: line.contains("primary"),
                            x_offset,
                            y_offset,
                        });
                    }
                }
            }
        }
    }
    
    // If no monitors found, provide a default one
    if monitors.is_empty() {
        monitors.push(MonitorInfo {
            index: 0,
            name: "Default".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: Some(60.0),
            primary: true,
            x_offset: 0,
            y_offset: 0,
        });
    }
    
    Ok(monitors)
}
