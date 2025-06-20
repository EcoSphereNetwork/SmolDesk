#!/bin/bash
# quick-install.sh - Schnelle Installation nur der essentiellen SmolDesk Dependencies

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 SmolDesk Quick Dependencies Install${NC}"
echo "======================================"

# Detect distro
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo "Cannot detect Linux distribution"
    exit 1
fi

echo -e "${BLUE}📋 Installing for: $DISTRO${NC}"

# Install based on distro
case $DISTRO in
    ubuntu|debian)
        echo -e "${BLUE}📦 Updating package database...${NC}"
        sudo apt update
        
        echo -e "${BLUE}🔧 Installing essential dependencies...${NC}"
        sudo apt install -y \
            curl \
            build-essential \
            pkg-config \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev \
            librsvg2-dev \
            libssl-dev \
            ffmpeg \
            xdotool \
            xclip \
            wlr-randr \
            ydotool \
            wl-clipboard \
            rpm \
            imagemagick
        
        # Node.js
        if ! command -v node >/dev/null 2>&1; then
            echo -e "${BLUE}📦 Installing Node.js...${NC}"
            curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
            sudo apt install -y nodejs
        fi
        ;;
        
    fedora|rhel|centos)
        echo -e "${BLUE}📦 Updating package database...${NC}"
        sudo dnf update -y
        
        echo -e "${BLUE}🔧 Installing essential dependencies...${NC}"
        sudo dnf install -y \
            curl \
            gcc \
            gcc-c++ \
            make \
            pkg-config \
            webkit2gtk4.1-devel \
            gtk3-devel \
            librsvg2-devel \
            openssl-devel \
            ffmpeg \
            nodejs \
            npm \
            xdotool \
            xclip \
            wlr-randr \
            ydotool \
            wl-clipboard \
            rpm-build \
            ImageMagick
        ;;
        
    arch|manjaro)
        echo -e "${BLUE}📦 Updating package database...${NC}"
        sudo pacman -Syu --noconfirm
        
        echo -e "${BLUE}🔧 Installing essential dependencies...${NC}"
        sudo pacman -S --noconfirm \
            curl \
            base-devel \
            webkit2gtk \
            gtk3 \
            librsvg \
            openssl \
            ffmpeg \
            nodejs \
            npm \
            xdotool \
            xclip \
            wlr-randr \
            ydotool \
            wl-clipboard \
            imagemagick
        ;;
        
    *)
        echo -e "${YELLOW}⚠️  Unsupported distribution: $DISTRO${NC}"
        echo "Please install dependencies manually using dependencies.md"
        exit 1
        ;;
esac

# Install Rust if not present
if ! command -v rustc >/dev/null 2>&1; then
    echo -e "${BLUE}🦀 Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install Tauri CLI
if ! cargo install --list | grep -q tauri-cli; then
    echo -e "${BLUE}⚡ Installing Tauri CLI...${NC}"
    cargo install tauri-cli
fi

echo ""
echo -e "${GREEN}✅ Essential dependencies installed!${NC}"
echo ""
echo -e "${BLUE}🔄 Please restart your terminal or run:${NC}"
echo "   source ~/.cargo/env"
echo ""
echo -e "${BLUE}🚀 Then continue with:${NC}"
echo "   ./validate-build.sh"
echo "   make setup && make package"
