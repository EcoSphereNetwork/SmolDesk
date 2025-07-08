// forwarder_trait.rs - Common interface for input forwarders

use crate::input_forwarding::types::*;
use crate::input_forwarding::error::InputForwardingError;

/// ImprovedInputForwarder trait defines the common interface for all input forwarders
/// regardless of the underlying display server or implementation details.
pub trait ImprovedInputForwarder: Send + Sync {
    /// Forward an input event to the operating system
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    
    /// Enable or disable input forwarding
    fn set_enabled(&self, enabled: bool);
    
    /// Check if input forwarding is currently enabled
    fn is_enabled(&self) -> bool;
    
    /// Configure multi-monitor settings
    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError>;
    
    /// Handle special system commands like Alt+Tab, Win+D, etc.
    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError>;
    
    /// Handle touch gestures with optional direction and magnitude
    fn handle_gesture(
        &self, 
        gesture: &TouchGesture, 
        direction: Option<&GestureDirection>, 
        magnitude: Option<f32>
    ) -> Result<(), InputForwardingError>;
}
