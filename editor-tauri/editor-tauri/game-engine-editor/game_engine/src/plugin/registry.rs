//! Plugin registry for managing installed plugins

use super::models::*;
use super::{PluginError, PluginInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin registry
pub struct PluginRegistry {
    registry_path: PathBuf,
    data: RegistryData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryData {
    plugins: HashMap<String, InstalledPlugin>,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledPlugin {
    plugin_id: String,
    version: String,
    installed_at: String,
    manifest: PluginManifest,
    path: PathBuf,
}

impl PluginRegistry {
    /// Create or load a plugin registry
    pub fn new(registry_path: PathBuf) -> Result<Self, PluginError> {
        let data = if registry_path.exists() {
            Self::load_registry(&registry_path)?
        } else {
            // Create new registry
            let parent = registry_path
                .parent()
                .ok_or_else(|| PluginError::RegistryError("Invalid registry path".to_string()))?;

            fs::create_dir_all(parent)
                .map_err(|e| PluginError::RegistryError(format!("Failed to create registry directory: {}", e)))?;

            RegistryData {
                plugins: HashMap::new(),
                version: "1".to_string(),
            }
        };

        Ok(Self {
            registry_path,
            data,
        })
    }

    /// Register an installed plugin
    pub fn register_plugin(&mut self, plugin: &PluginInfo) -> Result<(), PluginError> {
        let installed = InstalledPlugin {
            plugin_id: plugin.id.clone(),
            version: plugin.version.clone(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            manifest: plugin.manifest.clone(),
            path: PathBuf::from(format!(".gameengine/plugins/{}", plugin.id)),
        };

        self.data.plugins.insert(plugin.id.clone(), installed);
        self.save_registry()?;

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        self.data.plugins
            .remove(plugin_id)
            .ok_or_else(|| PluginError::NotFound(format!("Plugin {} not found in registry", plugin_id)))?;

        self.save_registry()?;
        Ok(())
    }

    /// Get plugin information
    pub fn get_plugin(&self, plugin_id: &str) -> Result<InstalledPluginView, PluginError> {
        let plugin = self
            .data.plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(format!("Plugin {} not found", plugin_id)))?;

        Ok(InstalledPluginView {
            id: plugin.plugin_id.clone(),
            version: plugin.version.clone(),
            installed_at: plugin.installed_at.clone(),
            manifest: plugin.manifest.clone(),
            path: plugin.path.clone(),
        })
    }

    /// List all installed plugins
    pub fn list_installed(&self) -> Result<Vec<PluginInfo>, PluginError> {
        let plugins: Vec<PluginInfo> = self
            .data
            .plugins
            .values()
            .map(|p| PluginInfo {
                id: p.plugin_id.clone(),
                name: p.manifest.display_name.clone(),
                description: p.manifest.description.clone(),
                author: PluginAuthor {
                    id: "unknown".to_string(),
                    name: "Unknown".to_string(),
                    email: None,
                    avatar: None,
                    website: None,
                },
                version: p.version.clone(),
                latest_version: p.version.clone(),
                categories: Vec::new(),
                tags: Vec::new(),
                license: "Unknown".to_string(),
                homepage: None,
                repository: None,
                documentation: None,
                screenshots: Vec::new(),
                videos: Vec::new(),
                rating: RatingInfo {
                    average: 0.0,
                    count: 0,
                    distribution: HashMap::new(),
                },
                downloads: 0,
                created_at: p.installed_at.clone(),
                updated_at: p.installed_at.clone(),
                dependencies: Vec::new(),
                compatibility: CompatibilityInfo {
                    engine_version_min: "1.0.0".to_string(),
                    engine_version_max: None,
                    platforms: Vec::new(),
                    features: Vec::new(),
                },
                pricing: PricingInfo {
                    pricing_type: PricingType::Free,
                    price: None,
                    currency: None,
                    trial_available: false,
                    subscription: None,
                },
                manifest: p.manifest.clone(),
            })
            .collect();

        Ok(plugins)
    }

    /// Check if a plugin is installed
    pub fn is_installed(&self, plugin_id: &str) -> bool {
        self.data.plugins.contains_key(plugin_id)
    }

    /// Get installed plugin count
    pub fn count(&self) -> usize {
        self.data.plugins.len()
    }

    /// Resolve dependencies
    pub fn resolve_dependencies(&self, dependencies: &[PluginDependency]) -> Result<DependencyResolution, PluginError> {
        let mut resolved = Vec::new();
        let mut missing = Vec::new();
        let mut conflicts = Vec::new();

        for dep in dependencies {
            if let Ok(installed) = self.get_plugin(&dep.plugin_id) {
                // Check version requirement
                if self.satisfies_version(&installed.version, &dep.version_requirement) {
                    resolved.push(dep.plugin_id.clone());
                } else {
                    conflicts.push(VersionConflict {
                        plugin_id: dep.plugin_id.clone(),
                        required: dep.version_requirement.clone(),
                        installed: installed.version,
                    });
                }
            } else if !dep.optional {
                missing.push(dep.plugin_id.clone());
            }
        }

        Ok(DependencyResolution {
            resolved,
            missing,
            conflicts,
        })
    }

    fn satisfies_version(&self, installed: &str, requirement: &str) -> bool {
        // Simple version comparison (can be enhanced with semver crate)
        if requirement.starts_with(">=") {
            let min_version = &requirement[2..];
            installed >= min_version
        } else if requirement.starts_with("^") {
            // Caret version (compatible updates)
            let min_version = &requirement[1..];
            self.compatible_version(installed, min_version)
        } else {
            installed == requirement
        }
    }

    fn compatible_version(&self, installed: &str, min: &str) -> bool {
        // Very simplified check - should use semver crate
        let installed_parts: Vec<u32> = installed
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let min_parts: Vec<u32> = min
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if installed_parts.is_empty() || min_parts.is_empty() {
            return installed >= min;
        }

        if installed_parts[0] != min_parts[0] {
            return false;
        }

        installed >= min
    }

    fn load_registry(path: &Path) -> Result<RegistryData, PluginError> {
        let content = fs::read_to_string(path)
            .map_err(|e| PluginError::RegistryError(format!("Failed to read registry: {}", e)))?;

        let data: RegistryData = serde_json::from_str(&content)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse registry: {}", e)))?;

        Ok(data)
    }

    fn save_registry(&self) -> Result<(), PluginError> {
        let content = serde_json::to_string_pretty(&self.data)
            .map_err(|e| PluginError::Serialization(format!("Failed to serialize registry: {}", e)))?;

        fs::write(&self.registry_path, content)
            .map_err(|e| PluginError::RegistryError(format!("Failed to write registry: {}", e)))?;

        Ok(())
    }
}

/// View of an installed plugin
#[derive(Debug, Clone)]
pub struct InstalledPluginView {
    pub id: String,
    pub version: String,
    pub installed_at: String,
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

/// Dependency resolution result
#[derive(Debug)]
pub struct DependencyResolution {
    pub resolved: Vec<String>,
    pub missing: Vec<String>,
    pub conflicts: Vec<VersionConflict>,
}

/// Version conflict
#[derive(Debug)]
pub struct VersionConflict {
    pub plugin_id: String,
    pub required: String,
    pub installed: String,
}
