# Makefile for SmolDesk - WebRTC Remote Desktop for Linux
# This provides a convenient interface for building and managing the project

.PHONY: all clean build dev test install uninstall deps check package help
.DEFAULT_GOAL := help

# Project configuration
PROJECT_NAME := smoldesk
VERSION := 1.0.0
DIST_DIR := dist
SRC_TAURI_DIR := src-tauri

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Color codes for output
RED := \033[31m
GREEN := \033[32m
YELLOW := \033[33m
BLUE := \033[34m
RESET := \033[0m

# Build targets
TARGETS := x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
PACKAGE_FORMATS := deb rpm appimage archive

# Default target - show help
help: ## Show this help message
	@echo "$(BLUE)SmolDesk Build System$(RESET)"
	@echo "====================="
	@echo ""
	@echo "$(GREEN)Available targets:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(YELLOW)%-15s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Examples:$(RESET)"
	@echo "  make deps          # Install all dependencies"
	@echo "  make dev           # Start development server"
	@echo "  make build         # Build for current platform"
	@echo "  make package       # Build all packages"
	@echo "  make clean         # Clean build artifacts"

# Development targets
dev: deps ## Start development server
	@echo "$(BLUE)Starting development server...$(RESET)"
	npm run dev

dev-tauri: deps ## Start Tauri development mode
	@echo "$(BLUE)Starting Tauri development mode...$(RESET)"
	npm run tauri dev

# Dependency management
deps: deps-node deps-rust ## Install all dependencies

deps-node: ## Install Node.js dependencies
	@echo "$(BLUE)Installing Node.js dependencies...$(RESET)"
	npm install

deps-rust: ## Install Rust dependencies
	@echo "$(BLUE)Installing Rust dependencies...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo fetch
	rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu || true

deps-system: ## Check and show system dependencies
	@echo "$(BLUE)Checking system dependencies...$(RESET)"
	@echo "Required packages:"
	@echo "  - FFmpeg (video encoding)"
	@echo "  - X11 tools: xdotool, xclip (for X11 support)"
	@echo "  - Wayland tools: ydotool, wl-clipboard (for Wayland support)"
	@echo "  - PipeWire (for Wayland screen capture)"
	@echo ""
	@echo "$(YELLOW)Ubuntu/Debian:$(RESET)"
	@echo "  sudo apt install ffmpeg xdotool ydotool xclip wl-clipboard pipewire"
	@echo ""
	@echo "$(YELLOW)Fedora/RHEL:$(RESET)"
	@echo "  sudo dnf install ffmpeg xdotool ydotool xclip wl-clipboard pipewire"

# Build targets
build: build-frontend build-backend ## Build the complete application

build-frontend: deps-node ## Build the React frontend
	@echo "$(BLUE)Building frontend...$(RESET)"
	npm run build

build-backend: deps-rust ## Build the Rust backend
	@echo "$(BLUE)Building backend...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo build --release

build-cross: deps-rust ## Cross-compile for all supported architectures
	@echo "$(BLUE)Cross-compiling for all targets...$(RESET)"
	@for target in $(TARGETS); do \
		echo "Building for $$target..."; \
		cd $(SRC_TAURI_DIR) && cargo build --release --target $$target || echo "$(YELLOW)Warning: Failed to build for $$target$(RESET)"; \
	done

# Package targets
package: package-all ## Create all package formats

package-all: build ## Create packages for all formats and architectures
	@echo "$(BLUE)Creating all packages...$(RESET)"
	chmod +x build-all-packages.sh
	./build-all-packages.sh

package-deb: build ## Create DEB packages
	@echo "$(BLUE)Creating DEB packages...$(RESET)"
	npm run build:deb

package-rpm: build ## Create RPM packages
	@echo "$(BLUE)Creating RPM packages...$(RESET)"
	npm run build:rpm

package-appimage: build ## Create AppImage
	@echo "$(BLUE)Creating AppImage...$(RESET)"
	npm run build:appimage

package-archive: build ## Create tar.gz archive
	@echo "$(BLUE)Creating archive...$(RESET)"
	npm run build:archive

# Testing targets
test: test-frontend test-backend ## Run all tests

test-frontend: deps-node ## Run frontend tests
	@echo "$(BLUE)Running frontend tests...$(RESET)"
	npm run test:run

test-backend: deps-rust ## Run backend tests
	@echo "$(BLUE)Running backend tests...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo test

test-integration: build ## Run integration tests
	@echo "$(BLUE)Running integration tests...$(RESET)"
	chmod +x scripts/test-packages.sh
	./scripts/test-packages.sh

# Code quality targets
check: check-frontend check-backend ## Run all code quality checks

check-frontend: deps-node ## Check frontend code quality
	@echo "$(BLUE)Checking frontend code quality...$(RESET)"
	npm run lint
	npm run typecheck

check-backend: deps-rust ## Check backend code quality
	@echo "$(BLUE)Checking backend code quality...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo clippy -- -D warnings
	cd $(SRC_TAURI_DIR) && cargo fmt --check

fix: fix-frontend fix-backend ## Fix code formatting issues

fix-frontend: deps-node ## Fix frontend code formatting
	@echo "$(BLUE)Fixing frontend code formatting...$(RESET)"
	npm run lint:fix
	npm run format

fix-backend: deps-rust ## Fix backend code formatting
	@echo "$(BLUE)Fixing backend code formatting...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo fmt

# Security targets
security-scan: deps-rust ## Run security audit
	@echo "$(BLUE)Running security audit...$(RESET)"
	npm audit
	cd $(SRC_TAURI_DIR) && cargo audit || echo "$(YELLOW)Install cargo-audit with: cargo install cargo-audit$(RESET)"

# Installation targets
install: package-deb ## Install SmolDesk on the current system
	@echo "$(BLUE)Installing SmolDesk...$(RESET)"
	@if [ -f "$(DIST_DIR)/smoldesk_$(VERSION)_amd64.deb" ]; then \
		sudo dpkg -i "$(DIST_DIR)/smoldesk_$(VERSION)_amd64.deb" || \
		(sudo apt-get install -f && sudo dpkg -i "$(DIST_DIR)/smoldesk_$(VERSION)_amd64.deb"); \
	else \
		echo "$(RED)DEB package not found. Run 'make package-deb' first.$(RESET)"; \
		exit 1; \
	fi

uninstall: ## Uninstall SmolDesk from the current system
	@echo "$(BLUE)Uninstalling SmolDesk...$(RESET)"
	sudo dpkg -r smoldesk || echo "$(YELLOW)SmolDesk was not installed via DEB package$(RESET)"

# Cleanup targets
clean: clean-frontend clean-backend clean-dist ## Clean all build artifacts

clean-frontend: ## Clean frontend build artifacts
	@echo "$(BLUE)Cleaning frontend build artifacts...$(RESET)"
	rm -rf dist/
	rm -rf node_modules/.cache/

clean-backend: ## Clean backend build artifacts
	@echo "$(BLUE)Cleaning backend build artifacts...$(RESET)"
	cd $(SRC_TAURI_DIR) && cargo clean

clean-dist: ## Clean distribution packages
	@echo "$(BLUE)Cleaning distribution packages...$(RESET)"
	rm -rf $(DIST_DIR)/

clean-all: clean ## Clean everything including dependencies
	@echo "$(BLUE)Cleaning all artifacts and dependencies...$(RESET)"
	rm -rf node_modules/
	cd $(SRC_TAURI_DIR) && rm -rf target/

# Documentation targets
docs: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(RESET)"
	npm run docs:build
	cd $(SRC_TAURI_DIR) && cargo doc --no-deps --open

docs-serve: docs ## Serve documentation locally
	@echo "$(BLUE)Serving documentation...$(RESET)"
	npm run docs:serve

# Release targets
release: clean package ## Create a complete release
	@echo "$(BLUE)Creating release build...$(RESET)"
	chmod +x scripts/sign-packages.sh
	./scripts/sign-packages.sh
	@echo "$(GREEN)Release build completed!$(RESET)"
	@echo "Packages available in: $(DIST_DIR)/"

# Utility targets
info: ## Show project information
	@echo "$(BLUE)Project Information$(RESET)"
	@echo "==================="
	@echo "Name:      $(PROJECT_NAME)"
	@echo "Version:   $(VERSION)"
	@echo "Platform:  $(UNAME_S) $(UNAME_M)"
	@echo "Node.js:   $$(node --version 2>/dev/null || echo 'Not found')"
	@echo "npm:       $$(npm --version 2>/dev/null || echo 'Not found')"
	@echo "Rust:      $$(rustc --version 2>/dev/null || echo 'Not found')"
	@echo "Cargo:     $$(cargo --version 2>/dev/null || echo 'Not found')"
	@echo ""
	@echo "$(BLUE)Build Targets:$(RESET)"
	@for target in $(TARGETS); do echo "  - $$target"; done
	@echo ""
	@echo "$(BLUE)Package Formats:$(RESET)"
	@for format in $(PACKAGE_FORMATS); do echo "  - $$format"; done

watch: deps ## Watch for file changes and rebuild
	@echo "$(BLUE)Watching for changes...$(RESET)"
	npm run dev &
	cd $(SRC_TAURI_DIR) && cargo watch -x "build --release" &
	wait

# Environment setup
setup: deps deps-system ## Setup complete development environment
	@echo "$(GREEN)Development environment setup complete!$(RESET)"
	@echo "Run 'make dev' to start the development server."

# Quick development tasks
quick-build: deps-node ## Quick development build (frontend only)
	@echo "$(BLUE)Quick development build...$(RESET)"
	npm run build

quick-test: deps-node deps-rust ## Quick test run
	@echo "$(BLUE)Quick test run...$(RESET)"
	npm run test:run &
	cd $(SRC_TAURI_DIR) && cargo test &
	wait

# Docker support (future enhancement)
docker-build: ## Build in Docker container
	@echo "$(BLUE)Building in Docker container...$(RESET)"
	@echo "$(YELLOW)Docker support coming soon...$(RESET)"

# Maintenance targets
update-deps: ## Update all dependencies
	@echo "$(BLUE)Updating dependencies...$(RESET)"
	npm update
	cd $(SRC_TAURI_DIR) && cargo update

check-outdated: ## Check for outdated dependencies
	@echo "$(BLUE)Checking for outdated dependencies...$(RESET)"
	npm outdated || true
	cd $(SRC_TAURI_DIR) && cargo outdated || echo "$(YELLOW)Install cargo-outdated with: cargo install cargo-outdated$(RESET)"
