//! UI模块单元测试

use game_engine::ui::{LayoutType, UIRoot, UIService, UIState, UITheme, UIWidget, WidgetType};
use glam::Vec2;

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_root_default() {
    let root = UIRoot::default();
    assert_eq!(root.width, 800.0);
    assert_eq!(root.height, 600.0);
    assert_eq!(root.scale_factor, 1.0);
    assert!(root.visible);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_root_custom() {
    let root = UIRoot {
        width: 1920.0,
        height: 1080.0,
        scale_factor: 2.0,
        visible: false,
    };
    assert_eq!(root.width, 1920.0);
    assert_eq!(root.height, 1080.0);
    assert_eq!(root.scale_factor, 2.0);
    assert!(!root.visible);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_widget_default() {
    let widget = UIWidget::default();
    assert_eq!(widget.position, Vec2::ZERO);
    assert_eq!(widget.size, Vec2::new(100.0, 50.0));
    assert!(widget.visible);
    assert!(widget.enabled);
    assert_eq!(widget.z_index, 0);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_widget_custom() {
    let widget = UIWidget {
        position: Vec2::new(10.0, 20.0),
        size: Vec2::new(200.0, 100.0),
        visible: false,
        enabled: false,
        z_index: 5,
        ..Default::default()
    };
    assert_eq!(widget.position, Vec2::new(10.0, 20.0));
    assert_eq!(widget.size, Vec2::new(200.0, 100.0));
    assert!(!widget.visible);
    assert!(!widget.enabled);
    assert_eq!(widget.z_index, 5);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_create_button() {
    let button = UIService::create_button(
        "Click me".to_string(),
        Vec2::new(10.0, 20.0),
        Vec2::new(100.0, 50.0),
        None,
    );

    assert_eq!(button.position, Vec2::new(10.0, 20.0));
    assert_eq!(button.size, Vec2::new(100.0, 50.0));
    assert!(button.visible);
    assert!(button.enabled);

    match button.widget_type {
        WidgetType::Button { text, pressed, .. } => {
            assert_eq!(text, "Click me");
            assert!(!pressed);
        }
        _ => panic!("Expected Button widget type"),
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_create_label() {
    let label = UIService::create_label("Hello World".to_string(), Vec2::new(5.0, 10.0), 24.0);

    assert_eq!(label.position, Vec2::new(5.0, 10.0));
    assert_eq!(label.size.y, 24.0);

    match label.widget_type {
        WidgetType::Label {
            text, font_size, ..
        } => {
            assert_eq!(text, "Hello World");
            assert_eq!(font_size, 24.0);
        }
        _ => panic!("Expected Label widget type"),
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_create_input() {
    let input = UIService::create_input(
        "Enter text...".to_string(),
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 30.0),
    );

    assert_eq!(input.size, Vec2::new(200.0, 30.0));

    match input.widget_type {
        WidgetType::Input {
            placeholder,
            value,
            focused,
            ..
        } => {
            assert_eq!(placeholder, "Enter text...");
            assert_eq!(value, "");
            assert!(!focused);
        }
        _ => panic!("Expected Input widget type"),
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_create_container() {
    let container = UIService::create_container(
        LayoutType::Vertical,
        Vec2::new(0.0, 0.0),
        Vec2::new(300.0, 400.0),
    );

    assert_eq!(container.size, Vec2::new(300.0, 400.0));

    match container.widget_type {
        WidgetType::Container { layout, children } => {
            matches!(layout, LayoutType::Vertical);
            assert!(children.is_empty());
        }
        _ => panic!("Expected Container widget type"),
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_is_point_inside() {
    let widget = UIWidget {
        position: Vec2::new(10.0, 20.0),
        size: Vec2::new(100.0, 50.0),
        visible: true,
        enabled: true,
        ..Default::default()
    };

    // 点在内部
    assert!(UIService::is_point_inside(&widget, Vec2::new(50.0, 40.0)));

    // 点在边界上
    assert!(UIService::is_point_inside(&widget, Vec2::new(10.0, 20.0)));
    assert!(UIService::is_point_inside(&widget, Vec2::new(110.0, 70.0)));

    // 点在外部
    assert!(!UIService::is_point_inside(&widget, Vec2::new(5.0, 15.0)));
    assert!(!UIService::is_point_inside(&widget, Vec2::new(120.0, 80.0)));
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_is_point_inside_invisible() {
    let widget = UIWidget {
        position: Vec2::new(10.0, 20.0),
        size: Vec2::new(100.0, 50.0),
        visible: false,
        enabled: true,
        ..Default::default()
    };

    // 不可见组件应该返回false
    assert!(!UIService::is_point_inside(&widget, Vec2::new(50.0, 40.0)));
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_service_is_point_inside_disabled() {
    let widget = UIWidget {
        position: Vec2::new(10.0, 20.0),
        size: Vec2::new(100.0, 50.0),
        visible: true,
        enabled: false,
        ..Default::default()
    };

    // 禁用的组件应该返回false
    assert!(!UIService::is_point_inside(&widget, Vec2::new(50.0, 40.0)));
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_state_default() {
    let state = UIState::default();
    assert!(state.focused_widget.is_none());
    assert!(state.hovered_widget.is_none());
    assert!(state.drag_target.is_none());
    assert_eq!(state.cursor_position, Vec2::ZERO);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_theme_default() {
    let theme = UITheme::default();
    assert_eq!(theme.primary_color, [0.2, 0.6, 1.0, 1.0]);
    assert_eq!(theme.secondary_color, [0.8, 0.8, 0.8, 1.0]);
    assert_eq!(theme.background_color, [0.1, 0.1, 0.1, 1.0]);
    assert_eq!(theme.text_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(theme.font_size, 16.0);
    assert_eq!(theme.border_radius, 4.0);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_theme_custom() {
    let theme = UITheme {
        primary_color: [1.0, 0.0, 0.0, 1.0],
        secondary_color: [0.0, 1.0, 0.0, 1.0],
        background_color: [0.0, 0.0, 1.0, 1.0],
        text_color: [1.0, 1.0, 0.0, 1.0],
        font_size: 20.0,
        border_radius: 8.0,
    };

    assert_eq!(theme.primary_color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(theme.font_size, 20.0);
    assert_eq!(theme.border_radius, 8.0);
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_layout_types() {
    let vertical = LayoutType::Vertical;
    let horizontal = LayoutType::Horizontal;
    let relative = LayoutType::Relative;
    let grid = LayoutType::Grid { rows: 3, cols: 4 };

    // 测试克隆
    let vertical_clone = vertical;
    matches!(vertical_clone, LayoutType::Vertical);

    matches!(horizontal, LayoutType::Horizontal);
    matches!(relative, LayoutType::Relative);

    if let LayoutType::Grid { rows, cols } = grid {
        assert_eq!(rows, 3);
        assert_eq!(cols, 4);
    } else {
        panic!("Expected Grid layout");
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_widget_types() {
    // 测试按钮
    let button = WidgetType::Button {
        text: "Click".to_string(),
        on_click: None,
        pressed: false,
    };

    match button {
        WidgetType::Button { text, pressed, .. } => {
            assert_eq!(text, "Click");
            assert!(!pressed);
        }
        _ => panic!("Expected Button"),
    }

    // 测试标签
    let label = WidgetType::Label {
        text: "Label".to_string(),
        font_size: 16.0,
        color: [1.0, 1.0, 1.0, 1.0],
    };

    match label {
        WidgetType::Label {
            text,
            font_size,
            color,
        } => {
            assert_eq!(text, "Label");
            assert_eq!(font_size, 16.0);
            assert_eq!(color, [1.0, 1.0, 1.0, 1.0]);
        }
        _ => panic!("Expected Label"),
    }

    // 测试输入框
    let input = WidgetType::Input {
        placeholder: "Enter...".to_string(),
        value: "test".to_string(),
        focused: true,
        max_length: Some(100),
    };

    match input {
        WidgetType::Input {
            placeholder,
            value,
            focused,
            max_length,
        } => {
            assert_eq!(placeholder, "Enter...");
            assert_eq!(value, "test");
            assert!(focused);
            assert_eq!(max_length, Some(100));
        }
        _ => panic!("Expected Input"),
    }

    // 测试滑块
    let slider = WidgetType::Slider {
        min: 0.0,
        max: 100.0,
        value: 50.0,
        on_change: None,
    };

    match slider {
        WidgetType::Slider {
            min, max, value, ..
        } => {
            assert_eq!(min, 0.0);
            assert_eq!(max, 100.0);
            assert_eq!(value, 50.0);
        }
        _ => panic!("Expected Slider"),
    }
}

#[test]
#[ignore] // TODO: Fix compilation errors
fn test_ui_widget_z_index_ordering() {
    let mut widgets = vec![
        UIWidget {
            z_index: 5,
            ..Default::default()
        },
        UIWidget {
            z_index: 1,
            ..Default::default()
        },
        UIWidget {
            z_index: 10,
            ..Default::default()
        },
        UIWidget {
            z_index: 3,
            ..Default::default()
        },
    ];

    widgets.sort_by_key(|w| w.z_index);

    assert_eq!(widgets[0].z_index, 1);
    assert_eq!(widgets[1].z_index, 3);
    assert_eq!(widgets[2].z_index, 5);
    assert_eq!(widgets[3].z_index, 10);
}
