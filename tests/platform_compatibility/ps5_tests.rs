//! PlayStation 5 Compatibility Tests

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::detection_extended::{Platform, Feature, platform_capabilities};
use game_engine::platform::mock::ps5_mock::PS5MockPlatform;
use game_engine::platform::mock::base_mock::MockPlatform;
use game_engine::platform::validation::CompatibilityValidator;

#[test]
fn test_ps5_platform_detection() {
    let platform = Platform::PlayStation5;
    assert!(platform.is_console());
    assert_eq!(
        platform.as_console_platform(),
        Some(ConsolePlatform::PlayStation5)
    );
}

#[test]
fn test_ps5_capabilities() {
    let caps = platform_capabilities(Platform::PlayStation5);

    // High-end hardware specs
    assert_eq!(caps.hardware.cpu_cores, 8);
    assert_eq!(caps.hardware.memory_mb, 16 * 1024);
    assert_eq!(caps.hardware.gpu_memory_mb, 16 * 1024);

    // Advanced features
    assert!(caps.supports_feature(Feature::RayTracing));
    assert!(caps.supports_feature(Feature::HDR));
    assert!(caps.supports_feature(Feature::Touchpad));
    assert!(caps.supports_feature(Feature::SpatialAudio));
    assert!(caps.supports_feature(Feature::VoiceChat));

    // Graphics capabilities
    assert_eq!(caps.max_texture_size, 16384);
    assert_eq!(caps.max_samplers, 64);
    assert_eq!(caps.max_render_targets, 8);
}

#[test]
fn test_ps5_mock_initialization() {
    let mut mock = PS5MockPlatform::new();
    assert!(mock.initialize().is_ok());
    assert!(mock.is_initialized());
}

#[test]
fn test_ps5_ray_tracing() {
    let mut mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // Should support ray tracing
    assert!(mock.supports_feature(Feature::RayTracing));

    // Enable ray tracing
    mock.enable_ray_tracing(true);
    assert!(mock.is_ray_tracing_enabled());
}

#[test]
fn test_ps5_ray_tracing_performance() {
    let mut mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // Without ray tracing - high FPS
    mock.enable_ray_tracing(false);
    assert!(mock.update(16.7).is_ok()); // 60 FPS

    // With ray tracing - lower FPS expected
    mock.enable_ray_tracing(true);
    assert!(mock.update(33.3).is_ok()); // 30 FPS with RT
    assert!(mock.update(16.7).is_err()); // 60 FPS too fast with RT
}

#[test]
fn test_ps5_memory_constraints() {
    let mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // Within PS5's 16GB limit
    assert!(mock.set_memory_usage(8 * 1024).is_ok());
    assert!(mock.set_memory_usage(16 * 1024).is_ok());

    // Beyond limit
    assert!(mock.set_memory_usage(32 * 1024).is_err());
}

#[test]
fn test_ps5_controller_features() {
    let mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // DualSense features
    assert!(mock.supports_feature(Feature::Vibration));
    assert!(mock.supports_feature(Feature::Touchpad));
    assert!(mock.supports_feature(Feature::MotionControls)); // Gyroscope
}

#[test]
fn test_ps5_validation() {
    let validator = CompatibilityValidator::new(Platform::PlayStation5);
    let report = validator.validate_all();

    // PS5 should pass all validations
    assert!(report.is_valid());
    assert!(report.is_perfect()); // Should have no warnings
    assert_eq!(report.failed, 0);
}

#[test]
fn test_ps5_certification_requirements() {
    let validator = CompatibilityValidator::new_strict(Platform::PlayStation5);

    // Certification requirements
    assert!(validator.validate_certification().is_ok());

    let report = validator.validate_all();
    assert!(report.is_valid());
}

#[test]
fn test_ps5_high_end_features() {
    let caps = platform_capabilities(Platform::PlayStation5);

    // Check all high-end features
    assert!(caps.supports_feature(Feature::RayTracing));
    assert!(caps.supports_feature(Feature::HDR));
    assert!(caps.supports_feature(Feature::SpatialAudio));
    assert!(caps.supports_feature(Feature::VoiceChat));
    assert!(caps.supports_feature(Feature::CrossPlatformPlay));
}

#[test]
fn test_ps5_performance_monitoring() {
    let mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // Set performance metrics
    mock.set_gpu_usage(0.8).unwrap();
    mock.set_cpu_usage(0.6).unwrap();
    mock.set_memory_usage(8 * 1024).unwrap();

    assert_eq!(mock.gpu_usage(), 0.8);
    assert_eq!(mock.cpu_usage(), 0.6);
    assert_eq!(mock.memory_usage(), 8 * 1024);
}

#[test]
fn test_ps5_boundary_conditions() {
    let mock = PS5MockPlatform::new();
    mock.initialize().unwrap();

    // Test performance constraint boundaries
    mock.set_performance_constraint(game_engine::platform::mock::base_mock::PerformanceConstraint::MaxGpuUsage(0.9));
    assert!(mock.set_gpu_usage(0.8).is_ok());
    assert!(mock.set_gpu_usage(0.95).is_err()); // Over limit
}

#[test]
fn test_ps5_advanced_graphics() {
    let caps = platform_capabilities(Platform::PlayStation5);

    // Advanced graphics capabilities
    assert_eq!(caps.max_texture_size, 16384); // 16K textures
    assert_eq!(caps.max_samplers, 64);
    assert_eq!(caps.max_render_targets, 8);

    // GPU features
    assert!(caps.hardware.gpu_features.contains(&"RayTracing".to_string()));
    assert!(caps.hardware.gpu_features.contains(&"Vulkan".to_string()));
}
