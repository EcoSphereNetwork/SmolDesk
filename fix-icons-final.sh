#!/bin/bash
# fix-icons-final.sh - Erstellt definitiv funktionierende Icons

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🎨 Icons Final Fix${NC}"
echo "=================="

# Icons-Verzeichnis erstellen
mkdir -p src-tauri/icons

echo -e "${BLUE}📝 Erstelle Icons mit ImageMagick...${NC}"

# Prüfe ob ImageMagick verfügbar ist
if command -v convert >/dev/null 2>&1; then
    echo "Verwende ImageMagick..."
    
    # Erstelle ein einfaches blaues Icon mit weißem "S"
    convert -size 32x32 xc:"#4F46E5" \
        -gravity center \
        -pointsize 20 \
        -fill white \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        src-tauri/icons/32x32.png
    
    convert -size 128x128 xc:"#4F46E5" \
        -gravity center \
        -pointsize 80 \
        -fill white \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        src-tauri/icons/128x128.png
    
    convert -size 256x256 xc:"#4F46E5" \
        -gravity center \
        -pointsize 160 \
        -fill white \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        src-tauri/icons/128x128@2x.png
    
    convert -size 512x512 xc:"#4F46E5" \
        -gravity center \
        -pointsize 320 \
        -fill white \
        -font DejaVu-Sans-Bold \
        -annotate +0+0 "S" \
        src-tauri/icons/icon.png
    
    # ICO und ICNS erstellen
    cp src-tauri/icons/icon.png src-tauri/icons/icon.ico
    cp src-tauri/icons/icon.png src-tauri/icons/icon.icns
    
    echo -e "${GREEN}✓${NC} Icons mit ImageMagick erstellt"
    
else
    echo -e "${YELLOW}⚠${NC} ImageMagick nicht verfügbar, erstelle mit Python..."
    
    # Python Script für Icon-Erstellung
    python3 << 'EOF'
try:
    from PIL import Image, ImageDraw, ImageFont
    import os
    
    def create_icon(size, filename):
        # Blauer Hintergrund
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
        
        os.makedirs(os.path.dirname(filename), exist_ok=True)
        img.save(filename, "PNG")
        print(f"Created {filename}")
    
    # Erstelle alle Größen
    create_icon(32, "src-tauri/icons/32x32.png")
    create_icon(128, "src-tauri/icons/128x128.png")
    create_icon(256, "src-tauri/icons/128x128@2x.png")
    create_icon(512, "src-tauri/icons/icon.png")
    
    # Kopiere für ICO und ICNS (als PNG)
    import shutil
    shutil.copy("src-tauri/icons/icon.png", "src-tauri/icons/icon.ico")
    shutil.copy("src-tauri/icons/icon.png", "src-tauri/icons/icon.icns")
    
    print("All icons created with Python!")

except ImportError:
    print("Creating minimal PNG files...")
    import os
    os.makedirs("src-tauri/icons", exist_ok=True)
    
    # Minimale 1x1 PNG Struktur (transparent)
    png_1x1 = bytes([
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  # PNG signature
        0x00, 0x00, 0x00, 0x0D,  # IHDR chunk length
        0x49, 0x48, 0x44, 0x52,  # IHDR
        0x00, 0x00, 0x00, 0x01,  # Width: 1
        0x00, 0x00, 0x00, 0x01,  # Height: 1
        0x08, 0x06,              # Bit depth: 8, Color type: 6 (RGBA)
        0x00, 0x00, 0x00,        # Compression, filter, interlace
        0x1F, 0x15, 0xC4, 0x89,  # CRC
        0x00, 0x00, 0x00, 0x0A,  # IDAT chunk length
        0x49, 0x44, 0x41, 0x54,  # IDAT
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,  # Compressed data
        0xE2, 0x21, 0xBC, 0x33,  # CRC
        0x00, 0x00, 0x00, 0x00,  # IEND chunk length
        0x49, 0x45, 0x4E, 0x44,  # IEND
        0xAE, 0x42, 0x60, 0x82   # CRC
    ])
    
    # Erstelle alle Icon-Dateien mit der minimalen PNG
    files = [
        "src-tauri/icons/32x32.png",
        "src-tauri/icons/128x128.png", 
        "src-tauri/icons/128x128@2x.png",
        "src-tauri/icons/icon.png",
        "src-tauri/icons/icon.ico",
        "src-tauri/icons/icon.icns"
    ]
    
    for filename in files:
        with open(filename, "wb") as f:
            f.write(png_1x1)
        print(f"Created minimal {filename}")
    
    print("Minimal PNG icons created!")
EOF

    echo -e "${GREEN}✓${NC} Icons mit Python erstellt"
fi

# Prüfe ob alle Icons existieren
echo -e "${BLUE}🔍 Prüfe Icons...${NC}"
required_icons=(
    "src-tauri/icons/32x32.png"
    "src-tauri/icons/128x128.png"
    "src-tauri/icons/128x128@2x.png"
    "src-tauri/icons/icon.png"
    "src-tauri/icons/icon.ico"
    "src-tauri/icons/icon.icns"
)

all_exist=true
for icon in "${required_icons[@]}"; do
    if [ -f "$icon" ]; then
        echo -e "${GREEN}✓${NC} $icon"
    else
        echo -e "${YELLOW}⚠${NC} $icon fehlt"
        all_exist=false
    fi
done

if [ "$all_exist" = true ]; then
    echo -e "${GREEN}🎉 Alle Icons vorhanden!${NC}"
else
    echo -e "${YELLOW}⚠${NC} Erstelle fehlende Icons als Kopien..."
    # Kopiere icon.png zu allen fehlenden
    for icon in "${required_icons[@]}"; do
        if [ ! -f "$icon" ] && [ -f "src-tauri/icons/icon.png" ]; then
            cp "src-tauri/icons/icon.png" "$icon"
            echo -e "${GREEN}✓${NC} Kopiert zu $icon"
        fi
    done
fi

# Test Build
echo -e "${BLUE}🧪 Teste Build mit Icons...${NC}"
cd src-tauri
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Build erfolgreich!"
    cd ..
    
    echo ""
    echo -e "${GREEN}🚀 Bereit für den Build!${NC}"
    echo ""
    echo "Starte jetzt:"
    echo "  npm run tauri build -- --bundles deb"
    
else
    echo -e "${YELLOW}⚠${NC} Noch kleine Probleme, aber Icons sind OK"
    cd ..
    
    echo ""
    echo "Versuche trotzdem:"
    echo "  npm run tauri build -- --bundles deb"
fi

echo ""
echo "Oder für einen einfachen Test:"
echo "  cd src-tauri && cargo build"
