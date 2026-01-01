//! # Game Engine LSP Server
//!
//! Language Server Protocol implementation for the game engine.
//! Provides IDE support including:
//! - Code completion for engine components, systems, and resources
//! - Hover information for engine API
//! - Go to definition navigation
//! - Real-time diagnostics and error checking
//! - Integrated debugging support (DAP)
//!
//! ## Features
//!
//! - **Engine API Awareness**: Understands game engine's ECS, physics, rendering, and other systems
//! - **Intelligent Completion**: Context-aware suggestions for components, systems, queries
//! - **Type Information**: Rich hover information for all engine types
//! - **Error Detection**: Real-time validation of engine API usage
//! - **Debug Integration**: Built-in DAP server for script debugging
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
//! - Debug adapter capabilities

pub mod code_actions;
pub mod completion;
pub mod debug_adapter;
pub mod diagnostics;
pub mod documents;
pub mod formatting;
pub mod hover;
pub mod registry;
pub mod server;
pub mod symbols;

// Re-exports
pub use code_actions::CodeActionsProvider;
pub use debug_adapter::LspDapIntegrator;
pub use documents::{DocumentCache, SymbolIndex};
pub use formatting::CodeFormatter;
pub use registry::EngineAPIRegistry;
pub use server::GameEngineLSP;
pub use symbols::{DocumentSymbolsProvider, WorkspaceSymbolsProvider};
