//! UI System Tests
//!
//! 测试UI系统的各个组件，包括布局、组件和事件处理。

#[cfg(test)]
mod ui_core_tests {
    use crate::ui::{LayoutType, UIRoot, UIWidget, WidgetType};
    use bevy_ecs::prelude::*;
    use glam::Vec2;

    #[test]
    fn test_ui_root_creation() {
        let root = UIRoot::default();
        assert_eq!(root.width, 800.0);
        assert_eq!(root.height, 600.0);
        assert_eq!(root.scale_factor, 1.0);
        assert!(root.visible);
    }

    #[test]
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
    fn test_ui_widget_creation() {
        let widget = UIWidget::default();
        assert!(widget.visible);
        assert!(widget.enabled);
        assert_eq!(widget.z_index, 0);
        assert_eq!(widget.position, Vec2::ZERO);
    }

    #[test]
    fn test_ui_widget_custom() {
        let widget = UIWidget {
            widget_type: WidgetType::Label {
                text: "Test Label".to_string(),
                font_size: 16.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            position: Vec2::new(10.0, 20.0),
            size: Vec2::new(100.0, 30.0),
            visible: false,
            enabled: false,
            z_index: 5,
        };
        assert!(!widget.visible);
        assert!(!widget.enabled);
        assert_eq!(widget.z_index, 5);
        assert_eq!(widget.position, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_widget_type_button() {
        let button_type = WidgetType::Button {
            text: "Click Me".to_string(),
            on_click: None,
            pressed: false,
        };
        match button_type {
            WidgetType::Button { text, .. } => {
                assert_eq!(text, "Click Me");
            }
            _ => panic!("Expected Button widget type"),
        }
    }

    #[test]
    fn test_widget_type_label() {
        let label_type = WidgetType::Label {
            text: "Hello World".to_string(),
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        match label_type {
            WidgetType::Label {
                text, font_size, ..
            } => {
                assert_eq!(text, "Hello World");
                assert_eq!(font_size, 14.0);
            }
            _ => panic!("Expected Label widget type"),
        }
    }

    #[test]
    fn test_widget_type_input() {
        let input_type = WidgetType::Input {
            placeholder: "Enter text...".to_string(),
            value: String::new(),
            focused: false,
            max_length: Some(100),
        };
        match input_type {
            WidgetType::Input {
                placeholder,
                max_length,
                ..
            } => {
                assert_eq!(placeholder, "Enter text...");
                assert_eq!(max_length, Some(100));
            }
            _ => panic!("Expected Input widget type"),
        }
    }

    #[test]
    fn test_widget_type_slider() {
        let slider_type = WidgetType::Slider {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            on_change: None,
        };
        match slider_type {
            WidgetType::Slider {
                min, max, value, ..
            } => {
                assert_eq!(min, 0.0);
                assert_eq!(max, 100.0);
                assert_eq!(value, 50.0);
            }
            _ => panic!("Expected Slider widget type"),
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use crate::ui::LayoutType;

    #[test]
    fn test_layout_type_vertical() {
        let layout = LayoutType::Vertical;
        match layout {
            LayoutType::Vertical => {}
            _ => panic!("Expected Vertical layout"),
        }
    }

    #[test]
    fn test_layout_type_horizontal() {
        let layout = LayoutType::Horizontal;
        match layout {
            LayoutType::Horizontal => {}
            _ => panic!("Expected Horizontal layout"),
        }
    }

    #[test]
    fn test_layout_type_relative() {
        let layout = LayoutType::Relative;
        match layout {
            LayoutType::Relative => {}
            _ => panic!("Expected Relative layout"),
        }
    }

    #[test]
    fn test_layout_type_grid() {
        let layout = LayoutType::Grid { rows: 3, cols: 4 };
        match layout {
            LayoutType::Grid { rows, cols } => {
                assert_eq!(rows, 3);
                assert_eq!(cols, 4);
            }
            _ => panic!("Expected Grid layout"),
        }
    }
}

#[cfg(test)]
mod widget_interaction_tests {
    use crate::ui::UIWidget;
    use glam::Vec2;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_button_click_callback() {
        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let button = crate::ui::WidgetType::Button {
            text: "Click Me".to_string(),
            on_click: Some(Box::new(move || {
                clicked_clone.store(true, Ordering::SeqCst);
            })),
            pressed: false,
        };

        if let crate::ui::WidgetType::Button { on_click, .. } = button {
            assert!(on_click.is_some());
            // Execute the callback
            if let Some(callback) = on_click {
                callback();
            }
        }
    }

    #[test]
    fn test_widget_visibility_toggle() {
        let mut widget = UIWidget {
            visible: true,
            ..Default::default()
        };
        assert!(widget.visible);

        widget.visible = false;
        assert!(!widget.visible);

        widget.visible = true;
        assert!(widget.visible);
    }

    #[test]
    fn test_widget_enable_toggle() {
        let mut widget = UIWidget {
            enabled: true,
            ..Default::default()
        };
        assert!(widget.enabled);

        widget.enabled = false;
        assert!(!widget.enabled);
    }

    #[test]
    fn test_widget_z_index() {
        let widget1 = UIWidget {
            z_index: 0,
            ..Default::default()
        };
        let widget2 = UIWidget {
            z_index: 10,
            ..Default::default()
        };
        let widget3 = UIWidget {
            z_index: -5,
            ..Default::default()
        };

        assert!(widget2.z_index > widget1.z_index);
        assert!(widget1.z_index > widget3.z_index);
    }
}

#[cfg(test)]
mod widget_positioning_tests {
    use crate::ui::UIWidget;
    use glam::Vec2;

    #[test]
    fn test_widget_at_position() {
        let widget = UIWidget {
            position: Vec2::new(100.0, 200.0),
            size: Vec2::new(50.0, 30.0),
            ..Default::default()
        };

        assert_eq!(widget.position.x, 100.0);
        assert_eq!(widget.position.y, 200.0);
        assert_eq!(widget.size.x, 50.0);
        assert_eq!(widget.size.y, 30.0);
    }

    #[test]
    fn test_widget_bounds() {
        let widget = UIWidget {
            position: Vec2::new(10.0, 20.0),
            size: Vec2::new(100.0, 50.0),
            ..Default::default()
        };

        // Calculate bounds
        let min_x = widget.position.x;
        let max_x = widget.position.x + widget.size.x;
        let min_y = widget.position.y;
        let max_y = widget.position.y + widget.size.y;

        assert_eq!(min_x, 10.0);
        assert_eq!(max_x, 110.0);
        assert_eq!(min_y, 20.0);
        assert_eq!(max_y, 70.0);
    }

    #[test]
    fn test_point_inside_widget() {
        let widget = UIWidget {
            position: Vec2::new(50.0, 50.0),
            size: Vec2::new(100.0, 80.0),
            ..Default::default()
        };

        // Point inside widget
        let point_inside = Vec2::new(100.0, 90.0);
        assert!(point_inside.x >= widget.position.x);
        assert!(point_inside.x <= widget.position.x + widget.size.x);
        assert!(point_inside.y >= widget.position.y);
        assert!(point_inside.y <= widget.position.y + widget.size.y);

        // Point outside widget
        let point_outside = Vec2::new(10.0, 10.0);
        assert!(point_outside.x < widget.position.x);
    }
}
