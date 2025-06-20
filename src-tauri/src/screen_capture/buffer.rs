// screen_capture/buffer.rs - Stream buffer implementation for continuous streams

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::screen_capture::types::FrameData;
use crate::screen_capture::error::ScreenCaptureError;

/// Stream buffer for managing continuous video streams
pub struct StreamBuffer {
    /// Queue of video frame chunks
    chunks: VecDeque<FrameData>,
    
    /// Maximum number of chunks to store
    max_size: usize,
    
    /// Total bytes currently stored in the buffer
    total_bytes: usize,
    
    /// Maximum buffer size in bytes
    max_bytes: usize,
    
    /// Whether to drop old frames when buffer is full
    drop_mode: DropMode,
    
    /// Time at which the newest frame was added
    latest_timestamp: Option<Instant>,
    
    /// Estimated frame duration based on configured FPS
    frame_duration: Duration,
    
    /// Stats about the buffer
    stats: BufferStats,
}

/// Mode for handling buffer overflow
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropMode {
    /// Drop oldest frames when buffer is full
    DropOldest,
    
    /// Drop newest frames when buffer is full (better for maintaining temporal consistency)
    DropNewest,
    
    /// Drop alternating frames to maintain some temporal consistency
    DropAlternating,
    
    /// Drop non-keyframes first, then oldest keyframes
    DropNonKeyframes,
}

/// Statistics about the buffer
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Number of frames added
    pub frames_added: u64,
    
    /// Number of frames dropped due to buffer overflow
    pub frames_dropped: u64,
    
    /// Number of frames read from the buffer
    pub frames_read: u64,
    
    /// Estimated buffer fill ratio (0.0 - 1.0)
    pub fill_ratio: f32,
    
    /// Current number of frames in buffer
    pub frame_count: usize,
    
    /// Estimated buffer latency in milliseconds
    pub latency_ms: f64,
}

impl StreamBuffer {
    /// Create a new stream buffer
    pub fn new(max_frames: usize, max_bytes_mb: usize, fps: u32, drop_mode: DropMode) -> Self {
        let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
        let max_bytes = max_bytes_mb * 1024 * 1024; // Convert MB to bytes
        
        StreamBuffer {
            chunks: VecDeque::with_capacity(max_frames),
            max_size: max_frames,
            total_bytes: 0,
            max_bytes,
            drop_mode,
            latest_timestamp: None,
            frame_duration,
            stats: BufferStats {
                frames_added: 0,
                frames_dropped: 0,
                frames_read: 0,
                fill_ratio: 0.0,
                frame_count: 0,
                latency_ms: 0.0,
            },
        }
    }
    
    /// Push a new frame to the buffer
    pub fn push_frame(&mut self, frame: FrameData) -> Result<(), ScreenCaptureError> {
        let frame_size = frame.data.len();
        
        // Update statistics
        self.stats.frames_added += 1;
        self.latest_timestamp = Some(Instant::now());
        
        // Check if buffer is full (by frames or bytes)
        let is_buffer_full = self.chunks.len() >= self.max_size || 
                             (self.total_bytes + frame_size) > self.max_bytes;
        
        if is_buffer_full {
            match self.drop_mode {
                DropMode::DropOldest => {
                    // Drop oldest frame to make room
                    if let Some(old_frame) = self.chunks.pop_front() {
                        self.total_bytes -= old_frame.data.len();
                        self.stats.frames_dropped += 1;
                    }
                },
                DropMode::DropNewest => {
                    // Don't add the new frame
                    self.stats.frames_dropped += 1;
                    return Ok(());
                },
                DropMode::DropAlternating => {
                    // Drop every other frame, starting from the oldest
                    let mut dropped = false;
                    for i in 0..self.chunks.len() {
                        if i % 2 == 0 {
                            if let Some(removed_frame) = self.chunks.remove(i) {
                                self.total_bytes -= removed_frame.data.len();
                                self.stats.frames_dropped += 1;
                                dropped = true;
                                break;
                            }
                        }
                    }
                    
                    // If we couldn't drop any alternating frames, drop the oldest
                    if !dropped {
                        if let Some(old_frame) = self.chunks.pop_front() {
                            self.total_bytes -= old_frame.data.len();
                            self.stats.frames_dropped += 1;
                        }
                    }
                },
                DropMode::DropNonKeyframes => {
                    // First try to drop non-keyframes
                    let mut dropped = false;
                    
                    // Find the oldest non-keyframe
                    for i in 0..self.chunks.len() {
                        if !self.chunks[i].keyframe {
                            if let Some(removed_frame) = self.chunks.remove(i) {
                                self.total_bytes -= removed_frame.data.len();
                                self.stats.frames_dropped += 1;
                                dropped = true;
                                break;
                            }
                        }
                    }
                    
                    // If we couldn't drop any non-keyframes, drop the oldest frame
                    if !dropped {
                        if let Some(old_frame) = self.chunks.pop_front() {
                            self.total_bytes -= old_frame.data.len();
                            self.stats.frames_dropped += 1;
                        }
                    }
                },
            }
        }
        
        // Add the new frame
        self.chunks.push_back(frame);
        self.total_bytes += frame_size;
        
        // Update buffer statistics
        self.update_stats();
        
        Ok(())
    }
    
    /// Get the next frame from the buffer
    pub fn get_next_frame(&mut self) -> Option<FrameData> {
        if self.chunks.is_empty() {
            return None;
        }
        
        let frame = self.chunks.pop_front()?;
        self.total_bytes -= frame.data.len();
        
        self.stats.frames_read += 1;
        self.update_stats();
        
        Some(frame)
    }
    
    /// Peek at the next frame without removing it
    pub fn peek_next_frame(&self) -> Option<&FrameData> {
        self.chunks.front()
    }
    
    /// Get the number of frames in the buffer
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.total_bytes = 0;
        self.latest_timestamp = None;
        self.update_stats();
    }
    
    /// Get the buffer statistics
    pub fn get_stats(&self) -> &BufferStats {
        &self.stats
    }
    
    /// Get buffer fill ratio (0.0 - 1.0)
    pub fn get_fill_ratio(&self) -> f32 {
        if self.max_size == 0 {
            return 0.0;
        }
        self.chunks.len() as f32 / self.max_size as f32
    }
    
    /// Get estimated latency in milliseconds
    pub fn get_latency_ms(&self) -> f64 {
        // Latency is roughly the time it would take to play all frames in the buffer
        (self.chunks.len() as f64) * self.frame_duration.as_secs_f64() * 1000.0
    }
    
    /// Get the current buffer size in bytes
    pub fn get_bytes(&self) -> usize {
        self.total_bytes
    }
    
    /// Get the maximum buffer size in bytes
    pub fn get_max_bytes(&self) -> usize {
        self.max_bytes
    }
    
    /// Update buffer statistics
    fn update_stats(&mut self) {
        self.stats.fill_ratio = self.get_fill_ratio();
        self.stats.frame_count = self.chunks.len();
        self.stats.latency_ms = self.get_latency_ms();
    }
    
    /// Adjust the buffer size
    pub fn resize(&mut self, max_frames: usize, max_bytes_mb: usize) {
        self.max_size = max_frames;
        self.max_bytes = max_bytes_mb * 1024 * 1024;
        
        // Trim the buffer if it's now over the new max size
        while self.chunks.len() > self.max_size || self.total_bytes > self.max_bytes {
            if let Some(old_frame) = self.chunks.pop_front() {
                self.total_bytes -= old_frame.data.len();
                self.stats.frames_dropped += 1;
            }
        }
        
        self.update_stats();
    }
    
    /// Set a new frame rate for latency calculation
    pub fn set_fps(&mut self, fps: u32) {
        self.frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
        self.update_stats();
    }
    
    /// Get all frames, draining the buffer
    pub fn drain(&mut self) -> Vec<FrameData> {
        let mut frames = Vec::with_capacity(self.chunks.len());
        
        while let Some(frame) = self.get_next_frame() {
            frames.push(frame);
        }
        
        frames
    }
    
    /// Change the drop mode
    pub fn set_drop_mode(&mut self, mode: DropMode) {
        self.drop_mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_basic_operations() {
        let mut buffer = StreamBuffer::new(5, 10, 30, DropMode::DropOldest);
        
        // Create test frames
        let frame1 = FrameData {
            data: vec![0; 100],
            timestamp: 1,
            keyframe: true,
            width: 640,
            height: 480,
            format: "h264".to_string(),
        };
        
        let frame2 = FrameData {
            data: vec![0; 200],
            timestamp: 2,
            keyframe: false,
            width: 640,
            height: 480,
            format: "h264".to_string(),
        };
        
        // Test push and get
        buffer.push_frame(frame1.clone()).unwrap();
        buffer.push_frame(frame2.clone()).unwrap();
        
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.get_bytes(), 300);
        
        let retrieved = buffer.get_next_frame().unwrap();
        assert_eq!(retrieved.timestamp, frame1.timestamp);
        assert_eq!(buffer.len(), 1);
        
        // Test clear
        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.get_bytes(), 0);
        assert!(buffer.is_empty());
    }
    
    #[test]
    fn test_buffer_overflow() {
        let mut buffer = StreamBuffer::new(3, 10, 30, DropMode::DropOldest);
        
        // Fill the buffer to capacity
        for i in 1..=5 {
            let frame = FrameData {
                data: vec![0; 100],
                timestamp: i,
                keyframe: i % 2 == 0, // Every other frame is a keyframe
                width: 640,
                height: 480,
                format: "h264".to_string(),
            };
            
            buffer.push_frame(frame).unwrap();
        }
        
        // Buffer should only contain the 3 newest frames
        assert_eq!(buffer.len(), 3);
        
        // The oldest frame should be frame 3
        let next_frame = buffer.get_next_frame().unwrap();
        assert_eq!(next_frame.timestamp, 3);
    }
}
