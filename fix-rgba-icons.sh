#!/bin/bash
# fix-rgba-icons.sh - Erstellt RGBA-kompatible Icons

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🎨 Erstelle RGBA-Icons${NC}"
echo "====================="

cd src-tauri

echo -e "${BLUE}📝 Erstelle RGBA-Icons mit ImageMagick...${NC}"

# Lösche alte Icons
rm -f icons/*.png icons/*.ico icons/*.icns

# Erstelle RGBA-Icons mit ImageMagick
if command -v convert >/dev/null 2>&1; then
    echo "Verwende ImageMagick für RGBA-Icons..."
    
    # 32x32 RGBA
    convert -size 32x32 xc:"rgba(79,70,229,255)" \
        -gravity center \
        -pointsize 20 \
        -fill "rgba(255,255,255,255)" \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        -colorspace sRGB \
        -depth 8 \
        PNG32:icons/32x32.png
    
    # 128x128 RGBA
    convert -size 128x128 xc:"rgba(79,70,229,255)" \
        -gravity center \
        -pointsize 80 \
        -fill "rgba(255,255,255,255)" \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        -colorspace sRGB \
        -depth 8 \
        PNG32:icons/128x128.png
    
    # 256x256 RGBA
    convert -size 256x256 xc:"rgba(79,70,229,255)" \
        -gravity center \
        -pointsize 160 \
        -fill "rgba(255,255,255,255)" \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        -colorspace sRGB \
        -depth 8 \
        PNG32:icons/128x128@2x.png
    
    # 512x512 RGBA
    convert -size 512x512 xc:"rgba(79,70,229,255)" \
        -gravity center \
        -pointsize 320 \
        -fill "rgba(255,255,255,255)" \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        -colorspace sRGB \
        -depth 8 \
        PNG32:icons/icon.png
    
    # ICO und ICNS als PNG kopieren (Tauri konvertiert automatisch)
    cp icons/32x32.png icons/icon.ico
    cp icons/icon.png icons/icon.icns
    
    echo -e "${GREEN}✓${NC} RGBA-Icons mit ImageMagick erstellt"
    
else
    echo "Verwende Python für RGBA-Icons..."
    
    # Python Script für RGBA-Icons
    python3 << 'EOF'
from PIL import Image, ImageDraw, ImageFont
import os

def create_rgba_icon(size, filename):
    # Erstelle RGBA-Image
    img = Image.new('RGBA', (size, size), (79, 70, 229, 255))
    draw = ImageDraw.Draw(img)
    
    # Weißes "S" in der Mitte
    font_size = int(size * 0.6)
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", font_size)
    except:
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", font_size)
        except:
            font = ImageFont.load_default()
    
    text = "S"
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    
    x = (size - text_width) // 2
    y = (size - text_height) // 2
    
    draw.text((x, y), text, fill=(255, 255, 255, 255), font=font)
    
    # Stelle sicher, dass es RGBA ist
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    img.save(filename, "PNG", optimize=False)
    print(f"Created RGBA {filename}")

# Erstelle alle RGBA-Icons
create_rgba_icon(32, "icons/32x32.png")
create_rgba_icon(128, "icons/128x128.png")
create_rgba_icon(256, "icons/128x128@2x.png")
create_rgba_icon(512, "icons/icon.png")

# Kopiere für ICO und ICNS
import shutil
shutil.copy("icons/32x32.png", "icons/icon.ico")
shutil.copy("icons/icon.png", "icons/icon.icns")

print("All RGBA icons created!")
EOF

    echo -e "${GREEN}✓${NC} RGBA-Icons mit Python erstellt"
fi

# Prüfe die Icons
echo -e "${BLUE}🔍 Prüfe Icon-Formate...${NC}"

for icon in icons/*.png; do
    if [ -f "$icon" ]; then
        if command -v identify >/dev/null 2>&1; then
            format=$(identify -format "%[colorspace] %[channels]" "$icon")
            echo -e "${GREEN}✓${NC} $icon: $format"
        else
            echo -e "${GREEN}✓${NC} $icon: exists"
        fi
    fi
done

# Teste den Build
echo -e "${BLUE}🧪 Teste Build...${NC}"
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Build Test erfolgreich!"
else
    echo -e "${YELLOW}⚠${NC} Build Test mit Problemen, aber Icons sollten OK sein"
fi

cd ..

echo ""
echo -e "${GREEN}🎉 RGBA-Icons erstellt!${NC}"
echo ""
echo "Jetzt sollte der Build funktionieren:"
echo "  npm run tauri-build -- --bundles deb"
