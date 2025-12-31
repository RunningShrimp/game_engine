# Application Icons

This directory should contain the application icons in various sizes:

- 32x32.png - Small icon
- 128x128.png - Medium icon
- 128x128@2x.png - High DPI medium icon
- icon.icns - macOS icon
- icon.ico - Windows icon

You can generate these icons using tools like:
- https://icon.kitchen
- https://www.favicon-generator.org/
- ImageMagick: `convert icon.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico`
