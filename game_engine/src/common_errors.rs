//  通用错误处理模块
//
//  提供统一的错误类型定义和处理模式，整合基础设施层和领域层错误。
//
//  ## 错误类型层次结构
//
//  ```text
//  GameEngineError (顶层错误类型)
//  ├── Infrastructure (基础设施层错误)
//  │   ├── Init (初始化错误)
//  │   ├── Render (渲染错误)
//  │   ├── Asset (资源错误)
//  │   ├── Physics (物理错误)
//  │   ├── Audio (音频错误)
//  │   ├── Script (脚本错误)
//  │   ├── Platform (平台错误)
//  │   ├── Io (IO错误)
//  │   └── General (通用错误)
//  └── Domain (领域层错误)
//      ├── Audio (音频领域错误)
//      ├── Physics (物理领域错误)
//      ├── Scene (场景领域错误)
//      └── General (通用领域错误)
//  ```

use thiserror::Error;

/// 游戏引擎统一错误类型
#[derive(Error, Debug)]
pub enum GameEngineError {
    /// 基础设施层错误
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),

    /// 领域层错误
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
}

/// 基础设施层错误类型
#[derive(Error, Debug)]
pub enum InfrastructureError {
    /// 初始化错误
    #[error("Initialization error: {0}")]
    Init(String),

    /// 渲染错误
    #[error("Render error: {0}")]
    Render(#[from] CommonRenderError),

    /// 资源错误
    #[error("Asset error: {0}")]
    Asset(#[from] CommonAssetError),

    /// 物理错误
    #[error("Physics error: {0}")]
    Physics(#[from] CommonPhysicsError),

    /// 音频错误
    #[error("Audio error: {0}")]
    Audio(#[from] CommonAudioError),

    /// 脚本错误
    #[error("Script error: {0}")]
    Script(#[from] CommonScriptError),

    /// 平台错误
    #[error("Platform error: {0}")]
    Platform(#[from] CommonPlatformError),

    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 窗口错误
    #[error("Window error: {0}")]
    Window(String),

    /// 渲染初始化错误
    #[error("Render initialization error: {0}")]
    RenderInit(#[from] wgpu::Error),

    /// 事件循环错误
    #[error("Event loop error: {0}")]
    EventLoop(String),

    /// 通用基础设施错误
    #[error("General infrastructure error: {0}")]
    General(String),
}

/// 领域层错误类型
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    /// 音频领域错误
    #[error("Audio domain error: {0}")]
    Audio(#[from] AudioDomainError),

    /// 物理领域错误
    #[error("Physics domain error: {0}")]
    Physics(#[from] PhysicsDomainError),

    /// 场景领域错误
    #[error("Scene domain error: {0}")]
    Scene(#[from] SceneDomainError),

    /// 通用领域错误
    #[error("Domain error: {0}")]
    General(String),
}

/// 渲染系统错误
#[derive(Error, Debug, Clone)]
pub enum CommonRenderError {
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

/// 资源管理错误
#[derive(Error, Debug)]
pub enum CommonAssetError {
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
#[derive(Error, Debug)]
pub enum CommonPhysicsError {
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
#[derive(Error, Debug)]
pub enum CommonAudioError {
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
pub enum CommonScriptError {
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
pub enum CommonPlatformError {
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

/// 音频领域错误
#[derive(Error, Debug, Clone)]
pub enum AudioDomainError {
    /// 音频源未找到
    #[error("Audio source not found: {0}")]
    SourceNotFound(String),

    /// 音频播放失败
    #[error("Audio playback failed: {0}")]
    PlaybackFailed(String),

    /// 无效音频格式
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),

    /// 音频设备错误
    #[error("Audio device error: {0}")]
    DeviceError(String),

    /// 音量超出范围
    #[error("Invalid volume: {0}")]
    InvalidVolume(f32),
}

/// 物理领域错误
#[derive(Error, Debug, Clone)]
pub enum PhysicsDomainError {
    /// 刚体未找到
    #[error("Physics body not found: {0}")]
    BodyNotFound(String),

    /// 碰撞体未找到
    #[error("Collider not found: {0}")]
    ColliderNotFound(String),

    /// 无效物理参数
    #[error("Invalid physics parameter: {0}")]
    InvalidParameter(String),

    /// 物理世界未初始化
    #[error("Physics world not initialized")]
    WorldNotInitialized,

    /// 关节创建失败
    #[error("Joint creation failed: {0}")]
    JointCreationFailed(String),

    /// 无效形状
    #[error("Invalid shape: {0}")]
    InvalidShape(String),

    /// 形状创建错误
    #[error("Shape creation error: {0}")]
    ShapeCreationError(String),

    /// 锁错误
    #[error("Lock error: {0}")]
    LockError(String),
}

/// 场景领域错误
#[derive(Error, Debug, Clone)]
pub enum SceneDomainError {
    /// 无效场景名称
    #[error("Invalid scene name: {0}")]
    InvalidName(String),

    /// 实体未找到
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// 场景未找到
    #[error("Scene not found: {0}")]
    SceneNotFound(String),

    /// 组件未找到
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// 序列化失败
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    /// 反序列化失败
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 使用默认值
    UseDefault,
    /// 跳过操作
    Skip,
    /// 记录错误并继续
    LogAndContinue,
    /// 抛出错误
    Fail,
}

/// 补偿操作
#[derive(Debug, Clone)]
pub struct CompensationAction {
    /// 操作ID
    pub id: String,
    /// 操作类型
    pub action_type: String,
    /// 补偿数据
    pub data: serde_json::Value,
}

impl CompensationAction {
    pub fn new(
        id: impl Into<String>,
        action_type: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            action_type: action_type.into(),
            data,
        }
    }
}

/// 统一的结果类型别名
pub type GameEngineResult<T> = Result<T, GameEngineError>;
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;
pub type DomainResult<T> = Result<T, DomainError>;
pub type RenderResult<T> = Result<T, CommonRenderError>;
pub type AssetResult<T> = Result<T, CommonAssetError>;
pub type PhysicsResult<T> = Result<T, CommonPhysicsError>;
pub type AudioResult<T> = Result<T, CommonAudioError>;
pub type ScriptResult<T> = Result<T, CommonScriptError>;
pub type PlatformResult<T> = Result<T, CommonPlatformError>;

/// 便捷的错误转换函数
impl GameEngineError {
    /// 将基础设施层错误转换为顶级错误
    pub fn infrastructure<E>(error: E) -> Self
    where
        InfrastructureError: From<E>,
    {
        Self::Infrastructure(error.into())
    }

    /// 将领域层错误转换为顶级错误
    pub fn domain<E>(error: E) -> Self
    where
        DomainError: From<E>,
    {
        Self::Domain(error.into())
    }
}

// From implementations for legacy domain error types
impl From<crate::domain::errors::AudioError> for AudioDomainError {
    fn from(error: crate::domain::errors::AudioError) -> Self {
        match error {
            crate::domain::errors::AudioError::SourceNotFound(msg) => Self::SourceNotFound(msg),
            crate::domain::errors::AudioError::PlaybackFailed(msg) => Self::PlaybackFailed(msg),
            crate::domain::errors::AudioError::InvalidFormat(msg) => Self::InvalidFormat(msg),
            crate::domain::errors::AudioError::DeviceError(msg) => Self::DeviceError(msg),
            crate::domain::errors::AudioError::InvalidVolume(vol) => Self::InvalidVolume(vol),
        }
    }
}

impl From<crate::domain::errors::PhysicsError> for PhysicsDomainError {
    fn from(error: crate::domain::errors::PhysicsError) -> Self {
        match error {
            crate::domain::errors::PhysicsError::BodyNotFound(msg) => Self::BodyNotFound(msg),
            crate::domain::errors::PhysicsError::ColliderNotFound(msg) => {
                Self::ColliderNotFound(msg)
            }
            crate::domain::errors::PhysicsError::InvalidParameter(msg) => {
                Self::InvalidParameter(msg)
            }
            crate::domain::errors::PhysicsError::WorldNotInitialized => Self::WorldNotInitialized,
            crate::domain::errors::PhysicsError::JointCreationFailed(msg) => {
                Self::JointCreationFailed(msg)
            }
            crate::domain::errors::PhysicsError::InvalidShape(msg) => Self::InvalidShape(msg),
            crate::domain::errors::PhysicsError::ShapeCreationError(msg) => {
                Self::ShapeCreationError(msg)
            }
            crate::domain::errors::PhysicsError::LockError(msg) => Self::LockError(msg),
        }
    }
}

impl From<crate::domain::errors::SceneError> for SceneDomainError {
    fn from(error: crate::domain::errors::SceneError) -> Self {
        match error {
            crate::domain::errors::SceneError::EntityNotFound(msg) => Self::EntityNotFound(msg),
            crate::domain::errors::SceneError::SceneNotFound(msg) => Self::SceneNotFound(msg),
            crate::domain::errors::SceneError::ComponentNotFound(msg) => {
                Self::ComponentNotFound(msg)
            }
            crate::domain::errors::SceneError::SerializationFailed(msg) => {
                Self::SerializationFailed(msg)
            }
            crate::domain::errors::SceneError::DeserializationFailed(msg) => {
                Self::DeserializationFailed(msg)
            }
        }
    }
}

// From implementations for legacy error types
impl From<crate::error::RenderError> for CommonRenderError {
    fn from(error: crate::error::RenderError) -> Self {
        match error {
            crate::error::RenderError::SurfaceCreation { message, .. } => {
                Self::SurfaceCreation(message)
            }
            crate::error::RenderError::Adapter { .. } => Self::NoAdapter,
            crate::error::RenderError::DeviceCreation { message, .. } => {
                Self::DeviceRequest(message)
            }
            crate::error::RenderError::ShaderCompilation { message, .. } => {
                Self::ShaderCompilation(message)
            }
            crate::error::RenderError::PipelineCreation { message, .. } => {
                Self::PipelineCreation(message)
            }
            crate::error::RenderError::TextureCreation { message, .. } => {
                Self::TextureCreation(message)
            }
            crate::error::RenderError::FrameSubmission { message, .. } => {
                Self::FrameSubmission(message)
            }
            crate::error::RenderError::InvalidState { message, .. } => Self::InvalidState(message),
            // Handle other variants that might exist
            _ => Self::InvalidState("Unknown render error".to_string()),
        }
    }
}

impl From<crate::error::ResourceError> for CommonAssetError {
    fn from(error: crate::error::ResourceError) -> Self {
        match error {
            crate::error::ResourceError::NotFound { path, .. } => Self::NotFound { path },
            // For other resource errors, convert to generic load failed
            _ => Self::LoadFailed {
                path: "unknown".to_string(),
                reason: format!("Resource error: {:?}", error),
            },
        }
    }
}

impl From<crate::error::PhysicsError> for CommonPhysicsError {
    fn from(error: crate::error::PhysicsError) -> Self {
        // 将具体物理错误压缩为统一的配置/初始化错误描述，避免枚举差异导致编译失败
        Self::InvalidConfig(format!("{:?}", error))
    }
}

impl From<crate::error::AudioError> for CommonAudioError {
    fn from(error: crate::error::AudioError) -> Self {
        // 将具体音频错误压缩为统一的播放错误描述，避免枚举差异导致编译失败
        Self::Playback(format!("{:?}", error))
    }
}

impl From<crate::error::ScriptError> for CommonScriptError {
    fn from(error: crate::error::ScriptError) -> Self {
        match error {
            crate::error::ScriptError::Compilation(msg) => Self::Compilation(msg),
            crate::error::ScriptError::Runtime(msg) => Self::Runtime(msg),
            crate::error::ScriptError::NotFound(path) => Self::NotFound(path),
            crate::error::ScriptError::InvalidBinding(msg) => Self::InvalidBinding(msg),
            crate::error::ScriptError::Timeout(ms) => Self::Timeout(ms),
        }
    }
}

impl From<crate::error::PlatformError> for CommonPlatformError {
    fn from(error: crate::error::PlatformError) -> Self {
        match error {
            crate::error::PlatformError::WindowCreation(msg) => Self::WindowCreation(msg),
            crate::error::PlatformError::EventLoop(msg) => Self::EventLoop(msg),
            crate::error::PlatformError::InputDevice(msg) => Self::InputDevice(msg),
            crate::error::PlatformError::Filesystem(msg) => Self::Filesystem(msg),
            crate::error::PlatformError::NotSupported(msg) => Self::NotSupported(msg),
        }
    }
}

/// 从现有错误类型的转换
impl From<crate::error::EngineError> for GameEngineError {
    fn from(error: crate::error::EngineError) -> Self {
        // 将引擎错误统一映射为基础设施层的通用错误，避免枚举差异导致编译问题
        Self::Infrastructure(InfrastructureError::General(format!("{:?}", error)))
    }
}

impl From<crate::domain::errors::DomainError> for GameEngineError {
    fn from(error: crate::domain::errors::DomainError) -> Self {
        match error {
            crate::domain::errors::DomainError::Audio(e) => {
                Self::Domain(DomainError::Audio(AudioDomainError::from(e)))
            }
            crate::domain::errors::DomainError::Physics(e) => {
                Self::Domain(DomainError::Physics(PhysicsDomainError::from(e)))
            }
            crate::domain::errors::DomainError::Scene(e) => {
                Self::Domain(DomainError::Scene(SceneDomainError::from(e)))
            }
            crate::domain::errors::DomainError::General(msg) => {
                Self::Domain(DomainError::General(msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_hierarchy() {
        // 测试基础设施层错误转换
        let render_err = CommonRenderError::NoAdapter;
        let infra_err: InfrastructureError = render_err.into();
        let game_err: GameEngineError = infra_err.into();
        assert!(matches!(game_err, GameEngineError::Infrastructure(_)));

        // 测试领域层错误转换
        let audio_err = AudioDomainError::InvalidVolume(1.5);
        let domain_err: DomainError = audio_err.into();
        let game_err: GameEngineError = domain_err.into();
        assert!(matches!(game_err, GameEngineError::Domain(_)));
    }

    #[test]
    fn test_legacy_error_conversion() {
        // 测试从旧的EngineError转换
        let old_err = crate::error::EngineError::general("test");
        let new_err: GameEngineError = old_err.into();
        assert!(matches!(
            new_err,
            GameEngineError::Infrastructure(InfrastructureError::General(_))
        ));

        // 测试从旧的DomainError转换
        let old_domain_err = crate::domain::errors::DomainError::General("test".to_string());
        let new_err: GameEngineError = old_domain_err.into();
        assert!(matches!(
            new_err,
            GameEngineError::Domain(DomainError::General(_))
        ));
    }

    #[test]
    fn test_error_display() {
        let err = GameEngineError::infrastructure(CommonRenderError::NoAdapter);
        let error_msg = err.to_string();
        assert!(error_msg.contains("Infrastructure error"));
        assert!(error_msg.contains("no compatible GPU found"));
    }
}
