// =============================================================================
// tests/unit/enhanced-webrtc.test.ts - Enhanced WebRTC Tests
// =============================================================================

import { describe, test, expect, beforeEach, vi } from 'vitest';
import { EnhancedWebRTCConnection, ConnectionQuality } from '../../src/utils/enhancedWebRTC';

describe('EnhancedWebRTCConnection', () => {
  let connection: EnhancedWebRTCConnection;

  beforeEach(() => {
    vi.clearAllMocks();
    
    connection = new EnhancedWebRTCConnection({
      signalingServer: 'ws://localhost:3000',
      iceGatheringTimeout: 5000,
      enableTrickleICE: true,
      bandwidthConstraints: {
        video: 2000,
        audio: 64
      },
      maxICERestarts: 3
    });
  });

  describe('Enhanced Features', () => {
    test('should configure bandwidth constraints', () => {
      connection.setBandwidthConstraints({
        video: 1500,
        audio: 32,
        screen: 3000
      });

      expect(connection['bandwidthConstraints']).toEqual({
        video: 1500,
        audio: 32,
        screen: 3000
      });
    });

    test('should handle ICE restart on connection failure', async () => {
      const mockPeerConnection = new MockRTCPeerConnection();
      connection['peerConnections'] = new Map([['peer-1', mockPeerConnection]]);

      // Simulate ICE failure
      await connection['handleICEFailure']('peer-1');

      expect(mockPeerConnection.createOffer).toHaveBeenCalledWith({
        iceRestart: true,
        offerToReceiveAudio: true,
        offerToReceiveVideo: true
      });
    });

    test('should monitor connection quality', async () => {
      const mockStats = new Map([
        ['inbound-rtp', {
          type: 'inbound-rtp',
          mediaType: 'video',
          packetsLost: 5,
          packetsReceived: 1000,
          jitter: 0.01,
          framesDecoded: 500,
          framesDropped: 2
        }],
        ['remote-inbound-rtp', {
          type: 'remote-inbound-rtp',
          roundTripTime: 0.05
        }]
      ]);

      const quality = connection['processConnectionStats']('peer-1', mockStats);
      expect(quality).toBe(ConnectionQuality.GOOD);
    });

    test('should configure TURN servers with fallback', () => {
      const turnServers = [
        { urls: 'turn:turn1.example.com:3478', username: 'user1', credential: 'pass1' },
        { urls: 'turn:turn2.example.com:3478', username: 'user2', credential: 'pass2' }
      ];

      connection.configureTURNServers(turnServers);
      expect(connection['prioritizedTurnServers']).toEqual(turnServers);
    });

    test('should remove duplicate ICE servers', () => {
      const servers = [
        { urls: 'stun:stun1.example.com' },
        { urls: 'stun:stun1.example.com' }, // duplicate
        { urls: 'stun:stun2.example.com' }
      ];

      const unique = connection['removeDuplicateIceServers'](servers);
      expect(unique).toHaveLength(2);
    });
  });

  describe('Connection Quality Assessment', () => {
    test('should classify excellent connection quality', () => {
      const mockStats = new Map([
        ['inbound-rtp', {
          type: 'inbound-rtp',
          mediaType: 'video',
          packetsLost: 0,
          packetsReceived: 1000,
          jitter: 0.001,
          framesDecoded: 500,
          framesDropped: 0
        }],
        ['remote-inbound-rtp', {
          type: 'remote-inbound-rtp',
          roundTripTime: 0.02
        }]
      ]);

      const quality = connection['processConnectionStats']('peer-1', mockStats);
      expect(quality).toBe(ConnectionQuality.EXCELLENT);
    });

    test('should classify critical connection quality', () => {
      const mockStats = new Map([
        ['inbound-rtp', {
          type: 'inbound-rtp',
          mediaType: 'video',
          packetsLost: 200,
          packetsReceived: 800,
          jitter: 0.1,
          framesDecoded: 300,
          framesDropped: 150
        }],
        ['remote-inbound-rtp', {
          type: 'remote-inbound-rtp',
          roundTripTime: 0.6
        }]
      ]);

      const quality = connection['processConnectionStats']('peer-1', mockStats);
      expect(quality).toBe(ConnectionQuality.CRITICAL);
    });
  });

  describe('SDP Modification', () => {
    test('should modify SDP for bandwidth constraints', () => {
      const originalSdp = `v=0
o=- 123456 2 IN IP4 127.0.0.1
s=-
m=video 9 UDP/TLS/RTP/SAVPF 96
a=rtpmap:96 VP8/90000
m=audio 9 UDP/TLS/RTP/SAVPF 111
a=rtpmap:111 opus/48000/2`;

      connection['bandwidthConstraints'] = {
        video: 2000,
        audio: 64
      };

      const modifiedSdp = connection['modifySdpForBandwidth'](originalSdp);
      
      expect(modifiedSdp).toContain('b=AS:2000');
      expect(modifiedSdp).toContain('b=AS:64');
    });

    test('should handle SDP without bandwidth constraints', () => {
      const originalSdp = 'm=video 9 UDP/TLS/RTP/SAVPF 96';
      const modifiedSdp = connection['modifySdpForBandwidth'](originalSdp);
      
      expect(modifiedSdp).toBe(originalSdp);
    });
  });

  describe('ICE Restart Logic', () => {
    test('should limit ICE restarts to maximum attempts', async () => {
      connection['iceRestartCounts'].set('peer-1', 3); // Already at max
      connection['maxICERestarts'] = 3;

      const eventSpy = vi.fn();
      connection.on('connection-quality-change', eventSpy);

      await connection['handleICEFailure']('peer-1');

      expect(eventSpy).toHaveBeenCalledWith({
        peerId: 'peer-1',
        quality: ConnectionQuality.DISCONNECTED,
        reason: 'Maximum ICE restarts reached'
      });
    });

    test('should increment restart count on ICE failure', async () => {
      connection['iceRestartCounts'].set('peer-1', 1);
      const mockPeerConnection = new MockRTCPeerConnection();
      connection['peerConnections'] = new Map([['peer-1', mockPeerConnection]]);

      await connection['handleICEFailure']('peer-1');

      expect(connection['iceRestartCounts'].get('peer-1')).toBe(2);
    });
  });

  describe('Stats Monitoring', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    test('should start stats monitoring for new peers', () => {
      connection['startStatsMonitoring']('peer-1');
      
      expect(connection['statsIntervals'].has('peer-1')).toBe(true);
    });

    test('should stop stats monitoring when peer leaves', () => {
      connection['startStatsMonitoring']('peer-1');
      connection['stopStatsMonitoring']('peer-1');
      
      expect(connection['statsIntervals'].has('peer-1')).toBe(false);
    });

    test('should emit quality change events', async () => {
      const mockPeerConnection = new MockRTCPeerConnection();
      const mockStats = new Map([
        ['inbound-rtp', {
          type: 'inbound-rtp',
          mediaType: 'video',
          packetsLost: 50,
          packetsReceived: 1000,
          jitter: 0.05,
          framesDecoded: 500,
          framesDropped: 25
        }]
      ]);

      mockPeerConnection.getStats.mockResolvedValue(mockStats);
      connection['peerConnections'] = new Map([['peer-1', mockPeerConnection]]);

      const qualityChangeSpy = vi.fn();
      connection.on('connection-quality-change', qualityChangeSpy);

      connection['startStatsMonitoring']('peer-1');
      
      // Advance timer to trigger stats check
      vi.advanceTimersByTime(2000);
      
      await new Promise(resolve => setTimeout(resolve, 0));
      
      expect(qualityChangeSpy).toHaveBeenCalled();
    });
  });

  describe('Connection Recreation', () => {
    test('should recreate failed connections', () => {
      const closeSpy = vi.fn();
      const createSpy = vi.fn();
      
      connection['closePeerConnection'] = closeSpy;
      connection['createPeerConnection'] = createSpy;

      connection['handleConnectionFailure']('peer-1');

      expect(closeSpy).toHaveBeenCalledWith('peer-1');
      expect(createSpy).toHaveBeenCalledWith('peer-1', true);
    });

    test('should expose recreateConnection method', () => {
      const handleFailureSpy = vi.spyOn(connection, 'handleConnectionFailure' as any);
      
      const result = connection.recreateConnection('peer-1');
      
      expect(handleFailureSpy).toHaveBeenCalledWith('peer-1');
      expect(result).toBe(true);
    });
  });

  describe('Track Management', () => {
    test('should add tracks to all peer connections', () => {
      const mockPeerConnection1 = new MockRTCPeerConnection();
      const mockPeerConnection2 = new MockRTCPeerConnection();
      
      connection['peerConnections'] = new Map([
        ['peer-1', mockPeerConnection1],
        ['peer-2', mockPeerConnection2]
      ]);

      // Mock forEachPeerConnection to actually iterate
      connection['forEachPeerConnection'] = (callback) => {
        connection['peerConnections'].forEach(callback);
      };

      const mockTrack = { id: 'track-1', kind: 'video' } as MediaStreamTrack;
      const mockStream = { id: 'stream-1' } as MediaStream;

      const count = connection.addTrackToPeers(mockTrack, mockStream);

      expect(count).toBe(2);
      expect(mockPeerConnection1.addTrack).toHaveBeenCalledWith(mockTrack, mockStream);
      expect(mockPeerConnection2.addTrack).toHaveBeenCalledWith(mockTrack, mockStream);
    });

    test('should handle track addition errors gracefully', () => {
      const mockPeerConnection = new MockRTCPeerConnection();
      mockPeerConnection.addTrack.mockImplementation(() => {
        throw new Error('Track addition failed');
      });
      
      connection['peerConnections'] = new Map([['peer-1', mockPeerConnection]]);
      connection['forEachPeerConnection'] = (callback) => {
        connection['peerConnections'].forEach(callback);
      };

      const mockTrack = { id: 'track-1' } as MediaStreamTrack;
      const mockStream = { id: 'stream-1' } as MediaStream;

      const count = connection.addTrackToPeers(mockTrack, mockStream);
      expect(count).toBe(0); // Should handle error gracefully
    });
  });
});
