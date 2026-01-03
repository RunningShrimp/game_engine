//! Cross-Platform Compatibility Tests

use game_engine::platform::detection_extended::{Platform, Feature, platform_capabilities};
use game_engine::platform::validation::{CompatibilityValidator, HardwareCapabilityMatrix};

#[test]
fn test_all_console_platforms_detection() {
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        assert!(platform.is_console());
        assert!(platform.as_console_platform().is_some());
    }
}

#[test]
fn test_capability_matrix_generation() {
    let matrix = HardwareCapabilityMatrix::new();

    // Should have all console platforms
    assert!(matrix.platforms.contains_key("Nintendo Switch"));
    assert!(matrix.platforms.contains_key("PlayStation 5"));
    assert!(matrix.platforms.contains_key("PlayStation 4"));
    assert!(matrix.platforms.contains_key("Xbox Series X/S"));
    assert!(matrix.platforms.contains_key("Xbox One"));
}

#[test]
fn test_ray_tracing_availability() {
    // Only PS5 and Xbox Series X should support ray tracing
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let series_x_caps = platform_capabilities(Platform::XboxSeries);
    let ps4_caps = platform_capabilities(Platform::PlayStation4);
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let xbox_one_caps = platform_capabilities(Platform::XboxOne);

    assert!(ps5_caps.supports_feature(Feature::RayTracing));
    assert!(series_x_caps.supports_feature(Feature::RayTracing));
    assert!(!ps4_caps.supports_feature(Feature::RayTracing));
    assert!(!switch_caps.supports_feature(Feature::RayTracing));
    assert!(!xbox_one_caps.supports_feature(Feature::RayTracing));
}

#[test]
fn test_hdr_availability() {
    // Switch should not have HDR, others should
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let ps4_caps = platform_capabilities(Platform::PlayStation4);
    let series_caps = platform_capabilities(Platform::XboxSeries);
    let xbox_one_caps = platform_capabilities(Platform::XboxOne);

    assert!(!switch_caps.supports_feature(Feature::HDR));
    assert!(ps5_caps.supports_feature(Feature::HDR));
    assert!(ps4_caps.supports_feature(Feature::HDR));
    assert!(series_caps.supports_feature(Feature::HDR));
    assert!(xbox_one_caps.supports_feature(Feature::HDR));
}

#[test]
fn test_motion_controls_availability() {
    // Switch and PS5 have motion controls
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let ps4_caps = platform_capabilities(Platform::PlayStation4);

    assert!(switch_caps.supports_feature(Feature::MotionControls));
    assert!(ps5_caps.supports_feature(Feature::MotionControls));
    assert!(ps4_caps.supports_feature(Feature::MotionControls));
}

#[test]
fn test_all_platforms_validation() {
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        let validator = CompatibilityValidator::new(platform);
        let report = validator.validate_all();

        // All platforms should pass basic validation
        assert!(report.is_valid(), "Platform {:?} failed validation", platform);
    }
}

#[test]
fn test_memory_tier_comparison() {
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let last_gen_caps = platform_capabilities(Platform::PlayStation4);
    let current_gen_caps = platform_capabilities(Platform::PlayStation5);

    // Memory hierarchy
    assert!(switch_caps.hardware.memory_mb < last_gen_caps.hardware.memory_mb);
    assert!(last_gen_caps.hardware.memory_mb <= current_gen_caps.hardware.memory_mb);

    assert_eq!(switch_caps.hardware.memory_mb, 4 * 1024); // 4GB
    assert_eq!(last_gen_caps.hardware.memory_mb, 8 * 1024); // 8GB
    assert_eq!(current_gen_caps.hardware.memory_mb, 16 * 1024); // 16GB
}

#[test]
fn test_gpu_capability_comparison() {
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);

    // PS5 has better GPU capabilities
    assert!(ps5_caps.max_samplers > switch_caps.max_samplers);
    assert!(ps5_caps.max_render_targets >= switch_caps.max_render_targets);
}

#[test]
fn test_online_multiplayer_support() {
    // All platforms should support online multiplayer
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        let caps = platform_capabilities(platform);
        assert!(
            caps.supports_feature(Feature::OnlineMultiplayer),
            "{:?} should support online multiplayer",
            platform
        );
    }
}

#[test]
fn test_achievements_support() {
    // All platforms should support achievements
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        let caps = platform_capabilities(platform);
        assert!(
            caps.supports_feature(Feature::Achievements),
            "{:?} should support achievements",
            platform
        );
    }
}

#[test]
fn test_cloud_save_support() {
    // All platforms should support cloud save
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        let caps = platform_capabilities(platform);
        assert!(
            caps.supports_feature(Feature::CloudSave),
            "{:?} should support cloud save",
            platform
        );
    }
}

#[test]
fn test_comparison_table_generation() {
    let matrix = HardwareCapabilityMatrix::new();
    let table = matrix.generate_comparison_table();

    // Check table structure
    assert!(table.contains("| Feature |"));
    assert!(table.contains("Nintendo Switch"));
    assert!(table.contains("PlayStation 5"));
    assert!(table.contains("Xbox"));

    // Check features are present
    assert!(table.contains("RayTracing"));
    assert!(table.contains("HDR"));
    assert!(table.contains("Vibration"));
}

#[test]
fn test_strict_mode_validation() {
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in platforms {
        let validator = CompatibilityValidator::new_strict(platform);
        let report = validator.validate_all();

        // In strict mode, all platforms should still be valid
        // but might have more warnings
        assert!(report.is_valid(), "Platform {:?} failed strict validation", platform);
    }
}

#[test]
fn test_performance_tier_classification() {
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let last_gen_caps = platform_capabilities(Platform::PlayStation4);
    let current_gen_caps = platform_capabilities(Platform::PlayStation5);

    // CPU cores - all have 8 except Switch
    assert_eq!(switch_caps.hardware.cpu_cores, 4);
    assert_eq!(last_gen_caps.hardware.cpu_cores, 8);
    assert_eq!(current_gen_caps.hardware.cpu_cores, 8);

    // CPU frequency increases with generations
    assert!(switch_caps.hardware.cpu_frequency_mhz < last_gen_caps.hardware.cpu_frequency_mhz);
    assert!(last_gen_caps.hardware.cpu_frequency_mhz < current_gen_caps.hardware.cpu_frequency_mhz);
}

#[test]
fn test_texture_size_limits() {
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let ps4_caps = platform_capabilities(Platform::PlayStation4);
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);

    // Current gen has highest texture limits
    assert_eq!(ps5_caps.max_texture_size, 16384);
    assert_eq!(ps4_caps.max_texture_size, 16384);
    assert_eq!(switch_caps.max_texture_size, 8192);
}

#[test]
fn test_cross_platform_feature_parity() {
    // Features that should be available on all platforms
    let universal_features = vec![
        Feature::OnlineMultiplayer,
        Feature::CloudSave,
        Feature::Leaderboards,
        Feature::Achievements,
    ];

    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    for platform in &platforms {
        let caps = platform_capabilities(*platform);
        for feature in &universal_features {
            assert!(
                caps.supports_feature(*feature),
                "{:?} missing feature {:?}",
                platform,
                feature
            );
        }
    }
}

#[test]
fn test_platform_specific_features() {
    // Platform-specific unique features
    let switch_caps = platform_capabilities(Platform::NintendoSwitch);
    let ps5_caps = platform_capabilities(Platform::PlayStation5);
    let ps4_caps = platform_capabilities(Platform::PlayStation4);

    // Switch unique: hybrid portability (implicit)
    assert!(switch_caps.supports_feature(Feature::MotionControls));

    // PlayStation unique: touchpad
    assert!(ps5_caps.supports_feature(Feature::Touchpad));
    assert!(ps4_caps.supports_feature(Feature::Touchpad));

    // Current gen unique: ray tracing
    assert!(ps5_caps.supports_feature(Feature::RayTracing));
    assert!(!ps4_caps.supports_feature(Feature::RayTracing));
}

#[test]
fn test_validation_report_comparison() {
    let platforms = vec![
        Platform::NintendoSwitch,
        Platform::PlayStation5,
        Platform::PlayStation4,
        Platform::XboxSeries,
        Platform::XboxOne,
    ];

    let mut reports = Vec::new();

    for platform in platforms {
        let validator = CompatibilityValidator::new(platform);
        let report = validator.validate_all();
        reports.push((platform, report));
    }

    // All should pass
    for (platform, report) in &reports {
        assert!(
            report.is_valid(),
            "{:?} validation failed: {:?}",
            platform,
            report.errors
        );
    }

    // PS5 should have highest pass rate (most capable)
    let ps5_report = reports.iter().find(|(p, _)| *p == Platform::PlayStation5).unwrap().1;
    let switch_report = reports.iter().find(|(p, _)| *p == Platform::NintendoSwitch).unwrap().1;

    // PS5 should have fewer warnings than Switch
    assert!(ps5_report.warnings.len() <= switch_report.warnings.len());
}
