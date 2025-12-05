/// 统一配置系统
///
/// 提供TOML/JSON配置文件、环境变量和运行时动态调整
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod graphics;
pub mod performance;

pub mod audio;
pub mod input;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            performance: PerformanceConfig::default(),
            audio: AudioConfig::default(),
            input: InputConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
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
        if let Ok(val) = env::var("ENGINE_GRAPHICS_WIDTH") {
            if let Ok(width) = val.parse() {
                self.graphics.resolution.width = width;
            }
        }
        if let Ok(val) = env::var("ENGINE_GRAPHICS_HEIGHT") {
            if let Ok(height) = val.parse() {
                self.graphics.resolution.height = height;
            }
        }
        if let Ok(val) = env::var("ENGINE_GRAPHICS_VSYNC") {
            self.graphics.vsync = val.parse().unwrap_or(self.graphics.vsync);
        }

        // 性能配置
        if let Ok(val) = env::var("ENGINE_PERFORMANCE_TARGET_FPS") {
            if let Ok(fps) = val.parse() {
                self.performance.target_fps = fps;
            }
        }
        if let Ok(val) = env::var("ENGINE_PERFORMANCE_AUTO_OPTIMIZE") {
            self.performance.auto_optimize = val.parse().unwrap_or(self.performance.auto_optimize);
        }

        // 音频配置
        if let Ok(val) = env::var("ENGINE_AUDIO_MASTER_VOLUME") {
            if let Ok(volume) = val.parse() {
                self.audio.master_volume = volume;
            }
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
            let config_path = PathBuf::from(home)
                .join(".config")
                .join("game_engine")
                .join("config.toml");

            if let Ok(config) = Self::from_toml_file(&config_path) {
                println!("Loaded config from {:?}", config_path);
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
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: EngineConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config.graphics.resolution.width,
            parsed.graphics.resolution.width
        );
    }

    #[test]
    fn test_json_serialization() {
        let config = EngineConfig::default();
        let json_str = serde_json::to_string(&config).unwrap();
        let parsed: EngineConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(
            config.graphics.resolution.width,
            parsed.graphics.resolution.width
        );
    }
}
