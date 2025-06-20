// src/components/FileTransfer.tsx - File Transfer UI Component

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { WebRTCConnection } from '../utils/webrtc';

// Type definitions
export interface TransferInfo {
  id: string;
  transfer_type: 'Upload' | 'Download';
  peer_id: string;
  status: 'Preparing' | 'Active' | 'Paused' | 'Completed' | 'Failed' | 'Cancelled';
  file_metadata: {
    name: string;
    size: number;
    mime_type: string;
    created: string;
    modified: string;
    permissions: number;
    attributes: Record<string, any>;
  };
  progress: {
    bytes_transferred: number;
    total_bytes: number;
    chunks_completed: number;
    total_chunks: number;
    transfer_rate: number;
    eta_seconds?: number;
  };
  started_at: number;
  last_activity: number;
  retry_count: number;
}

export interface TransferStats {
  uploads_started: number;
  downloads_completed: number;
  total_bytes_transferred: number;
  total_bytes_queued: number;
}

export interface FileTransferProps {
  webrtcConnection?: WebRTCConnection;
  onTransferComplete?: (transferId: string) => void;
  onError?: (error: string) => void;
}

const FileTransfer: React.FC<FileTransferProps> = ({
  webrtcConnection,
  onTransferComplete,
  onError,
}) => {
  const [activeTransfers, setActiveTransfers] = useState<TransferInfo[]>([]);
  const [transferStats, setTransferStats] = useState<TransferStats>({
    uploads_started: 0,
    downloads_completed: 0,
    total_bytes_transferred: 0,
    total_bytes_queued: 0,
  });
  const [showTransferDialog, setShowTransferDialog] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<FileList | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [transferHistory, setTransferHistory] = useState<TransferInfo[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  // Initialize file transfer system
  useEffect(() => {
    const initializeFileTransfer = async () => {
      try {
        // Initialize file transfer manager
        await invoke('initialize_file_transfer');
        
        // Get current transfer stats
        const stats = await invoke<TransferStats>('get_transfer_stats');
        setTransferStats(stats);
        
        // Get active transfers
        const transfers = await invoke<TransferInfo[]>('get_active_transfers');
        setActiveTransfers(transfers);
      } catch (error) {
        console.error('Failed to initialize file transfer:', error);
        if (onError) {
          onError(`Failed to initialize file transfer: ${error}`);
        }
      }
    };

    initializeFileTransfer();

    // Set up event listeners
    const unlistenTransferStarted = listen<{ transfer_id: string; transfer_type: string; file_metadata: any; peer_id: string }>('transfer-started', (event) => {
      console.log('Transfer started:', event.payload);
      refreshActiveTransfers();
    });

    const unlistenTransferProgress = listen<{ transfer_id: string; progress: any }>('transfer-progress', (event) => {
      const { transfer_id, progress } = event.payload;
      setActiveTransfers(prev => prev.map(transfer => 
        transfer.id === transfer_id 
          ? { ...transfer, progress, last_activity: Date.now() }
          : transfer
      ));
    });

    const unlistenTransferCompleted = listen<{ transfer_id: string }>('transfer-completed', (event) => {
      const { transfer_id } = event.payload;
      setActiveTransfers(prev => prev.map(transfer => 
        transfer.id === transfer_id 
          ? { ...transfer, status: 'Completed' }
          : transfer
      ));
      
      refreshTransferStats();
      if (onTransferComplete) {
        onTransferComplete(transfer_id);
      }
    });

    const unlistenTransferFailed = listen<{ transfer_id: string; error: string }>('transfer-failed', (event) => {
      const { transfer_id, error } = event.payload;
      setActiveTransfers(prev => prev.map(transfer => 
        transfer.id === transfer_id 
          ? { ...transfer, status: 'Failed' }
          : transfer
      ));
      
      if (onError) {
        onError(`Transfer failed: ${error}`);
      }
    });

    const unlistenTransferRequested = listen<{ transfer_id: string; peer_id: string; file_metadata: any }>('transfer-requested', (event) => {
      const { transfer_id, peer_id, file_metadata } = event.payload;
      
      // Show confirmation dialog for incoming transfers
      const accept = window.confirm(
        `${peer_id} wants to send you a file:\n` +
        `Name: ${file_metadata.name}\n` +
        `Size: ${formatBytes(file_metadata.size)}\n\n` +
        `Do you want to accept this file?`
      );
      
      if (accept) {
        handleAcceptTransfer(transfer_id);
      } else {
        handleRejectTransfer(transfer_id);
      }
    });

    return () => {
      unlistenTransferStarted.then(fn => fn());
      unlistenTransferProgress.then(fn => fn());
      unlistenTransferCompleted.then(fn => fn());
      unlistenTransferFailed.then(fn => fn());
      unlistenTransferRequested.then(fn => fn());
    };
  }, [onTransferComplete, onError]);

  // Refresh active transfers
  const refreshActiveTransfers = async () => {
    try {
      const transfers = await invoke<TransferInfo[]>('get_active_transfers');
      setActiveTransfers(transfers);
    } catch (error) {
      console.error('Failed to refresh transfers:', error);
    }
  };

  // Refresh transfer stats
  const refreshTransferStats = async () => {
    try {
      const stats = await invoke<TransferStats>('get_transfer_stats');
      setTransferStats(stats);
    } catch (error) {
      console.error('Failed to refresh stats:', error);
    }
  };

  // Handle file selection
  const handleFileSelect = (files: FileList) => {
    setSelectedFiles(files);
    setShowTransferDialog(true);
  };

  // Handle drag and drop
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      handleFileSelect(files);
    }
  }, []);

  // Start file transfer
  const handleStartTransfer = async (destinationPeer: string) => {
    if (!selectedFiles || selectedFiles.length === 0) {
      return;
    }

    try {
      for (let i = 0; i < selectedFiles.length; i++) {
        const file = selectedFiles[i];
        
        // Start transfer for each file
        const transferId = await invoke<string>('start_file_upload', {
          filePath: (file as any).path || file.name, // File path from drag/drop or name
          destinationPeer,
          metadata: {
            name: file.name,
            size: file.size,
            mime_type: file.type || 'application/octet-stream',
            created: new Date().toISOString(),
            modified: new Date(file.lastModified).toISOString(),
            permissions: 0o644,
            attributes: {}
          }
        });
        
        console.log(`Started transfer: ${transferId}`);
      }
      
      setShowTransferDialog(false);
      setSelectedFiles(null);
      refreshActiveTransfers();
      refreshTransferStats();
    } catch (error) {
      console.error('Failed to start transfer:', error);
      if (onError) {
        onError(`Failed to start transfer: ${error}`);
      }
    }
  };

  // Accept incoming transfer
  const handleAcceptTransfer = async (transferId: string) => {
    try {
      // Let user choose destination
      const destinationPath = await invoke<string>('show_save_dialog', {
        defaultPath: 'Downloads',
        filters: [{ name: 'All Files', extensions: ['*'] }]
      });
      
      if (destinationPath) {
        await invoke('accept_file_transfer', { transferId, destinationPath });
        refreshActiveTransfers();
      } else {
        await invoke('reject_file_transfer', { transferId, reason: 'User cancelled' });
      }
    } catch (error) {
      console.error('Failed to accept transfer:', error);
      if (onError) {
        onError(`Failed to accept transfer: ${error}`);
      }
    }
  };

  // Reject incoming transfer
  const handleRejectTransfer = async (transferId: string, reason?: string) => {
    try {
      await invoke('reject_file_transfer', { 
        transferId, 
        reason: reason || 'Transfer rejected by user' 
      });
      refreshActiveTransfers();
    } catch (error) {
      console.error('Failed to reject transfer:', error);
    }
  };

  // Pause/Resume transfer
  const handlePauseResumeTransfer = async (transferId: string, currentStatus: string) => {
    try {
      if (currentStatus === 'Active') {
        await invoke('pause_file_transfer', { transferId });
      } else if (currentStatus === 'Paused') {
        await invoke('resume_file_transfer', { transferId });
      }
      refreshActiveTransfers();
    } catch (error) {
      console.error('Failed to pause/resume transfer:', error);
    }
  };

  // Cancel transfer
  const handleCancelTransfer = async (transferId: string) => {
    try {
      await invoke('cancel_file_transfer', { transferId });
      refreshActiveTransfers();
    } catch (error) {
      console.error('Failed to cancel transfer:', error);
    }
  };

  // Format bytes
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Format transfer rate
  const formatTransferRate = (bytesPerSecond: number): string => {
    return `${formatBytes(bytesPerSecond)}/s`;
  };

  // Format ETA
  const formatETA = (seconds?: number): string => {
    if (!seconds || seconds <= 0) return 'Unknown';
    
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  // Calculate progress percentage
  const getProgressPercentage = (progress: TransferInfo['progress']): number => {
    if (progress.total_bytes === 0) return 0;
    return (progress.bytes_transferred / progress.total_bytes) * 100;
  };

  return (
    <div className="file-transfer">
      <div className="file-transfer-header">
        <h3>File Transfer</h3>
        <div className="file-transfer-controls">
          <button
            onClick={() => fileInputRef.current?.click()}
            className="send-files-button"
          >
            Send Files
          </button>
          <button
            onClick={() => setShowHistory(!showHistory)}
            className="history-button"
          >
            History
          </button>
        </div>
      </div>

      {/* Statistics */}
      <div className="transfer-stats">
        <div className="stat-item">
          <span>Uploads: {transferStats.uploads_started}</span>
        </div>
        <div className="stat-item">
          <span>Downloads: {transferStats.downloads_completed}</span>
        </div>
        <div className="stat-item">
          <span>Total: {formatBytes(transferStats.total_bytes_transferred)}</span>
        </div>
        <div className="stat-item">
          <span>Queued: {formatBytes(transferStats.total_bytes_queued)}</span>
        </div>
      </div>

      {/* Drop zone */}
      <div
        ref={dropZoneRef}
        className={`drop-zone ${dragOver ? 'drag-over' : ''}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
      >
        <div className="drop-zone-content">
          <div className="drop-zone-icon">📁</div>
          <p>Drag files here or click to select</p>
          <p className="drop-zone-hint">Files will be sent to connected peers</p>
        </div>
      </div>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        style={{ display: 'none' }}
        onChange={(e) => {
          if (e.target.files && e.target.files.length > 0) {
            handleFileSelect(e.target.files);
          }
        }}
      />

      {/* Active transfers */}
      {activeTransfers.length > 0 && (
        <div className="active-transfers">
          <h4>Active Transfers</h4>
          {activeTransfers.map((transfer) => (
            <div key={transfer.id} className="transfer-item">
              <div className="transfer-header">
                <div className="transfer-info">
                  <span className="transfer-name">{transfer.file_metadata.name}</span>
                  <span className="transfer-size">{formatBytes(transfer.file_metadata.size)}</span>
                  <span className={`transfer-status status-${transfer.status.toLowerCase()}`}>
                    {transfer.status}
                  </span>
                </div>
                <div className="transfer-controls">
                  {(transfer.status === 'Active' || transfer.status === 'Paused') && (
                    <button
                      onClick={() => handlePauseResumeTransfer(transfer.id, transfer.status)}
                      className="control-button"
                    >
                      {transfer.status === 'Active' ? '⏸️' : '▶️'}
                    </button>
                  )}
                  {transfer.status !== 'Completed' && transfer.status !== 'Failed' && (
                    <button
                      onClick={() => handleCancelTransfer(transfer.id)}
                      className="control-button cancel"
                    >
                      ❌
                    </button>
                  )}
                </div>
              </div>
              
              <div className="transfer-progress">
                <div className="progress-bar">
                  <div 
                    className="progress-fill"
                    style={{ width: `${getProgressPercentage(transfer.progress)}%` }}
                  />
                </div>
                <div className="progress-info">
                  <span>{getProgressPercentage(transfer.progress).toFixed(1)}%</span>
                  <span>
                    {transfer.progress.transfer_rate > 0 && 
                      formatTransferRate(transfer.progress.transfer_rate)
                    }
                  </span>
                  <span>ETA: {formatETA(transfer.progress.eta_seconds)}</span>
                </div>
              </div>
              
              <div className="transfer-details">
                <span>Peer: {transfer.peer_id}</span>
                <span>Type: {transfer.transfer_type}</span>
                <span>
                  Progress: {formatBytes(transfer.progress.bytes_transferred)} / {formatBytes(transfer.progress.total_bytes)}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Transfer dialog */}
      {showTransferDialog && selectedFiles && (
        <div className="transfer-dialog-overlay">
          <div className="transfer-dialog">
            <h3>Send Files</h3>
            <div className="selected-files">
              <h4>Selected Files:</h4>
              {Array.from(selectedFiles).map((file, index) => (
                <div key={index} className="selected-file">
                  <span>{file.name}</span>
                  <span>{formatBytes(file.size)}</span>
                </div>
              ))}
            </div>
            
            <div className="peer-selection">
              <label>Send to peer:</label>
              <select id="peer-select">
                <option value="">Select peer...</option>
                {/* Peers would be populated from WebRTC connection */}
                <option value="peer1">Peer 1</option>
                <option value="peer2">Peer 2</option>
              </select>
            </div>
            
            <div className="dialog-buttons">
              <button
                onClick={() => {
                  const select = document.getElementById('peer-select') as HTMLSelectElement;
                  if (select.value) {
                    handleStartTransfer(select.value);
                  }
                }}
                className="send-button"
              >
                Send Files
              </button>
              <button
                onClick={() => {
                  setShowTransferDialog(false);
                  setSelectedFiles(null);
                }}
                className="cancel-button"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Transfer history */}
      {showHistory && (
        <div className="transfer-history">
          <h4>Transfer History</h4>
          <div className="history-list">
            {transferHistory.map((transfer) => (
              <div key={transfer.id} className="history-item">
                <div className="history-info">
                  <span className="history-name">{transfer.file_metadata.name}</span>
                  <span className="history-size">{formatBytes(transfer.file_metadata.size)}</span>
                  <span className={`history-status status-${transfer.status.toLowerCase()}`}>
                    {transfer.status}
                  </span>
                </div>
                <div className="history-details">
                  <span>Peer: {transfer.peer_id}</span>
                  <span>Type: {transfer.transfer_type}</span>
                  <span>Date: {new Date(transfer.started_at).toLocaleDateString()}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default FileTransfer;
