#!/bin/bash
# sign-packages.sh

echo "🔐 Signing packages..."

# GPG-Key sollte bereits vorhanden sein
GPG_KEY_ID="your-gpg-key-id"

cd dist

# Alle Pakete signieren
for file in *.deb *.rpm *.AppImage *.tar.gz; do
    echo "Signing $file..."
    gpg --detach-sign --armor --local-user $GPG_KEY_ID "$file"
done

echo "✅ All packages signed!"
