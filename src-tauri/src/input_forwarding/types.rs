// types.rs - Core data types for input forwarding

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Display server enum
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

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
