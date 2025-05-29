// src-tauri/src/clipboard/types.rs - Typen für die Zwischenablage-Synchronisation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Typ des Zwischenablage-Inhalts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardContentType {
    Text,
    Image,
    Html,
    Files,
}

/// Metadaten für Zwischenablage-Einträge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMetadata {
    /// Größe der Daten in Bytes
    pub size: usize,
    
    /// MIME-Typ des Inhalts
    pub mime_type: String,
    
    /// Quelle des Eintrags (local, remote, etc.)
    pub source: String,
}

/// Ein Eintrag in der Zwischenablage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// Eindeutige ID des Eintrags
    pub id: String,
    
    /// Art des Inhalts
    pub content_type: ClipboardContentType,
    
    /// Die eigentlichen Daten (Text oder Base64-kodiert für Binärdaten)
    pub data: String,
    
    /// Metadaten
    pub metadata: ClipboardMetadata,
    
    /// Zeitstempel der Erstellung
    pub timestamp: DateTime<Utc>,
}

/// Trait für plattformspezifische Zwischenablage-Implementierungen
pub trait ClipboardProvider: Send + Sync {
    /// Holt Text aus der Zwischenablage
    fn get_text(&mut self) -> Result<String, crate::clipboard::error::ClipboardError>;
    
    /// Setzt Text in die Zwischenablage
    fn set_text(&mut self, text: &str) -> Result<(), crate::clipboard::error::ClipboardError>;
    
    /// Holt Bilddaten aus der Zwischenablage
    fn get_image(&mut self) -> Result<Vec<u8>, crate::clipboard::error::ClipboardError>;
    
    /// Setzt Bilddaten in die Zwischenablage
    fn set_image(&mut self, image_data: &[u8], format: &str) -> Result<(), crate::clipboard::error::ClipboardError>;
    
    /// Holt HTML-Inhalt aus der Zwischenablage
    fn get_html(&mut self) -> Result<String, crate::clipboard::error::ClipboardError> {
        // Standard-Implementierung: Fallback auf Text
        self.get_text()
    }
    
    /// Setzt HTML-Inhalt in die Zwischenablage
    fn set_html(&mut self, html: &str) -> Result<(), crate::clipboard::error::ClipboardError> {
        // Standard-Implementierung: Fallback auf Text
        self.set_text(html)
    }
    
    /// Holt Dateilisten aus der Zwischenablage
    fn get_files(&mut self) -> Result<Vec<String>, crate::clipboard::error::ClipboardError> {
        Err(crate::clipboard::error::ClipboardError::UnsupportedOperation("File clipboard not supported".to_string()))
    }
    
    /// Prüft, ob die Zwischenablage verfügbar ist
    fn is_available(&self) -> bool;
    
    /// Erstellt eine Kopie der Implementierung für Threading
    fn create_clone(&self) -> Box<dyn ClipboardProvider>;
    
    /// Holt die verfügbaren Formate in der Zwischenablage
    fn get_available_formats(&self) -> Vec<String> {
        vec!["text/plain".to_string()]
    }
}

/// Konfiguration für die Zwischenablage-Synchronisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncConfig {
    /// Ob die Synchronisation aktiviert ist
    pub enabled: bool,
    
    /// Maximale Größe für synchronisierte Inhalte (in Bytes)
    pub max_content_size: usize,
    
    /// Ob Bilder synchronisiert werden sollen
    pub sync_images: bool,
    
    /// Ob HTML synchronisiert werden soll
    pub sync_html: bool,
    
    /// Ob Dateien synchronisiert werden sollen
    pub sync_files: bool,
    
    /// Automatische Synchronisation bei Änderungen
    pub auto_sync: bool,
    
    /// Verlaufsgröße
    pub history_size: usize,
}

impl Default for ClipboardSyncConfig {
    fn default() -> Self {
        ClipboardSyncConfig {
            enabled: true,
            max_content_size: 10 * 1024 * 1024, // 10 MB
            sync_images: true,
            sync_html: true,
            sync_files: false, // Aus Sicherheitsgründen standardmäßig deaktiviert
            auto_sync: true,
            history_size: 50,
        }
    }
}

/// Synchronisationsereignis für die Zwischenablage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncEvent {
    /// Art des Ereignisses
    pub event_type: ClipboardSyncEventType,
    
    /// Der betroffene Eintrag
    pub entry: ClipboardEntry,
    
    /// Quelle des Ereignisses
    pub source: String,
    
    /// Zeitstempel des Ereignisses
    pub timestamp: DateTime<Utc>,
}

/// Arten von Synchronisationsereignissen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardSyncEventType {
    /// Neuer Inhalt wurde zur Zwischenablage hinzugefügt
    ContentAdded,
    
    /// Inhalt wurde von einem entfernten Peer empfangen
    ContentReceived,
    
    /// Inhalt wurde an einen entfernten Peer gesendet
    ContentSent,
    
    /// Inhalt wurde aus dem Verlauf gelöscht
    ContentDeleted,
    
    /// Verlauf wurde geleert
    HistoryCleared,
}

/// Statistiken für die Zwischenablage-Synchronisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncStats {
    /// Anzahl der synchronisierten Einträge
    pub entries_synced: u64,
    
    /// Gesamtgröße der synchronisierten Daten
    pub total_bytes_synced: u64,
    
    /// Anzahl der gesendeten Einträge
    pub entries_sent: u64,
    
    /// Anzahl der empfangenen Einträge
    pub entries_received: u64,
    
    /// Anzahl der Synchronisationsfehler
    pub sync_errors: u64,
    
    /// Letzte Synchronisation
    pub last_sync: Option<DateTime<Utc>>,
}

impl Default for ClipboardSyncStats {
    fn default() -> Self {
        ClipboardSyncStats {
            entries_synced: 0,
            total_bytes_synced: 0,
            entries_sent: 0,
            entries_received: 0,
            sync_errors: 0,
            last_sync: None,
        }
    }
}

/// Filter für Zwischenablage-Synchronisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncFilter {
    /// Minimale Textlänge für Synchronisation
    pub min_text_length: usize,
    
    /// Maximale Textlänge für Synchronisation  
    pub max_text_length: usize,
    
    /// Blockierte MIME-Typen
    pub blocked_mime_types: Vec<String>,
    
    /// Erlaubte MIME-Typen (leer = alle erlaubt)
    pub allowed_mime_types: Vec<String>,
    
    /// Blockierte Dateiendungen
    pub blocked_file_extensions: Vec<String>,
    
    /// Regex-Muster für blockierte Inhalte
    pub blocked_content_patterns: Vec<String>,
}

impl Default for ClipboardSyncFilter {
    fn default() -> Self {
        ClipboardSyncFilter {
            min_text_length: 1,
            max_text_length: 1024 * 1024, // 1 MB Text
            blocked_mime_types: vec![
                "application/octet-stream".to_string(),
                "application/x-executable".to_string(),
            ],
            allowed_mime_types: vec![],
            blocked_file_extensions: vec![
                "exe".to_string(),
                "bat".to_string(), 
                "cmd".to_string(),
                "com".to_string(),
                "scr".to_string(),
                "dll".to_string(),
            ],
            blocked_content_patterns: vec![],
        }
    }
}
