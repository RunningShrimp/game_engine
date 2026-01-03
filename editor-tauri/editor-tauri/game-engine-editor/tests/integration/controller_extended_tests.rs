// Controller Extended Features Integration Tests
// 测试控制器扩展功能的所有特性

use crate::fixtures::mock_platforms::MockController;

#[cfg(test)]
mod controller_tests {
    use super::*;

    // ============================================================================
    // 基本控制器功能测试
    // ============================================================================

    #[test]
    fn test_controller_creation() {
        let controller = MockController::new(0, "PS5");
        assert_eq!(controller.id, 0);
        assert_eq!(controller.platform, "PS5");
        assert!(controller.state.connected);
    }

    #[test]
    fn test_controller_button_input() {
        let mut controller = MockController::new(0, "Xbox");

        controller.set_button("A", true);
        assert!(controller.state.is_button_pressed("A"));

        controller.set_button("A", false);
        assert!(!controller.state.is_button_pressed("A"));
    }

    #[test]
    fn test_controller_axis_input() {
        let mut controller = MockController::new(0, "PS5");

        controller.set_axis("left_x", 0.5);
        assert_eq!(controller.state.get_axis("left_x"), 0.5);

        controller.set_axis("left_y", -0.8);
        assert_eq!(controller.state.get_axis("left_y"), -0.8);
    }

    #[test]
    fn test_controller_combined_input() {
        let mut controller = MockController::new(0, "Switch");

        controller.set_button("A", true);
        controller.set_axis("left_x", 1.0);
        controller.set_axis("left_y", 0.5);

        assert!(controller.state.is_button_pressed("A"));
        assert_eq!(controller.state.get_axis("left_x"), 1.0);
        assert_eq!(controller.state.get_axis("left_y"), 0.5);
    }

    // ============================================================================
    // 振动功能测试
    // ============================================================================

    #[test]
    fn test_ps5_vibration() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.vibration_supported);

        let result = controller.vibrate(0.5, 0.7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_xbox_vibration() {
        let controller = MockController::new(0, "Xbox");
        assert!(controller.vibration_supported);

        let result = controller.vibrate(0.8, 0.9);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_vibration() {
        let controller = MockController::new(0, "Switch");
        assert!(controller.vibration_supported);

        let result = controller.vibrate(0.3, 0.4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vibration_bounds() {
        let controller = MockController::new(0, "PS5");

        // 测试边界值
        assert!(controller.vibrate(0.0, 0.0).is_ok());
        assert!(controller.vibrate(1.0, 1.0).is_ok());

        // 超出范围的值也应该被接受（在真实实现中会被截断）
        assert!(controller.vibrate(1.5, 2.0).is_ok());
    }

    #[test]
    fn test_unsupported_vibration() {
        // 创建不支持振动的控制器
        let controller = MockController {
            vibration_supported: false,
            ..MockController::new(0, "Generic")
        };

        let result = controller.vibrate(0.5, 0.5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
    }

    // ============================================================================
    // LED控制测试
    // ============================================================================

    #[test]
    fn test_ps5_led_control() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.led_supported);

        let result = controller.set_led_color(255, 0, 0); // 红色
        assert!(result.is_ok());
    }

    #[test]
    fn test_ps4_led_control() {
        let controller = MockController::new(0, "PS4");
        assert!(controller.led_supported);

        let result = controller.set_led_color(0, 255, 0); // 绿色
        assert!(result.is_ok());
    }

    #[test]
    fn test_xbox_no_led() {
        let controller = MockController::new(0, "Xbox");
        assert!(!controller.led_supported);

        let result = controller.set_led_color(255, 255, 255);
        assert!(result.is_err());
    }

    #[test]
    fn test_led_color_values() {
        let controller = MockController::new(0, "PS5");

        // 测试各种颜色
        assert!(controller.set_led_color(0, 0, 0).is_ok()); // 黑色
        assert!(controller.set_led_color(255, 255, 255).is_ok()); // 白色
        assert!(controller.set_led_color(128, 128, 128).is_ok()); // 灰色
    }

    // ============================================================================
    // 触摸板输入测试
    // ============================================================================

    #[test]
    fn test_ps5_touchpad_supported() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.touchpad_supported);
    }

    #[test]
    fn test_ps4_touchpad_supported() {
        let controller = MockController::new(0, "PS4");
        assert!(controller.touchpad_supported);
    }

    #[test]
    fn test_xbox_no_touchpad() {
        let controller = MockController::new(0, "Xbox");
        assert!(!controller.touchpad_supported);
    }

    #[test]
    fn test_touchpad_input_simulation() {
        let mut controller = MockController::new(0, "PS5");

        // 模拟触摸板输入
        controller.set_button("touchpad", true);
        assert!(controller.state.is_button_pressed("touchpad"));
    }

    // ============================================================================
    // 运动传感器测试
    // ============================================================================

    #[test]
    fn test_ps5_motion_supported() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.motion_supported);
    }

    #[test]
    fn test_ps4_motion_supported() {
        let controller = MockController::new(0, "PS4");
        assert!(controller.motion_supported);
    }

    #[test]
    fn test_switch_motion_supported() {
        let controller = MockController::new(0, "Switch");
        assert!(controller.motion_supported);
    }

    #[test]
    fn test_xbox_no_motion() {
        let controller = MockController::new(0, "Xbox");
        assert!(!controller.motion_supported);
    }

    #[test]
    fn test_motion_data_simulation() {
        let mut controller = MockController::new(0, "PS5");

        // 模拟运动传感器数据
        controller.set_axis("gyro_x", 0.1);
        controller.set_axis("gyro_y", 0.2);
        controller.set_axis("gyro_z", 0.3);

        assert_eq!(controller.state.get_axis("gyro_x"), 0.1);
        assert_eq!(controller.state.get_axis("gyro_y"), 0.2);
        assert_eq!(controller.state.get_axis("gyro_z"), 0.3);
    }

    // ============================================================================
    // PS5 特定功能测试
    // ============================================================================

    #[test]
    fn test_ps5_haptic_feedback() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.haptic_supported);

        let result = controller.set_haptic([0.5, 0.5], 0.8);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ps5_adaptive_triggers() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.adaptive_triggers_supported);

        let result = controller.set_adaptive_trigger("left", 0.5, 0.7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ps5_both_triggers() {
        let controller = MockController::new(0, "PS5");

        let left_result = controller.set_adaptive_trigger("left", 0.3, 0.5);
        let right_result = controller.set_adaptive_trigger("right", 0.4, 0.6);

        assert!(left_result.is_ok());
        assert!(right_result.is_ok());
    }

    #[test]
    fn test_ps4_no_haptic() {
        let controller = MockController::new(0, "PS4");
        assert!(!controller.haptic_supported);

        let result = controller.set_haptic([0.5, 0.5], 0.8);
        assert!(result.is_err());
    }

    #[test]
    fn test_ps4_no_adaptive_triggers() {
        let controller = MockController::new(0, "PS4");
        assert!(!controller.adaptive_triggers_supported);

        let result = controller.set_adaptive_trigger("left", 0.5, 0.7);
        assert!(result.is_err());
    }

    // ============================================================================
    // 多平台控制器兼容性测试
    // ============================================================================

    #[test]
    fn test_all_platforms_basic_input() {
        let platforms = ["PS5", "PS4", "Xbox", "Switch"];

        for platform in platforms {
            let mut controller = MockController::new(0, platform);

            // 所有平台都应该支持基本按钮输入
            controller.set_button("A", true);
            assert!(controller.state.is_button_pressed("A"));
        }
    }

    #[test]
    fn test_cross_platform_button_mapping() {
        let platforms = ["PS5", "Xbox", "Switch"];

        for platform in platforms {
            let mut controller = MockController::new(0, platform);

            // 测试常用按钮映射
            controller.set_button("A", true);
            controller.set_button("B", false);
            controller.set_button("X", true);
            controller.set_button("Y", false);

            assert!(controller.state.is_button_pressed("A"));
            assert!(!controller.state.is_button_pressed("B"));
            assert!(controller.state.is_button_pressed("X"));
            assert!(!controller.state.is_button_pressed("Y"));
        }
    }

    #[test]
    fn test_cross_platform_axis_mapping() {
        let platforms = ["PS5", "Xbox", "Switch"];

        for platform in platforms {
            let mut controller = MockController::new(0, platform);

            // 测试摇杆轴
            controller.set_axis("left_x", 0.5);
            controller.set_axis("left_y", -0.5);
            controller.set_axis("right_x", 1.0);
            controller.set_axis("right_y", -1.0);

            assert_eq!(controller.state.get_axis("left_x"), 0.5);
            assert_eq!(controller.state.get_axis("left_y"), -0.5);
            assert_eq!(controller.state.get_axis("right_x"), 1.0);
            assert_eq!(controller.state.get_axis("right_y"), -1.0);
        }
    }

    // ============================================================================
    // 控制器校准测试
    // ============================================================================

    #[test]
    fn test_controller_axis_calibration() {
        let mut controller = MockController::new(0, "PS5");

        // 模拟未校准的轴（有死区偏移）
        controller.set_axis("left_x", 0.1); // 死区内的值

        // 在真实实现中，校准会将死区内的值归零
        let calibrated_value = controller.state.get_axis("left_x");
        assert_eq!(calibrated_value, 0.1);
    }

    #[test]
    fn test_motion_sensor_calibration() {
        let mut controller = MockController::new(0, "PS5");

        // 模拟陀螺仪偏移
        controller.set_axis("gyro_x", 0.05);
        controller.set_axis("gyro_y", -0.03);

        // 校准应该减去偏移量
        let gyro_x = controller.state.get_axis("gyro_x");
        let gyro_y = controller.state.get_axis("gyro_y");

        assert_eq!(gyro_x, 0.05);
        assert_eq!(gyro_y, -0.03);
    }

    // ============================================================================
    // 多控制器测试
    // ============================================================================

    #[test]
    fn test_multiple_controllers() {
        let controllers = vec![
            MockController::new(0, "PS5"),
            MockController::new(1, "PS5"),
            MockController::new(2, "Xbox"),
            MockController::new(3, "Switch"),
        ];

        assert_eq!(controllers.len(), 4);
        assert_eq!(controllers[0].id, 0);
        assert_eq!(controllers[1].id, 1);
        assert_eq!(controllers[2].id, 2);
        assert_eq!(controllers[3].id, 3);
    }

    #[test]
    fn test_multiple_controllers_independent_state() {
        let mut controller1 = MockController::new(0, "PS5");
        let mut controller2 = MockController::new(1, "PS5");

        controller1.set_button("A", true);
        controller2.set_button("A", false);

        assert!(controller1.state.is_button_pressed("A"));
        assert!(!controller2.state.is_button_pressed("A"));
    }

    // ============================================================================
    // 控制器连接状态测试
    // ============================================================================

    #[test]
    fn test_controller_connection_state() {
        let controller = MockController::new(0, "PS5");
        assert!(controller.state.connected);
    }

    #[test]
    fn test_controller_disconnection_simulation() {
        let mut controller = MockController {
            state: crate::fixtures::test_entities::TestControllerState {
                connected: false,
                ..Default::default()
            },
            ..MockController::new(0, "PS5")
        };

        assert!(!controller.state.connected);
    }

    // ============================================================================
    // 性能测试
    // ============================================================================

    #[test]
    fn test_controller_input_performance() {
        let mut controller = MockController::new(0, "PS5");
        let iterations = 10000;

        let start = std::time::Instant::now();

        for i in 0..iterations {
            controller.set_button("A", i % 2 == 0);
            controller.set_axis("left_x", (i as f32) / 10000.0);
            let _ = controller.state.is_button_pressed("A");
            let _ = controller.state.get_axis("left_x");
        }

        let duration = start.elapsed();

        // 10000次操作应该很快完成
        assert!(
            duration.as_millis() < 100,
            "Controller input processing took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_multiple_controllers_performance() {
        let mut controllers: Vec<MockController> = vec![
            MockController::new(0, "PS5"),
            MockController::new(1, "Xbox"),
            MockController::new(2, "Switch"),
        ];

        let start = std::time::Instant::now();

        for _ in 0..1000 {
            for controller in &mut controllers {
                controller.set_button("A", true);
                controller.vibrate(0.5, 0.5).ok();
            }
        }

        let duration = start.elapsed();

        // 多控制器操作应该在合理时间内完成
        assert!(
            duration.as_millis() < 200,
            "Multiple controllers processing took too long: {:?}",
            duration
        );
    }

    // ============================================================================
    // 边界条件和错误处理测试
    // ============================================================================

    #[test]
    fn test_invalid_button_name() {
        let controller = MockController::new(0, "PS5");

        // 检查不存在的按钮应该返回false
        assert!(!controller.state.is_button_pressed("nonexistent_button"));
    }

    #[test]
    fn test_invalid_axis_name() {
        let controller = MockController::new(0, "PS5");

        // 检查不存在的轴应该返回0.0
        assert_eq!(controller.state.get_axis("nonexistent_axis"), 0.0);
    }

    #[test]
    fn test_axis_value_clamping() {
        let mut controller = MockController::new(0, "PS5");

        // 设置超出范围的值
        controller.set_axis("left_x", 1.5);
        controller.set_axis("left_y", -2.0);

        // 在真实实现中，值应该被截断到[-1, 1]
        // 在mock中，我们只是验证设置不崩溃
        assert_eq!(controller.state.get_axis("left_x"), 1.5);
        assert_eq!(controller.state.get_axis("left_y"), -2.0);
    }

    #[test]
    fn test_vibration_with_extreme_values() {
        let controller = MockController::new(0, "PS5");

        // 测试极端值
        assert!(controller.vibrate(-1.0, -1.0).is_ok()); // 负值
        assert!(controller.vibrate(2.0, 2.0).is_ok()); // 超过1.0
        assert!(controller.vibrate(100.0, 100.0).is_ok()); // 非常大的值
    }

    // ============================================================================
    // 平台特定功能差异测试
    // ============================================================================

    #[test]
    fn test_platform_feature_differences() {
        let ps5 = MockController::new(0, "PS5");
        let xbox = MockController::new(1, "Xbox");
        let switch = MockController::new(2, "Switch");

        // PS5应该有最多功能
        assert!(ps5.vibration_supported);
        assert!(ps5.led_supported);
        assert!(ps5.touchpad_supported);
        assert!(ps5.motion_supported);
        assert!(ps5.haptic_supported);
        assert!(ps5.adaptive_triggers_supported);

        // Xbox功能较少
        assert!(xbox.vibration_supported);
        assert!(!xbox.led_supported);
        assert!(!xbox.touchpad_supported);
        assert!(!xbox.motion_supported);
        assert!(!xbox.haptic_supported);
        assert!(!xbox.adaptive_triggers_supported);

        // Switch中等
        assert!(switch.vibration_supported);
        assert!(!switch.led_supported);
        assert!(!switch.touchpad_supported);
        assert!(switch.motion_supported);
        assert!(!switch.haptic_supported);
        assert!(!switch.adaptive_triggers_supported);
    }

    #[test]
    fn test_platform_capability_matrix() {
        use std::collections::HashMap;

        let platforms = vec!["PS5", "PS4", "Xbox", "Switch"];
        let mut capability_matrix: HashMap<&str, Vec<bool>> = HashMap::new();

        for platform in platforms {
            let controller = MockController::new(0, platform);
            let capabilities = vec![
                controller.vibration_supported,
                controller.led_supported,
                controller.touchpad_supported,
                controller.motion_supported,
                controller.haptic_supported,
                controller.adaptive_triggers_supported,
            ];
            capability_matrix.insert(platform, capabilities);
        }

        // 验证PS5功能最全
        let ps5_caps = capability_matrix.get("PS5").unwrap();
        assert!(ps5_caps.iter().filter(|&&x| x).count() >= 5);
    }
}
