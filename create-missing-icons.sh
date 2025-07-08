#!/bin/bash
# create-missing-icons.sh - Erstellt fehlende Icons für SmolDesk

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🎨 Erstelle fehlende Icons für SmolDesk${NC}"
echo "==========================================="

# Icons-Verzeichnis erstellen
mkdir -p icons

# Prüfen ob ImageMagick verfügbar ist
if ! command -v convert >/dev/null 2>&1; then
    echo "ImageMagick nicht gefunden. Installiere es mit:"
    echo "sudo apt install imagemagick"
    exit 1
fi

# Logo als Basis verwenden (falls vorhanden)
if [ -f "docs/static/img/logo.png" ]; then
    BASE_IMAGE="docs/static/img/logo.png"
    echo -e "${GREEN}✓${NC} Verwende existierendes Logo als Basis"
else
    # Erstelle ein einfaches SVG-Logo
    echo -e "${BLUE}📝 Erstelle basis SVG-Logo...${NC}"
    cat > icons/logo.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512">
  <defs>
    <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#4F46E5;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#7C3AED;stop-opacity:1" />
    </linearGradient>
  </defs>
  <rect width="512" height="512" rx="64" fill="url(#grad1)"/>
  <g transform="translate(128,128)">
    <!-- Monitor -->
    <rect x="0" y="0" width="256" height="160" rx="16" fill="white" opacity="0.9"/>
    <rect x="16" y="16" width="224" height="128" rx="8" fill="#1F2937"/>
    
    <!-- Screen content -->
    <rect x="32" y="32" width="64" height="96" rx="4" fill="#10B981"/>
    <rect x="112" y="32" width="64" height="96" rx="4" fill="#F59E0B"/>
    <rect x="192" y="32" width="64" height="96" rx="4" fill="#EF4444"/>
    
    <!-- Connection lines -->
    <path d="M 128 180 Q 180 200 230 180" stroke="white" stroke-width="4" fill="none" opacity="0.7"/>
    <circle cx="128" cy="180" r="6" fill="white"/>
    <circle cx="230" cy="180" r="6" fill="white"/>
    
    <!-- Text -->
    <text x="128" y="220" text-anchor="middle" fill="white" font-family="Arial, sans-serif" font-size="24" font-weight="bold">SmolDesk</text>
  </g>
</svg>
EOF

    # SVG zu PNG konvertieren
    convert icons/logo.svg -background transparent icons/logo_512.png
    BASE_IMAGE="icons/logo_512.png"
    echo -e "${GREEN}✓${NC} Basis-Logo erstellt"
fi

# Alle benötigten Icon-Größen erstellen
echo -e "${BLUE}📐 Erstelle Icon-Größen...${NC}"

# PNG Icons
convert "$BASE_IMAGE" -resize 32x32 icons/32x32.png
convert "$BASE_IMAGE" -resize 128x128 icons/128x128.png
convert "$BASE_IMAGE" -resize 256x256 icons/128x128@2x.png
convert "$BASE_IMAGE" -resize 512x512 icons/icon.png

echo -e "${GREEN}✓${NC} PNG Icons erstellt"

# ICO für Windows
if command -v icotool >/dev/null 2>&1; then
    # Verwende icotool wenn verfügbar
    icotool -c -o icons/icon.ico icons/32x32.png icons/128x128.png
    echo -e "${GREEN}✓${NC} Windows ICO erstellt (mit icotool)"
else
    # Fallback mit ImageMagick
    convert icons/32x32.png icons/128x128.png icons/icon.ico
    echo -e "${GREEN}✓${NC} Windows ICO erstellt (mit ImageMagick)"
fi

# ICNS für macOS
if command -v png2icns >/dev/null 2>&1; then
    png2icns icons/icon.icns icons/512x512.png
    echo -e "${GREEN}✓${NC} macOS ICNS erstellt (mit png2icns)"
else
    # Verwende ImageMagick als Fallback
    convert "$BASE_IMAGE" -resize 512x512 icons/icon.icns
    echo -e "${GREEN}✓${NC} macOS ICNS erstellt (mit ImageMagick)"
fi

# Kopiere Logo auch zu docs/static/img/ falls das Verzeichnis existiert
mkdir -p docs/static/img
cp icons/icon.png docs/static/img/logo.png

echo ""
echo -e "${GREEN}🎉 Alle Icons erfolgreich erstellt!${NC}"
echo ""
echo "Erstellte Dateien:"
echo "  icons/32x32.png"
echo "  icons/128x128.png"
echo "  icons/128x128@2x.png"
echo "  icons/icon.png"
echo "  icons/icon.ico"
echo "  icons/icon.icns"
echo "  docs/static/img/logo.png"
echo ""
echo "Du kannst jetzt den Build fortsetzen."
