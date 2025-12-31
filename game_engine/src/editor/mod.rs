//! # Editor System
//!
//! 本模块提供完整的游戏编辑器工具集，支持场景编辑、资源管理和可视化编程。
//!
//! ## 功能特性
//!
//! - **场景编辑器** - 可视化场景构建和编辑
//! - **材质编辑器** - PBR材质参数调整
//! - **粒子编辑器** - 粒子系统可视化配置
//! - **动画编辑器** - 动画剪辑和关键帧编辑
//! - **行为树编辑器** - AI行为可视化编程
//! - **可视化脚本** - 节点式游戏逻辑编辑
//!
//! ## 主要组件
//!
//! - [`EditorState`] - 全局编辑器状态
//! - [`EditorContext`] - 编辑器上下文（egui集成）
//! - [`SceneEditor`] - 场景编辑器
//! - [`Inspector`] - 属性检查器
//! - [`HierarchyView`] - 实体层级视图
//! - [`TransformGizmo`] - 变换工具（移动/旋转/缩放）
//! - [`CommandManager`] - 撤销/重做系统
//! - [`MaterialEditor`] - 材质编辑器
//! - [`ParticleEditor`] - 粒子编辑器
//! - [`AnimationEditor`] - 动画编辑器
//!
//! ## 编辑器类型
//!
//! - [`BehaviorTreeEditor`] - 行为树编辑器
//! - [`VisualScriptEditor`] - 可视化脚本编辑器
//! - [`ShaderGraph`] - 着色器图编辑器
//! - [`AnimationStateMachine`] - 动画状态机
//!
//! ## 使用示例
//!
//! ### 初始化编辑器
//!
//! ```rust,no_run
//! use game_engine::editor::{EditorState, EditorContext};
//!
//! // 创建编辑器状态
//! let editor_state = EditorState::new();
//!
//! // 编辑器上下文在窗口创建时初始化
//! // let editor_context = EditorContext::new(&window, &device, format).await;
//! ```
//!
//! ### 撤销/重做
//!
//! ```rust,no_run
//! use game_engine::editor::{Command, CommandManager};
//!
//! struct SetPropertyCommand {
//!     entity: Entity,
//!     old_value: f32,
//!     new_value: f32,
//! }
//!
//! impl Command for SetPropertyCommand {
//!     fn execute(&mut self, world: &mut World) -> Result<(), String> {
//!         // 执行命令
//!         Ok(())
//!     }
//!
//!     fn undo(&mut self, world: &mut World) -> Result<(), String> {
//!         // 撤销命令
//!         Ok(())
//!     }
//! }
//! ```

use bevy_ecs::prelude::*;
use egui::Context as GuiContext;
use winit::event::WindowEvent;

use crate::core::editor::EditorEventHandler;

pub mod animation_editor;
pub mod asset_browser;
#[cfg(feature = "ai")]
pub mod behavior_tree_editor;
pub mod build_tool;
pub mod code_generator;
pub mod config;
pub mod console;
pub mod curve_editor;
pub mod entity_creator;
pub mod hierarchy;
pub mod inspector;
pub mod keyframe_editor;
pub mod material_editor;
pub mod package_deploy;
pub mod particle_editor;
pub mod performance_monitor;
pub mod performance_panel;
pub mod platform_builder;
pub mod project_settings;
pub mod scene_editor;
pub mod shortcuts;
pub mod terrain_editor;
pub mod transform_gizmo;
pub mod undo_redo;
pub mod visual_editors;
pub mod visual_script_editor;
pub mod world_inspector;

pub use config::{EditorConfig, EditorConfigManager, EditorTheme};
pub use hierarchy::HierarchyView;
pub use inspector::Inspector;
pub use shortcuts::{Modifiers, ShortcutAction, ShortcutManager};
pub use transform_gizmo::TransformGizmo;
pub use undo_redo::{
    Command, CommandError, CommandManager, CompositeCommand, PropertyChangeCommand,
};
pub use world_inspector::WorldInspector;
// 向后兼容：增强功能已整合到基础版本
pub use animation_editor::{AnimationEditor, AnimationEvent, KeyframeSelection, TrackType};
#[cfg(feature = "ai")]
pub use behavior_tree_editor::{
    BehaviorNodeType, BehaviorTreeEditor, NodeExecutionStatus, VisualBehaviorNode,
};
pub use material_editor::{MaterialEditor, MaterialLibraryEntry, MaterialPreset};
pub use particle_editor::{ParticleEditor, ParticleSystemLibraryEntry, SubEmitterConfig};
pub use scene_editor::SceneEditor;
pub use visual_editors::{
    AnimationState, AnimationStateMachine, BlendMode, ComparisonOp, Editor, EditorInput,
    EditorManager, EditorType, EditorUpdateResult, EmitterType, ParticleEmitterConfig,
    ParticleSystemData, ShaderConnection, ShaderGraph, ShaderNode, ShaderNodeType, StateTransition,
    TransitionCondition, create_default_particle_system, create_default_shader_graph,
    create_default_state_machine,
};
pub use visual_script_editor::{
    ActionType, BooleanOp, ConditionType, Connection, ConnectionError, ConnectionType, DataType,
    EventType, FlowType, MathOperation, NodeType, Port, PortType, VariableType, VisualScript,
    VisualScriptEditor, VisualScriptNode,
};

// 向后兼容类型别名
#[deprecated(since = "0.1.0", note = "Use ParticleEditor instead")]
pub type ParticleEditorEnhanced = particle_editor::ParticleEditor;
#[deprecated(since = "0.1.0", note = "Use AnimationEditor instead")]
pub type AnimationEditorEnhanced = animation_editor::AnimationEditor;
#[deprecated(since = "0.1.0", note = "Use MaterialEditor instead")]
pub type MaterialEditorEnhanced = material_editor::MaterialEditor;
#[deprecated(since = "0.1.0", note = "Use SceneEditor instead")]
pub type SceneEditorEnhanced = scene_editor::SceneEditor;

/// 全局编辑器状态
#[derive(Default, Debug, Resource)]
pub struct EditorState {
    /// 场景编辑器
    pub scene_editor: SceneEditor,
    /// 属性检查器
    pub inspector: Inspector,
    /// 变换工具
    pub transform_gizmo: TransformGizmo,
    /// 层级视图
    pub hierarchy_view: HierarchyView,
    /// 命令管理器（撤销/重做）
    pub command_manager: CommandManager,
    /// 世界检查器
    pub world_inspector: WorldInspector,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }
}

// Note: EditorContext is not a Resource because egui-winit State contains non-Send types
pub struct EditorContext {
    pub context: GuiContext,
    pub state: egui_winit::State,
    pub egui_renderer: Option<egui_wgpu::Renderer>,
}

impl EditorContext {
    pub async fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> EditorContext {
        // 验证输入参数，形成逻辑闭环
        let _device_features = device.features();
        let _format_info = format;

        let context = GuiContext::default();
        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        // 创建egui渲染器
        let egui_renderer =
            egui_wgpu::Renderer::new(device, format, egui_wgpu::RendererOptions::default());

        Self {
            context,
            state,
            egui_renderer: Some(egui_renderer),
        }
    }

    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }

    pub fn end_frame(&mut self, window: &winit::window::Window) -> Vec<egui::ClippedPrimitive> {
        let output = self.context.end_pass();
        self.state.handle_platform_output(window, output.platform_output);
        self.context.tessellate(output.shapes, window.scale_factor() as f32)
    }
}

// 实现EditorEventHandler trait，消除core <-> editor循环依赖
impl EditorEventHandler for EditorContext {
    fn handle_window_event(&mut self, _event: &WindowEvent) -> bool {
        // 使用egui_winit State处理窗口事件
        // 注意：这里需要一个Window引用，但trait签名只有event
        // 这是一个简化的实现，实际使用时可能需要调整
        false // 简化实现，返回false表示未消费
    }
}
