// screen_capture/utils.rs - Helper functions for screen capture

use std::process::Command;
use std::io::{Error as IoError, ErrorKind};
use crate::screen_capture::error::ScreenCaptureError;

/// Get current CPU usage
pub fn get_cpu_usage() -> Result<f32, ScreenCaptureError> {
    #[cfg(target_os = "linux")]
    {
        // Read /proc/stat to get CPU information
        let output = std::fs::read_to_string("/proc/stat")
            .map_err(|e| ScreenCaptureError::CaptureError(format!("Failed to read /proc/stat: {}", e)))?;
        
        // Parse CPU line (first line)
        if let Some(line) = output.lines().next() {
            let mut values = line.split_whitespace();
            
            // Skip "cpu" prefix
            values.next();
            
            // Read CPU time values
            let user = values.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            let nice = values.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            let system = values.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            let idle = values.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            
            // Calculate total and idle time
            let total = user + nice + system + idle;
            
            // Calculate usage percentage
            if total > 0 {
                let usage = 100.0 * (1.0 - (idle as f32 / total as f32));
                return Ok(usage);
            }
        }
        
        // Default if we couldn't calculate
        Ok(50.0)
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Return a default value for non-Linux systems
        Ok(50.0)
    }
}

/// Check if FFmpeg is installed and get its version
pub fn check_ffmpeg() -> Result<String, ScreenCaptureError> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|e| ScreenCaptureError::InitializationFailed(
            format!("Failed to execute ffmpeg: {}. Make sure FFmpeg is installed.", e)
        ))?;
    
    if !output.status.success() {
        return Err(ScreenCaptureError::InitializationFailed(
            "FFmpeg returned an error when checking version.".to_string()
        ));
    }
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    let version = version_str.lines().next()
        .unwrap_or("Unknown version")
        .to_string();
    
    Ok(version)
}

/// Check if a specific hardware acceleration method is available
pub fn check_hardware_acceleration(method: &crate::screen_capture::types::HardwareAcceleration) 
    -> Result<bool, ScreenCaptureError> {
    match method {
        crate::screen_capture::types::HardwareAcceleration::VAAPI => check_vaapi(),
        crate::screen_capture::types::HardwareAcceleration::NVENC => check_nvenc(),
        crate::screen_capture::types::HardwareAcceleration::QuickSync => check_quicksync(),
        crate::screen_capture::types::HardwareAcceleration::None => Ok(true),
    }
}

/// Check if VAAPI is available
fn check_vaapi() -> Result<bool, ScreenCaptureError> {
    // Check if the VAAPI device exists
    if std::path::Path::new("/dev/dri/renderD128").exists() {
        // Check if FFmpeg supports VAAPI
        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-hwaccels")
            .output()
            .map_err(|e| ScreenCaptureError::InitializationFailed(format!("Failed to check FFmpeg hwaccels: {}", e)))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        return Ok(output_str.contains("vaapi"));
    }
    
    Ok(false)
}

/// Check if NVENC (NVIDIA encoder) is available
fn check_nvenc() -> Result<bool, ScreenCaptureError> {
    // Check if FFmpeg supports NVENC
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-encoders")
        .output()
        .map_err(|e| ScreenCaptureError::InitializationFailed(format!("Failed to check FFmpeg encoders: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("h264_nvenc"))
}

/// Check if QuickSync is available
fn check_quicksync() -> Result<bool, ScreenCaptureError> {
    // Check if FFmpeg supports QuickSync
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-encoders")
        .output()
        .map_err(|e| ScreenCaptureError::InitializationFailed(format!("Failed to check FFmpeg encoders: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("h264_qsv"))
}

/// Create a temporary directory for screen capture artifacts
pub fn create_temp_directory() -> Result<std::path::PathBuf, ScreenCaptureError> {
    let temp_dir = std::env::temp_dir().join("smoldesk_capture");
    
    // Create directory if it doesn't exist
    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| ScreenCaptureError::InitializationFailed(
                format!("Failed to create temporary directory: {}", e)
            ))?;
    }
    
    Ok(temp_dir)
}

/// Clean up temporary directory
pub fn cleanup_temp_directory() -> Result<(), ScreenCaptureError> {
    let temp_dir = std::env::temp_dir().join("smoldesk_capture");
    
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| ScreenCaptureError::CaptureError(
                format!("Failed to clean up temporary directory: {}", e)
            ))?;
    }
    
    Ok(())
}

/// Generate a unique identifier for a capture session
pub fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let random = rand::random::<u16>();
    
    format!("capture_{}_{}", timestamp, random)
}

/// Convert frame data to base64 (for compatibility with old API)
pub fn frame_to_base64(data: &[u8]) -> String {
    base64::encode(data)
}

/// Get available video codecs supported by current FFmpeg installation
pub fn get_available_codecs() -> Result<Vec<String>, ScreenCaptureError> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-encoders")
        .output()
        .map_err(|e| ScreenCaptureError::InitializationFailed(format!("Failed to check FFmpeg encoders: {}", e)))?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut codecs = Vec::new();
    
    // Check for common video codecs
    if output_str.contains(" h264 ") || output_str.contains("libx264") || output_str.contains("h264_") {
        codecs.push("H264".to_string());
    }
    
    if output_str.contains(" vp8 ") || output_str.contains("libvpx") {
        codecs.push("VP8".to_string());
    }
    
    if output_str.contains(" vp9 ") || output_str.contains("libvpx-vp9") {
        codecs.push("VP9".to_string());
    }
    
    if output_str.contains(" av1 ") || output_str.contains("libaom-av1") {
        codecs.push("AV1".to_string());
    }
    
    // If no codecs were found, at least include H264 as it's most common
    if codecs.is_empty() {
        codecs.push("H264".to_string());
    }
    
    Ok(codecs)
}

/// Get available hardware acceleration methods
pub fn get_available_hardware_acceleration() -> Result<Vec<String>, ScreenCaptureError> {
    let mut methods = vec!["None".to_string()];
    
    if check_vaapi()? {
        methods.push("VAAPI".to_string());
    }
    
    if check_nvenc()? {
        methods.push("NVENC".to_string());
    }
    
    if check_quicksync()? {
        methods.push("QuickSync".to_string());
    }
    
    Ok(methods)
}

/// Kill a process by PID
pub fn kill_process(pid: u32) -> Result<(), IoError> {
    #[cfg(target_family = "unix")]
    {
        use std::process::Command;
        
        let status = Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .status()?;
        
        if status.success() {
            Ok(())
        } else {
            Err(IoError::new(ErrorKind::Other, "Failed to kill process"))
        }
    }
    
    #[cfg(target_family = "windows")]
    {
        use std::process::Command;
        
        let status = Command::new("taskkill")
            .arg("/F")
            .arg("/PID")
            .arg(pid.to_string())
            .status()?;
        
        if status.success() {
            Ok(())
        } else {
            Err(IoError::new(ErrorKind::Other, "Failed to kill process"))
        }
    }
}
