//! # Mobile Platform Module
//!
//! Provides mobile platform-specific functionality for iOS and Android.

pub mod config;
pub mod input;
pub mod services;

// JNI模块仅在Android平台可用
#[cfg(target_os = "android")]
pub mod jni;

// iOS FFI模块仅在iOS平台可用
#[cfg(target_os = "ios")]
pub mod ios_ffi;

pub use config::MobileConfig;

#[cfg(target_os = "ios")]
pub mod ios_services;

#[cfg(target_os = "android")]
pub mod android_services;

// Re-export commonly used types
pub use services::{
    Achievement, GameCenter, GooglePlayGames, Leaderboard, Notification, NotificationPlatform,
    PlayerInfo, PushNotificationService, ServiceError,
};

#[cfg(target_os = "ios")]
pub use ios_services::IOSPlatformServices;

#[cfg(target_os = "android")]
pub use android_services::AndroidPlatformServices;
