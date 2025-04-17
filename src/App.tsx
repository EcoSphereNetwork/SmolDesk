// src/App.tsx

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import ConnectionManager from './components/ConnectionManager';
import RemoteScreen from './components/RemoteScreen';
import './styles.css';

// Configuration interface for screen capture
interface CaptureConfig {
  fps: number;
  quality: number;
  codec: string;
  hardware_acceleration: string;
  capture_cursor: boolean;
  capture_audio: boolean;
}

// Monitor information interface
interface Monitor {
  index: number;
  name: string;
  width: number;
  height: number;
  refresh_rate?: number;
  primary: boolean;
}

const App: React.FC = () => {
  const [displayServer, setDisplayServer] = useState<string>('');
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [isHost, setIsHost] = useState<boolean>(false);
  const [isViewer, setIsViewer] = useState<boolean>(false);
  const [remoteStream, setRemoteStream] = useState<MediaStream | undefined>(undefined);
  const [localStream, setLocalStream] = useState<MediaStream | undefined>(undefined);
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [selectedMonitor, setSelectedMonitor] = useState<number>(0);
  const [captureConfig, setCaptureConfig] = useState<CaptureConfig>({
    fps: 30,
    quality: 80,
    codec: 'H264',
    hardware_acceleration: 'None',
    capture_cursor: true,
    capture_audio: false,
  });
  const [availableCodecs, setAvailableCodecs] = useState<string[]>([]);
  const [availableHwAccel, setAvailableHwAccel] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isCapturing, setIsCapturing] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<'host' | 'view'>('host');
  
  // Get display server and available monitors
  useEffect(() => {
    invoke<string>('get_display_server')
      .then(server => {
        setDisplayServer(server);
      })
      .catch(err => {
        setError(`Failed to get display server: ${err}`);
      });
    
    // Get available video codecs
    invoke<string[]>('get_video_codecs')
      .then(codecs => {
        setAvailableCodecs(codecs);
      })
      .catch(err => {
        console.error('Failed to get video codecs:', err);
      });
    
    // Get available hardware acceleration options
    invoke<string[]>('get_hardware_acceleration_options')
      .then(options => {
        setAvailableHwAccel(options);
      })
      .catch(err => {
        console.error('Failed to get hardware acceleration options:', err);
      });
    
    // Get monitors
    invoke<Monitor[]>('get_monitors')
      .then(monitorList => {
        setMonitors(monitorList);
        // Select primary monitor by default
        const primaryIndex = monitorList.findIndex(m => m.primary);
        if (primaryIndex !== -1) {
          setSelectedMonitor(primaryIndex);
        }
      })
      .catch(err => {
        setError(`Failed to get monitors: ${err}`);
      });
  }, []);
  
  // Start screen capture
  const startCapture = useCallback(async () => {
    try {
      await invoke('start_capture', {
        monitorIndex: selectedMonitor,
        config: captureConfig
      });
      setIsCapturing(true);
      setError(null);
    } catch (err) {
      setError(`Failed to start capture: ${err}`);
    }
  }, [selectedMonitor, captureConfig]);
  
  // Stop screen capture
  const stopCapture = useCallback(async () => {
    try {
      await invoke('stop_capture');
      setIsCapturing(false);
    } catch (err) {
      setError(`Failed to stop capture: ${err}`);
    }
  }, []);
  
  // Handle connection events
  const handleConnected = useCallback((peerId: string) => {
    setIsConnected(true);
  }, []);
  
  const handleDisconnected = useCallback(() => {
    setIsConnected(false);
    setRemoteStream(undefined);
    // Stop capturing if we were hosting
    if (isHost && isCapturing) {
      stopCapture();
      setIsHost(false);
    }
    setIsViewer(false);
  }, [isHost, isCapturing, stopCapture]);
  
  // Handle incoming stream
  const handleStream = useCallback((stream: MediaStream) => {
    setRemoteStream(stream);
    setIsViewer(true);
  }, []);
  
  // Start hosting
  const startHosting = useCallback(async () => {
    setIsHost(true);
    await startCapture();
  }, [startCapture]);
  
  // Stop hosting
  const stopHosting = useCallback(async () => {
    await stopCapture();
    setIsHost(false);
  }, [stopCapture]);
  
  // Handle config changes
  const handleConfigChange = (key: keyof CaptureConfig, value: any) => {
    setCaptureConfig(prev => ({
      ...prev,
      [key]: value
    }));
  };
  
  // Handle monitor selection
  const handleMonitorChange = (index: number) => {
    setSelectedMonitor(index);
  };
  
  // Handle tab change
  const handleTabChange = (tab: 'host' | 'view') => {
    setActiveTab(tab);
    // If switching away from host mode and currently hosting, stop
    if (tab !== 'host' && isHost && isCapturing) {
      stopHosting();
    }
  };

  return (
    <div className="container mx-auto p-4">
      <header className="bg-gray-800 text-white p-4 rounded-lg mb-4">
        <h1 className="text-2xl font-bold">SmolDesk</h1>
        <p className="text-sm">WebRTC-based Remote Desktop for Linux • {displayServer} Display Server</p>
        {error && (
          <div className="bg-red-600 text-white p-2 mt-2 rounded">
            {error}
          </div>
        )}
      </header>

      <div className="tabs flex mb-4">
        <button
          className={`px-4 py-2 ${activeTab === 'host' ? 'bg-blue-500 text-white' : 'bg-gray-200'} rounded-l`}
          onClick={() => handleTabChange('host')}
        >
          Host
        </button>
        <button
          className={`px-4 py-2 ${activeTab === 'view' ? 'bg-blue-500 text-white' : 'bg-gray-200'} rounded-r`}
          onClick={() => handleTabChange('view')}
        >
          View
        </button>
      </div>

      {activeTab === 'host' && (
        <div className="host-panel">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="settings-panel bg-gray-100 p-4 rounded-lg">
              <h2 className="text-xl font-bold mb-2">Capture Settings</h2>
              
              <div className="mb-4">
                <label className="block mb-1">Monitor</label>
                <select
                  className="w-full p-2 border rounded"
                  value={selectedMonitor}
                  onChange={(e) => handleMonitorChange(Number(e.target.value))}
                  disabled={isCapturing}
                >
                  {monitors.map((monitor, index) => (
                    <option key={index} value={index}>
                      {monitor.name} ({monitor.width}x{monitor.height})
                      {monitor.primary ? ' (Primary)' : ''}
                    </option>
                  ))}
                </select>
              </div>
              
              <div className="mb-4">
                <label className="block mb-1">Frame Rate</label>
                <input
                  type="range"
                  min="1"
                  max="60"
                  value={captureConfig.fps}
                  onChange={(e) => handleConfigChange('fps', Number(e.target.value))}
                  disabled={isCapturing}
                  className="w-full"
                />
                <div className="text-right">{captureConfig.fps} FPS</div>
              </div>
              
              <div className="mb-4">
                <label className="block mb-1">Quality</label>
                <input
                  type="range"
                  min="10"
                  max="100"
                  value={captureConfig.quality}
                  onChange={(e) => handleConfigChange('quality', Number(e.target.value))}
                  disabled={isCapturing}
                  className="w-full"
                />
                <div className="text-right">{captureConfig.quality}%</div>
              </div>
              
              <div className="mb-4">
                <label className="block mb-1">Video Codec</label>
                <select
                  className="w-full p-2 border rounded"
                  value={captureConfig.codec}
                  onChange={(e) => handleConfigChange('codec', e.target.value)}
                  disabled={isCapturing}
                >
                  {availableCodecs.map(codec => (
                    <option key={codec} value={codec}>{codec}</option>
                  ))}
                </select>
              </div>
              
              <div className="mb-4">
                <label className="block mb-1">Hardware Acceleration</label>
                <select
                  className="w-full p-2 border rounded"
                  value={captureConfig.hardware_acceleration}
                  onChange={(e) => handleConfigChange('hardware_acceleration', e.target.value)}
                  disabled={isCapturing}
                >
                  {availableHwAccel.map(option => (
                    <option key={option} value={option}>{option}</option>
                  ))}
                </select>
              </div>
              
              <div className="mb-4">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={captureConfig.capture_cursor}
                    onChange={(e) => handleConfigChange('capture_cursor', e.target.checked)}
                    disabled={isCapturing}
                    className="mr-2"
                  />
                  Capture Cursor
                </label>
              </div>
              
              <div className="mb-4">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={captureConfig.capture_audio}
                    onChange={(e) => handleConfigChange('capture_audio', e.target.checked)}
                    disabled={isCapturing}
                    className="mr-2"
                  />
                  Capture Audio
                </label>
              </div>
              
              <div className="action-buttons">
                {!isCapturing ? (
                  <button
                    onClick={startHosting}
                    className="bg-green-500 text-white px-4 py-2 rounded"
                    disabled={isHost}
                  >
                    Start Hosting
                  </button>
                ) : (
                  <button
                    onClick={stopHosting}
                    className="bg-red-500 text-white px-4 py-2 rounded"
                  >
                    Stop Hosting
                  </button>
                )}
              </div>
            </div>

            <div className="connection-panel bg-gray-100 p-4 rounded-lg">
              <h2 className="text-xl font-bold mb-2">Connection</h2>
              <ConnectionManager
                signalingServer="wss://signaling.smoldesk.example"
                onConnected={handleConnected}
                onDisconnected={handleDisconnected}
                onStream={handleStream}
                onError={(error) => setError(error.message)}
                autoConnect={false}
              />
            </div>
          </div>
          
          {isHost && isCapturing && (
            <div className="mt-4 p-4 bg-green-100 border-green-400 border rounded-lg">
              <h3 className="font-bold text-green-800">Hosting Active</h3>
              <p>Share your room ID with viewers to allow them to connect.</p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'view' && (
        <div className="view-panel">
          <div className="grid grid-cols-1 gap-4">
            <div className="connection-panel bg-gray-100 p-4 rounded-lg">
              <h2 className="text-xl font-bold mb-2">Connect to Remote Desktop</h2>
              <ConnectionManager
                signalingServer="wss://signaling.smoldesk.example"
                onConnected={handleConnected}
                onDisconnected={handleDisconnected}
                onStream={handleStream}
                onError={(error) => setError(error.message)}
                autoConnect={false}
              />
            </div>
            
            {isViewer && remoteStream && (
              <div className="remote-screen-wrapper bg-black rounded-lg overflow-hidden" style={{ height: '70vh' }}>
                <RemoteScreen
                  stream={remoteStream}
                  isConnected={isConnected}
                  inputEnabled={true}
                  onInputToggle={(enabled) => console.log('Input', enabled ? 'enabled' : 'disabled')}
                />
              </div>
            )}
            
            {isViewer && !remoteStream && (
              <div className="p-4 bg-blue-100 border-blue-400 border rounded-lg">
                <h3 className="font-bold text-blue-800">Connected</h3>
                <p>Waiting for remote screen stream...</p>
              </div>
            )}
          </div>
        </div>
      )}

      <footer className="mt-8 text-center text-gray-500 text-sm">
        <p>SmolDesk • WebRTC-based Remote Desktop for Linux</p>
        <p>Supports {displayServer} • Low-Latency P2P Connection</p>
      </footer>
    </div>
  );
};

export default App;
