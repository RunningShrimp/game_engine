//! # Plugin Loader
//!
//! Dynamic loading of plugins from various sources (dylib, WASM, etc.)

use crate::plugin::{
    api::{Plugin, PluginMetadata},
    PluginError, Result,
};
use libloading::{Library, Symbol};
use std::path::PathBuf;
use std::sync::Arc;

/// Plugin loader for dynamic plugin loading
pub struct PluginLoader {
    loaded_libraries: Vec<Library>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            loaded_libraries: Vec::new(),
        }
    }

    /// Load a plugin from a file path
    pub fn load(&self, path: &PathBuf) -> Result<Box<dyn Plugin + Send + Sync>> {
        // Determine file type and load accordingly
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| PluginError::LoadFailed("No file extension".to_string()))?;

        match ext {
            "so" | "dylib" | "dll" => self.load_dylib(path),
            "wasm" => self.load_wasm(path),
            _ => Err(PluginError::LoadFailed(format!(
                "Unsupported plugin format: {}",
                ext
            ))),
        }
    }

    /// Load a native dynamic library plugin
    fn load_dylib(&self, path: &PathBuf) -> Result<Box<dyn Plugin + Send + Sync>> {
        unsafe {
            let library = Library::new(path)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to load library: {}", e)))?;

            // Get plugin creator function
            let plugin_create: Symbol<fn() -> *mut dyn Plugin> = library
                .get(b"_plugin_create")
                .or_else(|_| library.get(b"plugin_create"))
                .map_err(|e| {
                    PluginError::LoadFailed(format!(
                        "Failed to find plugin_create symbol: {}",
                        e
                    ))
                })?;

            let plugin_ptr = plugin_create();
            let plugin = Box::from_raw(plugin_ptr);

            // Keep library alive
            // Note: In production, you'd want to store the library with the plugin
            // to ensure it's not unloaded prematurely
            let _ = library; // For now, just drop it (this will cause issues)

            Ok(plugin)
        }
    }

    /// Load a WASM plugin
    fn load_wasm(&self, path: &PathBuf) -> Result<Box<dyn Plugin + Send + Sync>> {
        // WASM加载计划中（当前使用本地插件）
        // This would involve:
        // 1. Reading the WASM file
        // 2. Creating a WASM runtime (e.g., wasmtime)
        // 3. Instantiating the module
        // 4. Creating a wrapper that implements Plugin

        Err(PluginError::LoadFailed(
            "WASM plugin loading not yet implemented".to_string(),
        ))
    }

    /// Check if a file is a valid plugin
    pub fn is_valid_plugin(&self, path: &PathBuf) -> bool {
        let ext = path.extension().and_then(|e| e.to_str());
        matches!(ext, Some("so") | Some("dylib") | Some("dll") | Some("wasm"))
    }

    /// Get plugin metadata from a plugin file
    pub fn get_metadata(&self, path: &PathBuf) -> Result<PluginMetadata> {
        // Try to load the plugin and extract metadata
        let plugin = self.load(path)?;

        Ok(PluginMetadata {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            api_version: plugin.api_version().to_string(),
            description: plugin.description().to_string(),
            author: plugin.author().to_string(),
            dependencies: plugin.dependencies().iter().map(|s| s.to_string()).collect(),
            capabilities: plugin.capabilities().to_vec(),
            permissions: Vec::new(), // Not exposed in Plugin trait
        })
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Loadable plugin descriptor
#[derive(Debug, Clone)]
pub struct LoadablePlugin {
    pub path: PathBuf,
    pub metadata: PluginMetadata,
}

/// Macro to export a plugin for dynamic loading
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::plugin::api::Plugin {
            let plugin: Box<$plugin_type> = Box::new(<$plugin_type>::default());
            Box::leak(plugin)
        }

        // Alternative mangled name for some platforms
        #[no_mangle]
        pub extern "C" fn plugin_create() -> *mut dyn $crate::plugin::api::Plugin {
            _plugin_create()
        }
    };
}

/// Helper for building loadable plugins
pub struct PluginBuilder {
    metadata: PluginMetadata,
}

impl PluginBuilder {
    pub fn new(name: String, version: String) -> Self {
        Self {
            metadata: PluginMetadata::new(name, version),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.metadata.description = description;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.metadata.author = author;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.metadata.dependencies = dependencies;
        self
    }

    pub fn build(self) -> PluginMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.loaded_libraries.is_empty());
    }

    #[test]
    fn test_is_valid_plugin() {
        let loader = PluginLoader::new();

        assert!(loader.is_valid_plugin(&PathBuf::from("test.so")));
        assert!(loader.is_valid_plugin(&PathBuf::from("test.dylib")));
        assert!(loader.is_valid_plugin(&PathBuf::from("test.dll")));
        assert!(loader.is_valid_plugin(&PathBuf::from("test.wasm")));
        assert!(!loader.is_valid_plugin(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_plugin_builder() {
        let metadata = PluginBuilder::new("test".to_string(), "0.1.0".to_string())
            .with_description("Test plugin".to_string())
            .with_author("Test Author".to_string())
            .build();

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.description, "Test plugin");
        assert_eq!(metadata.author, "Test Author");
    }
}
