# Package Format Specification

## Overview

This document defines the standard package format for game engine resources, plugins, and templates distributed through the marketplace.

## Package Structure

### Basic Structure

```
package-name/
├── package.toml           # Required: Package metadata
├── README.md              # Required: Documentation
├── LICENSE                # Required: License file
├── CHANGELOG.md           # Optional: Version history
├── manifest.json          # Optional: Asset manifest
├── preview/               # Optional: Preview images
│   ├── thumbnail.png
│   └── screenshots/
├── assets/                # Optional: Game assets
├── scripts/               # Optional: Scripts
├── data/                  # Optional: Data files
├── config/                # Optional: Configuration
└── install/               # Optional: Installation hooks
    └── install.lua
```

## Metadata Format

### package.toml

The `package.toml` file is the main metadata file for a package.

```toml
## Package Identification
[package]
name = "my-awesome-package"
version = "1.2.3"
display_name = "My Awesome Package"
description = """
A multi-line description of the package.
Can include detailed information about features,
usage instructions, and examples.
"""
type = "asset-pack"  # asset-pack, plugin, template, script

## Author Information
[author]
name = "Your Name"
email = "your.email@example.com"
website = "https://example.com"
organization = "Your Company"

## Licensing
[license]
type = "MIT"  # MIT, Apache-2.0, GPL-3.0, etc.
file = "LICENSE"

## Engine Compatibility
[engine]
version = ">=0.1.0,<2.0.0"
features = ["renderer", "physics", "audio"]

## Package Metadata
[metadata]
category = "textures"
tags = ["sci-fi", "metal", "pbr"]
keywords = ["texture", "material", "pbr"]
rating = 4.5
downloads = 1234
created = "2024-01-01T00:00:00Z"
updated = "2024-01-15T00:00:00Z"

## Dependencies
[dependencies]
# Format: package-name = "version requirement"
core-textures = "^1.0.0"
physics-plugin = "~2.5.0"
optional-plugin = ">=3.0.0"  # Marked optional via [optional-dependencies]

## Optional Dependencies
[optional-dependencies]
optional-plugin = true

## Assets
[assets]
# Define asset categories and patterns
textures = [
    "assets/textures/**/*.png",
    "assets/textures/**/*.jpg"
]
models = [
    "assets/models/**/*.gltf",
    "assets/models/**/*.glb"
]
audio = [
    "assets/audio/**/*.wav",
    "assets/audio/**/*.mp3"
]
shaders = [
    "assets/shaders/**/*.vert",
    "assets/shaders/**/*.frag",
    "assets/shaders/**/*.glsl"
]
animations = [
    "assets/animations/**/*.gltf"
]
fonts = [
    "assets/fonts/**/*.ttf"
]

## Scripts
[scripts]
# Define script files to load
lua = ["scripts/**/*.lua"]
rust = ["scripts/**/*.rs"]  # For plugins

## Installation
[install]
# Installation configuration
copy_files = true
create_directories = true
run_script = "install/install.lua"
verify_checksums = true

## Previews
[preview]
thumbnail = "preview/thumbnail.png"
screenshots = [
    "preview/screenshots/1.png",
    "preview/screenshots/2.png"
]
video = "preview/demo.mp4"

## Build
[build]
# For plugins that need compilation
compiler = "rust"
version = "1.70.0"
features = ["default"]
profile = "release"

## Checksums
[checksums]
algorithm = "sha256"
# Individual file checksums
[checksums.files]
"assets/textures/diffuse.png" = "abc123..."
"assets/models/character.gltf" = "def456..."

## Publishing
[publish]
# Marketplace publishing settings
visibility = "public"  # public, private, unlisted
price = 0.0  # Free
currency = "USD"

## Marketplace
[marketplace]
# Marketplace-specific metadata
featured = false
trending = false
verified = true
```

## Package Types

### 1. Asset Pack

Contains game assets like textures, models, audio, etc.

```toml
[package]
type = "asset-pack"
name = "sci-fi-textures"
version = "1.0.0"

[assets]
textures = ["assets/textures/**/*.png"]
materials = ["assets/materials/*.toml"]
```

### 2. Plugin

Contains executable code that extends the engine.

```toml
[package]
type = "plugin"
name = "custom-physics"
version = "1.0.0"

[build]
compiler = "rust"
target = "x86_64-unknown-linux-gnu"

[scripts]
rust = ["src/lib.rs"]
```

### 3. Template

Contains project or scene templates.

```toml
[package]
type = "template"
name = "fps-game-template"
version = "1.0.0"

[template]
type = "project"  # project, scene, entity
base_engine = "0.1.0"
includes_scripts = true
includes_assets = true
```

### 4. Script Collection

Contains reusable scripts.

```toml
[package]
type = "scripts"
name = "utility-scripts"
version = "1.0.0"

[scripts]
lua = ["scripts/**/*.lua"]
```

## Asset Manifest

### manifest.json

Optional JSON file that provides detailed asset metadata.

```json
{
  "version": "1.0",
  "assets": [
    {
      "id": "texture_001",
      "type": "texture",
      "path": "assets/textures/diffuse.png",
      "format": "png",
      "width": 1024,
      "height": 1024,
      "channels": 4,
      "compression": "none",
      "tags": ["diffuse", "albedo"],
      "metadata": {
        "color_space": "sRGB",
        "mipmaps": true
      }
    },
    {
      "id": "model_001",
      "type": "model",
      "path": "assets/models/character.gltf",
      "format": "gltf",
      "vertices": 5432,
      "triangles": 2716,
      "animations": ["idle", "walk", "run"],
      "tags": ["character", "humanoid"]
    }
  ]
}
```

## Installation Hooks

### install.lua

Lua scripts that run during installation.

```lua
-- Install script for package

function on_pre_install(context)
    print("Installing " .. context.package_name)
    print("Version: " .. context.package_version)
end

function on_install(context)
    -- Create directories
    os.execute("mkdir -p " .. context.install_path .. "/custom")

    -- Copy files
    os.execute("cp -r assets/* " .. context.install_path .. "/assets/")
end

function on_post_install(context)
    print("Installation complete!")
    print("Installed to: " .. context.install_path)
end

function on_uninstall(context)
    print("Uninstalling " .. context.package_name)
    -- Cleanup
end

function on_upgrade(context)
    print("Upgrading from " .. context.old_version .. " to " .. context.new_version)
end
```

## Versioning

### Semantic Versioning

Packages MUST follow Semantic Versioning 2.0.0:

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- MAJOR: Incompatible changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

### Pre-release Versions

```toml
version = "1.0.0-alpha.1"
version = "1.0.0-beta.1"
version = "1.0.0-rc.1"
```

### Build Metadata

```toml
version = "1.0.0+20130313144700"
```

## Dependency Version Constraints

### Caret (^)

Compatible with version, excluding major updates:

```toml
[dependencies]
package = "^1.2.3"  # >=1.2.3, <2.0.0
```

### Tilde (~)

Pin to minor version:

```toml
[dependencies]
package = "~1.2.3"  # >=1.2.3, <1.3.0
```

### Greater/Less Than

```toml
[dependencies]
package = ">=1.0.0"
package = "<2.0.0"
package = ">=1.0.0,<2.0.0"
```

### Wildcard

```toml
[dependencies]
package = "*"       # Any version
package = "1.*"     # >=1.0.0,<2.0.0
package = "1.2.*"   # >=1.2.0,<1.3.0
```

### Exact

```toml
[dependencies]
package = "=1.2.3"  # Exactly 1.2.3
```

## Checksums and Verification

### SHA256 Checksums

```toml
[checksums]
algorithm = "sha256"

[checksums.files]
"assets/texture.png" = "a1b2c3d4..."
"assets/model.gltf" = "e5f6g7h8..."
```

### GPG Signature

```bash
# Sign the package
gpg --detach-sign --armor package.tar.gz

# Verify the signature
gpg --verify package.tar.gz.asc package.tar.gz
```

## File Naming Conventions

### General Rules

- Use lowercase letters, numbers, and hyphens
- Avoid spaces and special characters
- Use descriptive names
- Keep names under 255 characters

### Examples

Good:
- `sci-fi-textures-pack`
- `character-model-v2`
- `utility-scripts`

Bad:
- `Sci Fi Textures Pack`
- `char_model_v2`
- `utils!!!`

## Compression

### Supported Formats

- **tar.gz** (default)
- **tar.bz2**
- **tar.xz**
- **zip**

### Compression Recommendation

For maximum compatibility:
```bash
tar -czf package.tar.gz package-directory/
```

## Metadata Fields Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| name | string | Unique package identifier |
| version | string | Semantic version |
| type | string | Package type |
| description | string | Package description |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| display_name | string | Human-readable name |
| author.name | string | Author name |
| author.email | string | Author email |
| license.type | string | License identifier |
| engine.version | string | Engine version requirement |
| dependencies | table | Package dependencies |

## Validation

### Required Files

- `package.toml` - Metadata
- `README.md` - Documentation
- `LICENSE` - License file

### Validation Checks

1. **Structure**: All required files present
2. **Syntax**: Valid TOML/JSON
3. **Version**: Valid semantic version
4. **Dependencies**: Resolvable dependencies
5. **Checksums**: Valid file checksums
6. **License**: Valid SPDX identifier
7. **Engine**: Compatible engine version

### Validation Tool

```bash
# Validate package
marketplace validate ./package-directory

# Validate package archive
marketplace validate ./package.tar.gz

# Detailed validation
marketplace validate --verbose ./package-directory
```

## Best Practices

1. **Clear Naming**: Use descriptive, searchable names
2. **Documentation**: Provide comprehensive README
3. **Versioning**: Follow semantic versioning
4. **Dependencies**: Minimize external dependencies
5. **Testing**: Test on multiple engine versions
6. **Licensing**: Use standard, compatible licenses
7. **Metadata**: Fill out all relevant fields
8. **Previews**: Include high-quality previews
9. **Changelog**: Maintain version history
10. **Examples**: Provide usage examples

## Examples

Complete example packages are available at:
- `/examples/packages/asset-pack/`
- `/examples/packages/plugin/`
- `/examples/packages/template/`
