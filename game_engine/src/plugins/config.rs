//! 插件配置系统
//!
//! 管理插件的配置文件和启用/禁用状态

use crate::impl_default;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 插件路径（动态加载）
    pub path: Option<PathBuf>,
    /// 插件配置参数
    pub parameters: HashMap<String, String>,
    /// 加载优先级（数字越小优先级越高）
    pub priority: u32,
}

/// 插件配置管理器
pub struct PluginConfigManager {
    /// 插件配置
    configs: HashMap<String, PluginConfig>,
    /// 配置文件路径
    config_path: PathBuf,
}

impl PluginConfigManager {
    /// 创建配置管理器
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        let config_path = config_path.into();
        let configs = Self::load_configs_internal(&config_path).unwrap_or_default();
        
        Self {
            configs,
            config_path,
        }
    }
    
    /// 内部加载配置
    fn load_configs_internal(path: &PathBuf) -> Result<HashMap<String, PluginConfig>, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let configs: Vec<PluginConfig> = serde_json::from_str(&content)?;
            Ok(configs.into_iter().map(|c| (c.name.clone(), c)).collect())
        } else {
            Ok(HashMap::new())
        }
    }
    
    /// 获取插件配置
    pub fn get_config(&self, name: &str) -> Option<&PluginConfig> {
        self.configs.get(name)
    }
    
    /// 获取插件配置（可变）
    pub fn get_config_mut(&mut self, name: &str) -> Option<&mut PluginConfig> {
        self.configs.get_mut(name)
    }
    
    /// 添加或更新插件配置
    pub fn set_config(&mut self, config: PluginConfig) {
        self.configs.insert(config.name.clone(), config);
    }
    
    /// 启用插件
    pub fn enable_plugin(&mut self, name: &str) -> bool {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = true;
            true
        } else {
            false
        }
    }
    
    /// 禁用插件
    pub fn disable_plugin(&mut self, name: &str) -> bool {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = false;
            true
        } else {
            false
        }
    }
    
    /// 获取所有启用的插件配置
    pub fn enabled_plugins(&self) -> Vec<&PluginConfig> {
        let mut plugins: Vec<_> = self.configs.values()
            .filter(|c| c.enabled)
            .collect();
        plugins.sort_by_key(|c| c.priority);
        plugins
    }
    
    /// 获取所有插件配置
    pub fn all_configs(&self) -> Vec<&PluginConfig> {
        let mut configs: Vec<_> = self.configs.values().collect();
        configs.sort_by_key(|c| c.priority);
        configs
    }
    
    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let configs: Vec<&PluginConfig> = self.configs.values().collect();
        let content = serde_json::to_string_pretty(&configs)?;
        fs::write(&self.config_path, content)?;
        
        Ok(())
    }
    
    /// 加载配置
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.configs = Self::load_configs_internal(&self.config_path)?;
        Ok(())
    }
}

impl_default!(PluginConfig {
    name: String::new(),
    enabled: true,
    path: None,
    parameters: HashMap::new(),
    priority: 100,
});

