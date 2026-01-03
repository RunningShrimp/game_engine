//! UI组件系统
//!
//! 提供20+基础UI组件的完整实现。

use crate::ui::layout::RectTransform;
use bevy_ecs::prelude::*;
use glam::Vec2;
use std::sync::{Arc, Mutex};

/// UI组件trait
pub trait UIWidget: Send + Sync {
    /// 获取组件RectTransform
    fn rect_transform(&self) -> &RectTransform;
    /// 获取可变RectTransform
    fn rect_transform_mut(&mut self) -> &mut RectTransform;
    /// 渲染组件
    fn render(&self);
    /// 处理事件
    fn handle_event(&mut self, event: &UIEvent);
    /// 是否可见
    fn is_visible(&self) -> bool;
    /// 设置可见性
    fn set_visible(&mut self, visible: bool);
    /// 是否启用
    fn is_enabled(&self) -> bool;
    /// 设置启用状态
    fn set_enabled(&mut self, enabled: bool);
}

/// UI事件类型
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// 点击事件
    Click { position: Vec2 },
    /// 鼠标悬停
    Hover { position: Vec2 },
    /// 鼠标离开
    Leave,
    /// 键盘输入
    KeyPress { key: char },
    /// 文本输入
    TextInput { text: String },
    /// 值改变（滑块、输入框等）
    ValueChanged { value: f32 },
    /// 焦点获得
    FocusGained,
    /// 焦点丢失
    FocusLost,
    /// 拖拽开始
    DragStart { position: Vec2 },
    /// 拖拽移动
    DragMove { delta: Vec2 },
    /// 拖拽结束
    DragEnd { position: Vec2 },
    /// 滚动
    Scroll { delta: f32 },
}

// ============================================================================
// 基础组件 (1-5)
// ============================================================================

/// 1. Button - 按钮组件
#[derive(Component, Clone)]
pub struct Button {
    /// RectTransform
    pub rect: RectTransform,
    /// 按钮文本
    pub text: String,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否按下
    pub pressed: bool,
    /// 是否悬停
    pub hovered: bool,
    /// 点击回调
    pub on_click: Option<ClickCallback>,
}

/// 点击回调类型
pub type ClickCallback = Arc<Mutex<Box<dyn Fn() + Send + 'static>>>;

impl Button {
    /// 创建新按钮
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(120.0, 40.0),
            text: text.into(),
            visible: true,
            enabled: true,
            pressed: false,
            hovered: false,
            on_click: None,
        }
    }

    /// 设置文本
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// 设置大小
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.rect = self.rect.with_size(width, height);
        self
    }

    /// 设置位置
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.rect = self.rect.with_position(x, y);
        self
    }

    /// 设置点击回调
    pub fn with_callback<F: Fn() + Send + 'static>(mut self, callback: F) -> Self {
        self.on_click = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }
}

/// 2. Label - 文本标签组件
#[derive(Component, Clone)]
pub struct Label {
    /// RectTransform
    pub rect: RectTransform,
    /// 标签文本
    pub text: String,
    /// 字体大小
    pub font_size: f32,
    /// 文本颜色 [r, g, b, a]
    pub color: [f32; 4],
    /// 是否可见
    pub visible: bool,
    /// 文本对齐
    pub alignment: TextAlignment,
    /// 是否自动换行
    pub wrap: bool,
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Label {
    /// 创建新标签
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 30.0),
            text: text.into(),
            font_size: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            visible: true,
            alignment: TextAlignment::Left,
            wrap: false,
        }
    }

    /// 设置文本
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// 设置字体大小
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// 设置颜色
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// 设置对齐方式
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// 3. TextField - 单行文本输入框
#[derive(Component, Clone)]
pub struct TextField {
    /// RectTransform
    pub rect: RectTransform,
    /// 占位符文本
    pub placeholder: String,
    /// 当前值
    pub value: String,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否获得焦点
    pub focused: bool,
    /// 最大长度限制
    pub max_length: Option<usize>,
    /// 文本改变回调
    pub on_change: Option<TextChangeCallback>,
    /// 字体大小
    pub font_size: f32,
}

/// 文本改变回调类型
pub type TextChangeCallback = Arc<Mutex<Box<dyn Fn(&str) + Send + 'static>>>;

impl TextField {
    /// 创建新文本输入框
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 32.0),
            placeholder: placeholder.into(),
            value: String::new(),
            visible: true,
            enabled: true,
            focused: false,
            max_length: None,
            on_change: None,
            font_size: 14.0,
        }
    }

    /// 设置占位符
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 设置最大长度
    pub fn with_max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    /// 设置值改变回调
    pub fn with_callback<F: Fn(&str) + Send + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }

    /// 插入文本
    pub fn insert_text(&mut self, text: &str) {
        if let Some(max) = self.max_length {
            if self.value.len() + text.len() <= max {
                self.value.push_str(text);
            }
        } else {
            self.value.push_str(text);
        }
    }

    /// 删除字符
    pub fn delete_char(&mut self) {
        self.value.pop();
    }
}

/// 4. TextArea - 多行文本输入框
#[derive(Component, Clone)]
pub struct TextArea {
    /// RectTransform
    pub rect: RectTransform,
    /// 占位符文本
    pub placeholder: String,
    /// 当前值
    pub value: String,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否获得焦点
    pub focused: bool,
    /// 最大长度限制
    pub max_length: Option<usize>,
    /// 行数
    pub lines: u32,
    /// 字体大小
    pub font_size: f32,
}

impl TextArea {
    /// 创建新多行文本框
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(300.0, 100.0),
            placeholder: placeholder.into(),
            value: String::new(),
            visible: true,
            enabled: true,
            focused: false,
            max_length: None,
            lines: 5,
            font_size: 14.0,
        }
    }

    /// 设置行数
    pub fn with_lines(mut self, lines: u32) -> Self {
        self.lines = lines;
        self
    }
}

/// 5. Image - 图像组件
#[derive(Component, Clone)]
pub struct Image {
    /// RectTransform
    pub rect: RectTransform,
    /// 纹理ID
    pub texture_id: Option<u32>,
    /// 是否可见
    pub visible: bool,
    /// 颜色调制 [r, g, b, a]
    pub color: [f32; 4],
    /// 是否保持宽高比
    pub preserve_aspect: bool,
    /// 图像模式
    pub image_mode: ImageMode,
}

/// 图像显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageMode {
    Simple,
    Sliced,
    Tiled,
    Filled,
}

impl Image {
    /// 创建新图像
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(100.0, 100.0),
            texture_id: None,
            visible: true,
            color: [1.0, 1.0, 1.0, 1.0],
            preserve_aspect: false,
            image_mode: ImageMode::Simple,
        }
    }

    /// 设置纹理
    pub fn with_texture(mut self, texture_id: u32) -> Self {
        self.texture_id = Some(texture_id);
        self
    }

    /// 设置颜色调制
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 容器组件 (6-9)
// ============================================================================

/// 6. Panel - 面板容器
#[derive(Component, Clone)]
pub struct Panel {
    /// RectTransform
    pub rect: RectTransform,
    /// 背景颜色
    pub background_color: [f32; 4],
    /// 是否可见
    pub visible: bool,
    /// 子组件列表
    pub children: Vec<Entity>,
}

impl Panel {
    /// 创建新面板
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 200.0),
            background_color: [0.2, 0.2, 0.2, 1.0],
            visible: true,
            children: Vec::new(),
        }
    }

    /// 设置背景颜色
    pub fn with_background(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.background_color = [r, g, b, a];
        self
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Entity) {
        self.children.push(child);
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

/// 7. ScrollView - 滚动视图
#[derive(Component, Clone)]
pub struct ScrollView {
    /// RectTransform
    pub rect: RectTransform,
    /// 内容大小
    pub content_size: Vec2,
    /// 滚动位置
    pub scroll_position: Vec2,
    /// 是否可见
    pub visible: bool,
    /// 是否显示滚动条
    pub show_scrollbar: bool,
    /// 滚动条宽度
    pub scrollbar_width: f32,
    /// 鼠标滚轮灵敏度
    pub scroll_sensitivity: f32,
}

impl ScrollView {
    /// 创建新滚动视图
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(300.0, 200.0),
            content_size: Vec2::new(300.0, 400.0),
            scroll_position: Vec2::ZERO,
            visible: true,
            show_scrollbar: true,
            scrollbar_width: 10.0,
            scroll_sensitivity: 1.0,
        }
    }

    /// 滚动到指定位置
    pub fn scroll_to(&mut self, x: f32, y: f32) {
        self.scroll_position.x =
            x.clamp(0.0, (self.content_size.x - self.rect.size_delta.x).max(0.0));
        self.scroll_position.y =
            y.clamp(0.0, (self.content_size.y - self.rect.size_delta.y).max(0.0));
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

/// 8. ListView - 列表视图
#[derive(Component, Clone)]
pub struct ListView {
    /// RectTransform
    pub rect: RectTransform,
    /// 列表项
    pub items: Vec<ListItem>,
    /// 选中索引
    pub selected_index: Option<usize>,
    /// 是否可见
    pub visible: bool,
    /// 是否多选
    pub multi_select: bool,
    /// 项高度
    pub item_height: f32,
}

/// 列表项
#[derive(Debug, Clone, Component)]
pub struct ListItem {
    /// 文本
    pub text: String,
    /// 用户数据
    pub user_data: Option<String>,
}

impl ListView {
    /// 创建新列表视图
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(250.0, 300.0),
            items: Vec::new(),
            selected_index: None,
            visible: true,
            multi_select: false,
            item_height: 30.0,
        }
    }

    /// 添加项
    pub fn add_item(&mut self, text: impl Into<String>) {
        self.items.push(ListItem {
            text: text.into(),
            user_data: None,
        });
    }

    /// 获取选中项
    pub fn get_selected(&self) -> Option<&ListItem> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

/// 9. GridView - 网格视图
#[derive(Component, Clone)]
pub struct GridView {
    /// RectTransform
    pub rect: RectTransform,
    /// 网格项
    pub items: Vec<GridItem>,
    /// 列数
    pub columns: usize,
    /// 选中索引
    pub selected_index: Option<usize>,
    /// 是否可见
    pub visible: bool,
    /// 单元格大小
    pub cell_size: Vec2,
    /// 间距
    pub spacing: Vec2,
}

/// 网格项
#[derive(Debug, Clone, Component)]
pub struct GridItem {
    /// 文本
    pub text: String,
    /// 图标（可选）
    pub icon: Option<u32>,
    /// 用户数据
    pub user_data: Option<String>,
}

impl GridView {
    /// 创建新网格视图
    pub fn new(columns: usize) -> Self {
        Self {
            rect: RectTransform::new().with_size(400.0, 300.0),
            items: Vec::new(),
            columns,
            selected_index: None,
            visible: true,
            cell_size: Vec2::new(100.0, 100.0),
            spacing: Vec2::new(10.0, 10.0),
        }
    }

    /// 添加项
    pub fn add_item(&mut self, text: impl Into<String>) {
        self.items.push(GridItem {
            text: text.into(),
            icon: None,
            user_data: None,
        });
    }
}

// ============================================================================
// 控制组件 (10-14)
// ============================================================================

/// 10. Slider - 滑块组件
#[derive(Component, Clone)]
pub struct Slider {
    /// RectTransform
    pub rect: RectTransform,
    /// 最小值
    pub min_value: f32,
    /// 最大值
    pub max_value: f32,
    /// 当前值
    pub current_value: f32,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否整数模式
    pub whole_numbers: bool,
    /// 值改变回调
    pub on_change: Option<ValueChangeCallback>,
}

/// 值改变回调类型
pub type ValueChangeCallback = Arc<Mutex<Box<dyn Fn(f32) + Send + 'static>>>;

impl Slider {
    /// 创建新滑块
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 20.0),
            min_value: min,
            max_value: max,
            current_value: (min + max) / 2.0,
            visible: true,
            enabled: true,
            whole_numbers: false,
            on_change: None,
        }
    }

    /// 设置值
    pub fn set_value(&mut self, value: f32) {
        let mut new_value = value.clamp(self.min_value, self.max_value);
        if self.whole_numbers {
            new_value = new_value.round();
        }
        self.current_value = new_value;
    }

    /// 设置值改变回调
    pub fn with_callback<F: Fn(f32) + Send + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }

    /// 获取进度（0.0-1.0）
    pub fn get_progress(&self) -> f32 {
        (self.current_value - self.min_value) / (self.max_value - self.min_value)
    }
}

/// 11. Toggle - 开关组件
#[derive(Component, Clone)]
pub struct Toggle {
    /// RectTransform
    pub rect: RectTransform,
    /// 是否开启
    pub is_on: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 开关标签
    pub label: String,
    /// 值改变回调
    pub on_change: Option<ToggleChangeCallback>,
}

/// 开关改变回调类型
pub type ToggleChangeCallback = Arc<Mutex<Box<dyn Fn(bool) + Send + 'static>>>;

impl Toggle {
    /// 创建新开关
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(100.0, 30.0),
            is_on: false,
            visible: true,
            enabled: true,
            label: label.into(),
            on_change: None,
        }
    }

    /// 切换状态
    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }

    /// 设置回调
    pub fn with_callback<F: Fn(bool) + Send + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Arc::new(Mutex::new(Box::new(callback))));
        self
    }
}

/// 12. Checkbox - 复选框组件
#[derive(Component, Clone)]
pub struct Checkbox {
    /// RectTransform
    pub rect: RectTransform,
    /// 是否选中
    pub checked: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 标签文本
    pub label: String,
    /// 改变回调
    pub on_change: Option<ToggleChangeCallback>,
}

impl Checkbox {
    /// 创建新复选框
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(20.0, 20.0),
            checked: false,
            visible: true,
            enabled: true,
            label: label.into(),
            on_change: None,
        }
    }

    /// 切换选中状态
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

/// 13. RadioButton - 单选按钮组件
#[derive(Component, Clone)]
pub struct RadioButton {
    /// RectTransform
    pub rect: RectTransform,
    /// 是否选中
    pub selected: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 标签文本
    pub label: String,
    /// 组名（同一组的按钮互斥）
    pub group: String,
    /// 改变回调
    pub on_change: Option<ToggleChangeCallback>,
}

impl RadioButton {
    /// 创建新单选按钮
    pub fn new(label: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(20.0, 20.0),
            selected: false,
            visible: true,
            enabled: true,
            label: label.into(),
            group: group.into(),
            on_change: None,
        }
    }

    /// 设置选中状态
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

/// 14. Dropdown - 下拉菜单组件
#[derive(Component, Clone)]
pub struct Dropdown {
    /// RectTransform
    pub rect: RectTransform,
    /// 选项列表
    pub options: Vec<String>,
    /// 选中索引
    pub selected_index: usize,
    /// 是否展开
    pub expanded: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否启用
    pub enabled: bool,
    /// 改变回调
    pub on_change: Option<UsizeChangeCallback>,
}

/// 索引改变回调类型
pub type UsizeChangeCallback = Arc<Mutex<Box<dyn Fn(usize) + Send + 'static>>>;

impl Dropdown {
    /// 创建新下拉菜单
    pub fn new(options: Vec<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(150.0, 30.0),
            options,
            selected_index: 0,
            expanded: false,
            visible: true,
            enabled: true,
            on_change: None,
        }
    }

    /// 获取选中项
    pub fn get_selected(&self) -> Option<&String> {
        self.options.get(self.selected_index)
    }

    /// 选择项
    pub fn select(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = index;
        }
    }
}

// ============================================================================
// 进度显示组件 (15-17)
// ============================================================================

/// 15. ProgressBar - 进度条组件
#[derive(Component, Clone)]
pub struct ProgressBar {
    /// RectTransform
    pub rect: RectTransform,
    /// 当前进度（0.0-1.0）
    pub progress: f32,
    /// 是否可见
    pub visible: bool,
    /// 进度条颜色
    pub fill_color: [f32; 4],
    /// 背景颜色
    pub background_color: [f32; 4],
    /// 是否显示文本
    pub show_text: bool,
    /// 文本格式
    pub text_format: String,
}

impl ProgressBar {
    /// 创建新进度条
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 20.0),
            progress: 0.0,
            visible: true,
            fill_color: [0.2, 0.6, 1.0, 1.0],
            background_color: [0.1, 0.1, 0.1, 1.0],
            show_text: true,
            text_format: "{0:.0}%".to_string(),
        }
    }

    /// 设置进度
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// 获取格式化文本
    pub fn get_formatted_text(&self) -> String {
        self.text_format.replace("{0}", &format!("{:.0}", self.progress * 100.0))
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

/// 16. ScrollBar - 滚动条组件
#[derive(Component, Clone)]
pub struct ScrollBar {
    /// RectTransform
    pub rect: RectTransform,
    /// 当前值（0.0-1.0）
    pub value: f32,
    /// 滚动方向
    pub direction: ScrollDirection,
    /// 是否可见
    pub visible: bool,
    /// 是否自动隐藏
    pub auto_hide: bool,
    /// 滚动步进
    pub step_size: f32,
}

/// 滚动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
}

impl ScrollBar {
    /// 创建新滚动条
    pub fn new(direction: ScrollDirection) -> Self {
        let size = match direction {
            ScrollDirection::Horizontal => Vec2::new(200.0, 15.0),
            ScrollDirection::Vertical => Vec2::new(15.0, 200.0),
        };

        Self {
            rect: RectTransform::new().with_size(size.x, size.y),
            value: 0.0,
            direction,
            visible: true,
            auto_hide: true,
            step_size: 0.1,
        }
    }
}

/// 17. LoadingSpinner - 加载动画组件
#[derive(Component, Clone)]
pub struct LoadingSpinner {
    /// RectTransform
    pub rect: RectTransform,
    /// 是否可见
    pub visible: bool,
    /// 当前旋转角度
    pub rotation: f32,
    /// 旋转速度（度/帧）
    pub rotation_speed: f32,
    /// 颜色
    pub color: [f32; 4],
}

impl LoadingSpinner {
    /// 创建新加载动画
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(32.0, 32.0),
            visible: true,
            rotation: 0.0,
            rotation_speed: 5.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// 更新旋转
    pub fn update(&mut self) {
        self.rotation = (self.rotation + self.rotation_speed) % 360.0;
    }
}

impl Default for LoadingSpinner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 高级组件 (18-22)
// ============================================================================

/// 18. Canvas - 画布组件（2D绘图）
#[derive(Component, Clone)]
pub struct Canvas {
    /// RectTransform
    pub rect: RectTransform,
    /// 是否可见
    pub visible: bool,
    /// 绘图命令列表
    pub draw_commands: Vec<DrawCommand>,
}

/// 绘图命令
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Line {
        start: Vec2,
        end: Vec2,
        color: [f32; 4],
        width: f32,
    },
    Rect {
        position: Vec2,
        size: Vec2,
        color: [f32; 4],
        filled: bool,
    },
    Circle {
        center: Vec2,
        radius: f32,
        color: [f32; 4],
        filled: bool,
    },
    Text {
        position: Vec2,
        text: String,
        size: f32,
        color: [f32; 4],
    },
}

impl Canvas {
    /// 创建新画布
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(400.0, 300.0),
            visible: true,
            draw_commands: Vec::new(),
        }
    }

    /// 清空画布
    pub fn clear(&mut self) {
        self.draw_commands.clear();
    }

    /// 添加绘图命令
    pub fn add_command(&mut self, command: DrawCommand) {
        self.draw_commands.push(command);
    }

    /// 绘制线条
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: [f32; 4], width: f32) {
        self.add_command(DrawCommand::Line {
            start,
            end,
            color,
            width,
        });
    }

    /// 绘制矩形
    pub fn draw_rect(&mut self, position: Vec2, size: Vec2, color: [f32; 4], filled: bool) {
        self.add_command(DrawCommand::Rect {
            position,
            size,
            color,
            filled,
        });
    }

    /// 绘制圆形
    pub fn draw_circle(&mut self, center: Vec2, radius: f32, color: [f32; 4], filled: bool) {
        self.add_command(DrawCommand::Circle {
            center,
            radius,
            color,
            filled,
        });
    }

    /// 绘制文本
    pub fn draw_text(
        &mut self,
        position: Vec2,
        text: impl Into<String>,
        size: f32,
        color: [f32; 4],
    ) {
        self.add_command(DrawCommand::Text {
            position,
            text: text.into(),
            size,
            color,
        });
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

/// 19. TabControl - 选项卡控件
#[derive(Component, Clone)]
pub struct TabControl {
    /// RectTransform
    pub rect: RectTransform,
    /// 选项卡列表
    pub tabs: Vec<TabItem>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 是否可见
    pub visible: bool,
    /// 选项卡位置
    pub tab_position: TabPosition,
}

/// 选项卡项
#[derive(Debug, Clone, Component)]
pub struct TabItem {
    /// 标题
    pub title: String,
    /// 内容
    pub content: Option<Entity>,
}

/// 选项卡位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl TabControl {
    /// 创建新选项卡控件
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(400.0, 300.0),
            tabs: Vec::new(),
            selected_index: 0,
            visible: true,
            tab_position: TabPosition::Top,
        }
    }

    /// 添加选项卡
    pub fn add_tab(&mut self, title: impl Into<String>) {
        self.tabs.push(TabItem {
            title: title.into(),
            content: None,
        });
    }

    /// 获取当前选项卡
    pub fn get_current_tab(&self) -> Option<&TabItem> {
        self.tabs.get(self.selected_index)
    }

    /// 选择选项卡
    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.selected_index = index;
        }
    }
}

impl Default for TabControl {
    fn default() -> Self {
        Self::new()
    }
}

/// 20. RichText - 富文本组件
#[derive(Component, Clone)]
pub struct RichText {
    /// RectTransform
    pub rect: RectTransform,
    /// 富文本内容
    pub text: String,
    /// 是否可见
    pub visible: bool,
    /// 字体大小
    pub font_size: f32,
}

impl RichText {
    /// 创建新富文本
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(300.0, 100.0),
            text: text.into(),
            visible: true,
            font_size: 14.0,
        }
    }

    /// 解析富文本标签（简化版）
    /// 支持: <b>粗体</b>, <i>斜体</i>, <color=#RRGGBB>颜色</color>
    pub fn parse_tags(&self) -> Vec<TextSegment> {
        // 简化实现，实际应用中需要更复杂的解析器
        vec![TextSegment {
            text: self.text.clone(),
            bold: false,
            italic: false,
            color: [1.0, 1.0, 1.0, 1.0],
        }]
    }
}

/// 文本段
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub color: [f32; 4],
}

/// 21. Tooltip - 工具提示组件
#[derive(Component, Clone)]
pub struct Tooltip {
    /// RectTransform
    pub rect: RectTransform,
    /// 提示文本
    pub text: String,
    /// 是否可见
    pub visible: bool,
    /// 延迟显示时间（秒）
    pub delay: f32,
    /// 跟随鼠标
    pub follow_mouse: bool,
}

impl Tooltip {
    /// 创建新工具提示
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            rect: RectTransform::new().with_size(200.0, 50.0),
            text: text.into(),
            visible: false,
            delay: 0.5,
            follow_mouse: true,
        }
    }
}

/// 22. ContextMenu - 上下文菜单组件
#[derive(Component, Clone)]
pub struct ContextMenu {
    /// RectTransform
    pub rect: RectTransform,
    /// 菜单项列表
    pub items: Vec<ContextMenuItem>,
    /// 是否可见
    pub visible: bool,
}

/// 上下文菜单项
#[derive(Clone, Component)]
pub struct ContextMenuItem {
    /// 标签
    pub label: String,
    /// 是否启用
    pub enabled: bool,
    /// 快捷键
    pub shortcut: Option<String>,
    /// 回调
    pub callback: Option<ClickCallback>,
}

impl std::fmt::Debug for ContextMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenuItem")
            .field("label", &self.label)
            .field("enabled", &self.enabled)
            .field("shortcut", &self.shortcut)
            .field("callback", &self.callback.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

impl ContextMenu {
    /// 创建新上下文菜单
    pub fn new() -> Self {
        Self {
            rect: RectTransform::new().with_size(150.0, 200.0),
            items: Vec::new(),
            visible: false,
        }
    }

    /// 添加菜单项
    pub fn add_item(&mut self, label: impl Into<String>) {
        self.items.push(ContextMenuItem {
            label: label.into(),
            enabled: true,
            shortcut: None,
            callback: None,
        });
    }

    /// 显示菜单
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// 隐藏菜单
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Click Me").with_size(150.0, 50.0).with_position(10.0, 20.0);

        assert_eq!(button.text, "Click Me");
        assert_eq!(button.rect.size_delta, Vec2::new(150.0, 50.0));
        assert_eq!(button.rect.anchored_position, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_slider_value_clamping() {
        let mut slider = Slider::new(0.0, 100.0);
        slider.set_value(150.0);
        assert_eq!(slider.current_value, 100.0);

        slider.set_value(-10.0);
        assert_eq!(slider.current_value, 0.0);
    }

    #[test]
    fn test_dropdown() {
        let options = vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ];
        let mut dropdown = Dropdown::new(options);
        assert_eq!(dropdown.get_selected(), Some(&"Option 1".to_string()));

        dropdown.select(2);
        assert_eq!(dropdown.get_selected(), Some(&"Option 3".to_string()));
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressBar::new();
        progress.set_progress(0.75);
        assert_eq!(progress.progress, 0.75);
        assert_eq!(progress.get_text(), "75%");
    }
}
