//! Plugin API definition
//!
//! Defines the core plugin trait and related types for the game engine plugin system.

use std::any::Any;
use crate::ecs::{ComponentRegistry, SystemScheduler};
use crate::error::Error;

/// Context provided to plugins during load/unload/update operations
#[derive(Clone)]
pub struct PluginContext {
    /// Engine API version
    pub engine_version: &'static str,
    /// Plugin data directory
    pub data_dir: std::path::PathBuf,
    /// Plugin configuration directory
    pub config_dir: std::path::PathBuf,
    /// Whether the plugin is in hot-reload mode
    pub hot_reload: bool,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(
        data_dir: std::path::PathBuf,
        config_dir: std::path::PathBuf,
        hot_reload: bool,
    ) -> Self {
        Self {
            engine_version: env!("CARGO_PKG_VERSION"),
            data_dir,
            config_dir,
            hot_reload,
        }
    }

    /// Get the engine version
    pub fn engine_version(&self) -> &str {
        self.engine_version
    }

    /// Get the plugin data directory
    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    /// Get the plugin config directory
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    /// Check if hot-reload is enabled
    pub fn is_hot_reload(&self) -> bool {
        self.hot_reload
    }
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: Option<String>,
    /// Plugin description
    pub description: Option<String>,
    /// Required engine version
    pub engine_version: Option<String>,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Website URL
    pub website: Option<String>,
    /// License
    pub license: Option<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            author: None,
            description: None,
            engine_version: None,
            dependencies: Vec::new(),
            website: None,
            license: None,
        }
    }

    /// Set author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set engine version requirement
    pub fn with_engine_version(mut self, version: String) -> Self {
        self.engine_version = Some(version);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Set website
    pub fn with_website(mut self, website: String) -> Self {
        self.website = Some(website);
        self
    }

    /// Set license
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// Check if this plugin is compatible with the given engine version
    pub fn is_compatible_with(&self, engine_version: &str) -> bool {
        if let Some(ref required) = self.engine_version {
            // Simple version check (can be enhanced with semver)
            engine_version == required
        } else {
            true
        }
    }
}

/// Core plugin trait that all plugins must implement
pub trait Plugin: Any + Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Called when the plugin is loaded
    ///
    /// This is where plugins should register their components, systems, and resources.
    fn on_load(&mut self, context: &PluginContext) -> Result<(), Error> {
        let _ = context;
        Ok(())
    }

    /// Called when the plugin is unloaded
    ///
    /// Plugins should clean up any resources they allocated here.
    fn on_unload(&mut self, context: &PluginContext) -> Result<(), Error> {
        let _ = context;
        Ok(())
    }

    /// Called every frame with the delta time
    fn on_update(&mut self, context: &PluginContext, delta: f32) {
        let _ = context;
        let _ = delta;
    }

    /// Called when the engine is in fixed update mode
    fn on_fixed_update(&mut self, context: &PluginContext, delta: f32) {
        let _ = context;
        let _ = delta;
    }

    /// Register custom components with the ECS
    fn register_components(&self, _registry: &mut ComponentRegistry) {
        // Default: no components
    }

    /// Register custom systems with the scheduler
    fn register_systems(&self, _scheduler: &mut SystemScheduler) {
        // Default: no systems
    }

    /// Handle events from the engine
    fn on_event(&mut self, context: &PluginContext, event: &PluginEvent) {
        let _ = context;
        let _ = event;
    }

    /// Get a reference to the plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable reference to the plugin as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Events that can be sent to plugins
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Engine is initializing
    EngineInit,
    /// Engine is shutting down
    EngineShutdown,
    /// Scene is being loaded
    SceneLoading(String),
    /// Scene has been loaded
    SceneLoaded(String),
    /// Scene is being unloaded
    SceneUnloading(String),
    /// Scene has been unloaded
    SceneUnloaded(String),
    /// Custom event with data
    Custom(String, String),
    /// Error occurred
    Error(String),
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loaded but not initialized
    Loaded,
    /// Plugin is initialized and running
    Running,
    /// Plugin is paused
    Paused,
    /// Plugin has encountered an error
    Error,
    /// Plugin is unloaded
    Unloaded,
}

/// Plugin statistics for monitoring
#[derive(Debug, Clone)]
pub struct PluginStats {
    /// Plugin name
    pub name: String,
    /// Current state
    pub state: PluginState,
    /// Time since last update (seconds)
    pub last_update_time: f32,
    /// Total update time (seconds)
    pub total_update_time: f32,
    /// Number of updates performed
    pub update_count: usize,
    /// Average update time (seconds)
    pub avg_update_time: f32,
}

impl PluginStats {
    /// Create new plugin stats
    pub fn new(name: String) -> Self {
        Self {
            name,
            state: PluginState::Loaded,
            last_update_time: 0.0,
            total_update_time: 0.0,
            update_count: 0,
            avg_update_time: 0.0,
        }
    }

    /// Update statistics after an update
    pub fn record_update(&mut self, delta_time: f32) {
        self.last_update_time = delta_time;
        self.total_update_time += delta_time;
        self.update_count += 1;
        self.avg_update_time = if self.update_count > 0 {
            self.total_update_time / self.update_count as f32
        } else {
            0.0
        };
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        self.last_update_time = 0.0;
        self.total_update_time = 0.0;
        self.update_count = 0;
        self.avg_update_time = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata::new("test-plugin".to_string(), "1.0.0".to_string())
            .with_author("Test Author".to_string())
            .with_description("A test plugin".to_string())
            .with_engine_version("0.1.0".to_string());

        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.is_compatible_with("0.1.0"));
        assert!(!metadata.is_compatible_with("0.2.0"));
    }

    #[test]
    fn test_plugin_context() {
        let context = PluginContext::new(
            std::path::PathBuf::from("/data"),
            std::path::PathBuf::from("/config"),
            true,
        );

        assert!(context.is_hot_reload());
        assert_eq!(context.data_dir(), std::path::Path::new("/data"));
        assert_eq!(context.config_dir(), std::path::Path::new("/config"));
    }

    #[test]
    fn test_plugin_stats() {
        let mut stats = PluginStats::new("test-plugin".to_string());
        stats.record_update(0.016);
        stats.record_update(0.017);
        stats.record_update(0.015);

        assert_eq!(stats.update_count, 3);
        assert_eq!(stats.total_update_time, 0.048);
        assert!((stats.avg_update_time - 0.016).abs() < 0.001);
    }
}
