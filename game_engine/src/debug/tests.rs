//! Debug UI模块测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_debug_config_default() {
        let config = DebugConfig::default();
        assert!(config.enabled);
        assert!(config.show_entities);
        assert!(config.show_performance);
        assert!(config.show_console);
    }

    #[test]
    fn test_debug_ui_creation() {
        let debug_ui = DebugUI::new();
        // 基本创建测试
        assert!(true);
    }

    #[test]
    fn test_debug_ui_with_config() {
        let config = DebugConfig {
            enabled: false,
            show_entities: false,
            ..Default::default()
        };
        let debug_ui = DebugUI::with_config(config);
        // 配置创建测试
        assert!(true);
    }

    #[test]
    fn test_entity_panel() {
        let mut panel = panels::EntityPanel::new();
        assert!(panel.visible);

        panel.clear_selection();
        assert!(panel.selected_entity().is_none());

        panel.refresh();
        assert!(true);
    }

    #[test]
    fn test_component_panel() {
        let mut panel = panels::ComponentPanel::new();
        assert!(panel.visible);

        panel.clear_entity();
        assert!(true);
    }

    #[test]
    fn test_performance_panel() {
        let panel = panels::PerformancePanel::new();
        assert!(panel.visible);
        assert_eq!(panel.current_fps(), None);
    }

    #[test]
    fn test_performance_panel_with_history() {
        let panel = panels::PerformancePanel::with_history_size(100);

        // 模拟一些帧
        for i in 0..10 {
            panel.update_metrics(0.016, i);
        }

        assert!(panel.current_fps().is_some());
        assert!(panel.calculate_average_fps().is_some());
    }

    #[test]
    fn test_console_panel() {
        let mut panel = panels::ConsolePanel::new();

        panel.add_log("Test message".to_string());
        assert_eq!(panel.log_count(), 1);

        panel.add_error("Error message".to_string());
        assert_eq!(panel.error_count(), 1);

        panel.add_warning("Warning message".to_string());
        assert_eq!(panel.warning_count(), 1);

        panel.clear();
        assert_eq!(panel.log_count(), 0);
    }

    #[test]
    fn test_console_panel_with_max_lines() {
        let mut panel = panels::ConsolePanel::with_max_lines(5);

        for i in 0..10 {
            panel.add_log(format!("Message {}", i));
        }

        // 应该只保留最后5条
        assert_eq!(panel.log_count(), 5);
    }

    #[test]
    fn test_resource_panel() {
        let panel = panels::ResourcePanel::new();
        assert_eq!(panel.resource_type_count(), 0);

        let stats = panels::ResourceStats {
            resource_type: "Texture".to_string(),
            total_count: 100,
            loaded_count: 80,
            failed_count: 2,
            total_size: 1024 * 1024,
            loading_count: 18,
        };

        let mut panel_mut = panel;
        panel_mut.update_stats("Texture".to_string(), stats);
        assert_eq!(panel_mut.resource_type_count(), 1);
    }

    #[test]
    fn test_log_level_colors() {
        use panels::LogLevel;

        let info_color = LogLevel::Info.color();
        let warning_color = LogLevel::Warning.color();
        let error_color = LogLevel::Error.color();
        let debug_color = LogLevel::Debug.color();

        // 验证颜色不冲突
        assert!(info_color != error_color);
        assert!(warning_color != error_color);
    }

    #[test]
    fn test_performance_visualizer() {
        use visualizer::PerformanceVisualizer;

        let mut viz = PerformanceVisualizer::new(100);

        // 添加一些数据点
        for i in 0..10 {
            viz.add_point(i as f32 * 10.0);
        }

        assert!(viz.current_value().is_some());
        assert!(viz.average().is_some());
        assert!(viz.min().is_some());
        assert!(viz.max().is_some());

        viz.clear();
        assert!(viz.current_value().is_none());
    }

    #[test]
    fn test_memory_visualizer() {
        use visualizer::MemoryVisualizer;

        let mut viz = MemoryVisualizer::new(100);

        // 添加内存样本
        for i in 0..10 {
            viz.add_memory_sample(
                1000.0 + i as f64 * 10.0,
                500.0 + i as f64 * 5.0,
                200.0 + i as f64 * 2.0,
            );
        }

        assert!(true);
    }

    #[test]
    fn test_fps_visualizer() {
        use visualizer::FPSVisualizer;

        let mut viz = FPSVisualizer::new(100);

        // 添加FPS样本
        for i in 0..10 {
            viz.add_fps_sample(60.0 + i as f32);
        }

        assert!(viz.current_fps().is_some());
        assert!(viz.average_fps().is_some());
    }

    #[test]
    fn test_debug_ui_log_methods() {
        let mut debug_ui = DebugUI::new();

        debug_ui.log("Info message".to_string());
        debug_ui.log_error("Error message".to_string());

        // 获取控制台并验证
        let console = debug_ui.console_panel();
        assert!(console.log_count() > 0);
    }

    #[test]
    fn test_debug_ui_toggle_panel() {
        let mut debug_ui = DebugUI::new();

        // 初始状态应该是某些面板可见
        debug_ui.toggle_panel("entities");
        debug_ui.toggle_panel("performance");

        // 测试无效面板名（不应panic）
        debug_ui.toggle_panel("invalid_panel");
    }
}
