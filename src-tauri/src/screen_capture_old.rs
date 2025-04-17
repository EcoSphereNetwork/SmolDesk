// src-tauri/src/screen_capture.rs

use std::error::Error;
use std::fmt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::Window;

// Error types for screen capture operations
#[derive(Debug)]
pub enum ScreenCaptureError {
    InitializationFailed(String),
    CaptureError(String),
    EncodingError(String),
    DisplayServerError(String),
    InvalidMonitor(String),
}

impl fmt::Display for ScreenCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScreenCaptureError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            ScreenCaptureError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            ScreenCaptureError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            ScreenCaptureError::DisplayServerError(msg) => write!(f, "Display server error: {}", msg),
            ScreenCaptureError::InvalidMonitor(msg) => write!(f, "Invalid monitor: {}", msg),
        }
    }
}

impl Error for ScreenCaptureError {}

// Display server type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

// Hardware acceleration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    None,
    VAAPI,
    NVENC,
    QuickSync,
}

// Screen capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    pub monitor_index: usize,
    pub fps: u32,
    pub quality: u32, // 0-100
    pub codec: VideoCodec,
    pub hardware_acceleration: HardwareAcceleration,
    pub capture_cursor: bool,
    pub capture_audio: bool,
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
        }
    }
}

// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: Option<f64>,
    pub primary: bool,
}

// Statistics for screen capturing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStats {
    pub fps: f64,
    pub bitrate: u64,
    pub encode_time: f64,
    pub frame_size: u64,
    pub frame_count: u64,
    pub dropped_frames: u64,
}

// Screen capture manager
pub struct ScreenCaptureManager {
    display_server: DisplayServer,
    config: Arc<Mutex<ScreenCaptureConfig>>,
    capture_thread: Option<thread::JoinHandle<()>>,
    monitors: Vec<MonitorInfo>,
    stats: Arc<Mutex<CaptureStats>>,
    running: Arc<Mutex<bool>>,
}

impl ScreenCaptureManager {
    // Create a new screen capture manager
    pub fn new() -> Result<Self, ScreenCaptureError> {
        let display_server = detect_display_server()?;
        
        let monitors = match display_server {
            DisplayServer::X11 => get_x11_monitors(),
            DisplayServer::Wayland => get_wayland_monitors(),
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ))
            }
        }?;

        Ok(ScreenCaptureManager {
            display_server,
            config: Arc::new(Mutex::new(ScreenCaptureConfig::default())),
            capture_thread: None,
            monitors,
            stats: Arc::new(Mutex::new(CaptureStats {
                fps: 0.0,
                bitrate: 0,
                encode_time: 0.0,
                frame_size: 0,
                frame_count: 0,
                dropped_frames: 0,
            })),
            running: Arc::new(Mutex::new(false)),
        })
    }

    // Get detected display server
    pub fn get_display_server(&self) -> DisplayServer {
        self.display_server.clone()
    }

    // Get available monitors
    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        self.monitors.clone()
    }

    // Refresh monitor list
    pub fn refresh_monitors(&mut self) -> Result<(), ScreenCaptureError> {
        self.monitors = match self.display_server {
            DisplayServer::X11 => get_x11_monitors(),
            DisplayServer::Wayland => get_wayland_monitors(),
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ))
            }
        }?;
        
        Ok(())
    }

    // Update capture configuration
    pub fn update_config(&self, config: ScreenCaptureConfig) -> Result<(), ScreenCaptureError> {
        // Validate monitor index
        if config.monitor_index >= self.monitors.len() {
            return Err(ScreenCaptureError::InvalidMonitor(format!(
                "Monitor index {} out of bounds (0-{})",
                config.monitor_index,
                self.monitors.len() - 1
            )));
        }

        let mut current_config = self.config.lock().unwrap();
        *current_config = config;
        
        Ok(())
    }

    // Start screen capture
    pub fn start_capture(&mut self, window: Window) -> Result<(), ScreenCaptureError> {
        // Check if already running
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Clone necessary data for the capture thread
        let config = self.config.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();
        let display_server = self.display_server.clone();
        let monitor = self.monitors[self.config.lock().unwrap().monitor_index].clone();

        // Create the capture thread
        self.capture_thread = Some(thread::spawn(move || {
            match display_server {
                DisplayServer::X11 => {
                    x11_capture_loop(config, running, stats, window, monitor);
                }
                DisplayServer::Wayland => {
                    wayland_capture_loop(config, running, stats, window, monitor);
                }
                DisplayServer::Unknown => {
                    // This shouldn't happen as we validate display server in new()
                    eprintln!("Unknown display server in capture thread");
                    return;
                }
            }
        }));

        Ok(())
    }

    // Stop screen capture
    pub fn stop_capture(&mut self) -> Result<(), ScreenCaptureError> {
        // Set running flag to false to signal the capture thread to stop
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
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

    // Get current capture statistics
    pub fn get_stats(&self) -> CaptureStats {
        self.stats.lock().unwrap().clone()
    }
}

// X11 screen capture loop
fn x11_capture_loop(
    config: Arc<Mutex<ScreenCaptureConfig>>,
    running: Arc<Mutex<bool>>,
    stats: Arc<Mutex<CaptureStats>>,
    window: Window,
    monitor: MonitorInfo,
) {
    // FFmpeg command for X11 capture
    let mut last_frame_time = Instant::now();
    let mut frame_count: u64 = 0;
    let mut total_encode_time = 0.0;

    while *running.lock().unwrap() {
        let now = Instant::now();
        let elapsed = now.duration_since(last_frame_time).as_secs_f64();
        
        // Limit capture rate to configured FPS
        let config_guard = config.lock().unwrap();
        let target_frame_time = 1.0 / config_guard.fps as f64;
        
        if elapsed < target_frame_time {
            let sleep_time = Duration::from_secs_f64(target_frame_time - elapsed);
            thread::sleep(sleep_time);
            continue;
        }
        
        // Capture frame using FFmpeg with X11 input
        let capture_start = Instant::now();
        
        // Build FFmpeg command for X11 capture
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-f")
            .arg("x11grab")
            .arg("-video_size")
            .arg(format!("{}x{}", monitor.width, monitor.height))
            .arg("-i")
            .arg(format!(":0.0+{},0", monitor.index * monitor.width)); // Assumes horizontal monitor arrangement
        
        if config_guard.capture_cursor {
            cmd.arg("-draw_mouse").arg("1");
        } else {
            cmd.arg("-draw_mouse").arg("0");
        }
        
        // Add hardware acceleration if enabled
        match config_guard.hardware_acceleration {
            HardwareAcceleration::VAAPI => {
                cmd.arg("-hwaccel").arg("vaapi");
                // Additional VAAPI-specific settings would go here
            }
            HardwareAcceleration::NVENC => {
                cmd.arg("-hwaccel").arg("cuda");
                // For H.264, use NVENC encoder
                if let VideoCodec::H264 = config_guard.codec {
                    cmd.arg("-c:v").arg("h264_nvenc");
                }
            }
            HardwareAcceleration::QuickSync => {
                cmd.arg("-hwaccel").arg("qsv");
                // Additional QuickSync-specific settings would go here
            }
            HardwareAcceleration::None => {
                // Software encoding, no special args needed
            }
        }
        
        // Configure video codec
        match config_guard.codec {
            VideoCodec::H264 => {
                if config_guard.hardware_acceleration != HardwareAcceleration::NVENC {
                    cmd.arg("-c:v").arg("libx264");
                }
                cmd.arg("-preset").arg("ultrafast")
                    .arg("-tune").arg("zerolatency")
                    .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
            }
            VideoCodec::VP8 => {
                cmd.arg("-c:v").arg("libvpx")
                    .arg("-deadline").arg("realtime")
                    .arg("-cpu-used").arg("8")
                    .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
            }
            VideoCodec::VP9 => {
                cmd.arg("-c:v").arg("libvpx-vp9")
                    .arg("-deadline").arg("realtime")
                    .arg("-cpu-used").arg("8")
                    .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
            }
            VideoCodec::AV1 => {
                cmd.arg("-c:v").arg("libaom-av1")
                    .arg("-cpu-used").arg("8")
                    .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
            }
        }
        
        // Output format settings
        cmd.arg("-f").arg("image2pipe")
            .arg("-pix_fmt").arg("yuv420p")
            .arg("-vsync").arg("2")
            .arg("-frames:v").arg("1") // Capture a single frame
            .arg("-");
        
        // Attempt to capture the frame
        cmd.stderr(Stdio::null());
        let output = cmd.output();
        
        let capture_end = Instant::now();
        let encode_time = capture_end.duration_since(capture_start).as_secs_f64();
        total_encode_time += encode_time;
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    frame_count += 1;
                    
                    // Send the frame to the frontend
                    let frame_data = base64::encode(&output.stdout);
                    let _ = window.emit("frame", frame_data);
                    
                    // Update stats
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.frame_count = frame_count;
                    stats_guard.encode_time = encode_time;
                    stats_guard.frame_size = output.stdout.len() as u64;
                    stats_guard.fps = 1.0 / elapsed;
                    stats_guard.bitrate = ((output.stdout.len() as f64) * 8.0 / elapsed) as u64; // bits per second
                    
                    // Also send stats to frontend
                    let _ = window.emit("capture_stats", stats_guard.clone());
                } else {
                    eprintln!("FFmpeg failed: {:?}", output.status);
                    // Increment dropped frames counter
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.dropped_frames += 1;
                }
            }
            Err(e) => {
                eprintln!("Error executing FFmpeg: {}", e);
                // Increment dropped frames counter
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.dropped_frames += 1;
            }
        }
        
        last_frame_time = now;
    }
}

// Wayland screen capture loop using pipewire-portal
fn wayland_capture_loop(
    config: Arc<Mutex<ScreenCaptureConfig>>,
    running: Arc<Mutex<bool>>,
    stats: Arc<Mutex<CaptureStats>>,
    window: Window,
    monitor: MonitorInfo,
) {
    // For Wayland, we'll use the PipeWire portal which is the recommended way
    // to capture screen on Wayland
    
    let mut last_frame_time = Instant::now();
    let mut frame_count: u64 = 0;
    let mut total_encode_time = 0.0;
    
    // Initialize PipeWire session
    let portal_process = match Command::new("grim")
        .arg("-t")
        .arg("png")
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn() {
            Ok(process) => process,
            Err(e) => {
                eprintln!("Failed to start grim: {}", e);
                return;
            }
        };
        
    // Check if we have access to screen capture
    if let Some(status) = portal_process.wait_with_output().ok() {
        if !status.status.success() {
            eprintln!("Screen capture permission denied or not available");
            return;
        }
    }
    
    while *running.lock().unwrap() {
        let now = Instant::now();
        let elapsed = now.duration_since(last_frame_time).as_secs_f64();
        
        // Limit capture rate to configured FPS
        let config_guard = config.lock().unwrap();
        let target_frame_time = 1.0 / config_guard.fps as f64;
        
        if elapsed < target_frame_time {
            let sleep_time = Duration::from_secs_f64(target_frame_time - elapsed);
            thread::sleep(sleep_time);
            continue;
        }
        
        // Using wl-screencast and pipewire for Wayland capture
        // Build GStreamer pipeline command for Wayland capture
        let capture_start = Instant::now();
        
        // Use xdg-desktop-portal and pipewire directly
        let mut cmd = Command::new("grim");
        if monitor.index > 0 {
            // For specific monitors, we need to specify the output name
            // This is a simplification; actual implementation needs to use wlr-randr or similar
            cmd.arg("-o").arg(&monitor.name);
        }
        cmd.arg("-t").arg("png").arg("-");
        
        // Execute and get frame
        cmd.stderr(Stdio::null());
        let output = cmd.output();
        
        let capture_end = Instant::now();
        let encode_time = capture_end.duration_since(capture_start).as_secs_f64();
        total_encode_time += encode_time;
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    frame_count += 1;
                    
                    // Convert PNG to desired codec using FFmpeg
                    let mut transcode_cmd = Command::new("ffmpeg");
                    transcode_cmd.arg("-i")
                        .arg("-") // Input from stdin
                        .arg("-f")
                        .arg("image2");
                    
                    // Apply hardware acceleration if configured
                    match config_guard.hardware_acceleration {
                        HardwareAcceleration::VAAPI => {
                            transcode_cmd.arg("-hwaccel").arg("vaapi");
                        }
                        HardwareAcceleration::NVENC => {
                            transcode_cmd.arg("-hwaccel").arg("cuda");
                            // For H.264, use NVENC encoder
                            if let VideoCodec::H264 = config_guard.codec {
                                transcode_cmd.arg("-c:v").arg("h264_nvenc");
                            }
                        }
                        HardwareAcceleration::QuickSync => {
                            transcode_cmd.arg("-hwaccel").arg("qsv");
                        }
                        HardwareAcceleration::None => {
                            // Software encoding, no special args needed
                        }
                    }
                    
                    // Configure video codec
                    match config_guard.codec {
                        VideoCodec::H264 => {
                            if config_guard.hardware_acceleration != HardwareAcceleration::NVENC {
                                transcode_cmd.arg("-c:v").arg("libx264");
                            }
                            transcode_cmd.arg("-preset").arg("ultrafast")
                                .arg("-tune").arg("zerolatency")
                                .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
                        }
                        VideoCodec::VP8 => {
                            transcode_cmd.arg("-c:v").arg("libvpx")
                                .arg("-deadline").arg("realtime")
                                .arg("-cpu-used").arg("8")
                                .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
                        }
                        VideoCodec::VP9 => {
                            transcode_cmd.arg("-c:v").arg("libvpx-vp9")
                                .arg("-deadline").arg("realtime")
                                .arg("-cpu-used").arg("8")
                                .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
                        }
                        VideoCodec::AV1 => {
                            transcode_cmd.arg("-c:v").arg("libaom-av1")
                                .arg("-cpu-used").arg("8")
                                .arg("-crf").arg((51 - (config_guard.quality / 2)).to_string());
                        }
                    }
                    
                    // Output settings
                    transcode_cmd.arg("-f").arg("image2pipe")
                        .arg("-pix_fmt").arg("yuv420p")
                        .arg("-frames:v").arg("1") // Process a single frame
                        .arg("-");
                    
                    transcode_cmd.stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null());
                    
                    // Execute transcode
                    let mut child = match transcode_cmd.spawn() {
                        Ok(child) => child,
                        Err(e) => {
                            eprintln!("Failed to start FFmpeg for transcoding: {}", e);
                            continue;
                        }
                    };
                    
                    // Write PNG data to FFmpeg stdin
                    if let Some(stdin) = &mut child.stdin {
                        if let Err(e) = stdin.write_all(&output.stdout) {
                            eprintln!("Failed to write to FFmpeg stdin: {}", e);
                            continue;
                        }
                    }
                    
                    // Get transcoded frame
                    let transcode_output = match child.wait_with_output() {
                        Ok(output) => output,
                        Err(e) => {
                            eprintln!("Failed to get FFmpeg output: {}", e);
                            continue;
                        }
                    };
                    
                    if transcode_output.status.success() {
                        // Send the frame to the frontend
                        let frame_data = base64::encode(&transcode_output.stdout);
                        let _ = window.emit("frame", frame_data);
                        
                        // Update stats
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.frame_count = frame_count;
                        stats_guard.encode_time = encode_time;
                        stats_guard.frame_size = transcode_output.stdout.len() as u64;
                        stats_guard.fps = 1.0 / elapsed;
                        stats_guard.bitrate = ((transcode_output.stdout.len() as f64) * 8.0 / elapsed) as u64;
                        
                        // Also send stats to frontend
                        let _ = window.emit("capture_stats", stats_guard.clone());
                    } else {
                        eprintln!("FFmpeg transcoding failed");
                        // Increment dropped frames counter
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.dropped_frames += 1;
                    }
                } else {
                    eprintln!("Grim failed: {:?}", output.status);
                    // Increment dropped frames counter
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.dropped_frames += 1;
                }
            }
            Err(e) => {
                eprintln!("Error executing grim: {}", e);
                // Increment dropped frames counter
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.dropped_frames += 1;
            }
        }
        
        last_frame_time = now;
    }
}

// Helper functions

// Detect which display server is being used
fn detect_display_server() -> Result<DisplayServer, ScreenCaptureError> {
    // Check for Wayland
    if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
        if !wayland_display.is_empty() {
            return Ok(DisplayServer::Wayland);
        }
    }
    
    // Check for X11
    if let Ok(display) = std::env::var("DISPLAY") {
        if !display.is_empty() {
            return Ok(DisplayServer::X11);
        }
    }
    
    // No supported display server found
    Err(ScreenCaptureError::DisplayServerError(
        "No supported display server detected".to_string(),
    ))
}

// Get monitor information for X11
fn get_x11_monitors() -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
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
                        monitors.push(MonitorInfo {
                            index: monitor_index,
                            name,
                            width,
                            height,
                            refresh_rate: None, // xrandr --listmonitors doesn't provide refresh rate
                            primary: line.contains("primary"),
                        });
                    }
                }
            }
        }
    }
    
    // If no monitors were found, try fallback method
    if monitors.is_empty() {
        // Get more detailed information with standard xrandr output
        let detailed_output = Command::new("xrandr")
            .output()
            .map_err(|e| {
                ScreenCaptureError::DisplayServerError(format!("Failed to execute xrandr: {}", e))
            })?;
        
        if !detailed_output.status.success() {
            return Err(ScreenCaptureError::DisplayServerError(
                "xrandr returned an error".to_string(),
            ));
        }
        
        let detailed_str = String::from_utf8_lossy(&detailed_output.stdout);
        let mut lines = detailed_str.lines();
        let mut index = 0;
        
        while let Some(line) = lines.next() {
            // Look for connected outputs
            if line.contains(" connected ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts[0].to_string();
                let primary = line.contains("primary");
                
                // Find the resolution in the next line if it's not in this one
                let res_line = if line.contains("x") { line } else { lines.next().unwrap_or("") };
                
                // Extract resolution and refresh rate
                let res_parts: Vec<&str> = res_line.split_whitespace().collect();
                for part in res_parts {
                    if part.contains("x") {
                        let res_details: Vec<&str> = part.split('+').collect();
                        let resolution = res_details[0];
                        let res_components: Vec<&str> = resolution.split('x').collect();
                        
                        if res_components.len() >= 2 {
                            if let (Ok(width), Ok(height)) = (
                                res_components[0].parse::<u32>(),
                                res_components[1].parse::<u32>(),
                            ) {
                                // Extract refresh rate if present
                                let mut refresh_rate = None;
                                if let Some(rate_part) = res_parts.iter().find(|p| p.ends_with("Hz")) {
                                    if let Ok(rate) = rate_part.trim_end_matches("Hz").trim_end_matches('*').trim_end_matches('+').parse::<f64>() {
                                        refresh_rate = Some(rate);
                                    }
                                }
                                
                                monitors.push(MonitorInfo {
                                    index,
                                    name,
                                    width,
                                    height,
                                    refresh_rate,
                                    primary,
                                });
                                
                                index += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if monitors.is_empty() {
        return Err(ScreenCaptureError::DisplayServerError(
            "No monitors detected".to_string(),
        ));
    }
    
    Ok(monitors)
}

// Get monitor information for Wayland
fn get_wayland_monitors() -> Result<Vec<MonitorInfo>, ScreenCaptureError> {
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
    
    // If both fail, try to use most basic detection
    // For now, just return a single monitor that represents the whole screen
    let monitors = vec![MonitorInfo {
        index: 0,
        name: "Wayland-0".to_string(),
        width: 1920, // Default assumption
        height: 1080, // Default assumption
        refresh_rate: Some(60.0), // Default assumption
        primary: true,
    }];
    
    Ok(monitors)
}

// Parse wlr-randr output
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
                primary: false, // Will be updated if found
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

// Parse swaymsg output (JSON)
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
                            
                            let refresh_rate = output.get("refresh")
                                .and_then(|v| v.as_f64());
                            
                            monitors.push(MonitorInfo {
                                index,
                                name: name.to_string(),
                                width,
                                height,
                                refresh_rate,
                                primary,
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
