---
title: "\U0001F680 **Automatische Installation:**"
description: ''
---
> ⚠️ Diese Datei wurde archiviert. Die aktuelle Version befindet sich unter `docs/development/dev-tools.md`

## 🚀 **Automatische Installation:**

```bash
# Vollständige Installation
chmod +x install-deps.sh
./install-deps.sh

# Mit optionalen Tools
./install-deps.sh --optional

# Nur Verifikation
./install-deps.sh --verify-only
```

## 🔧 **Manuelle Installation nach Distribution:**

### **Ubuntu/Debian:**
```bash
# Essential build tools
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    make

# Node.js (latest)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri dependencies
sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev

# Multimedia
sudo apt install -y \
    ffmpeg \
    pipewire \
    libpipewire-0.3-0

# X11 tools
sudo apt install -y \
    xrandr \
    xdotool \
    xclip \
    xsel

# Wayland tools
sudo apt install -y \
    wlr-randr \
    ydotool \
    wl-clipboard \
    grim \
    slurp

# Packaging tools
sudo apt install -y \
    dpkg-dev \
    rpm \
    imagemagick \
    fakeroot
```

### **Fedora/RHEL:**
```bash
# Essential build tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y \
    curl \
    wget \
    git \
    pkg-config \
    make

# Node.js and npm
sudo dnf install -y nodejs npm

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri dependencies
sudo dnf install -y \
    webkit2gtk4.1-devel \
    gtk3-devel \
    librsvg2-devel \
    openssl-devel

# Multimedia
sudo dnf install -y \
    ffmpeg \
    pipewire \
    pipewire-devel

# X11 tools
sudo dnf install -y \
    xrandr \
    xdotool \
    xclip

# Wayland tools
sudo dnf install -y \
    wlr-randr \
    ydotool \
    wl-clipboard

# Packaging tools
sudo dnf install -y \
    rpm-build \
    ImageMagick
```

### **Arch Linux:**
```bash
# Essential build tools
sudo pacman -S --noconfirm \
    base-devel \
    curl \
    wget \
    git \
    pkg-config

# Node.js and npm
sudo pacman -S --noconfirm nodejs npm

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri dependencies
sudo pacman -S --noconfirm \
    webkit2gtk \
    gtk3 \
    librsvg \
    openssl

# Multimedia
sudo pacman -S --noconfirm \
    ffmpeg \
    pipewire

# X11/Wayland tools
sudo pacman -S --noconfirm \
    xorg-xrandr \
    xdotool \
    xclip \
    wlr-randr \
    ydotool \
    wl-clipboard

# Packaging tools
sudo pacman -S --noconfirm \
    imagemagick \
    fakeroot
```

## 📋 **Dependency Categories:**

### **🔧 Essential (Required):**
- **Build Tools:** gcc, make, pkg-config, git
- **Node.js:** >= 18.0.0 + npm
- **Rust:** >= 1.70 + cargo
- **Tauri Core:** webkit2gtk, gtk3, librsvg
- **SSL:** openssl-dev/libssl-dev

### **📺 Multimedia (Core Features):**
- **Video:** ffmpeg
- **Audio:** pipewire (Linux)
- **Codecs:** H.264, VP8, VP9 support

### **🖥️ Display Servers:**
- **X11:** xrandr, xdotool, xclip
- **Wayland:** wlr-randr, ydotool, wl-clipboard

### **📦 Packaging (Build Output):**
- **DEB:** dpkg-dev, fakeroot
- **RPM:** rpm-build, rpmbuild
- **Icons:** imagemagick/ImageMagick

### **🎨 Optional (Enhanced Features):**
- **Development:** code, htop, tree, jq
- **Testing:** chromium, firefox
- **Graphics:** grim, slurp (screenshots)

## 🎯 **Minimum Requirements:**

```bash
# Absolute minimum for basic build
sudo apt install -y \
    nodejs npm \
    build-essential \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    ffmpeg

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 🔍 **Verification Commands:**

```bash
# Check versions
node --version    # >= 18
npm --version     # >= 9
rustc --version   # >= 1.70
cargo --version
ffmpeg -version

# Check Tauri
cargo install tauri-cli
cargo tauri --version

# Check display tools
xrandr --version      # X11
wlr-randr --help     # Wayland
echo $XDG_SESSION_TYPE  # Current session
```

## 🚨 **Common Issues:**

### **Node.js too old:**
```bash
# Remove old Node.js
sudo apt remove nodejs npm
# Install latest
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install nodejs
```

### **Rust not in PATH:**
```bash
source ~/.cargo/env
# Or restart terminal
```

### **WebKit not found:**
```bash
# Ubuntu 24.04+
sudo apt install libwebkit2gtk-4.1-dev
# Ubuntu 22.04 and older
sudo apt install libwebkit2gtk-4.0-dev
```

### **Permission issues:**
```bash
# Fix npm permissions
sudo chown -R $(whoami) ~/.npm
# Or use node version manager
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
```

## 📊 **Disk Space Requirements:**

- **Minimum:** ~2 GB (essential tools only)
- **Recommended:** ~5 GB (with optional tools)
- **Development:** ~10 GB (with IDEs and full toolchain)

## 🔗 **Official Documentation:**

- **Tauri:** https://tauri.app/v1/guides/getting-started/prerequisites
- **Node.js:** https://nodejs.org/en/download/package-manager
- **Rust:** https://rustup.rs/
- **WebKit:** https://webkitgtk.org/
