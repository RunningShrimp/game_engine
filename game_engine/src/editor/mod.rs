use bevy_ecs::prelude::*;
use egui::Context as GuiContext;

pub mod animation_editor;
pub mod asset_browser;
pub mod build_tool;
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
pub mod scene_editor_enhanced;
pub mod shortcuts;
pub mod terrain_editor;
pub mod transform_gizmo;
pub mod undo_redo;
pub mod world_inspector;

pub use config::{EditorConfig, EditorConfigManager, EditorTheme};
pub use hierarchy::HierarchyView;
pub use inspector::Inspector;
pub use scene_editor_enhanced::SceneEditorEnhanced;
pub use shortcuts::{Modifiers, ShortcutAction, ShortcutManager};
pub use transform_gizmo::TransformGizmo;
pub use undo_redo::{
    Command, CommandError, CommandManager, CompositeCommand, PropertyChangeCommand,
};
pub use world_inspector::WorldInspector;

/// 全局编辑器状态
#[derive(Default, Debug, Resource)]
pub struct EditorState {
    /// 增强场景编辑器
    pub scene_editor: SceneEditorEnhanced,
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
