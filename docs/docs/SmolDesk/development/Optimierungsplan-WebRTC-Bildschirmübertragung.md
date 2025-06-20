# Optimierungsplan für die WebRTC-Bildschirmübertragung

## 1. Problemanalyse

Die aktuelle Implementierung von SmolDesk hat folgende Schwachstellen:

1. **Ineffiziente Frame-Übertragung**: 
   - Bildschirmframes werden als einzelne Base64-kodierte Bilder übertragen
   - Hoher Overhead durch Base64-Kodierung (~33% Größenzuwachs)
   - Keine Nutzung von Interframe-Kompression wie in echten Videostreams

2. **Fehlende Integration zwischen Backend und WebRTC**:
   - Tauri-Events mit Bildschirmdaten werden nicht effizient in WebRTC-Streams umgewandelt
   - Keine direkte Verbindung zwischen FFmpeg-Output und WebRTC-MediaStream

3. **Keine Anpassung an Netzwerkbedingungen**:
   - Feste Kodierungsparameter unabhängig von verfügbarer Bandbreite
   - Keine Priorisierung wichtiger Bildregionen bei niedrigerer Bandbreite

## 2. Lösungsarchitektur

Wir implementieren eine optimierte Architektur mit folgenden Komponenten:

### Backend (Rust/Tauri)

1. **Kontinuierlicher Videostream statt Einzelbilder**:
   - FFmpeg für Bildschirmerfassung mit optimierten Parametern für niedrige Latenz
   - Implementierung eines Puffersystems zur gleichmäßigen Frame-Lieferung
   - Direkte Ausgabe von kodierten Videodaten ohne Base64-Umwandlung

2. **Hardware-Beschleunigung optimieren**:
   - Verbesserte VAAPI-Integration für Intel-GPUs
   - Optimierte NVENC-Unterstützung für NVIDIA-GPUs
   - Fallback-Mechanismen bei fehlender Hardware-Beschleunigung

3. **Adaptive Kodierung**:
   - Dynamische Anpassung der Kodierungsparameter basierend auf CPU-Auslastung
   - Frame-Skipping-Algorithmus bei hoher CPU-Last
   - Effizienter Codec-Wechsel basierend auf Hardware-Unterstützung

### Frontend (React/TypeScript)

1. **Effiziente Streaming-Pipeline**:
   - Implementierung einer WebCodecs-basierten Dekodierung für moderne Browser
   - Fallback auf MSE (Media Source Extensions) für ältere Browser
   - Pufferstrategien zur Vermeidung von Stottern bei Netzwerkfluktuation

2. **MediaStream-Integration**:
   - Umwandlung der dekodierten Frames in einen kontinuierlichen MediaStream
   - Nutzung von `MediaStreamTrack.captureStream()` für effiziente Weitergabe
   - Integration mit WebRTC-Peer-Verbindungen für Streaming an Clients

3. **Netzwerk-Adaptivität**:
   - Implementierung eines Bitrate-Estimation-Algorithmus
   - Feedback-Mechanismus zum Backend für dynamische Qualitätsanpassung
   - Priorisierung niedriger Latenz über Bildqualität (konfigurierbar)

## 3. Implementierungsplan

### Phase 1: Backend-Optimierung

1. Refactoring der `screen_capture.rs`:
   - Optimierung des FFmpeg-Kommandos für kontinuierliche Streams
   - Implementierung eines effizienteren Frame-Puffers
   - Bessere Fehlerbehandlung und Wiederherstellung bei Abstürzen

2. Tauri-Ereignis-Optimierung:
   - Komprimierte Binärdatenübertragung statt Base64
   - Chunk-basierte Übertragung für große Frames
   - Priorisierung der Übertragungskanäle für wichtige Daten

### Phase 2: Frontend-Integration

1. Streaming-Komponenten überarbeiten:
   - Implementierung einer `StreamReceiver`-Klasse für Tauri-Events
   - Integration mit WebCodecs für effiziente Dekodierung
   - Erstellung eines synthetischen MediaStreams aus dekodierten Frames

2. WebRTC-Optimierung:
   - Verbesserung der Verbindungsherstellung und ICE-Kandidaten-Verhandlung
   - Implementierung von Bandwidth Estimation für adaptives Streaming
   - Optimierte Datenkanäle für Steuerungsinformationen

### Phase 3: End-to-End-Optimierung

1. Latenzoptimierung:
   - Messung und Verfolgung der End-to-End-Latenz
   - Identifizierung und Beseitigung von Engpässen
   - Implementierung von Latenz-Metriken im UI

2. Qualitätsverbesserungen:
   - Dynamische Anpassung der Bildrate vs. Auflösung basierend auf Inhalt
   - Regionsbasierte Kodierung für relevante Bildschirmbereiche
   - Optimierung für Text-/Code-Lesbarkeit vs. Videowiedergabe

## 4. Messung und Erfolgsmetriken

- **Ziel-Latenz**: <200ms End-to-End
- **CPU-Auslastung**: <15% auf modernen Systemen
- **Adaptivität**: Funktionsfähig von 256Kbps bis 10Mbps Bandbreite
- **Wiederherstellungszeit**: <2s nach Netzwerkunterbrechungen

## 5. Nächste Schritte

1. Implementierung eines Proof-of-Concept für kontinuierliche Streaming-Pipeline
2. Benchmarking verschiedener Kodierungs- und Übertragungsstrategien
3. Prototyp für adaptive Qualitätsanpassung basierend auf Netzwerkbedingungen
