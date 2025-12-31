//! Plugin system
//!
//! Provides a comprehensive plugin architecture for extending the game engine.

pub mod api;
pub mod loader;
pub mod registry;

pub use api::{
    Plugin, PluginContext, PluginEvent, PluginMetadata, PluginState, PluginStats,
};
pub use loader::{PluginLibrary, PluginLoader, PluginLoaderConfig};
pub use registry::{PluginEntry, PluginMessage, PluginRegistry};

use std::sync::{Arc, RwLock};
use crate::ecs::{ComponentRegistry, SystemScheduler};
use crate::error::{Error, Result};

/// Plugin manager - combines loader and registry
pub struct PluginManager {
    /// Plugin loader
    loader: PluginLoader,
    /// Plugin registry
    registry: PluginRegistry,
    /// Component registry for plugin component registration
    component_registry: Option<Arc<RwLock<ComponentRegistry>>>,
    /// System scheduler for plugin system registration
    system_scheduler: Option<Arc<RwLock<SystemScheduler>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(loader: PluginLoader, registry: PluginRegistry) -> Self {
        Self {
            loader,
            registry,
            component_registry: None,
            system_scheduler: None,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(
            PluginLoader::with_default_config(),
            PluginRegistry::new(),
        )
    }

    /// Set the component registry
    pub fn with_component_registry(mut self, registry: Arc<RwLock<ComponentRegistry>>) -> Self {
        self.component_registry = Some(registry);
        self
    }

    /// Set the system scheduler
    pub fn with_system_scheduler(mut self, scheduler: Arc<RwLock<SystemScheduler>>) -> Self {
        self.system_scheduler = Some(scheduler);
        self
    }

    /// Initialize the plugin manager
    pub fn initialize(&mut self) -> Result<()> {
        // Discover plugins
        let plugin_paths = self.loader.discover_plugins()?;

        // Load discovered plugins
        for path in plugin_paths {
            if let Err(e) = self.load_plugin_from_path(&path) {
                eprintln!("Failed to load plugin {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Load a plugin from a file path
    pub fn load_plugin_from_path(&self, path: &std::path::Path) -> Result<String> {
        // Load using loader
        let name = self.loader.load_plugin(path)?;

        // Get the plugin
        let plugin = self.loader.get_plugin(&name)
            .ok_or_else(|| Error::PluginNotFound(name.clone()))?;

        // Register components and systems
        {
            let plugin_guard = plugin.read().unwrap();

            if let Some(ref registry) = self.component_registry {
                let mut component_registry = registry.write().unwrap();
                plugin_guard.register_components(&mut component_registry);
            }

            if let Some(ref scheduler) = self.system_scheduler {
                let mut system_scheduler = scheduler.write().unwrap();
                plugin_guard.register_systems(&mut system_scheduler);
            }
        }

        // Register in registry
        let metadata = {
            let libraries = self.loader.libraries.read().unwrap();
            let library = libraries.get(&name)
                .ok_or_else(|| Error::PluginNotFound(name.clone()))?;
            library.metadata().clone()
        };

        self.registry.register_plugin(metadata)?;

        Ok(name)
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        // Check if can unload
        if !self.registry.can_unload(name) {
            return Err(Error::PluginError(format!(
                "Cannot unload plugin {}: other plugins depend on it",
                name
            )));
        }

        // Unregister from registry
        self.registry.unregister_plugin(name)?;

        // Unload from loader
        self.loader.unload_plugin(name)?;

        Ok(())
    }

    /// Reload a plugin
    pub fn reload_plugin(&self, name: &str) -> Result<()> {
        self.loader.reload_plugin(name)
    }

    /// Get a plugin
    pub fn get_plugin(&self, name: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        self.loader.get_plugin(name)
    }

    /// Get all plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        self.loader.plugin_names()
    }

    /// Update all plugins
    pub fn update(&self, delta: f32) {
        self.loader.update_plugins(delta);
        self.registry.process_messages();
    }

    /// Get the loader
    pub fn loader(&self) -> &PluginLoader {
        &self.loader
    }

    /// Get the registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.loader.plugin_count()
    }

    /// Shutdown the plugin manager
    pub fn shutdown(&mut self) -> Result<()> {
        // Unload all plugins in reverse load order
        let plugins = self.registry.get_plugins_by_load_order();
        for plugin in plugins.into_iter().rev() {
            if let Err(e) = self.unload_plugin(&plugin.metadata.name) {
                eprintln!("Failed to unload plugin {}: {}", plugin.metadata.name, e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::with_default_config();
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_manager_with_registries() {
        let component_registry = Arc::new(RwLock::new(ComponentRegistry::new()));
        let system_scheduler = Arc::new(RwLock::new(SystemScheduler::new()));

        let manager = PluginManager::with_default_config()
            .with_component_registry(component_registry)
            .with_system_scheduler(system_scheduler);

        assert_eq!(manager.plugin_count(), 0);
    }
}
