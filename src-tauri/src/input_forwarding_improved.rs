// src-tauri/src/input_forwarding_improved.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;

// Improved Input Event Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
    TouchGesture,  // New type for touch gestures
    SpecialCommand, // New type for special commands (e.g., Win+Tab)
}

// Improved Mouse Button Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
    // Extended button types for precision trackpads
    TouchTap,
    TouchDoubleTap,
}

// Touch Gesture Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TouchGesture {
    Pinch,
    Rotate,
    ThreeFingerSwipe,
    FourFingerSwipe,
    TwoFingerScroll,
}

// Direction for gestures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureDirection {
    Left,
    Right,
    Up,
    Down,
}

// Special Commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpecialCommand {
    AppSwitcher,  // Alt+Tab / Win+Tab
    DesktopToggle, // Win+D / Show Desktop
    ScreenSnapshot, // PrintScreen / Win+Shift+S
    LockScreen,   // Win+L / Ctrl+Alt+L
    Custom(String), // Custom command
}

// Improved Input Event Structure
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
    pub monitor_index: Option<usize>, // For multi-monitor support
    pub gesture: Option<TouchGesture>, // For touch gestures
    pub gesture_direction: Option<GestureDirection>, // For gesture direction
    pub gesture_magnitude: Option<f32>, // For gesture magnitude
    pub special_command: Option<SpecialCommand>, // For special commands
}

// Configuration for multi-monitor setups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfiguration {
    pub index: usize,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f32,
    pub is_primary: bool,
}

// Input forwarding error types
#[derive(Debug)]
pub enum InputForwardingError {
    InitializationFailed(String),
    SendEventFailed(String),
    UnsupportedEvent(String),
    PermissionDenied(String),
    MonitorConfigError(String),
}

impl fmt::Display for InputForwardingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputForwardingError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            InputForwardingError::SendEventFailed(msg) => write!(f, "Failed to send event: {}", msg),
            InputForwardingError::UnsupportedEvent(msg) => write!(f, "Unsupported event: {}", msg),
            InputForwardingError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            InputForwardingError::MonitorConfigError(msg) => write!(f, "Monitor configuration error: {}", msg),
        }
    }
}

// Implementation of ImprovedInputForwarder trait for Wayland
impl ImprovedInputForwarder for ImprovedWaylandInputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    // Calculate absolute position considering monitors
                    let (abs_x, abs_y) = self.calculate_absolute_position(x, y, event.monitor_index);
                    
                    // Execute ydotool
                    let cmd_result = Command::new("ydotool")
                        .arg("mousemove")
                        .arg("--absolute")  // Use absolute coordinates
                        .arg(abs_x.to_string())
                        .arg(abs_y.to_string())
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
            },
            InputEventType::MouseButton => {
                if let (Some(button), Some(is_pressed)) = (&event.button, event.is_pressed) {
                    // For Wayland we use Linux button codes
                    let button_arg = match button {
                        MouseButton::Left => "BTN_LEFT",
                        MouseButton::Middle => "BTN_MIDDLE",
                        MouseButton::Right => "BTN_RIGHT",
                        MouseButton::Back => "BTN_SIDE",
                        MouseButton::Forward => "BTN_EXTRA",
                        MouseButton::ScrollUp | MouseButton::ScrollDown => {
                            return Err(InputForwardingError::UnsupportedEvent(
                                "Scroll events should use MouseScroll type".to_string()
                            ));
                        },
                        MouseButton::TouchTap => {
                            // Simulate left click for touch tap
                            let tap_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(true),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&tap_event)?;
                            
                            // Release after short delay
                            let release_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(false),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&release_event)?;
                            return Ok(());
                        },
                        MouseButton::TouchDoubleTap => {
                            // Simulate double-click by pressing and releasing twice
                            for _ in 0..2 {
                                // Press
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("1")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Failed to execute ydotool: {}", e)
                                    ));
                                }
                                
                                // Release
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("0")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Failed to execute ydotool: {}", e)
                                    ));
                                }
                            }
                            
                            return Ok(());
                        },
                    };
                    
                    let value = if is_pressed { "1" } else { "0" };
                    
                    // Execute ydotool command
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
                        "Mouse button event missing button or pressed state".to_string()
                    ))
                }
            },
            InputEventType::MouseScroll => {
                if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
                    // For vertical scrolling
                    if delta_y != 0.0 {
                        let value = if delta_y > 0.0 { "-1" } else { "1" };
                        let repeats = (delta_y.abs() as i32).max(1);
                        
                        for _ in 0..repeats {
                            let cmd_result = Command::new("ydotool")
                                .arg("input")
                                .arg("--type").arg("EV_REL")
                                .arg("--code").arg("REL_WHEEL")
                                .arg("--value").arg(value)
                                .output();
                            
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
                    }
                    
                    // For horizontal scrolling
                    if delta_x != 0.0 {
                        let value = if delta_x > 0.0 { "-1" } else { "1" };
                        let repeats = (delta_x.abs() as i32).max(1);
                        
                        for _ in 0..repeats {
                            let cmd_result = Command::new("ydotool")
                                .arg("input")
                                .arg("--type").arg("EV_REL")
                                .arg("--code").arg("REL_HWHEEL")
                                .arg("--value").arg(value)
                                .output();
                            
                            if let Err(e) = cmd_result {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Failed to execute horizontal scroll command: {}", e)
                                ));
                            }
                            
                            let output = cmd_result.unwrap();
                            if !output.status.success() {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Horizontal scroll command failed: {}", String::from_utf8_lossy(&output.stderr))
                                ));
                            }
                        }
                    }
                    
                    Ok(())
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "Mouse scroll event missing delta values".to_string()
                    ))
                }
            },
            InputEventType::KeyPress | InputEventType::KeyRelease => {
                self.forward_improved_key_event(event)
            },
            InputEventType::TouchGesture => {
                if let Some(gesture) = &event.gesture {
                    self.handle_gesture(gesture, event.gesture_direction.as_ref(), event.gesture_magnitude)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "TouchGesture event missing gesture type".to_string()
                    ))
                }
            },
            InputEventType::SpecialCommand => {
                if let Some(command) = &event.special_command {
                    self.handle_special_command(command)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "SpecialCommand event missing command type".to_string()
                    ))
                }
            },
        }
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError> {
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
        
        let mut monitor_config = self.monitors.lock().unwrap();
        *monitor_config = monitors;
        
        Ok(())
    }

    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        self.execute_special_command(command)
    }

    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        self.handle_wayland_gesture(gesture, direction, magnitude)
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

// Improved input forwarder trait
pub trait ImprovedInputForwarder: Send + Sync {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError>;
    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError>;
    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError>;
}

// Improved X11 input forwarder implementation
pub struct ImprovedX11InputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode to X11 keysym mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Active modifiers
    // Key combinations for special commands
    special_commands: HashMap<SpecialCommand, Vec<String>>,
}

impl ImprovedX11InputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Check if xdotool is installed
        let xdotool_check = Command::new("which")
            .arg("xdotool")
            .output();
        
        if xdotool_check.is_err() || !xdotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "xdotool is required for X11 input forwarding".to_string(),
            ));
        }
        
        // Initialize key mapping from JS keyCode to X11 keysym
        let mut key_mapping = HashMap::new();
        // Standard keys
        for i in 48..58 { key_mapping.insert(i, (i as u8 as char).to_string()); } // 0-9
        for i in 65..91 { key_mapping.insert(i, (i as u8 as char).to_lowercase().to_string()); } // A-Z
        
        // Function keys
        for i in 1..13 { key_mapping.insert(111 + i, format!("F{}", i)); }
        
        // Special keys
        key_mapping.insert(8, "BackSpace".to_string());
        key_mapping.insert(9, "Tab".to_string());
        key_mapping.insert(13, "Return".to_string());
        key_mapping.insert(16, "Shift_L".to_string());
        key_mapping.insert(17, "Control_L".to_string());
        key_mapping.insert(18, "Alt_L".to_string());
        key_mapping.insert(19, "Pause".to_string());
        key_mapping.insert(20, "Caps_Lock".to_string());
        key_mapping.insert(27, "Escape".to_string());
        key_mapping.insert(32, "space".to_string());
        key_mapping.insert(33, "Page_Up".to_string());
        key_mapping.insert(34, "Page_Down".to_string());
        key_mapping.insert(35, "End".to_string());
        key_mapping.insert(36, "Home".to_string());
        key_mapping.insert(37, "Left".to_string());
        key_mapping.insert(38, "Up".to_string());
        key_mapping.insert(39, "Right".to_string());
        key_mapping.insert(40, "Down".to_string());
        key_mapping.insert(45, "Insert".to_string());
        key_mapping.insert(46, "Delete".to_string());
        key_mapping.insert(91, "Super_L".to_string()); // Windows/Meta/Super key
        key_mapping.insert(93, "Menu".to_string());
        
        // Numpad keys
        for i in 0..10 { key_mapping.insert(96 + i, format!("KP_{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KP_Multiply".to_string());
        key_mapping.insert(107, "KP_Add".to_string());
        key_mapping.insert(109, "KP_Subtract".to_string());
        key_mapping.insert(110, "KP_Decimal".to_string());
        key_mapping.insert(111, "KP_Divide".to_string());
        
        // Initialize special commands
        let mut special_commands = HashMap::new();
        special_commands.insert(SpecialCommand::AppSwitcher, vec!["alt".to_string(), "Tab".to_string()]);
        special_commands.insert(SpecialCommand::DesktopToggle, vec!["super".to_string(), "d".to_string()]);
        special_commands.insert(SpecialCommand::ScreenSnapshot, vec!["Print".to_string()]);
        special_commands.insert(SpecialCommand::LockScreen, vec!["super".to_string(), "l".to_string()]);
        
        Ok(ImprovedX11InputForwarder {
            monitors: Arc::new(Mutex::new(Vec::new())),
            enabled: Arc::new(Mutex::new(true)),
            key_mapping,
            active_modifiers: Arc::new(Mutex::new(Vec::new())),
            special_commands,
        })
    }
    
    // Improved mouse movement calculation with multi-monitor support
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
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
    
    // Improved key event forwarding with special characters and modifiers
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // Get X11 key sym from mapping
            let key_sym = match self.key_mapping.get(&key_code) {
                Some(sym) => sym.clone(),
                None => format!("0x{:X}", key_code), // Fallback for unknown keys
            };
            
            let action = if is_pressed { "keydown" } else { "keyup" };
            
            // Manage modifiers
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // Create xdotool command
            let mut cmd = Command::new("xdotool");
            cmd.arg(action);
            
            // Add active modifiers
            for modifier in &*active_mods {
                match modifier.as_str() {
                    "shift" => cmd.arg("shift"),
                    "ctrl" => cmd.arg("ctrl"),
                    "alt" => cmd.arg("alt"),
                    "meta" => cmd.arg("super"),
                    _ => {}
                }
            }
            
            // Add key symbol
            cmd.arg(&key_sym);
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Error executing xdotool: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("xdotool {} failed: {}", action, String::from_utf8_lossy(&output.stderr))
                ));
            }
            
            Ok(())
        } else {
            Err(InputForwardingError::UnsupportedEvent("Key event missing keyCode or pressed state".to_string()))
        }
    }
    
    // Implementation of touch gestures
    fn handle_x11_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        match gesture {
            TouchGesture::TwoFingerScroll => {
                // Two-finger scroll is handled as a scroll event
                if let Some(dir) = direction {
                    let (delta_x, delta_y) = match dir {
                        GestureDirection::Left => (1.0, 0.0),
                        GestureDirection::Right => (-1.0, 0.0),
                        GestureDirection::Up => (0.0, 1.0),
                        GestureDirection::Down => (0.0, -1.0),
                    };
                    
                    let mag = magnitude.unwrap_or(1.0);
                    let scroll_event = InputEvent {
                        event_type: InputEventType::MouseScroll,
                        delta_x: Some(delta_x * mag),
                        delta_y: Some(delta_y * mag),
                        x: None,
                        y: None,
                        button: None,
                        key_code: None,
                        modifiers: None,
                        is_pressed: None,
                        monitor_index: None,
                        gesture: None,
                        gesture_direction: None,
                        gesture_magnitude: None,
                        special_command: None,
                    };
                    
                    return self.forward_event(&scroll_event);
                }
            }
        }
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError> {
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
        
        let mut monitor_config = self.monitors.lock().unwrap();
        *monitor_config = monitors;
        
        Ok(())
    }

    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        self.execute_special_command(command)
    }

    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        self.handle_x11_gesture(gesture, direction, magnitude)
    }
}

// Improved Wayland input forwarder implementation
pub struct ImprovedWaylandInputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode to Linux input event code mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Active modifiers
    special_commands: HashMap<SpecialCommand, Vec<String>>, // Key combinations for special commands
}

impl ImprovedWaylandInputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Check if ydotool is installed
        let ydotool_check = Command::new("which")
            .arg("ydotool")
            .output();
        
        if ydotool_check.is_err() || !ydotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "ydotool is required for Wayland input forwarding".to_string(),
            ));
        }
        
        // Initialize key mapping (for Wayland, a bit different from X11)
        let mut key_mapping = HashMap::new();
        
        // Standard keys (for Wayland we need Linux keycodes instead of X11 keysyms)
        for i in 48..58 { key_mapping.insert(i, format!("KEY_{}", (i - 48))); } // 0-9
        for i in 65..91 { 
            let c = (i as u8 as char).to_lowercase().next().unwrap();
            key_mapping.insert(i, format!("KEY_{}", c.to_uppercase())); 
        } // A-Z
        
        // Function keys
        for i in 1..13 { key_mapping.insert(111 + i, format!("KEY_F{}", i)); }
        
        // Special keys
        key_mapping.insert(8, "KEY_BACKSPACE".to_string());
        key_mapping.insert(9, "KEY_TAB".to_string());
        key_mapping.insert(13, "KEY_ENTER".to_string());
        key_mapping.insert(16, "KEY_LEFTSHIFT".to_string());
        key_mapping.insert(17, "KEY_LEFTCTRL".to_string());
        key_mapping.insert(18, "KEY_LEFTALT".to_string());
        key_mapping.insert(19, "KEY_PAUSE".to_string());
        key_mapping.insert(20, "KEY_CAPSLOCK".to_string());
        key_mapping.insert(27, "KEY_ESC".to_string());
        key_mapping.insert(32, "KEY_SPACE".to_string());
        key_mapping.insert(33, "KEY_PAGEUP".to_string());
        key_mapping.insert(34, "KEY_PAGEDOWN".to_string());
        key_mapping.insert(35, "KEY_END".to_string());
        key_mapping.insert(36, "KEY_HOME".to_string());
        key_mapping.insert(37, "KEY_LEFT".to_string());
        key_mapping.insert(38, "KEY_UP".to_string());
        key_mapping.insert(39, "KEY_RIGHT".to_string());
        key_mapping.insert(40, "KEY_DOWN".to_string());
        key_mapping.insert(45, "KEY_INSERT".to_string());
        key_mapping.insert(46, "KEY_DELETE".to_string());
        key_mapping.insert(91, "KEY_LEFTMETA".to_string()); // Windows/Meta/Super key
        key_mapping.insert(93, "KEY_MENU".to_string());
        
        // Numpad keys
        for i in 0..10 { key_mapping.insert(96 + i, format!("KEY_KP{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KEY_KPASTERISK".to_string());
        key_mapping.insert(107, "KEY_KPPLUS".to_string());
        key_mapping.insert(109, "KEY_KPMINUS".to_string());
        key_mapping.insert(110, "KEY_KPDOT".to_string());
        key_mapping.insert(111, "KEY_KPSLASH".to_string());
        
        // Initialize special commands
        let mut special_commands = HashMap::new();
        special_commands.insert(SpecialCommand::AppSwitcher, vec!["KEY_LEFTALT".to_string(), "KEY_TAB".to_string()]);
        special_commands.insert(SpecialCommand::DesktopToggle, vec!["KEY_LEFTMETA".to_string(), "KEY_D".to_string()]);
        special_commands.insert(SpecialCommand::ScreenSnapshot, vec!["KEY_PRINT".to_string()]);
        special_commands.insert(SpecialCommand::LockScreen, vec!["KEY_LEFTMETA".to_string(), "KEY_L".to_string()]);
        
        Ok(ImprovedWaylandInputForwarder {
            monitors: Arc::new(Mutex::new(Vec::new())),
            enabled: Arc::new(Mutex::new(true)),
            key_mapping,
            active_modifiers: Arc::new(Mutex::new(Vec::new())),
            special_commands,
        })
    }
    
    // Calculate absolute position for multi-monitor setups
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
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
    
    // Improved key event forwarding for Wayland
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // Get Linux key code from mapping
            let key_code_str = match self.key_mapping.get(&key_code) {
                Some(code) => code.clone(),
                None => format!("KEY_{}", key_code), // Fallback
            };
            
            let value = if is_pressed { "1" } else { "0" };
            
            // Manage modifiers
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // Create ydotool command
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(&key_code_str)
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
                "Key event missing keyCode or pressed state".to_string()
            ))
        }
    }
    
    // Implementation of touch gestures for Wayland
    fn handle_wayland_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        // Wayland gesture support is similar to X11, but uses ydotool
        match gesture {
            TouchGesture::TwoFingerScroll => {
                if let Some(dir) = direction {
                    let (delta_x, delta_y) = match dir {
                        GestureDirection::Left => (1.0, 0.0),
                        GestureDirection::Right => (-1.0, 0.0),
                        GestureDirection::Up => (0.0, 1.0),
                        GestureDirection::Down => (0.0, -1.0),
                    };
                    
                    let mag = magnitude.unwrap_or(1.0);
                    
                    // For Wayland we use EV_REL events
                    let rel_type = if delta_y != 0.0 { "REL_WHEEL" } else { "REL_HWHEEL" };
                    let value = if delta_y != 0.0 { 
                        if delta_y > 0.0 { "-1" } else { "1" } 
                    } else { 
                        if delta_x > 0.0 { "-1" } else { "1" } 
                    };
                    
                    let repeats = (mag.abs() as i32).max(1);
                    
                    for _ in 0..repeats {
                        let cmd_result = Command::new("ydotool")
                            .arg("input")
                            .arg("--type").arg("EV_REL")
                            .arg("--code").arg(rel_type)
                            .arg("--value").arg(value)
                            .output();
                        
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
                    
                    return Ok(());
                }
            },
            _ => {
                // For other gestures, simulate key combinations similar to X11
                // But implement them with ydotool commands
                let key_sequence = match gesture {
                    TouchGesture::Pinch => {
                        let zoom_in = magnitude.unwrap_or(0.0) > 0.0;
                        if zoom_in {
                            vec!["KEY_LEFTCTRL".to_string(), "KEY_KPPLUS".to_string()]
                        } else {
                            vec!["KEY_LEFTCTRL".to_string(), "KEY_KPMINUS".to_string()]
                        }
                    },
                    TouchGesture::ThreeFingerSwipe => {
                        if let Some(dir) = direction {
                            match dir {
                                GestureDirection::Left => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_LEFT".to_string()]
                                },
                                GestureDirection::Right => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_RIGHT".to_string()]
                                },
                                GestureDirection::Up => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_UP".to_string()]
                                },
                                GestureDirection::Down => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_DOWN".to_string()]
                                },
                            }
                        } else {
                            return Err(InputForwardingError::UnsupportedEvent(
                                "ThreeFingerSwipe requires a direction".to_string()
                            ));
                        }
                    },
                    _ => {
                        return Err(InputForwardingError::UnsupportedEvent(
                            format!("Unsupported gesture for Wayland: {:?}", gesture)
                        ));
                    }
                };
                
                // Press all keys
                for key in &key_sequence {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("1")  // keydown
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Failed to execute ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keydown failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                // Release all keys in reverse order
                for key in key_sequence.iter().rev() {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("0")  // keyup
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Failed to execute ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keyup failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent(
            "Incomplete gesture data for Wayland".to_string()
        ))
    }
    
    // Implementation of special commands for Wayland
    fn execute_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        // Get key combination for the command
        let key_sequence = match self.special_commands.get(command) {
            Some(keys) => keys,
            None => {
                // For custom commands, use direct string
                if let SpecialCommand::Custom(cmd_str) = command {
                    // Execute direct ydotool command
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("ydotool {}", cmd_str))
                        .output()
                        .map_err(|e| {
                            InputForwardingError::SendEventFailed(
                                format!("Failed to execute custom command: {}", e)
                            )
                        })?;
                    
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Custom command failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                    
                    return Ok(());
                } else {
                    return Err(InputForwardingError::UnsupportedEvent(
                        format!("No mapping for special command: {:?}", command)
                    ));
                }
            }
        };
        
        // Press all keys
        for key in key_sequence {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("1")  // keydown
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keydown failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        // Release all keys in reverse order
        for key in key_sequence.iter().rev() {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("0")  // keyup
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keyup failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        Ok(())
    }
}
            TouchGesture::Pinch => {
                // Pinch gesture simulated as Ctrl+Plus/Minus for zoom
                if let Some(mag) = magnitude {
                    let zoom_in = mag > 0.0;
                    
                    // Press Ctrl key
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Press Plus/Minus key depending on zoom direction
                    let zoom_key = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(if zoom_in { 107 } else { 109 }), // Plus or Minus
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Plus/Minus key
                    let zoom_key_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(if zoom_in { 107 } else { 109 }),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Ctrl key
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Execute events in sequence
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&zoom_key)?;
                    self.forward_event(&zoom_key_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            TouchGesture::ThreeFingerSwipe => {
                // Three-finger swipe for workspace switching (Ctrl+Alt+Arrow)
                if let Some(dir) = direction {
                    // Press Ctrl key
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Press Alt key
                    let alt_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(18), // Alt
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Determine arrow key based on direction
                    let arrow_key = match dir {
                        GestureDirection::Left => 37,  // Left
                        GestureDirection::Right => 39, // Right
                        GestureDirection::Up => 38,    // Up
                        GestureDirection::Down => 40,  // Down
                    };
                    
                    // Press arrow key
                    let arrow_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(arrow_key),
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release arrow key
                    let arrow_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(arrow_key),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Alt key
                    let alt_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(18),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Ctrl key
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Execute events in sequence
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&alt_down)?;
                    self.forward_event(&arrow_down)?;
                    self.forward_event(&arrow_up)?;
                    self.forward_event(&alt_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            _ => {
                return Err(InputForwardingError::UnsupportedEvent(
                    format!("Unsupported gesture: {:?}", gesture)
                ));
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent("Incomplete gesture data".to_string()))
    }
    
    // Implementation of special commands for X11
    fn execute_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        // Get key combination for the command
        let key_sequence = match self.special_commands.get(command) {
            Some(keys) => keys,
            None => {
                // For custom commands, use direct string
                if let SpecialCommand::Custom(cmd_str) = command {
                    // Execute direct xdotool command
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("xdotool {}", cmd_str))
                        .output()
                        .map_err(|e| {
                            InputForwardingError::SendEventFailed(
                                format!("Error executing custom command: {}", e)
                            )
                        })?;
                    
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Custom command failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                    
                    return Ok(());
                } else {
                    return Err(InputForwardingError::UnsupportedEvent(
                        format!("No mapping for special command: {:?}", command)
                    ));
                }
            }
        };
        
        // Build xdotool command for key sequence
        let mut cmd_args = vec!["key"];
        let mut key_string = String::new();
        
        for (i, key) in key_sequence.iter().enumerate() {
            if i > 0 {
                key_string.push('+');
            }
            key_string.push_str(key);
        }
        
        cmd_args.push(&key_string);
        
        // Execute command
        let output = Command::new("xdotool")
            .args(cmd_args)
            .output()
            .map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Error executing xdotool: {}", e))
            })?;
        
        if !output.status.success() {
            return Err(InputForwardingError::SendEventFailed(
                format!("xdotool key sequence failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(())
    }
}

// Implementation of ImprovedInputForwarder trait for X11
impl ImprovedInputForwarder for ImprovedX11InputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    // Calculate absolute position considering monitors
                    let (abs_x, abs_y) = self.calculate_absolute_position(x, y, event.monitor_index);
                    
                    // Execute xdotool
                    let cmd_result = Command::new("xdotool")
                        .arg("mousemove")
                        .arg(abs_x.to_string())
                        .arg(abs_y.to_string())
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
            },
            InputEventType::MouseButton => {
                if let (Some(button), Some(is_pressed)) = (&event.button, event.is_pressed) {
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
                        },
                        MouseButton::TouchTap => {
                            // Simulate left click for touch tap
                            let tap_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(true),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&tap_event)?;
                            
                            // Release after short delay
                            let release_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(false),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&release_event)?;
                            return Ok(());
                        },
                        MouseButton::TouchDoubleTap => {
                            // Use xdotool click to do a double-click
                            let cmd_result = Command::new("xdotool")
                                .arg("click")
                                .arg("--repeat")
                                .arg("2")
                                .arg("1")  // Left button
                                .output();
                            
                            return match cmd_result {
                                Ok(output) => {
                                    if output.status.success() {
                                        Ok(())
                                    } else {
                                        Err(InputForwardingError::SendEventFailed(
                                            format!("xdotool double-click failed: {}", String::from_utf8_lossy(&output.stderr))
                                        ))
                                    }
                                }
                                Err(e) => Err(InputForwardingError::SendEventFailed(
                                    format!("Failed to execute xdotool: {}", e)
                                )),
                            };
                        },
                    };
                    
                    let action = match is_pressed {
                        true => "mousedown",
                        false => "mouseup",
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
                        "Mouse button event missing button or pressed state".to_string()
                    ))
                }
            },
            InputEventType::MouseScroll => {
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




// src-tauri/src/input_forwarding_improved.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex}

// Factory function to create the appropriate input forwarder based on the display server
pub fn create_improved_input_forwarder(display_server: DisplayServer) -> Result<Box<dyn ImprovedInputForwarder>, InputForwardingError> {
    match display_server {
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

// Frontend integration interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputForwardingConfig {
    pub enable_touch_gestures: bool,
    pub enable_special_commands: bool,
    pub enable_multi_monitor: bool,
    pub keyboard_layout: String,
    pub monitors: Vec<MonitorConfiguration>,
    pub remap_keys: HashMap<String, String>,
    pub custom_commands: HashMap<String, String>,
}

// Example Tauri commands for main.rs
/*
#[tauri::command]
fn send_improved_input_event(event: InputEvent, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        forwarder.forward_event(&event)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Improved input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn configure_input_forwarding(config: InputForwardingConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &mut *input_forwarder {
        // Update multi-monitor configuration if enabled
        if config.enable_multi_monitor {
            forwarder.configure_monitors(config.monitors)
                .map_err(|e| e.to_string())?;
        }
        
        // Additional configurations could be added here
        
        Ok(())
    } else {
        Err("Improved input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn send_touch_gesture(
    gesture_type: String,
    direction: Option<String>,
    magnitude: Option<f32>,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        // Parse gesture type
        let gesture = match gesture_type.as_str() {
            "pinch" => TouchGesture::Pinch,
            "rotate" => TouchGesture::Rotate,
            "threeFingerSwipe" => TouchGesture::ThreeFingerSwipe,
            "fourFingerSwipe" => TouchGesture::FourFingerSwipe,
            "twoFingerScroll" => TouchGesture::TwoFingerScroll,
            _ => return Err(format!("Unknown gesture type: {}", gesture_type)),
        };
        
        // Parse direction if provided
        let dir = if let Some(dir_str) = direction {
            Some(match dir_str.as_str() {
                "left" => GestureDirection::Left,
                "right" => GestureDirection::Right,
                "up" => GestureDirection::Up,
                "down" => GestureDirection::Down,
                _ => return Err(format!("Unknown gesture direction: {}", dir_str)),
            })
        } else {
            None
        };
        
        // Forward gesture
        let event = InputEvent {
            event_type: InputEventType::TouchGesture,
            gesture: Some(gesture),
            gesture_direction: dir,
            gesture_magnitude: magnitude,
            x: None, y: None, button: None, key_code: None,
            modifiers: None, is_pressed: None, delta_x: None, delta_y: None,
            monitor_index: None, special_command: None,
        };
        
        forwarder.forward_event(&event)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Improved input forwarder not initialized".to_string())
    }
}

#[tauri::command]
fn execute_special_command(command_type: String, custom_cmd: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        // Parse command type
        let command = match command_type.as_str() {
            "appSwitcher" => SpecialCommand::AppSwitcher,
            "desktopToggle" => SpecialCommand::DesktopToggle,
            "screenSnapshot" => SpecialCommand::ScreenSnapshot,
            "lockScreen" => SpecialCommand::LockScreen,
            "custom" => {
                if let Some(cmd) = custom_cmd {
                    SpecialCommand::Custom(cmd)
                } else {
                    return Err("Custom command requires a command text".to_string());
                }
            },
            _ => return Err(format!("Unknown command type: {}", command_type)),
        };
        
        // Forward command
        let event = InputEvent {
            event_type: InputEventType::SpecialCommand,
            special_command: Some(command),
            x: None, y: None, button: None, key_code: None,
            modifiers: None, is_pressed: None, delta_x: None, delta_y: None,
            monitor_index: None, gesture: None, gesture_direction: None, gesture_magnitude: None,
        };
        
        forwarder.forward_event(&event)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Improved input forwarder not initialized".to_string())
    }
}
*/;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;

// Improved Input Event Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
    TouchGesture,  // New type for touch gestures
    SpecialCommand, // New type for special commands (e.g., Win+Tab)
}

// Improved Mouse Button Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
    // Extended button types for precision trackpads
    TouchTap,
    TouchDoubleTap,
}

// Touch Gesture Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TouchGesture {
    Pinch,
    Rotate,
    ThreeFingerSwipe,
    FourFingerSwipe,
    TwoFingerScroll,
}

// Direction for gestures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureDirection {
    Left,
    Right,
    Up,
    Down,
}

// Special Commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpecialCommand {
    AppSwitcher,  // Alt+Tab / Win+Tab
    DesktopToggle, // Win+D / Show Desktop
    ScreenSnapshot, // PrintScreen / Win+Shift+S
    LockScreen,   // Win+L / Ctrl+Alt+L
    Custom(String), // Custom command
}

// Improved Input Event Structure
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
    pub monitor_index: Option<usize>, // For multi-monitor support
    pub gesture: Option<TouchGesture>, // For touch gestures
    pub gesture_direction: Option<GestureDirection>, // For gesture direction
    pub gesture_magnitude: Option<f32>, // For gesture magnitude
    pub special_command: Option<SpecialCommand>, // For special commands
}

// Configuration for multi-monitor setups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfiguration {
    pub index: usize,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f32,
    pub is_primary: bool,
}

// Input forwarding error types
#[derive(Debug)]
pub enum InputForwardingError {
    InitializationFailed(String),
    SendEventFailed(String),
    UnsupportedEvent(String),
    PermissionDenied(String),
    MonitorConfigError(String),
}

impl fmt::Display for InputForwardingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputForwardingError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            InputForwardingError::SendEventFailed(msg) => write!(f, "Failed to send event: {}", msg),
            InputForwardingError::UnsupportedEvent(msg) => write!(f, "Unsupported event: {}", msg),
            InputForwardingError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            InputForwardingError::MonitorConfigError(msg) => write!(f, "Monitor configuration error: {}", msg),
        }
    }
}

// Implementation of ImprovedInputForwarder trait for Wayland
impl ImprovedInputForwarder for ImprovedWaylandInputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    // Calculate absolute position considering monitors
                    let (abs_x, abs_y) = self.calculate_absolute_position(x, y, event.monitor_index);
                    
                    // Execute ydotool
                    let cmd_result = Command::new("ydotool")
                        .arg("mousemove")
                        .arg("--absolute")  // Use absolute coordinates
                        .arg(abs_x.to_string())
                        .arg(abs_y.to_string())
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
            },
            InputEventType::MouseButton => {
                if let (Some(button), Some(is_pressed)) = (&event.button, event.is_pressed) {
                    // For Wayland we use Linux button codes
                    let button_arg = match button {
                        MouseButton::Left => "BTN_LEFT",
                        MouseButton::Middle => "BTN_MIDDLE",
                        MouseButton::Right => "BTN_RIGHT",
                        MouseButton::Back => "BTN_SIDE",
                        MouseButton::Forward => "BTN_EXTRA",
                        MouseButton::ScrollUp | MouseButton::ScrollDown => {
                            return Err(InputForwardingError::UnsupportedEvent(
                                "Scroll events should use MouseScroll type".to_string()
                            ));
                        },
                        MouseButton::TouchTap => {
                            // Simulate left click for touch tap
                            let tap_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(true),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&tap_event)?;
                            
                            // Release after short delay
                            let release_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(false),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&release_event)?;
                            return Ok(());
                        },
                        MouseButton::TouchDoubleTap => {
                            // Simulate double-click by pressing and releasing twice
                            for _ in 0..2 {
                                // Press
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("1")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Failed to execute ydotool: {}", e)
                                    ));
                                }
                                
                                // Release
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("0")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Failed to execute ydotool: {}", e)
                                    ));
                                }
                            }
                            
                            return Ok(());
                        },
                    };
                    
                    let value = if is_pressed { "1" } else { "0" };
                    
                    // Execute ydotool command
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
                        "Mouse button event missing button or pressed state".to_string()
                    ))
                }
            },
            InputEventType::MouseScroll => {
                if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
                    // For vertical scrolling
                    if delta_y != 0.0 {
                        let value = if delta_y > 0.0 { "-1" } else { "1" };
                        let repeats = (delta_y.abs() as i32).max(1);
                        
                        for _ in 0..repeats {
                            let cmd_result = Command::new("ydotool")
                                .arg("input")
                                .arg("--type").arg("EV_REL")
                                .arg("--code").arg("REL_WHEEL")
                                .arg("--value").arg(value)
                                .output();
                            
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
                    }
                    
                    // For horizontal scrolling
                    if delta_x != 0.0 {
                        let value = if delta_x > 0.0 { "-1" } else { "1" };
                        let repeats = (delta_x.abs() as i32).max(1);
                        
                        for _ in 0..repeats {
                            let cmd_result = Command::new("ydotool")
                                .arg("input")
                                .arg("--type").arg("EV_REL")
                                .arg("--code").arg("REL_HWHEEL")
                                .arg("--value").arg(value)
                                .output();
                            
                            if let Err(e) = cmd_result {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Failed to execute horizontal scroll command: {}", e)
                                ));
                            }
                            
                            let output = cmd_result.unwrap();
                            if !output.status.success() {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Horizontal scroll command failed: {}", String::from_utf8_lossy(&output.stderr))
                                ));
                            }
                        }
                    }
                    
                    Ok(())
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "Mouse scroll event missing delta values".to_string()
                    ))
                }
            },
            InputEventType::KeyPress | InputEventType::KeyRelease => {
                self.forward_improved_key_event(event)
            },
            InputEventType::TouchGesture => {
                if let Some(gesture) = &event.gesture {
                    self.handle_gesture(gesture, event.gesture_direction.as_ref(), event.gesture_magnitude)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "TouchGesture event missing gesture type".to_string()
                    ))
                }
            },
            InputEventType::SpecialCommand => {
                if let Some(command) = &event.special_command {
                    self.handle_special_command(command)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "SpecialCommand event missing command type".to_string()
                    ))
                }
            },
        }
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError> {
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
        
        let mut monitor_config = self.monitors.lock().unwrap();
        *monitor_config = monitors;
        
        Ok(())
    }

    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        self.execute_special_command(command)
    }

    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        self.handle_wayland_gesture(gesture, direction, magnitude)
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

// Improved input forwarder trait
pub trait ImprovedInputForwarder: Send + Sync {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError>;
    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError>;
    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError>;
}

// Improved X11 input forwarder implementation
pub struct ImprovedX11InputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode to X11 keysym mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Active modifiers
    // Key combinations for special commands
    special_commands: HashMap<SpecialCommand, Vec<String>>,
}

impl ImprovedX11InputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Check if xdotool is installed
        let xdotool_check = Command::new("which")
            .arg("xdotool")
            .output();
        
        if xdotool_check.is_err() || !xdotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "xdotool is required for X11 input forwarding".to_string(),
            ));
        }
        
        // Initialize key mapping from JS keyCode to X11 keysym
        let mut key_mapping = HashMap::new();
        // Standard keys
        for i in 48..58 { key_mapping.insert(i, (i as u8 as char).to_string()); } // 0-9
        for i in 65..91 { key_mapping.insert(i, (i as u8 as char).to_lowercase().to_string()); } // A-Z
        
        // Function keys
        for i in 1..13 { key_mapping.insert(111 + i, format!("F{}", i)); }
        
        // Special keys
        key_mapping.insert(8, "BackSpace".to_string());
        key_mapping.insert(9, "Tab".to_string());
        key_mapping.insert(13, "Return".to_string());
        key_mapping.insert(16, "Shift_L".to_string());
        key_mapping.insert(17, "Control_L".to_string());
        key_mapping.insert(18, "Alt_L".to_string());
        key_mapping.insert(19, "Pause".to_string());
        key_mapping.insert(20, "Caps_Lock".to_string());
        key_mapping.insert(27, "Escape".to_string());
        key_mapping.insert(32, "space".to_string());
        key_mapping.insert(33, "Page_Up".to_string());
        key_mapping.insert(34, "Page_Down".to_string());
        key_mapping.insert(35, "End".to_string());
        key_mapping.insert(36, "Home".to_string());
        key_mapping.insert(37, "Left".to_string());
        key_mapping.insert(38, "Up".to_string());
        key_mapping.insert(39, "Right".to_string());
        key_mapping.insert(40, "Down".to_string());
        key_mapping.insert(45, "Insert".to_string());
        key_mapping.insert(46, "Delete".to_string());
        key_mapping.insert(91, "Super_L".to_string()); // Windows/Meta/Super key
        key_mapping.insert(93, "Menu".to_string());
        
        // Numpad keys
        for i in 0..10 { key_mapping.insert(96 + i, format!("KP_{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KP_Multiply".to_string());
        key_mapping.insert(107, "KP_Add".to_string());
        key_mapping.insert(109, "KP_Subtract".to_string());
        key_mapping.insert(110, "KP_Decimal".to_string());
        key_mapping.insert(111, "KP_Divide".to_string());
        
        // Initialize special commands
        let mut special_commands = HashMap::new();
        special_commands.insert(SpecialCommand::AppSwitcher, vec!["alt".to_string(), "Tab".to_string()]);
        special_commands.insert(SpecialCommand::DesktopToggle, vec!["super".to_string(), "d".to_string()]);
        special_commands.insert(SpecialCommand::ScreenSnapshot, vec!["Print".to_string()]);
        special_commands.insert(SpecialCommand::LockScreen, vec!["super".to_string(), "l".to_string()]);
        
        Ok(ImprovedX11InputForwarder {
            monitors: Arc::new(Mutex::new(Vec::new())),
            enabled: Arc::new(Mutex::new(true)),
            key_mapping,
            active_modifiers: Arc::new(Mutex::new(Vec::new())),
            special_commands,
        })
    }
    
    // Improved mouse movement calculation with multi-monitor support
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
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
    
    // Improved key event forwarding with special characters and modifiers
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // Get X11 key sym from mapping
            let key_sym = match self.key_mapping.get(&key_code) {
                Some(sym) => sym.clone(),
                None => format!("0x{:X}", key_code), // Fallback for unknown keys
            };
            
            let action = if is_pressed { "keydown" } else { "keyup" };
            
            // Manage modifiers
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // Create xdotool command
            let mut cmd = Command::new("xdotool");
            cmd.arg(action);
            
            // Add active modifiers
            for modifier in &*active_mods {
                match modifier.as_str() {
                    "shift" => cmd.arg("shift"),
                    "ctrl" => cmd.arg("ctrl"),
                    "alt" => cmd.arg("alt"),
                    "meta" => cmd.arg("super"),
                    _ => {}
                }
            }
            
            // Add key symbol
            cmd.arg(&key_sym);
            
            // Execute command
            let output = cmd.output().map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Error executing xdotool: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("xdotool {} failed: {}", action, String::from_utf8_lossy(&output.stderr))
                ));
            }
            
            Ok(())
        } else {
            Err(InputForwardingError::UnsupportedEvent("Key event missing keyCode or pressed state".to_string()))
        }
    }
    
    // Implementation of touch gestures
    fn handle_x11_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        match gesture {
            TouchGesture::TwoFingerScroll => {
                // Two-finger scroll is handled as a scroll event
                if let Some(dir) = direction {
                    let (delta_x, delta_y) = match dir {
                        GestureDirection::Left => (1.0, 0.0),
                        GestureDirection::Right => (-1.0, 0.0),
                        GestureDirection::Up => (0.0, 1.0),
                        GestureDirection::Down => (0.0, -1.0),
                    };
                    
                    let mag = magnitude.unwrap_or(1.0);
                    let scroll_event = InputEvent {
                        event_type: InputEventType::MouseScroll,
                        delta_x: Some(delta_x * mag),
                        delta_y: Some(delta_y * mag),
                        x: None,
                        y: None,
                        button: None,
                        key_code: None,
                        modifiers: None,
                        is_pressed: None,
                        monitor_index: None,
                        gesture: None,
                        gesture_direction: None,
                        gesture_magnitude: None,
                        special_command: None,
                    };
                    
                    return self.forward_event(&scroll_event);
                }
            }
        }
    }

    fn set_enabled(&self, enabled: bool) {
        let mut state = self.enabled.lock().unwrap();
        *state = enabled;
    }

    fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError> {
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
        
        let mut monitor_config = self.monitors.lock().unwrap();
        *monitor_config = monitors;
        
        Ok(())
    }

    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        self.execute_special_command(command)
    }

    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        self.handle_x11_gesture(gesture, direction, magnitude)
    }
}

// Improved Wayland input forwarder implementation
pub struct ImprovedWaylandInputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode to Linux input event code mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Active modifiers
    special_commands: HashMap<SpecialCommand, Vec<String>>, // Key combinations for special commands
}

impl ImprovedWaylandInputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Check if ydotool is installed
        let ydotool_check = Command::new("which")
            .arg("ydotool")
            .output();
        
        if ydotool_check.is_err() || !ydotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "ydotool is required for Wayland input forwarding".to_string(),
            ));
        }
        
        // Initialize key mapping (for Wayland, a bit different from X11)
        let mut key_mapping = HashMap::new();
        
        // Standard keys (for Wayland we need Linux keycodes instead of X11 keysyms)
        for i in 48..58 { key_mapping.insert(i, format!("KEY_{}", (i - 48))); } // 0-9
        for i in 65..91 { 
            let c = (i as u8 as char).to_lowercase().next().unwrap();
            key_mapping.insert(i, format!("KEY_{}", c.to_uppercase())); 
        } // A-Z
        
        // Function keys
        for i in 1..13 { key_mapping.insert(111 + i, format!("KEY_F{}", i)); }
        
        // Special keys
        key_mapping.insert(8, "KEY_BACKSPACE".to_string());
        key_mapping.insert(9, "KEY_TAB".to_string());
        key_mapping.insert(13, "KEY_ENTER".to_string());
        key_mapping.insert(16, "KEY_LEFTSHIFT".to_string());
        key_mapping.insert(17, "KEY_LEFTCTRL".to_string());
        key_mapping.insert(18, "KEY_LEFTALT".to_string());
        key_mapping.insert(19, "KEY_PAUSE".to_string());
        key_mapping.insert(20, "KEY_CAPSLOCK".to_string());
        key_mapping.insert(27, "KEY_ESC".to_string());
        key_mapping.insert(32, "KEY_SPACE".to_string());
        key_mapping.insert(33, "KEY_PAGEUP".to_string());
        key_mapping.insert(34, "KEY_PAGEDOWN".to_string());
        key_mapping.insert(35, "KEY_END".to_string());
        key_mapping.insert(36, "KEY_HOME".to_string());
        key_mapping.insert(37, "KEY_LEFT".to_string());
        key_mapping.insert(38, "KEY_UP".to_string());
        key_mapping.insert(39, "KEY_RIGHT".to_string());
        key_mapping.insert(40, "KEY_DOWN".to_string());
        key_mapping.insert(45, "KEY_INSERT".to_string());
        key_mapping.insert(46, "KEY_DELETE".to_string());
        key_mapping.insert(91, "KEY_LEFTMETA".to_string()); // Windows/Meta/Super key
        key_mapping.insert(93, "KEY_MENU".to_string());
        
        // Numpad keys
        for i in 0..10 { key_mapping.insert(96 + i, format!("KEY_KP{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KEY_KPASTERISK".to_string());
        key_mapping.insert(107, "KEY_KPPLUS".to_string());
        key_mapping.insert(109, "KEY_KPMINUS".to_string());
        key_mapping.insert(110, "KEY_KPDOT".to_string());
        key_mapping.insert(111, "KEY_KPSLASH".to_string());
        
        // Initialize special commands
        let mut special_commands = HashMap::new();
        special_commands.insert(SpecialCommand::AppSwitcher, vec!["KEY_LEFTALT".to_string(), "KEY_TAB".to_string()]);
        special_commands.insert(SpecialCommand::DesktopToggle, vec!["KEY_LEFTMETA".to_string(), "KEY_D".to_string()]);
        special_commands.insert(SpecialCommand::ScreenSnapshot, vec!["KEY_PRINT".to_string()]);
        special_commands.insert(SpecialCommand::LockScreen, vec!["KEY_LEFTMETA".to_string(), "KEY_L".to_string()]);
        
        Ok(ImprovedWaylandInputForwarder {
            monitors: Arc::new(Mutex::new(Vec::new())),
            enabled: Arc::new(Mutex::new(true)),
            key_mapping,
            active_modifiers: Arc::new(Mutex::new(Vec::new())),
            special_commands,
        })
    }
    
    // Calculate absolute position for multi-monitor setups
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
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
    
    // Improved key event forwarding for Wayland
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // Get Linux key code from mapping
            let key_code_str = match self.key_mapping.get(&key_code) {
                Some(code) => code.clone(),
                None => format!("KEY_{}", key_code), // Fallback
            };
            
            let value = if is_pressed { "1" } else { "0" };
            
            // Manage modifiers
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // Create ydotool command
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(&key_code_str)
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
                "Key event missing keyCode or pressed state".to_string()
            ))
        }
    }
    
    // Implementation of touch gestures for Wayland
    fn handle_wayland_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        // Wayland gesture support is similar to X11, but uses ydotool
        match gesture {
            TouchGesture::TwoFingerScroll => {
                if let Some(dir) = direction {
                    let (delta_x, delta_y) = match dir {
                        GestureDirection::Left => (1.0, 0.0),
                        GestureDirection::Right => (-1.0, 0.0),
                        GestureDirection::Up => (0.0, 1.0),
                        GestureDirection::Down => (0.0, -1.0),
                    };
                    
                    let mag = magnitude.unwrap_or(1.0);
                    
                    // For Wayland we use EV_REL events
                    let rel_type = if delta_y != 0.0 { "REL_WHEEL" } else { "REL_HWHEEL" };
                    let value = if delta_y != 0.0 { 
                        if delta_y > 0.0 { "-1" } else { "1" } 
                    } else { 
                        if delta_x > 0.0 { "-1" } else { "1" } 
                    };
                    
                    let repeats = (mag.abs() as i32).max(1);
                    
                    for _ in 0..repeats {
                        let cmd_result = Command::new("ydotool")
                            .arg("input")
                            .arg("--type").arg("EV_REL")
                            .arg("--code").arg(rel_type)
                            .arg("--value").arg(value)
                            .output();
                        
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
                    
                    return Ok(());
                }
            },
            _ => {
                // For other gestures, simulate key combinations similar to X11
                // But implement them with ydotool commands
                let key_sequence = match gesture {
                    TouchGesture::Pinch => {
                        let zoom_in = magnitude.unwrap_or(0.0) > 0.0;
                        if zoom_in {
                            vec!["KEY_LEFTCTRL".to_string(), "KEY_KPPLUS".to_string()]
                        } else {
                            vec!["KEY_LEFTCTRL".to_string(), "KEY_KPMINUS".to_string()]
                        }
                    },
                    TouchGesture::ThreeFingerSwipe => {
                        if let Some(dir) = direction {
                            match dir {
                                GestureDirection::Left => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_LEFT".to_string()]
                                },
                                GestureDirection::Right => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_RIGHT".to_string()]
                                },
                                GestureDirection::Up => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_UP".to_string()]
                                },
                                GestureDirection::Down => {
                                    vec!["KEY_LEFTCTRL".to_string(), "KEY_LEFTALT".to_string(), "KEY_DOWN".to_string()]
                                },
                            }
                        } else {
                            return Err(InputForwardingError::UnsupportedEvent(
                                "ThreeFingerSwipe requires a direction".to_string()
                            ));
                        }
                    },
                    _ => {
                        return Err(InputForwardingError::UnsupportedEvent(
                            format!("Unsupported gesture for Wayland: {:?}", gesture)
                        ));
                    }
                };
                
                // Press all keys
                for key in &key_sequence {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("1")  // keydown
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Failed to execute ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keydown failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                // Release all keys in reverse order
                for key in key_sequence.iter().rev() {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("0")  // keyup
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Failed to execute ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keyup failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent(
            "Incomplete gesture data for Wayland".to_string()
        ))
    }
    
    // Implementation of special commands for Wayland
    fn execute_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        // Get key combination for the command
        let key_sequence = match self.special_commands.get(command) {
            Some(keys) => keys,
            None => {
                // For custom commands, use direct string
                if let SpecialCommand::Custom(cmd_str) = command {
                    // Execute direct ydotool command
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("ydotool {}", cmd_str))
                        .output()
                        .map_err(|e| {
                            InputForwardingError::SendEventFailed(
                                format!("Failed to execute custom command: {}", e)
                            )
                        })?;
                    
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Custom command failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                    
                    return Ok(());
                } else {
                    return Err(InputForwardingError::UnsupportedEvent(
                        format!("No mapping for special command: {:?}", command)
                    ));
                }
            }
        };
        
        // Press all keys
        for key in key_sequence {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("1")  // keydown
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keydown failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        // Release all keys in reverse order
        for key in key_sequence.iter().rev() {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("0")  // keyup
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Failed to execute ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keyup failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        Ok(())
    }
}
            TouchGesture::Pinch => {
                // Pinch gesture simulated as Ctrl+Plus/Minus for zoom
                if let Some(mag) = magnitude {
                    let zoom_in = mag > 0.0;
                    
                    // Press Ctrl key
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Press Plus/Minus key depending on zoom direction
                    let zoom_key = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(if zoom_in { 107 } else { 109 }), // Plus or Minus
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Plus/Minus key
                    let zoom_key_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(if zoom_in { 107 } else { 109 }),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Ctrl key
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Execute events in sequence
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&zoom_key)?;
                    self.forward_event(&zoom_key_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            TouchGesture::ThreeFingerSwipe => {
                // Three-finger swipe for workspace switching (Ctrl+Alt+Arrow)
                if let Some(dir) = direction {
                    // Press Ctrl key
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Press Alt key
                    let alt_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(18), // Alt
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Determine arrow key based on direction
                    let arrow_key = match dir {
                        GestureDirection::Left => 37,  // Left
                        GestureDirection::Right => 39, // Right
                        GestureDirection::Up => 38,    // Up
                        GestureDirection::Down => 40,  // Down
                    };
                    
                    // Press arrow key
                    let arrow_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(arrow_key),
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release arrow key
                    let arrow_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(arrow_key),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Alt key
                    let alt_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(18),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Release Ctrl key
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Execute events in sequence
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&alt_down)?;
                    self.forward_event(&arrow_down)?;
                    self.forward_event(&arrow_up)?;
                    self.forward_event(&alt_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            _ => {
                return Err(InputForwardingError::UnsupportedEvent(
                    format!("Unsupported gesture: {:?}", gesture)
                ));
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent("Incomplete gesture data".to_string()))
    }
    
    // Implementation of special commands for X11
    fn execute_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        // Get key combination for the command
        let key_sequence = match self.special_commands.get(command) {
            Some(keys) => keys,
            None => {
                // For custom commands, use direct string
                if let SpecialCommand::Custom(cmd_str) = command {
                    // Execute direct xdotool command
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("xdotool {}", cmd_str))
                        .output()
                        .map_err(|e| {
                            InputForwardingError::SendEventFailed(
                                format!("Error executing custom command: {}", e)
                            )
                        })?;
                    
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Custom command failed: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                    
                    return Ok(());
                } else {
                    return Err(InputForwardingError::UnsupportedEvent(
                        format!("No mapping for special command: {:?}", command)
                    ));
                }
            }
        };
        
        // Build xdotool command for key sequence
        let mut cmd_args = vec!["key"];
        let mut key_string = String::new();
        
        for (i, key) in key_sequence.iter().enumerate() {
            if i > 0 {
                key_string.push('+');
            }
            key_string.push_str(key);
        }
        
        cmd_args.push(&key_string);
        
        // Execute command
        let output = Command::new("xdotool")
            .args(cmd_args)
            .output()
            .map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Error executing xdotool: {}", e))
            })?;
        
        if !output.status.success() {
            return Err(InputForwardingError::SendEventFailed(
                format!("xdotool key sequence failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(())
    }
}

// Implementation of ImprovedInputForwarder trait for X11
impl ImprovedInputForwarder for ImprovedX11InputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    // Calculate absolute position considering monitors
                    let (abs_x, abs_y) = self.calculate_absolute_position(x, y, event.monitor_index);
                    
                    // Execute xdotool
                    let cmd_result = Command::new("xdotool")
                        .arg("mousemove")
                        .arg(abs_x.to_string())
                        .arg(abs_y.to_string())
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
            },
            InputEventType::MouseButton => {
                if let (Some(button), Some(is_pressed)) = (&event.button, event.is_pressed) {
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
                        },
                        MouseButton::TouchTap => {
                            // Simulate left click for touch tap
                            let tap_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(true),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&tap_event)?;
                            
                            // Release after short delay
                            let release_event = InputEvent {
                                event_type: InputEventType::MouseButton,
                                button: Some(MouseButton::Left),
                                is_pressed: Some(false),
                                x: event.x, y: event.y,
                                key_code: None, modifiers: None, delta_x: None, delta_y: None,
                                monitor_index: event.monitor_index, gesture: None, 
                                gesture_direction: None, gesture_magnitude: None, special_command: None,
                            };
                            self.forward_event(&release_event)?;
                            return Ok(());
                        },
                        MouseButton::TouchDoubleTap => {
                            // Use xdotool click to do a double-click
                            let cmd_result = Command::new("xdotool")
                                .arg("click")
                                .arg("--repeat")
                                .arg("2")
                                .arg("1")  // Left button
                                .output();
                            
                            return match cmd_result {
                                Ok(output) => {
                                    if output.status.success() {
                                        Ok(())
                                    } else {
                                        Err(InputForwardingError::SendEventFailed(
                                            format!("xdotool double-click failed: {}", String::from_utf8_lossy(&output.stderr))
                                        ))
                                    }
                                }
                                Err(e) => Err(InputForwardingError::SendEventFailed(
                                    format!("Failed to execute xdotool: {}", e)
                                )),
                            };
                        },
                    };
                    
                    let action = match is_pressed {
                        true => "mousedown",
                        false => "mouseup",
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
                        "Mouse button event missing button or pressed state".to_string()
                    ))
                }
            },
            InputEventType::MouseScroll => {
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
