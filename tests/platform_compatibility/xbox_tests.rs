//! Xbox Compatibility Tests

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::detection_extended::{Platform, Feature, platform_capabilities};
use game_engine::platform::mock::xbox_mock::XboxMockPlatform;
use game_engine::platform::mock::base_mock::MockPlatform;
use game_engine::platform::validation::CompatibilityValidator;

#[test]
fn test_xbox_series_x_detection() {
    let mock = XboxMockPlatform::new_series_x();
    assert_eq!(mock.platform(), Platform::XboxSeries);
    assert!(mock.is_series());
    assert!(mock.is_series_x());
}

#[test]
fn test_xbox_series_s_detection() {
    let mock = XboxMockPlatform::new_series_s();
    assert_eq!(mock.platform(), Platform::XboxSeries);
    assert!(mock.is_series());
    assert!(mock.is_series_s());
    assert!(!mock.is_series_x());
}

#[test]
fn test_xbox_one_detection() {
    let mock = XboxMockPlatform::new_xbox_one();
    assert_eq!(mock.platform(), Platform::XboxOne);
    assert!(!mock.is_series());
}

#[test]
fn test_xbox_series_x_capabilities() {
    let caps = platform_capabilities(Platform::XboxSeries);

    // High-end specs
    assert_eq!(caps.hardware.cpu_cores, 8);
    assert_eq!(caps.hardware.memory_mb, 16 * 1024);
    assert_eq!(caps.hardware.gpu_memory_mb, 16 * 1024);

    // Advanced features
    assert!(caps.supports_feature(Feature::RayTracing));
    assert!(caps.supports_feature(Feature::HDR));
    assert!(caps.supports_feature(Feature::SpatialAudio));
    assert!(caps.supports_feature(Feature::VoiceChat));
    assert!(caps.supports_feature(Feature::LanMultiplayer));
}

#[test]
fn test_xbox_series_x_ray_tracing() {
    let mock = XboxMockPlatform::new_series_x();
    mock.initialize().unwrap();

    // Series X has full ray tracing support
    assert!(mock.supports_feature(Feature::RayTracing));
}

#[test]
fn test_xbox_series_s_no_ray_tracing() {
    let mock = XboxMockPlatform::new_series_s();
    mock.initialize().unwrap();

    // Series S has limited or no ray tracing
    assert!(!mock.supports_feature(Feature::RayTracing));
}

#[test]
fn test_xbox_one_capabilities() {
    let caps = platform_capabilities(Platform::XboxOne);

    // Last-gen specs
    assert_eq!(caps.hardware.cpu_cores, 8);
    assert_eq!(caps.hardware.memory_mb, 8 * 1024);

    // Xbox One features
    assert!(caps.supports_feature(Feature::HDR));
    assert!(caps.supports_feature(Feature::LanMultiplayer));
    assert!(!caps.supports_feature(Feature::RayTracing));
}

#[test]
fn test_xbox_series_validation() {
    let validator = CompatibilityValidator::new(Platform::XboxSeries);
    let report = validator.validate_all();

    // Xbox Series should pass all validations
    assert!(report.is_valid());
    assert!(report.passed >= 5);
}

#[test]
fn test_xbox_one_validation() {
    let validator = CompatibilityValidator::new(Platform::XboxOne);
    let report = validator.validate_all();

    // Xbox One should pass validation
    assert!(report.is_valid());
    assert!(report.passed >= 4);
}

#[test]
fn test_xbox_certification_requirements() {
    let validator = CompatibilityValidator::new_strict(Platform::XboxSeries);

    // Xbox certification requires cloud save
    let result = validator.validate_certification();
    assert!(result.is_ok());
}

#[test]
fn test_xbox_series_x_memory() {
    let mock = XboxMockPlatform::new_series_x();
    mock.initialize().unwrap();

    // Within Series X's 16GB limit
    assert!(mock.set_memory_usage(8 * 1024).is_ok());
    assert!(mock.set_memory_usage(16 * 1024).is_ok());

    // Beyond limit
    assert!(mock.set_memory_usage(32 * 1024).is_err());
}

#[test]
fn test_xbox_one_memory() {
    let mock = XboxMockPlatform::new_xbox_one();
    mock.initialize().unwrap();

    // Within Xbox One's 8GB limit
    assert!(mock.set_memory_usage(4 * 1024).is_ok());
    assert!(mock.set_memory_usage(8 * 1024).is_ok());

    // Beyond limit
    assert!(mock.set_memory_usage(16 * 1024).is_err());
}

#[test]
fn test_xbox_cross_platform_play() {
    let series_x = XboxMockPlatform::new_series_x();
    let xbox_one = XboxMockPlatform::new_xbox_one();

    // Both should support cross-platform play
    assert!(series_x.supports_feature(Feature::CrossPlatformPlay));
    assert!(xbox_one.supports_feature(Feature::CrossPlatformPlay));
}

#[test]
fn test_xbox_lan_support() {
    let series = XboxMockPlatform::new_series_x();
    let one = XboxMockPlatform::new_xbox_one();

    // Both Xbox platforms support LAN
    assert!(series.supports_feature(Feature::LanMultiplayer));
    assert!(one.supports_feature(Feature::LanMultiplayer));
}

#[test]
fn test_xbox_series_x_performance() {
    let mut mock = XboxMockPlatform::new_series_x();
    mock.initialize().unwrap();

    // Series X performance
    assert!(mock.update(16.7).is_ok()); // 60 FPS
    assert!(mock.update(33.3).is_ok()); // 30 FPS
}

#[test]
fn test_xbox_series_s_performance() {
    let mut mock = XboxMockPlatform::new_series_s();
    mock.initialize().unwrap();

    // Series S targets lower resolution but same framerate
    assert!(mock.update(16.7).is_ok()); // 60 FPS
}

#[test]
fn test_xbox_boundary_conditions() {
    let series_x = XboxMockPlatform::new_series_x();
    let xbox_one = XboxMockPlatform::new_xbox_one();

    series_x.initialize().unwrap();
    xbox_one.initialize().unwrap();

    // Test Series X memory boundary
    assert!(series_x.set_memory_usage(16 * 1024).is_ok());
    assert!(series_x.set_memory_usage(16 * 1024 + 1).is_err());

    // Test Xbox One memory boundary
    assert!(xbox_one.set_memory_usage(8 * 1024).is_ok());
    assert!(xbox_one.set_memory_usage(8 * 1024 + 1).is_err());
}

#[test]
fn test_xbox_graphics_comparison() {
    let series_x_caps = platform_capabilities(Platform::XboxSeries);
    let xbox_one_caps = platform_capabilities(Platform::XboxOne);

    // Series X has better graphics capabilities
    assert_eq!(series_x_caps.max_samplers, 64);
    assert_eq!(xbox_one_caps.max_samplers, 32);

    // Both support same texture size
    assert_eq!(series_x_caps.max_texture_size, 16384);
    assert_eq!(xbox_one_caps.max_texture_size, 16384);
}

#[test]
fn test_xbox_remote_play() {
    let series = XboxMockPlatform::new_series_x();
    let one = XboxMockPlatform::new_xbox_one();

    // Remote play support
    assert!(series.supports_feature(Feature::RemotePlay));
    assert!(one.supports_feature(Feature::RemotePlay));
}
