# Installation Guide

Detailed installation instructions for all platforms.

## Requirements

### System Requirements

- **Operating System**:
  - Linux (Ubuntu 20.04+, Fedora 35+, Arch Linux)
  - macOS (11.0 Big Sur or later)
  - Windows 10/11 with MSVC

- **Hardware**:
  - CPU: Dual-core processor or better
  - RAM: 4GB minimum, 8GB recommended
  - GPU: OpenGL 3.3+ or Vulkan 1.1+ capable
  - Storage: 500MB for engine, additional space for projects

### Software Requirements

- **Rust**: 1.70 or later ([Install via rustup](https://rustup.rs/))
- **Git**: For cloning the repository
- **C Compiler**: For building some dependencies

## Platform-Specific Instructions

### Linux

#### Ubuntu/Debian

```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libx11-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libvulkan-dev \
    pkg-config \
    git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/yourusername/game_engine.git
cd game_engine
cargo build --release
```

#### Fedora

```bash
# Install system dependencies
sudo dnf install -y \
    gcc-c++ \
    libX11-devel \
    libXrandr-devel \
    libXi-devel \
    mesa-libGL-devel \
    vulkan-devel \
    pkg-config \
    git

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/yourusername/game_engine.git
cd game_engine
cargo build --release
```

#### Arch Linux

```bash
# Install system dependencies
sudo pacman -S \
    base-devel \
    libx11 \
    libxrandr \
    libxi \
    mesa \
    vulkan-devel \
    git

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/yourusername/game_engine.git
cd game_engine
cargo build --release
```

### macOS

#### Using Homebrew

```bash
# Install Xcode command line tools
xcode-select --install

# Install dependencies via Homebrew
brew install rustup-init git

# Initialize Rust
rustup-init

# Clone and build
git clone https://github.com/yourusername/game_engine.git
cd game_engine
cargo build --release
```

#### Manual Installation

1. Install Xcode from the App Store
2. Install Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```
3. Install Rust via rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```
4. Clone and build:
   ```bash
   git clone https://github.com/yourusername/game_engine.git
   cd game_engine
   cargo build --release
   ```

### Windows

#### Using Visual Studio

1. Install [Visual Studio 2022](https://visualstudio.microsoft.com/downloads/)
   - Select "Desktop development with C++" workload
   - Include Windows 10/11 SDK

2. Install [Rust via rustup](https://rustup.rs/)

3. Install Git from [git-scm.com](https://git-scm.com/download/win)

4. Clone and build using PowerShell or Command Prompt:
   ```cmd
   git clone https://github.com/yourusername/game_engine.git
   cd game_engine
   cargo build --release
   ```

#### Using MSYS2

```bash
# Install MSYS2 from https://www.msys2.org/

# In MSYS2 terminal, install dependencies
pacman -S \
    base-devel \
    mingw-w64-x86_64-toolchain \
    mingw-w64-x86_64-cmake \
    mingw-w64-x86_64-vulkan-devel \
    git

# Add Rust to PATH in MSYS2
export PATH="$PATH:/c/Users/$USER/.cargo/bin"

# Clone and build
git clone https://github.com/yourusername/game_engine.git
cd game_engine
cargo build --release
```

## Verification

After installation, verify everything is working:

```bash
# Run tests
cargo test --workspace

# Run a simple example
cargo run --example hello_world

# Build documentation
cargo doc --workspace --no-deps --all-features

# Run benchmarks
cargo bench --workspace
```

## Development Setup

### VS Code Setup

1. Install [VS Code](https://code.visualstudio.com/)
2. Install extensions:
   - rust-analyzer
   - CodeLLDB (debugger)
   - Even Better TOML
   - MarkdownLint

3. Configure VS Code (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy",
     "files.watcherExclude": {
       "**/target": true
     }
   }
   ```

### Other Editors

- **Vim/Neovim**: Install rust.vim and coc-rust-analyzer
- **Emacs**: Use rust-mode and lsp-mode
- **JetBrains CLion**: Install the Rust plugin

## Environment Variables

Optional environment variables for customization:

```bash
# Set Rust toolchain
export RUSTUP_TOOLCHAIN=stable

# Enable backtraces for debugging
export RUST_BACKTRACE=1

# Use more parallel jobs for compilation
export CARGO_BUILD_JOBS=8

# Enable nightly features (if needed)
export RUSTUP_TOOLCHAIN=nightly
```

## Feature Flags

The engine supports optional features via Cargo features:

```toml
[dependencies.game_engine]
version = "0.1"
features = [
    "rendering",    # Rendering system (default)
    "physics",      # Physics simulation (default)
    "audio",        # Audio system (default)
    "networking",   # Multiplayer networking (default)
    "profiling",    # Performance profiling
    "tracing",      # Detailed tracing logs
    "serde",        # Serialization support
]
```

### Minimal Installation

For a minimal engine without optional features:

```bash
cargo build --no-default-features
```

### Full-Featured Installation

For all features including development tools:

```bash
cargo build --all-features
```

## Cross-Compilation

### Cross-Compile from Linux to Windows

```bash
# Install cross-compilation tools
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64

# Build for Windows
cargo build --target x86_64-pc-windows-gnu --release
```

### Cross-Compile from Linux to macOS

```bash
# Install osxcross (requires macOS SDK)
git clone https://github.com tpoechtrager/osxcross.git
cd osxcross
./build.sh

# Build for macOS
cargo build --target x86_64-apple-darwin --release
```

### WebAssembly (WASM)

See [WASM Build Guide](./guides/wasm_build_guide.md) for detailed instructions:

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Build for WASM
wasm-pack build --target web
```

## Troubleshooting

### Common Issues

#### "Linker `cc` not found"

**Solution**: Install a C compiler:
- Linux: `sudo apt-get install build-essential`
- macOS: Install Xcode Command Line Tools
- Windows: Install Visual Studio Build Tools

#### "Vulkan headers not found"

**Solution**: Install Vulkan development headers:
- Linux: `sudo apt-get install libvulkan-dev`
- macOS: Install via Homebrew: `brew install vulkan-headers`
- Windows: Install Vulkan SDK from [lunarg.com](https://vulkan.lunarg.com/)

#### Out of Memory During Build

**Solution**: Limit parallel jobs:
```bash
cargo build --jobs 2
```

#### Slow Compilation

**Solution**: Use sccache for incremental compilation:
```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Getting Help

If you encounter other issues:

- [Troubleshooting Guide](./troubleshooting.md)
- [FAQ](./faq.md)
- [GitHub Issues](https://github.com/yourusername/game_engine/issues)

## Next Steps

After successful installation:

1. Read the [Quick Start Guide](./quickstart.md)
2. Explore [Examples](./examples.md)
3. Learn about [Architecture](./architecture/overview.md)
4. Review [Best Practices](./best_practices.md)

## Uninstallation

To remove the engine:

```bash
# Remove source code
rm -rf game_engine

# Remove cargo installation
cargo uninstall game_engine

# Clean build artifacts
cargo clean
```

To remove Rust entirely:
```bash
rustup self uninstall
```
