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

/// XR 错误
#[derive(Error, Debug)]
pub enum XrError {
    #[error("XR not supported")]
    NotSupported,
    #[error("XR session not ready")]
    SessionNotReady,
    #[error("XR frame discarded")]
    FrameDiscarded,
    #[error("XR runtime failure: {0}")]
    RuntimeFailure(String),
}

// ============================================================================
// 交换链 (Swapchain)
// ============================================================================

/// XR 交换链 trait
pub trait XrSwapchain: Send + Sync {
    fn acquire_image(&mut self) -> Result<u32, XrError>;
    fn wait_image(&mut self, timeout_ns: i64) -> Result<(), XrError>;
    fn release_image(&mut self) -> Result<(), XrError>;
    fn get_texture_view(&self, index: u32) -> Arc<wgpu::TextureView>;
    fn resolution(&self) -> (u32, u32);
}

// ============================================================================
// 控制器输入
// ============================================================================

/// 控制器手柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hand {
    Left,
    Right,
}

/// 控制器状态
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    pub pose: Pose,
    pub aim_pose: Pose,
    pub grip_pose: Pose,
    pub trigger: f32,
    pub squeeze: f32,
    pub thumbstick: [f32; 2],
    pub buttons: ControllerButtons,
}

/// 控制器按钮
#[derive(Debug, Clone, Copy, Default)]
pub struct ControllerButtons {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub menu: bool,
    pub trigger_click: bool,
    pub squeeze_click: bool,
    pub thumbstick_click: bool,
}

/// XR 输入 trait
pub trait XrInput: Send + Sync {
    fn get_controller(&self, hand: Hand) -> Option<&ControllerState>;
    fn get_head_pose(&self) -> Pose;
    fn vibrate(&mut self, hand: Hand, amplitude: f32, duration_ns: i64);
}

// ============================================================================
// 平台特定实现 (OpenXR)
// ============================================================================

// OpenXR 实现
#[cfg(not(target_arch = "wasm32"))]
pub mod openxr_impl;

// 重新导出OpenXR实现
#[cfg(not(target_arch = "wasm32"))]
pub use openxr_impl::{OpenXrBackend, OpenXrError, OpenXrSwapchain};

// XR 渲染器
pub mod renderer;
pub use renderer::XrRenderer;

// XR 输入系统
pub mod input;
pub use input::{
    ControllerButton, HandJoint, HandJointType, HandTrackingData, HapticFeedback, XrInputEvent,
    XrInputEventHandler, XrInputEventQueue, XrInputManager,
};

// XR 手部追踪
pub mod hand_tracking;
pub use hand_tracking::{Finger, HandJoints, HandTracker, HandTrackingConfig, HandTrackingState};

// XR 空间锚点
pub mod spatial_anchors;
pub use spatial_anchors::{AnchorId, SpatialAnchor, SpatialAnchorManager};

// ============================================================================
// 异步时间扭曲 (ATW - Asynchronous Time Warp)
// ============================================================================

pub mod atw {
    use super::*;

    /// ATW 重投影数据
    pub struct AtwReprojectionData {
        pub rendered_pose: Pose,
        pub current_pose: Pose,
        pub rendered_frame: wgpu::TextureView,
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
@group(0) @binding(3) var<uniform> params: AtwParams;

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
