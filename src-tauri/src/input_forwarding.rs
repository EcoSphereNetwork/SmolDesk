// src-tauri/src/input_forwarding.rs - Input Forwarding Module

use serde::{Deserialize, Serialize};

// Re-export public modules
pub mod types;
pub mod error;
pub mod forwarder_trait;
pub mod x11;
pub mod wayland;
pub mod factory;
pub mod utils;

// Re-export public items for easier access
pub use types::*;
pub use error::*;
pub use forwarder_trait::*;
pub use factory::*;

// Legacy types for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub button: Option<MouseButton>,
    pub key_code: Option<u32>,
    pub modifiers: Option<Vec<String>>,
    pub is_pressed: Option<bool>,
    pub delta_x: Option<f64>,
    pub delta_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
}

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

// Convert legacy InputEvent to new format
impl From<InputEvent> for types::InputEvent {
    fn from(legacy: InputEvent) -> Self {
        types::InputEvent {
            event_type: match legacy.event_type {
                InputEventType::MouseMove => types::InputEventType::MouseMove,
                InputEventType::MouseButton => types::InputEventType::MouseButton,
                InputEventType::MouseScroll => types::InputEventType::MouseScroll,
                InputEventType::KeyPress => types::InputEventType::KeyPress,
                InputEventType::KeyRelease => types::InputEventType::KeyRelease,
            },
            x: legacy.x,
            y: legacy.y,
            button: legacy.button.map(|b| match b {
                MouseButton::Left => types::MouseButton::Left,
                MouseButton::Middle => types::MouseButton::Middle,
                MouseButton::Right => types::MouseButton::Right,
                MouseButton::Back => types::MouseButton::Back,
                MouseButton::Forward => types::MouseButton::Forward,
                MouseButton::ScrollUp => types::MouseButton::ScrollUp,
                MouseButton::ScrollDown => types::MouseButton::ScrollDown,
            }),
            key_code: legacy.key_code,
            modifiers: legacy.modifiers,
            is_pressed: legacy.is_pressed,
            delta_x: legacy.delta_x.map(|d| d as f32),
            delta_y: legacy.delta_y.map(|d| d as f32),
            monitor_index: None,
            gesture: None,
            gesture_direction: None,
            gesture_magnitude: None,
            special_command: None,
        }
    }
}

// Wrapper trait implementation for legacy compatibility
pub struct LegacyInputForwarder {
    inner: Box<dyn ImprovedInputForwarder>,
}

impl LegacyInputForwarder {
    pub fn new(inner: Box<dyn ImprovedInputForwarder>) -> Self {
        Self { inner }
    }

    pub fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        let new_event: types::InputEvent = event.clone().into();
        self.inner.forward_event(&new_event)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.inner.set_enabled(enabled)
    }

    pub fn configure_monitors(&mut self, monitors: Vec<types::MonitorConfiguration>) -> Result<(), InputForwardingError> {
        self.inner.configure_monitors(monitors)
    }
}

// Legacy factory function
pub fn create_input_forwarder() -> Result<LegacyInputForwarder, InputForwardingError> {
    let inner = create_improved_input_forwarder(None)?;
    Ok(LegacyInputForwarder::new(inner))
}
