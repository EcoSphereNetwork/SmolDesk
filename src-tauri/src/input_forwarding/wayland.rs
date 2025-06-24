// wayland.rs - Wayland-specific input forwarding implementation

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::input_forwarding::types::*;
use crate::input_forwarding::error::InputForwardingError;
use crate::input_forwarding::forwarder_trait::ImprovedInputForwarder;
use crate::input_forwarding::utils;

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
        if !utils::check_tool_exists("ydotool") {
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
                    let monitors = self.monitors.lock().unwrap();
                    let (abs_x, abs_y) = utils::calculate_absolute_position(x, y, event.monitor_index, &monitors);
                    
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
                    self.handle_wayland_gesture(gesture, event.gesture_direction.as_ref(), event.gesture_magnitude)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "TouchGesture event missing gesture type".to_string()
                    ))
                }
            },
            InputEventType::SpecialCommand => {
                if let Some(command) = &event.special_command {
                    self.execute_special_command(command)
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
        utils::validate_monitor_config(&monitors)?;
        
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
