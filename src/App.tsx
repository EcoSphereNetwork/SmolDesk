// src/App.tsx - Erweiterte Version mit vollständiger Feature-Integration

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import ConnectionManager from './components/ConnectionManager';
import RemoteScreen from './components/RemoteScreen';
import ClipboardSync from './components/ClipboardSync';
import FileTransfer from './components/FileTransfer';
import { useSmolDesk } from './hooks/useSmolDesk';
import { SecurityManager, ConnectionMode } from './utils/securityManager';
import './styles.css';

// Theme-Konfiguration
type Theme = 'light' | 'dark' | 'auto';
type Language = 'en' | 'de' | 'fr' | 'es';

// Lokalisierungs-Interface
interface Translations {
  [key: string]: {
    [lang in Language]: string;
  };
}

const translations: Translations = {
  appTitle: {
    en: 'SmolDesk - WebRTC Remote Desktop',
    de: 'SmolDesk - WebRTC Remote Desktop',
    fr: 'SmolDesk - Bureau à distance WebRTC',
    es: 'SmolDesk - Escritorio remoto WebRTC'
  },
  displayServer: {
    en: 'Display Server',
    de: 'Display-Server',
    fr: 'Serveur d\'affichage',
    es: 'Servidor de pantalla'
  },
  hostMode: {
    en: 'Host',
    de: 'Host',
    fr: 'Hôte',
    es: 'Anfitrión'
  },
  viewMode: {
    en: 'View',
    de: 'Anzeigen',
    fr: 'Voir',
    es: 'Ver'
  },
  // Weitere Übersetzungen...
};

// Erweiterte Configuration Interface
interface AppConfig {
  theme: Theme;
  language: Language;
  autoConnectLastRoom: boolean;
  enableNotifications: boolean;
  enableClipboardSync: boolean;
  enableFileTransfer: boolean;
  enableSecureMode: boolean;
  captureConfig: CaptureConfig;
}

interface CaptureConfig {
  fps: number;
  quality: number;
  codec: string;
  hardware_acceleration: string;
  capture_cursor: boolean;
  capture_audio: boolean;
}

interface Monitor {
  index: number;
  name: string;
  width: number;
  height: number;
  refresh_rate?: number;
  primary: boolean;
}

const App: React.FC = () => {
  // Basis-State
  const [displayServer, setDisplayServer] = useState<string>('');
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [availableCodecs, setAvailableCodecs] = useState<string[]>([]);
  const [availableHwAccel, setAvailableHwAccel] = useState<string[]>([]);
  
  // App-Konfiguration
  const [config, setConfig] = useState<AppConfig>({
    theme: 'auto',
    language: 'en',
    autoConnectLastRoom: false,
    enableNotifications: true,
    enableClipboardSync: true,
    enableFileTransfer: true,
    enableSecureMode: true,
    captureConfig: {
      fps: 30,
      quality: 80,
      codec: 'H264',
      hardware_acceleration: 'None',
      capture_cursor: true,
      capture_audio: false,
    }
  });

  // UI-State
  const [activeTab, setActiveTab] = useState<'host' | 'view' | 'settings'>('host');
  const [showSidebar, setShowSidebar] = useState<boolean>(true);
  const [showNotifications, setShowNotifications] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [notifications, setNotifications] = useState<Array<{
    id: string;
    type: 'info' | 'success' | 'warning' | 'error';
    message: string;
    timestamp: Date;
  }>>([]);

  // SmolDesk Hook für vereinfachte Verwaltung
  const {
    status,
    error: smolDeskError,
    connectionQuality,
    createRoom,
    joinRoom,
    leaveRoom,
    startHosting,
    stopHosting,
    remoteStream,
    sendMessage,
    authenticate,
    stats,
    setQuality,
    setFps
  } = useSmolDesk({
    signalingServer: 'wss://signaling.smoldesk.example',
    defaultQuality: config.captureConfig.quality,
    defaultFps: config.captureConfig.fps,
    securityMode: config.enableSecureMode ? ConnectionMode.Protected : ConnectionMode.Public
  });

  // Security Manager
  const securityManager = SecurityManager.getInstance();

  // Initialisierung
  useEffect(() => {
    initializeApp();
    loadUserConfig();
    
    // Theme anwenden
    applyTheme(config.theme);
    
    // Event Listener für System-Events
    const unlistenNotification = listen<any>('system-notification', (event) => {
      addNotification('info', event.payload.message);
    });

    return () => {
      unlistenNotification.then(fn => fn());
    };
  }, []);

  // App initialisieren
  const initializeApp = async () => {
    try {
      // System-Informationen abrufen
      const [serverInfo, monitorList, codecList, hwAccelList] = await Promise.all([
        invoke<string>('get_display_server'),
        invoke<Monitor[]>('get_monitors'),
        invoke<string[]>('get_video_codecs'),
        invoke<string[]>('get_hardware_acceleration_options')
      ]);

      setDisplayServer(serverInfo);
      setMonitors(monitorList);
      setAvailableCodecs(codecList);
      setAvailableHwAccel(hwAccelList);

      // Security Manager initialisieren falls aktiviert
      if (config.enableSecureMode) {
        await securityManager.initialize('secure-smoldesk-key', ConnectionMode.Protected);
      }

      addNotification('success', 'SmolDesk initialized successfully');
    } catch (err: any) {
      setError(`Initialization failed: ${err}`);
      addNotification('error', `Initialization failed: ${err}`);
    }
  };

  // Benutzer-Konfiguration laden
  const loadUserConfig = async () => {
    try {
      const savedConfig = localStorage.getItem('smoldesk-config');
      if (savedConfig) {
        const parsedConfig = JSON.parse(savedConfig);
        setConfig(prev => ({ ...prev, ...parsedConfig }));
      }
    } catch (error) {
      console.warn('Failed to load user config:', error);
    }
  };

  // Konfiguration speichern
  const saveUserConfig = useCallback(() => {
    try {
      localStorage.setItem('smoldesk-config', JSON.stringify(config));
    } catch (error) {
      console.warn('Failed to save user config:', error);
    }
  }, [config]);

  // Theme anwenden
  const applyTheme = (theme: Theme) => {
    const root = document.documentElement;
    
    if (theme === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      root.setAttribute('data-theme', theme);
    }
  };

  // Übersetzung abrufen
  const t = (key: string): string => {
    return translations[key]?.[config.language] || key;
  };

  // Benachrichtigung hinzufügen
  const addNotification = (type: 'info' | 'success' | 'warning' | 'error', message: string) => {
    if (!config.enableNotifications) return;

    const notification = {
      id: Date.now().toString(),
      type,
      message,
      timestamp: new Date()
    };

    setNotifications(prev => [notification, ...prev.slice(0, 4)]); // Max 5 Benachrichtigungen

    // Auto-remove nach 5 Sekunden (außer bei Fehlern)
    if (type !== 'error') {
      setTimeout(() => {
        setNotifications(prev => prev.filter(n => n.id !== notification.id));
      }, 5000);
    }
  };

  // Hosting starten
  const handleStartHosting = async () => {
    try {
      const roomId = await createRoom();
      if (roomId) {
        const monitorIndex = monitors.findIndex(m => m.primary) || 0;
        const success = await startHosting(monitorIndex);
        
        if (success) {
          addNotification('success', `Hosting started. Room ID: ${roomId}`);
        } else {
          throw new Error('Failed to start hosting');
        }
      }
    } catch (error: any) {
      addNotification('error', `Failed to start hosting: ${error.message}`);
    }
  };

  // Konfiguration aktualisieren
  const updateConfig = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setConfig(prev => {
      const newConfig = { ...prev, [key]: value };
      
      // Spezielle Behandlung für bestimmte Konfigurationen
      if (key === 'theme') {
        applyTheme(value as Theme);
      }
      
      if (key === 'captureConfig') {
        const captureConfig = value as CaptureConfig;
        setQuality(captureConfig.quality);
        setFps(captureConfig.fps);
      }
      
      return newConfig;
    });
  };

  // Capture-Konfiguration aktualisieren
  const updateCaptureConfig = <K extends keyof CaptureConfig>(key: K, value: CaptureConfig[K]) => {
    updateConfig('captureConfig', {
      ...config.captureConfig,
      [key]: value
    });
  };

  // Error-Handling
  useEffect(() => {
    if (smolDeskError) {
      setError(smolDeskError);
      addNotification('error', smolDeskError);
    } else {
      setError(null);
    }
  }, [smolDeskError]);

  // Konfiguration automatisch speichern
  useEffect(() => {
    const timeoutId = setTimeout(saveUserConfig, 1000);
    return () => clearTimeout(timeoutId);
  }, [config, saveUserConfig]);

  return (
    <div className={`app ${config.theme}`} data-theme={config.theme}>
      {/* Header */}
      <header className="app-header">
        <div className="header-content">
          <div className="header-left">
            <h1 className="app-title">{t('appTitle')}</h1>
            <div className="system-info">
              <span className="display-server">{displayServer} {t('displayServer')}</span>
              <span className={`connection-status status-${status}`}>{status}</span>
              {connectionQuality !== 'disconnected' && (
                <span className={`connection-quality quality-${connectionQuality}`}>
                  {connectionQuality}
                </span>
              )}
            </div>
          </div>
          
          <div className="header-right">
            <div className="stats-display">
              <span>FPS: {stats.fps.toFixed(1)}</span>
              <span>Latency: {stats.latency}ms</span>
              <span>Quality: {config.captureConfig.quality}%</span>
            </div>
            
            <button 
              className="sidebar-toggle"
              onClick={() => setShowSidebar(!showSidebar)}
              aria-label="Toggle Sidebar"
            >
              ☰
            </button>
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <div className="error-banner">
            <span>⚠️ {error}</span>
            <button onClick={() => setError(null)}>✕</button>
          </div>
        )}
      </header>

      <div className="app-content">
        {/* Sidebar */}
        <aside className={`sidebar ${showSidebar ? 'visible' : 'hidden'}`}>
          <nav className="sidebar-nav">
            <button 
              className={`nav-button ${activeTab === 'host' ? 'active' : ''}`}
              onClick={() => setActiveTab('host')}
            >
              🖥️ {t('hostMode')}
            </button>
            <button 
              className={`nav-button ${activeTab === 'view' ? 'active' : ''}`}
              onClick={() => setActiveTab('view')}
            >
              👀 {t('viewMode')}
            </button>
            <button 
              className={`nav-button ${activeTab === 'settings' ? 'active' : ''}`}
              onClick={() => setActiveTab('settings')}
            >
              ⚙️ Settings
            </button>
          </nav>

          {/* Quick Stats */}
          <div className="sidebar-stats">
            <h3>System Stats</h3>
            <div className="stat-item">
              <span>Monitors:</span>
              <span>{monitors.length}</span>
            </div>
            <div className="stat-item">
              <span>Codecs:</span>
              <span>{availableCodecs.length}</span>
            </div>
            <div className="stat-item">
              <span>HW Accel:</span>
              <span>{availableHwAccel.filter(h => h !== 'None').length}</span>
            </div>
          </div>

          {/* Feature Toggles in Sidebar */}
          <div className="sidebar-features">
            <h3>Features</h3>
            <label className="feature-toggle">
              <input 
                type="checkbox" 
                checked={config.enableClipboardSync}
                onChange={(e) => updateConfig('enableClipboardSync', e.target.checked)}
              />
              📋 Clipboard Sync
            </label>
            <label className="feature-toggle">
              <input 
                type="checkbox" 
                checked={config.enableFileTransfer}
                onChange={(e) => updateConfig('enableFileTransfer', e.target.checked)}
              />
              📁 File Transfer
            </label>
            <label className="feature-toggle">
              <input 
                type="checkbox" 
                checked={config.enableSecureMode}
                onChange={(e) => updateConfig('enableSecureMode', e.target.checked)}
              />
              🔒 Secure Mode
            </label>
          </div>
        </aside>

        {/* Main Content */}
        <main className="main-content">
          {activeTab === 'host' && (
            <div className="host-panel">
              <div className="panel-grid">
                {/* Host Settings */}
                <section className="settings-section">
                  <h2>Host Settings</h2>
                  
                  <div className="setting-group">
                    <label>Monitor</label>
                    <select 
                      className="form-select"
                      value={monitors.findIndex(m => m.primary)}
                      onChange={(e) => {
                        // Monitor-Auswahl-Logik
                      }}
                    >
                      {monitors.map((monitor, index) => (
                        <option key={index} value={index}>
                          {monitor.name} ({monitor.width}x{monitor.height})
                          {monitor.primary ? ' (Primary)' : ''}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="setting-row">
                    <div className="setting-group">
                      <label>Frame Rate: {config.captureConfig.fps} FPS</label>
                      <input
                        type="range"
                        min="1"
                        max="60"
                        value={config.captureConfig.fps}
                        onChange={(e) => updateCaptureConfig('fps', Number(e.target.value))}
                        className="form-range"
                      />
                    </div>
                    
                    <div className="setting-group">
                      <label>Quality: {config.captureConfig.quality}%</label>
                      <input
                        type="range"
                        min="10"
                        max="100"
                        value={config.captureConfig.quality}
                        onChange={(e) => updateCaptureConfig('quality', Number(e.target.value))}
                        className="form-range"
                      />
                    </div>
                  </div>

                  <div className="setting-row">
                    <div className="setting-group">
                      <label>Video Codec</label>
                      <select
                        className="form-select"
                        value={config.captureConfig.codec}
                        onChange={(e) => updateCaptureConfig('codec', e.target.value)}
                      >
                        {availableCodecs.map(codec => (
                          <option key={codec} value={codec}>{codec}</option>
                        ))}
                      </select>
                    </div>
                    
                    <div className="setting-group">
                      <label>Hardware Acceleration</label>
                      <select
                        className="form-select"
                        value={config.captureConfig.hardware_acceleration}
                        onChange={(e) => updateCaptureConfig('hardware_acceleration', e.target.value)}
                      >
                        {availableHwAccel.map(option => (
                          <option key={option} value={option}>{option}</option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div className="checkbox-group">
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={config.captureConfig.capture_cursor}
                        onChange={(e) => updateCaptureConfig('capture_cursor', e.target.checked)}
                      />
                      Capture Cursor
                    </label>
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={config.captureConfig.capture_audio}
                        onChange={(e) => updateCaptureConfig('capture_audio', e.target.checked)}
                      />
                      Capture Audio
                    </label>
                  </div>

                  <div className="action-buttons">
                    <button 
                      className="btn btn-primary"
                      onClick={handleStartHosting}
                      disabled={status === 'hosting'}
                    >
                      {status === 'hosting' ? 'Stop Hosting' : 'Start Hosting'}
                    </button>
                  </div>
                </section>

                {/* Connection Manager */}
                <section className="connection-section">
                  <h2>Connection</h2>
                  <ConnectionManager
                    signalingServer="wss://signaling.smoldesk.example"
                    onConnected={(peerId) => addNotification('success', `Connected to ${peerId}`)}
                    onDisconnected={() => addNotification('info', 'Disconnected')}
                    onStream={(stream) => {
                      // Stream handling wird vom useSmolDesk Hook übernommen
                    }}
                    onError={(error) => addNotification('error', error.message)}
                    autoConnect={false}
                  />
                </section>
              </div>
            </div>
          )}

          {activeTab === 'view' && (
            <div className="view-panel">
              <div className="panel-header">
                <h2>Remote Desktop Viewer</h2>
                {remoteStream && (
                  <div className="stream-info">
                    Connected • {stats.resolution} • {stats.fps.toFixed(1)} FPS
                  </div>
                )}
              </div>

              {remoteStream ? (
                <div className="remote-screen-container">
                  <RemoteScreen
                    stream={remoteStream}
                    isConnected={status === 'viewing'}
                    inputEnabled={true}
                    onInputToggle={(enabled) => {
                      addNotification('info', `Input ${enabled ? 'enabled' : 'disabled'}`);
                    }}
                  />
                </div>
              ) : (
                <div className="no-stream-placeholder">
                  <div className="placeholder-content">
                    <h3>Not Connected</h3>
                    <p>Connect to a remote desktop to start viewing</p>
                    <ConnectionManager
                      signalingServer="wss://signaling.smoldesk.example"
                      onConnected={(peerId) => addNotification('success', `Connected to ${peerId}`)}
                      onDisconnected={() => addNotification('info', 'Disconnected')}
                      onStream={(stream) => {
                        // Stream wird vom useSmolDesk Hook verwaltet
                      }}
                      onError={(error) => addNotification('error', error.message)}
                      autoConnect={false}
                    />
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'settings' && (
            <div className="settings-panel">
              <div className="settings-grid">
                {/* General Settings */}
                <section className="settings-section">
                  <h2>General Settings</h2>
                  
                  <div className="setting-group">
                    <label>Theme</label>
                    <select
                      className="form-select"
                      value={config.theme}
                      onChange={(e) => updateConfig('theme', e.target.value as Theme)}
                    >
                      <option value="light">Light</option>
                      <option value="dark">Dark</option>
                      <option value="auto">Auto</option>
                    </select>
                  </div>

                  <div className="setting-group">
                    <label>Language</label>
                    <select
                      className="form-select"
                      value={config.language}
                      onChange={(e) => updateConfig('language', e.target.value as Language)}
                    >
                      <option value="en">English</option>
                      <option value="de">Deutsch</option>
                      <option value="fr">Français</option>
                      <option value="es">Español</option>
                    </select>
                  </div>

                  <div className="checkbox-group">
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={config.enableNotifications}
                        onChange={(e) => updateConfig('enableNotifications', e.target.checked)}
                      />
                      Enable Notifications
                    </label>
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={config.autoConnectLastRoom}
                        onChange={(e) => updateConfig('autoConnectLastRoom', e.target.checked)}
                      />
                      Auto-connect to last room
                    </label>
                  </div>
                </section>

                {/* System Information */}
                <section className="settings-section">
                  <h2>System Information</h2>
                  <div className="info-grid">
                    <div className="info-item">
                      <span className="info-label">Display Server:</span>
                      <span className="info-value">{displayServer}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Monitors:</span>
                      <span className="info-value">{monitors.length}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Available Codecs:</span>
                      <span className="info-value">{availableCodecs.join(', ')}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Hardware Acceleration:</span>
                      <span className="info-value">
                        {availableHwAccel.filter(h => h !== 'None').join(', ') || 'None'}
                      </span>
                    </div>
                  </div>
                </section>
              </div>
            </div>
          )}
        </main>

        {/* Feature Panels (Sidebar) */}
        {showSidebar && (
          <aside className="feature-sidebar">
            {config.enableClipboardSync && (
              <div className="feature-panel">
                <ClipboardSync
                  onSync={(entry) => addNotification('info', 'Clipboard synced')}
                  onError={(error) => addNotification('error', error)}
                />
              </div>
            )}

            {config.enableFileTransfer && (
              <div className="feature-panel">
                <FileTransfer
                  onTransferComplete={(transferId) => 
                    addNotification('success', 'File transfer completed')
                  }
                  onError={(error) => addNotification('error', error)}
                />
              </div>
            )}
          </aside>
        )}
      </div>

      {/* Notifications */}
      {showNotifications && notifications.length > 0 && (
        <div className="notifications-container">
          {notifications.map(notification => (
            <div 
              key={notification.id}
              className={`notification notification-${notification.type}`}
              role="alert"
            >
              <div className="notification-content">
                <span className="notification-message">{notification.message}</span>
                <span className="notification-time">
                  {notification.timestamp.toLocaleTimeString()}
                </span>
              </div>
              <button 
                className="notification-close"
                onClick={() => setNotifications(prev => 
                  prev.filter(n => n.id !== notification.id)
                )}
                aria-label="Close notification"
              >
                ✕
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default App;
