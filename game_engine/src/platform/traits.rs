//! # 平台抽象trait
//!
//! 提供统一的平台抽象，减少条件编译代码。
//!
//! ## 功能特性
//!
//! - **Platform trait**: 统一的平台接口
//! - **策略模式**: 运行时平台选择而非编译时
//! - **可测试性**: 易于mock和测试

use std::path::{Path, PathBuf};
use std::any::Any;

/// 平台类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformType {
    Windows,
    MacOS,
    Linux,
    IOS,
    Android,
    Web,
    Unknown,
}

impl PlatformType {
    /// 获取当前平台类型
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return PlatformType::Windows;

        #[cfg(target_os = "macos")]
        return PlatformType::MacOS;

        #[cfg(target_os = "linux")]
        return PlatformType::Linux;

        #[cfg(target_os = "ios")]
        return PlatformType::IOS;

        #[cfg(target_os = "android")]
        return PlatformType::Android;

        #[cfg(target_arch = "wasm32")]
        return PlatformType::Web;

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "ios",
            target_os = "android",
            target_arch = "wasm32"
        )))]
        return PlatformType::Unknown;
    }

    /// 是否为移动平台
    pub fn is_mobile(&self) -> bool {
        matches!(self, PlatformType::IOS | PlatformType::Android)
    }

    /// 是否为桌面平台
    pub fn is_desktop(&self) -> bool {
        matches!(self, PlatformType::Windows | PlatformType::MacOS | PlatformType::Linux)
    }

    /// 是否为Web平台
    pub fn is_web(&self) -> bool {
        matches!(self, PlatformType::Web)
    }
}

/// 平台抽象trait
///
/// 提供跨平台统一接口，减少条件编译代码。
pub trait Platform: Any + Send + Sync {
    /// 获取平台类型
    fn platform_type(&self) -> PlatformType;

    /// 获取平台名称
    fn name(&self) -> &str {
        match self.platform_type() {
            PlatformType::Windows => "Windows",
            PlatformType::MacOS => "macOS",
            PlatformType::Linux => "Linux",
            PlatformType::IOS => "iOS",
            PlatformType::Android => "Android",
            PlatformType::Web => "Web",
            PlatformType::Unknown => "Unknown",
        }
    }

    /// 平台特定的路径分隔符
    fn path_separator(&self) -> char {
        if matches!(self.platform_type(), PlatformType::Windows) {
            '\\'
        } else {
            '/'
        }
    }

    /// 规范化路径
    fn normalize_path(&self, path: &Path) -> PathBuf {
        let sep = self.path_separator();
        let path_str = path.to_string_lossy().replace('/', &sep.to_string()).replace('\\', &sep.to_string());
        PathBuf::from(path_str)
    }

    /// 获取应用程序数据目录
    fn app_data_dir(&self) -> Result<PathBuf, String>;

    /// 获取缓存目录
    fn cache_dir(&self) -> Result<PathBuf, String>;

    /// 获取临时目录
    fn temp_dir(&self) -> Result<PathBuf, String>;

    /// 是否支持触摸
    fn supports_touch(&self) -> bool {
        self.platform_type().is_mobile()
    }

    /// 是否支持键盘
    fn supports_keyboard(&self) -> bool {
        !self.platform_type().is_mobile()
    }

    /// 是否支持游戏手柄
    fn supports_gamepad(&self) -> bool {
        true
    }

    /// 将trait对象转为Any（用于downcast）
    fn as_any(&self) -> &dyn Any;
}

/// Windows平台实现
#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsPlatform;

#[cfg(target_os = "windows")]
impl Platform for WindowsPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Windows
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .map_err(|e| format!("Failed to get APPDATA: {}", e))
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        std::env::var("TEMP")
            .map(PathBuf::from)
            .map_err(|e| format!("Failed to get TEMP: {}", e))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// macOS平台实现
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MacOSPlatform;

#[cfg(target_os = "macos")]
impl Platform for MacOSPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::MacOS
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .map_err(|e| format!("Failed to get HOME: {}", e))?;
        Ok(PathBuf::from(home).join("Library/Application Support"))
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .map_err(|e| format!("Failed to get HOME: {}", e))?;
        Ok(PathBuf::from(home).join("Library/Caches"))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Linux平台实现
#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct LinuxPlatform;

#[cfg(target_os = "linux")]
impl Platform for LinuxPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Linux
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .map_err(|e| format!("Failed to get HOME: {}", e))?;
        Ok(PathBuf::from(home).join(".local/share"))
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .map_err(|e| format!("Failed to get HOME: {}", e))?;
        Ok(PathBuf::from(home).join(".cache"))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// iOS平台实现
#[cfg(target_os = "ios")]
#[derive(Debug)]
pub struct IOSPlatform;

#[cfg(target_os = "ios")]
impl Platform for IOSPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::IOS
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        // iOS使用沙盒，这里简化处理
        Ok(PathBuf::from("./"))
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("./cache"))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("./tmp"))
    }

    fn supports_touch(&self) -> bool {
        true
    }

    fn supports_keyboard(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Android平台实现
#[cfg(target_os = "android")]
#[derive(Debug)]
pub struct AndroidPlatform;

#[cfg(target_os = "android")]
impl Platform for AndroidPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Android
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        // Android使用内部存储
        Ok(PathBuf::from("/data/data/com.example.game"))
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("/data/data/com.example.game/cache"))
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("/data/data/com.example.game/cache"))
    }

    fn supports_touch(&self) -> bool {
        true
    }

    fn supports_keyboard(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 获取当前平台实例
pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    return Box::new(WindowsPlatform);

    #[cfg(target_os = "macos")]
    return Box::new(MacOSPlatform);

    #[cfg(target_os = "linux")]
    return Box::new(LinuxPlatform);

    #[cfg(target_os = "ios")]
    return Box::new(IOSPlatform);

    #[cfg(target_os = "android")]
    return Box::new(AndroidPlatform);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "ios",
        target_os = "android"
    )))]
    return Box::new(UnknownPlatform);
}

/// 未知平台实现
#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
)))]
#[derive(Debug)]
pub struct UnknownPlatform;

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
)))]
impl Platform for UnknownPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Unknown
    }

    fn app_data_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn cache_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        std::env::temp_dir().into_ok_or_else(|| "Failed to get temp dir".to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_type() {
        let platform_type = PlatformType::current();
        println!("Current platform: {:?}", platform_type);

        #[cfg(target_os = "macos")]
        assert_eq!(platform_type, PlatformType::MacOS);
    }

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        println!("Platform name: {}", platform.name());

        let app_data = platform.app_data_dir();
        println!("App data dir: {:?}", app_data);
    }

    #[test]
    fn test_platform_capabilities() {
        let platform = current_platform();

        // 检查能力
        let has_touch = platform.supports_touch();
        let has_keyboard = platform.supports_keyboard();

        #[cfg(target_os = "ios")]
        {
            assert!(has_touch);
            assert!(!has_keyboard);
        }

        #[cfg(target_os = "macos")]
        {
            assert!(!has_touch);
            assert!(has_keyboard);
        }
    }
}
