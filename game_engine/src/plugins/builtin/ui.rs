//! UI插件
//!
//! 提供基于wgpu的UI框架，支持现代UI组件和布局。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::ui::{UIState, UITheme, UIRoot, UIWidget, UIService, WidgetType, LayoutType};
use bevy_ecs::prelude::*;
use glam::Vec2;

/// UI插件配置
#[derive(Debug, Clone)]
pub struct UiConfig {
    /// 是否启用抗锯齿
    pub antialiasing: bool,
    /// 默认字体大小
    pub default_font_size: f32,
    /// 是否启用高DPI支持
    pub high_dpi: bool,
    /// UI根节点尺寸
    pub root_size: (f32, f32),
}

impl_default!(UiConfig {
    antialiasing: true,
    default_font_size: 16.0,
    high_dpi: true,
    root_size: (800.0, 600.0),
});

/// UI插件
pub struct UiPlugin {
    config: UiConfig,
}

impl UiPlugin {
    /// 创建UI插件
    pub fn new() -> Self {
        Self {
            config: UiConfig::default(),
        }
    }

    /// 使用自定义配置创建UI插件
    pub fn with_config(config: UiConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for UiPlugin {
    fn name(&self) -> &'static str {
        "UiPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides comprehensive UI framework based on wgpu with modern components and layout"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency {
                name: "RenderPlugin".to_string(),
                version_requirement: ">=1.0.0".to_string(),
            },
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入UI配置和状态
        app.insert_resource(self.config.clone());
        app.insert_resource(UIState::default());
        app.insert_resource(UITheme::default());

        // 创建UI根节点
        let root = UIRoot {
            width: self.config.root_size.0,
            height: self.config.root_size.1,
            scale_factor: if self.config.high_dpi { 2.0 } else { 1.0 },
            visible: true,
        };
        app.world_mut().spawn(root);

        // 添加UI系统
        app.add_systems(ui_update_system);
        app.add_systems(ui_layout_system);
        app.add_systems(ui_event_system);
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("UI plugin started:");
        println!("  Antialiasing: {}", self.config.antialiasing);
        println!("  High DPI: {}", self.config.high_dpi);
        println!("  Root size: {}x{}", self.config.root_size.0, self.config.root_size.1);

        // 创建示例UI
        create_example_ui(world);
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // UI更新逻辑已在系统函数中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("UI plugin shutting down");
    }
}

/// UI更新系统
pub fn ui_update_system(
    mut ui_state: ResMut<UIState>,
    mut query: Query<(Entity, &mut UIWidget)>,
) {
    // 更新UI组件状态
    for (entity, mut widget) in query.iter_mut() {
        // 检查悬停状态
        if UIService::is_point_inside(&widget, ui_state.cursor_position) {
            if ui_state.hovered_widget != Some(entity) {
                ui_state.hovered_widget = Some(entity);
                // 这里可以触发悬停事件
            }
        } else if ui_state.hovered_widget == Some(entity) {
            ui_state.hovered_widget = None;
            // 这里可以触发离开事件
        }
    }
}

/// UI布局系统
pub fn ui_layout_system(mut query: Query<&mut UIWidget>) {
    for mut widget in query.iter_mut() {
        if let WidgetType::Container { .. } = &widget.widget_type {
            UIService::update_layout(&mut widget);
        }
    }
}

/// UI事件系统
pub fn ui_event_system(
    mut ui_state: ResMut<UIState>,
    mut query: Query<(Entity, &mut UIWidget)>,
    // 这里需要输入事件，暂时简化
) {
    // 处理点击事件等
    // 实际实现需要从输入系统获取事件
}

/// 创建示例UI
fn create_example_ui(world: &mut World) {
    // 创建主容器
    let main_container = UIService::create_container(
        LayoutType::Vertical,
        Vec2::new(50.0, 50.0),
        Vec2::new(300.0, 400.0),
    );

    let container_entity = world.spawn(main_container);

    // 创建标题标签
    let title_label = UIService::create_label(
        "游戏引擎UI示例".to_string(),
        Vec2::new(10.0, 10.0),
        24.0,
    );

    world.spawn(title_label);

    // 创建按钮
    let mut click_count = 0;
    let start_button = UIService::create_button(
        "开始游戏".to_string(),
        Vec2::new(10.0, 50.0),
        Vec2::new(120.0, 40.0),
        Some(Box::new(move || {
            click_count += 1;
            println!("开始游戏按钮被点击了 {} 次", click_count);
        })),
    );

    world.spawn(start_button);

    // 创建设置按钮
    let settings_button = UIService::create_button(
        "设置".to_string(),
        Vec2::new(140.0, 50.0),
        Vec2::new(80.0, 40.0),
        Some(Box::new(|| {
            println!("设置按钮被点击");
        })),
    );

    world.spawn(settings_button);

    // 创建输入框
    let input_field = UIService::create_input(
        "输入玩家名称...".to_string(),
        Vec2::new(10.0, 110.0),
        Vec2::new(200.0, 30.0),
    );

    world.spawn(input_field);

    // 创建状态标签
    let status_label = UIService::create_label(
        "状态: 就绪".to_string(),
        Vec2::new(10.0, 160.0),
        14.0,
    );

    world.spawn(status_label);
}