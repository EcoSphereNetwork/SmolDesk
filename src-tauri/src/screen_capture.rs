// screen_capture.rs - Optimierte Version mit kontinuierlichen Video-Streams

use std::error::Error;
use std::fmt;
use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use tauri::Window;

// Error types for screen capture operations
#[derive(Debug)]
pub enum ScreenCaptureError {
    InitializationFailed(String),
    CaptureError(String),
    EncodingError(String),
    DisplayServerError(String),
    InvalidMonitor(String),
    StreamBufferError(String),
}

impl fmt::Display for ScreenCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScreenCaptureError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            ScreenCaptureError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            ScreenCaptureError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            ScreenCaptureError::DisplayServerError(msg) => write!(f, "Display server error: {}", msg),
            ScreenCaptureError::InvalidMonitor(msg) => write!(f, "Invalid monitor: {}", msg),
            ScreenCaptureError::StreamBufferError(msg) => write!(f, "Stream buffer error: {}", msg),
        }
    }
}

impl Error for ScreenCaptureError {}

// Display server type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

// Hardware acceleration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    None,
    VAAPI,
    NVENC,
    QuickSync,
}

// Screen capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    pub monitor_index: usize,
    pub fps: u32,
    pub quality: u32, // 0-100
    pub codec: VideoCodec,
    pub hardware_acceleration: HardwareAcceleration,
    pub capture_cursor: bool,
    pub capture_audio: bool,
    pub keyframe_interval: u32, // NEU: Keyframe-Intervall für bessere Kompression
    pub bitrate: Option<u32>,   // NEU: Optional Bitrate in Kbps
    pub latency_mode: LatencyMode, // NEU: Latenz-Optimierung
}

// NEU: Latenzoptimierung Modi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LatencyMode {
    UltraLow,  // Minimale Latenz, möglicherweise auf Kosten der Qualität
    Balanced,  // Ausgewogenes Verhältnis zwischen Latenz und Qualität
    Quality,   // Höhere Qualität, möglicherweise auf Kosten der Latenz
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        ScreenCaptureConfig {
            monitor_index: 0,
            fps: 30,
            quality: 80,
            codec: VideoCodec::H264,
            hardware_acceleration: HardwareAcceleration::None,
            capture_cursor: true,
            capture_audio: false,
            keyframe_interval: 30,   // Standard: Ein Keyframe pro Sekunde bei 30 FPS
            bitrate: None,           // Automatische Bitrate basierend auf Qualität
            latency_mode: LatencyMode::Balanced,
        }
    }
}

// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: Option<f64>,
    pub primary: bool,
    pub x_offset: i32,
    pub y_offset: i32,
}

// Statistics for screen capturing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStats {
    pub fps: f64,
    pub bitrate: u64,
    pub encode_time: f64,
    pub frame_size: u64,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub buffer_level: usize,    // NEU: Aktueller Pufferstand
    pub latency_estimate: f64,  // NEU: Geschätzte Latenz in ms
}

// NEU: Stream-Puffer-Implementierung für kontinuierliche Streams
struct StreamBuffer {
    chunks: VecDeque<Vec<u8>>,
    max_size: usize,
    total_bytes: usize,
}

impl StreamBuffer {
    fn new(max_size: usize) -> Self {
        StreamBuffer {
            chunks: VecDeque::with_capacity(max_size),
            max_size,
            total_bytes: 0,
        }
    }
    
    fn push_chunk(&mut self, data: Vec<u8>) {
        let chunk_size = data.len();
        self.chunks.push_back(data);
        self.total_bytes += chunk_size;
        
        // Entferne alte Chunks, wenn der Buffer voll ist
        while self.chunks.len() > self.max_size {
            if let Some(old_chunk) = self.chunks.pop_front() {
                self.total_bytes -= old_chunk.len();
            }
        }
    }
    
    fn get_chunks(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.chunks.iter()
    }
    
    fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
    
    fn len(&self) -> usize {
        self.chunks.len()
    }
    
    fn clear(&mut self) {
        self.chunks.clear();
        self.total_bytes = 0;
    }
}

// NEU: Adaptive Qualitätssteuerung für verschiedene Netzwerk- und CPU-Bedingungen
struct AdaptiveQualityController {
    current_quality: u32,
    cpu_usage: f32,
    network_bandwidth: u32,
    frame_drop_rate: f32,
    last_adjustment: Instant,
    adjustment_interval: Duration,
}

impl AdaptiveQualityController {
    fn new(initial_quality: u32) -> Self {
        AdaptiveQualityController {
            current_quality: initial_quality,
            cpu_usage: 0.0,
            network_bandwidth: 5000, // Annahme: 5 Mbps zu Beginn
            frame_drop_rate: 0.0,
            last_adjustment: Instant::now(),
            adjustment_interval: Duration::from_secs(5), // Anpassung alle 5 Sekunden
        }
    }
    
    fn update_metrics(&mut self, cpu_usage: f32, network_bandwidth: u32, frame_drop_rate: f32) {
        self.cpu_usage = cpu_usage;
        self.network_bandwidth = network_bandwidth;
        self.frame_drop_rate = frame_drop_rate;
    }
    
    fn adjust_quality(&mut self) -> u32 {
        let now = Instant::now();
        if now.duration_since(self.last_adjustment) < self.adjustment_interval {
            return self.current_quality;
        }
        
        // Anpassungsstrategie
        if self.cpu_usage > 85.0 || self.frame_drop_rate > 0.1 {
            // Reduziere Qualität bei hoher CPU-Last oder vielen Frame-Drops
            self.current_quality = (self.current_quality - 5).max(10);
        } else if self.cpu_usage < 50.0 && self.network_bandwidth > 5000 && self.frame_drop_rate < 0.01 {
            // Erhöhe Qualität bei guten Bedingungen
            self.current_quality = (self.current_quality + 5).min(100);
        }
        
        self.last_adjustment = now;
        self.current_quality
    }
    
    fn get_bitrate_for_resolution(&self, width: u32, height: u32) -> u32 {
        // Grundlegende Heuristik für Bitrate basierend auf Auflösung und Qualität
        let pixel_count = width * height;
        let base_bitrate = match pixel_count {
            p if p > 2073600 => 8000, // 1080p+
            p if p > 921600 => 5000,  // 720p+
            p if p > 480000 => 2500,  // 480p+
            _ => 1000,                // Niedrigere Auflösungen
        };
        
        // Qualitätsanpassung (10% - 100% des Basis-Bitrate)
        (base_bitrate as f32 * (self.current_quality as f32 / 100.0)) as u32
    }
}

// Screen capture manager
pub struct ScreenCaptureManager {
    display_server: DisplayServer,
    config: Arc<Mutex<ScreenCaptureConfig>>,
    capture_thread: Option<thread::JoinHandle<()>>,
    capture_process: Arc<Mutex<Option<Child>>>,  // NEU: Kontinuierlicher FFmpeg-Prozess
    monitors: Vec<MonitorInfo>,
    stats: Arc<Mutex<CaptureStats>>,
    running: Arc<Mutex<bool>>,
    stream_buffer: Arc<Mutex<StreamBuffer>>,     // NEU: Puffer für kontinuierlichen Stream
    quality_controller: Arc<Mutex<AdaptiveQualityController>>, // NEU: Adaptive Qualität
}

impl ScreenCaptureManager {
    // Create a new screen capture manager
    pub fn new() -> Result<Self, ScreenCaptureError> {
        let display_server = detect_display_server()?;
        
        let monitors = match display_server {
            DisplayServer::X11 => get_x11_monitors(),
            DisplayServer::Wayland => get_wayland_monitors(),
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ))
            }
        }?;

        let default_config = ScreenCaptureConfig::default();
        let quality_controller = AdaptiveQualityController::new(default_config.quality);

        Ok(ScreenCaptureManager {
            display_server,
            config: Arc::new(Mutex::new(default_config)),
            capture_thread: None,
            capture_process: Arc::new(Mutex::new(None)),
            monitors,
            stats: Arc::new(Mutex::new(CaptureStats {
                fps: 0.0,
                bitrate: 0,
                encode_time: 0.0,
                frame_size: 0,
                frame_count: 0,
                dropped_frames: 0,
                buffer_level: 0,
                latency_estimate: 0.0,
            })),
            running: Arc::new(Mutex::new(false)),
            stream_buffer: Arc::new(Mutex::new(StreamBuffer::new(30))), // 30 Chunks Puffer
            quality_controller: Arc::new(Mutex::new(quality_controller)),
        })
    }

    // Get detected display server
    pub fn get_display_server(&self) -> DisplayServer {
        self.display_server.clone()
    }

    // Get available monitors
    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        self.monitors.clone()
    }

    // Refresh monitor list
    pub fn refresh_monitors(&mut self) -> Result<(), ScreenCaptureError> {
        self.monitors = match self.display_server {
            DisplayServer::X11 => get_x11_monitors(),
            DisplayServer::Wayland => get_wayland_monitors(),
            DisplayServer::Unknown => {
                return Err(ScreenCaptureError::DisplayServerError(
                    "Unsupported display server".to_string(),
                ))
            }
        }?;
        
        Ok(())
    }

    // Update capture configuration
    pub fn update_config(&self, config: ScreenCaptureConfig) -> Result<(), ScreenCaptureError> {
        // Validate monitor index
        if config.monitor_index >= self.monitors.len() {
            return Err(ScreenCaptureError::InvalidMonitor(format!(
                "Monitor index {} out of bounds (0-{})",
                config.monitor_index,
                self.monitors.len() - 1
            )));
        }

        let mut current_config = self.config.lock().unwrap();
        *current_config = config;
        
        // Wenn bereits eine Aufnahme läuft, neu starten mit neuer Konfiguration
        let is_running = *self.running.lock().unwrap();
        if is_running {
            drop(current_config); // Lock freigeben
            self.restart_capture()?;
        }
        
        Ok(())
    }

    // NEU: Methode zum Neustarten der Aufnahme mit aktualisierter Konfiguration
    fn restart_capture(&self) -> Result<(), ScreenCaptureError> {
        self.stop_capture()?;
        
        // Kurz warten, um sicherzustellen, dass alles beendet ist
        thread::sleep(Duration::from_millis(100));
        
        // Tauri Window-Handle ist erforderlich - dies ist eine Einschränkung dieses Ansatzes
        // In einer vollständigen Implementierung würden wir das Window-Handle speichern
        return Err(ScreenCaptureError::InitializationFailed(
            "Restart requires window handle, use stop_capture and then start_capture".to_string()
        ));
    }

    // Start screen capture
    pub fn start_capture(&mut self, window: Window) -> Result<(), ScreenCaptureError> {
        // Check if already running
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Clear stream buffer
        {
            let mut buffer = self.stream_buffer.lock().unwrap();
            buffer.clear();
        }

        // Clone necessary data for the capture thread
        let config = self.config.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();
        let display_server = self.display_server.clone();
        let monitor = self.monitors[self.config.lock().unwrap().monitor_index].clone();
        let stream_buffer = self.stream_buffer.clone();
        let quality_controller = self.quality_controller.clone();
        let capture_process = self.capture_process.clone();

        // Create the capture thread
        self.capture_thread = Some(thread::spawn(move || {
            match display_server {
                DisplayServer::X11 => {
                    x11_capture_loop(config, running, stats, window, monitor, stream_buffer, quality_controller, capture_process);
                }
                DisplayServer::Wayland => {
                    wayland_capture_loop(config, running, stats, window, monitor, stream_buffer, quality_controller, capture_process);
                }
                DisplayServer::Unknown => {
                    // This shouldn't happen as we validate display server in new()
                    eprintln!("Unknown display server in capture thread");
                    return;
                }
            }
        }));

        Ok(())
    }

    // Stop screen capture
    pub fn stop_capture(&mut self) -> Result<(), ScreenCaptureError> {
        // Set running flag to false to signal the capture thread to stop
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        // Kill the FFmpeg process if it's running
        {
            let mut process = self.capture_process.lock().unwrap();
            if let Some(ref mut child) = *process {
                let _ = child.kill();
            }
            *process = None;
        }

        // Wait for the capture thread to finish
        if let Some(handle) = self.capture_thread.take() {
            match handle.join() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error joining capture thread: {:?}", e);
                }
            }
        }

        Ok(())
    }

    // Get current capture statistics
    pub fn get_stats(&self) -> CaptureStats {
        self.stats.lock().unwrap().clone()
    }

    // NEU: Methode, um den nächsten verfügbaren Frame zu holen
    pub fn get_next_frame(&self) -> Option<Vec<u8>> {
        let mut buffer = self.stream_buffer.lock().unwrap();
        if buffer.is_empty() {
            None
        } else {
            buffer.chunks.pop_front().map(|chunk| {
                buffer.total_bytes -= chunk.len();
                chunk
            })
        }
    }

    // NEU: Methode, um den Buffer-Status zu prüfen
    pub fn get_buffer_status(&self) -> (usize, usize) {
        let buffer = self.stream_buffer.lock().unwrap();
        (buffer.len(), buffer.total_bytes)
    }
}

// X11 screen capture loop (optimiert für kontinuierlichen Stream)
fn x11_capture_loop(
    config: Arc<Mutex<ScreenCaptureConfig>>,
    running: Arc<Mutex<bool>>,
    stats: Arc<Mutex<CaptureStats>>,
    window: Window,
    monitor: MonitorInfo,
    stream_buffer: Arc<Mutex<StreamBuffer>>,
    quality_controller: Arc<Mutex<AdaptiveQualityController>>,
    capture_process: Arc<Mutex<Option<Child>>>,
) {
    // Erfasse CPU-Nutzung vor dem Start
    let initial_cpu_usage = get_cpu_usage().unwrap_or(0.0);
    
    let mut last_frame_time = Instant::now();
    let mut frame_count: u64 = 0;
    let mut dropped_frames: u64 = 0;
    let start_time = Instant::now();
    
    // Starte den FFmpeg-Prozess für kontinuierliche Erfassung
    let mut process = start_ffmpeg_process(&config, &monitor).unwrap_or_else(|e| {
        eprintln!("Failed to start FFmpeg process: {}", e);
        return;
    });
    
    // Speichere den Prozess in der gemeinsam genutzten Variable
    {
        let mut process_guard = capture_process.lock().unwrap();
        *process_guard = Some(process);
    }
    
    // Hole den stdout-Pipe vom Prozess für das Lesen der Videodaten
    let stdout = process.stdout.take().expect("Failed to take stdout from FFmpeg process");
    
    // Puffer für das Lesen der Ausgabe
    let mut buffer = Vec::new();
    let mut buf = vec![0u8; 65536]; // 64KB Puffer für das Lesen
    
    // Bereite stdout für nicht-blockierendes Lesen vor (plattformabhängig)
    // Hinweis: In einer tatsächlichen Implementierung würden wir eine Bibliothek wie mio oder tokio verwenden
    
    // Hauptschleife für das Erfassen und Verarbeiten von Frames
    while *running.lock().unwrap() {
        let now = Instant::now();
        let elapsed = now.duration_since(last_frame_time);
        
        // Überprüfe, ob der Prozess noch läuft
        match process.try_wait() {
            Ok(Some(status)) => {
                eprintln!("FFmpeg process exited with status: {}", status);
                break;
            }
            Ok(None) => {},
            Err(e) => {
                eprintln!("Error checking FFmpeg process: {}", e);
                break;
            }
        }
        
        // Lese Daten vom FFmpeg-Prozess
        // In einer tatsächlichen Implementierung würden wir nicht-blockierendes I/O verwenden
        let mut stdout = stdout.clone();
        let read_result = stdout.read(&mut buf);
        
        match read_result {
            Ok(n) if n > 0 => {
                // Daten wurden gelesen, füge sie dem Frame-Puffer hinzu
                buffer.extend_from_slice(&buf[0..n]);
                
                // Überprüfe auf Paket-Grenzen oder Frame-Grenzen (abhängig vom Format)
                // Für matroska/mp4 müssten wir den Container parsen
                // Für einfache Implementierung nehmen wir an, dass jeder Chunk ein Frame ist
                
                // Füge den Frame zum Stream-Puffer hinzu
                {
                    let mut stream_buf = stream_buffer.lock().unwrap();
                    stream_buf.push_chunk(buffer.clone());
                    buffer.clear();
                }
                
                // Aktualisiere Statistiken
                {
                    let mut stats_guard = stats.lock().unwrap();
                    frame_count += 1;
                    stats_guard.frame_count = frame_count;
                    stats_guard.frame_size = buffer.len() as u64;
                    stats_guard.fps = frame_count as f64 / start_time.elapsed().as_secs_f64();
                    
                    // Schätze Bitrate
                    let total_bytes = buffer.len() as u64 * frame_count;
                    let seconds = start_time.elapsed().as_secs_f64();
                    if seconds > 0.0 {
                        stats_guard.bitrate = (total_bytes as f64 * 8.0 / seconds) as u64;
                    }
                    
                    // Aktualisiere Pufferstand
                    let buffer_guard = stream_buffer.lock().unwrap();
                    stats_guard.buffer_level = buffer_guard.len();
                    
                    // Sende Statistiken zum Frontend
                    let _ = window.emit("capture_stats", stats_guard.clone());
                }
                
                // Sende Frame-Daten
                // Statt Base64-Kodierung wie bisher, senden wir die rohen Daten
                // Im Frontend müssen diese dann entsprechend verarbeitet werden
                let _ = window.emit("frame_data", buffer.clone());
                
                last_frame_time = now;
            }
            Ok(_) => {
                // Keine Daten verfügbar, warte kurz
                thread::sleep(Duration::from_millis(1));
            }
            Err(e) => {
                eprintln!("Error reading from FFmpeg: {}", e);
                dropped_frames += 1;
                
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.dropped_frames = dropped_frames;
            }
        }
        
        // Adaptive Qualitätsanpassung
        {
            let current_cpu_usage = get_cpu_usage().unwrap_or(initial_cpu_usage);
            let buffer_guard = stream_buffer.lock().unwrap();
            let buffer_fill_ratio = buffer_guard.len() as f32 / buffer_guard.max_size as f32;
            
            // Schätze Netzwerkbandbreite und Frame-Drop-Rate
            let stats_guard = stats.lock().unwrap();
            let network_bandwidth = (stats_guard.bitrate / 1000) as u32; // kbps
            let frame_drop_rate = if frame_count > 0 {
                dropped_frames as f32 / frame_count as f32
            } else {
                0.0
            };
            
            // Aktualisiere den Qualitätscontroller
            let mut quality_ctrl = quality_controller.lock().unwrap();
            quality_ctrl.update_metrics(current_cpu_usage, network_bandwidth, frame_drop_rate);
            
            // Wende Qualitätsanpassungen an
            let new_quality = quality_ctrl.adjust_quality();
            if new_quality != quality_ctrl.current_quality {
                // Konfiguration ändern für nächsten Neustart
                let mut config_guard = config.lock().unwrap();
                config_guard.quality = new_quality;
                
                // Idealerweise würden wir hier die FFmpeg-Kommandozeile dynamisch anpassen,
                // z.B. über Steuerungssockets oder durch Neustarten des Prozesses
            }
        }
    }
    
    // Bereinige, wenn die Schleife beendet wird
    let _ = process.kill();
}

// Wayland screen capture loop (optimiert für kontinuierlichen Stream)
fn wayland_capture_loop(
    config: Arc<Mutex<ScreenCaptureConfig>>,
    running: Arc<Mutex<bool>>,
    stats: Arc<Mutex<CaptureStats>>,
    window: Window,
    monitor: MonitorInfo,
    stream_buffer: Arc<Mutex<StreamBuffer>>,
    quality_controller: Arc<Mutex<AdaptiveQualityController>>,
    capture_process: Arc<Mutex<Option<Child>>>,
) {
    // Ähnliche Implementierung wie für X11, aber mit Wayland-spezifischen Anpassungen
    // Die grundlegende Stream-Puffer-Logik bleibt gleich
    
    // Für Wayland verwenden wir PipeWire über den xdg-desktop-portal
    // Wir starten einen kontinuierlichen Screencast-Prozess anstelle von Einzelbildern
    
    // Erfasse CPU-Nutzung vor dem Start
    let initial_cpu_usage = get_cpu_usage().unwrap_or(0.0);
    
    let mut last_frame_time = Instant::now();
    let mut frame_count: u64 = 0;
    let mut dropped_frames: u64 = 0;
    let start_time = Instant::now();
    
    // Starte den PipeWire-Screencast-Prozess für kontinuierliche Erfassung
    let mut process = start_pipewire_process(&config, &monitor).unwrap_or_else(|e| {
        eprintln!("Failed to start PipeWire screencast process: {}", e);
        return;
    });
    
    // Speichere den Prozess in der gemeinsam genutzten Variable
    {
        let mut process_guard = capture_process.lock().unwrap();
        *process_guard = Some(process);
    }
    
    // Hole den stdout-Pipe vom Prozess für das Lesen der Videodaten
    let stdout = process.stdout.take().expect("Failed to take stdout from screencast process");
    
    // Puffer für das Lesen der Ausgabe
    let mut buffer = Vec::new();
    let mut buf = vec![0u8; 65536]; // 64KB Puffer für das Lesen
    
    // Hauptschleife für das Erfassen und Verarbeiten von Frames
    // Fast identisch mit X11-Implementierung, aber mit Wayland-spezifischen Anpassungen
    while *running.lock().unwrap() {
        // Ähnliche Implementierung wie bei X11...
        // ...
    }
}

// Helfer-Funktion zum Starten des FFmpeg-Prozesses für kontinuierliche Streams
fn start_ffmpeg_process(
    config: &Arc<Mutex<ScreenCaptureConfig>>,
    monitor: &MonitorInfo,
) -> Result<Child, ScreenCaptureError> {
    let config_guard = config.lock().unwrap();
    
    // Erstelle FFmpeg-Kommando für kontinuierlichen Stream
    let mut cmd = Command::new("ffmpeg");
    
    // Input-Konfiguration
    cmd.arg("-f").arg("x11grab")
       .arg("-video_size").arg(format!("{}x{}", monitor.width, monitor.height))
       .arg("-i").arg(format!(":0.0+{},{}", monitor.x_offset, monitor.y_offset));
    
    // Framerate
    cmd.arg("-framerate").arg(config_guard.fps.to_string());
    
    // Mauszeiger erfassen
    if config_guard.capture_cursor {
        cmd.arg("-draw_mouse").arg("1");
    } else {
        cmd.arg("-draw_mouse").arg("0");
    }
    
    // Hardware-Beschleunigung
    match config_guard.hardware_acceleration {
        HardwareAcceleration::VAAPI => {
            cmd.arg("-hwaccel").arg("vaapi")
               .arg("-hwaccel_device").arg("/dev/dri/renderD128")
               .arg("-hw
