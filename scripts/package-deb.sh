#!/bin/bash
set -e

TARGET="${1:-x86_64-unknown-linux-gnu}"
VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)

echo "Creating DEB package for $TARGET version $VERSION"

# Create package directory structure
PACKAGE_DIR="target/release/bundle/deb"
mkdir -p "$PACKAGE_DIR"

# Create temporary build directory
BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

# Copy Debian control files
cp -r packaging/debian/DEBIAN "$BUILD_DIR/"

# Copy binary
mkdir -p "$BUILD_DIR/opt/smoldesk"
if [ "$TARGET" = "aarch64-unknown-linux-gnu" ]; then
    cp "src-tauri/target/aarch64-unknown-linux-gnu/release/smoldesk" "$BUILD_DIR/opt/smoldesk/"
    ARCH="arm64"
else
    cp "src-tauri/target/release/smoldesk" "$BUILD_DIR/opt/smoldesk/"
    ARCH="amd64"
fi

# Copy signaling server
cp -r signaling-server "$BUILD_DIR/opt/smoldesk/"

# Copy desktop file and resources
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/pixmaps"
mkdir -p "$BUILD_DIR/usr/bin"

cp packaging/debian/usr/share/applications/smoldesk.desktop "$BUILD_DIR/usr/share/applications/"
cp docs/static/img/logo.png "$BUILD_DIR/usr/share/pixmaps/smoldesk.png"

# Create symlink
ln -s /opt/smoldesk/smoldesk "$BUILD_DIR/usr/bin/smoldesk"

# Update version and architecture in control file
sed -i "s/Version: .*/Version: $VERSION/" "$BUILD_DIR/DEBIAN/control"
sed -i "s/Architecture: .*/Architecture: $ARCH/" "$BUILD_DIR/DEBIAN/control"

# Set permissions
chmod 755 "$BUILD_DIR/DEBIAN/postinst"
chmod 755 "$BUILD_DIR/DEBIAN/prerm"
chmod 755 "$BUILD_DIR/opt/smoldesk/smoldesk"

# Build package
PACKAGE_NAME="smoldesk_${VERSION}_${ARCH}.deb"
dpkg-deb --build "$BUILD_DIR" "$PACKAGE_DIR/$PACKAGE_NAME"

echo "DEB package created: $PACKAGE_DIR/$PACKAGE_NAME"
