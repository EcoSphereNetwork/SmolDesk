// screen_capture/error.rs - Error types for screen capture operations

use std::error::Error;
use std::fmt;

/// Error types for screen capture operations
#[derive(Debug)]
pub enum ScreenCaptureError {
    /// Initialization of the capture system failed
    InitializationFailed(String),
    
    /// Error during capture process
    CaptureError(String),
    
    /// Error during video encoding
    EncodingError(String),
    
    /// Error related to display server detection or interaction
    DisplayServerError(String),
    
    /// Error when trying to capture from an invalid monitor
    InvalidMonitor(String),
    
    /// Error related to stream buffer operations
    StreamBufferError(String),
    
    /// Error with hardware acceleration
    HardwareAccelerationError(String),
    
    /// Error with FFmpeg process
    FFmpegError(String),
    
    /// Error with PipeWire process (Wayland)
    PipeWireError(String),
}

impl fmt::Display for ScreenCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScreenCaptureError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            ScreenCaptureError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            ScreenCaptureError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            ScreenCaptureError::DisplayServerError(msg) => write!(f, "Display server error: {}", msg),
            ScreenCaptureError::InvalidMonitor(msg) => write!(f, "Invalid monitor: {}", msg),
            ScreenCaptureError::StreamBufferError(msg) => write!(f, "Stream buffer error: {}", msg),
            ScreenCaptureError::HardwareAccelerationError(msg) => write!(f, "Hardware acceleration error: {}", msg),
            ScreenCaptureError::FFmpegError(msg) => write!(f, "FFmpeg error: {}", msg),
            ScreenCaptureError::PipeWireError(msg) => write!(f, "PipeWire error: {}", msg),
        }
    }
}

impl Error for ScreenCaptureError {}

// Helper conversion traits for working with Result
pub trait ScreenCaptureErrorExt<T> {
    fn with_context<C>(self, context: C) -> Result<T, ScreenCaptureError> 
    where
        C: FnOnce() -> String;
}

impl<T, E: fmt::Display> ScreenCaptureErrorExt<T> for Result<T, E> {
    fn with_context<C>(self, context: C) -> Result<T, ScreenCaptureError> 
    where
        C: FnOnce() -> String
    {
        self.map_err(|e| {
            ScreenCaptureError::CaptureError(format!("{}: {}", context(), e))
        })
    }
}

// Specialized conversion functions
pub fn to_init_error<E: fmt::Display>(e: E, context: &str) -> ScreenCaptureError {
    ScreenCaptureError::InitializationFailed(format!("{}: {}", context, e))
}

pub fn to_capture_error<E: fmt::Display>(e: E, context: &str) -> ScreenCaptureError {
    ScreenCaptureError::CaptureError(format!("{}: {}", context, e))
}

pub fn to_encoding_error<E: fmt::Display>(e: E, context: &str) -> ScreenCaptureError {
    ScreenCaptureError::EncodingError(format!("{}: {}", context, e))
}

pub fn to_ffmpeg_error<E: fmt::Display>(e: E, context: &str) -> ScreenCaptureError {
    ScreenCaptureError::FFmpegError(format!("{}: {}", context, e))
}
