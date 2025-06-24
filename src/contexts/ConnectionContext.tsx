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
