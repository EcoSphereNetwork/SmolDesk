// src-tauri/src/file_transfer/mod.rs - Dateiübertragungssystem für SmolDesk

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tokio::sync::mpsc;

pub mod error;
pub mod types;
pub mod chunk_manager;
pub mod security;

use error::FileTransferError;
use types::*;
use chunk_manager::ChunkManager;
use security::FileTransferSecurity;

/// Hauptmanager für Dateiübertragungen
pub struct FileTransferManager {
    /// Aktive Übertragungen (Upload und Download)
    active_transfers: Arc<Mutex<HashMap<String, TransferSession>>>,
    
    /// Chunk-Manager für die Verwaltung von Datei-Chunks
    chunk_manager: Arc<ChunkManager>,
    
    /// Sicherheitsmanager
    security: Arc<FileTransferSecurity>,
    
    /// Konfiguration
    config: TransferConfig,
    
    /// Event-Sender für UI-Updates
    event_sender: Option<mpsc::UnboundedSender<TransferEvent>>,
    
    /// Statistiken
    stats: Arc<Mutex<TransferStats>>,
}

impl FileTransferManager {
    /// Erstellt einen neuen FileTransferManager
    pub fn new(config: TransferConfig) -> Result<Self, FileTransferError> {
        let chunk_manager = Arc::new(ChunkManager::new(config.chunk_size));
        let security = Arc::new(FileTransferSecurity::new(config.encryption_enabled)?);
        
        Ok(FileTransferManager {
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
            chunk_manager,
            security,
            config,
            event_sender: None,
            stats: Arc::new(Mutex::new(TransferStats::default())),
        })
    }
    
    /// Setzt den Event-Sender für UI-Updates
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<TransferEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Startet eine neue Datei-Upload-Session
    pub async fn start_upload(
        &self,
        file_path: &Path,
        destination_peer: &str,
        metadata: Option<FileMetadata>
    ) -> Result<String, FileTransferError> {
        // Datei validieren
        if !file_path.exists() {
            return Err(FileTransferError::FileNotFound(file_path.to_string_lossy().to_string()));
        }
        
        if !file_path.is_file() {
            return Err(FileTransferError::InvalidFileType("Not a regular file".to_string()));
        }
        
        // Dateigröße prüfen
        let file_size = file_path.metadata()
            .map_err(|e| FileTransferError::IoError(e.to_string()))?
            .len();
        
        if file_size > self.config.max_file_size {
            return Err(FileTransferError::FileTooLarge(file_size, self.config.max_file_size));
        }
        
        // Transfer-ID generieren
        let transfer_id = Uuid::new_v4().to_string();
        
        // Datei-Hash berechnen
        let file_hash = self.calculate_file_hash(file_path).await?;
        
        // Metadaten erstellen
        let file_metadata = metadata.unwrap_or_else(|| FileMetadata {
            name: file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            size: file_size,
            mime_type: self.detect_mime_type(file_path),
            created: SystemTime::now(),
            modified: file_path.metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now()),
            permissions: self.get_file_permissions(file_path),
            attributes: HashMap::new(),
        });
        
        // Transfer-Session erstellen
        let session = TransferSession {
            id: transfer_id.clone(),
            transfer_type: TransferType::Upload,
            peer_id: destination_peer.to_string(),
            status: TransferStatus::Preparing,
            file_metadata: file_metadata.clone(),
            file_hash: Some(file_hash.clone()),
            source_path: Some(file_path.to_path_buf()),
            destination_path: None,
            progress: TransferProgress {
                bytes_transferred: 0,
                total_bytes: file_size,
                chunks_completed: 0,
                total_chunks: ((file_size + self.config.chunk_size as u64 - 1) / self.config.chunk_size as u64) as usize,
                transfer_rate: 0.0,
                eta_seconds: None,
            },
            started_at: Instant::now(),
            last_activity: Instant::now(),
            retry_count: 0,
            chunks: HashMap::new(),
        };
        
        // Session speichern
        {
            let mut transfers = self.active_transfers.lock().unwrap();
            transfers.insert(transfer_id.clone(), session);
        }
        
        // Event senden
        self.send_event(TransferEvent::TransferStarted {
            transfer_id: transfer_id.clone(),
            transfer_type: TransferType::Upload,
            file_metadata: file_metadata.clone(),
            peer_id: destination_peer.to_string(),
        }).await;
        
        // Upload-Anfrage an Peer senden
        self.send_transfer_request(destination_peer, TransferRequest {
            transfer_id: transfer_id.clone(),
            file_metadata,
            file_hash,
            chunk_size: self.config.chunk_size,
            total_chunks: ((file_size + self.config.chunk_size as u64 - 1) / self.config.chunk_size as u64) as usize,
            encryption_enabled: self.config.encryption_enabled,
        }).await?;
        
        // Statistiken aktualisieren
        {
            let mut stats = self.stats.lock().unwrap();
            stats.uploads_started += 1;
            stats.total_bytes_queued += file_size;
        }
        
        Ok(transfer_id)
    }
    
    /// Akzeptiert eine eingehende Dateiübertragung
    pub async fn accept_transfer(
        &self,
        transfer_id: &str,
        destination_path: &Path
    ) -> Result<(), FileTransferError> {
        // Zielverzeichnis validieren
        if let Some(parent) = destination_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| FileTransferError::IoError(e.to_string()))?;
            }
        }
        
        // Session aktualisieren
        {
            let mut transfers = self.active_transfers.lock().unwrap();
            if let Some(session) = transfers.get_mut(transfer_id) {
                session.destination_path = Some(destination_path.to_path_buf());
                session.status = TransferStatus::Active;
                session.last_activity = Instant::now();
            } else {
                return Err(FileTransferError::TransferNotFound(transfer_id.to_string()));
            }
        }
        
        // Akzeptanz-Nachricht senden
        self.send_transfer_response(transfer_id, TransferResponse::Accept {
            transfer_id: transfer_id.to_string(),
            ready: true,
        }).await?;
        
        // Event senden
        self.send_event(TransferEvent::TransferAccepted {
            transfer_id: transfer_id.to_string(),
        }).await;
        
        Ok(())
    }
    
    /// Lehnt eine eingehende Dateiübertragung ab
    pub async fn reject_transfer(
        &self,
        transfer_id: &str,
        reason: Option<&str>
    ) -> Result<(), FileTransferError> {
        // Session entfernen
        {
            let mut transfers = self.active_transfers.lock().unwrap();
            transfers.remove(transfer_id);
        }
        
        // Ablehnungs-Nachricht senden
        self.send_transfer_response(transfer_id, TransferResponse::Reject {
            transfer_id: transfer_id.to_string(),
            reason: reason.unwrap_or("Transfer rejected by user").to_string(),
        }).await?;
        
        // Event senden
        self.send_event(TransferEvent::TransferRejected {
            transfer_id: transfer_id.to_string(),
            reason: reason.unwrap_or("Transfer rejected by user").to_string(),
        }).await;
        
        Ok(())
    }
    
    /// Pausiert eine aktive Übertragung
    pub async fn pause_transfer(&self, transfer_id: &str) -> Result<(), FileTransferError> {
        let mut transfers = self.active_transfers.lock().unwrap();
        if let Some(session) = transfers.get_mut(transfer_id) {
            match session.status {
                TransferStatus::Active => {
                    session.status = TransferStatus::Paused;
                    session.last_activity = Instant::now();
                    
                    // Event senden
                    drop(transfers); // Mutex freigeben vor async
                    self.send_event(TransferEvent::TransferPaused {
                        transfer_id: transfer_id.to_string(),
                    }).await;
                    
                    Ok(())
                },
                _ => Err(FileTransferError::InvalidOperation(
                    format!("Cannot pause transfer in status: {:?}", session.status)
                ))
            }
        } else {
            Err(FileTransferError::TransferNotFound(transfer_id.to_string()))
        }
    }
    
    /// Setzt eine pausierte Übertragung fort
    pub async fn resume_transfer(&self, transfer_id: &str) -> Result<(), FileTransferError> {
        let mut transfers = self.active_transfers.lock().unwrap();
        if let Some(session) = transfers.get_mut(transfer_id) {
            match session.status {
                TransferStatus::Paused => {
                    session.status = TransferStatus::Active;
                    session.last_activity = Instant::now();
                    
                    // Event senden
                    drop(transfers); // Mutex freigeben vor async
                    self.send_event(TransferEvent::TransferResumed {
                        transfer_id: transfer_id.to_string(),
                    }).await;
                    
                    Ok(())
                },
                _ => Err(FileTransferError::InvalidOperation(
                    format!("Cannot resume transfer in status: {:?}", session.status)
                ))
            }
        } else {
            Err(FileTransferError::TransferNotFound(transfer_id.to_string()))
        }
    }
    
    /// Bricht eine Übertragung ab
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), FileTransferError> {
        // Session entfernen
        let session = {
            let mut transfers = self.active_transfers.lock().unwrap();
            transfers.remove(transfer_id)
        };
        
        if let Some(session) = session {
            // Unvollständige Datei löschen bei Downloads
            if session.transfer_type == TransferType::Download {
                if let Some(dest_path) = &session.destination_path {
                    let _ = std::fs::remove_file(dest_path);
                }
            }
            
            // Event senden
            self.send_event(TransferEvent::TransferCancelled {
                transfer_id: transfer_id.to_string(),
            }).await;
            
            Ok(())
        } else {
            Err(FileTransferError::TransferNotFound(transfer_id.to_string()))
        }
    }
    
    /// Verarbeitet eingehende Transfer-Nachrichten
    pub async fn handle_transfer_message(
        &self,
        peer_id: &str,
        message: TransferMessage
    ) -> Result<(), FileTransferError> {
        match message {
            TransferMessage::Request(request) => {
                self.handle_transfer_request(peer_id, request).await
            },
            TransferMessage::Response(response) => {
                self.handle_transfer_response(peer_id, response).await
            },
            TransferMessage::Chunk(chunk) => {
                self.handle_chunk_data(peer_id, chunk).await
            },
            TransferMessage::ChunkRequest(request) => {
                self.handle_chunk_request(peer_id, request).await
            },
            TransferMessage::Control(control) => {
                self.handle_control_message(peer_id, control).await
            }
        }
    }
    
    /// Holt Informationen über eine aktive Übertragung
    pub fn get_transfer_info(&self, transfer_id: &str) -> Option<TransferInfo> {
        let transfers = self.active_transfers.lock().unwrap();
        transfers.get(transfer_id).map(|session| TransferInfo {
            id: session.id.clone(),
            transfer_type: session.transfer_type.clone(),
            peer_id: session.peer_id.clone(),
            status: session.status.clone(),
            file_metadata: session.file_metadata.clone(),
            progress: session.progress.clone(),
            started_at: session.started_at,
            last_activity: session.last_activity,
            retry_count: session.retry_count,
        })
    }
    
    /// Holt alle aktiven Übertragungen
    pub fn get_active_transfers(&self) -> Vec<TransferInfo> {
        let transfers = self.active_transfers.lock().unwrap();
        transfers.values().map(|session| TransferInfo {
            id: session.id.clone(),
            transfer_type: session.transfer_type.clone(),
            peer_id: session.peer_id.clone(),
            status: session.status.clone(),
            file_metadata: session.file_metadata.clone(),
            progress: session.progress.clone(),
            started_at: session.started_at,
            last_activity: session.last_activity,
            retry_count: session.retry_count,
        }).collect()
    }
    
    /// Holt Übertragungsstatistiken
    pub fn get_stats(&self) -> TransferStats {
        self.stats.lock().unwrap().clone()
    }
    
    // Private Hilfsmethoden
    
    /// Berechnet den Hash einer Datei
    async fn calculate_file_hash(&self, file_path: &Path) -> Result<String, FileTransferError> {
        let mut file = File::open(file_path)
            .map_err(|e| FileTransferError::IoError(e.to_string()))?;
        
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; self.config.chunk_size];
        
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => hasher.update(&buffer[..n]),
                Err(e) => return Err(FileTransferError::IoError(e.to_string())),
            }
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// Erkennt den MIME-Typ einer Datei
    fn detect_mime_type(&self, file_path: &Path) -> String {
        // Vereinfachte MIME-Type-Erkennung basierend auf Dateiendung
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            "zip" => "application/zip",
            "json" => "application/json",
            "xml" => "application/xml",
            _ => "application/octet-stream",
        }.to_string()
    }
    
    /// Holt Dateiberechtigungen (vereinfacht)
    fn get_file_permissions(&self, _file_path: &Path) -> u32 {
        // Vereinfachte Implementierung - in einer vollständigen Version
        // würden hier die tatsächlichen Dateiberechtigungen ausgelesen
        0o644
    }
    
    /// Sendet ein Event an das UI
    async fn send_event(&self, event: TransferEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
    
    /// Sendet eine Transfer-Anfrage an einen Peer
    async fn send_transfer_request(
        &self,
        peer_id: &str,
        request: TransferRequest
    ) -> Result<(), FileTransferError> {
        // Hier würde die tatsächliche Netzwerkübertragung implementiert
        // Für jetzt als Platzhalter
        println!("Sending transfer request to {}: {:?}", peer_id, request);
        Ok(())
    }
    
    /// Sendet eine Transfer-Antwort an einen Peer
    async fn send_transfer_response(
        &self,
        transfer_id: &str,
        response: TransferResponse
    ) -> Result<(), FileTransferError> {
        // Hier würde die tatsächliche Netzwerkübertragung implementiert
        println!("Sending transfer response for {}: {:?}", transfer_id, response);
        Ok(())
    }
    
    /// Behandelt eingehende Transfer-Anfragen
    async fn handle_transfer_request(
        &self,
        peer_id: &str,
        request: TransferRequest
    ) -> Result<(), FileTransferError> {
        // Transfer-Session für Download erstellen
        let session = TransferSession {
            id: request.transfer_id.clone(),
            transfer_type: TransferType::Download,
            peer_id: peer_id.to_string(),
            status: TransferStatus::Pending,
            file_metadata: request.file_metadata.clone(),
            file_hash: Some(request.file_hash.clone()),
            source_path: None,
            destination_path: None,
            progress: TransferProgress {
                bytes_transferred: 0,
                total_bytes: request.file_metadata.size,
                chunks_completed: 0,
                total_chunks: request.total_chunks,
                transfer_rate: 0.0,
                eta_seconds: None,
            },
            started_at: Instant::now(),
            last_activity: Instant::now(),
            retry_count: 0,
            chunks: HashMap::new(),
        };
        
        // Session speichern
        {
            let mut transfers = self.active_transfers.lock().unwrap();
            transfers.insert(request.transfer_id.clone(), session);
        }
        
        // Event senden - UI wird Benutzer fragen, ob Transfer akzeptiert werden soll
        self.send_event(TransferEvent::TransferRequested {
            transfer_id: request.transfer_id.clone(),
            peer_id: peer_id.to_string(),
            file_metadata: request.file_metadata,
        }).await;
        
        Ok(())
    }
    
    /// Behandelt Transfer-Antworten
    async fn handle_transfer_response(
        &self,
        _peer_id: &str,
        response: TransferResponse
    ) -> Result<(), FileTransferError> {
        match response {
            TransferResponse::Accept { transfer_id, ready: _ } => {
                // Upload kann beginnen
                self.start_upload_chunks(&transfer_id).await?;
            },
            TransferResponse::Reject { transfer_id, reason } => {
                // Transfer abgebrochen
                self.cancel_transfer(&transfer_id).await?;
                self.send_event(TransferEvent::TransferRejected {
                    transfer_id,
                    reason,
                }).await;
            }
        }
        Ok(())
    }
    
    /// Behandelt eingehende Chunk-Daten
    async fn handle_chunk_data(
        &self,
        _peer_id: &str,
        chunk: ChunkData
    ) -> Result<(), FileTransferError> {
        let mut transfers = self.active_transfers.lock().unwrap();
        
        if let Some(session) = transfers.get_mut(&chunk.transfer_id) {
            // Chunk validieren und speichern
            if let Some(dest_path) = &session.destination_path {
                self.chunk_manager.write_chunk(
                    dest_path,
                    chunk.chunk_index,
                    &chunk.data,
                    chunk.chunk_hash.as_deref()
                ).await?;
                
                // Progress aktualisieren
                session.chunks.insert(chunk.chunk_index, ChunkStatus::Completed);
                session.progress.chunks_completed += 1;
                session.progress.bytes_transferred += chunk.data.len() as u64;
                session.last_activity = Instant::now();
                
                // Transfer-Rate berechnen
                let elapsed = session.started_at.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    session.progress.transfer_rate = session.progress.bytes_transferred as f64 / elapsed;
                    
                    // ETA schätzen
                    let remaining_bytes = session.progress.total_bytes - session.progress.bytes_transferred;
                    if session.progress.transfer_rate > 0.0 {
                        session.progress.eta_seconds = Some(remaining_bytes as f64 / session.progress.transfer_rate);
                    }
                }
                
                // Progress-Event senden
                drop(transfers); // Mutex freigeben vor async
                self.send_event(TransferEvent::TransferProgress {
                    transfer_id: chunk.transfer_id.clone(),
                    progress: session.progress.clone(),
                }).await;
                
                // Prüfen, ob Transfer komplett ist
                if session.progress.chunks_completed >= session.progress.total_chunks {
                    self.complete_download(&chunk.transfer_id).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Behandelt Chunk-Anfragen
    async fn handle_chunk_request(
        &self,
        peer_id: &str,
        request: ChunkRequest
    ) -> Result<(), FileTransferError> {
        let transfers = self.active_transfers.lock().unwrap();
        
        if let Some(session) = transfers.get(&request.transfer_id) {
            if let Some(source_path) = &session.source_path {
                // Chunk lesen und senden
                let chunk_data = self.chunk_manager.read_chunk(
                    source_path,
                    request.chunk_index,
                    self.config.chunk_size
                ).await?;
                
                // Chunk an Peer senden
                self.send_chunk_to_peer(peer_id, ChunkData {
                    transfer_id: request.transfer_id,
                    chunk_index: request.chunk_index,
                    data: chunk_data,
                    chunk_hash: None, // Wird vom ChunkManager berechnet
                }).await?;
            }
        }
        
        Ok(())
    }
    
    /// Behandelt Kontrollnachrichten
    async fn handle_control_message(
        &self,
        _peer_id: &str,
        control: ControlMessage
    ) -> Result<(), FileTransferError> {
        match control {
            ControlMessage::Pause { transfer_id } => {
                self.pause_transfer(&transfer_id).await?;
            },
            ControlMessage::Resume { transfer_id } => {
                self.resume_transfer(&transfer_id).await?;
            },
            ControlMessage::Cancel { transfer_id } => {
                self.cancel_transfer(&transfer_id).await?;
            }
        }
        Ok(())
    }
    
    /// Startet das Senden von Chunks für einen Upload
    async fn start_upload_chunks(&self, transfer_id: &str) -> Result<(), FileTransferError> {
        // Implementation würde hier Chunks in separaten Tasks senden
        // Für jetzt als Platzhalter
        println!("Starting upload chunks for transfer: {}", transfer_id);
        Ok(())
    }
    
    /// Schließt einen Download ab
    async fn complete_download(&self, transfer_id: &str) -> Result<(), FileTransferError> {
        let mut transfers = self.active_transfers.lock().unwrap();
        
        if let Some(session) = transfers.get_mut(transfer_id) {
            // Hash-Verifizierung
            if let Some(dest_path) = &session.destination_path {
                if let Some(expected_hash) = &session.file_hash {
                    let actual_hash = self.calculate_file_hash(dest_path).await?;
                    
                    if actual_hash != *expected_hash {
                        return Err(FileTransferError::HashMismatch {
                            expected: expected_hash.clone(),
                            actual: actual_hash,
                        });
                    }
                }
            }
            
            session.status = TransferStatus::Completed;
            
            // Event senden
            drop(transfers); // Mutex freigeben vor async
            self.send_event(TransferEvent::TransferCompleted {
                transfer_id: transfer_id.to_string(),
            }).await;
            
            // Statistiken aktualisieren
            {
                let mut stats = self.stats.lock().unwrap();
                stats.downloads_completed += 1;
                stats.total_bytes_transferred += session.file_metadata.size;
            }
        }
        
        Ok(())
    }
    
    /// Sendet einen Chunk an einen Peer
    async fn send_chunk_to_peer(
        &self,
        peer_id: &str,
        chunk: ChunkData
    ) -> Result<(), FileTransferError> {
        // Hier würde die tatsächliche Netzwerkübertragung implementiert
        println!("Sending chunk to {}: transfer_id={}, chunk_index={}, size={}", 
                 peer_id, chunk.transfer_id, chunk.chunk_index, chunk.data.len());
        Ok(())
    }
}
