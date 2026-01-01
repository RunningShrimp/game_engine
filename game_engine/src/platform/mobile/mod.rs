//! # Mobile Platform Module
//!
//! Provides mobile platform-specific functionality for iOS and Android.

pub mod config;
pub mod input;
pub mod performance;
pub mod services;

// JNI模块仅在Android平台可用
#[cfg(target_os = "android")]
pub mod jni;

// iOS FFI模块仅在iOS平台可用
#[cfg(target_os = "ios")]
pub mod ios_ffi;

// 推送通知FFI模块
pub mod push_ffi;

// 应用内购买FFI模块
pub mod in_app_purchase_ffi;

pub use config::MobileConfig;

#[cfg(target_os = "ios")]
pub mod ios_services;

#[cfg(target_os = "android")]
pub mod android_services;

// Re-export commonly used types
pub use performance::{
    AdaptiveQualityController, BatteryState, DeviceCapabilities, MemoryStats,
    MobilePerformanceOptimizer, PerformanceConfig, PerformanceError, PerformanceMode,
    PerformanceOptimizations, PerformanceSnapshot, QualityLevel, ThermalState,
};
pub use services::{
    Achievement, GameCenter, GooglePlayGames, InAppPurchaseService, Leaderboard, Notification,
    NotificationPlatform, PlayerInfo, PushNotificationService, ServiceError,
};

#[cfg(target_os = "ios")]
pub use ios_services::IOSPlatformServices;

#[cfg(target_os = "android")]
pub use android_services::AndroidPlatformServices;
