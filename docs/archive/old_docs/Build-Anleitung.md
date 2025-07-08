---
title: "\U0001F680 SmolDesk Build-Anleitung"
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/development/setup-android.md`
# 🚀 SmolDesk Build-Anleitung

## ⚠️ **WICHTIG: Vor dem ersten Build**

Das Projekt hat fehlende Dateien, die den Build verhindern. Führe **zuerst** diese Schritte aus:

## 1. 🔧 **Build-Fixes ausführen**

```bash
# Build-fixes Script ausführbar machen und starten
chmod +x build-fixes.sh
./build-fixes.sh
```

Das Script erstellt:
- ✅ Frontend Entry Points (`src/main.tsx`, `index.html`, `vite.config.ts`)
- ✅ Rust Backend Module (`screen_capture.rs`, `input_forwarding.rs`)  
- ✅ Icons und Assets
- ✅ Desktop Integration Dateien
- ✅ Basis CSS und TypeScript Konfiguration

## 2. 🔍 **Build-Validierung**

```bash
# Validierung Script ausführbar machen und prüfen
chmod +x validate-build.sh
./validate-build.sh
```

## 3. 📦 **Dependencies installieren**

```bash
# Node.js Dependencies
npm install

# Rust Dependencies
cd src-tauri
cargo fetch
cd ..
```

## 4. 🏗️ **System-Dependencies installieren**

### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install -y \
    nodejs npm \
    rust-all cargo \
    ffmpeg \
    xdotool ydotool \
    xclip wl-clipboard \
    dpkg-dev rpm \
    imagemagick \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Fedora/RHEL:
```bash
sudo dnf install -y \
    nodejs npm \
    rust cargo \
    ffmpeg \
    xdotool ydotool \
    xclip wl-clipboard \
    rpm-build \
    ImageMagick \
    webkit2gtk4.1-devel \
    gtk3-devel
```

## 5. 🚀 **Build starten**

### **Vollständiger Build:**
```bash
# Entwicklungsumgebung einrichten
make setup

# Alles bauen und alle Pakete erstellen
make package
```

### **Einzelne Schritte:**
```bash
# 1. Frontend bauen
make build-frontend

# 2. Backend bauen  
make build-backend

# 3. Spezifische Pakete
make package-deb        # DEB-Paket
make package-rpm        # RPM-Paket
make package-appimage   # AppImage
make package-archive    # TAR.GZ Archive
```

### **Detaillierter Build:**
```bash
# Alternatives Build-Script mit mehr Output
chmod +x build-all-packages.sh
./build-all-packages.sh
```

## 6. 🧪 **Entwicklung**

```bash
# Entwicklungsserver starten
make dev

# Oder spezifisch Tauri dev
make dev-tauri
```

## 7. 📋 **Verfügbare Pakete**

Nach erfolgreichem Build findest du in `dist/`:

```
dist/
├── smoldesk_1.0.0_amd64.deb          # Debian/Ubuntu
├── smoldesk-1.0.0-1.x86_64.rpm       # Fedora/RHEL  
├── SmolDesk_1.0.0_amd64.AppImage      # Universal Linux
├── smoldesk-1.0.0.tar.gz              # Archive
├── smoldesk-signaling-server-1.0.0.tar.gz  # Signaling Server
├── SHA256SUMS                         # Checksums
└── RELEASE_NOTES.md                   # Release Notes
```

## 8. 📦 **Installation testen**

### DEB-Paket:
```bash
sudo dpkg -i dist/smoldesk_1.0.0_amd64.deb
sudo apt-get install -f  # Fehlende Dependencies nachinstallieren
```

### RPM-Paket:
```bash
sudo rpm -i dist/smoldesk-1.0.0-1.x86_64.rpm
```

### AppImage:
```bash
chmod +x dist/SmolDesk_1.0.0_amd64.AppImage
./dist/SmolDesk_1.0.0_amd64.AppImage
```

## 9. 🐞 **Troubleshooting**

### **Problem: "Module nicht gefunden"**
```bash
# Fehlende Rust Module
./build-fixes.sh
```

### **Problem: "Icons fehlen"**
```bash
# ImageMagick installieren
sudo apt install imagemagick  # Ubuntu
sudo dnf install ImageMagick  # Fedora

# Icons neu generieren
./build-fixes.sh
```

### **Problem: "Frontend startet nicht"**
```bash
# Frontend Dependencies prüfen
npm install
npm run build
```

### **Problem: "Tauri build fehlschlägt"**
```bash
# Tauri CLI neu installieren
npm install -g @tauri-apps/cli

# Rust toolchain aktualisieren
rustup update
```

### **Problem: "System-Dependencies fehlen"**
```bash
# Prüfe was fehlt
./validate-build.sh

# Linux-spezifische Packages nachinstallieren
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev  # Ubuntu
sudo dnf install webkit2gtk4.1-devel gtk3-devel      # Fedora
```

## 10. 🎯 **Vollständiger Workflow**

```bash
# 1. Repository klonen/vorbereiten
git clone <repository> && cd smoldesk

# 2. Fehlende Dateien erstellen
chmod +x build-fixes.sh validate-build.sh
./build-fixes.sh

# 3. Validierung
./validate-build.sh

# 4. Dependencies installieren
npm install
cd src-tauri && cargo fetch && cd ..

# 5. Build starten
make setup
make package

# 6. Testen
make test
```

## 11. 📊 **Build-Status prüfen**

Das Validierungs-Script zeigt dir:
- ✅ **Grün**: Alles OK
- ⚠️  **Gelb**: Warnungen (Build funktioniert trotzdem)
- ❌ **Rot**: Kritische Fehler (Build wird fehlschlagen)

## 12. 🔧 **Make Targets Übersicht**

```bash
make help           # Zeigt alle verfügbaren Targets
make info           # Zeigt Projekt-Informationen
make deps           # Installiert alle Dependencies
make deps-system    # Zeigt System-Dependencies
make clean          # Bereinigt Build-Artifacts
make clean-all      # Bereinigt alles inkl. Dependencies
make check          # Code-Qualitätsprüfungen
make security-scan  # Security Audit
make release        # Vollständiger Release-Build
```

## ✅ **Erfolgreicher Build**

Wenn alles funktioniert, solltest du sehen:
```
✅ All packages tested successfully!
✅ Release build completed!
Packages available in: dist/
```

Dann kannst du SmolDesk installieren und testen! 🎉
