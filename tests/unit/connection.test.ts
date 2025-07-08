// =============================================================================
// tests/integration/connection.test.ts - Integration Tests
// =============================================================================

import { describe, test, expect, beforeEach, afterEach, vi } from 'vitest';
import { WebRTCConnection } from '../../src/utils/webrtc';
import { ScreenCaptureManager } from '../../src/utils/screenCapture';
import { SecurityManager } from '../../src/utils/securityManager';

describe.skip('SmolDesk Integration Tests', () => {
  let webrtcConnection: WebRTCConnection;
  let captureManager: ScreenCaptureManager;
  let securityManager: SecurityManager;

  beforeEach(async () => {
    // Initialize components
    webrtcConnection = new WebRTCConnection({
      signalingServer: 'ws://localhost:3000'
    });

    captureManager = new ScreenCaptureManager(webrtcConnection);
    securityManager = SecurityManager.getInstance();

    await securityManager.initialize('test-key');
  });

  afterEach(() => {
    webrtcConnection.disconnect();
  });

  describe('WebRTC and Screen Capture Integration', () => {
    test('should integrate screen capture with WebRTC', async () => {
      const connectPromise = new Promise((resolve) => {
        webrtcConnection.on('signaling-connected', resolve);
      });

      webrtcConnection.connect();
      await connectPromise;

      const captureResult = await captureManager.startCapture(0, {
        fps: 30,
        quality: 80,
        codec: 'H264'
      });

      expect(captureResult).toBe(true);
      expect(captureManager.getMediaStream()).toBeTruthy();
    });

    test('should handle WebRTC connection loss during capture', async () => {
      await captureManager.startCapture(0, {
        fps: 30,
        quality: 80
      });

      // Simulate connection loss
      webrtcConnection.disconnect();

      // Capture should continue locally
      expect(captureManager.isCapturing()).toBe(true);
    });
  });

  describe('Security and WebRTC Integration', () => {
    test('should create secure room and connect', async () => {
      const connectPromise = new Promise((resolve) => {
        webrtcConnection.on('signaling-connected', resolve);
      });

      webrtcConnection.connect();
      await connectPromise;

      const secureRoomId = await securityManager.createSecureRoom('password123');
      expect(secureRoomId).toBeTruthy();

      const roomId = secureRoomId!.split(':')[0];
      webrtcConnection.createRoom(roomId);

      const roomCreatedPromise = new Promise((resolve) => {
        webrtcConnection.on('room-created', resolve);
      });

      await roomCreatedPromise;
      expect(webrtcConnection['roomId']).toBe(roomId);
    });
  });
});
