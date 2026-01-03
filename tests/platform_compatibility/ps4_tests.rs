//! PlayStation 4 Compatibility Tests

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::detection_extended::{Platform, Feature, platform_capabilities};
use game_engine::platform::mock::ps4_mock::PS4MockPlatform;
use game_engine::platform::mock::base_mock::MockPlatform;
use game_engine::platform::validation::CompatibilityValidator;

#[test]
fn test_ps4_platform_detection() {
    let platform = Platform::PlayStation4;
    assert!(platform.is_console());
    assert_eq!(
        platform.as_console_platform(),
        Some(ConsolePlatform::PlayStation4)
    );
}

#[test]
fn test_ps4_capabilities() {
    let caps = platform_capabilities(Platform::PlayStation4);

    // Mid-range hardware specs
    assert_eq!(caps.hardware.cpu_cores, 8);
    assert_eq!(caps.hardware.memory_mb, 8 * 1024);
    assert_eq!(caps.hardware.gpu_memory_mb, 8 * 1024);

    // Standard PS4 features
    assert!(caps.supports_feature(Feature::Touchpad));
    assert!(caps.supports_feature(Feature::MotionControls)); // DualShock 4 gyro
    assert!(!caps.supports_feature(Feature::RayTracing));

    // Graphics capabilities
    assert_eq!(caps.max_texture_size, 16384);
    assert_eq!(caps.max_render_targets, 8);
}

#[test]
fn test_ps4_standard_mock() {
    let mut mock = PS4MockPlatform::new();
    mock.initialize().unwrap();

    assert!(!mock.is_pro());
    assert!(!mock.supports_feature(Feature::HDR));
}

#[test]
fn test_ps4_pro_mock() {
    let mut mock = PS4MockPlatform::new_pro();
    mock.initialize().unwrap();

    assert!(mock.is_pro());
    assert!(mock.supports_feature(Feature::HDR));
}

#[test]
fn test_ps4_pro_features() {
    let mock = PS4MockPlatform::new_pro();

    // PS4 Pro has HDR
    assert!(mock.supports_feature(Feature::HDR));
    assert!(!mock.supports_feature(Feature::RayTracing)); // Still no RT
}

#[test]
fn test_ps4_validation() {
    let validator = CompatibilityValidator::new(Platform::PlayStation4);
    let report = validator.validate_all();

    // PS4 should pass validation
    assert!(report.is_valid());
    assert!(report.passed >= 4);
}

#[test]
fn test_ps4_certification_requirements() {
    let validator = CompatibilityValidator::new_strict(Platform::PlayStation4);

    // PlayStation certification requirements
    let result = validator.validate_certification();
    assert!(result.is_ok());
}

#[test]
fn test_ps4_memory_constraints() {
    let mock = PS4MockPlatform::new();
    mock.initialize().unwrap();

    // Within PS4's 8GB limit
    assert!(mock.set_memory_usage(4 * 1024).is_ok());
    assert!(mock.set_memory_usage(8 * 1024).is_ok());

    // Beyond limit
    assert!(mock.set_memory_usage(16 * 1024).is_err());
}

#[test]
fn test_ps4_controller_features() {
    let mock = PS4MockPlatform::new();
    mock.initialize().unwrap();

    // DualShock 4 features
    assert!(mock.supports_feature(Feature::Vibration));
    assert!(mock.supports_feature(Feature::Touchpad));
    assert!(mock.supports_feature(Feature::MotionControls));
}

#[test]
fn test_ps4_standard_vs_pro() {
    let standard = PS4MockPlatform::new();
    let pro = PS4MockPlatform::new_pro();

    // Both should support same basic features
    assert!(standard.supports_feature(Feature::Touchpad));
    assert!(pro.supports_feature(Feature::Touchpad));

    // Only Pro has HDR
    assert!(!standard.supports_feature(Feature::HDR));
    assert!(pro.supports_feature(Feature::HDR));

    // Neither has ray tracing
    assert!(!standard.supports_feature(Feature::RayTracing));
    assert!(!pro.supports_feature(Feature::RayTracing));
}

#[test]
fn test_ps4_performance() {
    let mut mock = PS4MockPlatform::new();
    mock.initialize().unwrap();

    // Standard PS4 performance
    assert!(mock.update(16.7).is_ok()); // 60 FPS
    assert!(mock.update(33.3).is_ok()); // 30 FPS
}

#[test]
fn test_ps4_boundary_conditions() {
    let mock = PS4MockPlatform::new();
    mock.initialize().unwrap();

    // Test memory boundary
    assert!(mock.set_memory_usage(8 * 1024).is_ok()); // Exactly at limit
    assert!(mock.set_memory_usage(8 * 1024 + 1).is_err()); // Over limit
}

#[test]
fn test_ps4_graphics_limits() {
    let caps = platform_capabilities(Platform::PlayStation4);

    // Texture size
    assert_eq!(caps.max_texture_size, 16384);

    // Render targets
    assert_eq!(caps.max_render_targets, 8);

    // Samplers
    assert_eq!(caps.max_samplers, 32);
}
