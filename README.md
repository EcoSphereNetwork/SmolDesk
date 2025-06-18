<div align="center">
  <img src="./docs/static/img/logo.png" alt="SmolDesk Logo" width="200">
  <h1>SmolDesk</h1>
  <p>Ein WebRTC-basiertes Remote-Desktop-Tool für Linux mit niedrigen Latenzen und nativer Unterstützung für X11 und Wayland.</p>

  [![Contributors][contributors-shield]][contributors-url]
  [![Stars][stars-shield]][stars-url]
  [![Coverage][coverage-shield]][coverage-url]
  [![MIT License][license-shield]][license-url]
  <br/>
  [![Discord][discord-shield]][discord-url]
  [![Documentation][docs-shield]][docs-url]
  [![Project Credits][credits-shield]][credits-url]

  [Start Documentation](https://github.com/SmolDesk/SmolDesk/blob/main/docs/README.md) •
  [Report Bug](https://github.com/SmolDesk/SmolDesk/issues) •
  [Request Feature](https://github.com/SmolDesk/SmolDesk/issues)
</div>

## 📋 Table of Contents
- [About](#-about)
- [Key Features](#-key-features)
- [Getting Started](#-getting-started)
- [Project Structure](#-project-structure)
- [Development](#-development)
- [Testing](#-testing)
- [Deployment](#-deployment)
- [Contributing](#-contributing)
- [Support](#-support)
- [License](#-license)

## 🎯 About
SmolDesk ist ein modernes Remote-Desktop-Tool, das speziell für Linux entwickelt wurde und beide wichtigen Display-Server (X11 und Wayland) unterstützt. Durch die Verwendung von WebRTC ermöglicht SmolDesk Peer-to-Peer-Verbindungen mit niedriger Latenz, optimiert durch Hardware-Beschleunigung für eine flüssige Benutzererfahrung.

### Warum SmolDesk?
- 🚀 **Niedrige Latenz**: Optimiert für Reaktionsschnelligkeit (<200ms) für ein natürliches Benutzererlebnis
- 🔄 **WebRTC-Integration**: Peer-to-Peer-Verbindungen mit STUN/TURN-Fallback für NAT-Traversal
- 📊 **Optimierte Leistung**: Unterstützung für Hardware-Beschleunigung (VAAPI/NVENC)
- 🛡️ **Sicherheit**: OAuth2-Integration und verschlüsselte Verbindungen
- 📚 **Cross-Platform**: Host auf Linux, Zugriff von jedem modernen Browser aus

## ✨ Key Features

### Core Features
- 🖥️ **Display-Server-Unterstützung**: Vollständige Unterstützung für X11 und Wayland
- 🎮 **Input-Weiterleitung**: Präzise und reaktionsschnelle Maus- und Tastatursteuerung
- 📡 **NAT-Traversal**: STUN/TURN-Server für zuverlässige Verbindungen auch hinter Firewalls
- 📋 **Clipboard-Synchronisation**: Nahtlose Übertragung von Zwischenablage-Inhalten
- 🔄 **Dateitransfer**: Einfacher und sicherer Austausch von Dateien zwischen Host und Client

### Technische Highlights
- 🚀 **Rust-Backend**: Leistungsstarkes und sicheres Rust-Backend mit Tauri-Integration
- ⚛️ **React-Frontend**: Modernes, reaktives UI mit TypeScript und Tailwind CSS
- 🎬 **Hardware-Kodierung**: Unterstützung für VAAPI und NVENC für 4K@60FPS
- 🔐 **Authentifizierung**: OAuth2 mit PKCE-Unterstützung
- 🌐 **Multi-Monitor**: Unterstützung für mehrere Monitore und dynamisches Umschalten

## 🚀 Getting Started

### Systemvoraussetzungen
- **Host-System (Linux)**:
  - X11 oder Wayland
  - Für Hardware-Beschleunigung:
    - Intel-GPU: VAAPI-Bibliotheken
    - NVIDIA-GPU: CUDA und NVENC-Support
  - FFmpeg
  - Für X11: xdotool
  - Für Wayland: ydotool
  - Tauri-Build: libwebkit2gtk-4.0-dev, libjavascriptcoregtk-4.0-dev, libglib2.0-dev

### Build-Abhängigkeiten (Ubuntu/Debian)
Installiere folgende Pakete, um SmolDesk aus dem Quellcode zu bauen:

```bash
sudo apt install build-essential \
  libglib2.0-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libjavascriptcoregtk-4.0-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  pkg-config
```

Alternativ kannst du das Skript `scripts/dev-setup.sh` ausführen, um die Abhängigkeiten automatisch zu installieren.


- **Client-System**:
  - Moderner Browser mit WebRTC-Support (Chrome, Firefox, Edge, Safari)

### Installation

1. **Binärdateien herunterladen**
   ```bash
   # Für Debian/Ubuntu-basierte Systeme
   curl -L https://github.com/SmolDesk/SmolDesk/releases/latest/download/smoldesk_amd64.deb -o smoldesk.deb
   sudo apt install ./smoldesk.deb
   
   # Für Fedora/RHEL-basierte Systeme
   curl -L https://github.com/SmolDesk/SmolDesk/releases/latest/download/smoldesk.rpm -o smoldesk.rpm
   sudo dnf install ./smoldesk.rpm
   
   # Distribution-unabhängig (AppImage)
   curl -L https://github.com/SmolDesk/SmolDesk/releases/latest/download/SmolDesk.AppImage -o SmolDesk.AppImage
   chmod +x SmolDesk.AppImage
   ```

2. **Starten der Anwendung**
   ```bash
   smoldesk
   # oder bei Verwendung der AppImage
   ./SmolDesk.AppImage
   ```

## 📁 Project Structure
```
SmolDesk/
├── .github/                   # GitHub-Konfigurationen und Workflows
├── docs/                      # Projektdokumentation
│   ├── api/                  # API-Referenz
│   ├── user/                 # Benutzerhandbuch
│   └── technical/           # Technische Dokumentation
├── src/                      # Frontend-Quellcode (React)
│   ├── components/          # React-Komponenten
│   ├── hooks/               # React-Hooks
│   ├── utils/               # Hilfsfunktionen
│   └── contexts/            # React-Kontexte
├── src-tauri/               # Backend-Quellcode (Rust)
│   ├── src/                # Rust-Quellcode
│   └── Cargo.toml         # Rust-Abhängigkeiten
├── signaling-server/        # WebRTC Signaling-Server
│   └── index.js           # Server-Implementation
├── tests/                    # Testsuite
│   ├── unit/               # Unit-Tests
│   ├── integration/        # Integrationstests
│   └── e2e/                # End-to-End-Tests
├── package.json             # Projekt-Abhängigkeiten
└── README.md                # Projektdokumentation
```

## 💻 Development

### Entwicklungsumgebung einrichten
1. Repository klonen:
   ```bash
   git clone https://github.com/SmolDesk/SmolDesk.git
   cd SmolDesk
   # falls kein Remote gesetzt ist
   git remote add origin https://github.com/EcoSphereNetwork/SmolDesk.git
   ```

2. Dependencies installieren:
   ```bash
   npm install
   ```

3. Entwicklungsserver starten:
   ```bash
   npm run tauri dev
   ```

### Signaling-Server einrichten
```bash
cd signaling-server
npm install
node index.js
```

### Build erstellen
```bash
npm run tauri build
```

### Codex Setup
Codex agents rely on a working development environment.
Run `scripts/dev-env-check.sh` to verify your system and use `scripts/init-for-codex.sh` for initial install.

If tests fail due to missing vitest packages, execute `scripts/install-vitest.sh`.

## 🧪 Testing

### Tests ausführen
```bash
# Frontend-Tests
npm test

# Für eine komfortable Diagnose kann auch die Vitest UI gestartet werden:
npm run test:ui

# Die Tests nutzen gemockte Tauri-APIs. Diese befinden sich unter
tests/__mocks__ und werden automatisch geladen.

# Backend-Tests
cd src-tauri
cargo test

# End-to-End-Tests
npm run test:e2e
```

### Manuelle Tests
- **NAT-Traversal**: Testen der Verbindung über unterschiedliche Netzwerke
- **Latenz-Messungen**: Überprüfen der Input-zu-Output-Verzögerung
- **Browser-Kompatibilität**: Testen auf verschiedenen Browsern und Plattformen

> **Hinweis:** In Umgebungen ohne Internetzugang werden alle externen
> Netzwerkanfragen blockiert. Die Tests verwenden daher lokale Mocks,
> um Tauri-Funktionen und Browser-APIs zu simulieren.

## 🚢 Deployment

### Paketierung
```bash
# Debian/Ubuntu-Paket erstellen
npm run tauri build -- --target deb

# RPM-Paket erstellen
npm run tauri build -- --target rpm

# AppImage erstellen
npm run tauri build -- --target appimage
```

### Signaling-Server-Deployment
```bash
# Mit Docker
cd signaling-server
docker build -t smoldesk-signaling .
docker run -p 3000:3000 smoldesk-signaling

# Manuell auf einem Server
npm install -g pm2
pm2 start index.js --name smoldesk-signaling
```

## 🤝 Contributing

Wir freuen uns über Beiträge! Bitte lesen Sie unseren [Contributing Guide](CONTRIBUTING.md) für Details.

1. Repository forken
2. Feature-Branch erstellen:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. Änderungen committen:
   ```bash
   git commit -m 'feat: add amazing feature'
   ```
4. Branch pushen:
   ```bash
   git push origin feature/amazing-feature
   ```
5. Pull Request öffnen

## 💬 Support

- [Issue Tracker](https://github.com/SmolDesk/SmolDesk/issues)
- [Discussions](https://github.com/SmolDesk/SmolDesk/discussions)
- [Discord Community][discord-url]
- [Documentation][docs-url]

## 📄 License

Verteilt unter der MIT-Lizenz. Siehe [LICENSE](LICENSE) für weitere Informationen.

---

<div align="center">

### Repository Activity

[![Repository Activity][activity-graph]][activity-url]

</div>

<!-- MARKDOWN LINKS & IMAGES -->
[contributors-shield]: https://img.shields.io/github/contributors/SmolDesk/SmolDesk?style=for-the-badge&color=blue
[contributors-url]: https://github.com/SmolDesk/SmolDesk/graphs/contributors
[stars-shield]: https://img.shields.io/github/stars/SmolDesk/SmolDesk?style=for-the-badge&color=blue
[stars-url]: https://github.com/SmolDesk/SmolDesk/stargazers
[coverage-shield]: https://img.shields.io/codecov/c/github/SmolDesk/SmolDesk?style=for-the-badge&color=blue
[coverage-url]: https://codecov.io/github/SmolDesk/SmolDesk
[license-shield]: https://img.shields.io/github/license/SmolDesk/SmolDesk?style=for-the-badge&color=blue
[license-url]: https://github.com/SmolDesk/SmolDesk/blob/main/LICENSE
[discord-shield]: https://img.shields.io/badge/Discord-Join%20Us-purple?logo=discord&logoColor=white&style=for-the-badge
[discord-url]: https://discord.gg/smoldesk
[docs-shield]: https://img.shields.io/badge/Documentation-000?logo=googledocs&logoColor=FFE165&style=for-the-badge
[docs-url]: https://github.com/SmolDesk/SmolDesk/wiki
[credits-shield]: https://img.shields.io/badge/Project-Credits-blue?style=for-the-badge&color=FFE165&logo=github&logoColor=white
[credits-url]: https://github.com/SmolDesk/SmolDesk/blob/main/CREDITS.md
[activity-graph]: https://repobeats.axiom.co/api/embed/placeholder-for-smoldesk-activity-graph.svg
[activity-url]: https://repobeats.axiom.co
