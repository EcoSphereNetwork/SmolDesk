// src-tauri/src/clipboard/error.rs - Fehlerbehandlung für Zwischenablage-Synchronisation

use std::error::Error;
use std::fmt;

/// Fehlertypen für Zwischenablage-Operationen
#[derive(Debug)]
pub enum ClipboardError {
    /// Plattform wird nicht unterstützt
    UnsupportedPlatform(String),
    
    /// Zwischenablage ist nicht verfügbar
    ClipboardUnavailable(String),
    
    /// Zwischenablage ist leer
    EmptyClipboard,
    
    /// Ungültiges Datenformat
    InvalidFormat(String),
    
    /// Datei/Eintrag nicht gefunden
    EntryNotFound(String),
    
    /// Serialisierungsfehler
    SerializationError(String),
    
    /// Dekodierungsfehler
    DecodingError(String),
    
    /// Netzwerkfehler bei Synchronisation
    NetworkError(String),
    
    /// Berechtigungsfehler
    PermissionDenied(String),
    
    /// Nicht unterstützte Operation
    UnsupportedOperation(String),
    
    /// I/O-Fehler
    IoError(String),
    
    /// Timeout bei Operation
    Timeout(String),
    
    /// Inhalt zu groß
    ContentTooLarge(usize, usize), // (actual_size, max_size)
    
    /// Filter blockiert Inhalt
    ContentBlocked(String),
    
    /// Konfigurationsfehler
    ConfigError(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::UnsupportedPlatform(msg) => write!(f, "Unsupported platform: {}", msg),
            ClipboardError::ClipboardUnavailable(msg) => write!(f, "Clipboard unavailable: {}", msg),
            ClipboardError::EmptyClipboard => write!(f, "Clipboard is empty"),
            ClipboardError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ClipboardError::EntryNotFound(id) => write!(f, "Entry not found: {}", id),
            ClipboardError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ClipboardError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            ClipboardError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ClipboardError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ClipboardError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            ClipboardError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ClipboardError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ClipboardError::ContentTooLarge(actual, max) => {
                write!(f, "Content too large: {} bytes (max: {} bytes)", actual, max)
            },
            ClipboardError::ContentBlocked(reason) => write!(f, "Content blocked: {}", reason),
            ClipboardError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for ClipboardError {}

// Konvertierungen von Standard-Fehlern
impl From<std::io::Error> for ClipboardError {
    fn from(error: std::io::Error) -> Self {
        ClipboardError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for ClipboardError {
    fn from(error: serde_json::Error) -> Self {
        ClipboardError::SerializationError(error.to_string())
    }
}

impl From<base64::DecodeError> for ClipboardError {
    fn from(error: base64::DecodeError) -> Self {
        ClipboardError::DecodingError(error.to_string())
    }
}

/// Hilfsfunktionen für Fehlerbehandlung
pub fn clipboard_unavailable_error(context: &str) -> ClipboardError {
    ClipboardError::ClipboardUnavailable(context.to_string())
}

pub fn invalid_format_error(format: &str, context: &str) -> ClipboardError {
    ClipboardError::InvalidFormat(format!("{}: {}", format, context))
}

pub fn permission_denied_error(operation: &str) -> ClipboardError {
    ClipboardError::PermissionDenied(format!("Permission denied for operation: {}", operation))
}

pub fn content_too_large_error(actual_size: usize, max_size: usize) -> ClipboardError {
    ClipboardError::ContentTooLarge(actual_size, max_size)
}

pub fn unsupported_operation_error(operation: &str, platform: &str) -> ClipboardError {
    ClipboardError::UnsupportedOperation(format!("{} not supported on {}", operation, platform))
}

/// Trait für Konvertierung von Fehlern mit Kontext
pub trait ClipboardErrorExt<T> {
    fn with_clipboard_context<C>(self, context: C) -> Result<T, ClipboardError>
    where
        C: FnOnce() -> String;
}

impl<T, E: fmt::Display> ClipboardErrorExt<T> for Result<T, E> {
    fn with_clipboard_context<C>(self, context: C) -> Result<T, ClipboardError>
    where
        C: FnOnce() -> String
    {
        self.map_err(|e| {
            ClipboardError::IoError(format!("{}: {}", context(), e))
        })
    }
}
