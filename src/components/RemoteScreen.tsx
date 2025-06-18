// src/components/RemoteScreen.tsx

import React, { useRef, useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface RemoteScreenProps {
  stream?: MediaStream;
  isConnected: boolean;
  inputEnabled?: boolean;
  onInputToggle?: (enabled: boolean) => void;
}

export interface InputEvent {
  event_type: 'MouseMove' | 'MouseButton' | 'MouseScroll' | 'KeyPress' | 'KeyRelease';
  x?: number;
  y?: number;
  button?: 'Left' | 'Middle' | 'Right' | 'Back' | 'Forward' | 'ScrollUp' | 'ScrollDown';
  key_code?: number;
  modifiers?: string[];
  is_pressed?: boolean;
  delta_x?: number;
  delta_y?: number;
}

const RemoteScreen: React.FC<RemoteScreenProps> = ({
  stream,
  isConnected,
  inputEnabled = true,
  onInputToggle
}) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [localInputEnabled, setLocalInputEnabled] = useState(inputEnabled);
  const [scale, setScale] = useState(1);
  const [isLoading, setIsLoading] = useState(true);
  const [statsVisible, setStatsVisible] = useState(false);
  const [stats, setStats] = useState<{
    fps: number;
    bitrate: number;
    latency: number;
  }>({
    fps: 0,
    bitrate: 0,
    latency: 0
  });

  // Handle incoming stream
  useEffect(() => {
    if (stream && videoRef.current) {
      videoRef.current.srcObject = stream;
      videoRef.current.play().catch(error => {
        console.error('Error playing video:', error);
      });
      setIsLoading(false);
    } else {
      setIsLoading(true);
    }
  }, [stream]);

  // Listen for capture stats events from Tauri
  useEffect(() => {
    const unlisten = listen('capture_stats', (event) => {
      const captureStats = event.payload as any;
      setStats(prev => ({
        ...prev,
        fps: captureStats.fps,
        bitrate: Math.round(captureStats.bitrate / 1000), // Convert to kbps
      }));
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Calculate scale based on container and video dimensions
  const calculateScale = useCallback(() => {
    if (videoRef.current && containerRef.current) {
      const videoWidth = videoRef.current.videoWidth;
      const videoHeight = videoRef.current.videoHeight;
      
      if (videoWidth === 0 || videoHeight === 0) return;
      
      const containerWidth = containerRef.current.clientWidth;
      const containerHeight = containerRef.current.clientHeight;
      
      const widthScale = containerWidth / videoWidth;
      const heightScale = containerHeight / videoHeight;
      
      // Use the smaller scale to fit the video within the container
      const newScale = Math.min(widthScale, heightScale);
      setScale(newScale);
    }
  }, []);

  // Recalculate scale when window resizes
  useEffect(() => {
    const handleResize = () => {
      calculateScale();
    };
    
    window.addEventListener('resize', handleResize);
    // Initial calculation
    calculateScale();
    
    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [calculateScale]);

  // Update scale when video metadata is loaded
  const handleVideoMetadata = useCallback(() => {
    calculateScale();
    setIsLoading(false);
  }, [calculateScale]);

  // Toggle fullscreen
  const toggleFullscreen = useCallback(() => {
    if (!containerRef.current) return;
    
    if (!isFullscreen) {
      if (containerRef.current.requestFullscreen) {
        containerRef.current.requestFullscreen();
      }
    } else {
      if (document.exitFullscreen) {
        document.exitFullscreen();
      }
    }
  }, [isFullscreen]);

  // Monitor fullscreen state
  useEffect(() => {
    const handleFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
      calculateScale();
    };
    
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
    };
  }, [calculateScale]);

  // Toggle input forwarding
  const toggleInput = useCallback(() => {
    const newState = !localInputEnabled;
    setLocalInputEnabled(newState);
    
    // Update backend
    invoke('set_input_enabled', { enabled: newState })
      .catch(error => {
        console.error('Failed to toggle input forwarding:', error);
      });
    
    // Notify parent
    if (onInputToggle) {
      onInputToggle(newState);
    }
  }, [localInputEnabled, onInputToggle]);

  // Handle mouse move events
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLVideoElement>) => {
    if (!localInputEnabled || !isConnected) return;
    
    // Get position relative to the video element
    const video = videoRef.current;
    if (!video) return;
    
    const rect = video.getBoundingClientRect();
    const x = Math.round((e.clientX - rect.left) / scale);
    const y = Math.round((e.clientY - rect.top) / scale);
    
    // Ensure coordinates are within video bounds
    if (x < 0 || y < 0 || x > video.videoWidth || y > video.videoHeight) return;
    
    const inputEvent: InputEvent = {
      event_type: 'MouseMove',
      x,
      y,
    };
    
    invoke('send_input_event', { event: inputEvent })
      .catch(error => {
        console.error('Failed to send mouse move event:', error);
      });
  }, [localInputEnabled, isConnected, scale]);

  // Handle mouse button events
  const handleMouseButton = useCallback((e: React.MouseEvent<HTMLVideoElement>, isPressed: boolean) => {
    if (!localInputEnabled || !isConnected) return;
    e.preventDefault();
    
    // Map mouse button
    let button: InputEvent['button'];
    switch (e.button) {
      case 0:
        button = 'Left';
        break;
      case 1:
        button = 'Middle';
        break;
      case 2:
        button = 'Right';
        break;
      case 3:
        button = 'Back';
        break;
      case 4:
        button = 'Forward';
        break;
      default:
        return;
    }
    
    const inputEvent: InputEvent = {
      event_type: 'MouseButton',
      button,
      is_pressed: isPressed,
    };
    
    invoke('send_input_event', { event: inputEvent })
      .catch(error => {
        console.error('Failed to send mouse button event:', error);
      });
  }, [localInputEnabled, isConnected]);

  // Handle mouse wheel events
  const handleWheel = useCallback((e: React.WheelEvent<HTMLVideoElement>) => {
    if (!localInputEnabled || !isConnected) return;
    e.preventDefault();
    
    const inputEvent: InputEvent = {
      event_type: 'MouseScroll',
      delta_x: e.deltaX / 100, // Normalize delta values
      delta_y: e.deltaY / 100,
    };
    
    invoke('send_input_event', { event: inputEvent })
      .catch(error => {
        console.error('Failed to send mouse scroll event:', error);
      });
  }, [localInputEnabled, isConnected]);

  // Handle keyboard events
  const handleKeyEvent = useCallback((e: KeyboardEvent, isPressed: boolean) => {
    if (!localInputEnabled || !isConnected) return;
    
    // Prevent default browser actions for most keys
    if (e.key !== 'F11' && e.key !== 'F12') {
      e.preventDefault();
    }
    
    const modifiers: string[] = [];
    if (e.shiftKey) modifiers.push('shift');
    if (e.ctrlKey) modifiers.push('ctrl');
    if (e.altKey) modifiers.push('alt');
    if (e.metaKey) modifiers.push('meta');
    
    const inputEvent: InputEvent = {
      event_type: isPressed ? 'KeyPress' : 'KeyRelease',
      key_code: e.keyCode,
      modifiers: modifiers.length > 0 ? modifiers : undefined,
    };
    
    invoke('send_input_event', { event: inputEvent })
      .catch(error => {
        console.error(`Failed to send key ${isPressed ? 'press' : 'release'} event:`, error);
      });
  }, [localInputEnabled, isConnected]);

  // Set up keyboard event listeners
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => handleKeyEvent(e, true);
    const handleKeyUp = (e: KeyboardEvent) => handleKeyEvent(e, false);
    
    if (isConnected && localInputEnabled) {
      window.addEventListener('keydown', handleKeyDown);
      window.addEventListener('keyup', handleKeyUp);
    }
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [isConnected, localInputEnabled, handleKeyEvent]);

  // Toggle stats display
  const toggleStats = useCallback(() => {
    setStatsVisible(prev => !prev);
  }, []);

  return (
    <div 
      ref={containerRef} 
      className={`remote-screen-container ${isFullscreen ? 'fullscreen' : ''}`}
      style={{ position: 'relative', width: '100%', height: '100%', overflow: 'hidden' }}
    >
      {isLoading && (
        <div className="loading-overlay">
          <span>Waiting for stream...</span>
        </div>
      )}
      
      <video
        ref={videoRef}
        style={{
          display: isLoading ? 'none' : 'block',
          width: videoRef.current ? videoRef.current.videoWidth * scale : '100%',
          height: videoRef.current ? videoRef.current.videoHeight * scale : '100%',
          margin: '0 auto',
        }}
        onLoadedMetadata={handleVideoMetadata}
        onMouseMove={handleMouseMove}
        onMouseDown={(e) => handleMouseButton(e, true)}
        onMouseUp={(e) => handleMouseButton(e, false)}
        onWheel={handleWheel}
        onContextMenu={(e) => e.preventDefault()}
        autoPlay
        playsInline
      ></video>
      
      <div className="remote-screen-controls">
        <button onClick={toggleFullscreen} className="fullscreen-toggle">
          {isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
        </button>
        
        <button onClick={toggleInput} className={`input-toggle ${localInputEnabled ? 'enabled' : 'disabled'}`}>
          Input: {localInputEnabled ? 'On' : 'Off'}
        </button>
        
        <button onClick={toggleStats} className="stats-toggle">
          {statsVisible ? 'Hide Stats' : 'Show Stats'}
        </button>
      </div>
      
      {statsVisible && (
        <div className="stats-overlay">
          <div>FPS: {stats.fps.toFixed(1)}</div>
          <div>Bitrate: {stats.bitrate} kbps</div>
          <div>Latency: {stats.latency.toFixed(0)} ms</div>
        </div>
      )}
    </div>
  );
};

export default RemoteScreen;
