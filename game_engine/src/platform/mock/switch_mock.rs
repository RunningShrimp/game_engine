//! # Nintendo Switch Mock Platform
//!
//! Mock implementation of Nintendo Switch platform for testing.

use crate::platform::console::ConsolePlatform;
use crate::platform::detection_extended::{Feature, Platform};
use crate::platform::mock::base_mock::{
    BaseMockPlatform, MockError, MockPlatform, PerformanceConstraint,
};
use std::time::Duration;

/// Nintendo Switch mock platform
pub struct SwitchMockPlatform {
    base: BaseMockPlatform,
    docked: bool,
    frame_time: f32,
}

impl SwitchMockPlatform {
    /// Create a new Nintendo Switch mock
    pub fn new() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::NintendoSwitch),
            docked: false,    // Handheld mode by default
            frame_time: 33.3, // 30 FPS target
        }
    }

    /// Set docked mode
    pub fn set_docked(&mut self, docked: bool) {
        self.docked = docked;
        // Adjust target FPS based on mode
        self.frame_time = if docked { 16.7 } else { 33.3 }; // 60 FPS docked, 30 FPS handheld
    }

    /// Check if in docked mode
    pub fn is_docked(&self) -> bool {
        self.docked
    }
}

impl Default for SwitchMockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform for SwitchMockPlatform {
    fn platform(&self) -> Platform {
        self.base.platform()
    }

    fn console_platform(&self) -> ConsolePlatform {
        self.base.console_platform()
    }

    fn initialize(&mut self) -> Result<(), MockError> {
        self.base.initialize()?;
        // Add player 1 controller
        self.base.add_controller(0);
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<(), MockError> {
        // Simulate performance constraints
        if let PerformanceConstraint::MaxFrameTime(max_time) =
            *self.base.performance_constraint.lock().unwrap()
        {
            if delta_time > max_time {
                return Err(MockError::PerformanceConstraintViolation(format!(
                    "Frame time {delta_time}ms exceeds {max_time}ms"
                )));
            }
        }

        self.frame_time = delta_time;
        Ok(())
    }

    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::RayTracing => false,
            Feature::HDR => false,
            Feature::MotionControls => true,
            Feature::Vibration => true,
            Feature::OnlineMultiplayer => true,
            Feature::CloudSave => true,
            Feature::Leaderboards => true,
            Feature::Achievements => true,
            _ => false,
        }
    }

    fn memory_usage(&self) -> usize {
        *self.base.memory_usage_mb.lock().unwrap()
    }

    fn gpu_usage(&self) -> f32 {
        *self.base.gpu_usage.lock().unwrap()
    }

    fn cpu_usage(&self) -> f32 {
        *self.base.cpu_usage.lock().unwrap()
    }

    fn set_performance_constraint(&mut self, constraint: PerformanceConstraint) {
        *self.base.performance_constraint.lock().unwrap() = constraint;
    }

    fn get_controller(&self, id: u32) -> Option<crate::platform::console::ControllerState> {
        self.base.get_controller(id)
    }

    fn set_controller(&mut self, id: u32, state: crate::platform::console::ControllerState) {
        self.base.set_controller(id, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_mock() {
        let mut mock = SwitchMockPlatform::new();
        mock.initialize().unwrap();

        assert_eq!(mock.platform(), Platform::NintendoSwitch);
        assert!(!mock.is_docked());

        mock.set_docked(true);
        assert!(mock.is_docked());

        assert!(mock.supports_feature(Feature::MotionControls));
        assert!(!mock.supports_feature(Feature::RayTracing));
        assert!(!mock.supports_feature(Feature::HDR));
    }

    #[test]
    fn test_switch_performance_modes() {
        let mut mock = SwitchMockPlatform::new();

        // Handheld mode - 30 FPS
        assert!(!mock.is_docked());
        mock.update(33.3).unwrap();

        // Docked mode - 60 FPS
        mock.set_docked(true);
        mock.update(16.7).unwrap();
    }
}
