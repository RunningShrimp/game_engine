//! # Platform Detection and Capabilities System
//!
//! Extended platform detection system with compile-time and runtime capabilities checking.
//! Supports all console platforms with detailed hardware feature queries.

use crate::platform::console::{ConsoleConfig, ConsolePlatform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Comprehensive platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// Desktop platforms
    Windows,
    MacOS,
    Linux,

    /// Mobile platforms
    Android,
    IOS,
    HarmonyOS,

    /// Web platform
    Web,

    /// Console platforms
    NintendoSwitch,
    PlayStation5,
    PlayStation4,
    XboxSeries,
    XboxOne,

    /// Unknown platform
    Unknown,
}

impl Platform {
    /// Get human-readable platform name
    pub fn name(&self) -> &str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            Platform::Android => "Android",
            Platform::IOS => "iOS",
            Platform::HarmonyOS => "HarmonyOS",
            Platform::Web => "Web",
            Platform::NintendoSwitch => "Nintendo Switch",
            Platform::PlayStation5 => "PlayStation 5",
            Platform::PlayStation4 => "PlayStation 4",
            Platform::XboxSeries => "Xbox Series X/S",
            Platform::XboxOne => "Xbox One",
            Platform::Unknown => "Unknown",
        }
    }

    /// Check if this is a desktop platform
    pub fn is_desktop(&self) -> bool {
        matches!(self, Platform::Windows | Platform::MacOS | Platform::Linux)
    }

    /// Check if this is a mobile platform
    pub fn is_mobile(&self) -> bool {
        matches!(
            self,
            Platform::Android | Platform::IOS | Platform::HarmonyOS
        )
    }

    /// Check if this is a console platform
    pub fn is_console(&self) -> bool {
        matches!(
            self,
            Platform::NintendoSwitch
                | Platform::PlayStation5
                | Platform::PlayStation4
                | Platform::XboxSeries
                | Platform::XboxOne
        )
    }

    /// Convert to ConsolePlatform if applicable
    pub fn as_console_platform(&self) -> Option<ConsolePlatform> {
        match self {
            Platform::NintendoSwitch => Some(ConsolePlatform::NintendoSwitch),
            Platform::PlayStation5 => Some(ConsolePlatform::PlayStation5),
            Platform::PlayStation4 => Some(ConsolePlatform::PlayStation4),
            Platform::XboxSeries => Some(ConsolePlatform::XboxSeries),
            Platform::XboxOne => Some(ConsolePlatform::XboxOne),
            _ => None,
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Platform feature flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    // Graphics features
    RayTracing,
    HDR,
    VSync,
    AntiAliasing,
    ShadowMapping,
    Reflections,
    PostProcessing,

    // Controller features
    Vibration,
    MotionControls,
    Touchpad,
    Gyroscope,
    Accelerometer,

    // Network features
    OnlineMultiplayer,
    LanMultiplayer,
    CloudSave,
    Leaderboards,
    Achievements,

    // Audio features
    SpatialAudio,
    VoiceChat,

    // Platform features
    CrossPlatformPlay,
    RemotePlay,
    Streaming,
}

/// Hardware capability flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// CPU cores
    pub cpu_cores: usize,
    /// CPU frequency (MHz)
    pub cpu_frequency_mhz: u32,
    /// Available memory (MB)
    pub memory_mb: usize,
    /// GPU memory (MB)
    pub gpu_memory_mb: usize,
    /// Supported GPU features
    pub gpu_features: Vec<String>,
    /// Storage available (GB)
    pub storage_gb: usize,
    /// Supports SIMD
    pub supports_simd: bool,
    /// Supports 64-bit
    pub supports_64bit: bool,
}

impl Default for HardwareCapabilities {
    fn default() -> Self {
        Self {
            cpu_cores: 4,
            cpu_frequency_mhz: 2000,
            memory_mb: 4096,
            gpu_memory_mb: 1024,
            gpu_features: vec![],
            storage_gb: 50,
            supports_simd: false,
            supports_64bit: true,
        }
    }
}

/// Platform capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Platform type
    pub platform: Platform,
    /// Hardware capabilities
    pub hardware: HardwareCapabilities,
    /// Supported features
    pub supported_features: Vec<Feature>,
    /// Maximum texture size
    pub max_texture_size: u32,
    /// Supported texture formats
    pub texture_formats: Vec<String>,
    /// Maximum samplers
    pub max_samplers: u32,
    /// Maximum render targets
    pub max_render_targets: u32,
    /// Shader model
    pub shader_model: String,
}

impl PlatformCapabilities {
    /// Check if a feature is supported
    pub fn supports_feature(&self, feature: Feature) -> bool {
        self.supported_features.contains(&feature)
    }

    /// Check if multiple features are supported
    pub fn supports_features(&self, features: &[Feature]) -> bool {
        features.iter().all(|f| self.supports_feature(*f))
    }
}

/// Platform version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Build number
    pub build: Option<String>,
}

impl PlatformVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }

    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
}

impl fmt::Display for PlatformVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(build) = &self.build {
            write!(f, " ({})", build)?;
        }
        Ok(())
    }
}

/// Detect platform at compile time
pub fn detect_platform_compile_time() -> Platform {
    #[cfg(target_os = "windows")]
    {
        return Platform::Windows;
    }

    #[cfg(target_os = "macos")]
    {
        return Platform::MacOS;
    }

    #[cfg(target_os = "linux")]
    {
        return Platform::Linux;
    }

    #[cfg(target_os = "android")]
    {
        return Platform::Android;
    }

    #[cfg(target_os = "ios")]
    {
        return Platform::IOS;
    }

    #[cfg(target_arch = "wasm32")]
    {
        return Platform::Web;
    }

    #[cfg(all(
        feature = "harmonyos",
        any(target_os = "ohos", target_os = "harmonyos")
    ))]
    #[expect(unexpected_cfgs, reason = "ohos and harmonyos are custom target OS")]
    {
        return Platform::HarmonyOS;
    }

    #[cfg(all(
        any(target_os = "psp", target_os = "horizon"),
        not(feature = "mock-console")
    ))]
    {
        return Platform::NintendoSwitch;
    }

    #[cfg(all(target_os = "psx", not(feature = "mock-console")))]
    {
        return Platform::PlayStation4;
    }

    #[cfg(all(target_os = "ps5", not(feature = "mock-console")))]
    {
        return Platform::PlayStation5;
    }

    #[cfg(all(
        any(target_os = "xbox_one", target_os = "xbox"),
        not(feature = "mock-console")
    ))]
    {
        return Platform::XboxOne;
    }

    #[cfg(all(target_os = "xbox_series", not(feature = "mock-console")))]
    {
        return Platform::XboxSeries;
    }

    #[cfg(feature = "mock-console")]
    {
        // When using mock-console feature, default to desktop
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Platform::Unknown;
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios",
        target_arch = "wasm32"
    )))]
    {
        return Platform::Unknown;
    }
}

/// Detect platform at runtime (for mock/testing purposes)
pub fn detect_platform_runtime() -> Platform {
    // Check environment variables for testing overrides
    if let Ok(platform_str) = std::env::var("GAME_ENGINE_PLATFORM") {
        match platform_str.to_lowercase().as_str() {
            "switch" | "nintendo_switch" => return Platform::NintendoSwitch,
            "ps5" | "playstation5" => return Platform::PlayStation5,
            "ps4" | "playstation4" => return Platform::PlayStation4,
            "xbox_series" | "xboxseries" => return Platform::XboxSeries,
            "xbox_one" | "xboxone" => return Platform::XboxOne,
            "windows" => return Platform::Windows,
            "macos" => return Platform::MacOS,
            "linux" => return Platform::Linux,
            "android" => return Platform::Android,
            "ios" => return Platform::IOS,
            _ => {}
        }
    }

    // Fall back to compile-time detection
    detect_platform_compile_time()
}

/// Get capabilities for a specific platform
pub fn platform_capabilities(platform: Platform) -> PlatformCapabilities {
    let (hardware, features, max_texture, max_samplers, max_render_targets, shader_model) =
        match platform {
            Platform::NintendoSwitch => (
                HardwareCapabilities {
                    cpu_cores: 4,
                    cpu_frequency_mhz: 1020,
                    memory_mb: 4 * 1024,
                    gpu_memory_mb: 4 * 1024, // Shared memory
                    gpu_features: vec!["OpenGL_ES".into(), "Vulkan".into()],
                    storage_gb: 32,
                    supports_simd: true,
                    supports_64bit: false, // ARMv8 (32-bit mode)
                },
                vec![
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::OnlineMultiplayer,
                    Feature::CloudSave,
                    Feature::Leaderboards,
                    Feature::Achievements,
                    Feature::MotionControls,
                    Feature::Vibration,
                ],
                8192,
                16,
                4,
                "GLSL ES 3.2".into(),
            ),

            Platform::PlayStation5 => (
                HardwareCapabilities {
                    cpu_cores: 8,
                    cpu_frequency_mhz: 3500,
                    memory_mb: 16 * 1024,
                    gpu_memory_mb: 16 * 1024, // Shared memory
                    gpu_features: vec!["RayTracing".into(), "Vulkan".into(), "DirectX12".into()],
                    storage_gb: 825,
                    supports_simd: true,
                    supports_64bit: true,
                },
                vec![
                    Feature::RayTracing,
                    Feature::HDR,
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::Reflections,
                    Feature::PostProcessing,
                    Feature::OnlineMultiplayer,
                    Feature::CloudSave,
                    Feature::Leaderboards,
                    Feature::Achievements,
                    Feature::SpatialAudio,
                    Feature::VoiceChat,
                    Feature::Vibration,
                    Feature::Touchpad,
                    Feature::CrossPlatformPlay,
                ],
                16384,
                64,
                8,
                "SPIR-V 1.6".into(),
            ),

            Platform::PlayStation4 => (
                HardwareCapabilities {
                    cpu_cores: 8,
                    cpu_frequency_mhz: 1600,
                    memory_mb: 8 * 1024,
                    gpu_memory_mb: 8 * 1024, // Shared memory
                    gpu_features: vec!["OpenGL".into(), "Vulkan".into()],
                    storage_gb: 500,
                    supports_simd: true,
                    supports_64bit: true,
                },
                vec![
                    Feature::HDR,
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::Reflections,
                    Feature::PostProcessing,
                    Feature::OnlineMultiplayer,
                    Feature::CloudSave,
                    Feature::Leaderboards,
                    Feature::Achievements,
                    Feature::SpatialAudio,
                    Feature::Vibration,
                    Feature::Touchpad,
                ],
                16384,
                32,
                8,
                "GNMX".into(),
            ),

            Platform::XboxSeries => (
                HardwareCapabilities {
                    cpu_cores: 8,
                    cpu_frequency_mhz: 3800,
                    memory_mb: 16 * 1024,
                    gpu_memory_mb: 16 * 1024, // Shared memory
                    gpu_features: vec!["RayTracing".into(), "DirectX12".into(), "Vulkan".into()],
                    storage_gb: 1000,
                    supports_simd: true,
                    supports_64bit: true,
                },
                vec![
                    Feature::RayTracing,
                    Feature::HDR,
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::Reflections,
                    Feature::PostProcessing,
                    Feature::OnlineMultiplayer,
                    Feature::LanMultiplayer,
                    Feature::CloudSave,
                    Feature::Leaderboards,
                    Feature::Achievements,
                    Feature::SpatialAudio,
                    Feature::VoiceChat,
                    Feature::Vibration,
                    Feature::CrossPlatformPlay,
                    Feature::RemotePlay,
                ],
                16384,
                64,
                8,
                "DXIL".into(),
            ),

            Platform::XboxOne => (
                HardwareCapabilities {
                    cpu_cores: 8,
                    cpu_frequency_mhz: 1700,
                    memory_mb: 8 * 1024,
                    gpu_memory_mb: 8 * 1024, // Shared memory
                    gpu_features: vec!["DirectX11".into(), "DirectX12".into()],
                    storage_gb: 500,
                    supports_simd: true,
                    supports_64bit: true,
                },
                vec![
                    Feature::HDR,
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::Reflections,
                    Feature::PostProcessing,
                    Feature::OnlineMultiplayer,
                    Feature::LanMultiplayer,
                    Feature::CloudSave,
                    Feature::Leaderboards,
                    Feature::Achievements,
                    Feature::SpatialAudio,
                    Feature::Vibration,
                    Feature::CrossPlatformPlay,
                ],
                16384,
                32,
                8,
                "DXBC".into(),
            ),

            _ => (
                HardwareCapabilities::default(),
                vec![
                    Feature::VSync,
                    Feature::AntiAliasing,
                    Feature::ShadowMapping,
                    Feature::OnlineMultiplayer,
                ],
                8192,
                16,
                4,
                "Unknown".into(),
            ),
        };

    PlatformCapabilities {
        platform,
        hardware,
        supported_features: features,
        max_texture_size: max_texture,
        texture_formats: vec![
            "RGBA8".into(),
            "RGB8".into(),
            "RG8".into(),
            "R8".into(),
            "RGBA16F".into(),
            "RGBA32F".into(),
            "BC7".into(),
            "ETC2".into(),
        ],
        max_samplers,
        max_render_targets,
        shader_model,
    }
}

/// Check if a feature is supported on the current platform
pub fn is_feature_supported(feature: Feature) -> bool {
    let platform = detect_platform_runtime();
    let capabilities = platform_capabilities(platform);
    capabilities.supports_feature(feature)
}

/// Get current platform information
pub fn current_platform_info() -> (Platform, PlatformCapabilities) {
    let platform = detect_platform_runtime();
    let capabilities = platform_capabilities(platform);
    (platform, capabilities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform_compile_time();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_platform_classification() {
        assert!(Platform::Windows.is_desktop());
        assert!(Platform::MacOS.is_desktop());
        assert!(Platform::Linux.is_desktop());

        assert!(Platform::Android.is_mobile());
        assert!(Platform::IOS.is_mobile());
        assert!(Platform::HarmonyOS.is_mobile());

        assert!(Platform::NintendoSwitch.is_console());
        assert!(Platform::PlayStation5.is_console());
        assert!(Platform::PlayStation4.is_console());
        assert!(Platform::XboxSeries.is_console());
        assert!(Platform::XboxOne.is_console());
    }

    #[test]
    fn test_console_platform_conversion() {
        assert_eq!(
            Platform::NintendoSwitch.as_console_platform(),
            Some(ConsolePlatform::NintendoSwitch)
        );
        assert_eq!(
            Platform::PlayStation5.as_console_platform(),
            Some(ConsolePlatform::PlayStation5)
        );
        assert_eq!(Platform::Windows.as_console_platform(), None);
    }

    #[test]
    fn test_platform_capabilities() {
        let ps5_caps = platform_capabilities(Platform::PlayStation5);
        assert_eq!(ps5_caps.hardware.cpu_cores, 8);
        assert_eq!(ps5_caps.hardware.memory_mb, 16 * 1024);
        assert!(ps5_caps.supports_feature(Feature::RayTracing));
        assert!(ps5_caps.supports_feature(Feature::HDR));

        let switch_caps = platform_capabilities(Platform::NintendoSwitch);
        assert_eq!(switch_caps.hardware.cpu_cores, 4);
        assert_eq!(switch_caps.hardware.memory_mb, 4 * 1024);
        assert!(!switch_caps.supports_feature(Feature::RayTracing));
        assert!(!switch_caps.supports_feature(Feature::HDR));
        assert!(switch_caps.supports_feature(Feature::MotionControls));
    }

    #[test]
    fn test_feature_support() {
        let caps = platform_capabilities(Platform::PlayStation5);
        assert!(caps.supports_feature(Feature::RayTracing));
        assert!(caps.supports_features(&[Feature::RayTracing, Feature::HDR, Feature::VSync]));
        assert!(!caps.supports_features(&[
            Feature::RayTracing,
            Feature::Gyroscope // PS5 doesn't have gyroscope
        ]));
    }

    #[test]
    fn test_platform_version() {
        let version = PlatformVersion::new(1, 0, 0);
        assert_eq!(format!("{}", version), "1.0.0");

        let version_with_build = version.with_build("12345".to_string());
        assert_eq!(format!("{}", version_with_build), "1.0.0 (12345)");
    }
}
