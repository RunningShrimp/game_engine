# Platform Best Practices Guide

Comprehensive guide for developing console games across all supported platforms.

## Table of Contents

1. [Development Workflow](#development-workflow)
2. [Platform Abstraction](#platform-abstraction)
3. [Performance Optimization](#performance-optimization)
4. [Memory Management](#memory-management)
5. [Input Handling](#input-handling)
6. [Certification Compliance](#certification-compliance)
7. [Testing Strategies](#testing-strategies)
8. [Release Management](#release-management)

---

## Development Workflow

### 1. Platform Selection Strategy

**Start with PC Development:**
- Develop core gameplay on PC first
- Use mock platforms to simulate console behavior
- Implement platform abstraction layers from the start

**Example Platform Selection Flow:**
```rust
// Use feature flags for platform-specific code
#[cfg(target_os = "windows")]
fn platform_init() { /* Windows-specific */ }

#[cfg(feature = "ps5")]
fn platform_init() { /* PS5-specific */ }

// Or runtime detection
let platform = detect_platform();
match platform {
    Platform::NintendoSwitch => init_switch(),
    Platform::PlayStation5 => init_ps5(),
    // ...
}
```

### 2. Development Order

**Recommended Sequence:**
1. **PC/Linux** - Core gameplay and engine features
2. **Nintendo Switch** - Ensure performance on lowest spec
3. **PS4/Xbox One** - Last generation optimization
4. **PS5/Xbox Series** - Current generation enhancements

### 3. Build Configuration

**Use Cargo Features:**
```toml
[features]
default = []
ps5 = []
ps4 = []
xbox-series = []
xbox-one = []
switch = []
mock-console = []
```

---

## Platform Abstraction

### 1. Capability Detection

**Always Detect Capabilities at Runtime:**
```rust
use game_engine::platform::detection_extended::*;
use game_engine::platform::validation::*;

fn initialize_graphics() {
    let (platform, caps) = current_platform_info();

    // Configure based on capabilities
    let texture_quality = if caps.max_texture_size >= 16384 {
        TextureQuality::Ultra
    } else if caps.max_texture_size >= 8192 {
        TextureQuality::High
    } else {
        TextureQuality::Medium
    };

    let enable_ray_tracing = caps.supports_feature(Feature::RayTracing);
    let enable_hdr = caps.supports_feature(Feature::HDR);
}
```

### 2. Feature Flags

**Use Feature Flags for Platform-Specific Code:**
```rust
pub trait PlatformRenderer {
    fn render(&mut self, frame: &Frame);
    fn present(&mut self);
}

// Implement for each platform
struct PS5Renderer { /* ... */ }
struct SwitchRenderer { /* ... */ }

// Factory pattern
fn create_renderer(platform: Platform) -> Box<dyn PlatformRenderer> {
    match platform {
        Platform::PlayStation5 => Box::new(PS5Renderer::new()),
        Platform::NintendoSwitch => Box::new(SwitchRenderer::new()),
        _ => panic!("Unsupported platform"),
    }
}
```

### 3. Mock Platforms for Testing

**Use Mock Platforms Early in Development:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use game_engine::platform::mock::ps5_mock::PS5MockPlatform;

    #[test]
    fn test_ps5_performance() {
        let mut mock = PS5MockPlatform::new();
        mock.initialize().unwrap();

        // Test performance constraints
        assert!(mock.update(16.7).is_ok());
    }
}
```

---

## Performance Optimization

### 1. Target Frame Rates

**Platform-Specific Targets:**
```rust
pub fn get_target_fps(platform: Platform) -> u32 {
    match platform {
        // Switch often targets 30 FPS in handheld
        Platform::NintendoSwitch => 30,
        // Current gen targets 60-120 FPS
        Platform::PlayStation5 | Platform::XboxSeries => 60,
        // Last gen targets 30-60 FPS
        Platform::PlayStation4 | Platform::XboxOne => 60,
        _ => 60,
    }
}
```

### 2. Dynamic Resolution Scaling

**Implement for Consistent Performance:**
```rust
fn update_resolution_scale(frame_time_ms: f32, target_ms: f32) -> f32 {
    let scale = if frame_time_ms > target_ms * 1.2 {
        // Too slow, reduce resolution
        0.9
    } else if frame_time_ms < target_ms * 0.8 {
        // Fast enough, increase quality
        1.1
    } else {
        1.0
    };

    (current_scale() * scale).clamp(0.5, 1.0)
}
```

### 3. Level of Detail (LOD) System

**Distance-Based LOD Switching:**
```rust
fn select_lod_distance(platform: Platform) -> f32 {
    match platform {
        Platform::NintendoSwitch => 20.0,  // Closer LOD on Switch
        Platform::PlayStation5 => 50.0,    // Further LOD on PS5
        _ => 30.0,
    }
}
```

### 4. Asset Streaming

**Platform-Specific Streaming Strategies:**
```rust
fn get_streaming_config(platform: Platform) -> StreamingConfig {
    match platform {
        Platform::NintendoSwitch => StreamingConfig {
            buffer_size_mb: 128,
            prefetch_distance: 10.0,
            aggressive_streaming: true,
        },
        Platform::PlayStation5 => StreamingConfig {
            buffer_size_mb: 2048,  // Faster SSD
            prefetch_distance: 50.0,
            aggressive_streaming: false,
        },
        _ => StreamingConfig::default(),
    }
}
```

---

## Memory Management

### 1. Memory Budgets

**Platform-Specific Allocations:**
```rust
fn allocate_game_memory(platform: Platform) -> MemoryBudget {
    match platform {
        Platform::NintendoSwitch => MemoryBudget {
            textures: 1024,      // 1GB
            geometry: 512,        // 512MB
            audio: 256,           // 256MB
            systems: 512,         // 512MB
            total: 4096,          // 4GB total (3.2GB available)
        },
        Platform::PlayStation5 => MemoryBudget {
            textures: 6144,       // 6GB
            geometry: 2048,       // 2GB
            audio: 1024,          // 1GB
            systems: 4096,        // 4GB
            total: 16384,         // 16GB total (13GB available)
        },
        _ => MemoryBudget::default(),
    }
}
```

### 2. Texture Compression

**Use Platform-Optimal Formats:**
```rust
fn get_texture_format(platform: Platform) -> TextureFormat {
    match platform {
        Platform::NintendoSwitch => TextureFormat::ETC2, // Mobile standard
        Platform::PlayStation => TextureFormat::BC7,     // Console standard
        Platform::Xbox => TextureFormat::BC7,
        _ => TextureFormat::BC7,
    }
}
```

### 3. Memory Pools

**Implement Fixed-Size Pools:**
```rust
struct TexturePool {
    max_size_mb: usize,
    current_usage: usize,
    textures: Vec<Texture>,
}

impl TexturePool {
    fn allocate(&mut self, size: usize) -> Option<TextureId> {
        if self.current_usage + size <= self.max_size_mb {
            // Allocate
            Some(self.alloc_internal(size))
        } else {
            // Pool full, evict oldest
            self.evict_oldest();
            self.allocate(size)
        }
    }
}
```

### 4. Garbage Collection

**Platform-Specific GC Strategies:**
```rust
fn gc_strategy(platform: Platform) -> GCStrategy {
    match platform {
        Platform::NintendoSwitch => GCStrategy {
            frequency_ms: 1000,     // More frequent on Switch
            aggressive: true,
        },
        Platform::PlayStation5 => GCStrategy {
            frequency_ms: 5000,     // Less frequent on PS5
            aggressive: false,
        },
        _ => GCStrategy::default(),
    }
}
```

---

## Input Handling

### 1. Controller Support

**Support Multiple Controllers:**
```rust
fn handle_input(platform: Platform) {
    let max_controllers = match platform {
        Platform::NintendoSwitch => 8,   // Switch supports up to 8
        Platform::PlayStation5 => 4,     // PS5 supports up to 4
        Platform::XboxSeries => 8,       // Xbox supports up to 8
        _ => 4,
    };

    for id in 0..max_controllers {
        if let Some(state) = get_controller_state(id) {
            process_controller_input(id, state);
        }
    }
}
```

### 2. Motion Controls

**Platform-Specific Motion Features:**
```rust
fn use_motion_controls(platform: Platform) -> bool {
    match platform {
        // Switch Joy-Con have motion
        Platform::NintendoSwitch => true,
        // PS5 DualSense has gyroscope
        Platform::PlayStation5 => true,
        // PS4 DualShock 4 has gyroscope
        Platform::PlayStation4 => true,
        // Xbox controllers don't have motion
        Platform::XboxSeries | Platform::XboxOne => false,
        _ => false,
    }
}
```

### 3. Vibration

**Platform-Specific Haptics:**
```rust
fn set_vibration(platform: Platform, intensity: f32) {
    match platform {
        Platform::NintendoSwitch => {
            // Switch HD Rumble
            switch_set_rumble(intensity, intensity);
        }
        Platform::PlayStation5 => {
            // PS5 DualSense haptics
            dualsense_set_haptic(intensity);
        }
        Platform::PlayStation4 => {
            // PS4 basic vibration
            ds4_set_vibration(intensity, intensity);
        }
        Platform::XboxSeries | Platform::XboxOne => {
            // Xbox impulse triggers
            xbox_set_vibration(intensity, intensity);
        }
        _ => {}
    }
}
```

---

## Certification Compliance

### 1. Platform-Specific Requirements

**PlayStation Certification:**
```rust
fn validate_ps_certification() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Mandatory trophies
    if !has_trophy_integration() {
        errors.push("Missing trophy integration".into());
    }

    // Network features
    if !has_online_features() && !is_single_player_only() {
        errors.push("Missing online features".into());
    }

    // TRC compliance
    if !check_trc_compliance() {
        errors.push("TRC compliance failed".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

**Xbox Certification:**
```rust
fn validate_xbox_certification() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Mandatory achievements
    if !has_achievements() {
        errors.push("Missing achievements".into());
    }

    // Cloud save
    if !has_cloud_save() {
        errors.push("Missing cloud save".into());
    }

    // XCs compliance
    if !check_xcs_compliance() {
        errors.push("XCs compliance failed".into());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### 2. Pre-Certification Testing

**Use Built-In Validation:**
```rust
use game_engine::platform::validation::CompatibilityValidator;

fn run_pre_certification_checks(platform: Platform) {
    let validator = CompatibilityValidator::new_strict(platform);
    let report = validator.validate_all();

    println!("{}", report);

    if !report.is_valid() {
        panic!("Certification checks failed!");
    }
}
```

### 3. Error Handling

**Robust Error Handling Required:**
```rust
fn handle_platform_error(error: PlatformError) {
    match error {
        PlatformError::NetworkDisconnected => {
            // Show user-friendly message
            show_message("Network disconnected. Please check your connection.");
        }
        PlatformError::StorageFull => {
            // Prompt user to free space
            show_message("Storage full. Please free up space.");
        }
        PlatformError::ControllerDisconnected => {
            // Pause game, show controller disconnected
            pause_game();
            show_message("Controller disconnected. Please reconnect.");
        }
        _ => {
            // Log and show generic message
            log_error(&error);
            show_message("An error occurred. The game will now exit.");
        }
    }
}
```

---

## Testing Strategies

### 1. Platform-Specific Test Suites

**Run Platform Tests:**
```bash
# Run all platform compatibility tests
cargo test --test platform_compatibility

# Run specific platform tests
cargo test --test platform_compatibility switch_tests
cargo test --test platform_compatibility ps5_tests
```

### 2. Performance Profiling

**Profile on Each Platform:**
```rust
use game_engine::platform::console::ConsolePerformanceMonitor;

fn profile_performance(platform: Platform) {
    let mut monitor = ConsolePerformanceMonitor::new();

    for frame in 0..60 {
        let start = Instant::now();

        // Render frame
        render_frame();

        let frame_time = start.elapsed().as_secs_f32() * 1000.0;
        monitor.update_frame_time(frame_time);
    }

    let stats = monitor.get_stats();
    println!("FPS: {:.2}", stats.fps);
    println!("Frame Time: {:.2}ms", stats.frame_time_ms);
}
```

### 3. Memory Profiling

**Track Memory Usage:**
```rust
fn profile_memory(platform: Platform) {
    let initial = get_memory_usage_mb();

    // Load game assets
    load_assets();

    let loaded = get_memory_usage_mb();
    let asset_memory = loaded - initial;

    println!("Asset memory: {}MB", asset_memory);

    // Check against budget
    let budget = get_memory_budget(platform);
    if loaded > budget.total {
        println!("WARNING: Exceeded memory budget!");
    }
}
```

### 4. Automated Testing

**CI/CD Integration:**
```yaml
# .github/workflows/platform-test.yml
name: Platform Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run platform tests
        run: |
          cargo test --test platform_compatibility --verbose
```

---

## Release Management

### 1. Version Control

**Semantic Versioning for Cross-Platform:**
```toml
[package]
name = "my_game"
version = "1.0.0"  # Major.Minor.Patch

# Platform-specific versions in metadata
[package.metadata.platforms]
switch = "1.0.0"
ps5 = "1.0.0"
xbox-series = "1.0.0"
```

### 2. Patch Strategy

**Platform-Specific Patches:**
```rust
#[cfg(feature = "switch")]
fn apply_switch_patches() {
    // Switch-specific bug fixes
}

#[cfg(feature = "ps5")]
fn apply_ps5_patches() {
    // PS5-specific bug fixes
}
```

### 3. Update Deployment

**Staged Rollout:**
1. **PC/Linux** - Initial release (fastest approval)
2. **Xbox** - Typically 1-2 days for certification
3. **PlayStation** - Typically 3-5 days for certification
4. **Switch** - Typically 1-2 weeks for certification

### 4. Patch Notes

**Platform-Specific Notes:**
```markdown
## Patch 1.0.1 (2025-01-15)

### All Platforms
- Fixed crash in level 3
- Improved performance in multiplayer

### PlayStation 5
- Enhanced haptic feedback
- Fixed DualSense button mapping

### Nintendo Switch
- Improved docked mode performance
- Fixed Joy-Con disconnect issue

### Xbox Series X/S
- Added Quick Resume support
- Optimized for Series S
```

---

## Code Examples

### Complete Platform-Aware Initialization

```rust
use game_engine::platform::detection_extended::*;
use game_engine::platform::validation::*;

pub struct GameEngine {
    platform: Platform,
    capabilities: PlatformCapabilities,
}

impl GameEngine {
    pub fn new() -> Result<Self, EngineError> {
        // Detect platform
        let platform = detect_platform_runtime();
        let capabilities = platform_capabilities(platform);

        // Validate compatibility
        let validator = CompatibilityValidator::new(platform);
        let report = validator.validate_all();

        if !report.is_valid() {
            return Err(EngineError::CompatibilityError(report));
        }

        // Log platform info
        log_platform_info(&platform, &capabilities);

        Ok(Self {
            platform,
            capabilities,
        })
    }

    pub fn initialize(&mut self) -> Result<(), EngineError> {
        // Platform-specific initialization
        match self.platform {
            Platform::NintendoSwitch => self.init_switch()?,
            Platform::PlayStation5 => self.init_ps5()?,
            Platform::PlayStation4 => self.init_ps4()?,
            Platform::XboxSeries => self.init_xbox_series()?,
            Platform::XboxOne => self.init_xbox_one()?,
            _ => self.init_generic()?,
        }

        Ok(())
    }

    fn init_switch(&mut self) -> Result<(), EngineError> {
        log::info!("Initializing for Nintendo Switch");

        // Switch-specific optimizations
        set_target_fps(30);  // Conservative target
        enable_aggressive_streaming(true);
        set_texture_quality(TextureQuality::Medium);

        Ok(())
    }

    fn init_ps5(&mut self) -> Result<(), EngineError> {
        log::info!("Initializing for PlayStation 5");

        // PS5-specific features
        if self.capabilities.supports_feature(Feature::RayTracing) {
            enable_ray_tracing(true);
        }
        set_target_fps(60);
        set_texture_quality(TextureQuality::Ultra);

        Ok(())
    }

    // ... other platform init functions ...
}
```

---

## Checklist

### Before Submitting to Certification

- [ ] Run platform-specific test suite
- [ ] Run compatibility validation
- [ ] Test on actual hardware (not just emulators)
- [ ] Verify all certification requirements
- [ ] Test edge cases (network disconnect, low storage, etc.)
- [ ] Check memory usage is within limits
- [ ] Verify frame rate consistency
- [ ] Test with multiple controllers
- [ ] Verify achievement/trophy unlocking
- [ ] Test cloud save functionality
- [ ] Check error messages are platform-appropriate
- [ ] Verify age rating compliance
- [ ] Test on different firmware versions

---

## Resources

- **Platform SDKs:** Download from official developer portals
- **Certification Guidelines:** Available in platform documentation
- **Sample Code:** Check official GitHub repositories
- **Community Forums:** Gamedev.stackexchange, Reddit r/gamedev
- **Profiling Tools:** Use platform-specific profilers

---

## Version History

- **v1.0.0** (2025-01-02): Initial best practices guide
