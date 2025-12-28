// Unified Command/Event Protocol
//
// This protocol defines a language-agnostic interface between
//  scripting languages and a engine core.
//
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 从脚本发送到引擎的命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingCommand {
    // Entity Management
    /// 生成实体
    SpawnEntity { components: Vec<ComponentData> },
    /// 销毁实体
    DespawnEntity { entity_id: u64 },

    // Component Operations
    /// 设置组件
    SetComponent {
        entity_id: u64,
        component: ComponentData,
    },
    /// 获取组件
    GetComponent {
        entity_id: u64,
        component_type: String,
    },
    /// 移除组件
    RemoveComponent {
        entity_id: u64,
        component_type: String,
    },

    // Transform
    /// 设置位置
    SetPosition {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
    },
    /// 设置旋转
    SetRotation {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    },
    /// 设置缩放
    SetScale {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
    },
    // Rendering
    /// 渲染对象
    RenderObject { object_id: u64 },
    /// 设置相机
    SetCamera {
        entity_id: u64,
        fov: f32,
        near: f32,
        far: f32,
    },

    // Physics
    /// 添加物理力
    AddForce {
        entity_id: u64,
        fx: f32,
        fy: f32,
        fz: f32,
    },
    /// 设置速度
    SetVelocity {
        entity_id: u64,
        vx: f32,
        vy: f32,
        vz: f32,
    },

    // Animation
    /// 播放动画
    PlayAnimation {
        entity_id: u64,
        animation_name: String,
        loop_animation: bool,
    },
    /// 停止动画
    StopAnimation { entity_id: u64 },

    // Audio
    /// 播放声音
    PlaySound {
        sound_id: u64,
        volume: f32,
        pitch: f32,
    },
    /// 停止声音
    StopSound { sound_id: u64 },

    // Events
    /// 触发事件
    TriggerEvent {
        event_name: String,
        params: Vec<String>,
    },

    // System
    /// 执行系统命令
    ExecuteSystem {
        system_name: String,
        params: Vec<String>,
    },
}

/// 组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    /// 组件类型
    pub component_type: String,
    /// 组件数据（JSON字符串）
    pub data: String,
}

/// 事件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// 事件名称
    pub event_name: String,
    /// 事件参数
    pub params: Vec<String>,
}

/// 绑定协议响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingResponse {
    /// 成功响应
    Success { data: Option<String> },
    /// 错误响应
    Error { message: String, code: u32 },
}

/// 绑定错误
#[derive(Error, Debug)]
pub enum BindingError {
    /// 无效的命令
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// 无效的实体ID
    #[error("Invalid entity ID: {0}")]
    InvalidEntityId(u64),

    /// 组件未找到
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// 事件未找到
    #[error("Event not found: {0}")]
    EventNotFound(String),

    /// 协议错误
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

/// 别名类型：为向后兼容性提供
pub type BindingResult = BindingResponse;

/// 事件数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingEvent {
    /// 事件名称
    pub event_name: String,
    /// 事件参数
    pub params: Vec<String>,
}

/// 执行绑定命令的协议trait
pub trait BindingProtocol {
    /// 执行命令
    fn execute_command(&self, command: &BindingCommand) -> BindingResponse;

    /// 订阅事件
    fn subscribe_event(&self, event_name: &str) -> Result<(), BindingError>;

    /// 取消订阅事件
    fn unsubscribe_event(&self, event_name: &str) -> Result<(), BindingError>;
}

/// 语言绑定适配器trait
///
/// 提供脚本语言（JS、Lua等）到引擎的适配接口
pub trait BindingAdapter: Send + Sync {
    /// 初始化绑定
    fn init(&mut self);

    /// 绑定引擎API到脚本环境
    fn bind_engine_api(&mut self);

    /// 执行命令（从脚本到引擎）
    fn execute_command(&mut self, cmd: BindingCommand) -> BindingResult;

    /// 分发事件（从引擎到脚本）
    fn dispatch_event(&mut self, event: BindingEvent);

    /// 轮询命令（从脚本到引擎）
    fn poll_commands(&mut self) -> Vec<BindingCommand>;

    /// 关闭绑定
    fn shutdown(&mut self);
}
