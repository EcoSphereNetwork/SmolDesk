// src-tauri/src/clipboard/mod.rs - Zwischenablage-Synchronisation System

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

pub mod types;
pub mod x11_clipboard;
pub mod wayland_clipboard;
pub mod error;

use types::*;
use error::ClipboardError;

/// Zwischenablage-Manager für SmolDesk
pub struct ClipboardManager {
    /// Aktuelle Zwischenablage-Implementierung
    clipboard_impl: Box<dyn ClipboardProvider>,
    
    /// Lokaler Zwischenablage-Verlauf
    history: Arc<Mutex<Vec<ClipboardEntry>>>,
    
    /// Maximale Anzahl von Einträgen im Verlauf
    max_history_size: usize,
    
    /// Callback-Funktionen für Änderungen
    change_callbacks: Arc<Mutex<Vec<Box<dyn Fn(&ClipboardEntry) + Send + Sync>>>>,
    
    /// Überwachungs-Thread
    monitor_thread: Option<thread::JoinHandle<()>>,
    
    /// Überwachung aktiv
    monitoring: Arc<Mutex<bool>>,
    
    /// Letzter bekannter Zwischenablage-Inhalt (für Änderungserkennung)
    last_content: Arc<Mutex<Option<String>>>,
}

impl ClipboardManager {
    /// Erstellt einen neuen ClipboardManager
    pub fn new(display_server: crate::screen_capture::types::DisplayServer) -> Result<Self, ClipboardError> {
        let clipboard_impl: Box<dyn ClipboardProvider> = match display_server {
            crate::screen_capture::types::DisplayServer::X11 => {
                Box::new(x11_clipboard::X11ClipboardProvider::new()?)
            },
            crate::screen_capture::types::DisplayServer::Wayland => {
                Box::new(wayland_clipboard::WaylandClipboardProvider::new()?)
            },
            crate::screen_capture::types::DisplayServer::Unknown => {
                return Err(ClipboardError::UnsupportedPlatform("Unknown display server".to_string()));
            }
        };
        
        Ok(ClipboardManager {
            clipboard_impl,
            history: Arc::new(Mutex::new(Vec::new())),
            max_history_size: 50, // Maximal 50 Einträge im Verlauf
            change_callbacks: Arc::new(Mutex::new(Vec::new())),
            monitor_thread: None,
            monitoring: Arc::new(Mutex::new(false)),
            last_content: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Startet die Überwachung der Zwischenablage
    pub fn start_monitoring(&mut self) -> Result<(), ClipboardError> {
        // Prüfen, ob bereits überwacht wird
        {
            let monitoring = self.monitoring.lock().unwrap();
            if *monitoring {
                return Ok(()); // Bereits aktiv
            }
        }
        
        // Überwachung aktivieren
        {
            let mut monitoring = self.monitoring.lock().unwrap();
            *monitoring = true;
        }
        
        // Überwachungs-Thread starten
        let monitoring_flag = self.monitoring.clone();
        let history = self.history.clone();
        let callbacks = self.change_callbacks.clone();
        let last_content = self.last_content.clone();
        let max_history = self.max_history_size;
        
        // Clone der Implementierung für den Thread
        let mut clipboard_impl = self.clipboard_impl.create_clone();
        
        self.monitor_thread = Some(thread::spawn(move || {
            let mut poll_interval = Duration::from_millis(500); // Standard: alle 500ms prüfen
            
            while *monitoring_flag.lock().unwrap() {
                // Versuche aktuelle Zwischenablage zu lesen
                match clipboard_impl.get_text() {
                    Ok(current_content) => {
                        let mut should_notify = false;
                        let mut new_entry: Option<ClipboardEntry> = None;
                        
                        // Prüfen, ob sich der Inhalt geändert hat
                        {
                            let mut last = last_content.lock().unwrap();
                            if let Some(ref last_text) = *last {
                                if last_text != &current_content {
                                    should_notify = true;
                                }
                            } else if !current_content.is_empty() {
                                should_notify = true;
                            }
                            
                            if should_notify {
                                *last = Some(current_content.clone());
                                
                                new_entry = Some(ClipboardEntry {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    content_type: ClipboardContentType::Text,
                                    data: current_content.clone(),
                                    metadata: ClipboardMetadata {
                                        size: current_content.len(),
                                        mime_type: "text/plain".to_string(),
                                        source: "local".to_string(),
                                    },
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                        
                        // Neuen Eintrag zum Verlauf hinzufügen
                        if let Some(entry) = new_entry {
                            {
                                let mut hist = history.lock().unwrap();
                                hist.push(entry.clone());
                                
                                // Verlauf begrenzen
                                if hist.len() > max_history {
                                    hist.remove(0);
                                }
                            }
                            
                            // Callbacks benachrichtigen
                            let callbacks_guard = callbacks.lock().unwrap();
                            for callback in callbacks_guard.iter() {
                                callback(&entry);
                            }
                        }
                        
                        // Normale Polling-Rate
                        poll_interval = Duration::from_millis(500);
                    },
                    Err(ClipboardError::EmptyClipboard) => {
                        // Zwischenablage ist leer, das ist normal
                        poll_interval = Duration::from_millis(1000); // Weniger häufig prüfen
                    },
                    Err(_) => {
                        // Fehler beim Lesen, etwas langsamer versuchen
                        poll_interval = Duration::from_millis(2000);
                    }
                }
                
                thread::sleep(poll_interval);
            }
        }));
        
        Ok(())
    }
    
    /// Stoppt die Überwachung der Zwischenablage
    pub fn stop_monitoring(&mut self) {
        // Überwachung deaktivieren
        {
            let mut monitoring = self.monitoring.lock().unwrap();
            *monitoring = false;
        }
        
        // Thread beenden
        if let Some(handle) = self.monitor_thread.take() {
            let _ = handle.join();
        }
    }
    
    /// Holt den aktuellen Text aus der Zwischenablage
    pub fn get_text(&mut self) -> Result<String, ClipboardError> {
        self.clipboard_impl.get_text()
    }
    
    /// Setzt Text in die Zwischenablage
    pub fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.clipboard_impl.set_text(text)?;
        
        // Lokalen Cache aktualisieren
        {
            let mut last = self.last_content.lock().unwrap();
            *last = Some(text.to_string());
        }
        
        Ok(())
    }
    
    /// Holt Bilddaten aus der Zwischenablage
    pub fn get_image(&mut self) -> Result<Vec<u8>, ClipboardError> {
        self.clipboard_impl.get_image()
    }
    
    /// Setzt Bilddaten in die Zwischenablage
    pub fn set_image(&mut self, image_data: &[u8], format: &str) -> Result<(), ClipboardError> {
        self.clipboard_impl.set_image(image_data, format)
    }
    
    /// Holt den Zwischenablage-Verlauf
    pub fn get_history(&self) -> Vec<ClipboardEntry> {
        let history = self.history.lock().unwrap();
        history.clone()
    }
    
    /// Löscht den Zwischenablage-Verlauf
    pub fn clear_history(&self) {
        let mut history = self.history.lock().unwrap();
        history.clear();
    }
    
    /// Fügt einen Callback für Änderungen hinzu
    pub fn add_change_callback<F>(&self, callback: F) 
    where 
        F: Fn(&ClipboardEntry) + Send + Sync + 'static 
    {
        let mut callbacks = self.change_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }
    
    /// Exportiert einen Verlaufseintrag als JSON
    pub fn export_entry(&self, entry_id: &str) -> Result<String, ClipboardError> {
        let history = self.history.lock().unwrap();
        
        if let Some(entry) = history.iter().find(|e| e.id == entry_id) {
            serde_json::to_string(entry)
                .map_err(|e| ClipboardError::SerializationError(e.to_string()))
        } else {
            Err(ClipboardError::EntryNotFound(entry_id.to_string()))
        }
    }
    
    /// Importiert einen Verlaufseintrag aus JSON
    pub fn import_entry(&self, json_data: &str) -> Result<(), ClipboardError> {
        let entry: ClipboardEntry = serde_json::from_str(json_data)
            .map_err(|e| ClipboardError::SerializationError(e.to_string()))?;
        
        let mut history = self.history.lock().unwrap();
        history.push(entry);
        
        // Verlauf begrenzen
        if history.len() > self.max_history_size {
            history.remove(0);
        }
        
        Ok(())
    }
    
    /// Synchronisiert mit einem entfernten Zwischenablage-Eintrag
    pub fn sync_remote_entry(&mut self, entry: ClipboardEntry) -> Result<(), ClipboardError> {
        // Lokale Zwischenablage aktualisieren
        match entry.content_type {
            ClipboardContentType::Text => {
                self.set_text(&entry.data)?;
            },
            ClipboardContentType::Image => {
                let image_data = general_purpose::STANDARD.decode(&entry.data)
                    .map_err(|e| ClipboardError::DecodingError(e.to_string()))?;
                self.set_image(&image_data, &entry.metadata.mime_type)?;
            },
            ClipboardContentType::Html => {
                // HTML als Text behandeln für jetzt
                self.set_text(&entry.data)?;
            },
            ClipboardContentType::Files => {
                // Dateien können nicht direkt in die Zwischenablage gesetzt werden
                return Err(ClipboardError::UnsupportedOperation("Cannot set files to clipboard".to_string()));
            }
        }
        
        // Zum Verlauf hinzufügen
        {
            let mut history = self.history.lock().unwrap();
            
            // Prüfen, ob bereits vorhanden (Duplikate vermeiden)
            if !history.iter().any(|e| e.id == entry.id) {
                history.push(entry);
                
                // Verlauf begrenzen
                if history.len() > self.max_history_size {
                    history.remove(0);
                }
            }
        }
        
        Ok(())
    }
    
    /// Erstellt eine kompakte Repräsentation für die Netzwerkübertragung
    pub fn create_sync_entry(&self, entry: &ClipboardEntry) -> Result<String, ClipboardError> {
        // Für große Daten Base64-Kodierung verwenden
        let sync_entry = SyncClipboardEntry {
            id: entry.id.clone(),
            content_type: entry.content_type.clone(),
            data: match entry.content_type {
                ClipboardContentType::Image => {
                    // Bilddaten sind bereits Base64-kodiert
                    entry.data.clone()
                },
                _ => {
                    // Text-Daten Base64-kodieren für sichere Übertragung
                    general_purpose::STANDARD.encode(&entry.data)
                }
            },
            metadata: entry.metadata.clone(),
            timestamp: entry.timestamp,
        };
        
        serde_json::to_string(&sync_entry)
            .map_err(|e| ClipboardError::SerializationError(e.to_string()))
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}

// Vereinfachte Sync-Struktur für Netzwerkübertragung
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncClipboardEntry {
    pub id: String,
    pub content_type: ClipboardContentType,
    pub data: String, // Immer Base64-kodiert für Sync
    pub metadata: ClipboardMetadata,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
