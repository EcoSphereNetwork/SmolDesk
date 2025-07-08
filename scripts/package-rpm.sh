#!/bin/bash
set -e

TARGET="${1:-x86_64-unknown-linux-gnu}"
VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)

echo "Creating RPM package for $TARGET version $VERSION"

# Install rpmbuild if not available
if ! command -v rpmbuild &> /dev/null; then
    echo "Installing rpm build tools..."
    sudo apt-get update
    sudo apt-get install -y rpm
fi

# Create RPM build directories
RPM_BUILD_DIR="$HOME/rpmbuild"
mkdir -p "$RPM_BUILD_DIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

# Copy spec file
cp packaging/rpm/smoldesk.spec "$RPM_BUILD_DIR/SPECS/"

# Create source tarball
SOURCE_DIR="smoldesk-$VERSION"
mkdir -p "/tmp/$SOURCE_DIR"

# Copy necessary files for build
cp -r src-tauri "/tmp/$SOURCE_DIR/"
cp -r signaling-server "/tmp/$SOURCE_DIR/"
cp -r packaging "/tmp/$SOURCE_DIR/"
cp -r docs "/tmp/$SOURCE_DIR/"
cp package.json package-lock.json "/tmp/$SOURCE_DIR/"

# Create tarball
cd /tmp
tar czf "$RPM_BUILD_DIR/SOURCES/smoldesk-$VERSION.tar.gz" "$SOURCE_DIR"
rm -rf "/tmp/$SOURCE_DIR"

# Update spec file with current version
sed -i "s/Version: .*/Version: $VERSION/" "$RPM_BUILD_DIR/SPECS/smoldesk.spec"

# Build RPM
cd "$RPM_BUILD_DIR"
rpmbuild -ba SPECS/smoldesk.spec

# Copy built RPM to output directory
PACKAGE_DIR="target/release/bundle/rpm"
mkdir -p "$PACKAGE_DIR"

if [ "$TARGET" = "aarch64-unknown-linux-gnu" ]; then
    ARCH="aarch64"
else
    ARCH="x86_64"
fi

cp "RPMS/$ARCH/smoldesk-$VERSION-1.$ARCH.rpm" "$PACKAGE_DIR/"

echo "RPM package created: $PACKAGE_DIR/smoldesk-$VERSION-1.$ARCH.rpm"
