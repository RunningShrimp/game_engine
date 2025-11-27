//! UI 系统模块
//!
//! 提供用户界面的创建和管理功能。
//!
//! ## 功能特性
//!
//! - 组件化 UI 系统
//! - 布局管理
//! - 主题支持
//! - 事件处理

pub mod widgets;
pub mod layout;
pub mod theme;

use bevy_ecs::prelude::*;
use glam::Vec2;

/// 布局类型
#[derive(Clone, Copy)]
pub enum LayoutType {
    /// 垂直布局
    Vertical,
    /// 水平布局
    Horizontal,
    /// 相对布局
    Relative,
    /// 网格布局
    Grid { rows: u32, cols: u32 },
}

/// UI 根节点
#[derive(Component)]
pub struct UIRoot {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub visible: bool,
}

impl Default for UIRoot {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            scale_factor: 1.0,
            visible: true,
        }
    }
}

/// UI 组件
#[derive(Component)]
pub struct UIWidget {
    pub widget_type: WidgetType,
    pub position: Vec2,
    pub size: Vec2,
    pub visible: bool,
    pub enabled: bool,
    pub z_index: i32,
}

impl Default for UIWidget {
    fn default() -> Self {
        Self {
            widget_type: WidgetType::Container {
                layout: LayoutType::Vertical,
                children: Vec::new(),
            },
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 50.0),
            visible: true,
            enabled: true,
            z_index: 0,
        }
    }
}

/// 组件类型枚举
pub enum WidgetType {
    Button {
        text: String,
        on_click: Option<Box<dyn Fn() + Send + Sync>>,
        pressed: bool,
    },
    Label {
        text: String,
        font_size: f32,
        color: [f32; 4],
    },
    Input {
        placeholder: String,
        value: String,
        focused: bool,
        max_length: Option<usize>,
    },
    Container {
        layout: LayoutType,
        children: Vec<Entity>,
    },
    Image {
        texture_id: u32,
    },
    Slider {
        min: f32,
        max: f32,
        value: f32,
        on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    },
}

/// UI 状态资源
#[derive(Resource)]
pub struct UIState {
    pub focused_widget: Option<Entity>,
    pub hovered_widget: Option<Entity>,
    pub drag_target: Option<Entity>,
    pub cursor_position: Vec2,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            focused_widget: None,
            hovered_widget: None,
            drag_target: None,
            cursor_position: Vec2::ZERO,
        }
    }
}

/// UI 主题
#[derive(Resource)]
pub struct UITheme {
    pub primary_color: [f32; 4],
    pub secondary_color: [f32; 4],
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub font_size: f32,
    pub border_radius: f32,
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            primary_color: [0.2, 0.6, 1.0, 1.0], // Blue
            secondary_color: [0.8, 0.8, 0.8, 1.0], // Light gray
            background_color: [0.1, 0.1, 0.1, 1.0], // Dark background
            text_color: [1.0, 1.0, 1.0, 1.0], // White text
            font_size: 16.0,
            border_radius: 4.0,
        }
    }
}

/// UI 服务 - 封装 UI 业务逻辑
pub struct UIService;

impl UIService {
    /// 创建按钮组件
    pub fn create_button(
        text: String,
        position: Vec2,
        size: Vec2,
        on_click: Option<Box<dyn Fn() + Send + Sync>>,
    ) -> UIWidget {
        UIWidget {
            widget_type: WidgetType::Button {
                text,
                on_click,
                pressed: false,
            },
            position,
            size,
            visible: true,
            enabled: true,
            z_index: 0,
        }
    }

    /// 创建标签组件
    pub fn create_label(text: String, position: Vec2, font_size: f32) -> UIWidget {
        UIWidget {
            widget_type: WidgetType::Label {
                text,
                font_size,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            position,
            size: Vec2::new(200.0, font_size),
            visible: true,
            enabled: true,
            z_index: 0,
        }
    }

    /// 创建输入框组件
    pub fn create_input(placeholder: String, position: Vec2, size: Vec2) -> UIWidget {
        UIWidget {
            widget_type: WidgetType::Input {
                placeholder,
                value: String::new(),
                focused: false,
                max_length: None,
            },
            position,
            size,
            visible: true,
            enabled: true,
            z_index: 0,
        }
    }

    /// 创建容器组件
    pub fn create_container(layout: LayoutType, position: Vec2, size: Vec2) -> UIWidget {
        UIWidget {
            widget_type: WidgetType::Container {
                layout,
                children: Vec::new(),
            },
            position,
            size,
            visible: true,
            enabled: true,
            z_index: 0,
        }
    }

    /// 检查组件是否被点击
    pub fn is_point_inside(widget: &UIWidget, point: Vec2) -> bool {
        if !widget.visible || !widget.enabled {
            return false;
        }

        point.x >= widget.position.x
            && point.x <= widget.position.x + widget.size.x
            && point.y >= widget.position.y
            && point.y <= widget.position.y + widget.size.y
    }

    /// 更新组件布局
    pub fn update_layout(container: &mut UIWidget) {
        if let WidgetType::Container { layout, children } = &mut container.widget_type {
            match layout {
                LayoutType::Vertical => Self::layout_vertical(children),
                LayoutType::Horizontal => Self::layout_horizontal(children),
                LayoutType::Relative => {} // 相对布局不需要重新计算
                LayoutType::Grid { rows, cols } => Self::layout_grid(children, *rows, *cols),
            }
        }
    }

    fn layout_vertical(children: &mut [Entity]) {
        // TODO: 实现垂直布局逻辑
    }

    fn layout_horizontal(children: &mut [Entity]) {
        // TODO: 实现水平布局逻辑
    }

    fn layout_grid(children: &mut [Entity], rows: u32, cols: u32) {
        // TODO: 实现网格布局逻辑
        let _ = (rows, cols);
        let _ = children;
    }

    /// 处理点击事件
    pub fn handle_click(widget: &mut UIWidget, _click_pos: Vec2) {
        if let WidgetType::Button {
            on_click: Some(callback),
            ..
        } = &widget.widget_type
        {
            callback();
        }
    }
}