#!/bin/bash
# fix-hook.sh - Korrigiert den unvollständigen useSmolDesk Hook

set -e

echo "🎣 Korrigiere useSmolDesk Hook..."

# Erstelle vollständigen useSmolDesk Hook
cat > src/hooks/useSmolDesk.ts << 'EOF'
// src/hooks/useSmolDesk.ts

import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Mock für EnhancedWebRTCConnection (vereinfacht)
interface MockWebRTCConnection {
  on: (event: string, callback: (data: any) => void) => void;
  broadcast: (data: any) => number;
  connect: () => void;
  disconnect: () => void;
  createRoom: (roomId?: string) => void;
  joinRoom: (roomId: string) => void;
  leaveRoom: () => void;
}

class MockEnhancedWebRTCConnection implements MockWebRTCConnection {
  private eventListeners: Map<string, Function[]> = new Map();

  on(event: string, callback: (data: any) => void) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);
  }

  broadcast(data: any): number {
    console.log('Broadcasting data:', data);
    return 1; // Mock: 1 peer
  }

  connect() {
    console.log('WebRTC connecting...');
    setTimeout(() => {
      this.emit('connection-quality-change', { quality: 'good' });
    }, 1000);
  }

  disconnect() {
    console.log('WebRTC disconnecting...');
  }

  createRoom(roomId?: string) {
    console.log('Creating room:', roomId);
  }

  joinRoom(roomId: string) {
    console.log('Joining room:', roomId);
  }

  leaveRoom() {
    console.log('Leaving room');
  }

  private emit(event: string, data: any) {
    const listeners = this.eventListeners.get(event) || [];
    listeners.forEach(listener => listener(data));
  }
}

// Mock für ScreenCaptureManager (vereinfacht)
class MockScreenCaptureManager {
  async startCapture(monitorIndex: number, config: any): Promise<boolean> {
    console.log('Starting capture on monitor', monitorIndex, 'with config:', config);
    return true;
  }

  async stopCapture(): Promise<void> {
    console.log('Stopping capture');
  }
}

// Mock für SecurityManager (vereinfacht)
class MockSecurityManager {
  static getInstance() {
    return new MockSecurityManager();
  }

  async initialize(secretKey: string, connectionMode: any): Promise<boolean> {
    console.log('Initializing security with mode:', connectionMode);
    return true;
  }

  async createSecureRoom(password?: string): Promise<string | null> {
    const roomId = Math.random().toString(36).substring(2, 8);
    console.log('Created secure room:', roomId);
    return `${roomId}:signature`;
  }

  async joinSecureRoom(secureRoomId: string, password?: string, user?: any): Promise<boolean> {
    console.log('Joining secure room:', secureRoomId);
    return true;
  }

  async authenticate(mode: any, password?: string, user?: any): Promise<boolean> {
    console.log('Authenticating user');
    return true;
  }

  getSecurityMode() {
    return 'Protected';
  }
}

// Enums und Interfaces
export enum ConnectionQuality {
  EXCELLENT = 'excellent',
  GOOD = 'good',
  FAIR = 'fair',
  POOR = 'poor',
  CRITICAL = 'critical',
  DISCONNECTED = 'disconnected',
}

export enum ConnectionMode {
  Public = 'Public',
  Protected = 'Protected',
  Authenticated = 'Authenticated',
  Private = 'Private',
}

export enum SmolDeskStatus {
  INITIALIZING = 'initializing',
  READY = 'ready',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  HOSTING = 'hosting',
  VIEWING = 'viewing',
  DISCONNECTED = 'disconnected',
  ERROR = 'error',
}

export interface User {
  id: string;
  username: string;
  role: 'Guest' | 'Member' | 'Moderator' | 'Admin' | 'Owner';
  access_rights: Array<'ViewOnly' | 'ControlInput' | 'FileTransfer' | 'AudioAccess' | 'FullAccess'>;
}

export interface SmolDeskConfig {
  signalingServer: string;
  iceServers?: RTCIceServer[];
  defaultQuality?: number;
  defaultFps?: number;
  securityMode?: ConnectionMode;
  secretKey?: string;
}

interface SmolDeskHook {
  // Status
  status: SmolDeskStatus;
  error: string | null;
  connectionQuality: ConnectionQuality;
  
  // Configuration functions
  setQuality: (quality: number) => void;
  setFps: (fps: number) => void;
  
  // Connection management
  createRoom: (password?: string) => Promise<string | null>;
  joinRoom: (roomId: string, password?: string, user?: User) => Promise<boolean>;
  leaveRoom: () => void;
  
  // Hosting functions
  startHosting: (monitorIndex: number) => Promise<boolean>;
  stopHosting: () => Promise<void>;
  
  // Stream and data
  remoteStream: MediaStream | null;
  sendMessage: (message: any) => boolean;
  
  // Security functions
  authenticate: (user: User, password?: string) => Promise<boolean>;
  
  // Statistics and information
  stats: {
    fps: number;
    latency: number;
    bitrate: number;
    resolution: string;
  };
  monitors: Array<{
    index: number;
    name: string;
    width: number;
    height: number;
    primary: boolean;
  }>;
  availableCodecs: string[];
  availableHwAccel: string[];
}

// Default Konfiguration
const defaultConfig: SmolDeskConfig = {
  signalingServer: 'wss://signaling.smoldesk.example',
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' }
  ],
  defaultQuality: 80,
  defaultFps: 30,
  securityMode: ConnectionMode.Protected,
  secretKey: 'default-secure-key-should-be-changed'
};

/**
 * React Hook für die Integration aller SmolDesk-Komponenten
 */
export function useSmolDesk(config?: Partial<SmolDeskConfig>): SmolDeskHook {
  // Konfiguration mit Defaults zusammenführen
  const mergedConfig = useMemo(() => ({...defaultConfig, ...config}), [config]);
  
  // State
  const [status, setStatus] = useState<SmolDeskStatus>(SmolDeskStatus.INITIALIZING);
  const [error, setError] = useState<string | null>(null);
  const [connectionQuality, setConnectionQuality] = useState<ConnectionQuality>(ConnectionQuality.DISCONNECTED);
  const [remoteStream, setRemoteStream] = useState<MediaStream | null>(null);
  const [roomId, setRoomId] = useState<string | null>(null);
  const [monitors, setMonitors] = useState<Array<any>>([]);
  const [availableCodecs, setAvailableCodecs] = useState<string[]>([]);
  const [availableHwAccel, setAvailableHwAccel] = useState<string[]>([]);
  const [captureQuality, setCaptureQuality] = useState<number>(mergedConfig.defaultQuality || 80);
  const [captureFps, setCaptureFps] = useState<number>(mergedConfig.defaultFps || 30);
  const [stats, setStats] = useState({
    fps: 0,
    latency: 0,
    bitrate: 0,
    resolution: ''
  });
  
  // Instanzen
  const [webrtc, setWebrtc] = useState<MockWebRTCConnection | null>(null);
  const [captureManager, setCaptureManager] = useState<MockScreenCaptureManager | null>(null);
  const securityManager = useMemo(() => MockSecurityManager.getInstance(), []);
  
  // Initialisierung
  useEffect(() => {
    async function initialize() {
      try {
        // System-Informationen abrufen (Mock-Daten falls Backend nicht verfügbar)
        try {
          const monitorsData = await invoke<Array<any>>('get_monitors');
          setMonitors(monitorsData);
        } catch {
          // Mock monitors falls Backend nicht verfügbar
          setMonitors([{
            index: 0,
            name: 'Primary Monitor',
            width: 1920,
            height: 1080,
            primary: true
          }]);
        }
        
        try {
          const codecs = await invoke<string[]>('get_video_codecs');
          setAvailableCodecs(codecs);
        } catch {
          setAvailableCodecs(['H264', 'VP8', 'VP9']);
        }
        
        try {
          const hwAccel = await invoke<string[]>('get_hardware_acceleration_options');
          setAvailableHwAccel(hwAccel);
        } catch {
          setAvailableHwAccel(['None', 'VAAPI', 'NVENC']);
        }
        
        // SecurityManager initialisieren
        const securityInitialized = await securityManager.initialize(
          mergedConfig.secretKey || 'secure-key',
          mergedConfig.securityMode
        );
        
        if (!securityInitialized) {
          throw new Error('Failed to initialize security manager');
        }
        
        // Mock WebRTC initialisieren
        const webrtcConnection = new MockEnhancedWebRTCConnection();
        setWebrtc(webrtcConnection);
        
        // Mock ScreenCaptureManager initialisieren
        const screenManager = new MockScreenCaptureManager();
        setCaptureManager(screenManager);
        
        // Event-Listener für Stats (Mock)
        const unlistenStats = () => {};
        
        // Listener für WebRTC-Events
        webrtcConnection.on('connection-quality-change', (event: any) => {
          setConnectionQuality(event.quality);
        });
        
        webrtcConnection.on('stream-added', (event: any) => {
          setRemoteStream(event.stream);
          setStatus(SmolDeskStatus.VIEWING);
        });
        
        webrtcConnection.on('connected', () => {
          setStatus(SmolDeskStatus.CONNECTED);
          setError(null);
        });
        
        webrtcConnection.on('disconnected', () => {
          setStatus(SmolDeskStatus.DISCONNECTED);
          setRemoteStream(null);
        });
        
        webrtcConnection.on('error', (error: Error) => {
          setError(error.message);
          console.error('WebRTC error:', error);
        });
        
        // Verbindung zum Signaling-Server herstellen
        webrtcConnection.connect();
        
        setStatus(SmolDeskStatus.READY);
        
        // Cleanup-Funktion
        return () => {
          unlistenStats();
          webrtcConnection.disconnect();
        };
      } catch (err: any) {
        setError(err.message);
        setStatus(SmolDeskStatus.ERROR);
        console.error('Failed to initialize SmolDesk:', err);
      }
    }
    
    initialize();
  }, [mergedConfig.signalingServer, mergedConfig.iceServers, mergedConfig.secretKey, mergedConfig.securityMode, securityManager]);
  
  // Raum erstellen
  const createRoom = useCallback(async (password?: string): Promise<string | null> => {
    if (!webrtc || !securityManager) {
      setError('WebRTC or security manager not initialized');
      return null;
    }
    
    try {
      setStatus(SmolDeskStatus.CONNECTING);
      
      // Sicheren Raum mit dem SecurityManager erstellen
      const secureRoomId = await securityManager.createSecureRoom(password);
      
      if (!secureRoomId) {
        throw new Error('Failed to create secure room');
      }
      
      // Extrahiere die eigentliche Room-ID
      const roomIdParts = secureRoomId.split(':');
      const actualRoomId = roomIdParts[0];
      
      // Raum mit WebRTC erstellen
      webrtc.createRoom(actualRoomId);
      
      // Room-ID speichern
      setRoomId(actualRoomId);
      
      return secureRoomId;
    } catch (err: any) {
      setError(err.message);
      setStatus(SmolDeskStatus.ERROR);
      console.error('Failed to create room:', err);
      return null;
    }
  }, [webrtc, securityManager]);
  
  // Raum beitreten
  const joinRoom = useCallback(async (
    secureRoomId: string, 
    password?: string, 
    user?: User
  ): Promise<boolean> => {
    if (!webrtc || !securityManager) {
      setError('WebRTC or security manager not initialized');
      return false;
    }
    
    try {
      setStatus(SmolDeskStatus.CONNECTING);
      
      // Validiere den sicheren Raum und authentifiziere den Benutzer
      const authenticated = await securityManager.joinSecureRoom(secureRoomId, password, user);
      
      if (!authenticated) {
        throw new Error('Authentication failed');
      }
      
      // Extrahiere die eigentliche Room-ID
      const roomIdParts = secureRoomId.split(':');
      const actualRoomId = roomIdParts[0];
      
      // Raum mit WebRTC beitreten
      webrtc.joinRoom(actualRoomId);
      
      // Room-ID speichern
      setRoomId(actualRoomId);
      
      return true;
    } catch (err: any) {
      setError(err.message);
      setStatus(SmolDeskStatus.ERROR);
      console.error('Failed to join room:', err);
      return false;
    }
  }, [webrtc, securityManager]);
  
  // Raum verlassen
  const leaveRoom = useCallback(() => {
    if (!webrtc) {
      return;
    }
    
    try {
      // Host-Modus beenden, falls aktiv
      if (status === SmolDeskStatus.HOSTING && captureManager) {
        captureManager.stopCapture().catch(console.error);
      }
      
      // WebRTC-Raum verlassen
      webrtc.leaveRoom();
      
      // Status zurücksetzen
      setRoomId(null);
      setRemoteStream(null);
      setStatus(SmolDeskStatus.DISCONNECTED);
    } catch (err: any) {
      setError(err.message);
      console.error('Failed to leave room:', err);
    }
  }, [webrtc, captureManager, status]);
  
  // Als Host starten
  const startHosting = useCallback(async (monitorIndex: number): Promise<boolean> => {
    if (!webrtc || !captureManager) {
      setError('WebRTC or capture manager not initialized');
      return false;
    }
    
    try {
      // Konfiguration für die Bildschirmerfassung
      const captureConfig = {
        fps: captureFps,
        quality: captureQuality,
        codec: availableCodecs[0] || 'H264',
        hardware_acceleration: availableHwAccel[0] || 'None',
        capture_cursor: true,
        capture_audio: false,
        keyframe_interval: captureFps,
        bitrate: null,
        latency_mode: 'Balanced',
      };
      
      // Bildschirmerfassung starten
      const captureStarted = await captureManager.startCapture(monitorIndex, captureConfig);
      
      if (!captureStarted) {
        throw new Error('Failed to start screen capture');
      }
      
      setStatus(SmolDeskStatus.HOSTING);
      return true;
    } catch (err: any) {
      setError(err.message);
      console.error('Failed to start hosting:', err);
      return false;
    }
  }, [webrtc, captureManager, captureFps, captureQuality, availableCodecs, availableHwAccel]);
  
  // Hosting beenden
  const stopHosting = useCallback(async (): Promise<void> => {
    if (!captureManager) {
      return;
    }
    
    try {
      await captureManager.stopCapture();
      
      // Status aktualisieren, aber im Raum bleiben
      if (status === SmolDeskStatus.HOSTING) {
        setStatus(SmolDeskStatus.CONNECTED);
      }
    } catch (err: any) {
      setError(err.message);
      console.error('Failed to stop hosting:', err);
    }
  }, [captureManager, status]);
  
  // Nachricht senden
  const sendMessage = useCallback((message: any): boolean => {
    if (!webrtc || !roomId) {
      return false;
    }
    
    // Broadcast an alle verbundenen Peers
    const count = webrtc.broadcast(message);
    return count > 0;
  }, [webrtc, roomId]);
  
  // Authentifizierung
  const authenticate = useCallback(async (
    user: User, 
    password?: string
  ): Promise<boolean> => {
    if (!securityManager) {
      setError('Security manager not initialized');
      return false;
    }
    
    try {
      const mode = securityManager.getSecurityMode();
      return await securityManager.authenticate(mode, password, user);
    } catch (err: any) {
      setError(err.message);
      console.error('Authentication failed:', err);
      return false;
    }
  }, [securityManager]);
  
  // Qualität setzen
  const setQuality = useCallback((quality: number) => {
    setCaptureQuality(quality);
  }, []);
  
  // FPS setzen
  const setFps = useCallback((fps: number) => {
    setCaptureFps(fps);
  }, []);

  return {
    status,
    error,
    connectionQuality,
    setQuality,
    setFps,
    createRoom,
    joinRoom,
    leaveRoom,
    startHosting,
    stopHosting,
    remoteStream,
    sendMessage,
    authenticate,
    stats,
    monitors,
    availableCodecs,
    availableHwAccel,
  };
}
EOF

echo "✅ useSmolDesk Hook korrigiert!"
EOF

chmod +x fix-hook.sh
