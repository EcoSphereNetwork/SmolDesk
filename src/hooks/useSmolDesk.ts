// src/hooks/useSmolDesk.ts

import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { EnhancedWebRTCConnection, ConnectionQuality } from '../utils/enhancedWebRTC';
import { ScreenCaptureManager } from '../utils/screenCapture';
import { SecurityManager, ConnectionMode, User } from '../utils/securityManager';

// Konfiguration für useSmolDesk
export interface SmolDeskConfig {
  signalingServer: string;
  iceServers?: RTCIceServer[];
  defaultQuality?: number;
  defaultFps?: number;
  securityMode?: ConnectionMode;
  secretKey?: string;
}

// Status des SmolDesk
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

// Rückgabetyp des Hooks
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
  const [webrtc, setWebrtc] = useState<EnhancedWebRTCConnection | null>(null);
  const [captureManager, setCaptureManager] = useState<ScreenCaptureManager | null>(null);
  const securityManager = useMemo(() => SecurityManager.getInstance(), []);
  
  // Initialisierung
  useEffect(() => {
    async function initialize() {
      try {
        // System-Informationen abrufen
        const monitorsData = await invoke<Array<any>>('get_monitors');
        setMonitors(monitorsData);
        
        const codecs = await invoke<string[]>('get_video_codecs');
        setAvailableCodecs(codecs);
        
        const hwAccel = await invoke<string[]>('get_hardware_acceleration_options');
        setAvailableHwAccel(hwAccel);
        
        // SecurityManager initialisieren
        const securityInitialized = await securityManager.initialize(
          mergedConfig.secretKey || 'secure-key',
          mergedConfig.securityMode
        );
        
        if (!securityInitialized) {
          throw new Error('Failed to initialize security manager');
        }
        
        // WebRTC initialisieren
        const webrtcConnection = new EnhancedWebRTCConnection({
          signalingServer: mergedConfig.signalingServer,
          peerConnectionConfig: {
            iceServers: mergedConfig.iceServers || [],
          },
          autoReconnect: true,
          iceGatheringTimeout: 10000,
          enableTrickleICE: true,
          bandwidthConstraints: {
            video: 5000, // 5 Mbps
            audio: 50,   // 50 kbps
          },
        });
        
        setWebrtc(webrtcConnection);
        
        // ScreenCaptureManager initialisieren
        const screenManager = new ScreenCaptureManager(webrtcConnection);
        setCaptureManager(screenManager);
        
        // Event-Listener für Stats
        const unlistenStats = await listen<any>('stream-stats', (event) => {
          setStats({
            fps: event.payload.fps || 0,
            latency: event.payload.latency || 0,
            bitrate: 0, // Wird über WebRTC berechnet
            resolution: event.payload.resolution || ''
          });
        });
        
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
          screenManager.stopCapture().catch(console.error);
        };
      } catch (err: any) {
        setError(err.message);
        setStatus(SmolDeskStatus.ERROR);
        console.error('Failed to initialize SmolDesk:', err);
      }
    }
    
    initialize();
  }, [mergedConfig.signalingServer, mergedConfig.iceServers, mergedConfig.secretKey, mergedConfig.securityMode]);
  
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
      
      // Extrahiere die eigentliche Room-ID (ohne Signatur)
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
      
      // Extrahiere die eigentliche Room-ID (ohne Signatur)
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
        keyframe_interval: captureFps, // Ein Keyframe pro Sekunde
        bitrate: null, // Automatisch basierend auf Qualität
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
