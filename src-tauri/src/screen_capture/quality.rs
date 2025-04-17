// screen_capture/quality.rs - Adaptive quality controller for optimizing video streams

use std::time::{Duration, Instant};
use crate::screen_capture::config::{RateControlMode, ScreenCaptureConfig};

/// Adaptive quality controller for dynamically adjusting encoding parameters
pub struct AdaptiveQualityController {
    /// Current quality setting (0-100)
    current_quality: u32,
    
    /// CPU usage percentage (0-100)
    cpu_usage: f32,
    
    /// Network bandwidth in kbps
    network_bandwidth: u32,
    
    /// Frame drop rate (0.0-1.0)
    frame_drop_rate: f32,
    
    /// Last time quality was adjusted
    last_adjustment: Instant,
    
    /// Minimum time between adjustments
    adjustment_interval: Duration,
    
    /// History of recent quality values (for smoothing)
    quality_history: Vec<u32>,
    
    /// History of recent bandwidth measurements
    bandwidth_history: Vec<u32>,
    
    /// Configuration for quality adjustments
    config: QualityAdapterConfig,
    
    /// Target latency in milliseconds
    target_latency_ms: u32,
    
    /// Actual measured latency in milliseconds
    measured_latency_ms: u32,
}

/// Configuration for the quality adapter
pub struct QualityAdapterConfig {
    /// Maximum quality setting
    pub max_quality: u32,
    
    /// Minimum quality setting
    pub min_quality: u32,
    
    /// How aggressively to adjust quality (higher = more aggressive)
    pub adjustment_factor: f32,
    
    /// Minimum time between adjustments
    pub min_adjustment_interval_ms: u64,
    
    /// Threshold for CPU usage to trigger quality reduction
    pub cpu_threshold_high: f32,
    
    /// Threshold for CPU usage to allow quality increase
    pub cpu_threshold_low: f32,
    
    /// Threshold for frame drops to trigger quality reduction
    pub frame_drop_threshold: f32,
    
    /// Size of history buffer for smoothing
    pub history_size: usize,
    
    /// Whether to prioritize latency over quality
    pub prioritize_latency: bool,
}

impl Default for QualityAdapterConfig {
    fn default() -> Self {
        QualityAdapterConfig {
            max_quality: 100,
            min_quality: 10,
            adjustment_factor: 1.0,
            min_adjustment_interval_ms: 5000,
            cpu_threshold_high: 85.0,
            cpu_threshold_low: 50.0,
            frame_drop_threshold: 0.05,
            history_size: 5,
            prioritize_latency: true,
        }
    }
}

impl AdaptiveQualityController {
    /// Create a new adaptive quality controller
    pub fn new(initial_quality: u32, config: Option<QualityAdapterConfig>) -> Self {
        let config = config.unwrap_or_default();
        let quality = initial_quality.min(config.max_quality).max(config.min_quality);
        
        AdaptiveQualityController {
            current_quality: quality,
            cpu_usage: 0.0,
            network_bandwidth: 5000, // Default assumption: 5 Mbps
            frame_drop_rate: 0.0,
            last_adjustment: Instant::now(),
            adjustment_interval: Duration::from_millis(config.min_adjustment_interval_ms),
            quality_history: vec![quality; config.history_size],
            bandwidth_history: vec![5000; config.history_size],
            config,
            target_latency_ms: 200, // Default target latency
            measured_latency_ms: 0,
        }
    }
    
    /// Update metrics used for quality adaptation
    pub fn update_metrics(&mut self, cpu_usage: f32, network_bandwidth: u32, frame_drop_rate: f32, latency_ms: u32) {
        self.cpu_usage = cpu_usage;
        self.network_bandwidth = network_bandwidth;
        self.frame_drop_rate = frame_drop_rate;
        self.measured_latency_ms = latency_ms;
        
        // Update history
        self.bandwidth_history.push(network_bandwidth);
        if self.bandwidth_history.len() > self.config.history_size {
            self.bandwidth_history.remove(0);
        }
    }
    
    /// Adjust quality based on current metrics
    pub fn adjust_quality(&mut self) -> u32 {
        let now = Instant::now();
        if now.duration_since(self.last_adjustment) < self.adjustment_interval {
            return self.current_quality;
        }
        
        // Determine adjustment direction and magnitude
        let mut adjustment = 0;
        
        // Check CPU usage
        if self.cpu_usage > self.config.cpu_threshold_high {
            adjustment -= 5;
        }
        
        // Check frame drop rate
        if self.frame_drop_rate > self.config.frame_drop_threshold {
            adjustment -= 10;
        }
        
        // Check latency if we're prioritizing it
        if self.config.prioritize_latency && self.measured_latency_ms > self.target_latency_ms {
            let latency_factor = (self.measured_latency_ms as f32 / self.target_latency_ms as f32) - 1.0;
            adjustment -= (latency_factor * 10.0) as i32;
        }
        
        // If we have headroom, consider increasing quality
        if self.cpu_usage < self.config.cpu_threshold_low && 
           self.frame_drop_rate < (self.config.frame_drop_threshold / 2.0) &&
           self.measured_latency_ms < self.target_latency_ms {
            adjustment += 2;
        }
        
        // Scale adjustment by factor
        adjustment = (adjustment as f32 * self.config.adjustment_factor) as i32;
        
        // Apply adjustment
        let new_quality = (self.current_quality as i32 + adjustment)
            .max(self.config.min_quality as i32)
            .min(self.config.max_quality as i32) as u32;
        
        // Update quality
        self.current_quality = new_quality;
        self.quality_history.push(new_quality);
        if self.quality_history.len() > self.config.history_size {
            self.quality_history.remove(0);
        }
        
        self.last_adjustment = now;
        new_quality
    }
    
    /// Get the current quality setting
    pub fn get_quality(&self) -> u32 {
        self.current_quality
    }
    
    /// Get a smoothed quality value based on recent history
    pub fn get_smoothed_quality(&self) -> u32 {
        if self.quality_history.is_empty() {
            return self.current_quality;
        }
        
        let sum: u32 = self.quality_history.iter().sum();
        sum / self.quality_history.len() as u32
    }
    
    /// Calculate bitrate for a given resolution
    pub fn get_bitrate_for_resolution(&self, width: u32, height: u32) -> u32 {
        // Basic heuristic for bitrate based on resolution, quality, and available bandwidth
        let pixel_count = width * height;
        
        // Base bitrate depends on resolution
        let base_bitrate = match pixel_count {
            p if p > 2073600 => 8000, // 1080p+
            p if p > 921600 => 5000,  // 720p+
            p if p > 480000 => 2500,  // 480p+
            _ => 1000,                // Lower resolutions
        };
        
        // Quality adjustment (10% - 100% of base bitrate)
        let quality_factor = self.current_quality as f32 / 100.0;
        
        // Network bandwidth constraint (don't go above 80% of available bandwidth)
        let avg_bandwidth = self.get_average_bandwidth();
        let network_cap = (avg_bandwidth as f32 * 0.8) as u32;
        
        let bitrate = (base_bitrate as f32 * quality_factor) as u32;
        bitrate.min(network_cap)
    }
    
    /// Get average bandwidth from history
    fn get_average_bandwidth(&self) -> u32 {
        if self.bandwidth_history.is_empty() {
            return self.network_bandwidth;
        }
        
        let sum: u32 = self.bandwidth_history.iter().sum();
        sum / self.bandwidth_history.len() as u32
    }
    
    /// Set target latency
    pub fn set_target_latency(&mut self, latency_ms: u32) {
        self.target_latency_ms = latency_ms;
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: QualityAdapterConfig) {
        self.config = config;
        self.adjustment_interval = Duration::from_millis(config.min_adjustment_interval_ms);
        
        // Ensure current quality is within new bounds
        self.current_quality = self.current_quality
            .max(config.min_quality)
            .min(config.max_quality);
    }
    
    /// Generate FFmpeg parameters based on current quality settings
    pub fn generate_ffmpeg_params(&self, config: &ScreenCaptureConfig) -> Vec<String> {
        let mut params = Vec::new();
        
        // Quality-specific parameters
        match config.codec {
            crate::screen_capture::types::VideoCodec::H264 => {
                // CRF mode (lower = better quality)
                let crf = 51 - (self.current_quality / 2);
                params.push("-crf".to_string());
                params.push(crf.to_string());
                
                // Preset depends on quality and latency requirements
                let preset = if self.measured_latency_ms > self.target_latency_ms * 2 {
                    "ultrafast"
                } else if self.current_quality < 30 {
                    "superfast"
                } else if self.current_quality < 60 {
                    "veryfast"
                } else {
                    "medium"
                };
                
                params.push("-preset".to_string());
                params.push(preset.to_string());
                
                // Use zerolatency tuning for remote desktop
                params.push("-tune".to_string());
                params.push("zerolatency".to_string());
            },
            crate::screen_capture::types::VideoCodec::VP8 => {
                // VP8 quality (lower = better quality)
                let qp = 63 - (self.current_quality * 63 / 100);
                params.push("-qmin".to_string());
                params.push(qp.to_string());
                params.push("-qmax".to_string());
                params.push((qp + 10).min(63).to_string());
            },
            crate::screen_capture::types::VideoCodec::VP9 => {
                // VP9 quality (lower = better quality)
                let crf = 63 - (self.current_quality * 63 / 100);
                params.push("-crf".to_string());
                params.push(crf.to_string());
                
                // Speed depends on quality
                let speed = if self.current_quality < 30 {
                    8
                } else if self.current_quality < 60 {
                    6
                } else {
                    4
                };
                
                params.push("-speed".to_string());
                params.push(speed.to_string());
            },
            crate::screen_capture::types::VideoCodec::AV1 => {
                // AV1 quality (lower = better quality)
                let crf = 63 - (self.current_quality * 63 / 100);
                params.push("-crf".to_string());
                params.push(crf.to_string());
                
                // CPU usage depends on quality
                let cpu_used = if self.current_quality < 30 {
                    8
                } else if self.current_quality < 60 {
                    6
                } else {
                    4
                };
                
                params.push("-cpu-used".to_string());
                params.push(cpu_used.to_string());
            }
        }
        
        // If we have a specific bitrate preference, use it
        if let Some(target_bitrate) = config.bitrate {
            params.push("-b:v".to_string());
            params.push(format!("{}k", target_bitrate));
        }
        
        // Keyframe interval
        params.push("-g".to_string());
        params.push(config.keyframe_interval.to_string());
        
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_adjustment() {
        let mut controller = AdaptiveQualityController::new(80, None);
        
        // Test high CPU should lower quality
        controller.update_metrics(90.0, 5000, 0.01, 150);
        let new_quality = controller.adjust_quality();
        assert!(new_quality < 80);
        
        // Reset and test high frame drop rate
        let mut controller = AdaptiveQualityController::new(80, None);
        controller.update_metrics(50.0, 5000, 0.1, 150);
        let new_quality = controller.adjust_quality();
        assert!(new_quality < 80);
        
        // Reset and test low CPU and no drops should increase quality
        let mut controller = AdaptiveQualityController::new(50, None);
        controller.update_metrics(30.0, 10000, 0.01, 150);
        let new_quality = controller.adjust_quality();
        assert!(new_quality > 50);
    }
    
    #[test]
    fn test_bitrate_calculation() {
        let mut controller = AdaptiveQualityController::new(50, None);
        
        // Test different resolutions
        let bitrate_720p = controller.get_bitrate_for_resolution(1280, 720);
        let bitrate_1080p = controller.get_bitrate_for_resolution(1920, 1080);
        
        // Higher resolution should have higher bitrate
        assert!(bitrate_1080p > bitrate_720p);
        
        // Test quality impact
        controller.current_quality = 100;
        let bitrate_high_quality = controller.get_bitrate_for_resolution(1280, 720);
        
        controller.current_quality = 20;
        let bitrate_low_quality = controller.get_bitrate_for_resolution(1280, 720);
        
        // Higher quality should have higher bitrate
        assert!(bitrate_high_quality > bitrate_low_quality);
    }
}
