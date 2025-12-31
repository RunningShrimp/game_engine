#!/bin/bash
# Icon Generation Script
#
# This script helps generate application icons for different platforms.
# You need to install ImageMagick first:
#   macOS: brew install imagemagick
#   Linux: sudo apt install imagemagick
#   Windows: Download from https://imagemagick.org/

INPUT_ICON="icon.png"
OUTPUT_DIR="."

echo "Generating application icons..."

# Check if input icon exists
if [ ! -f "$INPUT_ICON" ]; then
    echo "Error: $INPUT_ICON not found!"
    echo "Please create a 1024x1024 PNG icon named 'icon.png' first."
    exit 1
fi

# Generate PNG icons
echo "Generating PNG icons..."
convert $INPUT_ICON -resize 32x32 32x32.png
convert $INPUT_ICON -resize 128x128 128x128.png
convert $INPUT_ICON -resize 256x256 128x128@2x.png
convert $INPUT_ICON -resize 512x512 icon.png

# Generate macOS icon
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Generating macOS .icns..."
    mkdir -p icon.iconset
    convert $INPUT_ICON -resize 16x16     icon.iconset/icon_16x16.png
    convert $INPUT_ICON -resize 32x32     icon.iconset/icon_16x16@2x.png
    convert $INPUT_ICON -resize 32x32     icon.iconset/icon_32x32.png
    convert $INPUT_ICON -resize 64x64     icon.iconset/icon_32x32@2x.png
    convert $INPUT_ICON -resize 128x128   icon.iconset/icon_128x128.png
    convert $INPUT_ICON -resize 256x256   icon.iconset/icon_128x128@2x.png
    convert $INPUT_ICON -resize 256x256   icon.iconset/icon_256x256.png
    convert $INPUT_ICON -resize 512x512   icon.iconset/icon_256x256@2x.png
    convert $INPUT_ICON -resize 512x512   icon.iconset/icon_512x512.png
    convert $INPUT_ICON -resize 1024x1024 icon.iconset/icon_512x512@2x.png
    iconutil -c icns icon.iconset
    rm -rf icon.iconset
fi

# Generate Windows icon
echo "Generating Windows .ico..."
convert $INPUT_ICON -resize 256x256 -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico

# Generate high-res icon
convert $INPUT_ICON -resize 1024x1024 icon_1024.png

echo "Done! Icons generated in $OUTPUT_DIR"
echo ""
echo "Generated files:"
echo "  - 32x32.png"
echo "  - 128x128.png"
echo "  - 128x128@2x.png"
echo "  - icon.png (512x512)"
echo "  - icon_1024.png"
echo "  - icon.ico (Windows)"
echo "  - icon.icns (macOS)"
