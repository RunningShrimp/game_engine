# UI系统完整指南

**版本**: 1.0.0
**更新日期**: 2025-01-01
**作者**: Game Engine Team

## 目录

1. [概述](#概述)
2. [快速开始](#快速开始)
3. [组件参考](#组件参考)
4. [布局系统](#布局系统)
5. [事件系统](#事件系统)
6. [主题系统](#主题系统)
7. [最佳实践](#最佳实践)
8. [性能优化](#性能优化)

---

## 概述

游戏引擎的UI系统提供了完整的游戏运行时UI框架，包括：

### 核心特性

- ✅ **22个基础组件** - Button, Label, TextField, Panel, Slider等
- ✅ **灵活布局系统** - Absolute/Horizontal/Vertical/Grid布局
- ✅ **完整事件系统** - 点击、悬停、拖拽、滚动等
- ✅ **主题定制** - 浅色/深色主题，自定义颜色和样式
- ✅ **RectTransform** - 锚点、Pivot、旋转、缩放支持

### 架构

```
UI系统架构
├── Framework (mod.rs) - 核心类型和UIService
├── Widgets (widgets.rs) - 22个UI组件
├── Layout (layout.rs) - RectTransform和布局算法
├── Events (events.rs) - 事件管理器和点击检测
└── Theme (theme.rs) - 主题和样式系统
```

---

## 快速开始

### 创建第一个UI

```rust
use bevy_ecs::prelude::*;
use game_engine::ui::widgets::*;
use game_engine::ui::theme::Theme;

fn main() {
    // 1. 创建按钮
    let button = Button::new("Click Me!")
        .with_size(150.0, 50.0)
        .with_position(100.0, 100.0)
        .with_callback(|| {
            println!("Button clicked!");
        });

    // 2. 创建标签
    let label = Label::new("Hello, UI!")
        .with_font_size(24.0)
        .with_color(1.0, 1.0, 1.0, 1.0);

    // 3. 应用主题
    let theme = Theme::dark();

    // 4. 渲染UI
    render_ui(&button, &label, &theme);
}
```

### 5分钟创建登录UI

```rust
fn create_login_ui() {
    // 面板容器
    let panel = Panel::new()
        .with_background(0.15, 0.15, 0.15, 0.95);

    // 标题
    let title = Label::new("Welcome Back!")
        .with_font_size(24.0)
        .with_alignment(TextAlignment::Center);

    // 用户名输入
    let username = TextField::new("Username")
        .with_max_length(30);

    // 密码输入
    let password = TextField::new("Password")
        .with_max_length(50);

    // 登录按钮
    let login_btn = Button::new("Login")
        .with_callback(|| {
            authenticate_user();
        });

    // 记住我
    let remember = Checkbox::new("Remember me");

    // 组装UI
    // （在实际引擎中，这里会创建Entity并添加Component）
}
```

---

## 组件参考

### 基础组件 (1-5)

#### 1. Button - 按钮

```rust
let button = Button::new("Click Me")
    .with_size(150.0, 50.0)      // 设置大小
    .with_position(10.0, 20.0)     // 设置位置
    .with_callback(|| {            // 设置回调
        println!("Clicked!");
    });

// 属性
button.text          // 按钮文本
button.pressed       // 是否按下
button.hovered       // 是否悬停
button.enabled       // 是否启用
```

#### 2. Label - 标签

```rust
let label = Label::new("Hello")
    .with_font_size(16.0)                      // 字体大小
    .with_color(1.0, 1.0, 1.0, 1.0)           // RGBA颜色
    .with_alignment(TextAlignment::Center);    // 对齐方式

// 对齐选项
TextAlignment::Left     // 左对齐
TextAlignment::Center   // 居中
TextAlignment::Right    // 右对齐
```

#### 3. TextField - 单行输入框

```rust
let input = TextField::new("Enter text...")
    .with_max_length(50)              // 最大长度
    .with_callback(|text| {           // 值改变回调
        println!("Input: {}", text);
    });

// 方法
input.insert_text("hello");  // 插入文本
input.delete_char();         // 删除字符
input.focused                // 是否获得焦点
```

#### 4. TextArea - 多行输入框

```rust
let textarea = TextArea::new("Enter message...")
    .with_lines(5);    // 行数
```

#### 5. Image - 图像

```rust
let image = Image::new()
    .with_texture(123)                    // 纹理ID
    .with_color(1.0, 1.0, 1.0, 1.0)       // 颜色调制

// 图像模式
ImageMode::Simple   // 简单拉伸
ImageMode::Sliced   // 九宫格切片
ImageMode::Tiled    // 平铺
ImageMode::Filled   // 填充
```

### 容器组件 (6-9)

#### 6. Panel - 面板

```rust
let panel = Panel::new()
    .with_background(0.2, 0.2, 0.2, 1.0);  // 背景色

panel.add_child(child_entity);  // 添加子组件
panel.children                  // 子组件列表
```

#### 7. ScrollView - 滚动视图

```rust
let scroll = ScrollView::new();
scroll.scroll_to(0.0, 100.0);  // 滚动到位置

// 属性
scroll.content_size       // 内容大小
scroll.scroll_position    // 滚动位置
scroll.show_scrollbar     // 显示滚动条
```

#### 8. ListView - 列表视图

```rust
let mut list = ListView::new();
list.add_item("Item 1");
list.add_item("Item 2");

let selected = list.get_selected();  // 获取选中项
list.selected_index                 // 选中索引
```

#### 9. GridView - 网格视图

```rust
let mut grid = GridView::new(3);  // 3列
grid.add_item("Grid 1");
grid.add_item("Grid 2");

// 属性
grid.columns       // 列数
grid.cell_size     // 单元格大小
grid.spacing       // 间距
```

### 控制组件 (10-14)

#### 10. Slider - 滑块

```rust
let mut slider = Slider::new(0.0, 100.0);  // 最小值, 最大值
slider.set_value(50.0);                   // 设置值

// 回调
slider.with_callback(|value| {
    println!("Value: {}", value);
});

// 属性
slider.current_value   // 当前值
slider.get_progress()  // 进度(0.0-1.0)
slider.whole_numbers   // 整数模式
```

#### 11. Toggle - 开关

```rust
let toggle = Toggle::new("Enable Feature")
    .with_callback(|is_on| {
        println!("Toggle: {}", is_on);
    });

toggle.toggle();  // 切换状态
```

#### 12. Checkbox - 复选框

```rust
let checkbox = Checkbox::new("Accept Terms")
    .with_callback(|checked| {
        println!("Checked: {}", checked);
    });

checkbox.toggle();  // 切换选中
checkbox.checked;   // 是否选中
```

#### 13. RadioButton - 单选按钮

```rust
let radio1 = RadioButton::new("Option A", "group1");
let radio2 = RadioButton::new("Option B", "group1");

// 同一组内的按钮互斥
radio1.group  // 组名
radio1.selected  // 是否选中
```

#### 14. Dropdown - 下拉菜单

```rust
let options = vec![
    "Option 1".to_string(),
    "Option 2".to_string(),
    "Option 3".to_string(),
];

let mut dropdown = Dropdown::new(options);
dropdown.select(1);  // 选择索引1
let selected = dropdown.get_selected();  // 获取选中项
```

### 进度显示组件 (15-17)

#### 15. ProgressBar - 进度条

```rust
let mut progress = ProgressBar::new();
progress.set_progress(0.75);  // 设置进度(0.0-1.0)

let text = progress.get_text();  // 获取文本 "75%"

// 属性
progress.fill_color       // 填充颜色
progress.background_color  // 背景颜色
progress.show_text        // 显示文本
```

#### 16. ScrollBar - 滚动条

```rust
let scrollbar_v = ScrollBar::new(ScrollDirection::Vertical);
let scrollbar_h = ScrollBar::new(ScrollDirection::Horizontal);

// 属性
scrollbar_v.value         // 当前值(0.0-1.0)
scrollbar_v.auto_hide     // 自动隐藏
scrollbar_v.step_size     // 滚动步进
```

#### 17. LoadingSpinner - 加载动画

```rust
let mut spinner = LoadingSpinner::new();
spinner.update();  // 更新动画

spinner.rotation  // 旋转角度
spinner.rotation_speed  // 旋转速度
```

### 高级组件 (18-22)

#### 18. Canvas - 画布

```rust
let mut canvas = Canvas::new();

use game_engine::ui::widgets::DrawCommand;

// 添加绘图命令
canvas.add_command(DrawCommand::Line {
    start: Vec2::new(10.0, 10.0),
    end: Vec2::new(100.0, 100.0),
    color: [1.0, 0.0, 0.0, 1.0],
    width: 2.0,
});

canvas.add_command(DrawCommand::Circle {
    center: Vec2::new(50.0, 50.0),
    radius: 25.0,
    color: [0.0, 0.5, 1.0, 1.0],
    filled: true,
});

canvas.clear();  // 清空画布
```

#### 19. TabControl - 选项卡

```rust
let mut tabs = TabControl::new();
tabs.add_tab("Tab 1");
tabs.add_tab("Tab 2");
tabs.add_tab("Tab 3");

tabs.selected_index  // 当前选中索引
tabs.get_current_tab()  // 获取当前选项卡

// 选项卡位置
TabPosition::Top
TabPosition::Bottom
TabPosition::Left
TabPosition::Right
```

#### 20. RichText - 富文本

```rust
let rich_text = RichText::new(
    "<b>Bold</b> and <i>italic</i> text"
);

// 解析标签（简化版）
let segments = rich_text.parse_tags();
```

#### 21. Tooltip - 工具提示

```rust
let tooltip = Tooltip::new("Helpful tooltip")
    .with_delay(0.5);  // 延迟显示

tooltip.visible  // 是否可见
tooltip.follow_mouse  // 跟随鼠标
```

#### 22. ContextMenu - 上下文菜单

```rust
let mut menu = ContextMenu::new();
menu.add_item("Cut");
menu.add_item("Copy");
menu.add_item("Paste");

menu.items  // 菜单项列表
menu.visible  // 是否可见
```

---

## 布局系统

### RectTransform

RectTransform是所有UI组件的核心，提供位置、大小、锚点、旋转和缩放。

```rust
use game_engine::ui::layout::RectTransform;

let rect = RectTransform::new()
    .with_position(50.0, 50.0)   // 位置偏移
    .with_size(200.0, 100.0)     // 大小
    .with_anchors(0.0, 0.0, 1.0, 1.0);  // 锚点(min_x, min_y, max_x, max_y)

// 预设位置
rect.set_top_left();   // 左上角
rect.set_center();     // 中心
rect.set_bottom_right(); // 右下角
rect.set_stretch();    // 拉伸填充

// 属性
rect.anchored_position  // 位置偏移
rect.size_delta         // 大小
rect.anchor_min         // 锚点最小值
rect.anchor_max         // 锚点最大值
rect.pivot              // Pivot点(0-1)
rect.rotation           // 旋转角度
rect.scale              // 缩放

// 计算世界位置和大小
let parent_size = Vec2::new(800.0, 600.0);
let world_pos = rect.world_position(parent_size);
let world_size = rect.world_size(parent_size);
```

### 布局算法

#### Absolute Layout - 绝对布局

```rust
use game_engine::ui::layout::AbsoluteLayout;

let layout = AbsoluteLayout;
// 子组件保持绝对位置，不自动排列
```

#### Horizontal Layout - 水平布局

```rust
use game_engine::ui::layout::HorizontalLayout;

let layout = HorizontalLayout {
    spacing: 10.0,    // 间距
    padding: 10.0,    // 内边距
};

layout.calculate(&mut children, parent_size);
```

#### Vertical Layout - 垂直布局

```rust
use game_engine::ui::layout::VerticalLayout;

let layout = VerticalLayout {
    spacing: 10.0,    // 间距
    padding: 10.0,    // 内边距
};

layout.calculate(&mut children, parent_size);
```

#### Grid Layout - 网格布局

```rust
use game_engine::ui::layout::GridLayout;

let layout = GridLayout {
    columns: 3,              // 列数
    row_spacing: 10.0,       // 行间距
    column_spacing: 10.0,    // 列间距
    cell_size: Vec2::new(100.0, 100.0),  // 单元格大小
};

layout.calculate(&mut children, parent_size);
```

### 布局示例

```rust
// 创建水平布局的按钮组
let mut button_group = vec![];
for i in 0..3 {
    let rect = RectTransform::new()
        .with_size(80.0, 30.0);
    let entity = commands.spawn().id();
    button_group.push((entity, rect));
}

let layout = HorizontalLayout {
    spacing: 10.0,
    padding: 20.0,
};
layout.calculate(&mut button_group, Vec2::new(400.0, 50.0));
```

---

## 事件系统

### UI事件类型

```rust
use game_engine::ui::events::{UIEventManager, UIEvent, EventListener};

// 创建事件管理器
let mut event_manager = UIEventManager::new();

// 添加事件监听器
let component_id = ComponentId::new();
event_manager.add_listener(component_id, EventListener {
    event_type: "click".to_string(),
    callback: Box::new(|event| {
        println!("Event: {:?}", event);
        return true;  // 返回true表示事件已处理
    }),
});

// 发送事件
event_manager.send_event(UIEvent::MouseClick {
    position: Vec2::new(100.0, 200.0),
    button: MouseButton::Left,
});

// 处理事件队列
event_manager.process_events();
```

### 事件类型列表

| 事件 | 描述 | 数据 |
|------|------|------|
| `MouseClick` | 鼠标点击 | position, button |
| `MouseRelease` | 鼠标释放 | position, button |
| `MouseMove` | 鼠标移动 | position, delta |
| `MouseScroll` | 鼠标滚轮 | delta |
| `KeyDown` | 键盘按下 | key, code |
| `KeyUp` | 键盘释放 | key, code |
| `Char` | 字符输入 | char |
| `FocusGained` | 获得焦点 | - |
| `FocusLost` | 失去焦点 | - |
| `ValueChanged` | 值改变 | new_value |
| `Custom` | 自定义事件 | event_type, data |

### 点击检测

```rust
use game_engine::ui::events::HitTester;

let mut hit_tester = HitTester::new();

// 注册组件点击区域
hit_tester.register(component_id, position, size);

// 测试点击
if let Some(hit_id) = hit_tester.test_click(mouse_pos) {
    println!("Clicked component: {:?}", hit_id);
}

// 测试悬停
if let Some(hovered_id) = hit_tester.test_hover(mouse_pos) {
    event_manager.set_hovered(hovered_id);
}
```

### 事件传播

UI事件支持三个阶段：

1. **捕获阶段** (Capture) - 从根到目标
2. **目标阶段** (Target) - 在目标组件
3. **冒泡阶段** (Bubbling) - 从目标到根

```rust
use game_engine::ui::events::EventPhase;

// 在事件处理中检查阶段
match phase {
    EventPhase::Capture => { /* 处理捕获 */ },
    EventPhase::Target => { /* 处理目标 */ },
    EventPhase::Bubbling => { /* 处理冒泡 */ },
}
```

---

## 主题系统

### 内置主题

```rust
use game_engine::ui::theme::Theme;

// 默认主题
let default = Theme::default();

// 浅色主题
let light = Theme::light();

// 深色主题
let dark = Theme::dark();
```

### 主题结构

```rust
pub struct Theme {
    pub colors: ColorScheme,    // 颜色方案
    pub fonts: FontScheme,      // 字体方案
    pub styles: StyleScheme,    // 样式方案
}
```

#### ColorScheme - 颜色方案

```rust
pub struct ColorScheme {
    pub primary: UIColor,      // 主色调
    pub secondary: UIColor,    // 次要色调
    pub success: UIColor,      // 成功色
    pub warning: UIColor,      // 警告色
    pub error: UIColor,        // 错误色
    pub info: UIColor,         // 信息色
    pub background: UIColor,   // 背景色
    pub surface: UIColor,      // 表面色
}
```

#### FontScheme - 字体方案

```rust
pub struct FontScheme {
    pub family: String,        // 字体家族
    pub sizes: FontSizes,      // 字体大小
    pub weights: FontWeights,  // 字体粗细
}

pub struct FontSizes {
    pub tiny: f32,      // 10.0
    pub small: f32,     // 12.0
    pub normal: f32,    // 14.0
    pub medium: f32,    // 16.0
    pub large: f32,     // 20.0
    pub huge: f32,      // 24.0
}
```

#### StyleScheme - 样式方案

```rust
pub struct StyleScheme {
    pub border_radius: f32,        // 4.0
    pub border_width: f32,         // 1.0
    pub shadow_offset: f32,        // 2.0
    pub shadow_blur: f32,          // 4.0
    pub shadow_color: UIColor,     // 阴影颜色
    pub transition_duration: f32,  // 0.2秒
}
```

### 自定义主题

```rust
let custom_theme = Theme {
    colors: ColorScheme {
        primary: UIColor::rgb(0.9, 0.3, 0.3),   // 红色
        secondary: UIColor::rgb(0.3, 0.9, 0.3), // 绿色
        background: UIColor::rgb(0.05, 0.05, 0.05),
        ..Default::default()
    },
    fonts: FontScheme {
        family: "Roboto".to_string(),
        ..Default::default()
    },
    styles: StyleScheme {
        border_radius: 8.0,
        ..Default::default()
    },
};
```

### 组件样式

```rust
use game_engine::ui::theme::UIStyle;

// 按钮样式（根据状态）
let button_style = UIStyle::button(&theme, hovered, pressed);

// 输入框样式（根据焦点状态）
let input_style = UIStyle::input(&theme, focused);
```

---

## 最佳实践

### 1. 组件层次结构

```rust
// 好的层次结构
Panel (Root)
├─ Header (Panel)
│  ├─ Logo (Image)
│  └─ Title (Label)
├─ Content (ScrollView)
│  └─ Items (ListView)
└─ Footer (Panel)
   └─ Buttons (Horizontal Layout)
      ├─ Cancel (Button)
      └─ Confirm (Button)

// 避免过深的嵌套（<5层）
```

### 2. 响应式设计

```rust
// 使用锚点实现响应式布局
let mut panel = Panel::new();
panel.rect.set_stretch();  // 自动填充父容器

// 使用相对位置
let child_rect = RectTransform::new()
    .with_anchors(0.5, 0.5, 0.5, 0.5)  // 中心锚点
    .with_position(0.0, 0.0);             // 相对中心
```

### 3. 事件处理优化

```rust
// 使用事件捕获避免事件冒泡
event_manager.set_capture(component_id);

// 事件处理完成后释放
event_manager.release_capture();

// 在回调中返回true阻止事件传播
callback: Box::new(|_event| {
    // 处理事件...
    return true;  // 阻止传播
})
```

### 4. 性能优化

```rust
// 1. 减少组件数量
// 避免创建不必要的组件

// 2. 使用对象池
// 对于列表项等重复组件，使用对象池

// 3. 延迟加载
// 只在需要时创建复杂组件

// 4. 批量更新
// 收集多个更新后一次性应用
```

### 5. 可访问性

```rust
// 为组件添加tooltip
let button = Button::new("Delete")
    .with_callback(|| delete_item());

let tooltip = Tooltip::new("Permanently delete this item");

// 使用清晰的标签
let checkbox = Checkbox::new("I agree to the terms and conditions");

// 提供键盘快捷键
// 在KeyEvent中监听快捷键
```

---

## 性能优化

### 性能指标

| 指标 | 目标 | 测量方法 |
|------|------|----------|
| UI渲染时间 | <16ms (60fps) | Tracy profiler |
| 组件数量 | <1000 | UI debugger |
| 事件延迟 | <50ms | 事件日志 |
| 内存占用 | <10MB | 内存分析器 |

### 优化策略

#### 1. 减少Draw Calls

```rust
// 使用图集(Texture Atlas)
// 合并相邻的相同类型组件
// 批量渲染相同材质的组件
```

#### 2. 布局缓存

```rust
// 只在组件改变时重新计算布局
// 使用dirty标志
if component.is_layout_dirty() {
    recalculate_layout(component);
    component.clear_dirty_flag();
}
```

#### 3. 事件节流

```rust
// 对高频事件(如MouseMove)进行节流
const THROTTLE_MS: u64 = 16;  // 60fps

if last_event_time + THROTTLE_MS < current_time {
    process_event();
    last_event_time = current_time;
}
```

#### 4. 惰性加载

```rust
// 只在可见时创建子组件
if scroll_view.is_visible(area) {
    create_lazy_components(area);
}
```

---

## 示例项目

### 完整示例：游戏HUD

```rust
fn create_game_hud() {
    // 主HUD面板
    let hud_panel = Panel::new()
        .with_background(0.0, 0.0, 0.0, 0.0);  // 透明背景

    // 生命条
    let health_bar = ProgressBar::new();
    health_bar.set_progress(0.8);

    // 魔法条
    let mana_bar = ProgressBar::new();
    mana_bar.set_progress(0.6);

    // 经验条
    let exp_bar = ProgressBar::new();
    exp_bar.set_progress(0.45);

    // 等级标签
    let level_label = Label::new("Level 42")
        .with_color(1.0, 0.8, 0.0, 1.0);

    // 技能栏
    let mut skill_bar = GridView::new(5);
    for i in 1..=5 {
        skill_bar.add_item(format!("Skill {}", i));
    }

    // 小地图
    let minimap = Image::new()
        .with_texture(MINIMAP_TEXTURE);
}
```

---

## 故障排除

### 问题1: 点击检测不工作

**症状**: 点击按钮没有反应

**解决方案**:
```rust
// 1. 检查组件是否可见
if !button.visible || !button.enabled {
    return;
}

// 2. 检查点击区域
let bounds = button.rect.get_bounds(parent_size);
println!("Bounds: {:?} to {:?}", bounds.0, bounds.1);

// 3. 检查事件监听器
event_manager.add_listener(button_id, listener);
```

### 问题2: 布局不正确

**症状**: 组件位置错误或重叠

**解决方案**:
```rust
// 1. 检查锚点
println!("Anchors: {:?} to {:?}",
    rect.anchor_min, rect.anchor_max);

// 2. 检查父组件大小
println!("Parent size: {:?}", parent_size);

// 3. 手动重新计算布局
layout.calculate(&mut children, parent_size);
```

### 问题3: 性能问题

**症状**: UI渲染卡顿

**解决方案**:
```rust
// 1. 使用Profiler
use game_engine::profiling::Profiler;
let _guard = Profiler::start("UI Render");

// 2. 减少组件数量
// 3. 启用布局缓存
// 4. 优化事件处理
```

---

## 参考资料

### 源代码
- `src/ui/mod.rs` - UI框架核心
- `src/ui/widgets.rs` - 22个UI组件
- `src/ui/layout.rs` - RectTransform和布局算法
- `src/ui/events.rs` - 事件系统
- `src/ui/theme.rs` - 主题系统

### 示例代码
- `examples/ui_demo.rs` - 完整UI示例
- `examples/ai_examples.rs` - AI行为树示例
- `examples/npc_presets/` - NPC预设示例

### 相关文档
- [NPC/AI Guide](./NPC_AI_GUIDE.md) - NPC和AI系统
- [API Reference](./api_reference.md) - API参考
- [Best Practices](./best_practices.md) - 最佳实践

---

**更新日志**:

**v1.0.0** (2025-01-01)
- ✅ 22个基础UI组件
- ✅ 完整布局系统 (Absolute/Horizontal/Vertical/Grid)
- ✅ 事件系统 (点击/悬停/拖拽/滚动等)
- ✅ 主题系统 (浅色/深色/自定义)
- ✅ 完整示例和文档

---

**维护者**: Game Engine Team
**许可证**: MIT
**反馈**: GitHub Issues
