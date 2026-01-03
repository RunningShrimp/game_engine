//! # Plugin Registry
//!
//! Plugin discovery, metadata management, and registration.

use crate::plugin::{
    api::{PluginConfig, PluginMetadata},
    Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Plugin registry for managing plugin metadata and configurations
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginDescriptor>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin descriptor
    pub fn register(&self, descriptor: PluginDescriptor) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(descriptor.metadata.name.clone(), descriptor);
        Ok(())
    }

    /// Register a plugin from a manifest file
    pub fn register_from_manifest(&self, path: &PathBuf) -> Result<String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::plugin::PluginError::Io(e))?;

        let manifest: PluginManifest = toml::from_str(&content)
            .map_err(|e| crate::plugin::PluginError::Serialization(e))?;

        let descriptor = PluginDescriptor {
            path: path.clone(),
            metadata: manifest.metadata,
            config: manifest.config,
        };

        let name = descriptor.metadata.name.clone();
        self.register(descriptor)?;
        Ok(name)
    }

    /// Unregister a plugin
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        plugins
            .remove(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;
        Ok(())
    }

    /// Get a plugin descriptor by name
    pub fn get(&self, name: &str) -> Option<PluginDescriptor> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    /// Check if a plugin is registered
    pub fn contains(&self, name: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(name)
    }

    /// Get all registered plugin names
    pub fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    /// Get all plugin descriptors
    pub fn all(&self) -> Vec<PluginDescriptor> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Enable a plugin
    pub fn enable_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        let plugin = plugins
            .get_mut(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;
        plugin.config.enabled = true;
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        let plugin = plugins
            .get_mut(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;
        plugin.config.enabled = false;
        Ok(())
    }

    /// Update plugin configuration
    pub fn update_config(&self, name: &str, config: PluginConfig) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();
        let plugin = plugins
            .get_mut(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;
        plugin.config = config;
        Ok(())
    }

    /// Save registry to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        let descriptors: Vec<&PluginDescriptor> = plugins.values().collect();

        let manifest_data = serde_json::to_string_pretty(descriptors)
            .map_err(|e| crate::plugin::PluginError::Serialization(e))?;

        std::fs::write(path, manifest_data)
            .map_err(|e| crate::plugin::PluginError::Io(e))?;

        Ok(())
    }

    /// Load registry from file
    pub fn load(&self, path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::plugin::PluginError::Io(e))?;

        let descriptors: Vec<PluginDescriptor> = serde_json::from_str(&content)
            .map_err(|e| crate::plugin::PluginError::Serialization(e))?;

        let mut plugins = self.plugins.write().unwrap();
        for descriptor in descriptors {
            plugins.insert(descriptor.metadata.name.clone(), descriptor);
        }

        Ok(())
    }

    /// Find plugins by capability
    pub fn find_by_capability(&self, capability: &str) -> Vec<PluginDescriptor> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|p| {
                p.metadata
                    .capabilities
                    .iter()
                    .any(|c| format!("{:?}", c) == capability || matches!(c, crate::plugin::api::PluginCapability::Custom(s) if s == capability))
            })
            .cloned()
            .collect()
    }

    /// Resolve plugin dependencies
    pub fn resolve_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let plugin = self
            .get(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;

        let mut resolved = Vec::new();
        let mut to_resolve = plugin.metadata.dependencies.clone();

        while let Some(dep_name) = to_resolve.pop() {
            if resolved.contains(&dep_name) {
                continue;
            }

            let dep_plugin = self
                .get(&dep_name)
                .ok_or_else(|| crate::plugin::PluginError::DependencyNotFound(dep_name.clone()))?;

            resolved.push(dep_name.clone());

            // Add dependencies of this dependency
            for dep_dep in dep_plugin.metadata.dependencies {
                if !resolved.contains(&dep_dep) && !to_resolve.contains(&dep_dep) {
                    to_resolve.push(dep_dep);
                }
            }
        }

        Ok(resolved)
    }

    /// Validate plugin version compatibility
    pub fn validate_version(&self, name: &str) -> Result<bool> {
        let plugin = self
            .get(name)
            .ok_or_else(|| crate::plugin::PluginError::NotFound(name.to_string()))?;

        // Check if plugin API version is supported
        let api_version = plugin.metadata.api_version;
        let min_version = crate::plugin::MIN_PLUGIN_API_VERSION;
        let max_version = crate::plugin::MAX_PLUGIN_API_VERSION;

        // Simple version comparison (semver would be better)
        Ok(api_version >= min_version && api_version <= max_version)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin descriptor containing metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub path: PathBuf,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
}

/// Plugin manifest file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
}

/// Plugin search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub enabled_only: bool,
    pub with_capability: Option<String>,
    pub min_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::api::PluginCapability;

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let registry = PluginRegistry::new();

        let metadata = PluginMetadata::new("test".to_string(), "0.1.0".to_string());
        let descriptor = PluginDescriptor {
            path: PathBuf::from("/test/path.so"),
            metadata,
            config: PluginConfig::default(),
        };

        registry.register(descriptor).unwrap();
        assert!(registry.contains("test"));
    }

    #[test]
    fn test_get_plugin() {
        let registry = PluginRegistry::new();

        let metadata = PluginMetadata::new("test".to_string(), "0.1.0".to_string());
        let descriptor = PluginDescriptor {
            path: PathBuf::from("/test/path.so"),
            metadata,
            config: PluginConfig::default(),
        };

        registry.register(descriptor).unwrap();
        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "test");
    }

    #[test]
    fn test_enable_disable_plugin() {
        let registry = PluginRegistry::new();

        let metadata = PluginMetadata::new("test".to_string(), "0.1.0".to_string());
        let descriptor = PluginDescriptor {
            path: PathBuf::from("/test/path.so"),
            metadata,
            config: PluginConfig::default(),
        };

        registry.register(descriptor).unwrap();

        registry.disable_plugin("test").unwrap();
        let plugin = registry.get("test").unwrap();
        assert!(!plugin.config.enabled);

        registry.enable_plugin("test").unwrap();
        let plugin = registry.get("test").unwrap();
        assert!(plugin.config.enabled);
    }

    #[test]
    fn test_find_by_capability() {
        let registry = PluginRegistry::new();

        let mut metadata = PluginMetadata::new("test".to_string(), "0.1.0".to_string());
        metadata.capabilities = vec![PluginCapability::Render];

        let descriptor = PluginDescriptor {
            path: PathBuf::from("/test/path.so"),
            metadata,
            config: PluginConfig::default(),
        };

        registry.register(descriptor).unwrap();

        let results = registry.find_by_capability("Render");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.name, "test");
    }
}
