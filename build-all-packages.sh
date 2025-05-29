#!/bin/bash
# build-all-packages.sh

set -e

echo "🔨 Building SmolDesk Packages..."

# Version aus package.json extrahieren
VERSION=$(node -p "require('./package.json').version")
echo "Version: $VERSION"

# Build-Verzeichnis erstellen
mkdir -p dist
rm -rf dist/*

echo "📦 Building DEB package..."
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles deb

echo "📦 Building RPM package..."
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles rpm

echo "📦 Building AppImage..."
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles appimage

echo "📦 Building TAR.GZ archive..."
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles archive

# Dateien ins dist Verzeichnis kopieren
cp src-tauri/target/release/bundle/deb/*.deb dist/
cp src-tauri/target/release/bundle/rpm/*.rpm dist/
cp src-tauri/target/release/bundle/appimage/*.AppImage dist/
cp src-tauri/target/release/bundle/archive/*.tar.gz dist/

# Signaling Server verpacken
echo "📦 Building Signaling Server..."
cd signaling-server
npm install --production
tar -czf ../dist/smoldesk-signaling-server-${VERSION}.tar.gz .
cd ..

echo "✅ All packages built successfully!"
ls -la dist/
