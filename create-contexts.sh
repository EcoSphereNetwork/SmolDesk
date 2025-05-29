#!/bin/bash
# create-contexts.sh - Erstellt fehlende React Contexts

set -e

echo "🎭 Erstelle React Contexts..."

# Erstelle ConnectionContext
cat > src/contexts/ConnectionContext.tsx << 'EOF'
// src/contexts/ConnectionContext.tsx

import React, { createContext, useContext, useReducer, ReactNode } from 'react';

// Types
interface ConnectionState {
  isConnected: boolean;
  roomId: string | null;
  peerId: string | null;
  peers: string[];
  connectionQuality: 'excellent' | 'good' | 'fair' | 'poor' | 'disconnected';
  error: string | null;
}

type ConnectionAction = 
  | { type: 'CONNECT'; payload: { peerId: string } }
  | { type: 'DISCONNECT' }
  | { type: 'JOIN_ROOM'; payload: { roomId: string } }
  | { type: 'LEAVE_ROOM' }
  | { type: 'PEER_JOINED'; payload: { peerId: string } }
  | { type: 'PEER_LEFT'; payload: { peerId: string } }
  | { type: 'QUALITY_CHANGE'; payload: { quality: ConnectionState['connectionQuality'] } }
  | { type: 'ERROR'; payload: { error: string } }
  | { type: 'CLEAR_ERROR' };

interface ConnectionContextType {
  state: ConnectionState;
  dispatch: React.Dispatch<ConnectionAction>;
  connect: (peerId: string) => void;
  disconnect: () => void;
  joinRoom: (roomId: string) => void;
  leaveRoom: () => void;
  clearError: () => void;
}

// Initial state
const initialState: ConnectionState = {
  isConnected: false,
  roomId: null,
  peerId: null,
  peers: [],
  connectionQuality: 'disconnected',
  error: null,
};

// Reducer
function connectionReducer(state: ConnectionState, action: ConnectionAction): ConnectionState {
  switch (action.type) {
    case 'CONNECT':
      return {
        ...state,
        isConnected: true,
        peerId: action.payload.peerId,
        connectionQuality: 'good',
        error: null,
      };
    
    case 'DISCONNECT':
      return {
        ...state,
        isConnected: false,
        peerId: null,
        roomId: null,
        peers: [],
        connectionQuality: 'disconnected',
      };
    
    case 'JOIN_ROOM':
      return {
        ...state,
        roomId: action.payload.roomId,
      };
    
    case 'LEAVE_ROOM':
      return {
        ...state,
        roomId: null,
        peers: [],
      };
    
    case 'PEER_JOINED':
      return {
        ...state,
        peers: [...state.peers, action.payload.peerId],
      };
    
    case 'PEER_LEFT':
      return {
        ...state,
        peers: state.peers.filter(id => id !== action.payload.peerId),
      };
    
    case 'QUALITY_CHANGE':
      return {
        ...state,
        connectionQuality: action.payload.quality,
      };
    
    case 'ERROR':
      return {
        ...state,
        error: action.payload.error,
      };
    
    case 'CLEAR_ERROR':
      return {
        ...state,
        error: null,
      };
    
    default:
      return state;
  }
}

// Context
const ConnectionContext = createContext<ConnectionContextType | undefined>(undefined);

// Provider component
interface ConnectionProviderProps {
  children: ReactNode;
}

export function ConnectionProvider({ children }: ConnectionProviderProps) {
  const [state, dispatch] = useReducer(connectionReducer, initialState);
  
  const connect = (peerId: string) => {
    dispatch({ type: 'CONNECT', payload: { peerId } });
  };
  
  const disconnect = () => {
    dispatch({ type: 'DISCONNECT' });
  };
  
  const joinRoom = (roomId: string) => {
    dispatch({ type: 'JOIN_ROOM', payload: { roomId } });
  };
  
  const leaveRoom = () => {
    dispatch({ type: 'LEAVE_ROOM' });
  };
  
  const clearError = () => {
    dispatch({ type: 'CLEAR_ERROR' });
  };
  
  const value: ConnectionContextType = {
    state,
    dispatch,
    connect,
    disconnect,
    joinRoom,
    leaveRoom,
    clearError,
  };
  
  return (
    <ConnectionContext.Provider value={value}>
      {children}
    </ConnectionContext.Provider>
  );
}

// Custom hook
export function useConnection() {
  const context = useContext(ConnectionContext);
  if (context === undefined) {
    throw new Error('useConnection must be used within a ConnectionProvider');
  }
  return context;
}

export default ConnectionContext;
EOF

# Erstelle WebRTC Hook
cat > src/hooks/useWebRTC.ts << 'EOF'
// src/hooks/useWebRTC.ts

import { useState, useEffect, useCallback, useRef } from 'react';

interface WebRTCHookOptions {
  signalingServer: string;
  iceServers?: RTCIceServer[];
}

interface WebRTCHookReturn {
  isConnected: boolean;
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  createOffer: () => Promise<void>;
  createAnswer: (offer: RTCSessionDescriptionInit) => Promise<void>;
  addIceCandidate: (candidate: RTCIceCandidateInit) => Promise<void>;
}

export function useWebRTC(options: WebRTCHookOptions): WebRTCHookReturn {
  const [isConnected, setIsConnected] = useState(false);
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStream, setRemoteStream] = useState<MediaStream | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  const peerConnectionRef = useRef<RTCPeerConnection | null>(null);
  const webSocketRef = useRef<WebSocket | null>(null);
  
  const createPeerConnection = useCallback(() => {
    const peerConnection = new RTCPeerConnection({
      iceServers: options.iceServers || [
        { urls: 'stun:stun.l.google.com:19302' }
      ]
    });
    
    peerConnection.onicecandidate = (event) => {
      if (event.candidate && webSocketRef.current) {
        webSocketRef.current.send(JSON.stringify({
          type: 'ice-candidate',
          candidate: event.candidate
        }));
      }
    };
    
    peerConnection.ontrack = (event) => {
      setRemoteStream(event.streams[0]);
    };
    
    peerConnection.onconnectionstatechange = () => {
      setIsConnected(peerConnection.connectionState === 'connected');
    };
    
    return peerConnection;
  }, [options.iceServers]);
  
  const connect = useCallback(async () => {
    try {
      // Create WebSocket connection
      const ws = new WebSocket(options.signalingServer);
      webSocketRef.current = ws;
      
      ws.onopen = () => {
        console.log('WebSocket connected');
      };
      
      ws.onerror = (error) => {
        setError('WebSocket connection failed');
        console.error('WebSocket error:', error);
      };
      
      ws.onmessage = async (event) => {
        const message = JSON.parse(event.data);
        
        switch (message.type) {
          case 'offer':
            await createAnswer(message.offer);
            break;
          case 'answer':
            if (peerConnectionRef.current) {
              await peerConnectionRef.current.setRemoteDescription(message.answer);
            }
            break;
          case 'ice-candidate':
            await addIceCandidate(message.candidate);
            break;
        }
      };
      
      // Create peer connection
      peerConnectionRef.current = createPeerConnection();
      
      // Get local stream (screen sharing)
      const stream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: true
      });
      
      setLocalStream(stream);
      
      // Add tracks to peer connection
      stream.getTracks().forEach(track => {
        peerConnectionRef.current!.addTrack(track, stream);
      });
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, [options.signalingServer, createPeerConnection]);
  
  const disconnect = useCallback(() => {
    if (peerConnectionRef.current) {
      peerConnectionRef.current.close();
      peerConnectionRef.current = null;
    }
    
    if (webSocketRef.current) {
      webSocketRef.current.close();
      webSocketRef.current = null;
    }
    
    if (localStream) {
      localStream.getTracks().forEach(track => track.stop());
      setLocalStream(null);
    }
    
    setRemoteStream(null);
    setIsConnected(false);
  }, [localStream]);
  
  const createOffer = useCallback(async () => {
    if (!peerConnectionRef.current || !webSocketRef.current) {
      throw new Error('Not connected');
    }
    
    const offer = await peerConnectionRef.current.createOffer();
    await peerConnectionRef.current.setLocalDescription(offer);
    
    webSocketRef.current.send(JSON.stringify({
      type: 'offer',
      offer: offer
    }));
  }, []);
  
  const createAnswer = useCallback(async (offer: RTCSessionDescriptionInit) => {
    if (!peerConnectionRef.current || !webSocketRef.current) {
      throw new Error('Not connected');
    }
    
    await peerConnectionRef.current.setRemoteDescription(offer);
    const answer = await peerConnectionRef.current.createAnswer();
    await peerConnectionRef.current.setLocalDescription(answer);
    
    webSocketRef.current.send(JSON.stringify({
      type: 'answer',
      answer: answer
    }));
  }, []);
  
  const addIceCandidate = useCallback(async (candidate: RTCIceCandidateInit) => {
    if (!peerConnectionRef.current) {
      throw new Error('Peer connection not available');
    }
    
    await peerConnectionRef.current.addIceCandidate(candidate);
  }, []);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);
  
  return {
    isConnected,
    localStream,
    remoteStream,
    error,
    connect,
    disconnect,
    createOffer,
    createAnswer,
    addIceCandidate,
  };
}
EOF

echo "✅ React Contexts und Hooks erstellt!"
EOF

chmod +x create-contexts.sh
