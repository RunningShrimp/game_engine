//! # Console Platform Support
//!
//! Provides console platform-specific functionality for Nintendo Switch, PlayStation, and Xbox.

pub mod achievements;
pub mod certification;
pub mod cloud_save;
pub mod controller_extended;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Console platform types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsolePlatform {
    /// Nintendo Switch
    NintendoSwitch,
    /// PlayStation 5
    PlayStation5,
    /// PlayStation 4
    PlayStation4,
    /// Xbox Series X/S
    XboxSeries,
    /// Xbox One
    XboxOne,
}

impl ConsolePlatform {
    /// Get the platform name
    pub fn name(&self) -> &str {
        match self {
            ConsolePlatform::NintendoSwitch => "Nintendo Switch",
            ConsolePlatform::PlayStation5 => "PlayStation 5",
            ConsolePlatform::PlayStation4 => "PlayStation 4",
            ConsolePlatform::XboxSeries => "Xbox Series X/S",
            ConsolePlatform::XboxOne => "Xbox One",
        }
    }

    /// Get the maximum memory available on this platform
    pub fn max_memory_mb(&self) -> usize {
        match self {
            ConsolePlatform::NintendoSwitch => 4 * 1024, // 4GB
            ConsolePlatform::PlayStation5 => 16 * 1024,  // 16GB
            ConsolePlatform::PlayStation4 => 8 * 1024,   // 8GB
            ConsolePlatform::XboxSeries => 16 * 1024,    // 16GB (Series X) / 10GB (Series S)
            ConsolePlatform::XboxOne => 8 * 1024,        // 8GB
        }
    }

    /// Get the target resolution for this platform
    pub fn target_resolution(&self) -> (u32, u32) {
        match self {
            ConsolePlatform::NintendoSwitch => (1920, 1080), // Up to 1080p in docked mode
            ConsolePlatform::PlayStation5 => (3840, 2160),   // 4K
            ConsolePlatform::PlayStation4 => (1920, 1080),   // 1080p
            ConsolePlatform::XboxSeries => (3840, 2160),     // 4K (Series X) / 1440p (Series S)
            ConsolePlatform::XboxOne => (1920, 1080),        // 1080p
        }
    }

    /// Check if this platform supports HDR
    pub fn supports_hdr(&self) -> bool {
        matches!(
            self,
            ConsolePlatform::PlayStation5
                | ConsolePlatform::PlayStation4
                | ConsolePlatform::XboxSeries
                | ConsolePlatform::XboxOne
        )
    }

    /// Check if this platform supports ray tracing
    pub fn supports_ray_tracing(&self) -> bool {
        matches!(
            self,
            ConsolePlatform::PlayStation5 | ConsolePlatform::XboxSeries
        )
    }
}

/// Console platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    /// Platform type
    pub platform: ConsolePlatform,
    /// Target FPS
    pub target_fps: u32,
    /// Performance mode (prioritize FPS over quality)
    pub performance_mode: bool,
    /// Quality mode (prioritize quality over FPS)
    pub quality_mode: bool,
    /// Enable HDR
    pub enable_hdr: bool,
    /// Enable ray tracing (if supported)
    pub enable_ray_tracing: bool,
    /// VSync enabled
    pub vsync_enabled: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            platform: ConsolePlatform::PlayStation5,
            target_fps: 60,
            performance_mode: false,
            quality_mode: false,
            enable_hdr: false,
            enable_ray_tracing: false,
            vsync_enabled: true,
        }
    }
}

impl ConsoleConfig {
    /// Create configuration from platform type
    pub fn from_platform(platform: ConsolePlatform) -> Self {
        let target_fps = match platform {
            ConsolePlatform::NintendoSwitch => 30, // Switch often targets 30 FPS
            _ => 60,
        };

        Self {
            platform,
            target_fps,
            performance_mode: false,
            quality_mode: false,
            enable_hdr: platform.supports_hdr(),
            enable_ray_tracing: platform.supports_ray_tracing(),
            vsync_enabled: true,
        }
    }

    /// Create configuration from extended platform type
    pub fn from_extended_platform(platform: crate::platform::detection_extended::Platform) -> Self {
        // Convert extended Platform to ConsolePlatform
        let console_platform = match platform {
            crate::platform::detection_extended::Platform::NintendoSwitch => {
                ConsolePlatform::NintendoSwitch
            }
            crate::platform::detection_extended::Platform::PlayStation5 => {
                ConsolePlatform::PlayStation5
            }
            crate::platform::detection_extended::Platform::PlayStation4 => {
                ConsolePlatform::PlayStation4
            }
            crate::platform::detection_extended::Platform::XboxSeries => {
                ConsolePlatform::XboxSeries
            }
            crate::platform::detection_extended::Platform::XboxOne => ConsolePlatform::XboxOne,
            _ => ConsolePlatform::PlayStation5, // Default to PS5 for non-console platforms
        };

        Self::from_platform(console_platform)
    }

    /// Apply console-specific settings to graphics config
    pub fn apply_to_graphics_config(&self, graphics_config: &mut GraphicsConfig) {
        graphics_config.target_fps = self.target_fps;

        if self.performance_mode {
            // Lower resolution for better performance
            graphics_config.resolution_scale = 0.8;
            graphics_config.shadows_enabled = false;
            graphics_config.reflections_enabled = false;
        }

        if self.quality_mode {
            // Higher quality settings
            graphics_config.resolution_scale = 1.0;
            graphics_config.shadows_enabled = true;
            graphics_config.reflections_enabled = true;
            graphics_config.anti_aliasing = AntiAliasing::TAA;
        }

        graphics_config.hdr_enabled = self.enable_hdr;
        graphics_config.ray_tracing_enabled = self.enable_ray_tracing;
        graphics_config.vsync_enabled = self.vsync_enabled;
    }
}

/// Graphics configuration placeholder
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    pub target_fps: u32,
    pub resolution_scale: f32,
    pub shadows_enabled: bool,
    pub reflections_enabled: bool,
    pub anti_aliasing: AntiAliasing,
    pub hdr_enabled: bool,
    pub ray_tracing_enabled: bool,
    pub vsync_enabled: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            resolution_scale: 1.0,
            shadows_enabled: true,
            reflections_enabled: true,
            anti_aliasing: AntiAliasing::FXAA,
            hdr_enabled: false,
            ray_tracing_enabled: false,
            vsync_enabled: true,
        }
    }
}

/// Anti-aliasing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiAliasing {
    Off,
    FXAA,
    TAA,
    MSAAx2,
    MSAAx4,
    MSAAx8,
}

/// Button state for controller
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub left_bumper: bool,
    pub right_bumper: bool,
    pub left_trigger: bool,
    pub right_trigger: bool,
    pub left_stick_click: bool,
    pub right_stick_click: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub menu: bool,
    pub view: bool,
}

/// Controller state
#[derive(Debug, Clone, Copy)]
pub struct ControllerState {
    pub id: u32,
    pub connected: bool,
    pub left_stick: (f32, f32),
    pub right_stick: (f32, f32),
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub buttons: ButtonState,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            id: 0,
            connected: false,
            left_stick: (0.0, 0.0),
            right_stick: (0.0, 0.0),
            left_trigger: 0.0,
            right_trigger: 0.0,
            buttons: ButtonState::default(),
        }
    }
}

/// Console input handler
pub struct ConsoleInputHandler {
    controllers: HashMap<u32, ControllerState>,
}

impl ConsoleInputHandler {
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
        }
    }

    /// Update controller state
    pub fn update_controller(&mut self, id: u32, state: ControllerState) {
        self.controllers.insert(id, state);
    }

    /// Get controller state
    pub fn get_controller(&self, id: u32) -> Option<&ControllerState> {
        self.controllers.get(&id)
    }

    /// Check if button is pressed
    pub fn is_button_pressed(&self, controller_id: u32, button: Button) -> bool {
        if let Some(controller) = self.get_controller(controller_id) {
            match button {
                Button::A => controller.buttons.a,
                Button::B => controller.buttons.b,
                Button::X => controller.buttons.x,
                Button::Y => controller.buttons.y,
                Button::LeftBumper => controller.buttons.left_bumper,
                Button::RightBumper => controller.buttons.right_bumper,
                Button::LeftTrigger => controller.buttons.left_trigger,
                Button::RightTrigger => controller.buttons.right_trigger,
                Button::LeftStick => controller.buttons.left_stick_click,
                Button::RightStick => controller.buttons.right_stick_click,
                Button::DPadUp => controller.buttons.dpad_up,
                Button::DPadDown => controller.buttons.dpad_down,
                Button::DPadLeft => controller.buttons.dpad_left,
                Button::DPadRight => controller.buttons.dpad_right,
                Button::Menu => controller.buttons.menu,
                Button::View => controller.buttons.view,
            }
        } else {
            false
        }
    }

    /// Get stick position
    pub fn get_stick_position(&self, controller_id: u32, stick: Stick) -> Option<(f32, f32)> {
        if let Some(controller) = self.get_controller(controller_id) {
            Some(match stick {
                Stick::Left => controller.left_stick,
                Stick::Right => controller.right_stick,
            })
        } else {
            None
        }
    }

    /// Get trigger value
    pub fn get_trigger_value(&self, controller_id: u32, trigger: Trigger) -> Option<f32> {
        if let Some(controller) = self.get_controller(controller_id) {
            Some(match trigger {
                Trigger::Left => controller.left_trigger,
                Trigger::Right => controller.right_trigger,
            })
        } else {
            None
        }
    }
}

impl Default for ConsoleInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Controller buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    A,
    B,
    X,
    Y,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Menu,
    View,
}

/// Analog sticks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stick {
    Left,
    Right,
}

/// Triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Left,
    Right,
}

/// Console performance monitor
pub struct ConsolePerformanceMonitor {
    frame_times: Vec<f32>,
    max_frame_time_samples: usize,
    gpu_usage: f32,
    cpu_usage: f32,
    memory_usage_mb: usize,
}

impl ConsolePerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            max_frame_time_samples: 60,
            gpu_usage: 0.0,
            cpu_usage: 0.0,
            memory_usage_mb: 0,
        }
    }

    /// Update frame time
    pub fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push(frame_time_ms);
        if self.frame_times.len() > self.max_frame_time_samples {
            self.frame_times.remove(0);
        }
    }

    /// Update GPU usage
    pub fn update_gpu_usage(&mut self, usage: f32) {
        self.gpu_usage = usage;
    }

    /// Update CPU usage
    pub fn update_cpu_usage(&mut self, usage: f32) {
        self.cpu_usage = usage;
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, usage_mb: usize) {
        self.memory_usage_mb = usage_mb;
    }

    /// Get current FPS
    pub fn current_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time: f32 =
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        1000.0 / avg_frame_time
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Check for performance issues
    pub fn check_performance_issues(&self, target_fps: u32) -> bool {
        let current_fps = self.current_fps();
        let target_fps_f32 = target_fps as f32;

        // Check if FPS is significantly below target
        if current_fps < target_fps_f32 * 0.9 {
            return true;
        }

        // Check if GPU usage is too high
        if self.gpu_usage > 0.95 {
            return true;
        }

        // Check if CPU usage is too high
        if self.cpu_usage > 0.95 {
            return true;
        }

        false
    }

    /// Get performance stats
    pub fn get_stats(&self) -> PerformanceStats {
        PerformanceStats {
            fps: self.current_fps(),
            frame_time_ms: self.average_frame_time(),
            gpu_usage: self.gpu_usage,
            cpu_usage: self.cpu_usage,
            memory_usage_mb: self.memory_usage_mb,
        }
    }
}

impl Default for ConsolePerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics
#[derive(Debug, Clone, Copy)]
pub struct PerformanceStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_usage: f32,
    pub cpu_usage: f32,
    pub memory_usage_mb: usize,
}

/// Check if current platform is a console platform
pub fn is_console_platform() -> bool {
    cfg!(any(
        target_os = "psp",
        target_os = "horizon",
        target_os = "psx"
    ))
}

/// Get console configuration for the current platform
pub fn get_console_config() -> Option<ConsoleConfig> {
    if is_console_platform() {
        let platform = if cfg!(target_os = "psp") {
            ConsolePlatform::NintendoSwitch
        } else if cfg!(target_os = "horizon") {
            ConsolePlatform::NintendoSwitch
        } else if cfg!(target_os = "psx") {
            ConsolePlatform::PlayStation4
        } else {
            return None;
        };

        Some(ConsoleConfig::from_platform(platform))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_platform_info() {
        let platform = ConsolePlatform::PlayStation5;
        assert_eq!(platform.name(), "PlayStation 5");
        assert_eq!(platform.max_memory_mb(), 16 * 1024);
        assert!(platform.supports_hdr());
        assert!(platform.supports_ray_tracing());
    }

    #[test]
    fn test_console_config() {
        let config = ConsoleConfig::from_platform(ConsolePlatform::NintendoSwitch);
        assert_eq!(config.platform, ConsolePlatform::NintendoSwitch);
        assert_eq!(config.target_fps, 30);
        assert!(!config.enable_hdr);
        assert!(!config.enable_ray_tracing);
    }

    #[test]
    fn test_input_handler() {
        let mut handler = ConsoleInputHandler::new();

        let state = ControllerState {
            id: 0,
            connected: true,
            left_stick: (0.5, 0.3),
            right_stick: (0.0, 0.0),
            left_trigger: 0.7,
            right_trigger: 0.0,
            buttons: ButtonState {
                a: true,
                ..Default::default()
            },
        };

        handler.update_controller(0, state);

        assert!(handler.is_button_pressed(0, Button::A));
        assert!(!handler.is_button_pressed(0, Button::B));
        assert_eq!(handler.get_stick_position(0, Stick::Left), Some((0.5, 0.3)));
        assert_eq!(handler.get_trigger_value(0, Trigger::Left), Some(0.7));
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = ConsolePerformanceMonitor::new();

        // Simulate 60 FPS
        for _ in 0..60 {
            monitor.update_frame_time(16.7);
        }

        let fps = monitor.current_fps();
        assert!((fps - 60.0).abs() < 1.0);

        monitor.update_gpu_usage(0.8);
        monitor.update_cpu_usage(0.6);
        monitor.update_memory_usage(2048);

        let stats = monitor.get_stats();
        assert!((stats.fps - 60.0).abs() < 1.0);
        assert_eq!(stats.gpu_usage, 0.8);
        assert_eq!(stats.cpu_usage, 0.6);
        assert_eq!(stats.memory_usage_mb, 2048);
    }
}
