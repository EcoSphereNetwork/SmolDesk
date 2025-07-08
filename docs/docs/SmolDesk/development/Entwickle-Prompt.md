# Entwickler-Prompt: SmolDesk - Weiterentwicklung des WebRTC-basierten Remote-Desktop-Tools

## Projektübersicht

Du übernimmst die Weiterentwicklung von SmolDesk, einem WebRTC-basierten Remote-Desktop-Tool für Linux, das sich aktuell in der Implementierungsphase befindet. Das Projekt basiert auf einer React/TypeScript-Frontend- und Rust/Tauri-Backend-Architektur und soll eine niedriglatente Peer-to-Peer-Verbindung ermöglichen, die sowohl X11 als auch Wayland unterstützt.

## Aktueller Projektstand

### Frontend (React/TypeScript)
- Grundlegende Benutzeroberfläche mit Connection Manager implementiert
- WebRTC-Integration für Peer-to-Peer-Verbindungen
- Remote Screen-Komponente für Anzeige und Interaktion
- Interface zum Steuern der Bildschirmerfassung und Eingabeweiterleitung

### Backend (Rust/Tauri)
- Grundlegende Architektur für X11- und Wayland-Bildschirmerfassung
- Implementierung zur Eingabeweiterleitung (Maus, Tastatur)
- Tauri-Kommandos für Frontend-Backend-Kommunikation
- Monitorerkennung für beide Display-Server

### Signaling-Server (Node.js)
- WebSocket-basierter Server für WebRTC-Signalisierung
- Raum-Management für Peer-Discovery
- Heartbeat-Mechanismus zur Verbindungsüberwachung

### Kernfunktionen
- WebRTC-Verbindungsaufbau mit SDP-Austausch und ICE-Kandidaten
- FFmpeg-Integration für Bildschirmaufnahme
- Support für verschiedene Videocodecs (H264, VP8, VP9, AV1)
- Hardwarebeschleunigung (VAAPI, NVENC, QuickSync)

## Zu entwickelnde Komponenten

### Phase 1: Optimierung und Fehlerbeseitigung

1. **WebRTC-Integration abschließen**
   - Probleme bei der Bildschirmübertragung zwischen Backend und Frontend beheben
   - Sicherstellung der stabilen Datenübertragung über WebRTC-Kanäle
   - Implementierung von STUN/TURN-Fallback für NAT-Traversal

2. **Input-Weiterleitung verbessern**
   - Spezialschlüssel und Tastenkombinationen unterstützen
   - Relative Positionsberechnung für Multi-Monitor-Setups optimieren
   - Gesten für Trackpads implementieren

3. **Verbindungssicherheit implementieren**
   - Authentifizierungsmechanismus einbauen
   - Datenkanal-Verschlüsselung implementieren
   - Zugriffssteuerungssystem erstellen

4. **Leistungsoptimierung**
   - Latenz unter 200ms erreichen
   - CPU-Auslastung bei Bildschirmerfassung reduzieren
   - Frame-Pufferstrategie entwickeln

### Phase 2: Erweiterte Funktionen

1. **Multi-Monitor-Unterstützung**
   - Dynamische Monitordetektion und -auswahl implementieren
   - Unterstützung für Monitor-Wechsel während einer Session
   - Individuelle Monitor-Streaming-Optionen

2. **Zwischenablage-Synchronisation**
   - Bidirektionale Zwischenablagenübertragung implementieren
   - Unterstützung für Text, Bilder und formatierte Inhalte
   - Zwischenablageverlauf und -verwaltung

3. **Dateiübertragung**
   - Sichere Dateiübertragung zwischen Host und Betrachter
   - Fortschrittsüberwachung und Wiederaufnahmefunktion
   - Verzeichnisübertragung

4. **Erweiterte Sicherheitsfunktionen**
   - OAuth2-Implementierung mit PKCE
   - HMAC-SHA256 für Message-Signing
   - Sitzungs-Berechtigungsverwaltung

### Phase 3: Hardwareoptimierung und Benutzerfreundlichkeit

1. **Hardwarebeschleunigung verbessern**
   - VAAPI-Integration für Intel-GPUs optimieren
   - NVENC-Unterstützung für NVIDIA-GPUs verbessern
   - QuickSync-Unterstützung für kompatible Intel-Prozessoren hinzufügen

2. **Latenzoptimierung**
   - Adaptive Qualität basierend auf Netzwerkbedingungen
   - Frame-Erfassungs- und Verarbeitungspipeline optimieren
   - Intelligentes Frame-Skipping implementieren

3. **UI/UX-Verbesserungen**
   - Anpassbare Tastaturkürzel
   - Verbindungsqualitätsanzeige
   - Bandbreitennutzungsstatistiken
   - Dunkles/helles Theme-Support

4. **Verpackung und Deployment**
   - Debian/Ubuntu-Pakete erstellen
   - RPM-Pakete für Fedora/RHEL bauen
   - AppImage für distributionsunabhängige Installation bereitstellen
   - Flatpak-Unterstützung für Sandbox-Ausführung

## Technische Anforderungen

- **Backend**: Rust mit Tauri-Integration
- **Frontend**: React 18+ mit TypeScript, Vite und Tailwind CSS
- **WebRTC**: Implementierung nach WebRTC-Standards
- **Latenz**: Ziel von <200ms für Bildschirmübertragung
- **Sicherheit**: OAuth2 mit PKCE, HMAC-SHA256 für Nachrichten
- **Kompatibilität**: X11 und Wayland, gängige Browser (Chrome, Firefox, Edge, Safari)
- **Hardware-Beschleunigung**: VAAPI, NVENC und QuickSync

## Spezifische Aufgaben

1. **FFmpeg-Integration optimieren**
   - Implementiere eine effizientere Kodierungspipeline für X11 und Wayland
   - Reduziere Frame-Latenz durch optimierte Pufferstrategie
   - Verbessere Hardwarebeschleunigung mit VAAPI/NVENC

2. **WebRTC-Datenkanalimplementierung abschließen**
   - Zuverlässige Übertragung von Eingabeereignissen sicherstellen
   - Implement Datenkanal für Zwischenablage und Dateiübertragung
   - Protokoll für RPC zwischen Client und Host definieren

3. **Netzwerkresilienz verbessern**
   - ICE-Kandidaten-Verhandlung optimieren
   - Verbindungswiederherstellung nach Unterbrechungen implementieren
   - Bandbreitenmanagement für verschiedene Netzwerkbedingungen

4. **Tests erweitern**
   - End-to-End-Tests für Hauptfunktionen implementieren
   - Latenz- und Performance-Messungen automatisieren
   - Kompatibilitätstests für verschiedene Linux-Distributionen

## Testmatrix

### Leistungstests
- Latenz unter verschiedenen Netzwerkbedingungen (LAN, WAN, mit Jitter und Paketverlust)
- CPU- und GPU-Auslastung
- Tests mit hochauflösenden Displays und mehreren Monitoren

### Kompatibilitätstests
- X11- und Wayland-Unterstützung auf verschiedenen Distributionen
- Tests mit verschiedenen Browserversionen für den Viewer
- Validierung der Hardwarebeschleunigung mit verschiedenen GPU-Modellen

### Sicherheitstests
- Durchführung von Penetrationstests
- Überprüfung der Verschlüsselung und Authentifizierung
- Validierung des Zugriffssteuerungssystems

## Wichtige Hinweise für die Entwicklung

- SmolDesk soll als nativer Ersatz für proprietäre Remote-Desktop-Lösungen dienen
- Fokus auf niedrige Latenz (<200ms) und hohe Bildqualität
- Einfachheit und intuitive Bedienung haben Priorität
- Sichere Verbindungen auch über öffentliche Netzwerke müssen gewährleistet sein
- Die Software muss ohne Administratorrechte auf Client-Seite funktionieren
- Implementierung soll mit dem bestehenden Entwicklungsplan und der technischen Spezifikation übereinstimmen

## Workflow und Code-Standards

- Folge dem Git-Flow-Workflow mit Feature-Branches und Pull Requests
- Verwende einheitliche Commit-Konventionen: `feat:`, `fix:`, `refactor:`, `docs:`, etc.
- Implementiere Unit- und Integrationstests für alle neuen Funktionen
- Dokumentiere alle öffentlichen API-Funktionen und Komponenten
- Halte den Code modular und wiederverwendbar
- Priorisiere Barrierefreiheit und Internationalisierung

## Berichterstattung

Halte in deinen Pull Requests fest:
- Welche Komponenten implementiert wurden
- Welche technischen Entscheidungen getroffen wurden und warum
- Welche Probleme aufgetreten sind und wie sie gelöst wurden
- Ergebnisse durchgeführter Tests
- Nächste geplante Schritte gemäß Implementation-Plan

Beginne mit einer detaillierten Analyse der aktuellen Probleme in der WebRTC-Integration und der Bildschirmerfassung, und erstelle einen priorisierten Aktionsplan für die nächsten zwei Wochen.
