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
//! This engine follows the **Anemic Domain Model (贫血模型)** pattern:
//! - **State (Resource)**: Pure data structures storing system state
//! - **Service**: Business logic encapsulation with static methods
//! - **System**: ECS systems for orchestration and scheduling
//!
//! ### Example
//!
//! ```ignore
//! use game_engine::audio::{AudioState, AudioService};
//!
//! fn play_sound(mut state: ResMut<AudioState>, entity: Entity) {
//!     AudioService::play_file(&state, entity, "sound.mp3", 1.0, false);
//! }
//! ```
//!
//! ## Modules
//!
//! - [`core`]: Core engine functionality
//! - [`ecs`]: Entity Component System
//! - [`render`]: Rendering system with post-processing
//! - [`physics`]: Physics simulation
//! - [`audio`]: Audio playback
//! - [`animation`]: Animation system
//! - [`editor`]: Editor tools
//! - [`performance`]: Performance profiling

// 启用文档缺失警告（逐步添加文档）
// #![warn(missing_docs)]  // TODO: 启用后逐步修复所有警告

/// Core engine functionality including the main engine loop and initialization
pub mod core;
/// Platform abstraction layer for cross-platform support
pub mod platform;
/// Rendering system with 2D/3D support
pub mod render;
/// Entity Component System for game object management
pub mod ecs;
/// Resource management for assets like textures and fonts
pub mod resources;
/// Physics simulation using Rapier
pub mod physics;
/// Built-in editor tools
pub mod editor;
/// Scripting system for game logic
pub mod scripting;
/// External services integration
pub mod services;
/// Audio playback system
pub mod audio;
/// Language bindings for scripting
pub mod bindings;
/// Migration utilities for transitioning from old APIs to new APIs
pub mod migration;
/// XR (VR/AR/MR) support
pub mod xr;
/// Performance profiling and optimization tools
pub mod performance;
/// Configuration system
pub mod config;
/// Animation system with keyframes
pub mod animation;
/// Scene management and serialization
pub mod scene;
/// Network synchronization framework
pub mod network;
/// AI system for intelligent agents
pub mod ai;
/// UI system for user interface management
pub mod ui;

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

