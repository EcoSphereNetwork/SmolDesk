#!/bin/bash
# generate-checksums.sh

cd dist

echo "🔐 Generating checksums..."

# SHA256 Checksums
sha256sum *.deb *.rpm *.AppImage *.tar.gz > SHA256SUMS

# MD5 Checksums (für Kompatibilität)
md5sum *.deb *.rpm *.AppImage *.tar.gz > MD5SUMS

echo "✅ Checksums generated!"
cat SHA256SUMS
