#!/bin/bash
# test-packages.sh

echo "🧪 Testing built packages..."

# DEB-Paket testen
echo "Testing DEB package..."
docker run --rm -v $(pwd)/dist:/packages ubuntu:latest bash -c "
    apt update && 
    apt install -y /packages/*.deb && 
    smoldesk --version
"

# RPM-Paket testen  
echo "Testing RPM package..."
docker run --rm -v $(pwd)/dist:/packages fedora:latest bash -c "
    dnf install -y /packages/*.rpm && 
    smoldesk --version
"

# AppImage testen
echo "Testing AppImage..."
chmod +x dist/*.AppImage
dist/*.AppImage --version

echo "✅ All packages tested successfully!"
