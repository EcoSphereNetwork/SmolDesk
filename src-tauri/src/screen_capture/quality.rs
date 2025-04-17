// screen_capture/quality.rs - Adaptive quality controller for optimizing video streams

use std::time::{Duration, Instant};
use crate::screen_capture::config::{RateControlMode, ScreenCaptureConfig};

/// Adaptive quality controller for dynamically adjusting encoding parameters
pub struct AdaptiveQualityController {
    /// Current quality setting (0-100)
    pub current_quality:
