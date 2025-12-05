//! 插件热加载系统
//!
//! 支持运行时动态加载和卸载插件，无需重启引擎

use super::{EnginePlugin, PluginRegistry, PluginError, PluginResult};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use thiserror::Error;

#[cfg(unix)]
use libloading::{Library, Symbol};

#[cfg(windows)]
use libloading::{Library, Symbol};

/// 热加载错误
#[derive(Error, Debug)]
pub enum HotReloadError {
    #[error("Failed to load library: {0}")]
    LoadError(String),
    #[error("Failed to get plugin symbol: {0}")]
    SymbolError(String),
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    #[error("File system error: {0}")]
    FileSystemError(String),
}

/// 动态加载的插件句柄
pub struct DynamicPluginHandle {
    /// 库句柄
    library: Library,
    /// 插件实例
    plugin: Box<dyn EnginePlugin>,
    /// 插件路径
    path: PathBuf,
    /// 最后修改时间
    last_modified: SystemTime,
}

/// 插件热加载管理器
pub struct HotReloadManager {
    /// 动态加载的插件
    dynamic_plugins: HashMap<String, DynamicPluginHandle>,
    /// 插件目录
    plugin_directory: PathBuf,
    /// 文件监视器
    watcher: Option<notify::RecommendedWatcher>,
    /// 待重载的插件列表
    pending_reloads: Arc<Mutex<Vec<String>>>,
    /// 是否启用热重载
    enabled: bool,
}

impl HotReloadManager {
    /// 创建热加载管理器
    pub fn new(plugin_directory: impl Into<PathBuf>) -> Result<Self, HotReloadError> {
        let plugin_dir = plugin_directory.into();
        
        // 确保目录存在
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        
        let pending_reloads = Arc::new(Mutex::new(Vec::new()));
        let pending_reloads_clone = pending_reloads.clone();
        
        // 创建文件监视器
        let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                if let EventKind::Modify(_) = event.kind {
                    for path in &event.paths {
                        if let Some(file_name) = path.file_name() {
                            if let Some(name) = file_name.to_str() {
                                // 检查是否是插件文件
                                if name.ends_with(".so") || name.ends_with(".dylib") || name.ends_with(".dll") {
                                    if let Ok(mut pending) = pending_reloads_clone.lock() {
                                        pending.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
        .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        
        // 监视插件目录
        watcher.watch(&plugin_dir, RecursiveMode::NonRecursive)
            .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        
        Ok(Self {
            dynamic_plugins: HashMap::new(),
            plugin_directory: plugin_dir,
            watcher: Some(watcher),
            pending_reloads,
            enabled: true,
        })
    }
    
    /// 加载动态插件
    pub fn load_plugin(&mut self, plugin_path: impl AsRef<Path>, registry: &mut PluginRegistry) -> Result<(), HotReloadError> {
        let plugin_path = plugin_path.as_ref();
        
        if !plugin_path.exists() {
            return Err(HotReloadError::LoadError(format!("Plugin file not found: {}", plugin_path.display())));
        }
        
        // 获取文件修改时间
        let metadata = std::fs::metadata(plugin_path)
            .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        let last_modified = metadata.modified()
            .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        
        // 加载动态库
        unsafe {
            let library = Library::new(plugin_path)
                .map_err(|e| HotReloadError::LoadError(format!("Failed to load library: {}", e)))?;
            
            // 获取插件创建函数
            // 约定：插件库必须导出 `create_plugin` 函数
            let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn EnginePlugin> = library
                .get(b"create_plugin")
                .map_err(|e| HotReloadError::SymbolError(format!("Failed to get create_plugin symbol: {}", e)))?;
            
            // 创建插件实例
            let plugin_ptr = create_plugin();
            if plugin_ptr.is_null() {
                return Err(HotReloadError::LoadError("create_plugin returned null".to_string()));
            }
            
            let plugin = unsafe { Box::from_raw(plugin_ptr) };
            let plugin_name = plugin.name().to_string();
            
            // 克隆插件元数据（因为 add 会消耗所有权）
            let plugin_metadata = plugin.metadata();
            
            // 添加到注册表
            registry.add(*plugin)
                .map_err(HotReloadError::PluginError)?;
            
            // 重新创建插件实例用于存储（因为注册表拥有原始实例）
            // 注意：实际实现中应该使用 Arc 或从注册表获取引用
            // 这里简化处理，重新创建实例
            let plugin_ptr2 = create_plugin();
            if plugin_ptr2.is_null() {
                // 如果重新创建失败，从注册表移除
                let _ = registry.remove_plugin(&plugin_name);
                return Err(HotReloadError::LoadError("Failed to create plugin instance for storage".to_string()));
            }
            let plugin2 = unsafe { Box::from_raw(plugin_ptr2) };
            
            // 存储句柄
            self.dynamic_plugins.insert(plugin_name.clone(), DynamicPluginHandle {
                library,
                plugin: plugin2,
                path: plugin_path.to_path_buf(),
                last_modified,
            });
            
            Ok(())
        }
    }
    
    /// 卸载插件
    pub fn unload_plugin(&mut self, plugin_name: &str, registry: &mut PluginRegistry) -> Result<(), HotReloadError> {
        if let Some(handle) = self.dynamic_plugins.remove(plugin_name) {
            // 从注册表移除（需要扩展 PluginRegistry 支持移除）
            // 这里简化处理，实际需要实现移除功能
            drop(handle.library); // 卸载库
            Ok(())
        } else {
            Err(HotReloadError::PluginNotFound(plugin_name.to_string()))
        }
    }
    
    /// 重载插件
    pub fn reload_plugin(&mut self, plugin_name: &str, registry: &mut PluginRegistry) -> Result<(), HotReloadError> {
        // 先卸载
        self.unload_plugin(plugin_name, registry)?;
        
        // 重新加载（需要保存路径）
        let plugin_path = {
            if let Some(handle) = self.dynamic_plugins.get(plugin_name) {
                handle.path.clone()
            } else {
                return Err(HotReloadError::PluginNotFound(plugin_name.to_string()));
            }
        };
        
        self.load_plugin(plugin_path, registry)?;
        
        Ok(())
    }
    
    /// 检查并处理待重载的插件
    pub fn check_and_reload(&mut self, registry: &mut PluginRegistry) -> Result<Vec<String>, HotReloadError> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        
        let pending = {
            let mut pending = self.pending_reloads.lock()
                .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
            pending.drain(..).collect::<Vec<_>>()
        };
        
        let mut reloaded = Vec::new();
        
        for plugin_name in pending {
            // 检查文件是否真的被修改
            if let Some(handle) = self.dynamic_plugins.get(&plugin_name) {
                if let Ok(metadata) = std::fs::metadata(&handle.path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > handle.last_modified {
                            // 文件已修改，重载插件
                            if self.reload_plugin(&plugin_name, registry).is_ok() {
                                reloaded.push(plugin_name);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(reloaded)
    }
    
    /// 扫描并加载所有插件
    pub fn scan_and_load(&mut self, registry: &mut PluginRegistry) -> Result<Vec<String>, HotReloadError> {
        let mut loaded = Vec::new();
        
        if !self.plugin_directory.exists() {
            return Ok(loaded);
        }
        
        let entries = std::fs::read_dir(&self.plugin_directory)
            .map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| HotReloadError::FileSystemError(e.to_string()))?;
            let path = entry.path();
            
            // 检查是否是插件文件
            if path.is_file() {
                let extension = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                if matches!(extension, "so" | "dylib" | "dll") {
                    if let Err(e) = self.load_plugin(&path, registry) {
                        eprintln!("Failed to load plugin {}: {}", path.display(), e);
                    } else {
                        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                            loaded.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// 启用/禁用热重载
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 获取已加载的插件列表
    pub fn loaded_plugins(&self) -> Vec<&str> {
        self.dynamic_plugins.keys().map(|s| s.as_str()).collect()
    }
}

impl Drop for HotReloadManager {
    fn drop(&mut self) {
        // 清理所有动态加载的插件
        self.dynamic_plugins.clear();
    }
}

/// 插件导出函数类型
/// 
/// 插件库必须导出此函数来创建插件实例
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn EnginePlugin {
    // 这是一个占位符，实际插件库会实现此函数
    std::ptr::null_mut()
}

