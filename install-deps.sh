#!/bin/bash
# install-deps.sh - Automatische Installation aller SmolDesk Dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

log_section() {
    echo -e "${PURPLE}[SECTION]${NC} $1"
    echo "=========================="
}

# Detect Linux distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        VERSION=$VERSION_ID
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
    elif [ -f /etc/redhat-release ]; then
        DISTRO="fedora"
    else
        DISTRO="unknown"
    fi
    
    log_info "Detected distribution: $DISTRO $VERSION"
}

# Check if running as root
check_sudo() {
    if [ "$EUID" -eq 0 ]; then
        SUDO=""
        log_warning "Running as root"
    else
        SUDO="sudo"
        if ! command -v sudo >/dev/null 2>&1; then
            log_error "sudo not found. Please run as root or install sudo."
            exit 1
        fi
    fi
}

# Install Node.js and npm (if not present)
install_nodejs() {
    log_section "Node.js und npm"
    
    if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
        local node_version=$(node --version)
        local npm_version=$(npm --version)
        log_success "Node.js already installed: $node_version"
        log_success "npm already installed: $npm_version"
        
        # Check if version is sufficient (>= 18)
        local major_version=$(echo $node_version | sed 's/v\([0-9]*\).*/\1/')
        if [ "$major_version" -lt 18 ]; then
            log_warning "Node.js version $node_version is older than recommended (18+)"
            log_info "Consider upgrading Node.js"
        fi
        return
    fi
    
    log_info "Installing Node.js and npm..."
    
    case $DISTRO in
        ubuntu|debian)
            # Use NodeSource repository for latest Node.js
            curl -fsSL https://deb.nodesource.com/setup_20.x | $SUDO -E bash -
            $SUDO apt-get install -y nodejs
            ;;
        fedora|centos|rhel)
            $SUDO dnf install -y nodejs npm
            ;;
        arch|manjaro)
            $SUDO pacman -S --noconfirm nodejs npm
            ;;
        opensuse*)
            $SUDO zypper install -y nodejs20 npm20
            ;;
        alpine)
            $SUDO apk add nodejs npm
            ;;
        *)
            log_warning "Unknown distribution. Please install Node.js manually."
            log_info "Visit: https://nodejs.org/"
            ;;
    esac
    
    log_success "Node.js and npm installed"
}

# Install Rust and Cargo
install_rust() {
    log_section "Rust und Cargo"
    
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        local rust_version=$(rustc --version)
        log_success "Rust already installed: $rust_version"
        return
    fi
    
    log_info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    log_success "Rust installed"
}

# Install system dependencies
install_system_deps() {
    log_section "System Dependencies"
    
    case $DISTRO in
        ubuntu|debian)
            log_info "Updating package database..."
            $SUDO apt-get update
            
            log_info "Installing essential build tools..."
            $SUDO apt-get install -y \
                build-essential \
                curl \
                wget \
                git \
                pkg-config \
                make
            
            log_info "Installing Tauri dependencies..."
            $SUDO apt-get install -y \
                libwebkit2gtk-4.1-dev \
                libgtk-3-dev \
                libayatana-appindicator3-dev \
                librsvg2-dev \
                libssl-dev
            
            log_info "Installing multimedia tools..."
            $SUDO apt-get install -y \
                ffmpeg \
                pipewire \
                libpipewire-0.3-0
            
            log_info "Installing X11 tools..."
            $SUDO apt-get install -y \
                x11-xserver-utils \
                xdotool \
                xclip \
                xsel
            
            log_info "Installing Wayland tools..."
            $SUDO apt-get install -y \
                wlr-randr \
                ydotool \
                wl-clipboard \
                grim \
                slurp
            
            log_info "Installing packaging tools..."
            $SUDO apt-get install -y \
                dpkg-dev \
                rpm \
                imagemagick \
                fakeroot \
                lintian
            ;;
            
        fedora|centos|rhel)
            log_info "Updating package database..."
            $SUDO dnf update -y
            
            log_info "Installing essential build tools..."
            $SUDO dnf groupinstall -y "Development Tools"
            $SUDO dnf install -y \
                curl \
                wget \
                git \
                pkg-config \
                make
            
            log_info "Installing Tauri dependencies..."
            $SUDO dnf install -y \
                webkit2gtk4.1-devel \
                gtk3-devel \
                librsvg2-devel \
                openssl-devel
            
            log_info "Installing multimedia tools..."
            $SUDO dnf install -y \
                ffmpeg \
                pipewire \
                pipewire-devel
            
            log_info "Installing X11 tools..."
            $SUDO dnf install -y \
                xrandr \
                xdotool \
                xclip \
                xsel
            
            log_info "Installing Wayland tools..."
            $SUDO dnf install -y \
                wlr-randr \
                ydotool \
                wl-clipboard \
                grim \
                slurp
            
            log_info "Installing packaging tools..."
            $SUDO dnf install -y \
                rpm-build \
                rpm-devel \
                ImageMagick
            ;;
            
        arch|manjaro)
            log_info "Updating package database..."
            $SUDO pacman -Syu --noconfirm
            
            log_info "Installing essential build tools..."
            $SUDO pacman -S --noconfirm \
                base-devel \
                curl \
                wget \
                git \
                pkg-config \
                make
            
            log_info "Installing Tauri dependencies..."
            $SUDO pacman -S --noconfirm \
                webkit2gtk \
                gtk3 \
                librsvg \
                openssl
            
            log_info "Installing multimedia tools..."
            $SUDO pacman -S --noconfirm \
                ffmpeg \
                pipewire
            
            log_info "Installing X11 tools..."
            $SUDO pacman -S --noconfirm \
                xorg-xrandr \
                xdotool \
                xclip \
                xsel
            
            log_info "Installing Wayland tools..."
            $SUDO pacman -S --noconfirm \
                wlr-randr \
                ydotool \
                wl-clipboard \
                grim \
                slurp
            
            log_info "Installing packaging tools..."
            $SUDO pacman -S --noconfirm \
                imagemagick \
                fakeroot
            ;;
            
        opensuse*)
            log_info "Updating package database..."
            $SUDO zypper refresh
            
            log_info "Installing essential build tools..."
            $SUDO zypper install -y \
                -t pattern devel_basis \
                curl \
                wget \
                git \
                pkg-config \
                make
            
            log_info "Installing Tauri dependencies..."
            $SUDO zypper install -y \
                webkit2gtk3-devel \
                gtk3-devel \
                librsvg-devel \
                libopenssl-devel
            
            log_info "Installing multimedia tools..."
            $SUDO zypper install -y \
                ffmpeg \
                pipewire
            
            log_info "Installing packaging tools..."
            $SUDO zypper install -y \
                rpm-build \
                ImageMagick
            ;;
            
        alpine)
            log_info "Updating package database..."
            $SUDO apk update
            
            log_info "Installing essential build tools..."
            $SUDO apk add \
                build-base \
                curl \
                wget \
                git \
                pkgconfig \
                make
            
            log_info "Installing Tauri dependencies..."
            $SUDO apk add \
                webkit2gtk-dev \
                gtk+3.0-dev \
                librsvg-dev \
                openssl-dev
            
            log_info "Installing multimedia tools..."
            $SUDO apk add \
                ffmpeg \
                pipewire
            
            log_info "Installing X11 tools..."
            $SUDO apk add \
                xrandr \
                xdotool \
                xclip
            
            log_info "Installing Wayland tools..."
            $SUDO apk add \
                wlr-randr \
                ydotool \
                wl-clipboard
            ;;
            
        *)
            log_error "Unsupported distribution: $DISTRO"
            log_info "Please install dependencies manually or contribute support for your distribution."
            return 1
            ;;
    esac
    
    log_success "System dependencies installed"
}

# Install development tools
install_dev_tools() {
    log_section "Development Tools"
    
    # Tauri CLI
    if ! command -v cargo-tauri >/dev/null 2>&1; then
        log_info "Installing Tauri CLI..."
        cargo install tauri-cli
    else
        log_success "Tauri CLI already installed"
    fi
    
    log_success "Development tools ready"
}

# Optional: Install additional useful tools
install_optional_tools() {
    log_section "Optional Tools"
    
    case $DISTRO in
        ubuntu|debian)
            log_info "Installing additional development tools..."
            $SUDO apt-get install -y \
                htop \
                tree \
                jq \
                unzip \
                zip \
                code \
                firefox \
                chromium-browser 2>/dev/null || true
            ;;
        fedora|centos|rhel)
            log_info "Installing additional development tools..."
            $SUDO dnf install -y \
                htop \
                tree \
                jq \
                unzip \
                zip 2>/dev/null || true
            ;;
        arch|manjaro)
            log_info "Installing additional development tools..."
            $SUDO pacman -S --noconfirm \
                htop \
                tree \
                jq \
                unzip \
                zip 2>/dev/null || true
            ;;
    esac
    
    log_success "Optional tools installed"
}

# Verify installation
verify_installation() {
    log_section "Installation Verification"
    
    local errors=0
    
    # Check essential tools
    local essential_tools=("node" "npm" "rustc" "cargo" "git" "make" "pkg-config")
    for tool in "${essential_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_success "$tool: $(command -v $tool)"
        else
            log_error "$tool not found"
            ((errors++))
        fi
    done
    
    # Check multimedia tools
    local multimedia_tools=("ffmpeg")
    for tool in "${multimedia_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_success "$tool: $(command -v $tool)"
        else
            log_warning "$tool not found (multimedia features may be limited)"
        fi
    done
    
    # Check X11 tools
    local x11_tools=("xrandr" "xdotool" "xclip")
    for tool in "${x11_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_success "$tool: $(command -v $tool)"
        else
            log_warning "$tool not found (X11 features may be limited)"
        fi
    done
    
    # Check Wayland tools
    local wayland_tools=("wlr-randr" "ydotool" "wl-copy")
    for tool in "${wayland_tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_success "$tool: $(command -v $tool)"
        else
            log_warning "$tool not found (Wayland features may be limited)"
        fi
    done
    
    if [ $errors -eq 0 ]; then
        log_success "All essential dependencies verified!"
        return 0
    else
        log_error "$errors essential dependencies missing"
        return 1
    fi
}

# Main installation function
main() {
    echo "🚀 SmolDesk Dependencies Installer"
    echo "==================================="
    echo ""
    
    # Parse command line arguments
    INSTALL_OPTIONAL=false
    SKIP_NODEJS=false
    SKIP_RUST=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help|-h)
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --help, -h          Show this help message"
                echo "  --optional          Install optional development tools"
                echo "  --skip-nodejs       Skip Node.js installation"
                echo "  --skip-rust         Skip Rust installation"
                echo "  --verify-only       Only verify existing installation"
                echo ""
                echo "Examples:"
                echo "  $0                  Install essential dependencies"
                echo "  $0 --optional       Install everything including optional tools"
                echo "  $0 --verify-only    Check what's already installed"
                exit 0
                ;;
            --optional)
                INSTALL_OPTIONAL=true
                shift
                ;;
            --skip-nodejs)
                SKIP_NODEJS=true
                shift
                ;;
            --skip-rust)
                SKIP_RUST=true
                shift
                ;;
            --verify-only)
                detect_distro
                verify_installation
                exit $?
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Detect system
    detect_distro
    check_sudo
    
    # Install components
    if [ "$SKIP_NODEJS" != true ]; then
        install_nodejs
    fi
    
    if [ "$SKIP_RUST" != true ]; then
        install_rust
    fi
    
    install_system_deps
    install_dev_tools
    
    if [ "$INSTALL_OPTIONAL" = true ]; then
        install_optional_tools
    fi
    
    # Verify installation
    if verify_installation; then
        echo ""
        log_success "🎉 All dependencies installed successfully!"
        echo ""
        log_info "Next steps:"
        echo "  1. Restart your terminal or run: source ~/.cargo/env"
        echo "  2. Run: ./validate-build.sh"
        echo "  3. Build SmolDesk: make setup && make package"
        echo ""
    else
        echo ""
        log_error "❌ Some dependencies failed to install"
        echo ""
        log_info "Please check the errors above and install missing dependencies manually"
        exit 1
    fi
}

# Run main function
main "$@"
