// ============================================================================
// OpenXR VR/AR/MR 集成模块
// 支持立体渲染、空间追踪、控制器输入
// ============================================================================

use crate::impl_default;
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// XR 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XrSessionState {
    Idle,
    Ready,
    Synchronized,
    Visible,
    Focused,
    Stopping,
    Exiting,
}

/// 视图姿态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: Quat,
}

impl_default!(Pose {
    position: Vec3::ZERO,
    orientation: Quat::IDENTITY,
});

impl Pose {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.orientation, self.position)
    }

    pub fn inverse(&self) -> Self {
        let inv_orientation = self.orientation.inverse();
        Self {
            position: inv_orientation * (-self.position),
            orientation: inv_orientation,
        }
    }
}

/// 视野参数 (Field of View)
#[derive(Debug, Clone, Copy)]
pub struct Fov {
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_up: f32,
    pub angle_down: f32,
}

impl Fov {
    pub fn to_projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        let tan_left = self.angle_left.tan();
        let tan_right = self.angle_right.tan();
        let tan_up = self.angle_up.tan();
        let tan_down = self.angle_down.tan();

        let tan_width = tan_right - tan_left;
        let tan_height = tan_up - tan_down;

        Mat4::from_cols_array(&[
            2.0 / tan_width,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / tan_height,
            0.0,
            0.0,
            (tan_right + tan_left) / tan_width,
            (tan_up + tan_down) / tan_height,
            -far / (far - near),
            -1.0,
            0.0,
            0.0,
            -(far * near) / (far - near),
            0.0,
        ])
    }
}

/// XR 视图 (单眼)
#[derive(Debug, Clone)]
pub struct XrView {
    pub pose: Pose,
    pub fov: Fov,
    pub view_index: u32,
}

impl XrView {
    pub fn view_matrix(&self) -> Mat4 {
        self.pose.inverse().to_matrix()
    }

    pub fn projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        self.fov.to_projection_matrix(near, far)
    }

    pub fn view_projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        self.projection_matrix(near, far) * self.view_matrix()
    }
}

/// XR 事件
#[derive(Debug, Clone)]
pub enum XrEvent {
    SessionStateChanged(XrSessionState),
    ReferenceSpaceChanged,
    InteractionProfileChanged,
}

/// XR 会话配置
#[derive(Debug, Clone)]
pub struct XrConfig {
    pub application_name: String,
    pub blend_mode: BlendMode,
    pub reference_space: ReferenceSpaceType,
}

impl_default!(XrConfig {
    application_name: "GameEngine XR".to_string(),
    blend_mode: BlendMode::Opaque,
    reference_space: ReferenceSpaceType::Stage,
});

/// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,     // VR
    Additive,   // AR (光学透视)
    AlphaBlend, // AR (视频透视)
}

/// 参考空间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceSpaceType {
    View,      // 头部相对
    Local,     // 起始位置
    Stage,     // 房间空间
    Unbounded, // 无边界 (AR)
}

/// XR 会话 trait
pub trait XrSession: Send + Sync {
    fn state(&self) -> XrSessionState;
    fn begin_frame(&mut self) -> Result<XrFrameState, XrError>;
    fn end_frame(&mut self, layers: &[XrCompositionLayer]) -> Result<(), XrError>;
    fn locate_views(&self, time: i64) -> Result<Vec<XrView>, XrError>;
    fn poll_events(&mut self) -> Vec<XrEvent>;
}

/// 帧状态
#[derive(Debug, Clone)]
pub struct XrFrameState {
    pub predicted_display_time: i64,
    pub predicted_display_period: i64,
    pub should_render: bool,
}

/// 合成层
#[derive(Debug, Clone)]
pub enum XrCompositionLayer {
    Projection {
        views: Vec<XrProjectionView>,
    },
    Quad {
        pose: Pose,
        size: [f32; 2],
        swapchain_index: u32,
    },
}

/// 投影视图
#[derive(Debug, Clone)]
pub struct XrProjectionView {
    pub pose: Pose,
    pub fov: Fov,
    pub swapchain_index: u32,
    pub image_rect: [i32; 4], // x, y, width, height
}

/// XR 相关错误类型
#[derive(Error, Debug)]
pub enum XrError {
    /// XR 功能不受支持
    #[error("XR not supported")]
    NotSupported,
    /// XR 会话未就绪
    #[error("XR session not ready")]
    SessionNotReady,
    /// XR 帧被丢弃
    #[error("XR frame discarded")]
    FrameDiscarded,
    /// XR 运行时失败
    #[error("XR runtime failure: {0}")]
    RuntimeFailure(String),
    /// 功能不受支持
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),
}

// ============================================================================
// 交换链 (Swapchain)
// ============================================================================

/// XR 交换链 trait，定义XR交换链所需的所有操作
pub trait XrSwapchain: Send + Sync {
    /// 获取下一个可用的图像索引
    fn acquire_image(&mut self) -> Result<u32, XrError>;
    /// 等待指定的图像可用于渲染
    fn wait_image(&mut self, timeout_ns: i64) -> Result<(), XrError>;
    /// 释放已渲染完成的图像
    fn release_image(&mut self) -> Result<(), XrError>;
    /// 获取指定索引的纹理视图
    fn get_texture_view(&self, index: u32) -> Arc<wgpu::TextureView>;
    /// 获取交换链的分辨率
    fn resolution(&self) -> (u32, u32);
}

// ============================================================================
// 控制器输入
// ============================================================================

/// 控制器手柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hand {
    /// 左手
    Left,
    /// 右手
    Right,
}

/// 控制器状态
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    /// 控制器的世界位置和旋转
    pub pose: Pose,
    /// 瞄准姿态（通常指向光线方向）
    pub aim_pose: Pose,
    /// 握持姿态（手指握住的位置）
    pub grip_pose: Pose,
    /// 扳机按压值 (0.0 - 1.0)
    pub trigger: f32,
    /// 挤压按压值 (0.0 - 1.0)
    pub squeeze: f32,
    /// 摇杆输入 [x, y]
    pub thumbstick: [f32; 2],
    /// 按钮状态
    pub buttons: ControllerButtons,
}

/// 控制器按钮
#[derive(Debug, Clone, Copy, Default)]
pub struct ControllerButtons {
    /// A 按钮（右手）/ X 按钮（左手）
    pub a: bool,
    /// B 按钮（右手）/ Y 按钮（左手）
    pub b: bool,
    /// X 按钮（左手）
    pub x: bool,
    /// Y 按钮（左手）
    pub y: bool,
    /// 菜单按钮
    pub menu: bool,
    /// 扳机点击
    pub trigger_click: bool,
    /// 挤压点击
    pub squeeze_click: bool,
    /// 摇杆点击
    pub thumbstick_click: bool,
}

/// XR 输入 trait，定义XR输入系统所需的所有操作
pub trait XrInput: Send + Sync {
    /// 获取指定手柄的控制器状态
    fn get_controller(&self, hand: Hand) -> Option<&ControllerState>;
    /// 获取头部（HMD）的当前姿态
    fn get_head_pose(&self) -> Pose;
    /// 对指定手柄施加震动反馈
    fn vibrate(&mut self, hand: Hand, amplitude: f32, duration_ns: i64);
}

// ============================================================================
// 平台特定实现 (OpenXR)
// ============================================================================

/// OpenXR 实现模块
#[cfg(not(target_arch = "wasm32"))]
pub mod openxr_impl;

// 重新导出OpenXR实现
#[cfg(not(target_arch = "wasm32"))]
pub use openxr_impl::{OpenXrBackend, OpenXrError, OpenXrSwapchain};

// ============================================================================
// 手势识别
// ============================================================================

/// 手势识别器
///
/// 基于手部追踪数据识别常见手势
pub struct GestureRecognizer {
    /// 手势历史记录
    gesture_history: std::collections::VecDeque<hand_tracking::GestureEvent>,
    /// 手势识别阈值
    recognition_threshold: f32,
}

impl GestureRecognizer {
    /// 创建新的手势识别器
    pub fn new() -> Self {
        Self {
            gesture_history: std::collections::VecDeque::with_capacity(10),
            recognition_threshold: 0.7, // 70%置信度
        }
    }

    /// 识别手势
    ///
    /// # 参数
    /// - `hand_joints`: 手部关节数据
    /// - `hand`: 手部（左手或右手）
    ///
    /// # 返回
    /// 如果识别到手势，返回手势事件
    pub fn recognize(&mut self, hand_joints: &hand_tracking::HandJoints, hand: Hand) -> Option<hand_tracking::GestureEvent> {
    if let Some(gesture) = hand_joints.detect_gesture() {
        let confidence = hand_joints.confidence();
        let position = hand_joints.get_palm_position().unwrap_or(Vec3::ZERO);
        let timestamp = crate::core::utils::current_timestamp_ms();

        // 将 HandGesture 转换为 Gesture
        let gesture_enum = match gesture {
            hand_tracking::HandGesture::Fist => hand_tracking::Gesture::Fist,
            hand_tracking::HandGesture::OpenHand => hand_tracking::Gesture::Open,
        };

        let event = hand_tracking::GestureEvent {
            gesture: gesture_enum,
            hand,
            confidence,
            position,
            timestamp,
        };

        // 记录到历史
        self.gesture_history.push_back(event.clone());
            if self.gesture_history.len() > 10 {
                self.gesture_history.pop_front();
            }

            // 如果置信度足够高，返回事件
            if confidence >= self.recognition_threshold {
                Some(event)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 设置识别阈值
    pub fn set_threshold(&mut self, threshold: f32) {
        self.recognition_threshold = threshold.clamp(0.0, 1.0);
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 眼动追踪
// ============================================================================

/// 眼动追踪接口（如果硬件支持）
///
/// 提供眼动追踪功能，用于注视点渲染等优化
#[cfg(feature = "xr")]
pub trait EyeTracking {
    /// 获取左眼注视点（归一化坐标，范围[-1, 1]）
    fn left_eye_gaze(&self) -> Option<(f32, f32)>;

    /// 获取右眼注视点（归一化坐标，范围[-1, 1]）
    fn right_eye_gaze(&self) -> Option<(f32, f32)>;

    /// 获取双眼注视点（归一化坐标，范围[-1, 1]）
    fn combined_gaze(&self) -> Option<(f32, f32)>;

    /// 检查眼动追踪是否可用
    fn is_available(&self) -> bool;

    /// 获取眼动追踪置信度（0.0-1.0）
    fn confidence(&self) -> f32;
}

/// 眼动追踪数据
#[derive(Debug, Clone)]
pub struct EyeTrackingData {
    /// 左眼注视点（归一化坐标）
    pub left_gaze: Option<(f32, f32)>,
    /// 右眼注视点（归一化坐标）
    pub right_gaze: Option<(f32, f32)>,
    /// 双眼注视点（归一化坐标）
    pub combined_gaze: Option<(f32, f32)>,
    /// 追踪置信度（0.0-1.0）
    pub confidence: f32,
    /// 是否可用
    pub available: bool,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}

impl Default for EyeTrackingData {
    fn default() -> Self {
        Self {
            left_gaze: None,
            right_gaze: None,
            combined_gaze: None,
            confidence: 0.0,
            available: false,
            timestamp: crate::core::utils::current_timestamp_ms(),
        }
    }
}

/// 眼动追踪事件
#[derive(Debug, Clone)]
pub struct EyeTrackingEvent {
    /// 眼动追踪数据
    pub data: EyeTrackingData,
    /// 事件类型
    pub event_type: EyeTrackingEventType,
}

/// 眼动追踪事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeTrackingEventType {
    /// 注视点变化
    GazeChanged,
    /// 眼动追踪开始
    TrackingStarted,
    /// 眼动追踪停止
    TrackingStopped,
    /// 眼动追踪丢失
    TrackingLost,
}

/// 眼动追踪管理器（占位实现）
///
/// 实际实现需要硬件支持（如Varjo、Pico等支持眼动追踪的VR头显）
pub struct EyeTrackingManager {
    /// 当前眼动追踪数据
    current_data: EyeTrackingData,
    /// 是否启用
    enabled: bool,
}

impl EyeTrackingManager {
    /// 创建新的眼动追踪管理器
    pub fn new() -> Self {
        Self {
            current_data: EyeTrackingData::default(),
            enabled: false,
        }
    }

    /// 检查是否支持眼动追踪
    pub fn is_supported(&self) -> bool {
        // 占位实现：实际应该检查硬件支持
        false
    }

    /// 启用眼动追踪
    pub fn enable(&mut self) -> Result<(), XrError> {
        if !self.is_supported() {
            return Err(XrError::FeatureNotSupported("Eye tracking not supported".to_string()));
        }
        self.enabled = true;
        Ok(())
    }

    /// 禁用眼动追踪
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 更新眼动追踪数据
    pub fn update(&mut self) -> Option<EyeTrackingEvent> {
        if !self.enabled || !self.is_supported() {
            return None;
        }

        // 占位实现：实际应该从硬件获取数据
        // 这里返回None表示数据不可用
        None
    }

    /// 获取当前眼动追踪数据
    pub fn get_data(&self) -> &EyeTrackingData {
        &self.current_data
    }
}

impl Default for EyeTrackingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// XR 渲染器模块
pub mod renderer;
pub use renderer::XrRenderer;

/// XR 输入系统模块
pub mod input;
pub use input::{
    ControllerButton, HandJoint, HandJointType, HandTrackingData, HapticFeedback, XrInputEvent,
    XrInputEventHandler, XrInputEventQueue, XrInputManager,
};
/// XR 手部追踪模块
pub mod hand_tracking;
pub use hand_tracking::{Finger, HandJoints, HandTracker, HandTrackingConfig, HandTrackingState};

/// XR 空间锚点模块
pub mod spatial_anchors;
pub use spatial_anchors::{AnchorId, SpatialAnchor, SpatialAnchorManager};

/// XR 空间映射模块
pub mod spatial_mapping;
pub use spatial_mapping::{
    DetectedPlane, MeshId, MeshTriangle, MeshVertex, PlaneId, PlaneType, SpatialMappingConfig,
    SpatialMappingManager, SpatialMesh,
};

// 重新导出手势识别和眼动追踪
// Gesture 和 GestureEvent 已在 hand_tracking.rs 中定义，在此处直接引用
// GestureRecognizer 已在上面定义，无需重新导出
// EyeTrackingData, EyeTrackingEvent, EyeTrackingEventType, EyeTrackingManager 已在上面定义，无需重新导出

/// 异步时间扭曲 (ATW - Asynchronous Time Warp) 模块
pub mod atw {
    use super::*;

    /// ATW 重投影数据，包含渲染和当前的姿态以及纹理
    pub struct AtwReprojectionData {
        /// 渲染时的头部姿态
        pub rendered_pose: Pose,
        /// 当前的头部姿态
        pub current_pose: Pose,
        /// 上一帧渲染的纹理视图
        pub rendered_frame: wgpu::TextureView,
        /// 可选的深度缓冲区
        pub depth_buffer: Option<wgpu::TextureView>,
    }

    /// 计算姿态差异矩阵
    pub fn compute_delta_rotation(rendered: &Pose, current: &Pose) -> Mat4 {
        let delta_orientation = current.orientation * rendered.orientation.inverse();
        Mat4::from_quat(delta_orientation)
    }

    /// ATW Compute Shader (WGSL)
    pub const ATW_SHADER: &str = r#"
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var depth_texture: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;
@group(0) @binding(4) var<uniform> params: AtwParams;

struct AtwParams {
    delta_rotation: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    projection: mat4x4<f32>,
    resolution: vec2<f32>,
};

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel = vec2<i32>(global_id.xy);
    let uv = (vec2<f32>(pixel) + 0.5) / params.resolution;
    
    // 获取深度
    let depth = textureLoad(depth_texture, pixel, 0).r;
    
    // 反投影到视图空间
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let clip_pos = vec4<f32>(ndc, depth, 1.0);
    var view_pos = params.inv_projection * clip_pos;
    view_pos /= view_pos.w;
    
    // 应用旋转差异
    let rotated_pos = params.delta_rotation * vec4<f32>(view_pos.xyz, 1.0);
    
    // 重新投影
    let new_clip = params.projection * rotated_pos;
    let new_ndc = new_clip.xy / new_clip.w;
    let new_uv = vec2<f32>(new_ndc.x * 0.5 + 0.5, 0.5 - new_ndc.y * 0.5);
    
    // 采样原始帧
    let color = textureSampleLevel(input_texture, tex_sampler, new_uv, 0.0);
    
    textureStore(output_texture, pixel, color);
}
"#;
}

// ============================================================================
// Foveated Rendering (注视点渲染)
// ============================================================================

pub mod foveated {
    use crate::impl_default;

    /// 注视点渲染配置
    #[derive(Debug, Clone)]
    pub struct FoveatedConfig {
        /// 是否启用
        pub enabled: bool,
        /// 中心区域半径 (归一化, 0-1)
        pub inner_radius: f32,
        /// 过渡区域半径
        pub middle_radius: f32,
        /// 外围区域半径
        pub outer_radius: f32,
        /// 中心分辨率缩放
        pub inner_scale: f32,
        /// 过渡分辨率缩放
        pub middle_scale: f32,
        /// 外围分辨率缩放
        pub outer_scale: f32,
        /// 注视点 (归一化坐标, 默认中心)
        pub gaze_point: [f32; 2],
    }

    impl_default!(FoveatedConfig {
        enabled: true,
        inner_radius: 0.2,
        middle_radius: 0.4,
        outer_radius: 1.0,
        inner_scale: 1.0,
        middle_scale: 0.5,
        outer_scale: 0.25,
        gaze_point: [0.5, 0.5],
    });

    /// 计算注视点渲染的分辨率缩放
    pub fn compute_resolution_scale(uv: [f32; 2], config: &FoveatedConfig) -> f32 {
        if !config.enabled {
            return 1.0;
        }

        let dx = uv[0] - config.gaze_point[0];
        let dy = uv[1] - config.gaze_point[1];
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < config.inner_radius {
            config.inner_scale
        } else if distance < config.middle_radius {
            let t = (distance - config.inner_radius) / (config.middle_radius - config.inner_radius);
            config.inner_scale + t * (config.middle_scale - config.inner_scale)
        } else {
            let t = ((distance - config.middle_radius)
                / (config.outer_radius - config.middle_radius))
                .min(1.0);
            config.middle_scale + t * (config.outer_scale - config.middle_scale)
        }
    }

    /// Foveated Rendering Shader (用于多分辨率渲染)
    pub const FOVEATED_RECONSTRUCT_SHADER: &str = r#"
@group(0) @binding(0) var inner_texture: texture_2d<f32>;
@group(0) @binding(1) var middle_texture: texture_2d<f32>;
@group(0) @binding(2) var outer_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var tex_sampler: sampler;
@group(0) @binding(5) var<uniform> params: FoveatedParams;

struct FoveatedParams {
    gaze_point: vec2<f32>,
    inner_radius: f32,
    middle_radius: f32,
    outer_radius: f32,
    resolution: vec2<f32>,
};

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel = vec2<i32>(global_id.xy);
    let uv = (vec2<f32>(pixel) + 0.5) / params.resolution;
    
    let offset = uv - params.gaze_point;
    let distance = length(offset);
    
    var color: vec4<f32>;
    
    if distance < params.inner_radius {
        color = textureSampleLevel(inner_texture, tex_sampler, uv, 0.0);
    } else if distance < params.middle_radius {
        let inner_color = textureSampleLevel(inner_texture, tex_sampler, uv, 0.0);
        let middle_color = textureSampleLevel(middle_texture, tex_sampler, uv, 0.0);
        let t = (distance - params.inner_radius) / (params.middle_radius - params.inner_radius);
        color = mix(inner_color, middle_color, t);
    } else {
        let middle_color = textureSampleLevel(middle_texture, tex_sampler, uv, 0.0);
        let outer_color = textureSampleLevel(outer_texture, tex_sampler, uv, 0.0);
        let t = min((distance - params.middle_radius) / (params.outer_radius - params.middle_radius), 1.0);
        color = mix(middle_color, outer_color, t);
    }
    
    textureStore(output_texture, pixel, color);
}
"#;
}

// ============================================================================
// 眼动追踪
// ============================================================================

pub mod eye_tracking {
    use super::*;

    /// 眼动追踪数据
    #[derive(Debug, Clone, Default)]
    pub struct EyeGazeData {
        /// 是否有效
        pub is_valid: bool,
        /// 注视方向 (归一化向量)
        pub gaze_direction: Vec3,
        /// 注视原点 (眼睛位置)
        pub gaze_origin: Vec3,
        /// 瞳孔直径 (毫米)
        pub pupil_diameter: f32,
        /// 眨眼状态
        pub blink: bool,
    }

    /// 眼动追踪 trait
    pub trait EyeTracker: Send + Sync {
        fn get_gaze(&self, eye: Eye) -> Option<EyeGazeData>;
        fn get_combined_gaze(&self) -> Option<EyeGazeData>;
        fn is_supported(&self) -> bool;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Eye {
        Left,
        Right,
    }
}

// ============================================================================
// XR 渲染器适配
// ============================================================================

/// XR 渲染上下文
pub struct XrRenderContext {
    pub view: XrView,
    pub render_target: Arc<wgpu::TextureView>,
    pub depth_target: Arc<wgpu::TextureView>,
}

/// 为 XR 准备渲染参数
pub fn prepare_xr_render(
    session: &dyn XrSession,
    swapchains: &[Box<dyn XrSwapchain>],
    frame_state: &XrFrameState,
) -> Result<Vec<XrRenderContext>, XrError> {
    let views = session.locate_views(frame_state.predicted_display_time)?;

    let mut contexts = Vec::with_capacity(views.len());

    for (i, view) in views.iter().enumerate() {
        if let Some(swapchain) = swapchains.get(i) {
            // NOTE: 深度目标创建逻辑待实现
            // Get texture views from swapchain (already wrapped in Arc)
            let render_view = swapchain.get_texture_view(0);
            contexts.push(XrRenderContext {
                view: view.clone(),
                render_target: render_view.clone(),
                depth_target: render_view.clone(), // 占位
            });
        }
    }

    Ok(contexts)
}
