// utils.rs - Common utilities for input forwarding

use std::process::Command;
use crate::input_forwarding::types::*;
use crate::input_forwarding::error::InputForwardingError;

/// Check if a specific command-line tool is installed
pub fn check_tool_exists(tool_name: &str) -> bool {
    let cmd = Command::new("which")
        .arg(tool_name)
        .output();
    
    match cmd {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Calculate absolute position on screen based on monitor configuration
pub fn calculate_absolute_position(
    x: i32, 
    y: i32, 
    monitor_index: Option<usize>,
    monitors: &[MonitorConfiguration]
) -> (i32, i32) {
    if monitors.is_empty() {
        return (x, y); // No monitor configuration, use direct position
    }
    
    let target_monitor = match monitor_index {
        Some(idx) if idx < monitors.len() => &monitors[idx],
        _ => monitors.iter().find(|m| m.is_primary).unwrap_or(&monitors[0]),
    };
    
    // Calculate absolute position relative to target monitor
    let abs_x = target_monitor.x_offset + (x as f32 * target_monitor.scale_factor) as i32;
    let abs_y = target_monitor.y_offset + (y as f32 * target_monitor.scale_factor) as i32;
    
    (abs_x, abs_y)
}

/// Validate a monitor configuration
pub fn validate_monitor_config(
    monitors: &[MonitorConfiguration]
) -> Result<(), InputForwardingError> {
    if monitors.is_empty() {
        return Err(InputForwardingError::MonitorConfigError(
            "Empty monitor configuration".to_string()
        ));
    }
    
    // Check if at least one primary monitor exists
    let has_primary = monitors.iter().any(|m| m.is_primary);
    if !has_primary {
        return Err(InputForwardingError::MonitorConfigError(
            "No primary monitor defined".to_string()
        ));
    }
    
    // Additional validation could be added here
    
    Ok(())
}

/// Execute a command with proper error handling
pub fn execute_command(
    cmd: &mut Command
) -> Result<(), InputForwardingError> {
    let output = cmd.output().map_err(|e| {
        InputForwardingError::SendEventFailed(
            format!("Failed to execute command: {}", e)
        )
    })?;
    
    if output.status.success() {
        Ok(())
    } else {
        Err(InputForwardingError::SendEventFailed(
            format!("Command failed: {}", String::from_utf8_lossy(&output.stderr))
        ))
    }
}

/// Create a keyboard mapping for numeric keys (applies to both X11 and Wayland)
pub fn create_numeric_key_mapping<F>(
    range_start: u32,
    range_end: u32, 
    formatter: F
) -> Vec<(u32, String)> 
where
    F: Fn(u32) -> String
{
    (range_start..range_end)
        .map(|i| (i, formatter(i)))
        .collect()
}
