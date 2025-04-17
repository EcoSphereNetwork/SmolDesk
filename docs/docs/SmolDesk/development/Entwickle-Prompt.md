# Entwickler-Prompt: SmolDesk - WebRTC-basiertes Remote-Desktop-Tool

## Ausgangssituation

Du übernimmst die Entwicklung von SmolDesk, einem WebRTC-basierten Remote-Desktop-Tool für Linux. Das Projekt soll eine effiziente Peer-to-Peer-Verbindung mit niedriger Latenz ermöglichen und sowohl X11 als auch Wayland unterstützen. Die Architektur besteht aus einem React-Frontend und einem Rust-Backend mit Tauri-Integration.

## Deine Aufgaben

### 1. Bestandsaufnahme und Analyse

- Verschaffe dir einen Überblick über die Projektstruktur
- Prüfe den aktuellen Entwicklungsstand der Komponenten:
  - WebRTC-Integration
  - Bildschirmübertragungsfunktionen
  - Input-Weiterleitung
  - Signaling-Server
- Sichte die vorhandene Dokumentation unter docs/
- Identifiziere fehlende Komponenten und Funktionalitäten basierend auf der Anforderungsanalyse

### 2. Planung und Priorisierung

- Erstelle einen Implementierungsplan nach dem Entwicklungsplan-Dokument:
  - Phase 1: WebRTC-Grundgerüst, Bildschirmaufnahme, Input-Forwarding
  - Phase 2: Sicherheitsfunktionen, Multi-Monitor-Support
  - Phase 3: Hardware-Optimierung (VAAPI/NVENC)
- Setze Prioritäten für die Implementierung basierend auf der Roadmap

### 3. Entwicklung der Kernfunktionalitäten

- Implementiere die WebRTC-Verbindung mit:
  - Signaling-Server mit WebSocket (Node.js)
  - Peer-Connection mit SDP-Handshake
  - STUN/TURN-Server für NAT-Traversal
- Entwickle die Bildschirmaufnahmekomponente:
  - X11-Support via FFmpeg
  - Wayland-Support über pipewire-portal
- Implementiere Input-Weiterleitung (Maus/Tastatur)
- Integriere Dateitransfer und Clipboard-Funktionalität

### 4. Frontend-Entwicklung

- Entwickle das React-Frontend mit Vite und TypeScript
- Erstelle React-Komponenten für:
  - Verbindungsmanagement
  - Monitor-Auswahl
  - Einstellungen für Videoqualität
  - Sicherheitsoptionen
- Integriere das Frontend mit dem Tauri-Backend

### 5. Testdurchführung

- Entwickle und führe Tests gemäß der Testmatrix durch:
  - NAT-Traversal-Tests
  - Latenz-Messungen (Ziel: <200ms)
  - Browserkompatibilitätstests
  - Sicherheitstests (OAuth2, HMAC-SHA256)
- Setze Automatisierungstests mit Selenium auf
- Integriere die Tests in eine CI/CD-Pipeline

### 6. Optimierung

- Implementiere Hardware-Beschleunigung:
  - VAAPI-Integration
  - NVIDIA NVENC-Support für 4K@60FPS
- Optimiere für niedrige CPU-Last (<15% bei 1080p)
- Verbessere NAT-Traversal mit Coturn und Redis-Clustering

### 7. Dokumentation

- Aktualisiere die technische Dokumentation
- Erstelle eine API-Referenz mit OpenAPI 3.0
- Dokumentiere Installationsanweisungen und Abhängigkeiten
- Verfasse Benutzerdokumentation

## Workflow

- Nach jedem bedeutenden Entwicklungsschritt:
  - Committe deine Änderungen mit klaren Commit-Nachrichten
  - Erstelle Pull Requests nach dem Format: `[Bereich]: Was wurde gemacht und warum`
  - Dokumentiere Fortschritte und Herausforderungen

- Für jede abgeschlossene Komponente:
  - Stelle sicher, dass Tests vorhanden sind
  - Überprüfe die Funktionalität in verschiedenen Szenarien
  - Aktualisiere die entsprechende Dokumentation

## Technische Anforderungen

- **Backend**: Rust mit Tauri-Integration
- **Frontend**: React 18+ mit TypeScript, Vite und Tailwind CSS
- **WebRTC**: Implementierung nach WebRTC-Standards
- **Latenz**: Ziel von <200ms für Bildschirmübertragung
- **Sicherheit**: OAuth2 mit PKCE, HMAC-SHA256 für Nachrichten
- **Kompatibilität**: X11 und Wayland, gängige Browser
- **Skalierung**: 1.000 gleichzeitige Verbindungen via Kubernetes

## Abnahmekriterien

Die Arbeit gilt als erfolgreich abgeschlossen, wenn:

- WebRTC-Verbindung mit P2P und TURN/STUN-Fallback funktioniert
- Bildschirmübertragung mit niedriger Latenz (<200ms) möglich ist
- Input-Weiterleitung korrekt funktioniert
- Hardware-Beschleunigung implementiert ist
- Sicherheitskonzept umgesetzt ist
- Tests erfolgreich durchgeführt wurden
- Dokumentation vollständig ist

## Berichterstattung

Halte in deinen Pull Requests klar fest:

- Welche Komponenten implementiert wurden
- Welche technischen Entscheidungen getroffen wurden und warum
- Welche Probleme aufgetreten sind und wie sie gelöst wurden
- Ergebnisse der durchgeführten Tests
- Nächste geplante Schritte gemäß Entwicklungsplan

## Wichtige Hinweise

- Folge der Architektur gemäß der technischen Spezifikation
- Achte besonders auf Latenzoptimierung und Sicherheit
- Teste gründlich NAT-Traversal und Browser-Kompatibilität
- Implementiere Fallback-Mechanismen für inkompatible Systeme
- Qualität geht vor Geschwindigkeit

Bitte beginne mit einer Analyse des aktuellen Stands und erstelle einen detaillierten Implementierungsplan basierend auf dem Entwicklungsplan-Dokument.
