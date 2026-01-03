//! # PlayStation 4 Mock Platform
//!
//! Mock implementation of PlayStation 4 platform for testing.

use crate::platform::console::ConsolePlatform;
use crate::platform::detection_extended::{Feature, Platform};
use crate::platform::mock::base_mock::{
    BaseMockPlatform, MockError, MockPlatform, PerformanceConstraint,
};

/// PlayStation 4 mock platform
pub struct PS4MockPlatform {
    base: BaseMockPlatform,
    is_pro: bool,
    frame_time: f32,
}

impl PS4MockPlatform {
    /// Create a new PS4 mock
    pub fn new() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::PlayStation4),
            is_pro: false,
            frame_time: 16.7, // 60 FPS target
        }
    }

    /// Create a new PS4 Pro mock
    pub fn new_pro() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::PlayStation4),
            is_pro: true,
            frame_time: 16.7,
        }
    }

    /// Check if this is PS4 Pro
    pub fn is_pro(&self) -> bool {
        self.is_pro
    }
}

impl Default for PS4MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform for PS4MockPlatform {
    fn platform(&self) -> Platform {
        self.base.platform()
    }

    fn console_platform(&self) -> ConsolePlatform {
        self.base.console_platform()
    }

    fn initialize(&mut self) -> Result<(), MockError> {
        self.base.initialize()?;
        // Add DualShock 4 controller
        self.base.add_controller(0);
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<(), MockError> {
        if let PerformanceConstraint::MaxFrameTime(max_time) =
            *self.base.performance_constraint.lock().unwrap()
        {
            if delta_time > max_time {
                return Err(MockError::PerformanceConstraintViolation(format!(
                    "Frame time {}ms exceeds {}ms",
                    delta_time, max_time
                )));
            }
        }

        self.frame_time = delta_time;
        Ok(())
    }

    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::RayTracing => false,
            Feature::HDR => self.is_pro,
            Feature::Vibration => true,
            Feature::Touchpad => true,
            Feature::MotionControls => true, // DualShock 4 has gyroscope
            Feature::OnlineMultiplayer => true,
            Feature::CloudSave => true,
            Feature::Leaderboards => true,
            Feature::Achievements => true,
            Feature::SpatialAudio => true,
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
    fn test_ps4_mock() {
        let mut mock = PS4MockPlatform::new();
        mock.initialize().unwrap();

        assert_eq!(mock.platform(), Platform::PlayStation4);
        assert!(!mock.is_pro());
        assert!(!mock.supports_feature(Feature::RayTracing));
        assert!(!mock.supports_feature(Feature::HDR));
        assert!(mock.supports_feature(Feature::Touchpad));
    }

    #[test]
    fn test_ps4_pro_mock() {
        let mut mock = PS4MockPlatform::new_pro();
        mock.initialize().unwrap();

        assert!(mock.is_pro());
        assert!(mock.supports_feature(Feature::HDR));
        assert!(!mock.supports_feature(Feature::RayTracing));
    }
}
