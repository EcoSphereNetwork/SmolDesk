#!/bin/bash
# validate-build.sh - Validiert alle Dateien und Abhängigkeiten für SmolDesk Build

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

check_file() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        log_success "$description: $file"
        return 0
    else
        log_error "$description fehlt: $file"
        return 1
    fi
}

check_directory() {
    local dir="$1"
    local description="$2"
    
    if [ -d "$dir" ]; then
        log_success "$description: $dir"
        return 0
    else
        log_error "$description fehlt: $dir"
        return 1
    fi
}

check_command() {
    local cmd="$1"
    local description="$2"
    
    if command -v "$cmd" >/dev/null 2>&1; then
        log_success "$description: $(which $cmd)"
        return 0
    else
        log_error "$description nicht gefunden: $cmd"
        return 1
    fi
}

check_optional_command() {
    local cmd="$1"
    local description="$2"
    
    if command -v "$cmd" >/dev/null 2>&1; then
        log_success "$description: $(which $cmd)"
        return 0
    else
        log_warning "$description nicht gefunden (optional): $cmd"
        return 1
    fi
}

echo "🔍 SmolDesk Build Validierung"
echo "============================="
echo ""

# Fehler-Counter
errors=0
warnings=0

# 1. Basis-Projektstruktur prüfen
log_info "Prüfe Basis-Projektstruktur..."

check_file "package.json" "Frontend package.json" || ((errors++))
check_file "src-tauri/Cargo.toml" "Rust Cargo.toml" || ((errors++))
check_file "src-tauri/tauri.conf.json" "Tauri Konfiguration" || ((errors++))
check_file "Makefile" "Makefile" || ((errors++))
check_file "build-all-packages.sh" "Build-Script" || ((errors++))

echo ""

# 2. Frontend-Dateien prüfen
log_info "Prüfe Frontend-Dateien..."

check_file "index.html" "HTML Entry Point" || ((errors++))
check_file "src/main.tsx" "React Entry Point" || ((errors++))
check_file "src/App.tsx" "React App Component" || ((errors++))
check_file "src/styles.css" "CSS Styles" || ((errors++))
check_file "vite.config.ts" "Vite Konfiguration" || ((errors++))
check_file "tsconfig.json" "TypeScript Konfiguration" || ((errors++))

# React Komponenten
check_file "src/components/ConnectionManager.tsx" "ConnectionManager Komponente" || ((warnings++))
check_file "src/components/RemoteScreen.tsx" "RemoteScreen Komponente" || ((warnings++))
check_file "src/components/ClipboardSync.tsx" "ClipboardSync Komponente" || ((warnings++))
check_file "src/components/FileTransfer.tsx" "FileTransfer Komponente" || ((warnings++))

# Utils
check_file "src/utils/webrtc.ts" "WebRTC Utils" || ((warnings++))
check_file "src/utils/enhancedWebRTC.ts" "Enhanced WebRTC" || ((warnings++))
check_file "src/utils/screenCapture.ts" "Screen Capture Utils" || ((warnings++))
check_file "src/utils/securityManager.ts" "Security Manager" || ((warnings++))

# Hooks
check_file "src/hooks/useSmolDesk.ts" "useSmolDesk Hook" || ((warnings++))

echo ""

# 3. Rust Backend-Dateien prüfen
log_info "Prüfe Rust Backend-Dateien..."

check_file "src-tauri/src/main.rs" "Rust Main" || ((errors++))
check_file "src-tauri/src/screen_capture.rs" "Screen Capture Modul" || ((errors++))
check_file "src-tauri/src/input_forwarding.rs" "Input Forwarding Modul" || ((errors++))
check_file "src-tauri/build.rs" "Rust Build Script" || ((warnings++))

echo ""

# 4. Icons und Assets prüfen
log_info "Prüfe Icons und Assets..."

check_file "icons/32x32.png" "32x32 Icon" || ((errors++))
check_file "icons/128x128.png" "128x128 Icon" || ((errors++))
check_file "icons/128x128@2x.png" "128x128@2x Icon" || ((errors++))
check_file "icons/icon.ico" "Windows Icon" || ((warnings++))
check_file "icons/icon.icns" "macOS Icon" || ((warnings++))
check_file "docs/static/img/logo.png" "Logo PNG" || ((errors++))

echo ""

# 5. Packaging-Dateien prüfen
log_info "Prüfe Packaging-Dateien..."

check_file "packaging/smoldesk.desktop" "Desktop Entry" || ((errors++))
check_directory "packaging/debian" "Debian Packaging" || ((warnings++))
check_directory "packaging/rpm" "RPM Packaging" || ((warnings++))
check_directory "packaging/appimage" "AppImage Packaging" || ((warnings++))

echo ""

# 6. Signaling Server prüfen
log_info "Prüfe Signaling Server..."

check_directory "signaling-server" "Signaling Server Verzeichnis" || ((warnings++))
check_file "signaling-server/package.json" "Signaling Server package.json" || ((warnings++))
check_file "signaling-server/index.js" "Signaling Server Main" || ((warnings++))

echo ""

# 7. System-Abhängigkeiten prüfen
log_info "Prüfe System-Abhängigkeiten..."

# Basis-Tools
check_command "node" "Node.js" || ((errors++))
check_command "npm" "npm" || ((errors++))
check_command "rustc" "Rust Compiler" || ((errors++))
check_command "cargo" "Cargo" || ((errors++))

# Build-Tools
check_command "make" "Make" || ((errors++))
check_optional_command "git" "Git" || ((warnings++))

# Linux-spezifische Tools
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    log_info "Prüfe Linux-spezifische Abhängigkeiten..."
    
    # Multimedia
    check_command "ffmpeg" "FFmpeg" || ((errors++))
    
    # Display Server Tools
    check_optional_command "xrandr" "X11 Tools (xrandr)" || ((warnings++))
    check_optional_command "wlr-randr" "Wayland Tools (wlr-randr)" || ((warnings++))
    
    # Input Tools
    check_optional_command "xdotool" "X11 Input (xdotool)" || ((warnings++))
    check_optional_command "ydotool" "Wayland Input (ydotool)" || ((warnings++))
    
    # Clipboard Tools
    check_optional_command "xclip" "X11 Clipboard (xclip)" || ((warnings++))
    check_optional_command "wl-copy" "Wayland Clipboard (wl-copy)" || ((warnings++))
    
    # Packaging Tools
    check_optional_command "dpkg-deb" "DEB Packaging" || ((warnings++))
    check_optional_command "rpmbuild" "RPM Packaging" || ((warnings++))
    
    # Icon Tools
    check_optional_command "convert" "ImageMagick (für Icons)" || ((warnings++))
fi

echo ""

# 8. Node.js Dependencies prüfen
log_info "Prüfe Node.js Dependencies..."

if [ -f "package.json" ] && [ -d "node_modules" ]; then
    log_success "node_modules vorhanden"
    
    # Prüfe wichtige Dependencies
    for dep in "@tauri-apps/cli" "@tauri-apps/api" "react" "react-dom" "vite"; do
        if [ -d "node_modules/$dep" ]; then
            log_success "Dependency: $dep"
        else
            log_warning "Dependency fehlt: $dep"
            ((warnings++))
        fi
    done
else
    log_warning "node_modules nicht gefunden - führe 'npm install' aus"
    ((warnings++))
fi

echo ""

# 9. Rust Dependencies prüfen
log_info "Prüfe Rust Dependencies..."

if [ -f "src-tauri/Cargo.lock" ]; then
    log_success "Cargo.lock vorhanden"
else
    log_warning "Cargo.lock nicht gefunden - führe 'cargo fetch' aus"
    ((warnings++))
fi

echo ""

# 10. Validiere JSON/TOML Dateien
log_info "Validiere Konfigurationsdateien..."

# package.json
if command -v node >/dev/null 2>&1; then
    if node -e "JSON.parse(require('fs').readFileSync('package.json', 'utf8'))" 2>/dev/null; then
        log_success "package.json ist valide"
    else
        log_error "package.json ist nicht valide JSON"
        ((errors++))
    fi
fi

# Cargo.toml
if command -v cargo >/dev/null 2>&1; then
    if cargo check --manifest-path src-tauri/Cargo.toml --quiet 2>/dev/null; then
        log_success "Cargo.toml ist valide"
    else
        log_warning "Cargo.toml könnte Probleme haben"
        ((warnings++))
    fi
fi

# tauri.conf.json
if command -v node >/dev/null 2>&1; then
    if node -e "JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8'))" 2>/dev/null; then
        log_success "tauri.conf.json ist valide"
    else
        log_error "tauri.conf.json ist nicht valide JSON"
        ((errors++))
    fi
fi

echo ""

# Zusammenfassung
echo "🎯 Validierungsergebnis:"
echo "======================="

if [ $errors -eq 0 ] && [ $warnings -eq 0 ]; then
    log_success "Alle Prüfungen bestanden! Build sollte funktionieren."
    echo ""
    echo "🚀 Zum Starten des Builds:"
    echo "   make setup"
    echo "   make build"
    echo "   make package"
    exit 0
elif [ $errors -eq 0 ]; then
    log_warning "$warnings Warnungen gefunden, aber Build sollte funktionieren."
    echo ""
    echo "🚀 Zum Starten des Builds:"
    echo "   make setup"
    echo "   make build"
    echo "   make package"
    exit 0
else
    log_error "$errors kritische Fehler und $warnings Warnungen gefunden!"
    echo ""
    echo "🔧 Führe zuerst dieses Script aus:"
    echo "   ./build-fixes.sh"
    echo ""
    echo "📦 Installiere dann die Dependencies:"
    echo "   npm install"
    echo "   cd src-tauri && cargo fetch"
    echo ""
    echo "🚀 Dann starte den Build:"
    echo "   make setup"
    echo "   make build"
    echo "   make package"
    exit 1
fi
