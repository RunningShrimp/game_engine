//! Nintendo Switch Compatibility Tests

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::detection_extended::{Platform, Feature, platform_capabilities};
use game_engine::platform::mock::switch_mock::SwitchMockPlatform;
use game_engine::platform::mock::base_mock::MockPlatform;
use game_engine::platform::validation::CompatibilityValidator;

#[test]
fn test_switch_platform_detection() {
    let platform = Platform::NintendoSwitch;
    assert!(platform.is_console());
    assert!(!platform.is_mobile());
    assert!(!platform.is_desktop());

    assert_eq!(
        platform.as_console_platform(),
        Some(ConsolePlatform::NintendoSwitch)
    );
}

#[test]
fn test_switch_capabilities() {
    let caps = platform_capabilities(Platform::NintendoSwitch);

    // Hardware specs
    assert_eq!(caps.hardware.cpu_cores, 4);
    assert_eq!(caps.hardware.memory_mb, 4 * 1024);
    assert_eq!(caps.hardware.gpu_memory_mb, 4 * 1024); // Shared memory

    // Features
    assert!(caps.supports_feature(Feature::MotionControls));
    assert!(caps.supports_feature(Feature::Vibration));
    assert!(caps.supports_feature(Feature::OnlineMultiplayer));
    assert!(!caps.supports_feature(Feature::RayTracing));
    assert!(!caps.supports_feature(Feature::HDR));

    // Graphics limits
    assert_eq!(caps.max_texture_size, 8192);
    assert_eq!(caps.max_render_targets, 4);
}

#[test]
fn test_switch_mock_initialization() {
    let mut mock = SwitchMockPlatform::new();
    assert!(mock.initialize().is_ok());
    assert!(mock.is_initialized());
}

#[test]
fn test_switch_docked_mode() {
    let mut mock = SwitchMockPlatform::new();

    // Default handheld mode
    assert!(!mock.is_docked());

    // Switch to docked mode
    mock.set_docked(true);
    assert!(mock.is_docked());
}

#[test]
fn test_switch_performance_constraints() {
    let mut mock = SwitchMockPlatform::new();
    mock.initialize().unwrap();

    // Handheld mode - 30 FPS
    mock.set_docked(false);
    assert!(mock.update(33.3).is_ok()); // 30 FPS
    assert!(mock.update(50.0).is_err()); // Too slow

    // Docked mode - 60 FPS
    mock.set_docked(true);
    assert!(mock.update(16.7).is_ok()); // 60 FPS
    assert!(mock.update(25.0).is_err()); // Too slow for docked
}

#[test]
fn test_switch_memory_constraints() {
    let mock = SwitchMockPlatform::new();
    mock.initialize().unwrap();

    // Within limits
    assert!(mock.set_memory_usage(2048).is_ok());

    // Beyond Switch's 4GB limit
    assert!(mock.set_memory_usage(8 * 1024).is_err());
}

#[test]
fn test_switch_controller_simulation() {
    let mock = SwitchMockPlatform::new();
    mock.initialize().unwrap();

    // Add Joy-Con controller
    mock.add_controller(0);

    // Test button inputs
    mock.press_button(0, game_engine::platform::mock::base_mock::MockButton::A);
    let state = mock.get_controller(0).unwrap();
    assert!(state.buttons.a);

    // Test motion controls feature
    assert!(mock.supports_feature(Feature::MotionControls));
}

#[test]
fn test_switch_validation() {
    let validator = CompatibilityValidator::new(Platform::NintendoSwitch);
    let report = validator.validate_all();

    // Switch should pass basic validation
    assert!(report.is_valid());
    assert!(report.passed >= 4);
}

#[test]
fn test_switch_compatibility_report() {
    let validator = CompatibilityValidator::new_strict(Platform::NintendoSwitch);
    let report = validator.validate_all();

    // Should have warnings about missing HDR and RayTracing
    assert!(report.warnings.len() > 0);
    assert!(report.is_valid());
}

#[test]
fn test_switch_boundary_conditions() {
    let mut mock = SwitchMockPlatform::new();
    mock.initialize().unwrap();

    // Test memory boundary
    assert!(mock.set_memory_usage(4 * 1024).is_ok()); // Exactly at limit
    assert!(mock.set_memory_usage(4 * 1024 + 1).is_err()); // Over limit

    // Test GPU usage boundary
    assert!(mock.set_gpu_usage(0.0).is_ok());
    assert!(mock.set_gpu_usage(1.0).is_ok());
    assert!(mock.set_gpu_usage(1.5).is_ok()); // Clamped to 1.0
}

#[test]
fn test_switch_error_handling() {
    let mut mock = SwitchMockPlatform::new();
    mock.initialize().unwrap();

    // Test non-existent controller
    assert!(mock.get_controller(99).is_none());

    // Test memory overflow
    let result = mock.set_memory_usage(16 * 1024);
    assert!(result.is_err());
}
