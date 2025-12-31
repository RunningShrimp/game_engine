//! UI系统使用示例
//!
//! 演示游戏引擎的22个UI组件的使用方法

use bevy_ecs::prelude::*;
use game_engine::ui::widgets::*;
use game_engine::ui::theme::{Theme, UIColor};
use game_engine::ui::layout::{RectTransform, HorizontalLayout, VerticalLayout, GridLayout};

fn main() {
    println!("=== 游戏引擎UI系统示例 ===\n");

    // 示例1: 基础组件
    example_1_basic_widgets();

    // 示例2: 容器组件
    example_2_container_widgets();

    // 示例3: 控制组件
    example_3_control_widgets();

    // 示例4: 进度显示组件
    example_4_progress_widgets();

    // 示例5: 高级组件
    example_5_advanced_widgets();

    // 示例6: 主题系统
    example_6_themes();

    // 示例7: 布局系统
    example_7_layouts();

    // 示例8: 事件处理
    example_8_events();
}

/// 示例1: 基础组件 (Button, Label, TextField, TextArea, Image)
fn example_1_basic_widgets() {
    println!("=== 示例1: 基础组件 ===\n");

    // 1. Button - 按钮组件
    let button = Button::new("Click Me")
        .with_size(150.0, 50.0)
        .with_position(100.0, 100.0)
        .with_callback(|| {
            println!("Button clicked!");
        });
    println!("✓ Button: '{}' at ({}, {})",
        button.text,
        button.rect.anchored_position.x,
        button.rect.anchored_position.y
    );

    // 2. Label - 文本标签组件
    let label = Label::new("Hello, World!")
        .with_font_size(20.0)
        .with_color(1.0, 1.0, 1.0, 1.0)
        .with_alignment(game_engine::ui::widgets::TextAlignment::Center);
    println!("✓ Label: '{}' (size: {})", label.text, label.font_size);

    // 3. TextField - 单行文本输入框
    let text_field = TextField::new("Enter text...")
        .with_max_length(50)
        .with_callback(|text| {
            println!("Text changed: {}", text);
        });
    println!("✓ TextField: placeholder='{}' (max_length: {:?})",
        text_field.placeholder, text_field.max_length
    );

    // 4. TextArea - 多行文本输入框
    let text_area = TextArea::new("Enter multiple lines...")
        .with_lines(5);
    println!("✓ TextArea: lines={}", text_area.lines);

    // 5. Image - 图像组件
    let image = Image::new()
        .with_texture(123)
        .with_color(1.0, 1.0, 1.0, 1.0);
    println!("✓ Image: texture_id={:?}", image.texture_id);

    println!();
}

/// 示例2: 容器组件 (Panel, ScrollView, ListView, GridView)
fn example_2_container_widgets() {
    println!("=== 示例2: 容器组件 ===\n");

    // 6. Panel - 面板容器
    let panel = Panel::new()
        .with_background(0.2, 0.2, 0.2, 1.0);
    println!("✓ Panel: background_color=[{:.1}, {:.1}, {:.1}, {:.1}]",
        panel.background_color[0], panel.background_color[1],
        panel.background_color[2], panel.background_color[3]
    );

    // 7. ScrollView - 滚动视图
    let scroll_view = ScrollView::new();
    println!("✓ ScrollView: content_size={:.0}x{:.0}, scroll_position={:.0},{:.0}",
        scroll_view.content_size.x, scroll_view.content_size.y,
        scroll_view.scroll_position.x, scroll_view.scroll_position.y
    );

    // 8. ListView - 列表视图
    let mut list_view = ListView::new();
    list_view.add_item("Item 1");
    list_view.add_item("Item 2");
    list_view.add_item("Item 3");
    println!("✓ ListView: {} items", list_view.items.len());

    // 9. GridView - 网格视图
    let mut grid_view = GridView::new(3);
    grid_view.add_item("Grid 1");
    grid_view.add_item("Grid 2");
    grid_view.add_item("Grid 3");
    grid_view.add_item("Grid 4");
    println!("✓ GridView: {} columns, {} items",
        grid_view.columns, grid_view.items.len()
    );

    println!();
}

/// 示例3: 控制组件 (Slider, Toggle, Checkbox, RadioButton, Dropdown)
fn example_3_control_widgets() {
    println!("=== 示例3: 控制组件 ===\n");

    // 10. Slider - 滑块组件
    let mut slider = Slider::new(0.0, 100.0);
    slider.set_value(75.0);
    println!("✓ Slider: range=[{:.1}, {:.1}], value={:.1}, progress={:.1}%",
        slider.min_value, slider.max_value,
        slider.current_value, slider.get_progress() * 100.0
    );

    // 11. Toggle - 开关组件
    let toggle = Toggle::new("Enable Feature");
    println!("✓ Toggle: label='{}', is_on={}", toggle.label, toggle.is_on);

    // 12. Checkbox - 复选框组件
    let checkbox = Checkbox::new("Accept Terms");
    println!("✓ Checkbox: label='{}', checked={}", checkbox.label, checkbox.checked);

    // 13. RadioButton - 单选按钮组件
    let radio1 = RadioButton::new("Option A", "group1");
    let radio2 = RadioButton::new("Option B", "group1");
    println!("✓ RadioButton: two buttons in group='{}'", radio1.group);

    // 14. Dropdown - 下拉菜单组件
    let options = vec![
        "Option 1".to_string(),
        "Option 2".to_string(),
        "Option 3".to_string(),
    ];
    let mut dropdown = Dropdown::new(options);
    dropdown.select(1);
    println!("✓ Dropdown: {} options, selected='{}'",
        dropdown.options.len(),
        dropdown.get_selected().unwrap_or(&"(none)".to_string())
    );

    println!();
}

/// 示例4: 进度显示组件 (ProgressBar, ScrollBar, LoadingSpinner)
fn example_4_progress_widgets() {
    println!("=== 示例4: 进度显示组件 ===\n");

    // 15. ProgressBar - 进度条组件
    let mut progress_bar = ProgressBar::new();
    progress_bar.set_progress(0.75);
    println!("✓ ProgressBar: progress={:.1}%, text='{}'",
        progress_bar.progress * 100.0, progress_bar.get_text()
    );

    // 16. ScrollBar - 滚动条组件
    let scroll_bar_v = ScrollBar::new(game_engine::ui::widgets::ScrollDirection::Vertical);
    let scroll_bar_h = ScrollBar::new(game_engine::ui::widgets::ScrollDirection::Horizontal);
    println!("✓ ScrollBar: Vertical size={:.0}x{:.0}, Horizontal size={:.0}x{:.0}",
        scroll_bar_v.rect.size_delta.x, scroll_bar_v.rect.size_delta.y,
        scroll_bar_h.rect.size_delta.x, scroll_bar_h.rect.size_delta.y
    );

    // 17. LoadingSpinner - 加载动画组件
    let mut spinner = LoadingSpinner::new();
    spinner.update();
    println!("✓ LoadingSpinner: rotation={:.1}°", spinner.rotation);

    println!();
}

/// 示例5: 高级组件 (Canvas, TabControl, RichText, Tooltip, ContextMenu)
fn example_5_advanced_widgets() {
    println!("=== 示例5: 高级组件 ===\n");

    // 18. Canvas - 画布组件
    let mut canvas = Canvas::new();
    use game_engine::ui::widgets::DrawCommand;
    canvas.add_command(DrawCommand::Line {
        start: glam::Vec2::new(10.0, 10.0),
        end: glam::Vec2::new(100.0, 100.0),
        color: [1.0, 0.0, 0.0, 1.0],
        width: 2.0,
    });
    println!("✓ Canvas: {} draw commands", canvas.draw_commands.len());

    // 19. TabControl - 选项卡控件
    let mut tab_control = TabControl::new();
    tab_control.add_tab("Tab 1");
    tab_control.add_tab("Tab 2");
    tab_control.add_tab("Tab 3");
    println!("✓ TabControl: {} tabs", tab_control.tabs.len());

    // 20. RichText - 富文本组件
    let rich_text = RichText::new("<b>Bold</b> and <i>italic</i> text");
    println!("✓ RichText: '{}'", rich_text.text);

    // 21. Tooltip - 工具提示组件
    let tooltip = Tooltip::new("This is a helpful tooltip");
    println!("✓ Tooltip: '{}', delay={}s", tooltip.text, tooltip.delay);

    // 22. ContextMenu - 上下文菜单组件
    let mut context_menu = ContextMenu::new();
    context_menu.add_item("Cut");
    context_menu.add_item("Copy");
    context_menu.add_item("Paste");
    println!("✓ ContextMenu: {} menu items", context_menu.items.len());

    println!();
}

/// 示例6: 主题系统
fn example_6_themes() {
    println!("=== 示例6: 主题系统 ===\n");

    // 默认主题
    let default_theme = Theme::default();
    println!("✓ Default Theme:");
    println!("  Primary: [{:.1}, {:.1}, {:.1}, {:.1}]",
        default_theme.colors.primary.r,
        default_theme.colors.primary.g,
        default_theme.colors.primary.b,
        default_theme.colors.primary.a
    );
    println!("  Background: [{:.1}, {:.1}, {:.1}, {:.1}]",
        default_theme.colors.background.r,
        default_theme.colors.background.g,
        default_theme.colors.background.b,
        default_theme.colors.background.a
    );

    // 浅色主题
    let light_theme = Theme::light();
    println!("\n✓ Light Theme:");
    println!("  Background: [{:.1}, {:.1}, {:.1}]",
        light_theme.colors.background.r,
        light_theme.colors.background.g,
        light_theme.colors.background.b
    );

    // 深色主题
    let dark_theme = Theme::dark();
    println!("✓ Dark Theme:");
    println!("  Background: [{:.1}, {:.1}, {:.1}]",
        dark_theme.colors.background.r,
        dark_theme.colors.background.g,
        dark_theme.colors.background.b
    );

    // 自定义主题
    let custom_theme = Theme {
        colors: game_engine::ui::theme::ColorScheme {
            primary: UIColor::rgb(0.9, 0.3, 0.3),
            secondary: UIColor::rgb(0.3, 0.9, 0.3),
            ..Default::default()
        },
        ..Default::default()
    };
    println!("\n✓ Custom Theme:");
    println!("  Primary: Red");
    println!("  Secondary: Green");

    println!();
}

/// 示例7: 布局系统
fn example_7_layouts() {
    println!("=== 示例7: 布局系统 ===\n");

    // RectTransform - 基础变换
    let rect = RectTransform::new()
        .with_position(50.0, 50.0)
        .with_size(200.0, 100.0);

    println!("✓ RectTransform:");
    println!("  Position: ({:.1}, {:.1})", rect.anchored_position.x, rect.anchored_position.y);
    println!("  Size: ({:.1}, {:.1})", rect.size_delta.x, rect.size_delta.y);

    // 预设位置
    let mut rect_top_left = RectTransform::new();
    rect_top_left.set_top_left();
    println!("\n✓ TopLeft Anchor: ({:.1}, {:.1})",
        rect_top_left.anchor_min.x, rect_top_left.anchor_min.y
    );

    let mut rect_center = RectTransform::new();
    rect_center.set_center();
    println!("✓ Center Anchor: ({:.1}, {:.1})",
        rect_center.anchor_min.x, rect_center.anchor_min.y
    );

    let mut rect_stretch = RectTransform::new();
    rect_stretch.set_stretch();
    println!("✓ Stretch Anchor: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        rect_stretch.anchor_min.x, rect_stretch.anchor_min.y,
        rect_stretch.anchor_max.x, rect_stretch.anchor_max.y
    );

    // 布局算法
    println!("\n✓ Layout Algorithms:");
    println!("  - Absolute Layout: 绝对定位");
    println!("  - Horizontal Layout: 水平排列");
    println!("  - Vertical Layout: 垂直排列");
    println!("  - Grid Layout: 网格排列");

    println!();
}

/// 示例8: 事件处理
fn example_8_events() {
    println!("=== 示例8: 事件处理 ===\n");

    use game_engine::ui::events::{UIEventManager, UIEvent, EventListener};

    let mut event_manager = UIEventManager::new();

    // 添加事件监听器
    let listener_id = bevy_ecs::component::ComponentId::new();
    event_manager.add_listener(listener_id, EventListener {
        event_type: "click".to_string(),
        callback: Box::new(|event| {
            println!("Event received: {:?}", event);
            true // 返回true表示事件已处理
        }),
    });

    // 发送事件
    event_manager.send_event(UIEvent::MouseClick {
        position: glam::Vec2::new(100.0, 200.0),
        button: game_engine::ui::events::MouseButton::Left,
    });

    // 处理事件队列
    event_manager.process_events();

    println!("✓ Event types:");
    println!("  - MouseClick, MouseRelease, MouseMove, MouseScroll");
    println!("  - KeyDown, KeyUp, Char");
    println!("  - FocusGained, FocusLost");
    println!("  - ValueChanged, Custom");

    println!();
}

/// 完整示例：创建登录UI
fn example_login_ui() {
    println!("=== 完整示例：登录UI ===\n");

    // 创建面板
    let mut panel = Panel::new()
        .with_background(0.15, 0.15, 0.15, 0.95);

    // 标题
    let title = Label::new("Welcome Back!")
        .with_font_size(24.0)
        .with_alignment(game_engine::ui::widgets::TextAlignment::Center)
        .with_color(1.0, 1.0, 1.0, 1.0);

    // 用户名输入框
    let username_field = TextField::new("Username")
        .with_max_length(30);

    // 密码输入框
    let password_field = TextField::new("Password")
        .with_max_length(50);

    // 登录按钮
    let login_button = Button::new("Login")
        .with_size(200.0, 40.0)
        .with_callback(|| {
            println!("Login button clicked!");
            // 这里执行登录逻辑
        });

    // 记住我复选框
    let remember_checkbox = Checkbox::new("Remember me");

    // 忘记密码标签
    let forgot_link = Label::new("Forgot password?")
        .with_color(0.2, 0.6, 1.0, 1.0)
        .with_font_size(12.0);

    println!("✓ Login UI created with:");
    println!("  - Panel container");
    println!("  - Title label");
    println!("  - Username input field");
    println!("  - Password input field");
    println!("  - Login button");
    println!("  - Remember me checkbox");
    println!("  - Forgot password link");

    println!("\nUI hierarchy:");
    println!("  Panel");
    println!("  ├─ Title (Label)");
    println!("  ├─ Username (TextField)");
    println!("  ├─ Password (TextField)");
    println!("  ├─ Remember Me (Checkbox)");
    println!("  ├─ Forgot Password (Label)");
    println!("  └─ Login (Button)");
}

/// 完整示例：创建设置UI
fn example_settings_ui() {
    println!("=== 完整示例：设置UI ===\n");

    // 创建选项卡控件
    let mut tab_control = TabControl::new();
    tab_control.add_tab("Graphics");
    tab_control.add_tab("Audio");
    tab_control.add_tab("Controls");

    // 音量滑块
    let master_volume = Slider::new(0.0, 100.0);
    let music_volume = Slider::new(0.0, 100.0);
    let sfx_volume = Slider::new(0.0, 100.0);

    // 全屏开关
    let fullscreen_toggle = Toggle::new("Fullscreen");

    // 垂直同步开关
    let vsync_toggle = Toggle::new("V-Sync");

    // 质量下拉菜单
    let quality_options = vec![
        "Low".to_string(),
        "Medium".to_string(),
        "High".to_string(),
        "Ultra".to_string(),
    ];
    let quality_dropdown = Dropdown::new(quality_options);

    println!("✓ Settings UI created with:");
    println!("  - Tab control (3 tabs)");
    println!("  - Volume sliders");
    println!("  - Display toggles");
    println!("  - Quality dropdown");

    println!("\nSettings hierarchy:");
    println!("  TabControl");
    println!("  ├─ Graphics Tab");
    println!("  │   ├─ Fullscreen (Toggle)");
    println!("  │   ├─ V-Sync (Toggle)");
    println!("  │   └─ Quality (Dropdown)");
    println!("  ├─ Audio Tab");
    println!("  │   ├─ Master Volume (Slider)");
    println!("  │   ├─ Music Volume (Slider)");
    println!("  │   └─ SFX Volume (Slider)");
    println!("  └─ Controls Tab");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Test")
            .with_size(100.0, 40.0);
        assert_eq!(button.text, "Test");
        assert_eq!(button.rect.size_delta, glam::Vec2::new(100.0, 40.0));
    }

    #[test]
    fn test_slider_clamping() {
        let mut slider = Slider::new(0.0, 100.0);
        slider.set_value(150.0);
        assert_eq!(slider.current_value, 100.0);
        slider.set_value(-10.0);
        assert_eq!(slider.current_value, 0.0);
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressBar::new();
        progress.set_progress(0.5);
        assert_eq!(progress.progress, 0.5);
        assert_eq!(progress.get_text(), "50%");
    }

    #[test]
    fn test_theme() {
        let theme = Theme::dark();
        assert!(theme.colors.background.r < 0.5);
    }

    #[test]
    fn test_dropdown_selection() {
        let options = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ];
        let mut dropdown = Dropdown::new(options);
        assert_eq!(dropdown.get_selected(), Some(&"A".to_string()));
        dropdown.select(2);
        assert_eq!(dropdown.get_selected(), Some(&"C".to_string()));
    }
}
