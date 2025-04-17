// src-tauri/src/input_forwarding.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::process::Command;

// Input event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
}

// Mouse button types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
}

// Input event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub button: Option<MouseButton>,
    pub key_code: Option<u32>,
    pub modifiers: Option<Vec<String>>,
    pub is_pressed: Option<bool>,
    pub delta_x: Option<f32>,
    pub delta_y: Option<f32>,
}

// Input forwarding error types
#[derive(Debug)]
pub enum InputForwardingError {
    InitializationFailed(String),
    SendEventFailed(String),
    UnsupportedEvent(String),
    PermissionDenied(String),
}

impl fmt::Display for InputForwardingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputForwardingError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            InputForwardingError::SendEventFailed(msg) => write!(f, "Failed to send event: {}", msg),
            InputForwardingError::UnsupportedEvent(msg) => write!(f, "Unsupported event: {}", msg),
            InputForwardingError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl Error for InputForwardingError {}

// Display server enum
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

// Input forwarder for X11
pub struct X11InputForwarder {
    screen_width: i32,
    screen_height: i32,
    enabled: Arc<Mutex<bool>>,
}

impl X11InputForwarder {
    pub fn new(screen_width: i32, screen_height: i32) -> Result<Self, InputForwardingError> {
        // Check if we have the necessary tools installed
        let xdotool_check = Command::new("which")
            .arg("xdotool")
            .output();
        
        if xdotool_check.is_err() || !xdotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "xdotool is required for X11 input forwarding".to_string(),
            ));
        }
        
        Ok(X11InputForwarder {
            screen_width,
            screen_height,
            enabled: Arc::new(Mutex::new(true)),
        })
    }
    
    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }
    
    pub fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => self.forward_mouse_move(event),
            InputEventType::MouseButton => self.forward_mouse_button(event),
            InputEventType::MouseScroll => self.forward_mouse_scroll(event),
            InputEventType::KeyPress | InputEventType::KeyRelease => self.forward_key_event(event),
        }
    }
    
    fn forward_mouse_move(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(x), Some(y)) = (event.x, event.y) {
            // Scale coordinates if needed
            let cmd_result = Command::new("xdotool")
                .arg("mousemove")
                .arg(x.to_string())
                .arg(y.to_string())
                .output();
            
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("xdotool mousemove failed: {}", String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute xdotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse move event missing coordinates".to_string()
            ))
        }
    }
    
    fn forward_mouse_button(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let Some(button) = &event.button {
            let button_arg = match button {
                MouseButton::Left => "1",
                MouseButton::Middle => "2",
                MouseButton::Right => "3",
                MouseButton::Back => "8",
                MouseButton::Forward => "9",
                MouseButton::ScrollUp | MouseButton::ScrollDown => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Scroll events should use MouseScroll type".to_string()
                    ));
                }
            };
            
            let action = match event.is_pressed {
                Some(true) => "mousedown",
                Some(false) => "mouseup",
                None => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Mouse button event missing pressed state".to_string()
                    ));
                }
            };
            
            // Execute xdotool command
            let cmd_result = Command::new("xdotool")
                .arg(action)
                .arg(button_arg)
                .output();
            
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("xdotool {} failed: {}", action, String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute xdotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse button event missing button".to_string()
            ))
        }
    }
    
    fn forward_mouse_scroll(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
            let mut commands = Vec::new();
            
            // Handle vertical scrolling
            if delta_y != 0.0 {
                let direction = if delta_y > 0.0 { "down" } else { "up" };
                let clicks = (delta_y.abs() as i32).max(1);
                
                for _ in 0..clicks {
                    commands.push(format!("xdotool click --repeat 1 {}", if direction == "up" { "4" } else { "5" }));
                }
            }
            
            // Handle horizontal scrolling
            if delta_x != 0.0 {
                let direction = if delta_x > 0.0 { "right" } else { "left" };
                let clicks = (delta_x.abs() as i32).max(1);
                
                for _ in 0..clicks {
                    commands.push(format!("xdotool click --repeat 1 {}", if direction == "left" { "6" } else { "7" }));
                }
            }
            
            // Execute all scroll commands
            for cmd_str in commands {
                let parts: Vec<&str> = cmd_str.split_whitespace().collect();
                let mut cmd = Command::new(parts[0]);
                for arg in &parts[1..] {
                    cmd.arg(arg);
                }
                
                let cmd_result = cmd.output();
                if let Err(e) = cmd_result {
                    return Err(InputForwardingError::SendEventFailed(
                        format!("Failed to execute scroll command: {}", e)
                    ));
                }
                
                let output = cmd_result.unwrap();
                if !output.status.success() {
                    return Err(InputForwardingError::SendEventFailed(
                        format!("Scroll command failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
            }
            
            Ok(())
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse scroll event missing delta values".to_string()
            ))
        }
    }
    
    fn forward_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let Some(key_code) = event.key_code {
            let action = match event.event_type {
                InputEventType::KeyPress => "keydown",
                InputEventType::KeyRelease => "keyup",
                _ => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Invalid event type for key event".to_string()
                    ));
                }
            };
            
            // Convert key code to X11 keysym
            // This is a simplified conversion - in practice, a more comprehensive 
            // mapping would be needed
            
            // Check if modifiers are present
            let mut modifier_args = Vec::new();
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    match modifier.as_str() {
                        "shift" => modifier_args.push("shift"),
                        "ctrl" => modifier_args.push("ctrl"),
                        "alt" => modifier_args.push("alt"),
                        "meta" => modifier_args.push("super"),
                        _ => {} // Ignore unknown modifiers
                    }
                }
            }
            
            // Build xdotool command
            let mut cmd = Command::new("xdotool");
            cmd.arg(action);
            
            // Add modifiers if any
            for modifier in &modifier_args {
                cmd.arg(modifier);
            }
            
            // Add key code (converting from JavaScript key code to X11 keysym)
            // This is a simplified conversion that works for most common keys
            cmd.arg(key_code.to_string());
            
            // Execute the command
            let cmd_result = cmd.output();
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("xdotool {} failed: {}", action, String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute xdotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Key event missing key code".to_string()
            ))
        }
    }
}

// Input forwarder for Wayland
pub struct WaylandInputForwarder {
    screen_width: i32,
    screen_height: i32,
    enabled: Arc<Mutex<bool>>,
}

impl WaylandInputForwarder {
    pub fn new(screen_width: i32, screen_height: i32) -> Result<Self, InputForwardingError> {
        // Check if we have the necessary tools installed
        // For Wayland, we can use ydotool which is a Wayland version of xdotool
        let ydotool_check = Command::new("which")
            .arg("ydotool")
            .output();
        
        if ydotool_check.is_err() || !ydotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "ydotool is required for Wayland input forwarding".to_string(),
            ));
        }
        
        Ok(WaylandInputForwarder {
            screen_width,
            screen_height,
            enabled: Arc::new(Mutex::new(true)),
        })
    }
    
    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }
    
    pub fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => self.forward_mouse_move(event),
            InputEventType::MouseButton => self.forward_mouse_button(event),
            InputEventType::MouseScroll => self.forward_mouse_scroll(event),
            InputEventType::KeyPress | InputEventType::KeyRelease => self.forward_key_event(event),
        }
    }
    
    fn forward_mouse_move(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(x), Some(y)) = (event.x, event.y) {
            // ydotool mousemove x y
            let cmd_result = Command::new("ydotool")
                .arg("mousemove")
                .arg("--absolute")  // Use absolute coordinates
                .arg(x.to_string())
                .arg(y.to_string())
                .output();
            
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("ydotool mousemove failed: {}", String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse move event missing coordinates".to_string()
            ))
        }
    }
    
    fn forward_mouse_button(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let Some(button) = &event.button {
            let button_arg = match button {
                MouseButton::Left => "0x110", // BTN_LEFT
                MouseButton::Middle => "0x112", // BTN_MIDDLE
                MouseButton::Right => "0x111", // BTN_RIGHT
                MouseButton::Back => "0x116", // BTN_SIDE
                MouseButton::Forward => "0x115", // BTN_EXTRA
                MouseButton::ScrollUp | MouseButton::ScrollDown => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Scroll events should use MouseScroll type".to_string()
                    ));
                }
            };
            
            let value = match event.is_pressed {
                Some(true) => "1",  // Button down
                Some(false) => "0", // Button up
                None => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Mouse button event missing pressed state".to_string()
                    ));
                }
            };
            
            // Execute ydotool command
            // ydotool input --type EV_KEY --code BTN_LEFT --value 1/0
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(button_arg)
                .arg("--value").arg(value)
                .output();
            
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("ydotool input failed: {}", String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse button event missing button".to_string()
            ))
        }
    }
    
    fn forward_mouse_scroll(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
            // In Wayland/ydotool, we use relative mouse wheel events
            // ydotool input --type EV_REL --code REL_WHEEL --value delta
            
            let mut commands = Vec::new();
            
            // Vertical scrolling
            if delta_y != 0.0 {
                let value = if delta_y > 0.0 { "-1" } else { "1" };
                let repeats = (delta_y.abs() as i32).max(1);
                
                for _ in 0..repeats {
                    commands.push(vec![
                        "ydotool", "input", "--type", "EV_REL", "--code", "REL_WHEEL", "--value", value
                    ]);
                }
            }
            
            // Horizontal scrolling
            if delta_x != 0.0 {
                let value = if delta_x > 0.0 { "-1" } else { "1" };
                let repeats = (delta_x.abs() as i32).max(1);
                
                for _ in 0..repeats {
                    commands.push(vec![
                        "ydotool", "input", "--type", "EV_REL", "--code", "REL_HWHEEL", "--value", value
                    ]);
                }
            }
            
            // Execute all commands
            for cmd_args in commands {
                let mut cmd = Command::new(cmd_args[0]);
                for arg in &cmd_args[1..] {
                    cmd.arg(arg);
                }
                
                let cmd_result = cmd.output();
                if let Err(e) = cmd_result {
                    return Err(InputForwardingError::SendEventFailed(
                        format!("Failed to execute scroll command: {}", e)
                    ));
                }
                
                let output = cmd_result.unwrap();
                if !output.status.success() {
                    return Err(InputForwardingError::SendEventFailed(
                        format!("Scroll command failed: {}", String::from_utf8_lossy(&output.stderr))
                    ));
                }
            }
            
            Ok(())
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Mouse scroll event missing delta values".to_string()
            ))
        }
    }
    
    fn forward_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let Some(key_code) = event.key_code {
            // Convert JavaScript key code to Linux input event code
            // This is a simplified mapping - in practice, a more complete mapping would be needed
            let input_code = format!("0x{:x}", key_code);
            
            let value = match event.event_type {
                InputEventType::KeyPress => "1",   // Key down
                InputEventType::KeyRelease => "0", // Key up
                _ => {
                    return Err(InputForwardingError::UnsupportedEvent(
                        "Invalid event type for key event".to_string()
                    ));
                }
            };
            
            // Execute ydotool command for key event
            // ydotool input --type EV_KEY --code KEY_A --value 1/0
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(input_code)
                .arg("--value").arg(value)
                .output();
            
            match cmd_result {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(InputForwardingError::SendEventFailed(
                            format!("ydotool input failed: {}", String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Key event missing key code".to_string()
            ))
        }
    }
}

// Input forwarder factory
pub fn create_input_forwarder(
    display_server: DisplayServer,
    screen_width: i32,
    screen_height: i32,
) -> Result<Box<dyn InputForwarder>, InputForwardingError> {
    match display_server {
        DisplayServer::X11 => {
            let forwarder = X11InputForwarder::new(screen_width, screen_height)?;
            Ok(Box::new(forwarder))
        }
        DisplayServer::Wayland => {
            let forwarder = WaylandInputForwarder::new(screen_width, screen_height)?;
            Ok(Box::new(forwarder))
        }
        DisplayServer::Unknown => Err(InputForwardingError::InitializationFailed(
            "Unknown display server".to_string(),
        )),
    }
}

// Input forwarder trait
pub trait InputForwarder: Send + Sync {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
}

// Implement trait for X11 input forwarder
impl InputForwarder for X11InputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        self.forward_event(event)
    }
    
    fn set_enabled(&self, enabled: bool) {
        self.set_enabled(enabled)
    }
    
    fn is_enabled(&self) -> bool {
        self.is_enabled()
    }
}

// Implement trait for Wayland input forwarder
impl InputForwarder for WaylandInputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        self.forward_event(event)
    }
    
    fn set_enabled(&self, enabled: bool) {
        self.set_enabled(enabled)
    }
    
    fn is_enabled(&self) -> bool {
        self.is_enabled()
    }
}

// Detect the current display server
pub fn detect_display_server() -> DisplayServer {
    // Check for Wayland
    if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
        if !wayland_display.is_empty() {
            return DisplayServer::Wayland;
        }
    }
    
    // Check for X11
    if let Ok(display) = std::env::var("DISPLAY") {
        if !display.is_empty() {
            return DisplayServer::X11;
        }
    }
    
    DisplayServer::Unknown
}
