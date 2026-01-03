# Platform Hardware Compatibility System - Quick Reference

## Quick Start

```rust
// 1. Detect Platform
use game_engine::platform::detection_extended::*;
let (platform, caps) = current_platform_info();

// 2. Check Features
if caps.supports_feature(Feature::RayTracing) {
    enable_ray_tracing();
}

// 3. Validate Compatibility
use game_engine::platform::validation::*;
let validator = CompatibilityValidator::new(platform);
let report = validator.validate_all();
assert!(report.is_valid());

// 4. Use Mock (Testing)
use game_engine::platform::mock::ps5_mock::PS5MockPlatform;
let mut mock = PS5MockPlatform::new();
mock.initialize().unwrap();
```

## File Structure

```
game_engine/
├── src/platform/
│   ├── detection_extended.rs    (Platform detection)
│   ├── validation.rs             (Compatibility validation)
│   └── mock/                     (Platform simulators)
│       ├── base_mock.rs
│       ├── switch_mock.rs
│       ├── ps5_mock.rs
│       ├── ps4_mock.rs
│       └── xbox_mock.rs
└── tests/platform_compatibility/
    ├── switch_tests.rs
    ├── ps5_tests.rs
    ├── ps4_tests.rs
    ├── xbox_tests.rs
    └── cross_platform_tests.rs

docs/
├── PLATFORM_COMPATIBILITY_MATRIX.md    (Hardware specs)
├── PLATFORM_LIMITATIONS.md             (Known issues)
├── PLATFORM_BEST_PRACTICES.md          (Dev guide)
└── PLATFORM_HARDWARE_COMPATIBILITY_SUMMARY.md
```

## Platform Specifications

| Platform | RAM | GPU | RT | HDR | Target FPS |
|----------|-----|-----|----|---- |------------|
| Switch   | 4GB | Tegra X1 | ❌ | ❌ | 30/60 |
| PS5      | 16GB | RDNA 2 | ✅ | ✅ | 60-120 |
| PS4      | 8GB | GCN | ❌ | ✅* | 30-60 |
| Xbox Series X | 16GB | RDNA 2 | ✅ | ✅ | 60-120 |
| Xbox Series S | 10GB | RDNA 2 | ❌ | ✅ | 60-120 |
| Xbox One | 8GB | GCN | ❌ | ✅ | 30-60 |

*PS4 Pro only

## Feature Detection

```rust
// Graphics Features
Feature::RayTracing        // PS5, Xbox Series X
Feature::HDR               // PS4/5, Xbox One/Series
Feature::VSync             // All platforms

// Controller Features
Feature::Vibration         // All platforms
Feature::MotionControls    // Switch, PS4/5
Feature::Touchpad          // PS4/5

// Network Features
Feature::OnlineMultiplayer // All platforms
Feature::LanMultiplayer    // Xbox only
Feature::CloudSave         // All platforms
Feature::CrossPlatformPlay // All platforms
```

## Validation Tests

```bash
# All platform tests
cargo test --test platform_compatibility

# Specific platform
cargo test --test platform_compatibility switch_tests
cargo test --test platform_compatibility ps5_tests
cargo test --test platform_compatibility cross_platform_tests
```

## Memory Budgets

```rust
fn get_memory_budget(platform: Platform) -> usize {
    match platform {
        Platform::NintendoSwitch => 3 * 1024,  // 3GB
        Platform::PlayStation4 => 5 * 1024,    // 5GB
        Platform::PlayStation5 => 13 * 1024,   // 13GB
        Platform::XboxSeries => 13 * 1024,     // 13GB
        Platform::XboxOne => 5 * 1024,         // 5GB
        _ => 2 * 1024,
    }
}
```

## Performance Targets

```rust
fn get_target_fps(platform: Platform) -> u32 {
    match platform {
        Platform::NintendoSwitch => 30,  // Handheld default
        Platform::PlayStation5 => 60,    // Can do 120
        Platform::PlayStation4 => 60,
        Platform::XboxSeries => 60,       // Can do 120
        Platform::XboxOne => 60,
        _ => 60,
    }
}
```

## Mock Platform Usage

```rust
// Switch: Docked/Handheld modes
let mut mock = SwitchMockPlatform::new();
mock.set_docked(true);

// PS5: Ray tracing
let mut mock = PS5MockPlatform::new();
mock.enable_ray_tracing(true);

// PS4: Pro variant
let mock = PS4MockPlatform::new_pro();

// Xbox: Series X/S/One
let mock = XboxMockPlatform::new_series_x();
let mock = XboxMockPlatform::new_series_s();
let mock = XboxMockPlatform::new_xbox_one();
```

## Validation Categories

1. **Controller API** - Input system compatibility
2. **Certification** - Platform requirements
3. **GPU** - Graphics capability validation
4. **Memory** - Memory constraint checking
5. **Performance** - CPU and performance validation

## Common Issues

**Out of Memory:**
```rust
// Switch: Use aggressive streaming
// PS4/Xbox: Optimize textures
// PS5/Xbox Series: Can use higher quality
```

**Performance Drops:**
```rust
// Implement dynamic resolution
// Use LOD system
// Reduce particle count
// Optimize physics
```

**Certification Failures:**
```rust
// Run validator in strict mode
let validator = CompatibilityValidator::new_strict(platform);
let report = validator.validate_all();
```

## Resources

- **Full Documentation:** See docs/PLATFORM_*.md
- **Tests:** tests/platform_compatibility/
- **Examples:** Check inline documentation in source files
- **Issues:** docs/PLATFORM_LIMITATIONS.md

## Version

- **Release:** v1.0.0
- **Date:** 2025-01-02
- **Platforms:** 5 console platforms
- **Tests:** 200+ test cases
- **Documentation:** 4 comprehensive guides
