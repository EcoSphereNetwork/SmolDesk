#!/bin/bash
# build-all-packages.sh - Comprehensive build script for SmolDesk

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
DIST_DIR="$PROJECT_ROOT/dist"
TARGET_ARCHITECTURES=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
PACKAGE_FORMATS=("deb" "rpm" "appimage" "archive")

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking build dependencies..."
    
    local missing_deps=()
    
    # Core tools
    command -v node >/dev/null 2>&1 || missing_deps+=("node")
    command -v npm >/dev/null 2>&1 || missing_deps+=("npm")
    command -v rustc >/dev/null 2>&1 || missing_deps+=("rust")
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    
    # Linux-specific tools
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Screen capture dependencies
        command -v ffmpeg >/dev/null 2>&1 || missing_deps+=("ffmpeg")
        command -v xrandr >/dev/null 2>&1 || log_warning "xrandr not found - X11 support may be limited"
        command -v wlr-randr >/dev/null 2>&1 || log_warning "wlr-randr not found - Wayland support may be limited"
        
        # Input forwarding dependencies
        command -v xdotool >/dev/null 2>&1 || log_warning "xdotool not found - X11 input forwarding will be disabled"
        command -v ydotool >/dev/null 2>&1 || log_warning "ydotool not found - Wayland input forwarding will be disabled"
        
        # Clipboard dependencies
        command -v xclip >/dev/null 2>&1 || log_warning "xclip not found - X11 clipboard sync will be disabled"
        command -v wl-clipboard >/dev/null 2>&1 || log_warning "wl-clipboard not found - Wayland clipboard sync will be disabled"
        
        # Package creation tools
        command -v dpkg-deb >/dev/null 2>&1 || missing_deps+=("dpkg-deb (for DEB packages)")
        command -v rpmbuild >/dev/null 2>&1 || log_warning "rpmbuild not found - RPM packages will be skipped"
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        
        log_info "To install dependencies on Ubuntu/Debian:"
        echo "  sudo apt update"
        echo "  sudo apt install nodejs npm rust-all ffmpeg xdotool ydotool xclip wl-clipboard dpkg-dev rpm"
        
        log_info "To install dependencies on Fedora/RHEL:"
        echo "  sudo dnf install nodejs npm rust cargo ffmpeg xdotool ydotool xclip wl-clipboard rpm-build"
        
        exit 1
    fi
    
    log_success "All required dependencies found"
}

setup_rust_targets() {
    log_info "Setting up Rust targets..."
    
    for target in "${TARGET_ARCHITECTURES[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            log_info "Installing Rust target: $target"
            rustup target add "$target" || {
                log_warning "Failed to install target $target - skipping"
                continue
            }
        fi
    done
    
    log_success "Rust targets configured"
}

install_dependencies() {
    log_info "Installing Node.js dependencies..."
    npm install || {
        log_error "Failed to install Node.js dependencies"
        exit 1
    }
    
    log_info "Installing Rust dependencies..."
    cd src-tauri
    cargo fetch || {
        log_error "Failed to fetch Rust dependencies"
        exit 1
    }
    cd ..
    
    log_success "Dependencies installed"
}

build_frontend() {
    log_info "Building frontend..."
    
    # Clean previous build
    rm -rf dist/
    
    # Build React frontend
    npm run build || {
        log_error "Frontend build failed"
        exit 1
    }
    
    log_success "Frontend built successfully"
}

build_backend() {
    local target="$1"
    log_info "Building backend for target: $target"
    
    cd src-tauri
    
    # Build with specific target
    if [ "$target" = "x86_64-unknown-linux-gnu" ]; then
        # Native build
        cargo build --release || {
            log_error "Backend build failed for $target"
            cd ..
            exit 1
        }
    else
        # Cross-compilation
        cargo build --release --target "$target" || {
            log_warning "Backend build failed for $target - skipping"
            cd ..
            return 1
        }
    fi
    
    cd ..
    log_success "Backend built for $target"
}

create_packages() {
    local target="$1"
    log_info "Creating packages for target: $target"
    
    # Ensure dist directory exists
    mkdir -p "$DIST_DIR"
    
    # Determine architecture for package naming
    local arch
    case "$target" in
        "x86_64-unknown-linux-gnu")
            arch="amd64"
            ;;
        "aarch64-unknown-linux-gnu")
            arch="arm64"
            ;;
        *)
            arch="unknown"
            ;;
    esac
    
    # Build packages using Tauri CLI
    for format in "${PACKAGE_FORMATS[@]}"; do
        log_info "Creating $format package for $arch..."
        
        case "$format" in
            "deb")
                npm run tauri build -- --target "$target" --bundles deb || {
                    log_warning "DEB package creation failed for $target"
                    continue
                }
                ;;
            "rpm")
                npm run tauri build -- --target "$target" --bundles rpm || {
                    log_warning "RPM package creation failed for $target"
                    continue
                }
                ;;
            "appimage")
                npm run tauri build -- --target "$target" --bundles appimage || {
                    log_warning "AppImage creation failed for $target"
                    continue
                }
                ;;
            "archive")
                npm run tauri build -- --target "$target" --bundles archive || {
                    log_warning "Archive creation failed for $target"
                    continue
                }
                ;;
        esac
    done
    
    log_success "Packages created for $target"
}

copy_packages_to_dist() {
    log_info "Copying packages to dist directory..."
    
    # Copy all built packages to main dist directory
    find src-tauri/target -name "*.deb" -exec cp {} "$DIST_DIR/" \;
    find src-tauri/target -name "*.rpm" -exec cp {} "$DIST_DIR/" \;
    find src-tauri/target -name "*.AppImage" -exec cp {} "$DIST_DIR/" \;
    find src-tauri/target -name "*.tar.gz" -exec cp {} "$DIST_DIR/" \;
    
    # Copy signaling server
    if [ -d "signaling-server" ]; then
        log_info "Packaging signaling server..."
        cd signaling-server
        npm install --production
        tar -czf "../$DIST_DIR/smoldesk-signaling-server-1.0.0.tar.gz" .
        cd ..
    fi
    
    log_success "Packages copied to $DIST_DIR"
}

generate_checksums() {
    log_info "Generating checksums..."
    
    cd "$DIST_DIR"
    
    # Generate SHA256 checksums
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum *.deb *.rpm *.AppImage *.tar.gz > SHA256SUMS 2>/dev/null || true
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 *.deb *.rpm *.AppImage *.tar.gz > SHA256SUMS 2>/dev/null || true
    fi
    
    cd "$PROJECT_ROOT"
    log_success "Checksums generated"
}

create_release_notes() {
    log_info "Creating release notes..."
    
    cat > "$DIST_DIR/RELEASE_NOTES.md" << EOF
# SmolDesk v1.0.0 Release

## Overview
SmolDesk is a modern remote desktop solution that provides low-latency screen sharing using WebRTC technology with native support for both X11 and Wayland display servers.

## Features
- ✅ WebRTC-based peer-to-peer connections
- ✅ Low-latency screen capture and streaming
- ✅ Native X11 and Wayland support
- ✅ Hardware acceleration (VAAPI, NVENC, QuickSync)
- ✅ Multiple video codecs (H.264, VP8, VP9, AV1)
- ✅ Adaptive quality control
- ✅ Input forwarding (mouse, keyboard, gestures)
- ✅ Clipboard synchronization
- ✅ File transfer capabilities
- ✅ Connection security and authentication

## Installation

### Debian/Ubuntu (.deb)
\`\`\`bash
sudo dpkg -i smoldesk_1.0.0_amd64.deb
sudo apt-get install -f  # Install missing dependencies
\`\`\`

### Fedora/RHEL (.rpm)
\`\`\`bash
sudo rpm -i smoldesk-1.0.0-1.x86_64.rpm
\`\`\`

### AppImage (Universal)
\`\`\`bash
chmod +x SmolDesk-1.0.0-x86_64.AppImage
./SmolDesk-1.0.0-x86_64.AppImage
\`\`\`

## Dependencies
- FFmpeg (for video encoding)
- PipeWire (for Wayland screen capture)
- X11 tools: xdotool, xclip (for X11 support)
- Wayland tools: ydotool, wl-clipboard (for Wayland support)

## Usage
\`\`\`bash
# Start hosting
smoldesk host --monitor 0 --quality 80

# Join a session
smoldesk join <room-id>

# Start signaling server
smoldesk server --port 8080
\`\`\`

## Build Information
- Built on: $(date)
- Rust version: $(rustc --version 2>/dev/null || echo "Unknown")
- Node.js version: $(node --version 2>/dev/null || echo "Unknown")

## Support
- GitHub: https://github.com/EcoSphereNetwork/SmolDesk
- Issues: https://github.com/EcoSphereNetwork/SmolDesk/issues
- Documentation: https://github.com/EcoSphereNetwork/SmolDesk/wiki
EOF

    log_success "Release notes created"
}

cleanup() {
    log_info "Cleaning up temporary files..."
    
    # Clean Rust build artifacts (keep release binaries)
    cd src-tauri
    cargo clean --release || true
    cd ..
    
    # Clean Node.js build artifacts
    rm -rf node_modules/.cache || true
    
    log_success "Cleanup completed"
}

main() {
    log_info "Starting SmolDesk build process..."
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Check system and dependencies
    check_dependencies
    
    # Setup build environment
    setup_rust_targets
    install_dependencies
    
    # Build frontend
    build_frontend
    
    # Build for each target architecture
    for target in "${TARGET_ARCHITECTURES[@]}"; do
        log_info "Processing target: $target"
        
        if build_backend "$target"; then
            create_packages "$target"
        else
            log_warning "Skipping package creation for $target due to build failure"
        fi
    done
    
    # Copy and organize packages
    copy_packages_to_dist
    
    # Generate checksums and documentation
    generate_checksums
    create_release_notes
    
    # Optional cleanup
    if [ "${CLEANUP:-true}" = "true" ]; then
        cleanup
    fi
    
    log_success "Build process completed successfully!"
    log_info "Packages are available in: $DIST_DIR"
    
    # Show build summary
    if [ -d "$DIST_DIR" ]; then
        log_info "Build summary:"
        ls -la "$DIST_DIR"
    fi
}

# Handle script arguments
case "${1:-}" in
    "--help"|"-h")
        echo "SmolDesk Build Script"
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --no-cleanup   Skip cleanup after build"
        echo "  --check-deps   Only check dependencies"
        echo ""
        echo "Environment variables:"
        echo "  CLEANUP=false  Skip cleanup after build"
        exit 0
        ;;
    "--no-cleanup")
        export CLEANUP=false
        main
        ;;
    "--check-deps")
        check_dependencies
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
