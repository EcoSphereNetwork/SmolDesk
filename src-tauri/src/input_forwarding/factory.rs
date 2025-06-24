// factory.rs - Factory functions for creating input forwarders

use std::env;

use crate::input_forwarding::types::*;
use crate::input_forwarding::error::InputForwardingError;
use crate::input_forwarding::forwarder_trait::ImprovedInputForwarder;
use crate::input_forwarding::x11::ImprovedX11InputForwarder;
use crate::input_forwarding::wayland::ImprovedWaylandInputForwarder;

/// Create the appropriate input forwarder based on display server
/// 
/// This factory function serves as the primary entry point for consumers of the
/// input forwarding system. It will detect the current display server if none
/// is provided and create the appropriate implementation.
/// 
/// # Arguments
/// 
/// * `display_server` - Optional display server to use. If `None`, will auto-detect.
/// 
/// # Returns
/// 
/// A boxed trait object implementing `ImprovedInputForwarder`
/// 
/// # Errors
/// 
/// Returns an error if the forwarder could not be initialized.
pub fn create_improved_input_forwarder(
    display_server: Option<DisplayServer>
) -> Result<Box<dyn ImprovedInputForwarder>, InputForwardingError> {
    // Use provided display server or auto-detect
    let server = display_server.unwrap_or_else(detect_display_server);
    
    match server {
        DisplayServer::X11 => {
            let forwarder = ImprovedX11InputForwarder::new()?;
            Ok(Box::new(forwarder))
        },
        DisplayServer::Wayland => {
            let forwarder = ImprovedWaylandInputForwarder::new()?;
            Ok(Box::new(forwarder))
        },
        DisplayServer::Unknown => {
            Err(InputForwardingError::InitializationFailed(
                "Unknown display server".to_string()
            ))
        },
    }
}

/// Detect the current display server environment
/// 
/// Examines environment variables to determine the active display server.
/// 
/// # Returns
/// 
/// The detected display server type (X11, Wayland, or Unknown)
pub fn detect_display_server() -> DisplayServer {
    // Check for Wayland
    if let Ok(wayland_display) = env::var("WAYLAND_DISPLAY") {
        if !wayland_display.is_empty() {
            return DisplayServer::Wayland;
        }
    }
    
    // Check for X11
    if let Ok(display) = env::var("DISPLAY") {
        if !display.is_empty() {
            return DisplayServer::X11;
        }
    }
    
    DisplayServer::Unknown
}

/// Creates an input forwarder with a specific configuration
/// 
/// This is a convenience function that creates an input forwarder and applies
/// configuration in a single step.
/// 
/// # Arguments
/// 
/// * `config` - The input forwarding configuration to apply
/// * `display_server` - Optional display server to use. If `None`, will auto-detect.
/// 
/// # Returns
/// 
/// A boxed trait object implementing `ImprovedInputForwarder` with the configuration applied
/// 
/// # Errors
/// 
/// Returns an error if the forwarder could not be initialized or configured.
pub fn create_configured_input_forwarder(
    config: &InputForwardingConfig,
    display_server: Option<DisplayServer>
) -> Result<Box<dyn ImprovedInputForwarder>, InputForwardingError> {
    let mut forwarder = create_improved_input_forwarder(display_server)?;
    
    // Apply configuration
    if config.enable_multi_monitor {
        forwarder.configure_monitors(config.monitors.clone())?;
    }
    
    Ok(forwarder)
}
