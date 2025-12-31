# Resource Marketplace Design

## Overview

The Resource Marketplace is an ecosystem for distributing and managing game assets, plugins, and tools. It provides a centralized platform for creators to share resources and for developers to discover and integrate them into their projects.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                    Marketplace Client                    │
│  - Search & Browse                                      │
│  - Download & Install                                   │
│  - Update Management                                    │
│  - Dependency Resolution                                │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                      Local Cache                         │
│  - Downloaded Packages                                  │
│  - Metadata Storage                                     │
│  - Version Information                                  │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                    Marketplace Server                    │
│  - Package Repository                                   │
│  - User Authentication                                  │
│  - Reviews & Ratings                                    │
│  - Analytics                                            │
└─────────────────────────────────────────────────────────┘
```

### Package Types

1. **Asset Packs**
   - Textures
   - 3D Models
   - Audio files
   - Shaders
   - Animations

2. **Plugins**
   - Gameplay systems
   - Tools
   - Extensions
   - Integrations

3. **Templates**
   - Project templates
   - Scene templates
   - Entity templates

4. **Scripts**
   - Lua scripts
   - Shader code
   - Configuration files

## API Specification

### Authentication

```rust
pub struct MarketplaceAuth {
    pub api_key: String,
    pub user_token: Option<String>,
}

impl MarketplaceAuth {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            user_token: None,
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        // Implementation
    }

    pub fn logout(&mut self) {
        self.user_token = None;
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_token.is_some()
    }
}
```

### Search API

```rust
pub struct SearchQuery {
    pub keywords: Vec<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub price_min: Option<f32>,
    pub price_max: Option<f32>,
    pub rating_min: Option<f32>,
    pub engine_version: Option<String>,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub enum SortField {
    Name,
    Downloads,
    Rating,
    Updated,
    Created,
    Price,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}
```

### Package API

```rust
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub category: String,
    pub tags: Vec<String>,
    pub price: Option<f32>,
    pub rating: f32,
    pub downloads: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub engine_version: String,
    pub dependencies: Vec<PackageDependency>,
    pub thumbnail_url: String,
    pub screenshots: Vec<String>,
}

pub struct PackageDependency {
    pub package_id: String,
    pub version_requirement: String,
}
```

### Download API

```rust
pub struct DownloadOptions {
    pub include_dependencies: bool,
    pub verify_checksums: bool,
    pub show_progress: bool,
    pub target_directory: PathBuf,
}

pub struct DownloadProgress {
    pub package_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed: f32, // bytes per second
    pub eta: Duration,
}
```

## Package Format

### Directory Structure

```
example-asset-pack/
├── package.toml          # Package metadata
├── README.md             # Documentation
├── LICENSE               # License file
├── preview.png           # Preview image
├── assets/
│   ├── textures/
│   │   └── *.png
│   ├── models/
│   │   └── *.gltf
│   ├── audio/
│   │   └── *.wav
│   └── shaders/
│       └── *.glsl
└── scripts/
    └── *.lua
```

### Metadata Format

```toml
[package]
name = "example-asset-pack"
version = "1.0.0"
display_name = "Example Asset Pack"
description = "A comprehensive set of game assets"
author = "Your Name"
email = "your.email@example.com"
license = "MIT"
license_file = "LICENSE"
website = "https://example.com"
repository = "https://github.com/example/asset-pack"
category = "asset-pack"
tags = ["textures", "models", "audio"]

[engine]
version = ">=0.1.0"
features = ["renderer", "audio"]

[dependencies]
# Format: package-name = "version requirement"
other-package = "^1.0.0"

[assets]
# Asset file patterns
textures = ["assets/textures/**/*.png"]
models = ["assets/models/**/*.gltf"]
audio = ["assets/audio/**/*.wav"]
shaders = ["assets/shaders/**/*.glsl"]

[scripts]
# Script files to load
lua = ["scripts/**/*.lua"]

[install]
# Installation instructions
copy_files = true
run_script = "scripts/install.lua"

[preview]
# Preview images
thumbnail = "preview.png"
screenshots = ["screenshots/*.png"]
```

## Version Management

### Semantic Versioning

Packages follow Semantic Versioning 2.0.0:
- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Version Constraints

```
^1.2.3  >= 1.2.3, < 2.0.0
~1.2.3  >= 1.2.3, < 1.3.0
>=1.2.3 >= 1.2.3
<2.0.0  < 2.0.0
*       Any version
```

### Update Strategy

```rust
pub enum UpdateStrategy {
    /// Update to latest compatible version
    LatestCompatible,
    /// Update to latest version (including major)
    Latest,
    /// Don't update automatically
    Manual,
    /// Update only within current major version
    SameMajor,
}
```

## Dependency Resolution

### Dependency Graph

The marketplace uses a dependency graph to resolve package dependencies:

```
Package A (1.0.0)
├── Package B (>=1.0.0)
│   └── Package D (^2.0.0)
└── Package C (~1.5.0)
    └── Package D (^2.0.0)
```

### Resolution Algorithm

1. Collect all dependencies
2. Check for version conflicts
3. Find compatible versions
4. Create installation plan
5. Verify no circular dependencies

## Installation Process

### Steps

1. **Download**: Download package to cache
2. **Verify**: Check checksums and signatures
3. **Extract**: Extract to temporary directory
4. **Validate**: Validate package structure
5. **Resolve**: Resolve dependencies
6. **Install**: Copy to target directory
7. **Register**: Register in package registry
8. **Cleanup**: Remove temporary files

### Rollback

If installation fails:
1. Remove partially installed files
2. Restore previous versions
3. Clean up temporary files
4. Report error to user

## Security

### Code Signing

Packages can be signed using GPG:
```bash
gpg --detach-sign --armor package.tar.gz
```

### Checksums

SHA256 checksums for all files:
```toml
[checksums]
algorithm = "sha256"

[checksums.files]
"assets/textures/diffuse.png" = "abc123..."
"assets/models/character.gltf" = "def456..."
```

### Sandboxing

Scripts run in a sandboxed environment:
- Limited file system access
- Network access restrictions
- CPU and memory limits
- Timeout enforcement

## API Client Implementation

```rust
use std::path::PathBuf;
use crate::error::Result;

pub struct MarketplaceClient {
    base_url: String,
    auth: MarketplaceAuth,
    cache_dir: PathBuf,
}

impl MarketplaceClient {
    pub fn new(base_url: String, cache_dir: PathBuf) -> Self {
        Self {
            base_url,
            auth: MarketplaceAuth::new(String::new()),
            cache_dir,
        }
    }

    /// Search for packages
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<PackageInfo>> {
        // Implementation
        Ok(vec![])
    }

    /// Get package details
    pub async fn get_package(&self, id: &str) -> Result<PackageInfo> {
        // Implementation
        Ok(PackageInfo {
            id: id.to_string(),
            // ... other fields
        })
    }

    /// Download a package
    pub async fn download_package(
        &self,
        id: &str,
        options: DownloadOptions,
        progress_callback: impl Fn(DownloadProgress),
    ) -> Result<PathBuf> {
        // Implementation
        Ok(PathBuf::from(""))
    }

    /// Install a package
    pub async fn install_package(
        &self,
        id: &str,
        version: &str,
        options: DownloadOptions,
    ) -> Result<()> {
        // Implementation
        Ok(())
    }

    /// Update a package
    pub async fn update_package(
        &self,
        id: &str,
        strategy: UpdateStrategy,
    ) -> Result<()> {
        // Implementation
        Ok(())
    }

    /// Uninstall a package
    pub async fn uninstall_package(&self, id: &str) -> Result<()> {
        // Implementation
        Ok(())
    }

    /// Check for updates
    pub async fn check_updates(&self) -> Result<Vec<PackageInfo>> {
        // Implementation
        Ok(vec![])
    }
}
```

## CLI Commands

```
# Search for packages
marketplace search "textures" --category assets

# Get package info
marketplace info example-asset-pack

# Install a package
marketplace install example-asset-pack --version 1.0.0

# Update all packages
marketplace update --all

# Update specific package
marketplace update example-asset-pack

# Uninstall a package
marketplace uninstall example-asset-pack

# List installed packages
marketplace list

# Check for updates
marketplace check-updates
```

## Best Practices

### For Package Authors

1. **Use semantic versioning**: Follow SemVer strictly
2. **Document dependencies**: Be explicit about version requirements
3. **Provide examples**: Include usage examples in README
4. **Test thoroughly**: Test on multiple engine versions
5. **Maintain compatibility**: Don't break existing APIs
6. **License clearly**: Use standard licenses
7. **Update frequently**: Keep packages up-to-date

### For Package Users

1. **Pin versions**: Use specific versions in production
2. **Review changes**: Check changelogs before updating
3. **Test updates**: Test in development first
4. **Report issues**: Report bugs to package authors
5. **Contribute back**: Submit improvements upstream
6. **Check licenses**: Verify license compatibility
7. **Backup data**: Backup before major updates

## Future Enhancements

- [ ] Package reviews and ratings
- [ ] User profiles and portfolios
- [ ] Payment processing for paid assets
- [ ] Automatic dependency scanning
- [ ] Package analytics
- [ ] CI/CD integration
- [ ] Package deprecation system
- [ ] Monetization options
- [ ] Community features
- [ ] Integration with version control
