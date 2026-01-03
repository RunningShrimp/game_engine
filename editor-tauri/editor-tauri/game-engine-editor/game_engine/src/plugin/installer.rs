//! Plugin installer for installing and uninstalling plugins

use super::{PluginError, PluginPackage};
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin installer
pub struct PluginInstaller {
    install_path: PathBuf,
}

/// Installation options
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub force: bool,
    pub skip_dependencies: bool,
    pub no_confirm: bool,
}

impl PluginInstaller {
    /// Create a new plugin installer
    pub fn new(install_path: PathBuf) -> Result<Self, PluginError> {
        if !install_path.exists() {
            fs::create_dir_all(&install_path)
                .map_err(|e| PluginError::InstallerError(format!("Failed to create install directory: {}", e)))?;
        }

        Ok(Self { install_path })
    }

    /// Install a plugin package
    pub async fn install(&self, package: &PluginPackage) -> Result<InstallResult, PluginError> {
        let plugin_dir = self.install_path.join(&package.plugin_id);

        // Check if already installed
        if plugin_dir.exists() {
            return Err(PluginError::InstallerError(format!(
                "Plugin {} is already installed at {:?}",
                package.plugin_id, plugin_dir
            )));
        }

        // Create plugin directory
        fs::create_dir_all(&plugin_dir)
            .map_err(|e| PluginError::InstallerError(format!("Failed to create plugin directory: {}", e)))?;

        // Install files
        for file in &package.files {
            let file_path = plugin_dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| PluginError::InstallerError(format!("Failed to create directory: {}", e)))?;
            }

            // Write file
            fs::write(&file_path, &file.content)
                .map_err(|e| PluginError::InstallerError(format!("Failed to write file: {}", e)))?;

            // Set executable permission if needed
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path)
                    .map_err(|e| PluginError::InstallerError(format!("Failed to get metadata: {}", e)))?
                    .permissions();
                perms.set_mode(perms.mode() | 0o755);
                fs::set_permissions(&file_path, perms)
                    .map_err(|e| PluginError::InstallerError(format!("Failed to set permissions: {}", e)))?;
            }
        }

        // Run post-install script if present
        let post_install_script = plugin_dir.join("scripts").join("post_install.sh");
        if post_install_script.exists() {
            self.run_script(&post_install_script, &plugin_dir)?;
        }

        Ok(InstallResult {
            plugin_id: package.plugin_id.clone(),
            version: package.version.clone(),
            path: plugin_dir,
        })
    }

    /// Uninstall a plugin
    pub async fn uninstall(&self, plugin_id: &str) -> Result<(), PluginError> {
        let plugin_dir = self.install_path.join(plugin_id);

        if !plugin_dir.exists() {
            return Err(PluginError::NotFound(format!(
                "Plugin {} is not installed",
                plugin_id
            )));
        }

        // Run pre-uninstall script if present
        let pre_uninstall_script = plugin_dir.join("scripts").join("pre_uninstall.sh");
        if pre_uninstall_script.exists() {
            self.run_script(&pre_uninstall_script, &plugin_dir)?;
        }

        // Remove plugin directory
        fs::remove_dir_all(&plugin_dir)
            .map_err(|e| PluginError::InstallerError(format!("Failed to remove plugin directory: {}", e)))?;

        Ok(())
    }

    /// Update an installed plugin
    pub async fn update(&self, package: &PluginPackage) -> Result<UpdateResult, PluginError> {
        let plugin_dir = self.install_path.join(&package.plugin_id);

        if !plugin_dir.exists() {
            return Err(PluginError::NotFound(format!(
                "Plugin {} is not installed",
                package.plugin_id
            )));
        }

        // Backup current installation
        let backup_dir = self.install_path.join(format!(".backup_{}", package.plugin_id));
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir)
                .map_err(|e| PluginError::InstallerError(format!("Failed to remove backup: {}", e)))?;
        }

        self.copy_directory(&plugin_dir, &backup_dir)?;

        // Remove old installation
        fs::remove_dir_all(&plugin_dir)
            .map_err(|e| PluginError::InstallerError(format!("Failed to remove old installation: {}", e)))?;

        // Install new version
        match self.install(package).await {
            Ok(result) => {
                // Remove backup
                let _ = fs::remove_dir_all(&backup_dir);
                Ok(UpdateResult::Success {
                    path: result.path,
                    version: package.version.clone(),
                })
            }
            Err(e) => {
                // Restore from backup
                let _ = self.copy_directory(&backup_dir, &plugin_dir);
                let _ = fs::remove_dir_all(&backup_dir);
                Err(e)
            }
        }
    }

    /// Get installed plugin info
    pub fn get_installed(&self, plugin_id: &str) -> Result<InstalledPluginInfo, PluginError> {
        let plugin_dir = self.install_path.join(plugin_id);

        if !plugin_dir.exists() {
            return Err(PluginError::NotFound(format!("Plugin {} not found", plugin_id)));
        }

        let manifest_path = plugin_dir.join("plugin.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::InstallerError(format!("Failed to read manifest: {}", e)))?;

        let manifest: super::PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| PluginError::Serialization(format!("Failed to parse manifest: {}", e)))?;

        // Get file list and size
        let mut files = Vec::new();
        let mut total_size = 0;

        for entry in walkdir::WalkDir::new(&plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_type().is_dir())
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(&plugin_dir)
                .map_err(|e| PluginError::InstallerError(format!("Failed to get relative path: {}", e)))?;

            let metadata = fs::metadata(path)
                .map_err(|e| PluginError::InstallerError(format!("Failed to get metadata: {}", e)))?;

            total_size += metadata.len();

            files.push(InstalledFileInfo {
                path: relative_path.to_string_lossy().to_string(),
                size: metadata.len(),
                modified: metadata.modified()
                    .map(|t| {
                        t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0)
                    })
                    .unwrap_or(0),
            });
        }

        Ok(InstalledPluginInfo {
            id: plugin_id.to_string(),
            version: manifest.version.clone(),
            path: plugin_dir,
            manifest,
            files,
            total_size,
        })
    }

    /// List all installed plugins
    pub fn list_installed(&self) -> Result<Vec<String>, PluginError> {
        let mut plugins = Vec::new();

        for entry in fs::read_dir(&self.install_path)
            .map_err(|e| PluginError::InstallerError(format!("Failed to read install directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| PluginError::InstallerError(format!("Failed to read directory entry: {}", e)))?;

            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip backup directories
                    if !name.starts_with('.') {
                        plugins.push(name.to_string());
                    }
                }
            }
        }

        Ok(plugins)
    }

    fn run_script(&self, script_path: &Path, working_dir: &Path) -> Result<(), PluginError> {
        #[cfg(unix)]
        {
            use std::process::Command;

            let status = Command::new("bash")
                .arg(script_path)
                .current_dir(working_dir)
                .status()
                .map_err(|e| PluginError::InstallerError(format!("Failed to run script: {}", e)))?;

            if !status.success() {
                return Err(PluginError::InstallerError(format!(
                    "Script failed with exit code: {:?}",
                    status.code()
                )));
            }
        }

        Ok(())
    }

    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<(), PluginError> {
        if dst.exists() {
            fs::remove_dir_all(dst)
                .map_err(|e| PluginError::InstallerError(format!("Failed to remove directory: {}", e)))?;
        }

        fs::create_dir_all(dst)
            .map_err(|e| PluginError::InstallerError(format!("Failed to create directory: {}", e)))?;

        for entry in walkdir::WalkDir::new(src)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();
            let relative_path = src_path
                .strip_prefix(src)
                .map_err(|e| PluginError::InstallerError(format!("Failed to get relative path: {}", e)))?;

            let dst_path = dst.join(relative_path);

            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)
                    .map_err(|e| PluginError::InstallerError(format!("Failed to create directory: {}", e)))?;
            } else {
                fs::copy(src_path, &dst_path)
                    .map_err(|e| PluginError::InstallerError(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }
}

/// Installation result
#[derive(Debug)]
pub struct InstallResult {
    pub plugin_id: String,
    pub version: String,
    pub path: PathBuf,
}

/// Update result
#[derive(Debug)]
pub enum UpdateResult {
    Success { path: PathBuf, version: String },
    RolledBack,
}

/// Installed plugin information
#[derive(Debug)]
pub struct InstalledPluginInfo {
    pub id: String,
    pub version: String,
    pub path: PathBuf,
    pub manifest: super::PluginManifest,
    pub files: Vec<InstalledFileInfo>,
    pub total_size: u64,
}

/// Installed file information
#[derive(Debug)]
pub struct InstalledFileInfo {
    pub path: String,
    pub size: u64,
    pub modified: u64,
}
