//! # Configuration System
//!
//! 本模块提供统一的引擎配置系统，支持多种配置来源和格式。
//!
//! ## 功能特性
//!
//! - **多格式支持** - TOML和JSON配置文件
//! - **环境变量覆盖** - 运行时通过环境变量调整配置
//! - **分层配置** - 图形、性能、音频、输入等独立模块
//! - **自动查找** - 智能查找配置文件位置
//! - **配置验证** - 自动验证配置参数有效性
//!
//! ## 主要组件
//!
//! - [`EngineConfig`] - 引擎主配置结构
//! - [`GraphicsConfig`] - 图形渲染配置
//! - [`PerformanceConfig`] - 性能优化配置
//! - [`AudioConfig`] - 音频系统配置
//! - [`InputConfig`] - 输入系统配置
//! - [`LoggingConfig`] - 日志系统配置
//! - [`ConfigError`] - 配置错误类型
//!
//! ## 使用示例
//!
//! ### 加载配置文件
//!
//! ```rust,no_run
//! use game_engine::config::EngineConfig;
//!
//! // 从TOML文件加载
//! let config = EngineConfig::from_toml_file("config.toml")
//!     .expect("Failed to load config.toml");
//!
//! // 从JSON文件加载
//! let config = EngineConfig::from_json_file("config.json")
//!     .expect("Failed to load config.json");
//!
//! // 自动查找配置文件
//! let config = EngineConfig::load_or_default();
//! ```
//!
//! ### 环境变量覆盖
//!
//! ```bash
//! # 设置图形配置
//! export ENGINE_GRAPHICS_WIDTH=1920
//! export ENGINE_GRAPHICS_HEIGHT=1080
//! export ENGINE_GRAPHICS_VSYNC=true
//!
//! # 设置性能配置
//! export ENGINE_PERFORMANCE_TARGET_FPS=60
//! export ENGINE_PERFORMANCE_AUTO_OPTIMIZE=true
//!
//! # 设置音频配置
//! export ENGINE_AUDIO_MASTER_VOLUME=0.8
//! ```
//!
//! ### 运行时调整
//!
//! ```rust,no_run
//! # use game_engine::config::EngineConfig;
//! let mut config = EngineConfig::new();
//!
//! // 应用环境变量覆盖
//! config.apply_env_overrides();
//!
//! // 验证配置
//! config.validate().expect("Invalid configuration");
//! ```
//!
//! ## 配置文件查找顺序
//!
//! 1. `./config.toml` - 当前目录的TOML配置
//! 2. `./config.json` - 当前目录的JSON配置
//! 3. `~/.config/game_engine/config.toml` - 用户配置目录
//! 4. 默认配置 - 如果以上都不存在
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 图形配置模块
pub mod graphics;

/// 性能配置模块
pub mod performance;

/// 音频配置模块
pub mod audio;

/// 输入配置模块
pub mod input;

/// 移动端配置模块（仅Android/iOS）
#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod mobile;

pub use audio::AudioConfig;
pub use graphics::GraphicsConfig;
pub use input::InputConfig;
pub use performance::PerformanceConfig;

/// 引擎配置错误
#[derive(Error, Debug)]
pub enum ConfigError {
    /// 文件读取错误
    #[error("Config file error: {0}")]
    FileError(#[from] std::io::Error),
    /// 解析错误
    #[error("Config parse error: {0}")]
    ParseError(String),
    /// 验证错误
    #[error("Config validation error: {0}")]
    ValidationError(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// 引擎主配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineConfig {
    /// 图形配置
    pub graphics: GraphicsConfig,

    /// 性能配置
    pub performance: PerformanceConfig,

    /// 音频配置
    pub audio: AudioConfig,

    /// 输入配置
    pub input: InputConfig,

    /// 日志配置
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl EngineConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从TOML文件加载配置
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let content = fs::read_to_string(path).map_err(ConfigError::FileError)?;
        Self::from_toml_str(&content)
    }

    /// 从TOML字符串解析配置
    pub fn from_toml_str(content: &str) -> ConfigResult<Self> {
        toml::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// 从JSON文件加载配置
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let content = fs::read_to_string(path).map_err(ConfigError::FileError)?;
        Self::from_json_str(&content)
    }

    /// 从JSON字符串解析配置
    pub fn from_json_str(content: &str) -> ConfigResult<Self> {
        serde_json::from_str(content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// 保存为TOML文件
    pub fn save_toml<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        fs::write(path, content).map_err(ConfigError::FileError)
    }

    /// 保存为JSON文件
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        fs::write(path, content).map_err(ConfigError::FileError)
    }

    /// 从环境变量覆盖配置
    pub fn apply_env_overrides(&mut self) {
        // 图形配置
        if let Ok(val) = env::var("ENGINE_GRAPHICS_WIDTH")
            && let Ok(width) = val.parse()
        {
            self.graphics.resolution.width = width;
        }
        if let Ok(val) = env::var("ENGINE_GRAPHICS_HEIGHT")
            && let Ok(height) = val.parse()
        {
            self.graphics.resolution.height = height;
        }
        if let Ok(val) = env::var("ENGINE_GRAPHICS_VSYNC")
            && let Ok(vsync) = val.parse::<bool>()
        {
            self.graphics.vsync = vsync;
        }

        // 性能配置
        if let Ok(val) = env::var("ENGINE_PERFORMANCE_TARGET_FPS")
            && let Ok(fps) = val.parse()
        {
            self.performance.target_fps = fps;
        }
        if let Ok(val) = env::var("ENGINE_PERFORMANCE_AUTO_OPTIMIZE")
            && let Ok(auto_optimize) = val.parse::<bool>()
        {
            self.performance.auto_optimize = auto_optimize;
        }

        // 音频配置
        if let Ok(val) = env::var("ENGINE_AUDIO_MASTER_VOLUME")
            && let Ok(volume) = val.parse()
        {
            self.audio.master_volume = volume;
        }
    }

    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        self.graphics.validate()?;
        self.performance.validate()?;
        self.audio.validate()?;
        self.input.validate()?;
        Ok(())
    }

    /// 自动查找并加载配置文件
    ///
    /// 按以下顺序查找：
    /// 1. ./config.toml
    /// 2. ./config.json
    /// 3. ~/.config/game_engine/config.toml
    /// 4. 使用默认配置
    pub fn load_or_default() -> Self {
        // 尝试当前目录的TOML
        if let Ok(config) = Self::from_toml_file("config.toml") {
            println!("Loaded config from config.toml");
            return config;
        }

        // 尝试当前目录的JSON
        if let Ok(config) = Self::from_json_file("config.json") {
            println!("Loaded config from config.json");
            return config;
        }

        // 尝试用户配置目录
        if let Some(home) = env::var_os("HOME") {
            let config_path =
                PathBuf::from(home).join(".config").join("game_engine").join("config.toml");

            if let Ok(config) = Self::from_toml_file(&config_path) {
                println!("Loaded config from {config_path:?}");
                return config;
            }
        }

        // 使用默认配置
        println!("Using default configuration");
        Self::default()
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: LogLevel,

    /// 是否输出到文件
    pub log_to_file: bool,

    /// 日志文件路径
    pub log_file_path: String,

    /// 是否输出到控制台
    pub log_to_console: bool,
}

use crate::impl_default;

impl_default!(LoggingConfig {
    level: LogLevel::Info,
    log_to_file: false,
    log_file_path: "game_engine.log".to_string(),
    log_to_console: true,
});

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// 跟踪
    Trace,
    /// 调试
    Debug,
    /// 信息
    Info,
    /// 警告
    Warn,
    /// 错误
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_toml_serialization() {
        let config = EngineConfig::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize config to TOML");
        let parsed: EngineConfig = toml::from_str(&toml_str).expect("Failed to parse TOML");
        assert_eq!(
            config.graphics.resolution.width,
            parsed.graphics.resolution.width
        );
    }

    #[test]
    fn test_json_serialization() {
        let config = EngineConfig::default();
        let json_str = serde_json::to_string(&config).expect("Failed to serialize config to JSON");
        let parsed: EngineConfig = serde_json::from_str(&json_str).expect("Failed to parse JSON");
        assert_eq!(
            config.graphics.resolution.width,
            parsed.graphics.resolution.width
        );
    }
}
