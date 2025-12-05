//! 统一错误处理模块
//!
//! 提供引擎范围内的统一错误类型定义
//!
//! ## 错误类型分层
//!
//! - **基础设施层错误** (`core::error`): 用于基础设施层的错误（初始化、设备等）
//! - **领域层错误** (`domain::errors`): 用于领域逻辑的错误（业务规则、验证等）
//!
//! `EngineError` 可以同时处理基础设施层和领域层的错误。

use thiserror::Error;

/// 引擎核心错误类型
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Initialization error: {0}")]
    Init(String),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("Asset error: {0}")]
    Asset(#[from] AssetError),

    #[error("Physics error: {0}")]
    Physics(#[from] PhysicsError),

    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Script error: {0}")]
    Script(#[from] ScriptError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Window creation failed: {0}")]
    Window(String),

    #[error("Renderer initialization failed: {0}")]
    RenderInit(#[from] wgpu::Error),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("General error: {0}")]
    General(String),
}

/// 渲染系统错误
#[derive(Error, Debug, Clone)]
pub enum RenderError {
    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to request adapter: no compatible GPU found")]
    NoAdapter,

    #[error("Failed to request device: {0}")]
    DeviceRequest(String),

    #[error("Failed to create shader: {0}")]
    ShaderCompilation(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to create texture: {0}")]
    TextureCreation(String),

    #[error("Surface error: {0}")]
    Surface(String),

    #[error("Frame submission error: {0}")]
    FrameSubmission(String),

    #[error("Invalid render state: {0}")]
    InvalidState(String),
}

// Note: wgpu::CreateSurfaceError does not exist in wgpu 0.20+
// Surface creation errors are now handled through wgpu::Error

/// 资源管理错误
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {path}")]
    NotFound { path: String },

    #[error("Failed to load asset: {path}, reason: {reason}")]
    LoadFailed { path: String, reason: String },

    #[error("Invalid asset format: {path}, expected: {expected}")]
    InvalidFormat { path: String, expected: String },

    #[error("Asset decode error: {0}")]
    Decode(String),

    #[error("Asset dependency missing: {0}")]
    DependencyMissing(String),
}

/// 物理系统错误（基础设施层）
///
/// 注意：领域层有更详细的 `domain::errors::PhysicsError`。
/// 此类型用于基础设施层的物理系统初始化错误。
#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("Invalid rigid body handle")]
    InvalidRigidBody,

    #[error("Invalid collider handle")]
    InvalidCollider,

    #[error("Physics world not initialized")]
    NotInitialized,

    #[error("Invalid physics configuration: {0}")]
    InvalidConfig(String),
}

/// 音频系统错误（基础设施层）
///
/// 注意：领域层有更详细的 `domain::errors::AudioError`。
/// 此类型用于基础设施层的音频系统初始化错误。
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio device")]
    DeviceInit,

    #[error("Audio file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to decode audio: {0}")]
    DecodeFailed(String),

    #[error("Playback error: {0}")]
    Playback(String),

    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
}

/// 脚本系统错误
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Script compilation error: {0}")]
    Compilation(String),

    #[error("Script runtime error: {0}")]
    Runtime(String),

    #[error("Script not found: {0}")]
    NotFound(String),

    #[error("Invalid script binding: {0}")]
    InvalidBinding(String),

    #[error("Script timeout after {0}ms")]
    Timeout(u64),
}

/// 平台层错误
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("Input device error: {0}")]
    InputDevice(String),

    #[error("Filesystem error: {0}")]
    Filesystem(String),

    #[error("Platform not supported: {0}")]
    NotSupported(String),
}

/// 引擎结果类型别名
pub type EngineResult<T> = Result<T, EngineError>;
pub type RenderResult<T> = Result<T, RenderError>;
pub type AssetResult<T> = Result<T, AssetError>;
pub type PhysicsResult<T> = Result<T, PhysicsError>;
pub type AudioResult<T> = Result<T, AudioError>;
pub type ScriptResult<T> = Result<T, ScriptError>;
pub type PlatformResult<T> = Result<T, PlatformError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let asset_err = AssetError::NotFound {
            path: "test.png".to_string(),
        };
        let engine_err: EngineError = asset_err.into();
        assert!(matches!(engine_err, EngineError::Asset(_)));
    }

    #[test]
    fn test_error_display() {
        let err = RenderError::NoAdapter;
        assert_eq!(
            err.to_string(),
            "Failed to request adapter: no compatible GPU found"
        );
    }
}
