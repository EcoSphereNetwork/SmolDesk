// src/utils/screenCapture.ts

import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { WebRTCConnection } from './webrtc';

interface StreamInfo {
  fps: number;
  latency: number;
  resolution: string;
}

/**
 * Handles the integration between Tauri's screen capture and WebRTC
 */
export class ScreenCaptureManager {
  private webrtcConnection: WebRTCConnection;
  private captureActive: boolean = false;
  private streamTrack: MediaStreamTrack | null = null;
  private mediaStream: MediaStream | null = null;
  private encoder: VideoEncoder | null = null;
  private decoder: VideoDecoder | null = null;
  private frameListeners: Set<(frame: VideoFrame) => void> = new Set();
  private statsInterval: number | null = null;
  
  /**
   * Create a new ScreenCaptureManager
   * @param webrtcConnection The WebRTC connection to use
   */
  constructor(webrtcConnection: WebRTCConnection) {
    this.webrtcConnection = webrtcConnection;
  }
  
  /**
   * Start capturing the screen
   * @param monitorIndex Index of the monitor to capture
   * @param config Capture configuration
   */
  async startCapture(monitorIndex: number, config: any): Promise<boolean> {
    try {
      // Start the Tauri backend capture process
      await invoke('start_capture', {
        monitorIndex,
        config,
      });
      
      this.captureActive = true;
      
      // Set up stream processing
      await this.setupStreamProcessing();
      
      // Start stats monitoring
      this.startStatsMonitoring();
      
      return true;
    } catch (error) {
      console.error('Failed to start capture:', error);
      return false;
    }
  }
  
  /**
   * Stop capturing the screen
   */
  async stopCapture(): Promise<boolean> {
    try {
      await invoke('stop_capture');
      
      this.captureActive = false;
      
      // Clean up resources
      this.cleanupResources();
      
      return true;
    } catch (error) {
      console.error('Failed to stop capture:', error);
      return false;
    }
  }
  
  /**
   * Set up stream processing for efficient WebRTC integration
   */
  private async setupStreamProcessing(): Promise<void> {
    if (!('VideoEncoder' in window) || !('VideoDecoder' in window)) {
      console.warn('WebCodecs not supported, falling back to traditional processing.');
      this.setupTraditionalProcessing();
      return;
    }
    
    try {
      // Set up video decoder for incoming frames
      this.setupVideoDecoder();
      
      // Set up video encoder for outgoing frames
      this.setupVideoEncoder();
      
      // Create mediastream with a synthetic video track
      this.createSyntheticMediaStream();
      
      // Listen for frame data from Tauri backend
      this.setupFrameListener();
    } catch (error) {
      console.error('Error setting up WebCodecs processing:', error);
      this.setupTraditionalProcessing();
    }
  }
  
  /**
   * Create a synthetic media stream using a canvas
   */
  private createSyntheticMediaStream(): void {
    // Create a canvas element (not attached to DOM)
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    
    if (!ctx) {
      throw new Error('Failed to create canvas context');
    }
    
    // Start with a default size, will be updated with actual frame sizes
    canvas.width = 1920;
    canvas.height = 1080;
    
    // Create a stream from the canvas
    const stream = canvas.captureStream(30); // 30fps target
    
    // Store the stream and track for later use
    this.mediaStream = stream;
    this.streamTrack = stream.getVideoTracks()[0];
    
    // Add the stream to the WebRTC connection
    this.addStreamToWebRTC();
    
    // Add a frame renderer to update the canvas
    this.frameListeners.add((frame) => {
      // Resize canvas if needed
      if (canvas.width !== frame.codedWidth || canvas.height !== frame.codedHeight) {
        canvas.width = frame.codedWidth;
        canvas.height = frame.codedHeight;
      }
      
      // Draw the frame on the canvas
      ctx.drawImage(frame, 0, 0);
      frame.close(); // Release the frame
    });
  }
  
  /**
   * Set up video decoder for processing incoming frames
   */
  private setupVideoDecoder(): void {
    this.decoder = new VideoDecoder({
      output: (frame) => {
        // Notify all frame listeners
        this.frameListeners.forEach(listener => {
          try {
            // Clone the frame for each listener
            const clone = new VideoFrame(frame, { duration: frame.duration });
            listener(clone);
          } catch (e) {
            console.error('Error in frame listener:', e);
          }
        });
        
        // Release the original frame
        frame.close();
      },
      error: (error) => {
        console.error('Video decoder error:', error);
      }
    });
    
    // Configure the decoder with default parameters (will be updated with actual stream info)
    this.decoder.configure({
      codec: 'vp8', // Default codec, will be updated based on actual stream
    });
  }
  
  /**
   * Set up video encoder for processing outgoing frames
   */
  private setupVideoEncoder(): void {
    this.encoder = new VideoEncoder({
      output: (chunk) => {
        // Process encoded chunks for WebRTC
        const data = new Uint8Array(chunk.byteLength);
        chunk.copyTo(data);
        
        // Send encoded data over WebRTC data channel
        this.sendEncodedVideoChunk(data, chunk.type === 'key');
      },
      error: (error) => {
        console.error('Video encoder error:', error);
      }
    });
    
    // Configure the encoder with default parameters
    this.encoder.configure({
      codec: 'vp8',
      width: 1920,
      height: 1080,
      bitrate: 2_000_000, // 2 Mbps
      framerate: 30,
    });
  }
  
  /**
   * Set up a listener for frame data from Tauri backend
   */
  private setupFrameListener(): void {
    listen('frame_data', (event) => {
      if (!this.captureActive || !this.decoder) return;
      
      try {
        const data = event.payload as string;
        
        // Decode base64 data
        const binaryData = this.base64ToArrayBuffer(data);
        
        // Decode the frame using WebCodecs
        this.decoder.decode(new EncodedVideoChunk({
          type: 'key', // Assume key frame (could be optimized with metadata)
          timestamp: performance.now() * 1000, // Convert to microseconds
          data: binaryData,
        }));
      } catch (error) {
        console.error('Error processing frame data:', error);
      }
    });
    
    // Also listen for frame format changes
    listen('frame_format', (event) => {
      const format = event.payload as {
        codec: string;
        width: number;
        height: number;
      };
      
      // Reconfigure the decoder with the actual stream parameters
      if (this.decoder) {
        this.decoder.configure({
          codec: format.codec.toLowerCase(),
        });
      }
      
      // Update encoder parameters as well
      if (this.encoder) {
        this.encoder.configure({
          codec: format.codec.toLowerCase(),
          width: format.width,
          height: format.height,
          bitrate: 2_000_000, // 2 Mbps (could be made adaptive)
          framerate: 30,
        });
      }
    });
  }
  
  /**
   * Set up traditional processing for browsers without WebCodecs
   */
  private setupTraditionalProcessing(): void {
    // Create an HTML video element (not attached to DOM)
    const videoElement = document.createElement('video');
    videoElement.autoplay = true;
    videoElement.muted = true;
    
    // Create a canvas for processing
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    
    if (!ctx) {
      throw new Error('Failed to create canvas context');
    }
    
    // Default canvas size (will be updated)
    canvas.width = 1920;
    canvas.height = 1080;
    
    // Create a media stream from the canvas
    const stream = canvas.captureStream(30);
    this.mediaStream = stream;
    this.streamTrack = stream.getVideoTracks()[0];
    
    // Add the stream to WebRTC
    this.addStreamToWebRTC();
    
    // Listen for frame data from Tauri
    listen('frame_data', (event) => {
      if (!this.captureActive) return;
      
      try {
        const data = event.payload as string;
        
        // Create a data URL from the base64 string
        const dataUrl = `data:image/png;base64,${data}`;
        
        // Create an image element to load the data
        const img = new Image();
        img.onload = () => {
          // Resize canvas if needed
          if (canvas.width !== img.width || canvas.height !== img.height) {
            canvas.width = img.width;
            canvas.height = img.height;
          }
          
          // Draw the image on the canvas
          ctx.drawImage(img, 0, 0);
        };
        
        // Load the image
        img.src = dataUrl;
      } catch (error) {
        console.error('Error processing frame data:', error);
      }
    });
  }
  
  /**
   * Add the media stream to the WebRTC connection
   */
  private addStreamToWebRTC(): void {
    if (!this.mediaStream) return;
    
    // Replace or add tracks to existing peer connections
    this.mediaStream.getTracks().forEach(track => {
      // Add track to all peer connections
      this.webrtcConnection.addTrackToPeers(track, this.mediaStream!);
    });
  }
  
  /**
   * Send encoded video chunks over WebRTC
   */
  private sendEncodedVideoChunk(data: Uint8Array, isKeyFrame: boolean): void {
    // This method would integrate with WebRTC's RTCRtpSender for more efficient streaming
    // However, since we're using the built-in MediaStream API, this is not used directly
    // It's included here for future optimizations
  }
  
  /**
   * Convert base64 string to ArrayBuffer
   */
  private base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binaryString = window.atob(base64);
    const length = binaryString.length;
    const bytes = new Uint8Array(length);
    
    for (let i = 0; i < length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    
    return bytes.buffer;
  }
  
  /**
   * Start monitoring stream statistics
   */
  private startStatsMonitoring(): void {
    // Clear any existing interval
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
    }
    
    // Poll for stream stats every second
    this.statsInterval = window.setInterval(async () => {
      try {
        const streamInfo = await invoke<StreamInfo>('get_stream_info');
        
        // Dispatch stats to any listeners
        window.dispatchEvent(new CustomEvent('stream-stats', {
          detail: streamInfo
        }));
        
        // Log stats periodically
        console.debug('Stream stats:', streamInfo);
      } catch (error) {
        console.error('Failed to get stream info:', error);
      }
    }, 1000);
  }
  
  /**
   * Clean up resources when stopping capture
   */
  private cleanupResources(): void {
    // Stop stats monitoring
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = null;
    }
    
    // Close the decoder and encoder
    if (this.decoder) {
      try {
        this.decoder.close();
      } catch (e) {
        console.warn('Error closing decoder:', e);
      }
      this.decoder = null;
    }
    
    if (this.encoder) {
      try {
        this.encoder.close();
      } catch (e) {
        console.warn('Error closing encoder:', e);
      }
      this.encoder = null;
    }
    
    // Stop all media tracks
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach(track => track.stop());
      this.mediaStream = null;
    }
    
    this.streamTrack = null;
    this.frameListeners.clear();
  }
  
  /**
   * Get the current media stream
   */
  getMediaStream(): MediaStream | null {
    return this.mediaStream;
  }
  
  /**
   * Check if capture is active
   */
  isCapturing(): boolean {
    return this.captureActive;
  }
  
  /**
   * Register a frame listener
   */
  addFrameListener(listener: (frame: VideoFrame) => void): void {
    this.frameListeners.add(listener);
  }
  
  /**
   * Remove a frame listener
   */
  removeFrameListener(listener: (frame: VideoFrame) => void): void {
    this.frameListeners.delete(listener);
  }
}

// Extend the WebRTCConnection interface to add the track to peers
declare module './webrtc' {
  interface WebRTCConnection {
    addTrackToPeers(track: MediaStreamTrack, stream: MediaStream): number;
  }
}
