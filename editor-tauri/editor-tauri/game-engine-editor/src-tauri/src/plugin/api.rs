//! # Plugin API
//!
//! Core traits and types for plugin development.

use crate::plugin::{PluginError, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// Core plugin trait that all plugins must implement
pub trait Plugin: Any {
    /// Plugin name (unique identifier)
    fn name(&self) -> &str;

    /// Plugin version (semantic versioning)
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Plugin API version required
    fn api_version(&self) -> &str {
        "0.1.0"
    }

    /// Plugin dependencies (list of plugin names)
    fn dependencies(&self) -> &[&str] {
        &[]
    }

    /// Plugin description
    fn description(&self) -> &str {
        ""
    }

    /// Plugin author
    fn author(&self) -> &str {
        ""
    }

    /// Plugin capabilities
    fn capabilities(&self) -> &[PluginCapability] {
        &[]
    }

    /// Called when plugin is loaded
    fn on_load(&mut self, context: PluginContext) -> Result<()>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self, context: PluginContext) -> Result<()> {
        let _ = context;
        Ok(())
    }

    /// Called each frame
    fn on_update(&mut self, context: PluginContext, delta_time: f32) {
        let _ = context;
        let _ = delta_time;
    }

    /// Called when an event occurs
    fn on_event(&mut self, event: &PluginEvent) {
        let _ = event;
    }

    /// Get plugin-specific configuration
    fn config(&self) -> Option<PluginConfig> {
        None
    }

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Context provided to plugins during lifecycle callbacks
pub struct PluginContext {
    /// Engine API for accessing engine services
    pub engine_api: EngineApi,

    /// Resource manager for accessing engine resources
    pub resource_manager: ResourceManager,

    /// Plugin configuration
    pub config: PluginConfig,

    /// Plugin-specific data storage
    pub data: HashMap<String, String>,
}

impl PluginContext {
    pub fn new(engine_api: EngineApi, resource_manager: ResourceManager, config: PluginConfig) -> Self {
        Self {
            engine_api,
            resource_manager,
            config,
            data: HashMap::new(),
        }
    }

    /// Get plugin-specific data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Set plugin-specific data
    pub fn set_data(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<PluginPermission>,
}

impl PluginMetadata {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            api_version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<PluginCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<PluginPermission>) -> Self {
        self.permissions = permissions;
        self
    }
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Can render graphics
    Render,
    /// Can play audio
    Audio,
    /// Can access physics
    Physics,
    /// Can access network
    Network,
    /// Can access filesystem
    FileSystem,
    /// Can access UI
    UserInterface,
    /// Can modify scenes
    SceneModification,
    /// Can access asset pipeline
    AssetPipeline,
    /// Custom capability
    Custom(String),
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// Read-only access
    Read,
    /// Write access
    Write,
    /// Network access
    Network,
    /// Filesystem access
    Filesystem,
    /// Custom permission
    Custom(String),
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub settings: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub auto_load: bool,
    pub hot_reload: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
            enabled: true,
            auto_load: true,
            hot_reload: false,
        }
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.settings.insert(key, value);
        self
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.settings.get(key) {
            Some(value) => {
                let parsed = serde_json::from_value(value.clone())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    pub fn set<T>(&mut self, key: String, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_value(value)?;
        self.settings.insert(key, serialized);
        Ok(())
    }
}

/// Plugin events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// Plugin loaded
    PluginLoaded { name: String },
    /// Plugin unloaded
    PluginUnloaded { name: String },
    /// Plugin error
    PluginError { name: String, error: String },
    /// Scene loaded
    SceneLoaded { path: String },
    /// Scene saved
    SceneSaved { path: String },
    /// Asset imported
    AssetImported { path: String },
    /// Engine tick
    Tick { delta_time: f32 },
    /// Custom event
    Custom { type_: String, data: serde_json::Value },
}

/// Engine API exposed to plugins
pub struct EngineApi {
    // This would contain actual engine API methods
    // For now, it's a placeholder
    _private: (),
}

impl EngineApi {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Get engine version
    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    // 引擎API方法计划中（当前使用基础实现）
    // - Get scene
    // - Register component
    // - Access renderer
    // - Access physics
    // - etc.
}

impl Default for EngineApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource manager for plugins
pub struct ResourceManager {
    // This would contain actual resource management methods
    _private: (),
}

impl ResourceManager {
    pub fn new() -> Self {
        Self { _private: () }
    }

    // 资源管理方法计划中（当前使用基础实现）
    // - Load asset
    // - Save asset
    // - Get asset info
    // - etc.
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for creating plugins
#[macro_export]
macro_rules! plugin {
    ($struct_name:ident, $name:expr, $version:expr) => {
        impl $crate::plugin::Plugin for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn version(&self) -> &str {
                $version
            }
        }
    };
}

/// Helper macro for declaring plugin metadata
#[macro_export]
macro_rules! plugin_metadata {
    ($name:expr, $version:expr) => {
        $crate::plugin::PluginMetadata::new($name.to_string(), $version.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test_plugin"
        }

        fn version(&self) -> &str {
            "0.1.0"
        }

        fn on_load(&mut self, _context: PluginContext) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_basic() {
        let plugin = TestPlugin;
        assert_eq!(plugin.name(), "test_plugin");
        assert_eq!(plugin.version(), "0.1.0");
    }

    #[test]
    fn test_metadata() {
        let meta = PluginMetadata::new("test".to_string(), "0.1.0".to_string())
            .with_description("Test plugin".to_string())
            .with_author("Test Author".to_string());

        assert_eq!(meta.name, "test");
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.description, "Test plugin");
        assert_eq!(meta.author, "Test Author");
    }

    #[test]
    fn test_config() {
        let mut config = PluginConfig::new();
        config.set("test_key".to_string(), 42).unwrap();
        let value: Option<i32> = config.get("test_key").unwrap();
        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_event_serialization() {
        let event = PluginEvent::Tick { delta_time: 0.016 };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: PluginEvent = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            PluginEvent::Tick { delta_time } => {
                assert_eq!(delta_time, 0.016);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
