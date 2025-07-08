---
title: SmolDesk Benutzerhandbuch
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Der aktuelle Inhalt befindet sich unter `docs/usage/viewer.md`

# SmolDesk Benutzerhandbuch

## Inhaltsverzeichnis
1. [Installation](#installation)
2. [Erste Schritte](#erste-schritte)
3. [Als Host einrichten](#als-host-einrichten)
4. [Zu einem Host verbinden](#zu-einem-host-verbinden)
5. [Erweiterte Funktionen](#erweiterte-funktionen)
6. [Problembehandlung](#problembehandlung)

## Installation

### Debian/Ubuntu
```bash
# Download der neuesten Version
wget https://github.com/EcoSphereNetwork/SmolDesk/releases/latest/download/smoldesk_1.0.0_amd64.deb

# Installation
sudo apt install ./smoldesk_1.0.0_amd64.deb
```

### Fedora/RHEL/openSUSE
```bash
# Download der neuesten Version
wget https://github.com/EcoSphereNetwork/SmolDesk/releases/latest/download/smoldesk-1.0.0-1.x86_64.rpm

# Installation
sudo dnf install ./smoldesk-1.0.0-1.x86_64.rpm
# oder für openSUSE:
sudo zypper install ./smoldesk-1.0.0-1.x86_64.rpm
```

### AppImage (Universal)
```bash
# Download und ausführbar machen
wget https://github.com/EcoSphereNetwork/SmolDesk/releases/latest/download/SmolDesk-1.0.0-x86_64.AppImage
chmod +x SmolDesk-1.0.0-x86_64.AppImage

# Starten
./SmolDesk-1.0.0-x86_64.AppImage
```

### Flatpak
```bash
# Download und Installation
wget https://github.com/EcoSphereNetwork/SmolDesk/releases/latest/download/SmolDesk.flatpak
flatpak install SmolDesk.flatpak

# Starten
flatpak run com.smoldesk.SmolDesk
```

## Erste Schritte

### Systemanforderungen
- **Host-System**: Linux mit X11 oder Wayland
- **Client**: Jeder moderne Browser mit WebRTC-Unterstützung
- **Netzwerk**: Internetverbindung für Signaling (STUN/TURN optional)

### Abhängigkeiten
SmolDesk benötigt je nach Display-Server unterschiedliche Tools:

**Für X11:**
- `xdotool` (Input-Forwarding)
- `xclip` (Zwischenablage)
- `ffmpeg` (Bildschirmaufnahme)

**Für Wayland:**
- `ydotool` (Input-Forwarding)
- `wl-clipboard` (Zwischenablage)
- `ffmpeg` mit PipeWire-Unterstützung

### Erste Konfiguration
1. Starten Sie SmolDesk
2. Das System erkennt automatisch Ihren Display-Server
3. Verfügbare Monitore werden angezeigt
4. Wählen Sie Ihre bevorzugten Einstellungen

## Als Host einrichten

### 1. Hosting-Tab öffnen
- Klicken Sie auf den "Host"-Tab in der Seitenleiste
- Überprüfen Sie die erkannten Monitore und Einstellungen

### 2. Capture-Einstellungen konfigurieren
```
Frame Rate: 30 FPS (empfohlen für gute Balance)
Quality: 80% (höhere Werte für bessere Qualität)
Video Codec: H264 (beste Kompatibilität)
Hardware Acceleration: VAAPI/NVENC falls verfügbar
```

### 3. Monitor auswählen
- Wählen Sie den zu teilenden Monitor aus der Dropdown-Liste
- Der primäre Monitor ist standardmäßig vorausgewählt

### 4. Hosting starten
- Klicken Sie auf "Start Hosting"
- Ein Room-Code wird generiert und angezeigt
- Teilen Sie diesen Code mit den Personen, die sich verbinden möchten

### 5. Sicherheitsoptionen
- **Öffentlich**: Jeder mit dem Room-Code kann beitreten
- **Geschützt**: Zusätzliches Passwort erforderlich
- **Privat**: Nur explizit eingeladene Benutzer

## Zu einem Host verbinden

### 1. Viewer-Tab öffnen
- Wechseln Sie zum "View"-Tab
- Geben Sie den erhaltenen Room-Code ein

### 2. Verbindung herstellen
- Klicken Sie auf "Connect"
- Geben Sie bei geschützten Räumen das Passwort ein
- Warten Sie auf die Verbindungsherstellung

### 3. Remote-Steuerung
- **Maus**: Bewegen Sie die Maus für Remote-Steuerung
- **Tastatur**: Alle Tasteneingaben werden weitergeleitet
- **Vollbild**: F11 oder Vollbild-Button für immersive Erfahrung

### 4. Input-Toggle
- Button "Input: On/Off" zum Aktivieren/Deaktivieren der Eingabe
- Nützlich um zwischen Ansicht und Steuerung zu wechseln

## Erweiterte Funktionen

### Zwischenablage-Synchronisation
- Automatische Synchronisation von Text zwischen Host und Client
- Unterstützung für Bilder (PNG, JPEG, GIF)
- HTML-Inhalt wird als Text übertragen
- Konfigurierbar in den Einstellungen

### Dateiübertragung
- Drag & Drop von Dateien in die Anwendung
- Unterstützung für mehrere Dateien gleichzeitig
- Fortschrittsanzeige und Pause/Resume-Funktionalität
- Maximale Dateigröße standardmäßig 100MB

### Multi-Monitor-Unterstützung
- Dynamisches Wechseln zwischen Monitoren während einer Session
- Individuelle Einstellungen pro Monitor
- Unterstützung für verschiedene Auflösungen und Bildwiederholraten

### Hardware-Beschleunigung
- **VAAPI**: Intel-GPUs und AMD-GPUs
- **NVENC**: NVIDIA-GPUs
- **QuickSync**: Intel-CPUs mit integrierter Grafik
- Automatische Erkennung und Fallback auf Software-Encoding

## Problembehandlung

### Häufige Probleme

#### Verbindung schlägt fehl
**Symptom**: Keine Verbindung möglich
**Lösung**:
1. Prüfen Sie die Internetverbindung
2. Stellen Sie sicher, dass der Signaling-Server erreichbar ist
3. Überprüfen Sie Firewall-Einstellungen
4. Versuchen Sie einen anderen Browser

#### Schlechte Bildqualität
**Symptom**: Pixelige oder ruckelige Übertragung
**Lösung**:
1. Reduzieren Sie die FPS auf 15-20
2. Verringern Sie die Qualitätseinstellung
3. Prüfen Sie die Netzwerkbandbreite
4. Aktivieren Sie Hardware-Beschleunigung

#### Input-Forwarding funktioniert nicht
**Symptom**: Maus/Tastatur-Eingaben werden nicht übertragen
**Lösung**:

**Für X11:**
```bash
# Installieren Sie xdotool
sudo apt install xdotool  # Debian/Ubuntu
sudo dnf install xdotool  # Fedora
```

**Für Wayland:**
```bash
# Installieren Sie ydotool
sudo apt install ydotool  # Debian/Ubuntu
sudo dnf install ydotool  # Fedora

# Starten Sie ydotool-Daemon
sudo systemctl start ydotool
sudo systemctl enable ydotool
```

#### Zwischenablage-Sync funktioniert nicht
**Symptom**: Inhalte werden nicht synchronisiert
**Lösung**:

**Für X11:**
```bash
sudo apt install xclip
```

**Für Wayland:**
```bash
sudo apt install wl-clipboard
```

### Leistungsoptimierung

#### Für niedrige Latenz (<100ms)
```
FPS: 60
Quality: 70%
Codec: H264
Hardware Acceleration: Aktiviert
Latency Mode: Ultra Low
```

#### Für niedrige Bandbreite
```
FPS: 15
Quality: 50%
Codec: VP9
Hardware Acceleration: Nach Verfügbarkeit
```

#### Für hohe Qualität
```
FPS: 30
Quality: 90%
Codec: H264
Hardware Acceleration: Aktiviert
Keyframe Interval: 60
```

### Log-Dateien

**Systemweite Installation:**
```
/var/log/smoldesk/
~/.local/share/smoldesk/logs/
```

**AppImage:**
```
~/.local/share/SmolDesk/logs/
```

**Flatpak:**
```
~/.var/app/com.smoldesk.SmolDesk/data/logs/
```

### Support

Bei Problemen können Sie:
1. Das [GitHub Issue Tracker](https://github.com/EcoSphereNetwork/SmolDesk/issues) nutzen
2. Die [Diskussionen](https://github.com/EcoSphereNetwork/SmolDesk/discussions) durchsuchen
3. Die [Community](https://discord.gg/smoldesk) im Discord kontaktieren

**Bevor Sie einen Bug-Report erstellen:**
1. Prüfen Sie die Log-Dateien
2. Reproduzieren Sie das Problem
3. Sammeln Sie Systeminformationen:
   ```bash
   smoldesk --version
   echo $XDG_SESSION_TYPE
   ffmpeg -version | head -1
   ```
```
