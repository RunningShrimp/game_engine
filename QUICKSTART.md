# Game Engine Quick Start Guide

Welcome to the Game Engine! This guide will help you get up and running quickly.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Project Structure](#project-structure)
- [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Building](#building)
- [Examples](#examples)
- [Common Tasks](#common-tasks)
- [Troubleshooting](#troubleshooting)
- [Resources](#resources)

---

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** 1.85.0 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **Git** ([Install Git](https://git-scm.com/downloads))
- **C Compiler** (for some dependencies)
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: `build-essential` package
  - Windows: [MSVC Build Tools](https://visualstudio.microsoft.com/downloads/)

### Optional Tools

- **VSCode** with rust-analyzer extension for the best development experience
- **cargo-watch** for automatic rebuilding during development
- **cargo-audit** for security checks

```bash
# Install optional tools
cargo install cargo-watch cargo-audit
```

---

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd game_engine
```

### 2. Verify Toolchain

The project uses `rust-toolchain.toml` to ensure the correct Rust version. Verify:

```bash
rustc --version
cargo --version
```

### 3. Build the Project

```bash
# Debug build (faster compilation)
cargo build --workspace

# Release build (optimized)
cargo build --workspace --release
```

### 4. Run Tests

Verify everything works by running the test suite:

```bash
cargo test --workspace
```

---

## Project Structure

```
game_engine/
├── game_engine/           # Main engine library
│   ├── src/
│   │   ├── ai/           # AI systems (behavior trees, pathfinding)
│   │   ├── audio/        # Audio engine
│   │   ├── core/         # Core engine components
│   │   ├── ecs/          # Entity Component System
│   │   ├── input/        # Input handling
│   │   ├── physics/      # Physics engine
│   │   ├── rendering/    # Rendering system
│   │   ├── resources/    # Resource management
│   │   └── scripting/    # Scripting support
│   └── Cargo.toml
├── game_engine_macros/   # Procedural macros
├── game_engine_profiling/# Profiling tools
├── examples/             # Example projects
├── scripts/              # Development scripts
├── docs/                 # Documentation
└── tests/                # Integration tests
```

---

## Quick Start

### Your First Game

Create a new file `examples/my_first_game.rs`:

```rust
use game_engine::prelude::*;

fn main() -> GameResult {
    // Initialize the game
    let mut game = Game::new(GameConfig {
        title: "My First Game".to_string(),
        window_size: (800, 600),
        ..Default::default()
    })?;

    // Run the game loop
    game.run(|world| {
        // Your game logic here
        Ok(())
    })
}
```

Run your game:

```bash
cargo run --example my_first_game
```

---

## Development Workflow

### Using Development Scripts

The project includes helpful scripts in the `scripts/` directory:

```bash
# Start development environment (auto-rebuild on changes)
./scripts/dev.sh

# Run all tests
./scripts/test.sh

# Clean build artifacts
./scripts/clean.sh
```

### VSCode Integration

The project includes VSCode configuration for Rust development:

- **Auto-formatting** on save
- **Inline errors** and warnings
- **Code completion** with rust-analyzer
- **Integrated terminal** tasks

Press `Ctrl+Shift+B` to see available tasks:
- `cargo: check` - Quick type check
- `cargo: test` - Run tests
- `cargo: clippy` - Lint checks
- `cargo: build-release` - Optimized build
- `cargo: doc` - Generate documentation

### Git Hooks

Pre-commit and pre-push hooks automatically:
- Check code formatting
- Run clippy lints
- Execute unit tests
- Detect potential secrets

---

## Testing

### Run All Tests

```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --test '*'

# Documentation tests
cargo test --workspace --doc

# All tests combined
./scripts/test.sh
```

### Run Specific Tests

```bash
# Run tests in a specific module
cargo test --package game_engine --lib ai::

# Run a specific test
cargo test --package game_engine --lib test_name

# Run tests with output
cargo test -- --nocapture
```

### Test Coverage

Generate coverage report:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Lcov
```

---

## Building

### Debug Build

Fast compilation, unoptimized:

```bash
cargo build --workspace
```

### Release Build

Slower compilation, optimized for performance:

```bash
cargo build --workspace --release
```

### Custom Features

Build with specific features:

```bash
# Build with WebAssembly support
cargo build --workspace --features wasm

# Build with all features
cargo build --workspace --all-features
```

---

## Examples

The project includes numerous examples demonstrating various features:

```bash
# List all examples
cargo example --workspace

# Run a specific example
cargo run --example basic_physics
cargo run --example behavior_tree
cargo run --example pathfinding

# Run all examples
for example in $(cargo example --workspace 2>&1 | grep -oP '\-\- \K[^ ]+'); do
    cargo run --example "$example"
done
```

### Example Categories

- **Basic**: Getting started examples
- **Physics**: Physics simulation demos
- **AI**: Behavior trees and pathfinding
- **Audio**: Sound and music playback
- **Rendering**: Graphics demos
- **ECS**: Entity Component System usage
- **Scripting**: WASM and Lua integration

---

## Common Tasks

### Add a New Component

```rust
use game_engine::ecs::prelude::*;

#[derive(Component, Debug)]
struct Health {
    current: f32,
    maximum: f32,
}

impl Health {
    fn new(max: f32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }
}
```

### Create a System

```rust
use game_engine::ecs::prelude::*;

fn health_regen_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        health.current = (health.current + 0.1).min(health.maximum);
    }
}

// Register the system
game.add_system(health_regen_system);
```

### Load Resources

```rust
use game_engine::resources::ResourceManager;

let manager = ResourceManager::new();

// Load a texture
let texture = manager.load_texture("assets/sprites/player.png")?;

// Load audio
let sound = manager.load_sound("assets/audio/jump.wav")?;
```

### Handle Input

```rust
use game_engine::input::{Input, KeyCode};

fn handle_input(input: &Input) {
    if input.is_key_pressed(KeyCode::W) {
        // Move forward
    }

    if input.is_key_down(KeyCode::Space) {
        // Jump
    }
}
```

---

## Troubleshooting

### Build Errors

**Problem**: Compilation fails with "linking with cc failed"

**Solution**: Install C compiler for your platform:
- macOS: `xcode-select --install`
- Linux: `sudo apt install build-essential`
- Windows: Install [MSVC Build Tools](https://visualstudio.microsoft.com/downloads/)

### Out of Memory

**Problem**: Build runs out of memory

**Solution**: Limit parallel jobs:
```bash
cargo build --jobs 2
```

### Slow Compilation

**Problem**: Initial build is slow

**Solution**: This is normal for Rust. Subsequent builds will be faster due to cargo's caching.

### Test Failures

**Problem**: Tests fail with "audio backend not connected"

**Solution**: Some audio tests may require a running audio server. Use:
```bash
cargo test --workspace --lib -- --skip audio
```

### IDE Issues

**Problem**: VSCode shows errors but `cargo check` works

**Solution**: Restart rust-analyzer:
1. Press `Ctrl+Shift+P`
2. Type "Restart rust-analyzer"

---

## Resources

### Documentation

- **API Documentation**: Run `cargo doc --open` to view locally
- **Online Docs**: [Link to deployed docs](https://<username>.github.io/game_engine/)
- **Examples**: Check the `examples/` directory

### Community

- **Issues**: Report bugs and request features on GitHub Issues
- **Discussions**: Ask questions and share ideas in GitHub Discussions

### Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Quick contribution steps:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and lints
5. Submit a pull request

### Development Tools

The project uses several development tools:

- **rust-analyzer**: Rust language server for IDE support
- **cargo-watch**: Auto-rebuild on file changes
- **cargo-audit**: Security vulnerability scanner
- **cargo-tarpaulin**: Code coverage tool
- **Git hooks**: Pre-commit and pre-push validation

---

## Next Steps

Now that you're set up, explore:

1. **Examples**: Learn by example in the `examples/` directory
2. **API Docs**: Dive deep into the API documentation
3. **Source Code**: Read the well-commented source code
4. **Tests**: Check out tests for usage patterns
5. **Benchmarks**: See performance characteristics

Happy game development! 🎮

---

## Getting Help

If you encounter any issues:

1. Check the [Troubleshooting](#troubleshooting) section
2. Search [existing issues](https://github.com/<username>/game_engine/issues)
3. Ask a question in [Discussions](https://github.com/<username>/game_engine/discussions)
4. Create a new issue with details

---

**Version**: 0.1.0
**Last Updated**: 2025-12-29
**Rust Edition**: 2024
