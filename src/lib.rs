//! # Game Engine
//!
//! A high-performance cross-platform 2D/3D game engine built with Rust.
//!
//! ## Features
//!
//! - **ECS Architecture**: Entity Component System for efficient game object management
//! - **Cross-Platform Rendering**: 2D/3D rendering with wgpu backend, including post-processing effects
//! - **Physics**: Integrated Rapier physics engine for 2D and 3D
//! - **Audio**: Audio system for sound effects and music
//! - **Animation**: Keyframe-based animation system
//! - **Editor**: Built-in editor tools for game development
//! - **Performance**: Profiling and optimization tools (SIMD, dirty tracking)
//!
//! ## Architecture Design
//!
//! This engine follows the **Rich Domain Model (富领域对象)** pattern:
//! - **Domain Objects**: Rich domain objects with encapsulated business logic
//! - **Aggregates**: Aggregates ensure business rules within boundaries
//! - **Domain Services**: True domain services with dependency injection
//! - **Error Handling**: Domain-specific errors with recovery strategies
//!
//! ### Example
//!
//! ```ignore
//! use game_engine::domain::{AudioSource, AudioSourceId, AudioDomainService};
//!
//! fn play_sound(service: &mut AudioDomainService) {
//!     let mut source = AudioSource::from_file(AudioSourceId(1), "sound.mp3").unwrap();
//!     source.play().unwrap();
//! }
//! ```
//!
//! ## Modules
//!
//! - [`core`]: Core engine functionality
//! - [`domain`]: Rich domain objects and services
//! - [`ecs`]: Entity Component System
//! - [`render`]: Rendering system with post-processing
//! - [`physics`]: Physics simulation
//! - [`audio`]: Audio playback
//! - [`animation`]: Animation system
//! - [`network`]: Network synchronization for multiplayer games
//! - [`xr`]: VR/AR support via OpenXR
//! - [`editor`]: Editor tools
//! - [`performance`]: Performance profiling and optimization
//! - [`plugins`]: Plugin system with hot-reload support
//!
//! ## Quick Start
//!
//! ```no_run
//! use game_engine::core::Engine;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     Engine::run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Examples
//!
//! See the [`examples`](../examples/index.html) directory for complete examples:
//! - `hello_world` - Basic engine usage
//! - `rendering` - Rendering examples
//! - `physics` - Physics simulation
//! - `animation` - Animation system
//! - `multiplayer` - Network synchronization
//! - `game` - Complete game example
//!
//! ## Documentation
//!
//! - [API Reference](https://docs.rs/game_engine)
//! - [User Guide](../docs/user_guide/index.html)
//! - [Best Practices](../docs/BEST_PRACTICES.md)

// 启用文档缺失警告（逐步添加文档）
#![warn(missing_docs)]

/// AI system for intelligent agents
pub mod ai;
/// Animation system with keyframes
pub mod animation;
/// Audio playback system
pub mod audio;
/// Language bindings for scripting
pub mod bindings;
/// Configuration system
pub mod config;
/// Core engine functionality including the main engine loop and initialization
pub mod core;
/// Domain layer with rich domain objects
pub mod domain;
/// Entity Component System for game object management
pub mod ecs;
/// Built-in editor tools
pub mod editor;
/// Network synchronization framework
pub mod network;
/// Performance profiling and optimization tools
pub mod performance;
/// Physics simulation using Rapier
pub mod physics;
/// Platform abstraction layer for cross-platform support
pub mod platform;
/// Rendering system with 2D/3D support
pub mod render;
/// Resource management for assets like textures and fonts
pub mod resources;
/// Scene management and serialization
pub mod scene;
/// Scripting system for game logic
pub mod scripting;
/// External services integration
pub mod services;
/// UI system for user interface management
pub mod ui;
/// XR (VR/AR/MR) support
pub mod xr;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    core::Engine::run();
}

// Hint: On WASM, complex UI can be rendered using DOM overlay for better accessibility and text handling.
// Use web-sys to manipulate DOM elements layered on top of the canvas.
