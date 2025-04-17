// screen_capture/manager.rs - Screen capture manager implementation

use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Window;

use crate::screen_capture::types::{DisplayServer, CaptureStats, MonitorInfo, FrameData, ScreenCapturer, MonitorDetector};
use crate::screen_capture::error::ScreenCaptureError;
use crate::screen_capture::config::ScreenCaptureConfig;
use crate::screen_capture::buffer::{StreamBuffer, DropMode};
use crate::screen_capture::quality::AdaptiveQualityController;
use crate::screen_capture::x11::{X11ScreenCapturer, X11MonitorDetector, get_x11_monitors};
use crate::screen_capture::wayland::{WaylandScreenCapturer, WaylandMonitorDetector, get_wayland_monitors};
use crate::screen_capture::utils;

/// Screen capture manager
pub struct ScreenCaptureManager {
    /// Current display server type
    display_server: DisplayServer,
    
    /// Screen capture configuration
    config: Arc<Mutex<ScreenCaptureConfig>>,
    
    /// Available monitors
    monitors: Vec<MonitorInfo>,
    
    /// Statistics
    stats: Arc<Mutex<CaptureStats>>,
    
    /// Whether capture is running
    running: Arc<Mutex<bool>>,
    
    /// Stream buffer
    stream_buffer: Arc<Mutex<StreamBuffer>>,
    
    /// Quality controller
    quality_controller: Arc<Mutex<AdaptiveQualityController>>,
    
    /// The actual screen capturer implementation
    capturer: Option<Box<dyn ScreenCapturer>>,
}

impl ScreenCaptureManager {
    /// Create a new screen capture manager
    pub fn new() -> Result<Self, ScreenCaptureError> {
        // Detect display server
        let display_server = detect_display_server()?;
        
        // Get available monitors
        let monitors = match display_server {
            DisplayServer::X11 => get_x11_monitors(),
            DisplayServer::Wayland => get_wayland_monitors(),
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ))
            }
        }?;
        
        // Create default configuration
        let default_config = ScreenCaptureConfig::default();
        
        // Create quality controller with default configuration
        let quality_controller = AdaptiveQualityController::new(default_config.quality, None);
        
        // Create stream buffer
        // Buffer size based on FPS and latency target (e.g., 3 seconds of frames)
        let buffer_size = (default_config.fps * 3) as usize;
        let stream_buffer = StreamBuffer::new(buffer_size, 10, default_config.fps, DropMode::DropOldest);
        
        // Create default stats
        let stats = CaptureStats {
            fps: 0.0,
            bitrate: 0,
            encode_time: 0.0,
            frame_size: 0,
            frame_count: 0,
            dropped_frames: 0,
            buffer_level: 0,
            latency_estimate: 0.0,
        };
        
        Ok(ScreenCaptureManager {
            display_server,
            config: Arc::new(Mutex::new(default_config)),
            monitors,
            stats: Arc::new(Mutex::new(stats)),
            running: Arc::new(Mutex::new(false)),
            stream_buffer: Arc::new(Mutex::new(stream_buffer)),
            quality_controller: Arc::new(Mutex::new(quality_controller)),
            capturer: None,
        })
    }
    
    /// Get detected display server
    pub fn get_display_server(&self) -> DisplayServer {
        self.display_server.clone()
    }
    
    /// Get available monitors
    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        self.monitors.clone()
    }
    
    /// Refresh monitor list
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
    
    /// Update capture configuration
    pub fn update_config(&self, config: ScreenCaptureConfig) -> Result<(), ScreenCaptureError> {
        // Validate monitor index
        if config.monitor_index >= self.monitors.len() {
            return Err(ScreenCaptureError::InvalidMonitor(format!(
                "Monitor index {} out of bounds (0-{})",
                config.monitor_index,
                self.monitors.len() - 1
            )));
        }
        
        // Update buffer size if FPS changed
        {
            let mut current_config = self.config.lock().unwrap();
            let old_fps = current_config.fps;
            
            if old_fps != config.fps {
                let mut buffer = self.stream_buffer.lock().unwrap();
                buffer.set_fps(config.fps);
            }
            
            *current_config = config;
        }
        
        // If already running, restart capture with new config
        let is_running = *self.running.lock().unwrap();
        if is_running {
            self.restart_capture()?;
        }
        
        Ok(())
    }
    
    /// Restart the capture with new configuration
    fn restart_capture(&self) -> Result<(), ScreenCaptureError> {
        // This is a simplified implementation - in a real app, you'd want to preserve
        // the window handle and restart more gracefully
        
        // Stop existing capture
        if let Some(capturer) = &self.capturer {
            capturer.stop_capture()?;
        }
        
        // Note: In a fully implemented version, you'd recreate the capturer with the 
        // new configuration and restart it. Since we don't have access to the window
        // handle at this point in the code, we'll let the caller handle restart.
        
        // Set running to false to indicate we need a full restart
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }
        
        Ok(())
    }
    
    /// Start screen capture
    pub fn start_capture(&mut self, window: Window) -> Result<(), ScreenCaptureError> {
        // Check if already running
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        // Get current configuration
        let config = self.config.clone();
        let config_guard = config.lock().unwrap();
        let monitor_index = config_guard.monitor_index;
        drop(config_guard);
        
        // Check if monitor index is valid
        if monitor_index >= self.monitors.len() {
            return Err(ScreenCaptureError::InvalidMonitor(format!(
                "Monitor index {} out of bounds (0-{})",
                monitor_index,
                self.monitors.len() - 1
            )));
        }
        
        // Get the monitor to capture
        let monitor = self.monitors[monitor_index].clone();
        
        // Clear stream buffer
        {
            let mut buffer = self.stream_buffer.lock().unwrap();
            buffer.clear();
        }
        
        // Create capturer based on display server
        let capturer: Box<dyn ScreenCapturer> = match self.display_server {
            DisplayServer::X11 => {
                let x11_capturer = X11ScreenCapturer::new(
                    self.config.clone(),
                    monitor,
                    self.stream_buffer.clone(),
                    self.quality_controller.clone(),
                    self.stats.clone()
                )?;
                
                Box::new(x11_capturer)
            },
            DisplayServer::Wayland => {
                let wayland_capturer = WaylandScreenCapturer::new(
                    self.config.clone(),
                    monitor,
                    self.stream_buffer.clone(),
                    self.quality_controller.clone(),
                    self.stats.clone()
                )?;
                
                Box::new(wayland_capturer)
            },
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ));
            }
        };
        
        // Start the capture
        capturer.start_capture()?;
        
        // Store the capturer
        self.capturer = Some(capturer);
        
        // Create a listener for frontend frame requests
        let stream_buffer = self.stream_buffer.clone();
        let _window = window.clone();
        
        // Optionally set up a thread to periodically send frames to the UI
        // This is only needed if the UI needs regular updates without explicit requests
        let _frame_sender_thread = thread::spawn(move || {
            let mut last_frame_time = std::time::Instant::now();
            
            while _window.is_visible().unwrap_or(false) {
                // Rate limit to avoid overwhelming the UI
                let elapsed = last_frame_time.elapsed();
                if elapsed < std::time::Duration::from_millis(33) {  // ~30 FPS for UI updates
                    std::thread::sleep(std::time::Duration::from_millis(33) - elapsed);
                }
                
                // Get a frame from buffer (peek, don't remove)
                let frame_preview = {
                    let stream_buf = stream_buffer.lock().unwrap();
                    stream_buf.peek_next_frame().map(|f| f.data.clone())
                };
                
                // Send to UI
                if let Some(frame_data) = frame_preview {
                    let _ = _window.emit("frame_data", utils::frame_to_base64(&frame_data));
                }
                
                last_frame_time = std::time::Instant::now();
            }
        });
        
        Ok(())
    }
    
    /// Stop screen capture
    pub fn stop_capture(&mut self) -> Result<(), ScreenCaptureError> {
        // Set running flag to false
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }
        
        // Stop the capturer if it exists
        if let Some(capturer) = &mut self.capturer {
            capturer.stop_capture()?;
        }
        
        // Remove the capturer
        self.capturer = None;
        
        Ok(())
    }
    
    /// Get a frame from the capturer
    pub fn get_next_frame(&mut self) -> Option<FrameData> {
        if let Some(capturer) = &mut self.capturer {
            capturer.get_next_frame()
        } else {
            None
        }
    }
    
    /// Get capture statistics
    pub fn get_stats(&self) -> CaptureStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Detect which display server is being used
pub fn detect_display_server() -> Result<DisplayServer, ScreenCaptureError> {
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
