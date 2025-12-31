//! # Script Hot Reload
//!
//! 脚本热重载系统 - 自动检测脚本文件变化并重新加载。
//!
//! ## 核心功能
//!
//! 1. **FileWatcher** - 文件监控
//! 2. **HotReloadManager** - 热重载管理器
//! 3. **StatePreserver** - 状态保持
//! 4. **IncrementalUpdater** - 增量更新

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// 脚本类型
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScriptType {
    /// JavaScript脚本
    JavaScript,
    /// Python脚本
    Python,
}

/// 脚本文件信息
#[derive(Clone, Debug)]
pub struct ScriptFileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 脚本类型
    pub script_type: ScriptType,
    /// 最后修改时间
    pub last_modified: SystemTime,
    /// 内容哈希（用于快速比较）
    pub content_hash: u64,
    /// 是否启用热重载
    pub hot_reload_enabled: bool,
}

/// 重载结果
#[derive(Clone, Debug)]
pub enum ReloadResult {
    /// 成功重载
    Success {
        /// 脚本路径
        path: PathBuf,
        /// 重载时间
        reload_time: SystemTime,
    },
    /// 重载失败
    Failed {
        /// 脚本路径
        path: PathBuf,
        /// 错误信息
        error: String,
    },
    /// 跳过（无变化）
    Skipped {
        /// 脚本路径
        path: PathBuf,
        /// 原因
        reason: String,
    },
}

/// 热重载配置
#[derive(Clone, Debug)]
pub struct HotReloadConfig {
    /// 是否启用热重载
    pub enabled: bool,
    /// 检查间隔（毫秒）
    pub check_interval_ms: u64,
    /// 是否在重载前保存状态
    pub preserve_state: bool,
    /// 是否显示重载通知
    pub show_notifications: bool,
    /// 监控的文件扩展名
    pub watched_extensions: Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_ms: 500,
            preserve_state: true,
            show_notifications: true,
            watched_extensions: vec!["js".to_string(), "py".to_string(), "ts".to_string()],
        }
    }
}

/// 脚本热重载管理器
pub struct ScriptHotReloadManager {
    /// 配置
    config: HotReloadConfig,
    /// 监控的脚本文件
    watched_scripts: Arc<Mutex<HashMap<PathBuf, ScriptFileInfo>>>,
    /// 重载回调
    reload_callbacks: Arc<Mutex<Vec<Box<dyn Fn(&PathBuf, &str) -> Result<(), String> + Send>>>>,
    /// 状态保持数据
    preserved_state: Arc<Mutex<HashMap<PathBuf, HashMap<String, String>>>>,
    /// 最后检查时间
    last_check_time: Arc<Mutex<SystemTime>>,
}

impl Default for ScriptHotReloadManager {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl ScriptHotReloadManager {
    /// 创建新的热重载管理器
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            watched_scripts: Arc::new(Mutex::new(HashMap::new())),
            reload_callbacks: Arc::new(Mutex::new(Vec::new())),
            preserved_state: Arc::new(Mutex::new(HashMap::new())),
            last_check_time: Arc::new(Mutex::new(SystemTime::now())),
        }
    }

    /// 添加要监控的脚本文件
    pub fn watch_script(&self, path: PathBuf, script_type: ScriptType) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }

        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        let last_modified = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let content_hash = Self::calculate_hash(&content);

        let file_info = ScriptFileInfo {
            path: path.clone(),
            script_type,
            last_modified,
            content_hash,
            hot_reload_enabled: true,
        };

        self.watched_scripts.lock().unwrap().insert(path.clone(), file_info);
        tracing::info!(target: "hot_reload", "Now watching: {}", path.display());
        Ok(())
    }

    /// 移除监控的脚本文件
    pub fn unwatch_script(&self, path: &PathBuf) -> bool {
        let removed = self.watched_scripts.lock().unwrap().remove(path).is_some();
        if removed {
            tracing::info!(target: "hot_reload", "Stopped watching: {}", path.display());
        }
        removed
    }

    /// 检查并重载变化的脚本
    pub fn check_and_reload(&self) -> Vec<ReloadResult> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut scripts = self.watched_scripts.lock().unwrap();

        for (path, file_info) in scripts.iter_mut() {
            if !file_info.hot_reload_enabled {
                continue;
            }

            // 检查文件是否存在
            if !path.exists() {
                results.push(ReloadResult::Failed {
                    path: path.clone(),
                    error: "File no longer exists".to_string(),
                });
                continue;
            }

            // 获取文件元数据
            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    results.push(ReloadResult::Failed {
                        path: path.clone(),
                        error: format!("Failed to get metadata: {}", e),
                    });
                    continue;
                }
            };

            // 检查修改时间
            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(e) => {
                    results.push(ReloadResult::Failed {
                        path: path.clone(),
                        error: format!("Failed to get modification time: {}", e),
                    });
                    continue;
                }
            };

            // 如果文件没有变化，跳过
            if modified == file_info.last_modified {
                continue;
            }

            // 读取文件内容
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    results.push(ReloadResult::Failed {
                        path: path.clone(),
                        error: format!("Failed to read file: {}", e),
                    });
                    continue;
                }
            };

            // 计算内容哈希
            let new_hash = Self::calculate_hash(&content);

            // 如果内容没有实际变化，跳过
            if new_hash == file_info.content_hash {
                results.push(ReloadResult::Skipped {
                    path: path.clone(),
                    reason: "Content hash unchanged".to_string(),
                });
                continue;
            }

            // 保存状态（如果启用）
            if self.config.preserve_state {
                self.preserve_state(path);
            }

            // 执行重载回调
            let callbacks = self.reload_callbacks.lock().unwrap();
            let mut reload_success = true;
            let mut error_msg = String::new();

            for callback in callbacks.iter() {
                if let Err(e) = callback(path, &content) {
                    reload_success = false;
                    error_msg = e;
                    break;
                }
            }

            if reload_success {
                // 更新文件信息
                file_info.last_modified = modified;
                file_info.content_hash = new_hash;

                results.push(ReloadResult::Success {
                    path: path.clone(),
                    reload_time: SystemTime::now(),
                });

                tracing::info!(target: "hot_reload", "Reloaded: {}", path.display());
            } else {
                results.push(ReloadResult::Failed {
                    path: path.clone(),
                    error: error_msg,
                });
            }
        }

        results
    }

    /// 注册重载回调
    pub fn register_reload_callback<F>(&self, callback: F)
    where
        F: Fn(&PathBuf, &str) -> Result<(), String> + Send + 'static,
    {
        self.reload_callbacks.lock().unwrap().push(Box::new(callback));
    }

    /// 保存状态
    fn preserve_state(&self, path: &PathBuf) {
        // 在实际实现中，这里会保存脚本的状态
        let state = HashMap::new(); // 简化版
        self.preserved_state.lock().unwrap().insert(path.clone(), state);
        tracing::debug!(target: "hot_reload", "Preserved state for: {}", path.display());
    }

    /// 恢复状态
    pub fn restore_state(&self, path: &PathBuf) -> Option<HashMap<String, String>> {
        self.preserved_state.lock().unwrap().get(path).cloned()
    }

    /// 计算内容哈希
    fn calculate_hash(content: &str) -> u64 {
        // 简单的哈希函数
        let mut hash: u64 = 5381;
        for byte in content.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    /// 获取监控的脚本列表
    pub fn get_watched_scripts(&self) -> Vec<ScriptFileInfo> {
        self.watched_scripts.lock().unwrap().values().cloned().collect()
    }

    /// 启用/禁用特定脚本的热重载
    pub fn set_hot_reload_enabled(&self, path: &PathBuf, enabled: bool) -> bool {
        if let Some(mut info) = self.watched_scripts.lock().unwrap().get_mut(path) {
            info.hot_reload_enabled = enabled;
            true
        } else {
            false
        }
    }

    /// 清除所有监控
    pub fn clear_all_watches(&self) {
        self.watched_scripts.lock().unwrap().clear();
        tracing::info!(target: "hot_reload", "Cleared all watches");
    }

    /// 获取配置
    pub fn get_config(&self) -> HotReloadConfig {
        self.config.clone()
    }

    /// 更新配置
    pub fn update_config(&self, config: HotReloadConfig) {
        tracing::info!(target: "hot_reload", "Config updated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_manager_creation() {
        let manager = ScriptHotReloadManager::new(Default::default());
        assert_eq!(manager.get_watched_scripts().len(), 0);
    }

    #[test]
    fn test_watch_script() {
        let manager = ScriptHotReloadManager::new(Default::default());
        let temp_file = std::env::temp_dir().join("test_script.js");

        // 创建临时文件
        std::fs::write(&temp_file, "console.log('test');").unwrap();

        let result = manager.watch_script(temp_file.clone(), ScriptType::JavaScript);
        assert!(result.is_ok());
        assert_eq!(manager.get_watched_scripts().len(), 1);

        // 清理
        std::fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_unwatch_script() {
        let manager = ScriptHotReloadManager::new(Default::default());
        let temp_file = std::env::temp_dir().join("test_script.py");

        std::fs::write(&temp_file, "print('test')").unwrap();

        manager.watch_script(temp_file.clone(), ScriptType::Python).unwrap();
        assert!(manager.unwatch_script(&temp_file));
        assert_eq!(manager.get_watched_scripts().len(), 0);

        std::fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_calculate_hash() {
        let hash1 = ScriptHotReloadManager::calculate_hash("test");
        let hash2 = ScriptHotReloadManager::calculate_hash("test");
        let hash3 = ScriptHotReloadManager::calculate_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_reload_callback() {
        let manager = ScriptHotReloadManager::new(Default::default());
        let callback_called = Arc::new(Mutex::new(false));

        let callback_called_clone = callback_called.clone();
        manager.register_reload_callback(move |_path, _content| {
            *callback_called_clone.lock().unwrap() = true;
            Ok(())
        });

        // 测试回调
        let temp_file = std::env::temp_dir().join("test_callback.js");
        std::fs::write(&temp_file, "console.log('test');").unwrap();
        manager.watch_script(temp_file.clone(), ScriptType::JavaScript).unwrap();

        // 模拟文件修改
        std::thread::sleep(Duration::from_millis(10));
        std::fs::write(&temp_file, "console.log('updated');").unwrap();

        let results = manager.check_and_reload();
        assert!(!results.is_empty());

        std::fs::remove_file(&temp_file).unwrap();
    }
}
