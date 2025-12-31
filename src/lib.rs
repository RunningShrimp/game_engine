//! Game Engine - A high-performance game engine with plugin system and resource marketplace
//!
//! This library provides:
//! - Plugin system for extensibility
//! - Resource marketplace integration
//! - Unity project migration tools
//!
//! ## Example
//!
//! ```rust
//! use game_engine::plugins::PluginManager;
//!
//! let manager = PluginManager::with_default_config();
//! manager.initialize()?;
//! ```

pub mod error;
pub mod plugins;
pub mod tools;

// Re-export commonly used types
pub use error::{Error, Result};
pub use plugins::{Plugin, PluginContext, PluginManager, PluginMetadata};

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine name
pub const ENGINE_NAME: &str = "Game Engine";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_engine_name() {
        assert_eq!(ENGINE_NAME, "Game Engine");
    }
}
