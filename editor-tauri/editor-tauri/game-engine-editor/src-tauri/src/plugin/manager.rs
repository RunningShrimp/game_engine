//! # Plugin Manager
//!
//! Manages plugin lifecycle, loading, unloading, and coordination.

use crate::plugin::{
    api::{Plugin, PluginContext, PluginEvent},
    events::EventBus,
    loader::PluginLoader,
    registry::PluginRegistry,
    PluginError, PluginState, Result,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex as AsyncMutex;

/// Plugin manager for coordinating all plugin operations
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    loader: Arc<PluginLoader>,
    event_bus: Arc<EventBus>,
    plugins: AsyncMutex<HashMap<String, LoadedPlugin>>,
    plugin_dirs: Vec<PathBuf>,
}

struct LoadedPlugin {
    plugin: Box<dyn Plugin + Send + Sync>,
    state: PluginState,
    context: PluginContext,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            loader: Arc::new(PluginLoader::new()),
            event_bus: Arc::new(EventBus::new()),
            plugins: AsyncMutex::new(HashMap::new()),
            plugin_dirs: Vec::new(),
        }
    }

    /// Add a plugin search directory
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.plugin_dirs.push(dir);
    }

    /// Discover plugins in registered directories
    pub async fn discover_plugins(&self) -> Result<Vec<String>> {
        let mut discovered = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            let entries = std::fs::read_dir(dir)
                .map_err(|e| PluginError::Io(e))?;

            for entry in entries {
                let entry = entry.map_err(|e| PluginError::Io(e))?;
                let path = entry.path();

                // Look for plugin manifests or dynamic libraries
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("toml") {
                        // Plugin manifest
                        if let Ok(name) = self.registry.register_from_manifest(&path) {
                            discovered.push(name);
                        }
                    }
                }
            }
        }

        Ok(discovered)
    }

    /// Load a plugin by name
    pub async fn load_plugin(&self, name: &str) -> Result<()> {
        // Check if already loaded
        let mut plugins = self.plugins.lock().await;
        if plugins.contains_key(name) {
            return Err(PluginError::LoadFailed(format!(
                "Plugin '{}' already loaded",
                name
            )));
        }

        // Get plugin descriptor from registry
        let descriptor = self
            .registry
            .get(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Check dependencies
        for dep in &descriptor.metadata.dependencies {
            if !plugins.contains_key(dep) {
                return Err(PluginError::DependencyNotFound(dep.clone()));
            }
        }

        // Load the plugin
        let plugin = self.loader.load(&descriptor.path)?;
        let context = PluginContext::new(
            crate::plugin::api::EngineApi::new(),
            crate::plugin::api::ResourceManager::new(),
            descriptor.config.clone(),
        );

        // Initialize plugin
        let mut loaded_plugin = LoadedPlugin {
            plugin,
            state: PluginState::Loading,
            context,
        };

        loaded_plugin.plugin.on_load(loaded_plugin.context.clone())?;
        loaded_plugin.state = PluginState::Loaded;

        // Store plugin
        plugins.insert(name.to_string(), loaded_plugin);

        // Publish event
        self.event_bus
            .publish(PluginEvent::PluginLoaded {
                name: name.to_string(),
            })
            .await;

        Ok(())
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.lock().await;

        let loaded_plugin = plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        loaded_plugin.state = PluginState::Unloading;

        // Unload plugin
        loaded_plugin.plugin.on_unload(loaded_plugin.context.clone())?;

        plugins.remove(name);

        // Publish event
        self.event_bus
            .publish(PluginEvent::PluginUnloaded {
                name: name.to_string(),
            })
            .await;

        Ok(())
    }

    /// Reload a plugin (hot reload)
    pub async fn reload_plugin(&self, name: &str) -> Result<()> {
        // Unload if loaded
        let is_loaded = self.plugins.lock().await.contains_key(name);
        if is_loaded {
            self.unload_plugin(name).await?;
        }

        // Reload
        self.load_plugin(name).await?;

        Ok(())
    }

    /// Update all loaded plugins
    pub async fn update(&self, delta_time: f32) {
        let plugins = self.plugins.lock().await;

        for (_, loaded_plugin) in plugins.iter() {
            if loaded_plugin.state == PluginState::Loaded {
                loaded_plugin
                    .plugin
                    .on_update(loaded_plugin.context.clone(), delta_time);
            }
        }
    }

    /// Get plugin by name
    pub async fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin + Send + Sync>> {
        let plugins = self.plugins.lock().await;
        plugins.get(name).map(|p| {
            // Use unsafe to convert reference to Arc (this is a simplified version)
            // In production, you'd need proper Arc handling
            // For now, we'll return None as this needs proper implementation
            None
        }).flatten()
    }

    /// Check if a plugin is loaded
    pub async fn is_loaded(&self, name: &str) -> bool {
        let plugins = self.plugins.lock().await;
        plugins.contains_key(name)
    }

    /// Get list of loaded plugins
    pub async fn loaded_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.lock().await;
        plugins.keys().cloned().collect()
    }

    /// Get plugin state
    pub async fn plugin_state(&self, name: &str) -> Option<PluginState> {
        let plugins = self.plugins.lock().await;
        plugins.get(name).map(|p| p.state)
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, name: &str) -> Result<()> {
        self.registry.enable_plugin(name)
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, name: &str) -> Result<()> {
        // Unload if currently loaded
        if self.is_loaded(name).await {
            self.unload_plugin(name).await?;
        }

        self.registry.disable_plugin(name)
    }

    /// Subscribe to plugin events
    pub fn subscribe_events(&self) -> crate::plugin::events::EventSubscriber {
        self.event_bus.subscribe()
    }

    /// Get registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        Arc::clone(&self.registry)
    }

    /// Get event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.event_bus)
    }

    /// Get statistics for all plugins
    pub async fn statistics(&self) -> HashMap<String, PluginStats> {
        // 使用基础统计收集（完整统计计划中）
        HashMap::new()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
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

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.loaded_plugins().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_plugin_dir() {
        let mut manager = PluginManager::new();
        let dir = PathBuf::from("/tmp/plugins");
        manager.add_plugin_dir(dir);
        assert_eq!(manager.plugin_dirs.len(), 1);
    }
}
