// src/utils/enhancedWebRTC.ts

import { WebRTCConnection, WebRTCConnectionEvent, PeerConnectionConfig, WebRTCConnectionOptions } from './webrtc';

/**
 * Extended configuration for EnhancedWebRTCConnection
 */
export interface EnhancedWebRTCOptions extends WebRTCConnectionOptions {
  // ICE gathering timeout in milliseconds
  iceGatheringTimeout?: number;
  
  // Prioritized TURN servers to try in order
  prioritizedTurnServers?: RTCIceServer[];
  
  // Max reconnection attempts for lost connections
  maxICERestarts?: number;
  
  // Bandwidth allocation preferences
  bandwidthConstraints?: {
    video?: number;  // kbps
    audio?: number;  // kbps
    screen?: number; // kbps
  };
  
  // Whether to enable trickle ICE (recommended)
  enableTrickleICE?: boolean;
}

/**
 * Connection status with enhanced monitoring
 */
export enum ConnectionQuality {
  EXCELLENT = 'excellent',
  GOOD = 'good',
  FAIR = 'fair',
  POOR = 'poor',
  CRITICAL = 'critical',
  DISCONNECTED = 'disconnected',
}

/**
 * Enhanced WebRTC connection with improved ICE handling and NAT traversal
 */
export class EnhancedWebRTCConnection extends WebRTCConnection {
  private iceGatheringTimeout: number;
  private prioritizedTurnServers: RTCIceServer[];
  private maxICERestarts: number;
  private bandwidthConstraints?: {
    video?: number;
    audio?: number;
    screen?: number;
  };
  private enableTrickleICE: boolean;
  
  // Connection monitoring
  private iceRestartCounts: Map<string, number> = new Map();
  private connectionQualities: Map<string, ConnectionQuality> = new Map();
  private statsIntervals: Map<string, number> = new Map();
  
  /**
   * Create a new enhanced WebRTC connection
   */
  constructor(options: EnhancedWebRTCOptions) {
    super(options);
    
    // Set up enhanced options with defaults
    this.iceGatheringTimeout = options.iceGatheringTimeout || 10000; // 10 seconds
    this.prioritizedTurnServers = options.prioritizedTurnServers || [];
    this.maxICERestarts = options.maxICERestarts || 5;
    this.bandwidthConstraints = options.bandwidthConstraints;
    this.enableTrickleICE = options.enableTrickleICE !== false; // Default to true
    
    // Set up enhanced event listeners
    this.setupEnhancedEventListeners();
  }
  
  /**
   * Set up enhanced event listeners for better ICE handling
   */
  private setupEnhancedEventListeners(): void {
    // Listen for ICE connection state changes
    this.on(WebRTCConnectionEvent.ICE_CONNECTION_STATE_CHANGE, this.handleICEConnectionStateChange.bind(this));
    
    // Listen for peer connection state changes
    this.on(WebRTCConnectionEvent.PEER_CONNECTION_STATE_CHANGE, this.handlePeerConnectionStateChange.bind(this));
    
    // Listen for new peer connections
    this.on(WebRTCConnectionEvent.PEER_JOINED, (event) => {
      const { peerId } = event;
      this.iceRestartCounts.set(peerId, 0);
      this.connectionQualities.set(peerId, ConnectionQuality.GOOD);
      
      // Start monitoring stats for this peer
      this.startStatsMonitoring(peerId);
    });
    
    // Listen for peer disconnections
    this.on(WebRTCConnectionEvent.PEER_LEFT, (event) => {
      const { peerId } = event;
      this.iceRestartCounts.delete(peerId);
      this.connectionQualities.delete(peerId);
      
      // Stop stats monitoring for this peer
      this.stopStatsMonitoring(peerId);
    });
  }
  
  /**
   * Handle ICE connection state changes
   */
  private handleICEConnectionStateChange(event: { peerId: string, state: RTCIceConnectionState }): void {
    const { peerId, state } = event;
    
    switch (state) {
      case 'checking':
        console.log(`ICE checking for peer ${peerId}`);
        break;
        
      case 'connected':
        console.log(`ICE connected for peer ${peerId}`);
        // Reset ICE restart count on successful connection
        this.iceRestartCounts.set(peerId, 0);
        break;
        
      case 'completed':
        console.log(`ICE completed for peer ${peerId}`);
        break;
        
      case 'failed':
        console.warn(`ICE failed for peer ${peerId}`);
        this.handleICEFailure(peerId);
        break;
        
      case 'disconnected':
        console.warn(`ICE disconnected for peer ${peerId}`);
        // Give some time for auto-recovery before intervening
        setTimeout(() => {
          const peerConnection = this.getPeerConnection(peerId);
          if (peerConnection && peerConnection.iceConnectionState === 'disconnected') {
            this.handleICEFailure(peerId);
          }
        }, 5000);
        break;
        
      case 'closed':
        console.log(`ICE closed for peer ${peerId}`);
        break;
    }
  }
  
  /**
   * Handle peer connection state changes
   */
  private handlePeerConnectionStateChange(event: { peerId: string, state: RTCPeerConnectionState }): void {
    const { peerId, state } = event;
    
    switch (state) {
      case 'new':
        console.log(`Peer connection new for ${peerId}`);
        break;
        
      case 'connecting':
        console.log(`Peer connection connecting for ${peerId}`);
        break;
        
      case 'connected':
        console.log(`Peer connection connected for ${peerId}`);
        break;
        
      case 'disconnected':
        console.warn(`Peer connection disconnected for ${peerId}`);
        break;
        
      case 'failed':
        console.error(`Peer connection failed for ${peerId}`);
        // Try to restart the connection
        this.handleConnectionFailure(peerId);
        break;
        
      case 'closed':
        console.log(`Peer connection closed for ${peerId}`);
        break;
    }
  }
  
  /**
   * Handle ICE failures by attempting ICE restarts
   */
  private async handleICEFailure(peerId: string): Promise<void> {
    const currentRestarts = this.iceRestartCounts.get(peerId) || 0;
    
    if (currentRestarts >= this.maxICERestarts) {
      console.error(`Maximum ICE restarts reached for peer ${peerId}`);
      this.emit('connection-quality-change', {
        peerId,
        quality: ConnectionQuality.DISCONNECTED,
        reason: 'Maximum ICE restarts reached'
      });
      return;
    }
    
    console.log(`Attempting ICE restart for peer ${peerId} (attempt ${currentRestarts + 1}/${this.maxICERestarts})`);
    this.iceRestartCounts.set(peerId, currentRestarts + 1);
    
    // Get the peer connection
    const peerConnection = this.getPeerConnection(peerId);
    if (!peerConnection) {
      console.error(`Peer connection for ${peerId} not found`);
      return;
    }
    
    try {
      // Create a new offer with ICE restart flag
      const offer = await peerConnection.createOffer({ 
        iceRestart: true,
        offerToReceiveAudio: true,
        offerToReceiveVideo: true
      });
      
      // Set the local description
      await peerConnection.setLocalDescription(offer);
      
      // Send the offer to the peer
      this.sendSignalingMessage({
        type: 'offer',
        targetId: peerId,
        offer: peerConnection.localDescription
      });
      
      console.log(`ICE restart offer sent to peer ${peerId}`);
    } catch (error) {
      console.error(`Failed to restart ICE for peer ${peerId}:`, error);
    }
  }
  
  /**
   * Handle connection failures by attempting to recreate the connection
   */
  private handleConnectionFailure(peerId: string): void {
    console.log(`Attempting to recreate connection for peer ${peerId}`);
    
    // Close the existing connection
    this.closePeerConnection(peerId);
    
    // Create a new connection and send offer
    this.createPeerConnection(peerId, true);
    
    console.log(`Connection recreated for peer ${peerId}`);
  }
  
  /**
   * Start monitoring connection stats for a peer
   */
  private startStatsMonitoring(peerId: string): void {
    // Stop any existing monitoring
    this.stopStatsMonitoring(peerId);
    
    // Set up a new interval for stats monitoring
    const intervalId = window.setInterval(async () => {
      const peerConnection = this.getPeerConnection(peerId);
      if (!peerConnection) {
        this.stopStatsMonitoring(peerId);
        return;
      }
      
      try {
        // Get RTCStats from the peer connection
        const stats = await peerConnection.getStats();
        
        // Process the stats to determine connection quality
        const quality = this.processConnectionStats(peerId, stats);
        
        // Emit an event if quality changed
        if (quality !== this.connectionQualities.get(peerId)) {
          this.connectionQualities.set(peerId, quality);
          this.emit('connection-quality-change', {
            peerId,
            quality,
            stats
          });
        }
      } catch (error) {
        console.error(`Error getting stats for peer ${peerId}:`, error);
      }
    }, 2000); // Check every 2 seconds
    
    this.statsIntervals.set(peerId, intervalId);
  }
  
  /**
   * Stop monitoring connection stats for a peer
   */
  private stopStatsMonitoring(peerId: string): void {
    const intervalId = this.statsIntervals.get(peerId);
    if (intervalId) {
      clearInterval(intervalId);
      this.statsIntervals.delete(peerId);
    }
  }
  
  /**
   * Process connection stats to determine quality
   */
  private processConnectionStats(peerId: string, stats: RTCStatsReport): ConnectionQuality {
    let packetsLost = 0;
    let packetsReceived = 0;
    let bytesReceived = 0;
    let jitter = 0;
    let rtt = 0;
    let framesDecoded = 0;
    let framesDropped = 0;
    
    // Process stats entries
    stats.forEach(stat => {
      if (stat.type === 'inbound-rtp' && stat.mediaType === 'video') {
        packetsLost = stat.packetsLost || 0;
        packetsReceived = stat.packetsReceived || 0;
        bytesReceived = stat.bytesReceived || 0;
        jitter = stat.jitter || 0;
        framesDecoded = stat.framesDecoded || 0;
        framesDropped = stat.framesDropped || 0;
      }
      
      if (stat.type === 'remote-inbound-rtp') {
        rtt = stat.roundTripTime || 0;
      }
    });
    
    // Calculate packet loss ratio
    const totalPackets = packetsReceived + packetsLost;
    const packetLossRatio = totalPackets > 0 ? packetsLost / totalPackets : 0;
    
    // Calculate frame drop ratio
    const totalFrames = framesDecoded + framesDropped;
    const frameDropRatio = totalFrames > 0 ? framesDropped / totalFrames : 0;
    
    // Determine quality based on metrics
    if (packetLossRatio > 0.15 || rtt > 500 || frameDropRatio > 0.3) {
      return ConnectionQuality.CRITICAL;
    } else if (packetLossRatio > 0.1 || rtt > 300 || frameDropRatio > 0.2) {
      return ConnectionQuality.POOR;
    } else if (packetLossRatio > 0.05 || rtt > 200 || frameDropRatio > 0.1) {
      return ConnectionQuality.FAIR;
    } else if (packetLossRatio > 0.01 || rtt > 100 || frameDropRatio > 0.05) {
      return ConnectionQuality.GOOD;
    } else {
      return ConnectionQuality.EXCELLENT;
    }
  }
  
  /**
   * Create a peer connection with enhanced ICE handling
   */
  protected createPeerConnection(peerId: string, initiator: boolean): RTCPeerConnection {
    // Start with the base implementation
    const peerConnection = super.createPeerConnection(peerId, initiator);
    
    // Apply bandwidth constraints if specified
    if (this.bandwidthConstraints) {
      this.applyBandwidthConstraints(peerConnection);
    }
    
    // Return the enhanced peer connection
    return peerConnection;
  }
  
  /**
   * Apply bandwidth constraints to the peer connection
   */
  private applyBandwidthConstraints(peerConnection: RTCPeerConnection): void {
    if (!this.bandwidthConstraints) return;
    
    // Add event listener for SDP creation to modify the bandwidth
    const originalSetLocalDescription = peerConnection.setLocalDescription.bind(peerConnection);
    peerConnection.setLocalDescription = async (description?: RTCSessionDescriptionInit) => {
      if (!description) {
        return originalSetLocalDescription();
      }
      
      // Modify SDP to include bandwidth constraints
      const sdp = this.modifySdpForBandwidth(description.sdp || '');
      const modifiedDescription = {
        ...description,
        sdp
      };
      
      return originalSetLocalDescription(modifiedDescription);
    };
  }
  
  /**
   * Modify SDP to include bandwidth constraints
   */
  private modifySdpForBandwidth(sdp: string): string {
    if (!this.bandwidthConstraints) return sdp;
    
    let modifiedSdp = sdp;
    
    // Add b=AS line for video
    if (this.bandwidthConstraints.video) {
      modifiedSdp = modifiedSdp.replace(
        /m=video .*\r\n/g,
        match => `${match}b=AS:${this.bandwidthConstraints?.video}\r\n`
      );
    }
    
    // Add b=AS line for audio
    if (this.bandwidthConstraints.audio) {
      modifiedSdp = modifiedSdp.replace(
        /m=audio .*\r\n/g,
        match => `${match}b=AS:${this.bandwidthConstraints?.audio}\r\n`
      );
    }
    
    return modifiedSdp;
  }
  
  /**
   * Add a track to all active peer connections
   */
  public addTrackToPeers(track: MediaStreamTrack, stream: MediaStream): number {
    let addedCount = 0;
    
    this.forEachPeerConnection((peerConnection, peerId) => {
      try {
        peerConnection.addTrack(track, stream);
        addedCount++;
      } catch (error) {
        console.error(`Failed to add track to peer ${peerId}:`, error);
      }
    });
    
    return addedCount;
  }
  
  /**
   * Override to expose the forEachPeerConnection method
   */
  protected forEachPeerConnection(callback: (peerConnection: RTCPeerConnection, peerId: string) => void): void {
    // This is a placeholder. The method should be overridden by subclasses
    // that have access to the peer connections.
    // This is here to fulfill the type system requirements
  }
  
  /**
   * Get a peer connection by ID
   */
  protected getPeerConnection(peerId: string): RTCPeerConnection | null {
    // This is a placeholder. The method should be overridden by subclasses
    // that have access to the peer connections.
    // This is here to fulfill the type system requirements
    return null;
  }
  
  /**
   * Configure TURN servers with fallback support
   */
  public configureTURNServers(servers: RTCIceServer[]): void {
    // Update prioritized TURN servers
    this.prioritizedTurnServers = servers;
    
    // Update all active peer connections with the new ICE servers
    this.forEachPeerConnection((peerConnection) => {
      // Add all servers to the configuration
      const currentConfig = peerConnection.getConfiguration();
      const newServers = [...(currentConfig.iceServers || []), ...servers];
      
      // Remove duplicates
      const uniqueServers = this.removeDuplicateIceServers(newServers);
      
      // Update the configuration
      peerConnection.setConfiguration({
        ...currentConfig,
        iceServers: uniqueServers
      });
    });
  }
  
  /**
   * Set bandwidth constraints
   */
  public setBandwidthConstraints(constraints: { video?: number, audio?: number, screen?: number }): void {
    this.bandwidthConstraints = constraints;
    
    // Apply to all existing connections
    this.forEachPeerConnection((peerConnection) => {
      this.applyBandwidthConstraints(peerConnection);
    });
  }
  
  /**
   * Remove duplicate ICE servers from configuration
   */
  private removeDuplicateIceServers(servers: RTCIceServer[]): RTCIceServer[] {
    const uniqueUrls = new Set<string>();
    const result: RTCIceServer[] = [];
    
    for (const server of servers) {
      const urls = Array.isArray(server.urls) ? server.urls : [server.urls];
      const filteredUrls = urls.filter(url => {
        if (uniqueUrls.has(url)) {
          return false;
        }
        uniqueUrls.add(url);
        return true;
      });
      
      if (filteredUrls.length > 0) {
        result.push({
          ...server,
          urls: filteredUrls
        });
      }
    }
    
    return result;
  }
  
  /**
   * Get the current connection quality for a peer
   */
  public getConnectionQuality(peerId: string): ConnectionQuality {
    return this.connectionQualities.get(peerId) || ConnectionQuality.DISCONNECTED;
  }
  
  /**
   * Recreate a peer connection
   */
  public recreateConnection(peerId: string): boolean {
    try {
      this.handleConnectionFailure(peerId);
      return true;
    } catch (error) {
      console.error(`Failed to recreate connection for peer ${peerId}:`, error);
      return false;
    }
  }
}
