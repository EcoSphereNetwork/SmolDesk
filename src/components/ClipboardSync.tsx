// src/components/ClipboardSync.tsx - Frontend-Komponente für Zwischenablage-Synchronisation

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { WebRTCConnection } from '../utils/webrtc';

// Typdefinitionen
interface ClipboardEntry {
  id: string;
  content_type: 'Text' | 'Image' | 'Html' | 'Files';
  data: string;
  metadata: {
    size: number;
    mime_type: string;
    source: string;
  };
  timestamp: string;
}

interface ClipboardSyncConfig {
  enabled: boolean;
  max_content_size: number;
  sync_images: boolean;
  sync_html: boolean;
  sync_files: boolean;
  auto_sync: boolean;
  history_size: number;
}

interface ClipboardSyncProps {
  webrtcConnection?: WebRTCConnection;
  onSync?: (entry: ClipboardEntry) => void;
  onError?: (error: string) => void;
}

const ClipboardSync: React.FC<ClipboardSyncProps> = ({
  webrtcConnection,
  onSync,
  onError,
}) => {
  const [isEnabled, setIsEnabled] = useState<boolean>(false);
  const [history, setHistory] = useState<ClipboardEntry[]>([]);
  const [config, setConfig] = useState<ClipboardSyncConfig>({
    enabled: true,
    max_content_size: 10 * 1024 * 1024, // 10 MB
    sync_images: true,
    sync_html: true,
    sync_files: false,
    auto_sync: true,
    history_size: 50,
  });
  const [currentEntry, setCurrentEntry] = useState<ClipboardEntry | null>(null);
  const [stats, setStats] = useState({
    entries_synced: 0,
    total_bytes_synced: 0,
    entries_sent: 0,
    entries_received: 0,
    sync_errors: 0,
    last_sync: null as string | null,
  });
  const [showHistory, setShowHistory] = useState<boolean>(false);
  const [selectedEntry, setSelectedEntry] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState<string>('');
  const [filterType, setFilterType] = useState<string>('all');
  
  const syncInProgress = useRef<boolean>(false);
  const lastSyncTime = useRef<number>(0);

  // Initialisierung
  useEffect(() => {
    initializeClipboardSync();
    
    // Event-Listener für Änderungen
    const unlistenClipboardChange = listen<ClipboardEntry>('clipboard-changed', (event) => {
      handleClipboardChange(event.payload);
    });
    
    const unlistenRemoteClipboard = listen<ClipboardEntry>('remote-clipboard', (event) => {
      handleRemoteClipboardData(event.payload);
    });
    
    return () => {
      unlistenClipboardChange.then(fn => fn());
      unlistenRemoteClipboard.then(fn => fn());
      
      // Synchronisation stoppen
      if (isEnabled) {
        stopClipboardSync();
      }
    };
  }, []);

  // WebRTC-Integration
  useEffect(() => {
    if (webrtcConnection && isEnabled) {
      // Listener für eingehende Zwischenablage-Daten
      webrtcConnection.on('data-channel-message', (event: any) => {
        if (event.data.type === 'clipboard-sync') {
          handleRemoteClipboardData(event.data.entry);
        }
      });
    }
  }, [webrtcConnection, isEnabled]);

  // Initialisierung der Zwischenablage-Synchronisation
  const initializeClipboardSync = async () => {
    try {
      const displayServer = await invoke<string>('get_display_server');
      await invoke('initialize_clipboard_manager', { displayServer });
      
      // Verlauf laden
      const clipboardHistory = await invoke<ClipboardEntry[]>('get_clipboard_history');
      setHistory(clipboardHistory);
      
      setIsEnabled(true);
    } catch (error) {
      console.error('Failed to initialize clipboard sync:', error);
      if (onError) {
        onError(`Failed to initialize clipboard sync: ${error}`);
      }
    }
  };

  // Zwischenablage-Synchronisation starten
  const startClipboardSync = async () => {
    try {
      await invoke('start_clipboard_monitoring');
      setIsEnabled(true);
    } catch (error) {
      console.error('Failed to start clipboard sync:', error);
      if (onError) {
        onError(`Failed to start clipboard sync: ${error}`);
      }
    }
  };

  // Zwischenablage-Synchronisation stoppen
  const stopClipboardSync = async () => {
    try {
      await invoke('stop_clipboard_monitoring');
      setIsEnabled(false);
    } catch (error) {
      console.error('Failed to stop clipboard sync:', error);
    }
  };

  // Lokale Zwischenablage-Änderung behandeln
  const handleClipboardChange = useCallback(async (entry: ClipboardEntry) => {
    if (!isEnabled || syncInProgress.current) return;
    
    // Rate limiting: Max. 1 Sync pro Sekunde
    const now = Date.now();
    if (now - lastSyncTime.current < 1000) {
      return;
    }
    lastSyncTime.current = now;
    
    try {
      syncInProgress.current = true;
      
      // Prüfen, ob der Eintrag synchronisiert werden soll
      if (shouldSyncEntry(entry)) {
        // Zum lokalen Verlauf hinzufügen
        setHistory(prev => [entry, ...prev.slice(0, config.history_size - 1)]);
        setCurrentEntry(entry);
        
        // An Remote-Peers senden, falls verbunden
        if (webrtcConnection && config.auto_sync) {
          sendClipboardToRemote(entry);
        }
        
        // Statistiken aktualisieren
        setStats(prev => ({
          ...prev,
          entries_synced: prev.entries_synced + 1,
          total_bytes_synced: prev.total_bytes_synced + entry.metadata.size,
          entries_sent: prev.entries_sent + (webrtcConnection ? 1 : 0),
          last_sync: new Date().toISOString(),
        }));
        
        if (onSync) {
          onSync(entry);
        }
      }
    } catch (error) {
      console.error('Error handling clipboard change:', error);
      setStats(prev => ({
        ...prev,
        sync_errors: prev.sync_errors + 1,
      }));
    } finally {
      syncInProgress.current = false;
    }
  }, [isEnabled, config, webrtcConnection, onSync]);

  // Remote-Zwischenablage-Daten behandeln
  const handleRemoteClipboardData = useCallback(async (entry: ClipboardEntry) => {
    if (!isEnabled || syncInProgress.current) return;
    
    try {
      syncInProgress.current = true;
      
      // Prüfen, ob der Eintrag akzeptiert werden soll
      if (shouldAcceptRemoteEntry(entry)) {
        // Zur lokalen Zwischenablage hinzufügen
        await invoke('sync_remote_clipboard_entry', { entry });
        
        // Zum Verlauf hinzufügen
        setHistory(prev => [
          { ...entry, metadata: { ...entry.metadata, source: 'remote' } },
          ...prev.slice(0, config.history_size - 1)
        ]);
        setCurrentEntry(entry);
        
        // Statistiken aktualisieren
        setStats(prev => ({
          ...prev,
          entries_received: prev.entries_received + 1,
          total_bytes_synced: prev.total_bytes_synced + entry.metadata.size,
          last_sync: new Date().toISOString(),
        }));
      }
    } catch (error) {
      console.error('Error handling remote clipboard data:', error);
      setStats(prev => ({
        ...prev,
        sync_errors: prev.sync_errors + 1,
      }));
      
      if (onError) {
        onError(`Failed to sync remote clipboard: ${error}`);
      }
    } finally {
      syncInProgress.current = false;
    }
  }, [isEnabled, config, onError]);

  // Zwischenablage-Eintrag an Remote senden
  const sendClipboardToRemote = (entry: ClipboardEntry) => {
    if (!webrtcConnection) return;
    
    const syncData = {
      type: 'clipboard-sync',
      entry,
      timestamp: new Date().toISOString(),
    };
    
    // An alle verbundenen Peers senden
    const sentCount = webrtcConnection.broadcast(syncData);
    
    if (sentCount > 0) {
      setStats(prev => ({
        ...prev,
        entries_sent: prev.entries_sent + sentCount,
      }));
    }
  };

  // Prüfen, ob ein Eintrag synchronisiert werden soll
  const shouldSyncEntry = (entry: ClipboardEntry): boolean => {
    if (!config.enabled) return false;
    
    // Größenprüfung
    if (entry.metadata.size > config.max_content_size) return false;
    
    // Typ-spezifische Prüfungen
    switch (entry.content_type) {
      case 'Image':
        return config.sync_images;
      case 'Html':
        return config.sync_html;
      case 'Files':
        return config.sync_files;
      case 'Text':
        return true;
      default:
        return false;
    }
  };

  // Prüfen, ob ein Remote-Eintrag akzeptiert werden soll
  const shouldAcceptRemoteEntry = (entry: ClipboardEntry): boolean => {
    // Gleiche Logik wie shouldSyncEntry
    return shouldSyncEntry(entry);
  };

  // Eintrag in die Zwischenablage setzen
  const setClipboardEntry = async (entry: ClipboardEntry) => {
    try {
      switch (entry.content_type) {
        case 'Text':
        case 'Html':
          await invoke('set_clipboard_text', { text: entry.data });
          break;
        case 'Image':
          await invoke('set_clipboard_image', { 
            imageData: entry.data, 
            format: entry.metadata.mime_type.split('/')[1] || 'png' 
          });
          break;
        default:
          throw new Error(`Unsupported content type: ${entry.content_type}`);
      }
      
      setCurrentEntry(entry);
    } catch (error) {
      console.error('Failed to set clipboard entry:', error);
      if (onError) {
        onError(`Failed to set clipboard: ${error}`);
      }
    }
  };

  // Verlauf löschen
  const clearHistory = async () => {
    try {
      await invoke('clear_clipboard_history');
      setHistory([]);
    } catch (error) {
      console.error('Failed to clear clipboard history:', error);
    }
  };

  // Konfigurations-Handler
  const handleConfigChange = (key: keyof ClipboardSyncConfig, value: any) => {
    setConfig(prev => ({ ...prev, [key]: value }));
  };

  // Filter anwenden
  const filteredHistory = history.filter(entry => {
    // Textfilter
    if (searchTerm && !entry.data.toLowerCase().includes(searchTerm.toLowerCase())) {
      return false;
    }
    
    // Typfilter
    if (filterType !== 'all' && entry.content_type.toLowerCase() !== filterType.toLowerCase()) {
      return false;
    }
    
    return true;
  });

  // Formatierte Anzeige für Einträge
  const formatEntryPreview = (entry: ClipboardEntry): string => {
    switch (entry.content_type) {
      case 'Text':
        return entry.data.substring(0, 100) + (entry.data.length > 100 ? '...' : '');
      case 'Html':
        const textContent = entry.data.replace(/<[^>]+>/g, '');
        return textContent.substring(0, 100) + (textContent.length > 100 ? '...' : '');
      case 'Image':
        return `Image (${entry.metadata.mime_type}, ${formatBytes(entry.metadata.size)})`;
      case 'Files':
        const files = entry.data.split('\n').filter(f => f.trim());
        return `${files.length} file(s): ${files[0]} ${files.length > 1 ? '...' : ''}`;
      default:
        return 'Unknown content';
    }
  };

  // Bytes formatieren
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Zeit formatieren
  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="clipboard-sync">
      <div className="clipboard-sync-header">
        <h3>Clipboard Sync</h3>
        <div className="clipboard-sync-controls">
          <button
            onClick={isEnabled ? stopClipboardSync : startClipboardSync}
            className={`toggle-button ${isEnabled ? 'enabled' : 'disabled'}`}
          >
            {isEnabled ? 'Disable' : 'Enable'}
          </button>
          <button
            onClick={() => setShowHistory(!showHistory)}
            className="history-button"
          >
            History ({history.length})
          </button>
        </div>
      </div>

      {/* Status und Statistiken */}
      <div className="clipboard-status">
        <div className="status-item">
          <span>Status: </span>
          <span className={isEnabled ? 'status-enabled' : 'status-disabled'}>
            {isEnabled ? 'Active' : 'Inactive'}
          </span>
        </div>
        <div className="status-item">
          <span>Synced: {stats.entries_synced}</span>
        </div>
        <div className="status-item">
          <span>Sent: {stats.entries_sent}</span>
        </div>
        <div className="status-item">
          <span>Received: {stats.entries_received}</span>
        </div>
        {stats.sync_errors > 0 && (
          <div className="status-item error">
            <span>Errors: {stats.sync_errors}</span>
          </div>
        )}
      </div>

      {/* Aktueller Eintrag */}
      {currentEntry && (
        <div className="current-entry">
          <h4>Current Clipboard</h4>
          <div className="entry-preview">
            <div className="entry-type">{currentEntry.content_type}</div>
            <div className="entry-content">{formatEntryPreview(currentEntry)}</div>
            <div className="entry-meta">
              {formatBytes(currentEntry.metadata.size)} • {formatTime(currentEntry.timestamp)}
            </div>
          </div>
        </div>
      )}

      {/* Konfiguration */}
      <div className="clipboard-config">
        <h4>Settings</h4>
        <div className="config-row">
          <label>
            <input
              type="checkbox"
              checked={config.sync_images}
              onChange={(e) => handleConfigChange('sync_images', e.target.checked)}
            />
            Sync Images
          </label>
        </div>
        <div className="config-row">
          <label>
            <input
              type="checkbox"
              checked={config.sync_html}
              onChange={(e) => handleConfigChange('sync_html', e.target.checked)}
            />
            Sync HTML
          </label>
        </div>
        <div className="config-row">
          <label>
            <input
              type="checkbox"
              checked={config.sync_files}
              onChange={(e) => handleConfigChange('sync_files', e.target.checked)}
            />
            Sync Files
          </label>
        </div>
        <div className="config-row">
          <label>
            <input
              type="checkbox"
              checked={config.auto_sync}
              onChange={(e) => handleConfigChange('auto_sync', e.target.checked)}
            />
            Auto Sync
          </label>
        </div>
      </div>

      {/* Verlauf */}
      {showHistory && (
        <div className="clipboard-history">
          <div className="history-header">
            <h4>Clipboard History</h4>
            <div className="history-controls">
              <input
                type="search"
                placeholder="Search..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="search-input"
              />
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="filter-select"
              >
                <option value="all">All Types</option>
                <option value="text">Text</option>
                <option value="html">HTML</option>
                <option value="image">Images</option>
                <option value="files">Files</option>
              </select>
              <button onClick={clearHistory} className="clear-button">
                Clear All
              </button>
            </div>
          </div>

          <div className="history-list">
            {filteredHistory.map((entry) => (
              <div
                key={entry.id}
                className={`history-entry ${selectedEntry === entry.id ? 'selected' : ''}`}
                onClick={() => setSelectedEntry(entry.id)}
                onDoubleClick={() => setClipboardEntry(entry)}
              >
                <div className="entry-header">
                  <span className="entry-type">{entry.content_type}</span>
                  <span className="entry-time">{formatTime(entry.timestamp)}</span>
                  <span className="entry-source">{entry.metadata.source}</span>
                </div>
                <div className="entry-preview">
                  {formatEntryPreview(entry)}
                </div>
                <div className="entry-meta">
                  {formatBytes(entry.metadata.size)}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ClipboardSync;
