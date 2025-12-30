//  UI 系统模块
//
//  提供用户界面的创建和管理功能。
//
//  ## 功能特性
//
//  - 组件化 UI 系统
//  - 布局管理
//  - 主题支持
//  - 事件处理

/// UI布局模块
pub mod layout;
/// UI主题模块
pub mod theme;
/// UI组件模块
pub mod widgets;

// 测试模块
#[cfg(test)]
mod tests;

use crate::impl_default;
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
    Grid {
        /// 行数
        rows: u32,
        /// 列数
        cols: u32,
    },
}

/// UI 根节点
#[derive(Component)]
pub struct UIRoot {
    /// UI 宽度
    pub width: f32,
    /// UI 高度
    pub height: f32,
    /// 缩放因子
    pub scale_factor: f32,
    /// 是否可见
    pub visible: bool,
}

impl_default!(UIRoot {
    width: 800.0,
    height: 600.0,
    scale_factor: 1.0,
    visible: true,
});

/// UI 组件
#[derive(Component)]
pub struct UIWidget {
    /// 组件类型
    pub widget_type: WidgetType,
    /// 组件位置
    pub position: Vec2,
    /// 组件尺寸
    pub size: Vec2,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// Z轴索引
    pub z_index: i32,
}

impl_default!(UIWidget {
    widget_type: WidgetType::Container {
        layout: LayoutType::Vertical,
        children: Vec::new(),
    },
    position: Vec2::ZERO,
    size: Vec2::new(100.0, 50.0),
    visible: true,
    enabled: true,
    z_index: 0,
});

/// 组件类型枚举
pub enum WidgetType {
    /// 按钮组件
    Button {
        /// 按钮文本
        text: String,
        /// 点击回调函数
        on_click: Option<Box<dyn Fn() + Send + Sync>>,
        /// 是否按下状态
        pressed: bool,
    },
    /// 标签组件
    Label {
        /// 标签文本
        text: String,
        /// 字体大小
        font_size: f32,
        /// 文本颜色 [r, g, b, a]
        color: [f32; 4],
    },
    /// 输入框组件
    Input {
        /// 占位符文本
        placeholder: String,
        /// 当前值
        value: String,
        /// 是否获得焦点
        focused: bool,
        /// 最大长度限制
        max_length: Option<usize>,
    },
    /// 容器组件
    Container {
        /// 布局类型
        layout: LayoutType,
        /// 子组件列表
        children: Vec<Entity>,
    },
    /// 图像组件
    Image {
        /// 纹理ID
        texture_id: u32,
    },
    /// 滑块组件
    Slider {
        /// 最小值
        min: f32,
        /// 最大值
        max: f32,
        /// 当前值
        value: f32,
        /// 值改变回调函数
        on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    },
}

/// UI 状态资源
#[derive(Resource, Default)]
pub struct UIState {
    /// 当前获得焦点的组件
    pub focused_widget: Option<Entity>,
    /// 当前鼠标悬停的组件
    pub hovered_widget: Option<Entity>,
    /// 当前拖拽目标组件
    pub drag_target: Option<Entity>,
    /// 鼠标光标位置
    pub cursor_position: Vec2,
}

/// UI 主题
#[derive(Resource)]
pub struct UITheme {
    /// 主要颜色 [r, g, b, a]
    pub primary_color: [f32; 4],
    /// 次要颜色 [r, g, b, a]
    pub secondary_color: [f32; 4],
    /// 背景颜色 [r, g, b, a]
    pub background_color: [f32; 4],
    /// 文本颜色 [r, g, b, a]
    pub text_color: [f32; 4],
    /// 字体大小
    pub font_size: f32,
    /// 边框圆角半径
    pub border_radius: f32,
}

impl_default!(UITheme {
    primary_color: [0.2, 0.6, 1.0, 1.0],
    secondary_color: [0.8, 0.8, 0.8, 1.0],
    background_color: [0.1, 0.1, 0.1, 1.0],
    text_color: [1.0, 1.0, 1.0, 1.0],
    font_size: 16.0,
    border_radius: 4.0,
});

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

    fn layout_vertical(_children: &mut [Entity]) {
        // NOTE: 垂直布局逻辑待实现，当前为占位符
        // 计划实现：按垂直方向排列子元素，支持间距和对齐
    }

    fn layout_horizontal(_children: &mut [Entity]) {
        // NOTE: 水平布局逻辑待实现，当前为占位符
        // 计划实现：按水平方向排列子元素，支持间距和对齐
    }

    fn layout_grid(children: &mut [Entity], rows: u32, cols: u32) {
        // NOTE: 网格布局逻辑待实现，当前为占位符
        // 计划实现：按网格排列子元素，支持行列间距
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
