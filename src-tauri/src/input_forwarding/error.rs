// error.rs - Error handling for input forwarding

use std::error::Error;
use std::fmt;

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

impl Error for InputForwardingError {}

// Helper conversion traits for working with Result
pub trait InputForwardingErrorExt<T> {
    fn with_context<C>(self, context: C) -> Result<T, InputForwardingError> 
    where
        C: FnOnce() -> String;
}

impl<T, E: fmt::Display> InputForwardingErrorExt<T> for Result<T, E> {
    fn with_context<C>(self, context: C) -> Result<T, InputForwardingError> 
    where
        C: FnOnce() -> String
    {
        self.map_err(|e| {
            InputForwardingError::SendEventFailed(format!("{}: {}", context(), e))
        })
    }
}
