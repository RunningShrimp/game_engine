//! Plugin loader
//!
//! Handles dynamic loading of plugins from shared libraries.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::error::{Error, Result};
use super::api::{Plugin, PluginContext, PluginMetadata, PluginState, PluginStats};

/// Plugin library handle
#[derive(Clone)]
pub struct PluginLibrary {
    /// Path to the plugin library
    path: PathBuf,
    /// Library handle (platform-specific)
    #[cfg(unix)]
    _handle: Option<libloading::Library>,
    #[cfg(windows)]
    _handle: Option<libloading::Library>,
    /// Metadata
    metadata: PluginMetadata,
}

impl PluginLibrary {
    /// Create a new plugin library
    fn new(path: PathBuf, metadata: PluginMetadata) -> Self {
        Self {
            path,
            #[cfg(unix)]
            _handle: None,
            #[cfg(windows)]
            _handle: None,
            metadata,
        }
    }

    /// Get the library path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

/// Plugin loader configuration
#[derive(Clone, Debug)]
pub struct PluginLoaderConfig {
    /// Directories to search for plugins
    pub search_paths: Vec<PathBuf>,
    /// Whether to enable hot-reload
    pub hot_reload: bool,
    /// Whether to load plugins automatically
    pub auto_load: bool,
    /// Engine version for compatibility checking
    pub engine_version: String,
}

impl Default for PluginLoaderConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("plugins"),
                PathBuf::from("./plugins"),
            ],
            hot_reload: false,
            auto_load: true,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Plugin loader
pub struct PluginLoader {
    /// Configuration
    config: PluginLoaderConfig,
    /// Loaded plugins
    plugins: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn Plugin>>>>>>,
    /// Plugin libraries
    libraries: Arc<RwLock<HashMap<String, PluginLibrary>>>,
    /// Plugin states
    states: Arc<RwLock<HashMap<String, PluginState>>>,
    /// Plugin statistics
    stats: Arc<RwLock<HashMap<String, PluginStats>>>,
    /// Loaded dependencies
    loaded_dependencies: Arc<RwLock<HashSet<String>>>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(config: PluginLoaderConfig) -> Self {
        Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            libraries: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
            loaded_dependencies: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PluginLoaderConfig::default())
    }

    /// Discover plugins in the search paths
    pub fn discover_plugins(&self) -> Result<Vec<PathBuf>> {
        let mut plugin_paths = Vec::new();

        for search_path in &self.config.search_paths {
            if !search_path.exists() {
                continue;
            }

            self.scan_directory(search_path, &mut plugin_paths)?;
        }

        Ok(plugin_paths)
    }

    /// Scan a directory for plugin libraries
    fn scan_directory(&self, dir: &Path, plugin_paths: &mut Vec<PathBuf>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::IoError(format!("Failed to read directory {}: {}", dir.display(), e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::IoError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Check if it's a plugin library
            if path.is_file() && self.is_plugin_library(&path) {
                plugin_paths.push(path);
            } else if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path, plugin_paths)?;
            }
        }

        Ok(())
    }

    /// Check if a file is a plugin library
    fn is_plugin_library(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            let extension = path.extension().and_then(|e| e.to_str());
            extension == Some("so") || extension == Some("dylib")
        }

        #[cfg(windows)]
        {
            let extension = path.extension().and_then(|e| e.to_str());
            extension == Some("dll")
        }
    }

    /// Load a plugin from a file
    pub fn load_plugin(&self, path: &Path) -> Result<String> {
        // Load the library
        let library = unsafe {
            libloading::Library::new(path)
                .map_err(|e| Error::PluginLoadError(format!("Failed to load library {}: {}", path.display(), e)))?
        };

        // Get the plugin creation function
        let create_plugin: libloading::Symbol<fn() -> Box<dyn Plugin>> = unsafe {
            library.get(b"create_plugin")
                .map_err(|_| Error::PluginLoadError(
                    "Plugin does not export create_plugin function".to_string()
                ))?
        };

        // Create the plugin instance
        let mut plugin = create_plugin();
        let metadata = plugin.metadata().clone();

        // Check compatibility
        if !metadata.is_compatible_with(&self.config.engine_version) {
            return Err(Error::PluginIncompatibleError {
                plugin: metadata.name.clone(),
                required: metadata.engine_version.unwrap_or_default(),
                current: self.config.engine_version.clone(),
            });
        }

        // Check dependencies
        self.check_dependencies(&metadata)?;

        // Create plugin context
        let plugin_name = metadata.name.clone();
        let context = self.create_context(&plugin_name);

        // Initialize the plugin
        plugin.on_load(&context)?;

        // Store the plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name.clone(), Arc::new(RwLock::new(plugin)));
        }

        // Store the library
        {
            let mut libraries = self.libraries.write().unwrap();
            libraries.insert(plugin_name.clone(), PluginLibrary::new(path.to_path_buf(), metadata.clone()));
        }

        // Set state to running
        {
            let mut states = self.states.write().unwrap();
            states.insert(plugin_name.clone(), PluginState::Running);
        }

        // Initialize statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.insert(plugin_name.clone(), PluginStats::new(plugin_name.clone()));
        }

        // Mark dependencies as loaded
        {
            let mut loaded_deps = self.loaded_dependencies.write().unwrap();
            for dep in &metadata.dependencies {
                loaded_deps.insert(dep.clone());
            }
        }

        Ok(plugin_name)
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        // Get the plugin
        let plugin = {
            let plugins = self.plugins.read().unwrap();
            plugins.get(name)
                .cloned()
                .ok_or_else(|| Error::PluginNotFound(name.to_string()))?
        };

        // Create context
        let context = self.create_context(name);

        // Call unload
        {
            let mut plugin_guard = plugin.write().unwrap();
            plugin_guard.on_unload(&context)?;
        }

        // Remove plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(name);
        }

        // Remove library
        {
            let mut libraries = self.libraries.write().unwrap();
            libraries.remove(name);
        }

        // Set state to unloaded
        {
            let mut states = self.states.write().unwrap();
            states.insert(name.to_string(), PluginState::Unloaded);
        }

        // Remove statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.remove(name);
        }

        Ok(())
    }

    /// Reload a plugin (for hot-reload)
    pub fn reload_plugin(&self, name: &str) -> Result<()> {
        if !self.config.hot_reload {
            return Err(Error::PluginError("Hot-reload is not enabled".to_string()));
        }

        // Get the library path
        let path = {
            let libraries = self.libraries.read().unwrap();
            let library = libraries.get(name)
                .ok_or_else(|| Error::PluginNotFound(name.to_string()))?;
            library.path().to_path_buf()
        };

        // Unload the plugin
        self.unload_plugin(name)?;

        // Reload the plugin
        self.load_plugin(&path)?;

        Ok(())
    }

    /// Check if plugin dependencies are satisfied
    fn check_dependencies(&self, metadata: &PluginMetadata) -> Result<()> {
        let loaded_deps = self.loaded_dependencies.read().unwrap();

        for dep in &metadata.dependencies {
            if !loaded_deps.contains(dep) {
                return Err(Error::PluginDependencyError {
                    plugin: metadata.name.clone(),
                    missing: dep.clone(),
                });
            }
        }

        Ok(())
    }

    /// Create a plugin context
    fn create_context(&self, name: &str) -> PluginContext {
        let data_dir = PathBuf::from("plugins").join(name).join("data");
        let config_dir = PathBuf::from("plugins").join(name).join("config");

        PluginContext::new(data_dir, config_dir, self.config.hot_reload)
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    /// Get all plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    /// Get plugin state
    pub fn plugin_state(&self, name: &str) -> Option<PluginState> {
        let states = self.states.read().unwrap();
        states.get(name).copied()
    }

    /// Get plugin statistics
    pub fn plugin_stats(&self, name: &str) -> Option<PluginStats> {
        let stats = self.stats.read().unwrap();
        stats.get(name).cloned()
    }

    /// Update all plugins
    pub fn update_plugins(&self, delta: f32) {
        let plugins = self.plugins.read().unwrap();
        let mut stats = self.stats.write().unwrap();
        let mut states = self.states.write().unwrap();

        for (name, plugin) in plugins.iter() {
            let start_time = std::time::Instant::now();

            {
                let mut plugin_guard = plugin.write().unwrap();
                let context = self.create_context(name);
                plugin_guard.on_update(&context, delta);
            }

            let elapsed = start_time.elapsed().as_secs_f32();

            // Update statistics
            if let Some(stat) = stats.get_mut(name) {
                stat.record_update(elapsed);
            }

            // Update state
            if let Some(state) = states.get_mut(name) {
                *state = PluginState::Running;
            }
        }
    }

    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        let plugins = self.plugins.read().unwrap();
        plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_plugin_loader_config() {
        let config = PluginLoaderConfig::default();
        assert_eq!(config.search_paths.len(), 2);
        assert!(!config.hot_reload);
        assert!(config.auto_load);
    }

    #[test]
    fn test_discover_plugins() {
        let loader = PluginLoader::with_default_config();

        // Create temporary plugin directory
        let temp_dir = std::env::temp_dir().join("plugin_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let result = loader.discover_plugins();
        assert!(result.is_ok());

        // Cleanup
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
