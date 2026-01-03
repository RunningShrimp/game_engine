//! # Base Mock Platform
//!
//! Base traits and structures for platform mocking.

use crate::platform::console::{ButtonState, ConsolePlatform, ControllerState};
use crate::platform::detection_extended::{Feature, Platform};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock platform trait
pub trait MockPlatform: Send + Sync {
    /// Get the platform type
    fn platform(&self) -> Platform;

    /// Get the console platform type
    fn console_platform(&self) -> ConsolePlatform;

    /// Initialize the mock platform
    fn initialize(&mut self) -> Result<(), MockError>;

    /// Update the mock platform state
    fn update(&mut self, delta_time: f32) -> Result<(), MockError>;

    /// Check if feature is supported
    fn supports_feature(&self, feature: Feature) -> bool;

    /// Get current memory usage (MB)
    fn memory_usage(&self) -> usize;

    /// Get current GPU usage (0.0 - 1.0)
    fn gpu_usage(&self) -> f32;

    /// Get current CPU usage (0.0 - 1.0)
    fn cpu_usage(&self) -> f32;

    /// Set performance constraint
    fn set_performance_constraint(&mut self, constraint: PerformanceConstraint);

    /// Get controller state
    fn get_controller(&self, id: u32) -> Option<ControllerState>;

    /// Set controller state
    fn set_controller(&mut self, id: u32, state: ControllerState);
}

/// Mock error types
#[derive(Debug, thiserror::Error)]
pub enum MockError {
    #[error("Platform not initialized")]
    NotInitialized,

    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    #[error("Controller not found: {0}")]
    ControllerNotFound(u32),

    #[error("Memory limit exceeded: {0}MB > {1}MB")]
    MemoryLimitExceeded(usize, usize),

    #[error("Performance constraint violation: {0}")]
    PerformanceConstraintViolation(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Performance constraint types
#[derive(Debug, Clone, Copy)]
pub enum PerformanceConstraint {
    /// No constraints
    None,

    /// Limit frame time (ms)
    MaxFrameTime(f32),

    /// Limit memory usage (MB)
    MaxMemoryUsage(usize),

    /// Limit GPU usage (0.0 - 1.0)
    MaxGpuUsage(f32),

    /// Limit CPU usage (0.0 - 1.0)
    MaxCpuUsage(f32),
}

/// Base mock platform implementation
#[derive(Clone)]
pub struct BaseMockPlatform {
    pub platform: Platform,
    pub console_platform: ConsolePlatform,
    pub controllers: Arc<Mutex<HashMap<u32, ControllerState>>>,
    pub memory_usage_mb: Arc<Mutex<usize>>,
    pub max_memory_mb: usize,
    pub gpu_usage: Arc<Mutex<f32>>,
    pub cpu_usage: Arc<Mutex<f32>>,
    pub performance_constraint: Arc<Mutex<PerformanceConstraint>>,
    pub initialized: Arc<Mutex<bool>>,
}

impl BaseMockPlatform {
    /// Create a new base mock platform
    pub fn new(console_platform: ConsolePlatform) -> Self {
        let platform = match console_platform {
            ConsolePlatform::NintendoSwitch => Platform::NintendoSwitch,
            ConsolePlatform::PlayStation5 => Platform::PlayStation5,
            ConsolePlatform::PlayStation4 => Platform::PlayStation4,
            ConsolePlatform::XboxSeries => Platform::XboxSeries,
            ConsolePlatform::XboxOne => Platform::XboxOne,
        };

        let max_memory_mb = console_platform.max_memory_mb();

        Self {
            platform,
            console_platform,
            controllers: Arc::new(Mutex::new(HashMap::new())),
            memory_usage_mb: Arc::new(Mutex::new(0)),
            max_memory_mb,
            gpu_usage: Arc::new(Mutex::new(0.0)),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            performance_constraint: Arc::new(Mutex::new(PerformanceConstraint::None)),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    /// Set memory usage
    pub fn set_memory_usage(&self, usage_mb: usize) -> Result<(), MockError> {
        let constraint = *self.performance_constraint.lock().unwrap();

        if let PerformanceConstraint::MaxMemoryUsage(max_mb) = constraint {
            if usage_mb > max_mb {
                return Err(MockError::MemoryLimitExceeded(usage_mb, max_mb));
            }
        }

        if usage_mb > self.max_memory_mb {
            return Err(MockError::MemoryLimitExceeded(usage_mb, self.max_memory_mb));
        }

        *self.memory_usage_mb.lock().unwrap() = usage_mb;
        Ok(())
    }

    /// Set GPU usage
    pub fn set_gpu_usage(&self, usage: f32) -> Result<(), MockError> {
        let constraint = *self.performance_constraint.lock().unwrap();

        if let PerformanceConstraint::MaxGpuUsage(max_usage) = constraint {
            if usage > max_usage {
                return Err(MockError::PerformanceConstraintViolation(format!(
                    "GPU usage {} exceeds limit {}",
                    usage, max_usage
                )));
            }
        }

        *self.gpu_usage.lock().unwrap() = usage.clamp(0.0, 1.0);
        Ok(())
    }

    /// Set CPU usage
    pub fn set_cpu_usage(&self, usage: f32) -> Result<(), MockError> {
        let constraint = *self.performance_constraint.lock().unwrap();

        if let PerformanceConstraint::MaxCpuUsage(max_usage) = constraint {
            if usage > max_usage {
                return Err(MockError::PerformanceConstraintViolation(format!(
                    "CPU usage {} exceeds limit {}",
                    usage, max_usage
                )));
            }
        }

        *self.cpu_usage.lock().unwrap() = usage.clamp(0.0, 1.0);
        Ok(())
    }

    /// Add a controller
    pub fn add_controller(&self, id: u32) {
        let mut controllers = self.controllers.lock().unwrap();
        controllers.insert(
            id,
            ControllerState {
                id,
                connected: true,
                ..Default::default()
            },
        );
    }

    /// Remove a controller
    pub fn remove_controller(&self, id: u32) {
        let mut controllers = self.controllers.lock().unwrap();
        controllers.remove(&id);
    }

    /// Simulate button press
    pub fn press_button(&self, controller_id: u32, button: MockButton) {
        let mut controllers = self.controllers.lock().unwrap();
        if let Some(controller) = controllers.get_mut(&controller_id) {
            match button {
                MockButton::A => controller.buttons.a = true,
                MockButton::B => controller.buttons.b = true,
                MockButton::X => controller.buttons.x = true,
                MockButton::Y => controller.buttons.y = true,
                MockButton::LeftBumper => controller.buttons.left_bumper = true,
                MockButton::RightBumper => controller.buttons.right_bumper = true,
                MockButton::Menu => controller.buttons.menu = true,
                MockButton::View => controller.buttons.view = true,
                MockButton::DPadUp => controller.buttons.dpad_up = true,
                MockButton::DPadDown => controller.buttons.dpad_down = true,
                MockButton::DPadLeft => controller.buttons.dpad_left = true,
                MockButton::DPadRight => controller.buttons.dpad_right = true,
            }
        }
    }

    /// Simulate button release
    pub fn release_button(&self, controller_id: u32, button: MockButton) {
        let mut controllers = self.controllers.lock().unwrap();
        if let Some(controller) = controllers.get_mut(&controller_id) {
            match button {
                MockButton::A => controller.buttons.a = false,
                MockButton::B => controller.buttons.b = false,
                MockButton::X => controller.buttons.x = false,
                MockButton::Y => controller.buttons.y = false,
                MockButton::LeftBumper => controller.buttons.left_bumper = false,
                MockButton::RightBumper => controller.buttons.right_bumper = false,
                MockButton::Menu => controller.buttons.menu = false,
                MockButton::View => controller.buttons.view = false,
                MockButton::DPadUp => controller.buttons.dpad_up = false,
                MockButton::DPadDown => controller.buttons.dpad_down = false,
                MockButton::DPadLeft => controller.buttons.dpad_left = false,
                MockButton::DPadRight => controller.buttons.dpad_right = false,
            }
        }
    }

    /// Set stick position
    pub fn set_stick(&self, controller_id: u32, stick: MockStick, x: f32, y: f32) {
        let mut controllers = self.controllers.lock().unwrap();
        if let Some(controller) = controllers.get_mut(&controller_id) {
            match stick {
                MockStick::Left => controller.left_stick = (x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0)),
                MockStick::Right => {
                    controller.right_stick = (x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0))
                }
            }
        }
    }

    /// Set trigger value
    pub fn set_trigger(&self, controller_id: u32, trigger: MockTrigger, value: f32) {
        let mut controllers = self.controllers.lock().unwrap();
        if let Some(controller) = controllers.get_mut(&controller_id) {
            match trigger {
                MockTrigger::Left => controller.left_trigger = value.clamp(0.0, 1.0),
                MockTrigger::Right => controller.right_trigger = value.clamp(0.0, 1.0),
            }
        }
    }

    /// Get platform type
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Get console platform type
    pub fn console_platform(&self) -> ConsolePlatform {
        self.console_platform
    }

    /// Initialize the platform
    pub fn initialize(&self) -> Result<(), MockError> {
        *self.initialized.lock().unwrap() = true;
        Ok(())
    }

    /// Get controller state
    pub fn get_controller(&self, id: u32) -> Option<ControllerState> {
        self.controllers.lock().unwrap().get(&id).cloned()
    }

    /// Set controller state
    pub fn set_controller(&self, id: u32, state: ControllerState) {
        self.controllers.lock().unwrap().insert(id, state);
    }
}

/// Mock button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockButton {
    A,
    B,
    X,
    Y,
    LeftBumper,
    RightBumper,
    Menu,
    View,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// Mock stick types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockStick {
    Left,
    Right,
}

/// Mock trigger types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockTrigger {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_mock_platform() {
        let mock = BaseMockPlatform::new(ConsolePlatform::PlayStation5);
        assert_eq!(mock.platform(), Platform::PlayStation5);
        assert_eq!(mock.console_platform(), ConsolePlatform::PlayStation5);
        assert!(!mock.is_initialized());
    }

    #[test]
    fn test_memory_constraint() {
        let mock = BaseMockPlatform::new(ConsolePlatform::NintendoSwitch);
        mock.initialize().unwrap();

        // Should work within limits
        assert!(mock.set_memory_usage(2048).is_ok());

        // Should fail beyond limits
        assert!(mock.set_memory_usage(8 * 1024).is_err());
    }

    #[test]
    fn test_controller_simulation() {
        let mock = BaseMockPlatform::new(ConsolePlatform::XboxSeries);
        mock.initialize().unwrap();

        mock.add_controller(0);
        mock.press_button(0, MockButton::A);

        let state = mock.get_controller(0).unwrap();
        assert!(state.buttons.a);

        mock.release_button(0, MockButton::A);
        let state = mock.get_controller(0).unwrap();
        assert!(!state.buttons.a);

        mock.set_stick(0, MockStick::Left, 0.5, 0.3);
        let state = mock.get_controller(0).unwrap();
        assert_eq!(state.left_stick, (0.5, 0.3));

        mock.set_trigger(0, MockTrigger::Left, 0.7);
        let state = mock.get_controller(0).unwrap();
        assert_eq!(state.left_trigger, 0.7);
    }
}
