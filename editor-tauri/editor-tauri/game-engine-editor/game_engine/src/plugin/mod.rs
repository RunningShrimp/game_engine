//! Plugin Marketplace Module
//!
//! This module provides functionality for discovering, installing, and managing plugins
//! from a centralized marketplace.

pub mod marketplace;
pub mod registry;
pub mod installer;
pub mod cli;
pub mod models;

pub use marketplace::{Marketplace, MarketplaceConfig};
pub use registry::{PluginRegistry, PluginInfo, PluginVersion};
pub use installer::{PluginInstaller, InstallOptions};
pub use cli::PluginCli;
pub use models::*;

use std::path::PathBuf;

/// Main plugin manager that coordinates all plugin operations
pub struct PluginManager {
    marketplace: Marketplace,
    registry: PluginRegistry,
    installer: PluginInstaller,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginManagerConfig) -> Result<Self, PluginError> {
        let marketplace = Marketplace::new(config.marketplace_config)?;
        let registry = PluginRegistry::new(config.registry_path)?;
        let installer = PluginInstaller::new(config.install_path)?;

        Ok(Self {
            marketplace,
            registry,
            installer,
        })
    }

    /// Search for plugins in the marketplace
    pub async fn search(&self, query: &str, filters: SearchFilters) -> Result<Vec<PluginInfo>, PluginError> {
        self.marketplace.search(query, filters).await
    }

    /// Install a plugin
    pub async fn install(&mut self, plugin_id: &str, version: Option<&str>) -> Result<InstallResult, PluginError> {
        let plugin_info = self.marketplace.get_plugin(plugin_id).await?;
        let version_to_install = version.unwrap_or(&plugin_info.latest_version);

        let plugin_package = self.marketplace.download_plugin(plugin_id, version_to_install).await?;

        self.installer.install(&plugin_package).await?;

        self.registry.register_plugin(&plugin_info)?;

        Ok(InstallResult {
            plugin_id: plugin_id.to_string(),
            version: version_to_install.to_string(),
            path: plugin_package.install_path,
        })
    }

    /// Uninstall a plugin
    pub async fn uninstall(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let plugin_info = self.registry.get_plugin(plugin_id)?;

        self.installer.uninstall(plugin_id).await?;
        self.registry.unregister_plugin(plugin_id)?;

        Ok(())
    }

    /// Update a plugin to the latest version
    pub async fn update(&mut self, plugin_id: &str) -> Result<UpdateResult, PluginError> {
        let installed = self.registry.get_plugin(plugin_id)?;
        let latest = self.marketplace.get_plugin(plugin_id).await?;

        if installed.version == latest.latest_version {
            return Ok(UpdateResult::AlreadyUpToDate);
        }

        self.install(plugin_id, Some(&latest.latest_version)).await?;

        Ok(UpdateResult::Updated {
            old_version: installed.version,
            new_version: latest.latest_version,
        })
    }

    /// List all installed plugins
    pub fn list_installed(&self) -> Result<Vec<PluginInfo>, PluginError> {
        self.registry.list_installed()
    }

    /// Check for updates for installed plugins
    pub async fn check_updates(&self) -> Result<Vec<UpdateAvailable>, PluginError> {
        let installed = self.registry.list_installed()?;
        let mut updates = Vec::new();

        for plugin in installed {
            let latest = self.marketplace.get_plugin(&plugin.id).await?;
            if plugin.version != latest.latest_version {
                updates.push(UpdateAvailable {
                    plugin_id: plugin.id.clone(),
                    plugin_name: plugin.name.clone(),
                    current_version: plugin.version,
                    latest_version: latest.latest_version,
                });
            }
        }

        Ok(updates)
    }
}

/// Configuration for the plugin manager
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    pub marketplace_config: MarketplaceConfig,
    pub registry_path: PathBuf,
    pub install_path: PathBuf,
}

/// Result of installing a plugin
#[derive(Debug)]
pub struct InstallResult {
    pub plugin_id: String,
    pub version: String,
    pub path: PathBuf,
}

/// Result of updating a plugin
#[derive(Debug)]
pub enum UpdateResult {
    Updated { old_version: String, new_version: String },
    AlreadyUpToDate,
}

/// Information about an available update
#[derive(Debug)]
pub struct UpdateAvailable {
    pub plugin_id: String,
    pub plugin_name: String,
    pub current_version: String,
    pub latest_version: String,
}

/// Error types for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Marketplace error: {0}")]
    MarketplaceError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Installer error: {0}")]
    InstallerError(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
