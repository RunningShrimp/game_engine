//! # Game Engine LSP Server
//!
//! Language Server Protocol implementation for the game engine.
//! Provides IDE support including:
//! - Code completion for engine components, systems, and resources
//! - Hover information for engine API
//! - Go to definition navigation
//! - Real-time diagnostics and error checking
//!
//! ## Features
//!
//! - **Engine API Awareness**: Understands game engine's ECS, physics, rendering, and other systems
//! - **Intelligent Completion**: Context-aware suggestions for components, systems, queries
//! - **Type Information**: Rich hover information for all engine types
//! - **Error Detection**: Real-time validation of engine API usage
//!
//! ## Usage
//!
//! ```bash
//! # Start the LSP server
//! cargo run --bin game-engine-lsp
//! ```
//!
//! ## Configuration
//!
//! The LSP server can be configured via client capabilities:
//! - Text document synchronization
//! - Completion item capabilities
//! - Hover capabilities
//! - Definition capabilities

pub mod server;
pub mod completion;
pub mod diagnostics;
pub mod hover;
pub mod registry;

// Re-exports
pub use server::GameEngineLSP;
pub use registry::EngineAPIRegistry;
