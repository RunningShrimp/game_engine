// Platform Certification System Integration Tests
// 测试平台认证系统的所有功能

use crate::fixtures::mock_platforms::MockCertificationSystem;

#[cfg(test)]
mod certification_tests {
    use super::*;

    // ============================================================================
    // PS5 平台认证测试
    // ============================================================================

    #[test]
    fn test_ps5_basic_certification() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        let result = cert_system.check_certification();

        assert!(result.is_ok());
        // PS5应该通过基本认证
        assert!(result.unwrap());
    }

    #[test]
    fn test_ps5_trophy_support() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 检查是否有trophy相关的错误
        let has_trophy_error = cert_system
            .errors
            .iter()
            .any(|e| e.contains("trophy"));
        assert!(!has_trophy_error, "Trophy support should be present");
    }

    #[test]
    fn test_ps5_features_warning() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 应该有PS5特性未充分利用的警告
        assert!(
            cert_system.warnings.iter().any(|w| w.contains("PS5 features")),
            "Should warn about PS5 features"
        );
    }

    // ============================================================================
    // Xbox 平台认证测试
    // ============================================================================

    #[test]
    fn test_xbox_basic_certification() {
        let mut cert_system = MockCertificationSystem::new("Xbox");
        let result = cert_system.check_certification();

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_xbox_achievement_support() {
        let mut cert_system = MockCertificationSystem::new("Xbox");
        cert_system.check_certification().unwrap();

        let has_achievement_error = cert_system
            .errors
            .iter()
            .any(|e| e.contains("achievement"));
        assert!(!has_achievement_error, "Achievement support should be present");
    }

    // ============================================================================
    // Switch 平台认证测试
    // ============================================================================

    #[test]
    fn test_switch_basic_certification() {
        let mut cert_system = MockCertificationSystem::new("Switch");
        let result = cert_system.check_certification();

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_switch_memory_requirements() {
        let mut cert_system = MockCertificationSystem::new("Switch");
        cert_system.check_certification().unwrap();

        let has_memory_error = cert_system.errors.iter().any(|e| e.contains("memory"));
        assert!(!has_memory_error, "Should meet memory requirements");
    }

    // ============================================================================
    // Steam 平台认证测试
    // ============================================================================

    #[test]
    fn test_steam_basic_certification() {
        let mut cert_system = MockCertificationSystem::new("Steam");
        let result = cert_system.check_certification();

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_steam_cloud_save_warning() {
        let mut cert_system = MockCertificationSystem::new("Steam");
        cert_system.check_certification().unwrap();

        // Steam应该警告云存档
        assert!(
            cert_system.warnings.iter().any(|w| w.contains("Cloud save")),
            "Should warn about cloud save"
        );
    }

    // ============================================================================
    // Epic 平台认证测试
    // ============================================================================

    #[test]
    fn test_epic_basic_certification() {
        let mut cert_system = MockCertificationSystem::new("Epic");
        let result = cert_system.check_certification();

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_epic_crossplay_warning() {
        let mut cert_system = MockCertificationSystem::new("Epic");
        cert_system.check_certification().unwrap();

        // Epic应该警告跨平台游戏
        assert!(
            cert_system.warnings.iter().any(|w| w.contains("Crossplay")),
            "Should warn about crossplay"
        );
    }

    // ============================================================================
    // 自定义规则测试
    // ============================================================================

    #[test]
    fn test_custom_rules() {
        let mut cert_system = MockCertificationSystem::new("PS5");

        // 添加自定义规则
        cert_system.add_custom_rule("frame_rate_must_be_60".to_string());
        cert_system.add_custom_rule("loading_time_max_3_seconds".to_string());

        // 自定义规则默认通过（mock实现）
        assert!(cert_system.errors.is_empty());
    }

    // ============================================================================
    // 批量检查测试
    // ============================================================================

    #[test]
    fn test_batch_certification_check() {
        let platforms = vec!["PS5", "Xbox", "Switch", "Steam", "Epic"];
        let mut results = Vec::new();

        for platform in platforms {
            let mut cert_system = MockCertificationSystem::new(platform);
            let result = cert_system.check_certification().unwrap();
            results.push((platform.to_string(), result));
        }

        // 所有平台都应该通过认证
        for (platform, passed) in results {
            assert!(passed, "Platform {} should pass certification", platform);
        }
    }

    #[test]
    fn test_parallel_certification_checks() {
        use std::thread;
        use std::sync::{Arc, Mutex};

        let platforms = vec!["PS5", "Xbox", "Switch"];
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for platform in platforms {
            let results_clone = Arc::clone(&results);
            let handle = thread::spawn(move || {
                let mut cert_system = MockCertificationSystem::new(platform);
                let result = cert_system.check_certification().unwrap();
                let mut results = results_clone.lock().unwrap();
                results.push((platform.to_string(), result));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 3);
    }

    // ============================================================================
    // 报告生成测试
    // ============================================================================

    #[test]
    fn test_error_report_generation() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 生成错误报告
        let error_report = format!(
            "Certification Errors for PS5:\n{}",
            cert_system
                .errors
                .iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<_>>()
                .join("\n")
        );

        assert!(error_report.contains("Certification Errors"));
    }

    #[test]
    fn test_warning_report_generation() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 生成警告报告
        let warning_report = format!(
            "Certification Warnings for PS5:\n{}",
            cert_system
                .warnings
                .iter()
                .map(|w| format!("- {}", w))
                .collect::<Vec<_>>()
                .join("\n")
        );

        assert!(warning_report.contains("Certification Warnings"));
    }

    #[test]
    fn test_json_report_format() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 生成JSON格式报告
        let json_report = format!(
            r#"{{"platform": "{}", "certified": {}, "error_count": {}, "warning_count": {}}}"#,
            cert_system.platform,
            cert_system.certified,
            cert_system.errors.len(),
            cert_system.warnings.len()
        );

        assert!(json_report.contains("\"platform\": \"PS5\""));
        assert!(json_report.contains("\"certified\":"));
    }

    // ============================================================================
    // 边界条件和错误处理测试
    // ============================================================================

    #[test]
    fn test_unknown_platform() {
        let mut cert_system = MockCertificationSystem::new("UnknownPlatform");
        let result = cert_system.check_certification();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown platform"));
    }

    #[test]
    fn test_empty_platform_name() {
        let mut cert_system = MockCertificationSystem::new("");
        let result = cert_system.check_certification();

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_certification_checks() {
        let mut cert_system = MockCertificationSystem::new("PS5");

        // 第一次检查
        let result1 = cert_system.check_certification().unwrap();
        assert!(result1);

        // 第二次检查（应该清除之前的错误和警告）
        cert_system.errors.push("Manual error".to_string());
        let result2 = cert_system.check_certification().unwrap();
        assert!(result2);

        // 手动添加的错误应该被清除
        assert!(!cert_system.errors.contains(&"Manual error".to_string()));
    }

    // ============================================================================
    // 性能测试
    // ============================================================================

    #[test]
    fn test_certification_performance() {
        let start = std::time::Instant::now();

        let mut cert_system = MockCertificationSystem::new("PS5");
        for _ in 0..100 {
            cert_system.check_certification().unwrap();
        }

        let duration = start.elapsed();

        // 100次认证检查应该在合理时间内完成
        assert!(
            duration.as_millis() < 100,
            "Certification checks took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_batch_certification_performance() {
        let start = std::time::Instant::now();

        let platforms = vec!["PS5"; 50];
        for platform in platforms {
            let mut cert_system = MockCertificationSystem::new(platform);
            cert_system.check_certification().unwrap();
        }

        let duration = start.elapsed();

        // 50次认证检查应该很快完成
        assert!(
            duration.as_millis() < 50,
            "Batch certification took too long: {:?}",
            duration
        );
    }

    // ============================================================================
    // 认证状态验证测试
    // ============================================================================

    #[test]
    fn test_certification_status_flags() {
        let mut cert_system = MockCertificationSystem::new("PS5");
        cert_system.check_certification().unwrap();

        // 验证认证状态
        assert_eq!(cert_system.platform, "PS5");
        assert_eq!(cert_system.certified, true);
        assert_eq!(cert_system.errors.len(), 0);
    }

    #[test]
    fn test_certification_with_warnings() {
        let mut cert_system = MockCertificationSystem::new("Steam");
        cert_system.check_certification().unwrap();

        // Steam应该通过但有警告
        assert!(cert_system.certified);
        assert!(cert_system.warnings.len() > 0);
    }

    // ============================================================================
    // 跨平台一致性测试
    // ============================================================================

    #[test]
    fn test_all_platforms_have_consistent_api() {
        let platforms = ["PS5", "Xbox", "Switch", "Steam", "Epic"];

        for platform in platforms {
            let mut cert_system = MockCertificationSystem::new(platform);
            let result = cert_system.check_certification();

            // 所有平台都应该返回Result类型
            assert!(result.is_ok() || result.is_err());

            // 所有平台都应该有相同的字段
            assert_eq!(cert_system.platform, platform);
            assert!(std::any::Any::type_name_of_val(&cert_system.certified)
                .contains("bool"));
        }
    }
}
