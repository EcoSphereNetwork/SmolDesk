// tests/unit/screen-capture.test.ts 

import { describe, test, expect, beforeEach, afterEach, vi, Mock } from 'vitest';
import { ScreenCaptureManager } from '../../src/utils/screenCapture';
import { WebRTCConnection } from '../../src/utils/webrtc';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/tauri');
vi.mock('@tauri-apps/api/event');

// Mock WebRTC APIs
const mockWebRTCConnection = {
  addTrackToPeers: vi.fn().mockReturnValue(2),
  on: vi.fn(),
  off: vi.fn(),
} as Partial<WebRTCConnection>;

// Mock WebCodecs APIs
const mockVideoEncoder = {
  configure: vi.fn(),
  encode: vi.fn(),
  close: vi.fn(),
};

const mockVideoDecoder = {
  configure: vi.fn(),
  decode: vi.fn(),
  close: vi.fn(),
};

// Mock Canvas APIs
const mockCanvas = {
  width: 1920,
  height: 1080,
  getContext: vi.fn().mockReturnValue({
    drawImage: vi.fn(),
  }),
  captureStream: vi.fn().mockReturnValue({
    getVideoTracks: vi.fn().mockReturnValue([
      { id: 'mock-video-track', stop: vi.fn() }
    ])
  }),
};

// Mock global APIs
Object.defineProperty(global, 'VideoEncoder', {
  value: vi.fn(() => mockVideoEncoder),
  writable: true,
});

Object.defineProperty(global, 'VideoDecoder', {  
  value: vi.fn(() => mockVideoDecoder),
  writable: true,
});

Object.defineProperty(global, 'EncodedVideoChunk', {
  value: vi.fn().mockImplementation((data) => data),
  writable: true,
});

Object.defineProperty(global, 'VideoFrame', {
  value: vi.fn().mockImplementation((source, init) => ({
    codedWidth: 1920,
    codedHeight: 1080,
    duration: init?.duration || 33333,
    close: vi.fn(),
  })),
  writable: true,
});

Object.defineProperty(document, 'createElement', {
  value: vi.fn((tag: string) => {
    if (tag === 'canvas') return mockCanvas;
    if (tag === 'video') return { autoplay: true, muted: true };
    if (tag === 'img') return { 
      onload: null, 
      src: '', 
      width: 1920, 
      height: 1080,
      addEventListener: vi.fn()
    };
    return {};
  }),
  writable: true,
});

Object.defineProperty(global, 'atob', {
  value: vi.fn((str) => 'mock-decoded-data'),
  writable: true,
});

Object.defineProperty(global, 'btoa', {
  value: vi.fn((str) => 'mock-encoded-data'),
  writable: true,
});

describe('ScreenCaptureManager', () => {
  let captureManager: ScreenCaptureManager;
  let mockInvoke: Mock;
  let mockListen: Mock;

  beforeEach(() => {
    mockInvoke = invoke as Mock;
    mockListen = listen as Mock;
    
    // Reset all mocks
    vi.clearAllMocks();
    
    // Setup default mock implementations
    mockInvoke.mockResolvedValue(true);
    mockListen.mockResolvedValue(() => {});
    
    captureManager = new ScreenCaptureManager(mockWebRTCConnection as WebRTCConnection);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Construction', () => {
    test('should create ScreenCaptureManager instance', () => {
      expect(captureManager).toBeInstanceOf(ScreenCaptureManager);
      expect(captureManager.isCapturing()).toBe(false);
      expect(captureManager.getMediaStream()).toBeNull();
    });
  });

  describe('startCapture', () => {
    test('should start capture successfully with valid parameters', async () => {
      const config = {
        fps: 30,
        quality: 80,
        codec: 'H264',
        hardware_acceleration: 'VAAPI',
      };

      const result = await captureManager.startCapture(0, config);
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('start_capture', {
        monitorIndex: 0,
        config,
      });
      expect(captureManager.isCapturing()).toBe(true);
    });

    test('should handle capture start failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed to start capture'));
      
      const result = await captureManager.startCapture(0, {});
      
      expect(result).toBe(false);
      expect(captureManager.isCapturing()).toBe(false);
    });

    test('should setup WebCodecs processing when supported', async () => {
      await captureManager.startCapture(0, {});
      
      expect(mockVideoDecoder.configure).toHaveBeenCalled();
      expect(mockVideoEncoder.configure).toHaveBeenCalled();
      expect(mockListen).toHaveBeenCalledWith('frame_data', expect.any(Function));
    });

    test('should fallback to traditional processing when WebCodecs not available', async () => {
      // Temporarily remove WebCodecs support
      const originalVideoEncoder = global.VideoEncoder;
      delete (global as any).VideoEncoder;
      
      await captureManager.startCapture(0, {});
      
      expect(mockListen).toHaveBeenCalledWith('frame_data', expect.any(Function));
      
      // Restore WebCodecs
      global.VideoEncoder = originalVideoEncoder;
    });

    test('should configure encoder with proper parameters', async () => {
      await captureManager.startCapture(0, {});
      
      expect(mockVideoEncoder.configure).toHaveBeenCalledWith({
        codec: 'vp8',
        width: 1920,
        height: 1080,
        bitrate: 2_000_000,
        framerate: 30,
      });
    });
  });

  describe('stopCapture', () => {
    test('should stop capture successfully', async () => {
      await captureManager.startCapture(0, {});
      
      const result = await captureManager.stopCapture();
      
      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('stop_capture');
      expect(captureManager.isCapturing()).toBe(false);
    });

    test('should handle stop capture failure', async () => {
      await captureManager.startCapture(0, {});
      mockInvoke.mockRejectedValueOnce(new Error('Failed to stop capture'));
      
      const result = await captureManager.stopCapture();
      
      expect(result).toBe(false);
    });

    test('should cleanup resources when stopping', async () => {
      await captureManager.startCapture(0, {});
      await captureManager.stopCapture();
      
      expect(mockVideoDecoder.close).toHaveBeenCalled();
      expect(mockVideoEncoder.close).toHaveBeenCalled();
      expect(captureManager.getMediaStream()).toBeNull();
    });
  });

  describe('Frame Processing', () => {
    test('should process frame data with WebCodecs', async () => {
      await captureManager.startCapture(0, {});
      
      // Get the frame listener
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      // Mock frame data event
      const mockFrameData = {
        payload: btoa('mock-frame-data')
      };
      
      expect(() => frameListener(mockFrameData)).not.toThrow();
      expect(mockVideoDecoder.decode).toHaveBeenCalled();
    });

    test('should handle frame processing errors gracefully', async () => {
      mockVideoDecoder.decode.mockImplementationOnce(() => {
        throw new Error('Decode error');
      });
      
      await captureManager.startCapture(0, {});
      
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      expect(() => frameListener({ payload: 'invalid-data' })).not.toThrow();
    });

    test('should process traditional frame data without WebCodecs', async () => {
      // Remove WebCodecs support
      delete (global as any).VideoEncoder;
      delete (global as any).VideoDecoder;
      
      await captureManager.startCapture(0, {});
      
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      const mockFrameData = {
        payload: 'base64-image-data'
      };
      
      expect(() => frameListener(mockFrameData)).not.toThrow();
    });
  });

  describe('Frame Listeners', () => {
    test('should add and remove frame listeners', async () => {
      const listener = vi.fn();
      
      captureManager.addFrameListener(listener);
      captureManager.addFrameListener(listener); // Should not add duplicate
      
      captureManager.removeFrameListener(listener);
      
      expect(captureManager['frameListeners'].size).toBe(0);
    });

    test('should notify frame listeners of new frames', async () => {
      const listener = vi.fn();
      captureManager.addFrameListener(listener);
      
      await captureManager.startCapture(0, {});
      
      // Simulate frame processing
      const mockFrame = { codedWidth: 1920, codedHeight: 1080, close: vi.fn() };
      
      // Call the frame output callback directly
      const decoderOutput = (mockVideoDecoder.constructor as Mock).mock.calls[0][0].output;
      decoderOutput(mockFrame);
      
      expect(listener).toHaveBeenCalled();
    });

    test('should handle listener errors gracefully', async () => {
      const errorListener = vi.fn(() => {
        throw new Error('Listener error');
      });
      
      captureManager.addFrameListener(errorListener);
      
      await captureManager.startCapture(0, {});
      
      const mockFrame = { codedWidth: 1920, codedHeight: 1080, close: vi.fn() };
      const decoderOutput = (mockVideoDecoder.constructor as Mock).mock.calls[0][0].output;
      
      expect(() => decoderOutput(mockFrame)).not.toThrow();
    });
  });

  describe('Media Stream Integration', () => {
    test('should create synthetic media stream', async () => {
      await captureManager.startCapture(0, {});
      
      const stream = captureManager.getMediaStream();
      expect(stream).toBeTruthy();
      expect(mockWebRTCConnection.addTrackToPeers).toHaveBeenCalled();
    });

    test('should handle canvas context creation failure', async () => {
      mockCanvas.getContext.mockReturnValueOnce(null);
      
      await expect(captureManager.startCapture(0, {})).rejects.toThrow(
        'Failed to create canvas context'
      );
    });

    test('should resize canvas when frame size changes', async () => {
      await captureManager.startCapture(0, {});
      
      const listener = captureManager['frameListeners'].values().next().value;
      const mockFrame = { 
        codedWidth: 2560, 
        codedHeight: 1440, 
        close: vi.fn() 
      };
      
      listener(mockFrame);
      
      expect(mockCanvas.width).toBe(2560);
      expect(mockCanvas.height).toBe(1440);
    });
  });

  describe('Stats Monitoring', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    test('should start stats monitoring during capture', async () => {
      mockInvoke.mockResolvedValueOnce(true); // start_capture
      mockInvoke.mockResolvedValue({ // get_stream_info
        fps: 30,
        latency: 50,
        resolution: '1920x1080'
      });

      await captureManager.startCapture(0, {});
      
      // Fast-forward past the stats interval
      vi.advanceTimersByTime(1000);
      
      expect(mockInvoke).toHaveBeenCalledWith('get_stream_info');
    });

    test('should dispatch custom events with stats', async () => {
      const eventSpy = vi.spyOn(window, 'dispatchEvent');
      
      mockInvoke.mockResolvedValueOnce(true);
      mockInvoke.mockResolvedValue({
        fps: 30,
        latency: 50,
        resolution: '1920x1080'
      });

      await captureManager.startCapture(0, {});
      vi.advanceTimersByTime(1000);
      
      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'stream-stats',
          detail: expect.objectContaining({
            fps: 30,
            latency: 50,
            resolution: '1920x1080'
          })
        })
      );
    });

    test('should stop stats monitoring when capture stops', async () => {
      await captureManager.startCapture(0, {});
      await captureManager.stopCapture();
      
      mockInvoke.mockClear();
      
      // Fast-forward past the stats interval
      vi.advanceTimersByTime(1000);
      
      expect(mockInvoke).not.toHaveBeenCalledWith('get_stream_info');
    });

    test('should handle stats retrieval errors', async () => {
      mockInvoke.mockResolvedValueOnce(true);
      mockInvoke.mockRejectedValue(new Error('Stats error'));

      await captureManager.startCapture(0, {});
      
      expect(() => vi.advanceTimersByTime(1000)).not.toThrow();
    });
  });

  describe('Format Changes', () => {
    test('should handle frame format changes', async () => {
      await captureManager.startCapture(0, {});
      
      const formatListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_format'
      )[1];
      
      const newFormat = {
        payload: {
          codec: 'VP9',
          width: 2560,
          height: 1440
        }
      };
      
      formatListener(newFormat);
      
      expect(mockVideoDecoder.configure).toHaveBeenCalledWith({
        codec: 'vp9'
      });
      
      expect(mockVideoEncoder.configure).toHaveBeenCalledWith({
        codec: 'vp9',
        width: 2560,
        height: 1440,
        bitrate: 2_000_000,
        framerate: 30,
      });
    });

    test('should handle invalid format changes', async () => {
      await captureManager.startCapture(0, {});
      
      const formatListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_format'
      )[1];
      
      expect(() => formatListener({ payload: null })).not.toThrow();
      expect(() => formatListener({ payload: {} })).not.toThrow();
    });
  });

  describe('Error Handling', () => {
    test('should handle invalid base64 data gracefully', async () => {
      await captureManager.startCapture(0, {});
      
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      // Should not throw on invalid base64
      expect(() => frameListener({ payload: 'invalid-base64!' })).not.toThrow();
    });

    test('should handle decoder configuration errors', async () => {
      mockVideoDecoder.configure.mockImplementationOnce(() => {
        throw new Error('Configuration error');
      });
      
      await captureManager.startCapture(0, {});
      
      const formatListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_format'
      )[1];
      
      expect(() => formatListener({
        payload: { codec: 'vp8', width: 1920, height: 1080 }
      })).not.toThrow();
    });

    test('should handle encoder output errors', async () => {
      const mockEncoderOutput = vi.fn().mockImplementation(() => {
        throw new Error('Encoder output error');
      });
      
      (mockVideoEncoder.constructor as Mock).mockImplementationOnce((config) => ({
        ...mockVideoEncoder,
        output: mockEncoderOutput
      }));
      
      expect(() => captureManager.startCapture(0, {})).not.toThrow();
    });
  });

  describe('Memory Management', () => {
    test('should cleanup all resources on destruction', async () => {
      await captureManager.startCapture(0, {});
      
      // Manually call cleanup
      captureManager['cleanupResources']();
      
      expect(mockVideoDecoder.close).toHaveBeenCalled();
      expect(mockVideoEncoder.close).toHaveBeenCalled();
      expect(captureManager.getMediaStream()).toBeNull();
      expect(captureManager['frameListeners'].size).toBe(0);
    });

    test('should handle cleanup errors gracefully', async () => {
      mockVideoDecoder.close.mockImplementationOnce(() => {
        throw new Error('Close error');
      });
      
      await captureManager.startCapture(0, {});
      
      expect(() => captureManager['cleanupResources']()).not.toThrow();
    });

    test('should stop media tracks when cleaning up', async () => {
      const mockTrack = { stop: vi.fn() };
      mockCanvas.captureStream.mockReturnValueOnce({
        getVideoTracks: () => [mockTrack],
        getTracks: () => [mockTrack]
      });
      
      await captureManager.startCapture(0, {});
      captureManager['cleanupResources']();
      
      expect(mockTrack.stop).toHaveBeenCalled();
    });
  });

  describe('Performance', () => {
    test('should limit frame listener execution time', async () => {
      const slowListener = vi.fn(() => {
        // Simulate slow processing
        const end = Date.now() + 100;
        while (Date.now() < end) {}
      });
      
      captureManager.addFrameListener(slowListener);
      
      await captureManager.startCapture(0, {});
      
      const mockFrame = { codedWidth: 1920, codedHeight: 1080, close: vi.fn() };
      const decoderOutput = (mockVideoDecoder.constructor as Mock).mock.calls[0][0].output;
      
      const startTime = Date.now();
      decoderOutput(mockFrame);
      const endTime = Date.now();
      
      // Should complete reasonably quickly despite slow listener
      expect(endTime - startTime).toBeLessThan(500);
    });

    test('should handle multiple simultaneous frame processing', async () => {
      await captureManager.startCapture(0, {});
      
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      // Process multiple frames simultaneously
      const promises = Array.from({ length: 10 }, (_, i) => 
        Promise.resolve().then(() => frameListener({ payload: `frame-${i}` }))
      );
      
      expect(() => Promise.all(promises)).not.toThrow();
    });
  });

  describe('Edge Cases', () => {
    test('should handle capture start when already capturing', async () => {
      await captureManager.startCapture(0, {});
      
      // Should handle gracefully
      const result = await captureManager.startCapture(0, {});
      expect(result).toBe(true);
    });

    test('should handle stop capture when not capturing', async () => {
      const result = await captureManager.stopCapture();
      expect(result).toBe(true);
    });

    test('should handle empty frame data', async () => {
      await captureManager.startCapture(0, {});
      
      const frameListener = mockListen.mock.calls.find(
        call => call[0] === 'frame_data'
      )[1];
      
      expect(() => frameListener({ payload: '' })).not.toThrow();
      expect(() => frameListener({ payload: null })).not.toThrow();
      expect(() => frameListener({})).not.toThrow();
    });

    test('should handle canvas capture stream failure', async () => {
      mockCanvas.captureStream.mockImplementationOnce(() => {
        throw new Error('Capture stream error');
      });
      
      await expect(captureManager.startCapture(0, {})).rejects.toThrow();
    });
  });
});

describe('ScreenCaptureManager Integration', () => {
  test('should integrate with WebRTC connection properly', async () => {
    const captureManager = new ScreenCaptureManager(mockWebRTCConnection as WebRTCConnection);
    
    await captureManager.startCapture(0, {});
    
    expect(mockWebRTCConnection.addTrackToPeers).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'mock-video-track' }),
      expect.any(Object)
    );
  });

  test('should handle WebRTC connection failures gracefully', async () => {
    mockWebRTCConnection.addTrackToPeers = vi.fn().mockImplementationOnce(() => {
      throw new Error('WebRTC error');
    });
    
    const captureManager = new ScreenCaptureManager(mockWebRTCConnection as WebRTCConnection);
    
    // Should not throw even if WebRTC integration fails
    await expect(captureManager.startCapture(0, {})).resolves.toBe(true);
  });

  test('should handle multiple track additions', async () => {
    const captureManager = new ScreenCaptureManager(mockWebRTCConnection as WebRTCConnection);
    
    // Mock multiple tracks
    mockCanvas.captureStream.mockReturnValueOnce({
      getVideoTracks: () => [
        { id: 'track-1' },
        { id: 'track-2' }
      ],
      getTracks: () => [
        { id: 'track-1', stop: vi.fn() },
        { id: 'track-2', stop: vi.fn() }
      ]
    });
    
    await captureManager.startCapture(0, {});
    
    expect(mockWebRTCConnection.addTrackToPeers).toHaveBeenCalledTimes(2);
  });
});

