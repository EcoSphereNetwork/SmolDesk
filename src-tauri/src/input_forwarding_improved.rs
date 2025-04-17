// src-tauri/src/input_forwarding_improved.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;

// Erweiterte Input-Event-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
    TouchGesture,  // Neuer Typ für Touch-Gesten
    SpecialCommand, // Neuer Typ für spezielle Befehle (z.B. Win+Tab)
}

// Erweiterte Maus-Button-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
    // Erweiterte Button-Typen für Präzisions-Trackpads
    TouchTap,
    TouchDoubleTap,
}

// Touch-Gesten-Typ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TouchGesture {
    Pinch,
    Rotate,
    ThreeFingerSwipe,
    FourFingerSwipe,
    TwoFingerScroll,
}

// Richtung für Gesten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureDirection {
    Left,
    Right,
    Up,
    Down,
}

// Spezielle Befehle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialCommand {
    AppSwitcher,  // Alt+Tab / Win+Tab
    DesktopToggle, // Win+D / Show Desktop
    ScreenSnapshot, // PrintScreen / Win+Shift+S
    LockScreen,   // Win+L / Ctrl+Alt+L
    Custom(String), // Benutzerdefinierter Befehl
}

// Erweiterte Input-Event-Struktur
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
    pub monitor_index: Option<usize>, // Für Multi-Monitor-Unterstützung
    pub gesture: Option<TouchGesture>, // Für Touch-Gesten
    pub gesture_direction: Option<GestureDirection>, // Für Gesten-Richtung
    pub gesture_magnitude: Option<f32>, // Für Gesten-Stärke
    pub special_command: Option<SpecialCommand>, // Für spezielle Befehle
}

// Konfiguration für Multi-Monitor-Setups
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

// Input-Weiterleitungsfehler
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
            InputForwardingError::InitializationFailed(msg) => write!(f, "Initialisierung fehlgeschlagen: {}", msg),
            InputForwardingError::SendEventFailed(msg) => write!(f, "Event-Übertragung fehlgeschlagen: {}", msg),
            InputForwardingError::UnsupportedEvent(msg) => write!(f, "Nicht unterstütztes Event: {}", msg),
            InputForwardingError::PermissionDenied(msg) => write!(f, "Zugriff verweigert: {}", msg),
            InputForwardingError::MonitorConfigError(msg) => write!(f, "Monitor-Konfigurationsfehler: {}", msg),
        }
    }
}

impl Error for InputForwardingError {}

// Verbesserte Input-Forwarder-Trait
pub trait ImprovedInputForwarder: Send + Sync {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError>;
    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError>;
    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError>;
}

// Verbesserte X11-Input-Forwarder-Implementierung
pub struct ImprovedX11InputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode zu X11 keysym Mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Aktiv gehaltene Modifikatoren
    // Tastenkombinationen für spezielle Befehle
    special_commands: HashMap<SpecialCommand, Vec<String>>,
}

impl ImprovedX11InputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Prüfen, ob xdotool installiert ist
        let xdotool_check = Command::new("which")
            .arg("xdotool")
            .output();
        
        if xdotool_check.is_err() || !xdotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "xdotool ist erforderlich für X11-Input-Weiterleitung".to_string(),
            ));
        }
        
        // Initialisiere Key-Mapping von JS keyCode zu X11 keysym
        let mut key_mapping = HashMap::new();
        // Standardtasten
        for i in 48..58 { key_mapping.insert(i, (i as u8 as char).to_string()); } // 0-9
        for i in 65..91 { key_mapping.insert(i, (i as u8 as char).to_lowercase().to_string()); } // A-Z
        
        // Funktionstasten
        for i in 1..13 { key_mapping.insert(111 + i, format!("F{}", i)); }
        
        // Spezialtasten
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
        key_mapping.insert(91, "Super_L".to_string()); // Windows/Meta/Super-Taste
        key_mapping.insert(93, "Menu".to_string());
        
        // Numpad-Tasten
        for i in 0..10 { key_mapping.insert(96 + i, format!("KP_{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KP_Multiply".to_string());
        key_mapping.insert(107, "KP_Add".to_string());
        key_mapping.insert(109, "KP_Subtract".to_string());
        key_mapping.insert(110, "KP_Decimal".to_string());
        key_mapping.insert(111, "KP_Divide".to_string());
        
        // Spezielle Befehle initialisieren
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
    
    // Verbesserte Mausbewegungsberechnung mit Multi-Monitor-Unterstützung
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
        if monitors.is_empty() {
            return (x, y); // Keine Monitor-Konfiguration, direkte Positionsverwendung
        }
        
        let target_monitor = match monitor_index {
            Some(idx) if idx < monitors.len() => &monitors[idx],
            _ => monitors.iter().find(|m| m.is_primary).unwrap_or(&monitors[0]),
        };
        
        // Berechne absolute Position relativ zum Zielmonitor
        let abs_x = target_monitor.x_offset + (x as f32 * target_monitor.scale_factor) as i32;
        let abs_y = target_monitor.y_offset + (y as f32 * target_monitor.scale_factor) as i32;
        
        (abs_x, abs_y)
    }
    
    // Verbesserte Tastenevent-Weiterleitung mit Sonderzeichen und Modifikatoren
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // X11 key sym aus dem Mapping holen
            let key_sym = match self.key_mapping.get(&key_code) {
                Some(sym) => sym.clone(),
                None => format!("0x{:X}", key_code), // Fallback für unbekannte Tasten
            };
            
            let action = if is_pressed { "keydown" } else { "keyup" };
            
            // Modifikatoren verwalten
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // xdotool-Kommando erstellen
            let mut cmd = Command::new("xdotool");
            cmd.arg(action);
            
            // Aktive Modifikatoren hinzufügen
            for modifier in &*active_mods {
                match modifier.as_str() {
                    "shift" => cmd.arg("shift"),
                    "ctrl" => cmd.arg("ctrl"),
                    "alt" => cmd.arg("alt"),
                    "meta" => cmd.arg("super"),
                    _ => {}
                }
            }
            
            // Tastensymbol hinzufügen
            cmd.arg(&key_sym);
            
            // Kommando ausführen
            let output = cmd.output().map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Fehler beim Ausführen von xdotool: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("xdotool {} fehlgeschlagen: {}", action, String::from_utf8_lossy(&output.stderr))
                ));
            }
            
            Ok(())
        } else {
            Err(InputForwardingError::UnsupportedEvent("Tastenevent fehlt keyCode oder pressed-Status".to_string()))
        }
    }
    
    // Implementierung der Touch-Gesten
    fn handle_x11_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        match gesture {
            TouchGesture::TwoFingerScroll => {
                // Two-finger scroll wird als Scrollevent behandelt
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
            },
            TouchGesture::Pinch => {
                // Pinch-Geste als Strg+Plus/Minus für Zoom simulieren
                if let Some(mag) = magnitude {
                    let zoom_in = mag > 0.0;
                    
                    // Strg-Taste drücken
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Plus/Minus-Taste drücken je nach Zoom-Richtung
                    let zoom_key = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(if zoom_in { 107 } else { 109 }), // Plus oder Minus
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Plus/Minus-Taste loslassen
                    let zoom_key_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(if zoom_in { 107 } else { 109 }),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Strg-Taste loslassen
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Events in Reihenfolge ausführen
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&zoom_key)?;
                    self.forward_event(&zoom_key_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            TouchGesture::ThreeFingerSwipe => {
                // Drei-Finger-Swipe für Workspace-Wechsel (Strg+Alt+Pfeiltaste)
                if let Some(dir) = direction {
                    // Strg-Taste drücken
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Alt-Taste drücken
                    let alt_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(18), // Alt
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Bestimme Pfeiltaste basierend auf Richtung
                    let arrow_key = match dir {
                        GestureDirection::Left => 37,  // Links
                        GestureDirection::Right => 39, // Rechts
                        GestureDirection::Up => 38,    // Oben
                        GestureDirection::Down => 40,  // Unten
                    };
                    
                    // Pfeiltaste drücken
                    let arrow_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(arrow_key),
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Pfeiltaste loslassen
                    let arrow_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(arrow_key),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Alt-Taste loslassen
                    let alt_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(18),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Strg-Taste loslassen
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Events in Reihenfolge ausführen
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
                    format!("Nicht unterstützte Geste: {:?}", gesture)
                ));
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent("Unvollständige Gestendaten".to_string()))

// Implementierung des ImprovedInputForwarder-Traits für Wayland
impl ImprovedInputForwarder for ImprovedWaylandInputForwarder {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        match event.event_type {
            InputEventType::MouseMove => {
                if let (Some(x), Some(y)) = (event.x, event.y) {
                    // Berechne absolute Position unter Berücksichtigung von Monitoren
                    let (abs_x, abs_y) = self.calculate_absolute_position(x, y, event.monitor_index);
                    
                    // Führe ydotool aus (für Wayland)
                    let cmd_result = Command::new("ydotool")
                        .arg("mousemove")
                        .arg("--absolute")  // Absolute Koordinaten
                        .arg(abs_x.to_string())
                        .arg(abs_y.to_string())
                        .output();
                    
                    match cmd_result {
                        Ok(output) => {
                            if output.status.success() {
                                Ok(())
                            } else {
                                Err(InputForwardingError::SendEventFailed(
                                    format!("ydotool mousemove fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                                ))
                            }
                        }
                        Err(e) => Err(InputForwardingError::SendEventFailed(
                            format!("Fehler beim Ausführen von ydotool: {}", e)
                        )),
                    }
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "MouseMove-Event fehlt Koordinaten".to_string()
                    ))
                }
            },
            InputEventType::MouseButton => {
                if let (Some(button), Some(is_pressed)) = (&event.button, event.is_pressed) {
                    // Für Wayland verwenden wir die Linux-Button-Codes
                    let button_code = match button {
                        MouseButton::Left => "BTN_LEFT",
                        MouseButton::Middle => "BTN_MIDDLE",
                        MouseButton::Right => "BTN_RIGHT",
                        MouseButton::Back => "BTN_SIDE",
                        MouseButton::Forward => "BTN_EXTRA",
                        MouseButton::ScrollUp | MouseButton::ScrollDown => {
                            return Err(InputForwardingError::UnsupportedEvent(
                                "Scroll-Events sollten MouseScroll-Typ verwenden".to_string()
                            ));
                        },
                        MouseButton::TouchTap => {
                            // Simuliere Linksklick für Touch-Tap
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
                            
                            // Nach kurzem Delay loslassen
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
                            // Simuliere Doppelklick durch zweimaliges Drücken und Loslassen
                            for _ in 0..2 {
                                // Drücken
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("1")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Fehler beim Ausführen von ydotool: {}", e)
                                    ));
                                }
                                
                                // Loslassen
                                let cmd_result = Command::new("ydotool")
                                    .arg("input")
                                    .arg("--type").arg("EV_KEY")
                                    .arg("--code").arg("BTN_LEFT")
                                    .arg("--value").arg("0")
                                    .output();
                                
                                if let Err(e) = cmd_result {
                                    return Err(InputForwardingError::SendEventFailed(
                                        format!("Fehler beim Ausführen von ydotool: {}", e)
                                    ));
                                }
                            }
                            
                            return Ok(());
                        },
                    };
                    
                    let value = if is_pressed { "1" } else { "0" };
                    
                    // Führe ydotool aus
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(button_code)
                        .arg("--value").arg(value)
                        .output();
                    
                    match cmd_result {
                        Ok(output) => {
                            if output.status.success() {
                                Ok(())
                            } else {
                                Err(InputForwardingError::SendEventFailed(
                                    format!("ydotool input fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                                ))
                            }
                        }
                        Err(e) => Err(InputForwardingError::SendEventFailed(
                            format!("Fehler beim Ausführen von ydotool: {}", e)
                        )),
                    }
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "MouseButton-Event fehlt Button oder pressed-Status".to_string()
                    ))
                }
            },
            InputEventType::MouseScroll => {
                if let (Some(delta_x), Some(delta_y)) = (event.delta_x, event.delta_y) {
                    // Für vertikales Scrollen
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
                                    format!("Fehler beim Ausführen des Scroll-Befehls: {}", e)
                                ));
                            }
                            
                            let output = cmd_result.unwrap();
                            if !output.status.success() {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Scroll-Befehl fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                                ));
                            }
                        }
                    }
                    
                    // Für horizontales Scrollen
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
                                    format!("Fehler beim Ausführen des horizontalen Scroll-Befehls: {}", e)
                                ));
                            }
                            
                            let output = cmd_result.unwrap();
                            if !output.status.success() {
                                return Err(InputForwardingError::SendEventFailed(
                                    format!("Horizontaler Scroll-Befehl fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                                ));
                            }
                        }
                    }
                    
                    Ok(())
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "MouseScroll-Event fehlt delta-Werte".to_string()
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
                        "TouchGesture-Event fehlt Gesten-Typ".to_string()
                    ))
                }
            },
            InputEventType::SpecialCommand => {
                if let Some(command) = &event.special_command {
                    self.handle_special_command(command)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "SpecialCommand-Event fehlt Command-Typ".to_string()
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
                "Leere Monitor-Konfiguration".to_string()
            ));
        }
        
        // Überprüfe, ob mindestens ein primärer Monitor existiert
        let has_primary = monitors.iter().any(|m| m.is_primary);
        if !has_primary {
            return Err(InputForwardingError::MonitorConfigError(
                "Kein primärer Monitor definiert".to_string()
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

// Factory-Funktion zum Erstellen des passenden Input-Forwarders basierend auf dem Display-Server
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
                "Unbekannter Display-Server".to_string()
            ))
        },
    }
}

// Frontend-Integrationsschnittstelle
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

// Erweiterung für main.rs Tauri-Kommandos

/*
// In main.rs müssen die folgenden Tauri-Befehle hinzugefügt werden:

#[tauri::command]
fn send_improved_input_event(event: InputEvent, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        forwarder.forward_event(&event)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    } else {
        Err("Verbesserter Input-Forwarder nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn configure_input_forwarding(config: InputForwardingConfig, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &mut *input_forwarder {
        // Multi-Monitor-Konfiguration aktualisieren, wenn aktiviert
        if config.enable_multi_monitor {
            forwarder.configure_monitors(config.monitors)
                .map_err(|e| e.to_string())?;
        }
        
        // Weitere Konfigurationen könnten hier hinzugefügt werden
        
        Ok(())
    } else {
        Err("Verbesserter Input-Forwarder nicht initialisiert".to_string())
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
        // Gesten-Typ parsen
        let gesture = match gesture_type.as_str() {
            "pinch" => TouchGesture::Pinch,
            "rotate" => TouchGesture::Rotate,
            "threeFingerSwipe" => TouchGesture::ThreeFingerSwipe,
            "fourFingerSwipe" => TouchGesture::FourFingerSwipe,
            "twoFingerScroll" => TouchGesture::TwoFingerScroll,
            _ => return Err(format!("Unbekannter Gesten-Typ: {}", gesture_type)),
        };
        
        // Richtung parsen, wenn vorhanden
        let dir = if let Some(dir_str) = direction {
            Some(match dir_str.as_str() {
                "left" => GestureDirection::Left,
                "right" => GestureDirection::Right,
                "up" => GestureDirection::Up,
                "down" => GestureDirection::Down,
                _ => return Err(format!("Unbekannte Gesten-Richtung: {}", dir_str)),
            })
        } else {
            None
        };
        
        // Geste weiterleiten
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
        Err("Verbesserter Input-Forwarder nicht initialisiert".to_string())
    }
}

#[tauri::command]
fn execute_special_command(command_type: String, custom_cmd: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let input_forwarder = state.improved_input_forwarder.lock().unwrap();
    
    if let Some(forwarder) = &*input_forwarder {
        // Befehlstyp parsen
        let command = match command_type.as_str() {
            "appSwitcher" => SpecialCommand::AppSwitcher,
            "desktopToggle" => SpecialCommand::DesktopToggle,
            "screenSnapshot" => SpecialCommand::ScreenSnapshot,
            "lockScreen" => SpecialCommand::LockScreen,
            "custom" => {
                if let Some(cmd) = custom_cmd {
                    SpecialCommand::Custom(cmd)
                } else {
                    return Err("Benutzerdefinierter Befehl benötigt einen Befehlstext".to_string());
                }
            },
            _ => return Err(format!("Unbekannter Befehlstyp: {}", command_type)),
        };
        
        // Befehl weiterleiten
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
        Err("Verbesserter Input-Forwarder nicht initialisiert".to_string())
    }
}
*/// src-tauri/src/input_forwarding_improved.rs

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;

// Erweiterte Input-Event-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    MouseMove,
    MouseButton,
    MouseScroll,
    KeyPress,
    KeyRelease,
    TouchGesture,  // Neuer Typ für Touch-Gesten
    SpecialCommand, // Neuer Typ für spezielle Befehle (z.B. Win+Tab)
}

// Erweiterte Maus-Button-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    ScrollUp,
    ScrollDown,
    // Erweiterte Button-Typen für Präzisions-Trackpads
    TouchTap,
    TouchDoubleTap,
}

// Touch-Gesten-Typ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TouchGesture {
    Pinch,
    Rotate,
    ThreeFingerSwipe,
    FourFingerSwipe,
    TwoFingerScroll,
}

// Richtung für Gesten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureDirection {
    Left,
    Right,
    Up,
    Down,
}

// Spezielle Befehle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialCommand {
    AppSwitcher,  // Alt+Tab / Win+Tab
    DesktopToggle, // Win+D / Show Desktop
    ScreenSnapshot, // PrintScreen / Win+Shift+S
    LockScreen,   // Win+L / Ctrl+Alt+L
    Custom(String), // Benutzerdefinierter Befehl
}

// Erweiterte Input-Event-Struktur
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
    pub monitor_index: Option<usize>, // Für Multi-Monitor-Unterstützung
    pub gesture: Option<TouchGesture>, // Für Touch-Gesten
    pub gesture_direction: Option<GestureDirection>, // Für Gesten-Richtung
    pub gesture_magnitude: Option<f32>, // Für Gesten-Stärke
    pub special_command: Option<SpecialCommand>, // Für spezielle Befehle
}

// Konfiguration für Multi-Monitor-Setups
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

// Input-Weiterleitungsfehler
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
            InputForwardingError::InitializationFailed(msg) => write!(f, "Initialisierung fehlgeschlagen: {}", msg),
            InputForwardingError::SendEventFailed(msg) => write!(f, "Event-Übertragung fehlgeschlagen: {}", msg),
            InputForwardingError::UnsupportedEvent(msg) => write!(f, "Nicht unterstütztes Event: {}", msg),
            InputForwardingError::PermissionDenied(msg) => write!(f, "Zugriff verweigert: {}", msg),
            InputForwardingError::MonitorConfigError(msg) => write!(f, "Monitor-Konfigurationsfehler: {}", msg),
        }
    }
}

impl Error for InputForwardingError {}

// Verbesserte Input-Forwarder-Trait
pub trait ImprovedInputForwarder: Send + Sync {
    fn forward_event(&self, event: &InputEvent) -> Result<(), InputForwardingError>;
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn configure_monitors(&mut self, monitors: Vec<MonitorConfiguration>) -> Result<(), InputForwardingError>;
    fn handle_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError>;
    fn handle_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError>;
}

// Verbesserte X11-Input-Forwarder-Implementierung
pub struct ImprovedX11InputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode zu X11 keysym Mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Aktiv gehaltene Modifikatoren
    // Tastenkombinationen für spezielle Befehle
    special_commands: HashMap<SpecialCommand, Vec<String>>,
}

impl ImprovedX11InputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Prüfen, ob xdotool installiert ist
        let xdotool_check = Command::new("which")
            .arg("xdotool")
            .output();
        
        if xdotool_check.is_err() || !xdotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "xdotool ist erforderlich für X11-Input-Weiterleitung".to_string(),
            ));
        }
        
        // Initialisiere Key-Mapping von JS keyCode zu X11 keysym
        let mut key_mapping = HashMap::new();
        // Standardtasten
        for i in 48..58 { key_mapping.insert(i, (i as u8 as char).to_string()); } // 0-9
        for i in 65..91 { key_mapping.insert(i, (i as u8 as char).to_lowercase().to_string()); } // A-Z
        
        // Funktionstasten
        for i in 1..13 { key_mapping.insert(111 + i, format!("F{}", i)); }
        
        // Spezialtasten
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
        key_mapping.insert(91, "Super_L".to_string()); // Windows/Meta/Super-Taste
        key_mapping.insert(93, "Menu".to_string());
        
        // Numpad-Tasten
        for i in 0..10 { key_mapping.insert(96 + i, format!("KP_{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KP_Multiply".to_string());
        key_mapping.insert(107, "KP_Add".to_string());
        key_mapping.insert(109, "KP_Subtract".to_string());
        key_mapping.insert(110, "KP_Decimal".to_string());
        key_mapping.insert(111, "KP_Divide".to_string());
        
        // Spezielle Befehle initialisieren
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
    
    // Verbesserte Mausbewegungsberechnung mit Multi-Monitor-Unterstützung
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
        if monitors.is_empty() {
            return (x, y); // Keine Monitor-Konfiguration, direkte Positionsverwendung
        }
        
        let target_monitor = match monitor_index {
            Some(idx) if idx < monitors.len() => &monitors[idx],
            _ => monitors.iter().find(|m| m.is_primary).unwrap_or(&monitors[0]),
        };
        
        // Berechne absolute Position relativ zum Zielmonitor
        let abs_x = target_monitor.x_offset + (x as f32 * target_monitor.scale_factor) as i32;
        let abs_y = target_monitor.y_offset + (y as f32 * target_monitor.scale_factor) as i32;
        
        (abs_x, abs_y)
    }
    
    // Verbesserte Tastenevent-Weiterleitung mit Sonderzeichen und Modifikatoren
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // X11 key sym aus dem Mapping holen
            let key_sym = match self.key_mapping.get(&key_code) {
                Some(sym) => sym.clone(),
                None => format!("0x{:X}", key_code), // Fallback für unbekannte Tasten
            };
            
            let action = if is_pressed { "keydown" } else { "keyup" };
            
            // Modifikatoren verwalten
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // xdotool-Kommando erstellen
            let mut cmd = Command::new("xdotool");
            cmd.arg(action);
            
            // Aktive Modifikatoren hinzufügen
            for modifier in &*active_mods {
                match modifier.as_str() {
                    "shift" => cmd.arg("shift"),
                    "ctrl" => cmd.arg("ctrl"),
                    "alt" => cmd.arg("alt"),
                    "meta" => cmd.arg("super"),
                    _ => {}
                }
            }
            
            // Tastensymbol hinzufügen
            cmd.arg(&key_sym);
            
            // Kommando ausführen
            let output = cmd.output().map_err(|e| {
                InputForwardingError::SendEventFailed(format!("Fehler beim Ausführen von xdotool: {}", e))
            })?;
            
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("xdotool {} fehlgeschlagen: {}", action, String::from_utf8_lossy(&output.stderr))
                ));
            }
            
                                Ok(())
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "MouseScroll-Event fehlt delta-Werte".to_string()
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
                        "TouchGesture-Event fehlt Gesten-Typ".to_string()
                    ))
                }
            },
            InputEventType::SpecialCommand => {
                if let Some(command) = &event.special_command {
                    self.handle_special_command(command)
                } else {
                    Err(InputForwardingError::UnsupportedEvent(
                        "SpecialCommand-Event fehlt Command-Typ".to_string()
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
                "Leere Monitor-Konfiguration".to_string()
            ));
        }
        
        // Überprüfe, ob mindestens ein primärer Monitor existiert
        let has_primary = monitors.iter().any(|m| m.is_primary);
        if !has_primary {
            return Err(InputForwardingError::MonitorConfigError(
                "Kein primärer Monitor definiert".to_string()
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

// Verbesserte Wayland-Input-Forwarder-Implementierung
pub struct ImprovedWaylandInputForwarder {
    monitors: Arc<Mutex<Vec<MonitorConfiguration>>>,
    enabled: Arc<Mutex<bool>>,
    key_mapping: HashMap<u32, String>, // JavaScript keyCode zu Linux input event code Mapping
    active_modifiers: Arc<Mutex<Vec<String>>>, // Aktiv gehaltene Modifikatoren
    special_commands: HashMap<SpecialCommand, Vec<String>>, // Tastenkombinationen für spezielle Befehle
}

impl ImprovedWaylandInputForwarder {
    pub fn new() -> Result<Self, InputForwardingError> {
        // Prüfen, ob ydotool installiert ist
        let ydotool_check = Command::new("which")
            .arg("ydotool")
            .output();
        
        if ydotool_check.is_err() || !ydotool_check.unwrap().status.success() {
            return Err(InputForwardingError::InitializationFailed(
                "ydotool ist erforderlich für Wayland-Input-Weiterleitung".to_string(),
            ));
        }
        
        // Key-Mapping initialisieren (für Wayland etwas anders als X11)
        let mut key_mapping = HashMap::new();
        
        // Standardtasten (für Wayland brauchen wir Linux-Keycodes statt X11-Keysyms)
        for i in 48..58 { key_mapping.insert(i, format!("KEY_{}", (i - 48))); } // 0-9
        for i in 65..91 { 
            let c = (i as u8 as char).to_lowercase().next().unwrap();
            key_mapping.insert(i, format!("KEY_{}", c.to_uppercase())); 
        } // A-Z
        
        // Funktionstasten
        for i in 1..13 { key_mapping.insert(111 + i, format!("KEY_F{}", i)); }
        
        // Spezialtasten
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
        key_mapping.insert(91, "KEY_LEFTMETA".to_string()); // Windows/Meta/Super-Taste
        key_mapping.insert(93, "KEY_MENU".to_string());
        
        // Numpad-Tasten
        for i in 0..10 { key_mapping.insert(96 + i, format!("KEY_KP{}", i)); } // Numpad 0-9
        key_mapping.insert(106, "KEY_KPASTERISK".to_string());
        key_mapping.insert(107, "KEY_KPPLUS".to_string());
        key_mapping.insert(109, "KEY_KPMINUS".to_string());
        key_mapping.insert(110, "KEY_KPDOT".to_string());
        key_mapping.insert(111, "KEY_KPSLASH".to_string());
        
        // Spezielle Befehle initialisieren
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
    
    // Berechnung der absoluten Position für Multi-Monitor
    fn calculate_absolute_position(&self, x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
        let monitors = self.monitors.lock().unwrap();
        
        if monitors.is_empty() {
            return (x, y); // Keine Monitor-Konfiguration, direkte Positionsverwendung
        }
        
        let target_monitor = match monitor_index {
            Some(idx) if idx < monitors.len() => &monitors[idx],
            _ => monitors.iter().find(|m| m.is_primary).unwrap_or(&monitors[0]),
        };
        
        // Berechne absolute Position relativ zum Zielmonitor
        let abs_x = target_monitor.x_offset + (x as f32 * target_monitor.scale_factor) as i32;
        let abs_y = target_monitor.y_offset + (y as f32 * target_monitor.scale_factor) as i32;
        
        (abs_x, abs_y)
    }
    
    // Verbesserte Tastenevent-Weiterleitung für Wayland
    fn forward_improved_key_event(&self, event: &InputEvent) -> Result<(), InputForwardingError> {
        if let (Some(key_code), Some(is_pressed)) = (event.key_code, event.is_pressed) {
            let mut active_mods = self.active_modifiers.lock().unwrap();
            
            // Linux key code aus dem Mapping holen
            let key_code_str = match self.key_mapping.get(&key_code) {
                Some(code) => code.clone(),
                None => format!("KEY_{}", key_code), // Fallback
            };
            
            let value = if is_pressed { "1" } else { "0" };
            
            // Modifikatoren verwalten
            if let Some(modifiers) = &event.modifiers {
                for modifier in modifiers {
                    if is_pressed && !active_mods.contains(modifier) {
                        active_mods.push(modifier.clone());
                    } else if !is_pressed {
                        active_mods.retain(|m| m != modifier);
                    }
                }
            }
            
            // ydotool-Kommando erstellen
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
                            format!("ydotool input fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                        ))
                    }
                }
                Err(e) => Err(InputForwardingError::SendEventFailed(
                    format!("Fehler beim Ausführen von ydotool: {}", e)
                )),
            }
        } else {
            Err(InputForwardingError::UnsupportedEvent(
                "Tastenevent fehlt keyCode oder pressed-Status".to_string()
            ))
        }
    }
    
    // Implementierung von Touch-Gesten für Wayland
    fn handle_wayland_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        // Wayland-Gestenunterstützung ist ähnlich wie X11, aber verwendet ydotool
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
                    
                    // Für Wayland verwenden wir EV_REL-Events
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
                                format!("Fehler beim Ausführen des Scroll-Befehls: {}", e)
                            ));
                        }
                        
                        let output = cmd_result.unwrap();
                        if !output.status.success() {
                            return Err(InputForwardingError::SendEventFailed(
                                format!("Scroll-Befehl fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                            ));
                        }
                    }
                    
                    return Ok(());
                }
            },
            _ => {
                // Für andere Gesten simulieren wir Tastenkombinationen ähnlich wie in X11
                // Aber implementieren sie mit ydotool-Befehlen
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
                                "ThreeFingerSwipe benötigt eine Richtung".to_string()
                            ));
                        }
                    },
                    _ => {
                        return Err(InputForwardingError::UnsupportedEvent(
                            format!("Nicht unterstützte Geste für Wayland: {:?}", gesture)
                        ));
                    }
                };
                
                // Drücke alle Tasten
                for key in &key_sequence {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("1")  // keydown
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Fehler beim Ausführen von ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keydown fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                // Lasse alle Tasten in umgekehrter Reihenfolge los
                for key in key_sequence.iter().rev() {
                    let cmd_result = Command::new("ydotool")
                        .arg("input")
                        .arg("--type").arg("EV_KEY")
                        .arg("--code").arg(key)
                        .arg("--value").arg("0")  // keyup
                        .output();
                    
                    if let Err(e) = cmd_result {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Fehler beim Ausführen von ydotool: {}", e)
                        ));
                    }
                    
                    let output = cmd_result.unwrap();
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("ydotool keyup fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent(
            "Unvollständige Gestendaten für Wayland".to_string()
        ))
    }
    
    // Implementierung spezieller Befehle für Wayland
    fn execute_special_command(&self, command: &SpecialCommand) -> Result<(), InputForwardingError> {
        // Hole die Tastenkombination für den Befehl
        let key_sequence = match self.special_commands.get(command) {
            Some(keys) => keys,
            None => {
                // Für Custom-Befehle direkten String verwenden
                if let SpecialCommand::Custom(cmd_str) = command {
                    // Direkten ydotool-Befehl ausführen
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("ydotool {}", cmd_str))
                        .output()
                        .map_err(|e| {
                            InputForwardingError::SendEventFailed(
                                format!("Fehler beim Ausführen des benutzerdefinierten Befehls: {}", e)
                            )
                        })?;
                    
                    if !output.status.success() {
                        return Err(InputForwardingError::SendEventFailed(
                            format!("Benutzerdefinierter Befehl fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                        ));
                    }
                    
                    return Ok(());
                } else {
                    return Err(InputForwardingError::UnsupportedEvent(
                        format!("Kein Mapping für Spezialbefehl: {:?}", command)
                    ));
                }
            }
        };
        
        // Drücke alle Tasten
        for key in key_sequence {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("1")  // keydown
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Fehler beim Ausführen von ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keydown fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        // Lasse alle Tasten in umgekehrter Reihenfolge los
        for key in key_sequence.iter().rev() {
            let cmd_result = Command::new("ydotool")
                .arg("input")
                .arg("--type").arg("EV_KEY")
                .arg("--code").arg(key)
                .arg("--value").arg("0")  // keyup
                .output();
            
            if let Err(e) = cmd_result {
                return Err(InputForwardingError::SendEventFailed(
                    format!("Fehler beim Ausführen von ydotool: {}", e)
                ));
            }
            
            let output = cmd_result.unwrap();
            if !output.status.success() {
                return Err(InputForwardingError::SendEventFailed(
                    format!("ydotool keyup fehlgeschlagen: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        Ok(())
    }
}
        } else {
            Err(InputForwardingError::UnsupportedEvent("Tastenevent fehlt keyCode oder pressed-Status".to_string()))
        }
    }
    
    // Implementierung der Touch-Gesten
    fn handle_x11_gesture(&self, gesture: &TouchGesture, direction: Option<&GestureDirection>, magnitude: Option<f32>) -> Result<(), InputForwardingError> {
        match gesture {
            TouchGesture::TwoFingerScroll => {
                // Two-finger scroll wird als Scrollevent behandelt
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
            },
            TouchGesture::Pinch => {
                // Pinch-Geste als Strg+Plus/Minus für Zoom simulieren
                if let Some(mag) = magnitude {
                    let zoom_in = mag > 0.0;
                    
                    // Strg-Taste drücken
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Plus/Minus-Taste drücken je nach Zoom-Richtung
                    let zoom_key = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(if zoom_in { 107 } else { 109 }), // Plus oder Minus
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Plus/Minus-Taste loslassen
                    let zoom_key_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(if zoom_in { 107 } else { 109 }),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Strg-Taste loslassen
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Events in Reihenfolge ausführen
                    self.forward_event(&ctrl_down)?;
                    self.forward_event(&zoom_key)?;
                    self.forward_event(&zoom_key_up)?;
                    self.forward_event(&ctrl_up)?;
                    
                    return Ok(());
                }
            },
            TouchGesture::ThreeFingerSwipe => {
                // Drei-Finger-Swipe für Workspace-Wechsel (Strg+Alt+Pfeiltaste)
                if let Some(dir) = direction {
                    // Strg-Taste drücken
                    let ctrl_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(17), // Ctrl
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Alt-Taste drücken
                    let alt_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(18), // Alt
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Bestimme Pfeiltaste basierend auf Richtung
                    let arrow_key = match dir {
                        GestureDirection::Left => 37,  // Links
                        GestureDirection::Right => 39, // Rechts
                        GestureDirection::Up => 38,    // Oben
                        GestureDirection::Down => 40,  // Unten
                    };
                    
                    // Pfeiltaste drücken
                    let arrow_down = InputEvent {
                        event_type: InputEventType::KeyPress,
                        key_code: Some(arrow_key),
                        is_pressed: Some(true),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Pfeiltaste loslassen
                    let arrow_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(arrow_key),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string(), "alt".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Alt-Taste loslassen
                    let alt_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(18),
                        is_pressed: Some(false),
                        modifiers: Some(vec!["ctrl".to_string()]),
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Strg-Taste loslassen
                    let ctrl_up = InputEvent {
                        event_type: InputEventType::KeyRelease,
                        key_code: Some(17),
                        is_pressed: Some(false),
                        modifiers: None,
                        x: None, y: None, button: None, delta_x: None, delta_y: None,
                        monitor_index: None, gesture: None, gesture_direction: None,
                        gesture_magnitude: None, special_command: None,
                    };
                    
                    // Events in Reihenfolge ausführen
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
                    format!("Nicht unterstützte Geste: {:?}", gesture)
                ));
            }
        }
        
        Err(InputForwardingError::UnsupportedEvent("Unvollständige Gestendaten".to_string()))
