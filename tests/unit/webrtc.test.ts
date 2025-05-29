// =============================================================================
// tests/unit/webrtc.test.ts - WebRTC Connection Tests
// =============================================================================

import { describe, test, expect, beforeEach, afterEach, vi, Mock } from 'vitest';
import { WebRTCConnection, WebRTCConnectionEvent, WebRTCConnectionOptions } from '../../src/utils/webrtc';

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(public url: string) {
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 0);
  }

  send = vi.fn();
  close = vi.fn(() => {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  });
}

// Mock RTCPeerConnection
class MockRTCPeerConnection {
  localDescription: RTCSessionDescription | null = null;
  remoteDescription: RTCSessionDescription | null = null;
  iceConnectionState: RTCIceConnectionState = 'new';
  connectionState: RTCPeerConnectionState = 'new';
  signalingState: RTCSignalingState = 'stable';

  onicecandidate: ((event: RTCPeerConnectionIceEvent) => void) | null = null;
  ontrack: ((event: RTCTrackEvent) => void) | null = null;
  ondatachannel: ((event: RTCDataChannelEvent) => void) | null = null;
  oniceconnectionstatechange: (() => void) | null = null;
  onconnectionstatechange: (() => void) | null = null;
  onsignalingstatechange: (() => void) | null = null;

  addTrack = vi.fn();
  removeTrack = vi.fn();
  close = vi.fn();
  addIceCandidate = vi.fn();
  getStats = vi.fn().mockResolvedValue(new Map());
  getConfiguration = vi.fn().mockReturnValue({ iceServers: [] });
  setConfiguration = vi.fn();

  createOffer = vi.fn().mockResolvedValue({
    type: 'offer',
    sdp: 'mock-offer-sdp'
  });

  createAnswer = vi.fn().mockResolvedValue({
    type: 'answer',
    sdp: 'mock-answer-sdp'
  });

  setLocalDescription = vi.fn().mockImplementation((desc) => {
    this.localDescription = desc;
    return Promise.resolve();
  });

  setRemoteDescription = vi.fn().mockImplementation((desc) => {
    this.remoteDescription = desc;
    return Promise.resolve();
  });

  createDataChannel = vi.fn().mockReturnValue({
    readyState: 'open',
    send: vi.fn(),
    close: vi.fn(),
    onopen: null,
    onclose: null,
    onmessage: null,
    onerror: null,
  });
}

// Mock navigator.mediaDevices
Object.defineProperty(global.navigator, 'mediaDevices', {
  value: {
    getDisplayMedia: vi.fn().mockResolvedValue({
      getTracks: () => [{ id: 'mock-video-track', kind: 'video' }]
    })
  },
  writable: true
});

// Set up global mocks
Object.defineProperty(global, 'WebSocket', {
  value: MockWebSocket,
  writable: true
});

Object.defineProperty(global, 'RTCPeerConnection', {
  value: MockRTCPeerConnection,
  writable: true
});

Object.defineProperty(global, 'RTCSessionDescription', {
  value: vi.fn().mockImplementation((init) => init),
  writable: true
});

Object.defineProperty(global, 'RTCIceCandidate', {
  value: vi.fn().mockImplementation((init) => init),
  writable: true
});

describe('WebRTCConnection', () => {
  let connection: WebRTCConnection;
  let options: WebRTCConnectionOptions;

  beforeEach(() => {
    vi.clearAllMocks();
    
    options = {
      signalingServer: 'ws://localhost:3000',
      peerConnectionConfig: {
        iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
      },
      autoReconnect: true,
      reconnectInterval: 1000,
      maxReconnectAttempts: 3
    };

    connection = new WebRTCConnection(options);
  });

  afterEach(() => {
    connection.disconnect();
  });

  describe('Construction', () => {
    test('should create WebRTCConnection with default options', () => {
      const conn = new WebRTCConnection({
        signalingServer: 'ws://localhost:3000'
      });
      
      expect(conn).toBeInstanceOf(WebRTCConnection);
    });

    test('should create WebRTCConnection with custom options', () => {
      expect(connection).toBeInstanceOf(WebRTCConnection);
    });
  });

  describe('Connection Management', () => {
    test('should connect to signaling server', async () => {
      const connectSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.SIGNALING_CONNECTED, connectSpy);

      connection.connect();

      await new Promise(resolve => setTimeout(resolve, 10));
      expect(connectSpy).toHaveBeenCalled();
    });

    test('should handle connection errors', async () => {
      const errorSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ERROR, errorSpy);

      // Mock WebSocket to throw error
      MockWebSocket.prototype.constructor = vi.fn().mockImplementation(() => {
        throw new Error('Connection failed');
      });

      connection.connect();

      await new Promise(resolve => setTimeout(resolve, 10));
      expect(errorSpy).toHaveBeenCalled();
    });

    test('should disconnect from signaling server', async () => {
      const disconnectSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.DISCONNECTED, disconnectSpy);

      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));
      
      connection.disconnect();
      expect(disconnectSpy).toHaveBeenCalled();
    });

    test('should handle reconnection', async () => {
      const reconnectSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.RECONNECTING, reconnectSpy);

      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      // Simulate connection loss
      const ws = connection['signalingSocket'];
      ws?.onclose?.(new CloseEvent('close'));

      await new Promise(resolve => setTimeout(resolve, 10));
      expect(reconnectSpy).toHaveBeenCalled();
    });
  });

  describe('Room Management', () => {
    beforeEach(async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));
    });

    test('should create room', async () => {
      const roomCreatedSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ROOM_CREATED, roomCreatedSpy);

      connection.createRoom('test-room');

      // Simulate server response
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-created',
          roomId: 'test-room'
        })
      }));

      expect(roomCreatedSpy).toHaveBeenCalledWith({ roomId: 'test-room' });
    });

    test('should join room', async () => {
      const roomJoinedSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ROOM_JOINED, roomJoinedSpy);

      connection.joinRoom('test-room');

      // Simulate server response
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));

      expect(roomJoinedSpy).toHaveBeenCalledWith({
        roomId: 'test-room',
        peers: [],
        settings: undefined
      });
    });

    test('should leave room', async () => {
      const roomLeftSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ROOM_LEFT, roomLeftSpy);

      // First join a room
      connection.joinRoom('test-room');
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));

      // Then leave it
      connection.leaveRoom();
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-left',
          roomId: 'test-room'
        })
      }));

      expect(roomLeftSpy).toHaveBeenCalled();
    });
  });

  describe('Peer Connection Management', () => {
    beforeEach(async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));
      
      // Join a room first
      connection.joinRoom('test-room');
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));
    });

    test('should handle peer joined', async () => {
      const peerJoinedSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.PEER_JOINED, peerJoinedSpy);

      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      expect(peerJoinedSpy).toHaveBeenCalledWith({ peerId: 'peer-1' });
    });

    test('should handle peer left', async () => {
      const peerLeftSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.PEER_LEFT, peerLeftSpy);

      // First add peer
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      // Then remove peer
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-left',
          peerId: 'peer-1'
        })
      }));

      expect(peerLeftSpy).toHaveBeenCalledWith({ peerId: 'peer-1' });
    });

    test('should handle WebRTC offer', async () => {
      const ws = connection['signalingSocket'];
      
      // Simulate receiving an offer
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'offer',
          peerId: 'peer-1',
          offer: {
            type: 'offer',
            sdp: 'mock-offer-sdp'
          }
        })
      }));

      // Verify peer connection was created
      expect(connection['peerConnections'].has('peer-1')).toBe(true);
    });

    test('should handle WebRTC answer', async () => {
      // First create a peer connection by simulating peer joined
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      // Then handle answer
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'answer',
          peerId: 'peer-1',
          answer: {
            type: 'answer',
            sdp: 'mock-answer-sdp'
          }
        })
      }));

      const peerConnection = connection['peerConnections'].get('peer-1');
      expect(peerConnection?.setRemoteDescription).toHaveBeenCalled();
    });

    test('should handle ICE candidates', async () => {
      // First create a peer connection
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      // Handle ICE candidate
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'ice-candidate',
          peerId: 'peer-1',
          candidate: {
            candidate: 'mock-candidate',
            sdpMid: 'video',
            sdpMLineIndex: 0
          }
        })
      }));

      const peerConnection = connection['peerConnections'].get('peer-1');
      expect(peerConnection?.addIceCandidate).toHaveBeenCalled();
    });
  });

  describe('Media Stream Handling', () => {
    beforeEach(async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));
    });

    test('should get local stream', async () => {
      const stream = await connection.getLocalStream({
        audio: false,
        video: true
      });

      expect(stream).toBeDefined();
      expect(stream.getTracks()).toHaveLength(1);
    });

    test('should handle stream errors', async () => {
      const errorSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ERROR, errorSpy);

      // Mock getDisplayMedia to throw error
      navigator.mediaDevices.getDisplayMedia = vi.fn().mockRejectedValue(
        new Error('Permission denied')
      );

      await expect(connection.getLocalStream()).rejects.toThrow('Permission denied');
      expect(errorSpy).toHaveBeenCalled();
    });

    test('should add tracks to existing peer connections', async () => {
      // First join room and add peer
      connection.joinRoom('test-room');
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));

      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      // Then get local stream
      await connection.getLocalStream();

      const peerConnection = connection['peerConnections'].get('peer-1');
      expect(peerConnection?.addTrack).toHaveBeenCalled();
    });
  });

  describe('Data Channels', () => {
    beforeEach(async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));
      
      // Set up peer connection
      connection.joinRoom('test-room');
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));

      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));
    });

    test('should send data to specific peer', () => {
      const result = connection.sendData('peer-1', { message: 'hello' });
      expect(result).toBe(true);
    });

    test('should handle send data to non-existent peer', () => {
      const result = connection.sendData('non-existent', { message: 'hello' });
      expect(result).toBe(false);
    });

    test('should broadcast data to all peers', () => {
      const count = connection.broadcast({ message: 'hello everyone' });
      expect(count).toBe(1);
    });

    test('should handle data channel messages', () => {
      const messageSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.DATA_CHANNEL_MESSAGE, messageSpy);

      // Simulate data channel message
      const dataChannel = connection['dataChannels'].get('peer-1');
      dataChannel?.onmessage?.({
        data: JSON.stringify({ message: 'hello' })
      } as MessageEvent);

      expect(messageSpy).toHaveBeenCalledWith({
        peerId: 'peer-1',
        data: { message: 'hello' }
      });
    });
  });

  describe('Event System', () => {
    test('should add and remove event listeners', () => {
      const listener = vi.fn();
      
      connection.on(WebRTCConnectionEvent.CONNECTED, listener);
      connection.emit(WebRTCConnectionEvent.CONNECTED, { test: 'data' });
      
      expect(listener).toHaveBeenCalledWith({ test: 'data' });
      
      connection.off(WebRTCConnectionEvent.CONNECTED, listener);
      connection.emit(WebRTCConnectionEvent.CONNECTED, { test: 'data2' });
      
      expect(listener).toHaveBeenCalledTimes(1);
    });

    test('should handle listener errors gracefully', () => {
      const errorListener = vi.fn(() => {
        throw new Error('Listener error');
      });
      
      connection.on(WebRTCConnectionEvent.CONNECTED, errorListener);
      
      expect(() => connection.emit(WebRTCConnectionEvent.CONNECTED)).not.toThrow();
    });
  });

  describe('Ping/Pong Mechanism', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    test('should send ping messages periodically', async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = connection['signalingSocket'];
      const sendSpy = vi.spyOn(ws!, 'send');

      // Fast-forward 30 seconds
      vi.advanceTimersByTime(30000);

      expect(sendSpy).toHaveBeenCalledWith(
        JSON.stringify({ type: 'ping' })
      );
    });

    test('should handle pong responses', async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = connection['signalingSocket'];
      
      // Simulate pong response (should not cause any errors)
      expect(() => {
        ws?.onmessage?.(new MessageEvent('message', {
          data: JSON.stringify({ type: 'pong' })
        }));
      }).not.toThrow();
    });
  });

  describe('Error Handling', () => {
    test('should handle malformed signaling messages', async () => {
      const errorSpy = vi.fn();
      connection.on(WebRTCConnectionEvent.ERROR, errorSpy);

      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: 'invalid-json'
      }));

      expect(errorSpy).toHaveBeenCalled();
    });

    test('should handle unknown message types', async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = connection['signalingSocket'];
      
      expect(() => {
        ws?.onmessage?.(new MessageEvent('message', {
          data: JSON.stringify({ type: 'unknown-type' })
        }));
      }).not.toThrow();
    });

    test('should handle peer connection creation errors', async () => {
      // Mock RTCPeerConnection to throw error
      const originalRTCPeerConnection = global.RTCPeerConnection;
      global.RTCPeerConnection = vi.fn().mockImplementation(() => {
        throw new Error('PeerConnection error');
      });

      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      const ws = connection['signalingSocket'];
      
      expect(() => {
        ws?.onmessage?.(new MessageEvent('message', {
          data: JSON.stringify({
            type: 'peer-joined',
            peerId: 'peer-1'
          })
        }));
      }).not.toThrow();

      // Restore original
      global.RTCPeerConnection = originalRTCPeerConnection;
    });
  });

  describe('Cleanup', () => {
    test('should close all peer connections on disconnect', async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      // Add some peers
      connection.joinRoom('test-room');
      const ws = connection['signalingSocket'];
      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'room-joined',
          roomId: 'test-room',
          peers: []
        })
      }));

      ws?.onmessage?.(new MessageEvent('message', {
        data: JSON.stringify({
          type: 'peer-joined',
          peerId: 'peer-1'
        })
      }));

      const peerConnection = connection['peerConnections'].get('peer-1');
      const closeSpy = vi.spyOn(peerConnection!, 'close');

      connection.disconnect();

      expect(closeSpy).toHaveBeenCalled();
      expect(connection['peerConnections'].size).toBe(0);
    });

    test('should clear reconnect timers on disconnect', async () => {
      connection.connect();
      await new Promise(resolve => setTimeout(resolve, 10));

      // Simulate connection loss to start reconnect timer
      const ws = connection['signalingSocket'];
      ws?.onclose?.(new CloseEvent('close'));

      // Then disconnect before reconnect
      connection.disconnect();

      expect(connection['reconnectTimer']).toBeNull();
    });
  });
});
