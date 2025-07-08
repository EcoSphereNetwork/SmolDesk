// src/components/ConnectionManager.tsx

import React, { useState, useEffect, useCallback } from 'react';
import { WebRTCConnection, WebRTCConnectionEvent } from '../utils/webrtc';

export interface ConnectionManagerProps {
  onConnected?: (peerId: string) => void;
  onDisconnected?: () => void;
  onStream?: (stream: MediaStream) => void;
  onError?: (error: Error) => void;
  signalingServer: string;
  autoConnect?: boolean;
}

const ConnectionManager: React.FC<ConnectionManagerProps> = ({
  onConnected,
  onDisconnected,
  onStream,
  onError,
  signalingServer,
  autoConnect = false,
}) => {
  const [connection, setConnection] = useState<WebRTCConnection | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<string>('disconnected');
  const [roomId, setRoomId] = useState<string>('');
  const [isHost, setIsHost] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [connectedPeers, setConnectedPeers] = useState<string[]>([]);

  // Initialize WebRTC connection
  useEffect(() => {
    const webrtcConnection = new WebRTCConnection({
      signalingServer,
      autoReconnect: true,
      peerConnectionConfig: {
        iceServers: [
          { urls: 'stun:stun.l.google.com:19302' },
          { urls: 'stun:stun1.l.google.com:19302' },
          {
            urls: 'turn:turn.smoldesk.example:3478',
            username: 'smoldesk',
            credential: 'smoldesk123',
          },
        ],
      },
    });

    // Set up event listeners
    webrtcConnection.on(WebRTCConnectionEvent.SIGNALING_CONNECTED, () => {
      setConnectionStatus('signaling-connected');
      setError(null);
      
      if (autoConnect && roomId) {
        if (isHost) {
          webrtcConnection.createRoom(roomId);
        } else {
          webrtcConnection.joinRoom(roomId);
        }
      }
    });

    webrtcConnection.on(WebRTCConnectionEvent.SIGNALING_DISCONNECTED, () => {
      setConnectionStatus('signaling-disconnected');
    });

    webrtcConnection.on(WebRTCConnectionEvent.CONNECTED, (event: { clientId: string }) => {
      setConnectionStatus('connected');
      setError(null);
    });

    webrtcConnection.on(WebRTCConnectionEvent.DISCONNECTED, () => {
      setConnectionStatus('disconnected');
      setConnectedPeers([]);
      if (onDisconnected) {
        onDisconnected();
      }
    });

    webrtcConnection.on(WebRTCConnectionEvent.ERROR, (error: Error) => {
      setError(error.message);
      if (onError) {
        onError(error);
      }
    });

    webrtcConnection.on(WebRTCConnectionEvent.ROOM_CREATED, (event: { roomId: string }) => {
      setConnectionStatus('room-created');
      setRoomId(event.roomId);
    });

    webrtcConnection.on(WebRTCConnectionEvent.ROOM_JOINED, (event: { roomId: string, peers: string[] }) => {
      setConnectionStatus('room-joined');
      setRoomId(event.roomId);
      setConnectedPeers(event.peers);
    });

    webrtcConnection.on(WebRTCConnectionEvent.PEER_JOINED, (event: { peerId: string }) => {
      setConnectedPeers(prev => [...prev, event.peerId]);
      if (onConnected) {
        onConnected(event.peerId);
      }
    });

    webrtcConnection.on(WebRTCConnectionEvent.PEER_LEFT, (event: { peerId: string }) => {
      setConnectedPeers(prev => prev.filter(id => id !== event.peerId));
    });

    webrtcConnection.on(WebRTCConnectionEvent.STREAM_ADDED, (event: { peerId: string, stream: MediaStream }) => {
      if (onStream) {
        onStream(event.stream);
      }
    });

    setConnection(webrtcConnection);

    // Connect to signaling server
    if (autoConnect) {
      webrtcConnection.connect();
    }

    // Cleanup on unmount
    return () => {
      webrtcConnection.disconnect();
    };
  }, [signalingServer, autoConnect, onConnected, onDisconnected, onStream, onError, roomId, isHost]);

  // Connect to signaling server
  const connect = useCallback(() => {
    if (connection) {
      connection.connect();
    }
  }, [connection]);

  // Disconnect from signaling server
  const disconnect = useCallback(() => {
    if (connection) {
      connection.disconnect();
    }
  }, [connection]);

  // Create a new room
  const createRoom = useCallback((customRoomId?: string) => {
    if (connection) {
      setIsHost(true);
      connection.createRoom(customRoomId);
    }
  }, [connection]);

  // Join an existing room
  const joinRoom = useCallback((roomIdToJoin: string) => {
    if (connection) {
      setIsHost(false);
      setRoomId(roomIdToJoin);
      connection.joinRoom(roomIdToJoin);
    }
  }, [connection]);

  // Leave the current room
  const leaveRoom = useCallback(() => {
    if (connection) {
      connection.leaveRoom();
      setRoomId('');
      setConnectedPeers([]);
    }
  }, [connection]);

  // Get local screen stream
  const getLocalStream = useCallback(async () => {
    if (connection) {
      try {
        const stream = await connection.getLocalStream({
          audio: false, // No audio by default
          video: true, // Get screen video
        });
        return stream;
      } catch (error) {
        setError(`Failed to get local stream: ${(error as Error).message}`);
        throw error;
      }
    }
    throw new Error('WebRTC connection not initialized');
  }, [connection]);

  // Send data to a specific peer
  const sendData = useCallback((peerId: string, data: any): boolean => {
    if (connection) {
      return connection.sendData(peerId, data);
    }
    return false;
  }, [connection]);

  // Broadcast data to all connected peers
  const broadcast = useCallback((data: any): number => {
    if (connection) {
      return connection.broadcast(data);
    }
    return 0;
  }, [connection]);

  // Generate a random room ID (for convenience)
  const generateRandomRoomId = useCallback(() => {
    const randomId = Math.random().toString(36).substring(2, 10);
    setRoomId(randomId);
    return randomId;
  }, []);

  return (
    <div className="connection-manager">
      <div className="connection-status">
        <h3>Connection Status: <span className={`status-${connectionStatus}`}>{connectionStatus}</span></h3>
        {error && <div className="error-message">Error: {error}</div>}
      </div>

      <div className="connection-controls">
        {connectionStatus === 'disconnected' || connectionStatus === 'signaling-disconnected' ? (
          <button onClick={connect} className="connect-button">
            Connect to Server
          </button>
        ) : (
          <button onClick={disconnect} className="disconnect-button">
            Disconnect
          </button>
        )}

        {connectionStatus === 'signaling-connected' && (
          <div className="room-controls">
            <div className="create-room">
              <input
                type="text"
                value={roomId}
                onChange={(e) => setRoomId(e.target.value)}
                placeholder="Room ID (optional)"
                className="room-id-input"
              />
              <button onClick={() => createRoom(roomId)} className="create-room-button">
                Create Room
              </button>
              <button onClick={() => createRoom(generateRandomRoomId())} className="random-room-button">
                Random Room
              </button>
            </div>

            <div className="join-room">
              <input
                type="text"
                value={roomId}
                onChange={(e) => setRoomId(e.target.value)}
                placeholder="Room ID to join"
                className="room-id-input"
              />
              <button onClick={() => joinRoom(roomId)} className="join-room-button">
                Join Room
              </button>
            </div>
          </div>
        )}

        {(connectionStatus === 'room-created' || connectionStatus === 'room-joined') && (
          <div className="active-room">
            <div className="room-info">
              <strong>Room ID:</strong> {roomId} 
              <button onClick={() => {
                navigator.clipboard.writeText(roomId);
              }} className="copy-button">
                Copy
              </button>
            </div>
            <div className="connected-peers">
              <strong>Connected Peers:</strong> {connectedPeers.length === 0 ? 'None' : connectedPeers.join(', ')}
            </div>
            <button onClick={leaveRoom} className="leave-room-button">
              Leave Room
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default ConnectionManager;
