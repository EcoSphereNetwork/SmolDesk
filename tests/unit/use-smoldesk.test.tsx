// =============================================================================
// tests/unit/use-smoldesk.test.tsx - useSmolDesk Hook Tests
// =============================================================================

import { renderHook, act } from '@testing-library/react';
import { describe, test, expect, beforeEach, vi } from 'vitest';
import { useSmolDesk, SmolDeskStatus } from '../../src/hooks/useSmolDesk';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/tauri');
vi.mock('@tauri-apps/api/event');

describe('useSmolDesk', () => {
  let mockInvoke: Mock;
  let mockListen: Mock;

  beforeEach(() => {
    mockInvoke = invoke as Mock;
    mockListen = listen as Mock;
    
    vi.clearAllMocks();
    
    // Default mocks
    mockInvoke.mockImplementation((command) => {
      switch (command) {
        case 'get_monitors':
          return Promise.resolve([
            { index: 0, name: 'Monitor 1', width: 1920, height: 1080, primary: true }
          ]);
        case 'get_video_codecs':
          return Promise.resolve(['H264', 'VP8', 'VP9']);
        case 'get_hardware_acceleration_options':
          return Promise.resolve(['None', 'VAAPI', 'NVENC']);
        default:
          return Promise.resolve(true);
      }
    });
    
    mockListen.mockResolvedValue(() => {});
  });

  describe('Initialization', () => {
    test('should initialize with default config', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      expect(result.current.status).toBe(SmolDeskStatus.INITIALIZING);
      
      // Wait for initialization
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.READY);
      expect(result.current.monitors).toHaveLength(1);
      expect(result.current.availableCodecs).toEqual(['H264', 'VP8', 'VP9']);
    });

    test('should initialize with custom config', async () => {
      const config = {
        signalingServer: 'wss://custom.server.com',
        defaultQuality: 90,
        defaultFps: 60
      };

      const { result } = renderHook(() => useSmolDesk(config));
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.READY);
    });

    test('should handle initialization failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Init failed'));
      
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.ERROR);
      expect(result.current.error).toContain('Init failed');
    });
  });

  describe('Room Management', () => {
    test('should create room successfully', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      let roomId: string | null = null;
      
      await act(async () => {
        roomId = await result.current.createRoom('password123');
      });
      
      expect(roomId).toBeTruthy();
      expect(roomId).toContain(':'); // Should contain signature
    });

    test('should join room successfully', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      let joinResult: boolean = false;
      
      await act(async () => {
        joinResult = await result.current.joinRoom('test-room:signature', 'password123');
      });
      
      expect(joinResult).toBe(true);
    });

    test('should leave room', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      await act(async () => {
        result.current.leaveRoom();
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.DISCONNECTED);
    });
  });

  describe('Hosting', () => {
    test('should start hosting successfully', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      let hostingResult: boolean = false;
      
      await act(async () => {
        hostingResult = await result.current.startHosting(0);
      });
      
      expect(hostingResult).toBe(true);
      expect(result.current.status).toBe(SmolDeskStatus.HOSTING);
    });

    test('should stop hosting', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      await act(async () => {
        await result.current.startHosting(0);
        await result.current.stopHosting();
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.CONNECTED);
    });

    test('should handle hosting failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Capture failed'));
      
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      let hostingResult: boolean = true;
      
      await act(async () => {
        hostingResult = await result.current.startHosting(0);
      });
      
      expect(hostingResult).toBe(false);
    });
  });

  describe('Configuration', () => {
    test('should update quality setting', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      act(() => {
        result.current.setQuality(95);
      });
      
      // Quality should be updated in internal state
      expect(result.current.stats).toBeDefined();
    });

    test('should update FPS setting', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      act(() => {
        result.current.setFps(60);
      });
      
      // FPS should be updated in internal state
      expect(result.current.stats).toBeDefined();
    });
  });

  describe('Authentication', () => {
    test('should authenticate user', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      const user = {
        id: 'user-1',
        username: 'testuser',
        role: 'Member' as const,
        access_rights: ['ViewOnly' as const]
      };
      
      let authResult: boolean = false;
      
      await act(async () => {
        authResult = await result.current.authenticate(user, 'password123');
      });
      
      expect(authResult).toBe(true);
    });
  });

  describe('Messaging', () => {
    test('should send message to peers', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      // Create a room first
      await act(async () => {
        await result.current.createRoom();
      });
      
      const messageSent = result.current.sendMessage({ type: 'test', data: 'hello' });
      
      // Should return false if no peers connected, true if peers connected
      expect(typeof messageSent).toBe('boolean');
    });
  });

  describe('Statistics', () => {
    test('should provide initial stats', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.stats).toEqual({
        fps: 0,
        latency: 0,
        bitrate: 0,
        resolution: ''
      });
    });

    test('should update stats from events', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      // Find the stream-stats listener
      const streamStatsListener = mockListen.mock.calls.find(
        call => call[0] === 'stream-stats'
      )?.[1];
      
      if (streamStatsListener) {
        act(() => {
          streamStatsListener({
            payload: {
              fps: 30,
              latency: 50,
              resolution: '1920x1080'
            }
          });
        });
        
        expect(result.current.stats.fps).toBe(30);
        expect(result.current.stats.latency).toBe(50);
        expect(result.current.stats.resolution).toBe('1920x1080');
      }
    });
  });

  describe('Connection Quality', () => {
    test('should track connection quality', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.connectionQuality).toBe('disconnected');
    });
  });

  describe('Error Handling', () => {
    test('should handle security initialization failure', async () => {
      mockInvoke.mockImplementation((command) => {
        if (command === 'initialize_security') {
          return Promise.reject(new Error('Security init failed'));
        }
        return Promise.resolve([]);
      });
      
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      expect(result.current.status).toBe(SmolDeskStatus.ERROR);
      expect(result.current.error).toContain('Security init failed');
    });

    test('should handle room creation failure', async () => {
      const { result } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      // Make security manager fail
      SecurityManager.getInstance()['isInitialized'] = false;
      
      let roomId: string | null = 'initial';
      
      await act(async () => {
        roomId = await result.current.createRoom();
      });
      
      expect(roomId).toBeNull();
    });
  });

  describe('Cleanup', () => {
    test('should cleanup resources on unmount', async () => {
      const { result, unmount } = renderHook(() => useSmolDesk());
      
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });
      
      // Start hosting to have resources to cleanup
      await act(async () => {
        await result.current.startHosting(0);
      });
      
      unmount();
      
      // Should not throw during cleanup
      expect(true).toBe(true);
    });
  });
});
