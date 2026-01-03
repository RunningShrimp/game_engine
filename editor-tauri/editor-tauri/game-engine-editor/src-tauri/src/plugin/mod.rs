//! # Game Engine Editor Plugin System
//!
//! This module provides a comprehensive plugin system for extending the game engine editor.
//!
//! ## Architecture
//!
//! The plugin system consists of:
//! - **Plugin API**: Core trait and types for plugin development
//! - **Plugin Manager**: Lifecycle management and orchestration
//! - **Plugin Loader**: Dynamic loading of plugins (dylib, WASM)
//! - **Sandbox System**: Isolation and security
//! - **Event System**: Pub/sub communication between plugins and engine
//! - **Registry**: Plugin discovery and metadata management
//!
//! ## Features
//!
//! - Type-safe Rust API
//! - Hot reload support
//! - ABI stability
//! - Sandboxed execution
//! - Dependency injection
//! - Version management
//! - Multi-language support (Rust, WASM, TypeScript, Lua)
//!
//! ## Quick Start
//!
//! ```rust
//! use game_engine_editor::plugin::{Plugin, PluginContext, PluginManager};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str {
//!         "my_plugin"
//!     }
//!
//!     fn version(&self) -> &str {
//!         "0.1.0"
//!     }
//!
//!     fn on_load(&mut self, ctx: PluginContext) -> Result<(), Box<dyn std::error::Error>> {
//!         println!("Plugin loaded!");
//!         Ok(())
//!     }
//! }
//! ```

pub mod api;
pub mod loader;
pub mod manager;
pub mod sandbox;
pub mod events;
pub mod registry;

pub mod sdk;

// Re-exports for convenience
pub use api::{
    Plugin,
    PluginContext,
    PluginMetadata,
    PluginCapability,
    PluginPermission,
    PluginConfig,
    PluginEvent,
    EngineApi,
    ResourceManager,
};
pub use manager::PluginManager;
pub use loader::{PluginLoader, LoadablePlugin};
pub use sandbox::Sandbox;
pub use events::{EventBus, EventSubscriber};
pub use registry::{PluginRegistry, PluginDescriptor};

/// Plugin system version
pub const PLUGIN_SYSTEM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported plugin API version
pub const MIN_PLUGIN_API_VERSION: &str = "0.1.0";

/// Maximum supported plugin API version
pub const MAX_PLUGIN_API_VERSION: &str = "0.1.0";

/// Plugin system errors
pub type Result<T> = std::result::Result<T, PluginError>;

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin version incompatible: required {required}, found {found}")]
    IncompatibleVersion { required: String, found: String },

    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),

    #[error("Plugin permission denied: {0}")]
    PermissionDenied(String),

    #[error("Plugin sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Plugin ABI mismatch: {0}")]
    AbiMismatch(String),

    #[error("Plugin event error: {0}")]
    EventError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Plugin load state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

/// Plugin statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginStats {
    pub load_time_ms: u64,
    pub memory_usage_bytes: u64,
    pub event_count: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert!(!PLUGIN_SYSTEM_VERSION.is_empty());
        assert!(!MIN_PLUGIN_API_VERSION.is_empty());
        assert!(!MAX_PLUGIN_API_VERSION.is_empty());
    }

    #[test]
    fn test_error_display() {
        let err = PluginError::NotFound("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
