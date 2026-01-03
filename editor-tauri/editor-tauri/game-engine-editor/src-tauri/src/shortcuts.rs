/**
 * 快捷键持久化 (Rust后端)
 * 提供快捷键配置的保存和加载功能
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub version: String,
    pub shortcuts: HashMap<String, KeySequence>,
    pub disabled: Vec<String>,
    pub metadata: Option<ConfigMetadata>,
}

/// 按键序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySequence {
    pub keys: Vec<KeyCombo>,
}

/// 按键组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCombo {
    pub key: String,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

/// 配置元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub exported_at: String,
    pub platform: String,
    pub preset: Option<String>,
}

/// 快捷键持久化管理器
pub struct ShortcutPersistence {
    config_path: PathBuf,
}

impl ShortcutPersistence {
    /// 创建新的持久化管理器
    pub fn new(config_dir: &Path) -> Self {
        let config_path = config_dir.join("shortcuts.json");
        Self { config_path }
    }

    /// 保存快捷键配置
    pub fn save_config(&self, config: &ShortcutConfig) -> Result<(), String> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // 序列化为JSON
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // 写入文件
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// 加载快捷键配置
    pub fn load_config(&self) -> Result<Option<ShortcutConfig>, String> {
        // 检查文件是否存在
        if !self.config_path.exists() {
            return Ok(None);
        }

        // 读取文件
        let json = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        // 反序列化
        let config: ShortcutConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(Some(config))
    }

    /// 导出快捷键配置到指定路径
    pub fn export_config(&self, config: &ShortcutConfig, path: &Path) -> Result<(), String> {
        // 序列化为JSON
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // 写入文件
        fs::write(path, json)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        Ok(())
    }

    /// 从指定路径导入快捷键配置
    pub fn import_config(&self, path: &Path) -> Result<ShortcutConfig, String> {
        // 读取文件
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;

        // 反序列化
        let config: ShortcutConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(config)
    }

    /// 删除配置文件
    pub fn delete_config(&self) -> Result<(), String> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| format!("Failed to delete config file: {}", e))?;
        }
        Ok(())
    }

    /// 备份当前配置
    pub fn backup_config(&self) -> Result<PathBuf, String> {
        if !self.config_path.exists() {
            return Err("Config file does not exist".to_string());
        }

        // 创建备份文件名
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.config_path.with_extension(format!("json.{}", timestamp));

        // 复制文件
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        Ok(backup_path)
    }

    /// 获取所有备份文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, String> {
        let config_dir = self.config_path.parent().ok_or("No config directory")?;
        let config_name = self.config_path
            .file_name()
            .ok_or("No config file name")?
            .to_string_lossy()
            .replace(".json", "");

        let mut backups = Vec::new();

        for entry in fs::read_dir(config_dir)
            .map_err(|e| format!("Failed to read config directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // 检查是否为备份文件
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with(&format!("{}.", config_name)) && name_str.ends_with(".json") {
                    backups.push(path);
                }
            }
        }

        // 按修改时间排序（最新的在前）
        backups.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        Ok(backups)
    }
}

/// Tauri命令：保存快捷键配置
#[tauri::command]
pub fn save_shortcut_config(app: AppHandle, config: ShortcutConfig) -> Result<(), String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    persistence.save_config(&config)
}

/// Tauri命令：加载快捷键配置
#[tauri::command]
pub fn load_shortcut_config(app: AppHandle) -> Result<Option<ShortcutConfig>, String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    persistence.load_config()
}

/// Tauri命令：导出快捷键配置
#[tauri::command]
pub fn export_shortcut_config(
    app: AppHandle,
    config: ShortcutConfig,
    path: String,
) -> Result<(), String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    let export_path = PathBuf::from(path);
    persistence.export_config(&config, &export_path)
}

/// Tauri命令：导入快捷键配置
#[tauri::command]
pub fn import_shortcut_config(app: AppHandle, path: String) -> Result<ShortcutConfig, String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    let import_path = PathBuf::from(path);
    persistence.import_config(&import_path)
}

/// Tauri命令：重置快捷键配置
#[tauri::command]
pub fn reset_shortcut_config(app: AppHandle) -> Result<(), String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    persistence.delete_config()
}

/// Tauri命令：备份快捷键配置
#[tauri::command]
pub fn backup_shortcut_config(app: AppHandle) -> Result<String, String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    let backup_path = persistence.backup_config()?;
    Ok(backup_path.to_string_lossy().to_string())
}

/// Tauri命令：列出备份文件
#[tauri::command]
pub fn list_shortcut_backups(app: AppHandle) -> Result<Vec<String>, String> {
    let config_dir = app.path().config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let persistence = ShortcutPersistence::new(&config_dir);
    let backups = persistence.list_backups()?;
    Ok(backups.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ShortcutPersistence::new(temp_dir.path());

        let mut config = ShortcutConfig {
            version: "1.0.0".to_string(),
            shortcuts: HashMap::new(),
            disabled: Vec::new(),
            metadata: None,
        };

        config.shortcuts.insert("test.shortcut".to_string(), KeySequence {
            keys: vec![KeyCombo {
                key: "A".to_string(),
                ctrl: true,
                shift: false,
                alt: false,
                meta: false,
            }],
        });

        // 保存配置
        persistence.save_config(&config).unwrap();

        // 加载配置
        let loaded = persistence.load_config().unwrap().unwrap();
        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.shortcuts.len(), 1);
        assert!(loaded.shortcuts.contains_key("test.shortcut"));
    }

    #[test]
    fn test_export_and_import_config() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = ShortcutPersistence::new(temp_dir.path());

        let config = ShortcutConfig {
            version: "1.0.0".to_string(),
            shortcuts: HashMap::new(),
            disabled: Vec::new(),
            metadata: None,
        };

        let export_path = temp_dir.path().join("exported.json");

        // 导出配置
        persistence.export_config(&config, &export_path).unwrap();

        // 导入配置
        let imported = persistence.import_config(&export_path).unwrap();
        assert_eq!(imported.version, "1.0.0");
    }
}
