// src/utils/webrtc.ts

export interface PeerConnectionConfig {
  iceServers: RTCIceServer[];
}

export interface WebRTCConnectionOptions {
  signalingServer: string;
  peerConnectionConfig?: PeerConnectionConfig;
  autoReconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

export interface StreamOptions {
  audio: boolean;
  video: boolean | MediaTrackConstraints;
}

type SignalingMessage = {
  type: string;
  [key: string]: any;
};

/**
 * Events emitted by WebRTCConnection
 */
export enum WebRTCConnectionEvent {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error',
  SIGNALING_CONNECTED = 'signaling-connected',
  SIGNALING_DISCONNECTED = 'signaling-disconnected',
  STREAM_ADDED = 'stream-added',
  STREAM_REMOVED = 'stream-removed',
  DATA_CHANNEL_OPEN = 'data-channel-open',
  DATA_CHANNEL_CLOSE = 'data-channel-close',
  DATA_CHANNEL_MESSAGE = 'data-channel-message',
  ICE_CANDIDATE = 'ice-candidate',
  ICE_CONNECTION_STATE_CHANGE = 'ice-connection-state-change',
  SIGNALING_STATE_CHANGE = 'signaling-state-change',
  PEER_CONNECTION_STATE_CHANGE = 'peer-connection-state-change',
  ROOM_CREATED = 'room-created',
  ROOM_JOINED = 'room-joined',
  ROOM_LEFT = 'room-left',
  PEER_JOINED = 'peer-joined',
  PEER_LEFT = 'peer-left',
}

/**
 * Manages WebRTC connections and signaling
 */
export class WebRTCConnection {
  private signalingServer: string;
  private peerConnectionConfig: PeerConnectionConfig;
  private signalingSocket: WebSocket | null = null;
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private dataChannels: Map<string, RTCDataChannel> = new Map();
  private clientId: string | null = null;
  private token: string | null = null;
  private roomId: string | null = null;
  private localStream: MediaStream | null = null;
  private eventListeners: Map<string, Set<Function>> = new Map();
  private reconnectTimer: number | null = null;
  private reconnectAttempts = 0;
  private autoReconnect: boolean;
  private reconnectInterval: number;
  private maxReconnectAttempts: number;
  private isReconnecting = false;
  private pingInterval: number | null = null;

  /**
   * Create a new WebRTCConnection
   */
  constructor(options: WebRTCConnectionOptions) {
    this.signalingServer = options.signalingServer;
    this.peerConnectionConfig = options.peerConnectionConfig || {
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' },
      ],
    };
    this.autoReconnect = options.autoReconnect ?? true;
    this.reconnectInterval = options.reconnectInterval ?? 5000;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5;
  }

  /**
   * Connect to the signaling server
   */
  public connect(): void {
    if (this.signalingSocket && this.signalingSocket.readyState === WebSocket.OPEN) {
      console.warn('Already connected to signaling server');
      return;
    }

    this.emit(WebRTCConnectionEvent.CONNECTING);

    try {
      this.signalingSocket = new WebSocket(this.signalingServer);

      this.signalingSocket.onopen = () => {
        this.isReconnecting = false;
        this.reconnectAttempts = 0;
        this.emit(WebRTCConnectionEvent.SIGNALING_CONNECTED);
        
        // Set up ping interval to keep connection alive
        this.pingInterval = window.setInterval(() => {
          if (this.signalingSocket && this.signalingSocket.readyState === WebSocket.OPEN) {
            this.sendSignalingMessage({ type: 'ping' });
          }
        }, 30000);
      };

      this.signalingSocket.onclose = () => {
        this.emit(WebRTCConnectionEvent.SIGNALING_DISCONNECTED);
        
        if (this.pingInterval) {
          clearInterval(this.pingInterval);
          this.pingInterval = null;
        }

        if (this.autoReconnect && !this.isReconnecting) {
          this.handleReconnect();
        }
      };

      this.signalingSocket.onerror = (error) => {
        this.emit(WebRTCConnectionEvent.ERROR, error);
      };

      this.signalingSocket.onmessage = (event) => {
        this.handleSignalingMessage(event.data);
      };
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
      if (this.autoReconnect) {
        this.handleReconnect();
      }
    }
  }

  /**
   * Disconnect from the signaling server and close all peer connections
   */
  public disconnect(): void {
    // Clear any reconnect timers
    if (this.reconnectTimer) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }

    // Leave current room
    if (this.roomId) {
      this.leaveRoom();
    }

    // Close all peer connections
    this.peerConnections.forEach((pc, peerId) => {
      this.closePeerConnection(peerId);
    });

    // Close signaling connection
    if (this.signalingSocket) {
      this.signalingSocket.close();
      this.signalingSocket = null;
    }

    this.clientId = null;
    this.token = null;
    
    this.emit(WebRTCConnectionEvent.DISCONNECTED);
  }

  /**
   * Create a new room on the signaling server
   */
  public createRoom(roomId?: string, settings?: any): void {
    if (!this.signalingSocket || this.signalingSocket.readyState !== WebSocket.OPEN) {
      this.emit(WebRTCConnectionEvent.ERROR, new Error('Not connected to signaling server'));
      return;
    }

    this.sendSignalingMessage({
      type: 'create-room',
      roomId,
      settings
    });
  }

  /**
   * Join an existing room
   */
  public joinRoom(roomId: string): void {
    if (!this.signalingSocket || this.signalingSocket.readyState !== WebSocket.OPEN) {
      this.emit(WebRTCConnectionEvent.ERROR, new Error('Not connected to signaling server'));
      return;
    }

    this.sendSignalingMessage({
      type: 'join-room',
      roomId
    });
  }

  /**
   * Leave the current room
   */
  public leaveRoom(): void {
    if (!this.roomId) {
      return;
    }

    this.sendSignalingMessage({
      type: 'leave-room'
    });

    // Close all peer connections
    this.peerConnections.forEach((pc, peerId) => {
      this.closePeerConnection(peerId);
    });

    this.roomId = null;
  }

  /**
   * Get local media stream (screen sharing)
   */
  public async getLocalStream(options: StreamOptions = { audio: false, video: true }): Promise<MediaStream> {
    try {
      // For screen sharing, we use getDisplayMedia instead of getUserMedia
      const stream = await navigator.mediaDevices.getDisplayMedia({
        audio: options.audio,
        video: options.video || true
      });

      this.localStream = stream;

      // Add tracks to existing peer connections
      if (this.peerConnections.size > 0) {
        this.peerConnections.forEach((pc) => {
          stream.getTracks().forEach(track => {
            pc.addTrack(track, stream);
          });
        });
      }

      return stream;
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
      throw error;
    }
  }

  /**
   * Send data to a specific peer via data channel
   */
  public sendData(peerId: string, data: any): boolean {
    const dataChannel = this.dataChannels.get(peerId);
    
    if (!dataChannel || dataChannel.readyState !== 'open') {
      return false;
    }

    try {
      // If data is not a string, stringify it
      const message = typeof data === 'string' ? data : JSON.stringify(data);
      dataChannel.send(message);
      return true;
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
      return false;
    }
  }

  /**
   * Broadcast data to all connected peers
   */
  public broadcast(data: any): number {
    let successCount = 0;
    
    this.dataChannels.forEach((dataChannel, peerId) => {
      if (this.sendData(peerId, data)) {
        successCount++;
      }
    });
    
    return successCount;
  }

  /**
   * Add event listener
   */
  public on(event: WebRTCConnectionEvent, callback: Function): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    
    this.eventListeners.get(event)!.add(callback);
  }

  /**
   * Remove event listener
   */
  public off(event: WebRTCConnectionEvent, callback: Function): void {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event)!.delete(callback);
    }
  }

  /**
   * Emit event to all listeners
   */
  private emit(event: WebRTCConnectionEvent, ...args: any[]): void {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event)!.forEach(callback => {
        try {
          callback(...args);
        } catch (error) {
          console.error(`Error in event listener for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Handle reconnecting to the signaling server
   */
  private handleReconnect(): void {
    if (this.isReconnecting || this.reconnectAttempts >= this.maxReconnectAttempts) {
      return;
    }

    this.isReconnecting = true;
    this.reconnectAttempts++;
    
    this.emit(WebRTCConnectionEvent.RECONNECTING, {
      attempt: this.reconnectAttempts,
      maxAttempts: this.maxReconnectAttempts
    });

    this.reconnectTimer = window.setTimeout(() => {
      this.connect();
    }, this.reconnectInterval);
  }

  /**
   * Send message to the signaling server
   */
  private sendSignalingMessage(message: SignalingMessage): void {
    if (!this.signalingSocket || this.signalingSocket.readyState !== WebSocket.OPEN) {
      this.emit(WebRTCConnectionEvent.ERROR, new Error('Not connected to signaling server'));
      return;
    }

    try {
      this.signalingSocket.send(JSON.stringify(message));
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Handle incoming message from the signaling server
   */
  private handleSignalingMessage(data: string): void {
    try {
      const message = JSON.parse(data);

      switch (message.type) {
        case 'welcome':
          this.handleWelcome(message);
          break;
        
        case 'room-created':
          this.handleRoomCreated(message);
          break;
        
        case 'room-joined':
          this.handleRoomJoined(message);
          break;
        
        case 'room-left':
          this.handleRoomLeft(message);
          break;
        
        case 'peer-joined':
          this.handlePeerJoined(message);
          break;
        
        case 'peer-left':
          this.handlePeerLeft(message);
          break;
        
        case 'peer-disconnected':
          this.handlePeerDisconnected(message);
          break;
        
        case 'offer':
          this.handleOffer(message);
          break;
        
        case 'answer':
          this.handleAnswer(message);
          break;
        
        case 'ice-candidate':
          this.handleIceCandidate(message);
          break;
        
        case 'error':
          this.emit(WebRTCConnectionEvent.ERROR, new Error(message.message));
          break;
        
        case 'pong':
          // Ignore pong responses
          break;
        
        default:
          console.warn('Unknown message type:', message.type);
      }
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Handle welcome message from signaling server
   */
  private handleWelcome(message: any): void {
    this.clientId = message.clientId;
    this.token = message.token;
    this.emit(WebRTCConnectionEvent.CONNECTED, { clientId: this.clientId });
  }

  /**
   * Handle room created message
   */
  private handleRoomCreated(message: any): void {
    this.roomId = message.roomId;
    this.emit(WebRTCConnectionEvent.ROOM_CREATED, { roomId: this.roomId });
  }

  /**
   * Handle room joined message
   */
  private handleRoomJoined(message: any): void {
    this.roomId = message.roomId;
    
    // Create peer connections for existing peers in the room
    if (message.peers && Array.isArray(message.peers)) {
      message.peers.forEach((peerId: string) => {
        this.createPeerConnection(peerId, true);  // Create connection and send offer
      });
    }
    
    this.emit(WebRTCConnectionEvent.ROOM_JOINED, {
      roomId: this.roomId,
      peers: message.peers || [],
      settings: message.settings
    });
  }

  /**
   * Handle room left message
   */
  private handleRoomLeft(message: any): void {
    const oldRoomId = this.roomId;
    this.roomId = null;
    this.emit(WebRTCConnectionEvent.ROOM_LEFT, { roomId: oldRoomId });
  }

  /**
   * Handle peer joined message
   */
  private handlePeerJoined(message: any): void {
    const peerId = message.peerId;
    
    // Create a new peer connection (but don't send offer - wait for the peer to do it)
    this.createPeerConnection(peerId, false);
    
    this.emit(WebRTCConnectionEvent.PEER_JOINED, { peerId });
  }

  /**
   * Handle peer left message
   */
  private handlePeerLeft(message: any): void {
    const peerId = message.peerId;
    
    this.closePeerConnection(peerId);
    
    this.emit(WebRTCConnectionEvent.PEER_LEFT, { peerId });
  }

  /**
   * Handle peer disconnected message
   */
  private handlePeerDisconnected(message: any): void {
    this.handlePeerLeft(message);
  }

  /**
   * Handle offer from remote peer
   */
  private async handleOffer(message: any): Promise<void> {
    const peerId = message.peerId;
    const offer = message.offer;
    
    // Create peer connection if it doesn't exist
    if (!this.peerConnections.has(peerId)) {
      this.createPeerConnection(peerId, false);
    }
    
    const peerConnection = this.peerConnections.get(peerId)!;
    
    try {
      await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
      
      const answer = await peerConnection.createAnswer();
      await peerConnection.setLocalDescription(answer);
      
      this.sendSignalingMessage({
        type: 'answer',
        targetId: peerId,
        answer: peerConnection.localDescription
      });
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Handle answer from remote peer
   */
  private async handleAnswer(message: any): Promise<void> {
    const peerId = message.peerId;
    const answer = message.answer;
    
    if (!this.peerConnections.has(peerId)) {
      console.warn(`Received answer from unknown peer: ${peerId}`);
      return;
    }
    
    const peerConnection = this.peerConnections.get(peerId)!;
    
    try {
      await peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Handle ICE candidate from remote peer
   */
  private handleIceCandidate(message: any): void {
    const peerId = message.peerId;
    const candidate = message.candidate;
    
    if (!this.peerConnections.has(peerId)) {
      console.warn(`Received ICE candidate from unknown peer: ${peerId}`);
      return;
    }
    
    const peerConnection = this.peerConnections.get(peerId)!;
    
    try {
      peerConnection.addIceCandidate(new RTCIceCandidate(candidate));
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Create a new peer connection
   */
  private createPeerConnection(peerId: string, initiator: boolean): RTCPeerConnection {
    if (this.peerConnections.has(peerId)) {
      this.closePeerConnection(peerId);
    }
    
    const peerConnection = new RTCPeerConnection(this.peerConnectionConfig);
    this.peerConnections.set(peerId, peerConnection);
    
    // Create data channel if initiator
    if (initiator) {
      const dataChannel = peerConnection.createDataChannel('data', {
        ordered: true,
      });
      
      this.setupDataChannel(peerId, dataChannel);
    } else {
      // Otherwise listen for data channel
      peerConnection.ondatachannel = (event) => {
        this.setupDataChannel(peerId, event.channel);
      };
    }
    
    // Add local stream tracks to the connection
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => {
        peerConnection.addTrack(track, this.localStream!);
      });
    }
    
    // Handle incoming tracks
    peerConnection.ontrack = (event) => {
      this.emit(WebRTCConnectionEvent.STREAM_ADDED, {
        peerId,
        stream: event.streams[0]
      });
    };
    
    // Handle ICE candidates
    peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        this.emit(WebRTCConnectionEvent.ICE_CANDIDATE, {
          peerId,
          candidate: event.candidate
        });
        
        this.sendSignalingMessage({
          type: 'ice-candidate',
          targetId: peerId,
          candidate: event.candidate
        });
      }
    };
    
    // Monitor connection state
    peerConnection.oniceconnectionstatechange = () => {
      this.emit(WebRTCConnectionEvent.ICE_CONNECTION_STATE_CHANGE, {
        peerId,
        state: peerConnection.iceConnectionState
      });
      
      // Handle disconnection
      if (
        peerConnection.iceConnectionState === 'disconnected' ||
        peerConnection.iceConnectionState === 'failed' ||
        peerConnection.iceConnectionState === 'closed'
      ) {
        this.emit(WebRTCConnectionEvent.PEER_LEFT, { peerId });
      }
    };
    
    peerConnection.onsignalingstatechange = () => {
      this.emit(WebRTCConnectionEvent.SIGNALING_STATE_CHANGE, {
        peerId,
        state: peerConnection.signalingState
      });
    };
    
    peerConnection.onconnectionstatechange = () => {
      this.emit(WebRTCConnectionEvent.PEER_CONNECTION_STATE_CHANGE, {
        peerId,
        state: peerConnection.connectionState
      });
    };
    
    // Create and send offer if initiator
    if (initiator) {
      this.createAndSendOffer(peerId, peerConnection);
    }
    
    return peerConnection;
  }

  /**
   * Close a peer connection
   */
  private closePeerConnection(peerId: string): void {
    // Close data channel
    if (this.dataChannels.has(peerId)) {
      const dataChannel = this.dataChannels.get(peerId)!;
      dataChannel.close();
      this.dataChannels.delete(peerId);
    }
    
    // Close peer connection
    if (this.peerConnections.has(peerId)) {
      const peerConnection = this.peerConnections.get(peerId)!;
      peerConnection.close();
      this.peerConnections.delete(peerId);
      
      this.emit(WebRTCConnectionEvent.STREAM_REMOVED, { peerId });
    }
  }

  /**
   * Create and send offer to remote peer
   */
  private async createAndSendOffer(peerId: string, peerConnection: RTCPeerConnection): Promise<void> {
    try {
      const offer = await peerConnection.createOffer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: true
      });
      
      await peerConnection.setLocalDescription(offer);
      
      this.sendSignalingMessage({
        type: 'offer',
        targetId: peerId,
        offer: peerConnection.localDescription
      });
    } catch (error) {
      this.emit(WebRTCConnectionEvent.ERROR, error);
    }
  }

  /**
   * Set up data channel
   */
  private setupDataChannel(peerId: string, dataChannel: RTCDataChannel): void {
    this.dataChannels.set(peerId, dataChannel);
    
    dataChannel.onopen = () => {
      this.emit(WebRTCConnectionEvent.DATA_CHANNEL_OPEN, { peerId });
    };
    
    dataChannel.onclose = () => {
      this.emit(WebRTCConnectionEvent.DATA_CHANNEL_CLOSE, { peerId });
    };
    
    dataChannel.onmessage = (event) => {
      let data = event.data;
      
      // Try to parse JSON data
      try {
        data = JSON.parse(event.data);
      } catch {
        // Keep as is if not JSON
      }
      
      this.emit(WebRTCConnectionEvent.DATA_CHANNEL_MESSAGE, {
        peerId,
        data
      });
    };
  }
}
