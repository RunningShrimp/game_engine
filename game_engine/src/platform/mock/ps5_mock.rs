//! # PlayStation 5 Mock Platform
//!
//! Mock implementation of PlayStation 5 platform for testing.

use crate::platform::console::ConsolePlatform;
use crate::platform::detection_extended::{Feature, Platform};
use crate::platform::mock::base_mock::{
    BaseMockPlatform, MockError, MockPlatform, PerformanceConstraint,
};

/// PlayStation 5 mock platform
pub struct PS5MockPlatform {
    base: BaseMockPlatform,
    frame_time: f32,
    ray_tracing_enabled: bool,
}

impl PS5MockPlatform {
    /// Create a new PS5 mock
    pub fn new() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::PlayStation5),
            frame_time: 16.7, // 60 FPS target
            ray_tracing_enabled: false,
        }
    }

    /// Enable ray tracing
    pub fn enable_ray_tracing(&mut self, enabled: bool) {
        self.ray_tracing_enabled = enabled;
    }

    /// Check if ray tracing is enabled
    pub fn is_ray_tracing_enabled(&self) -> bool {
        self.ray_tracing_enabled
    }
}

impl Default for PS5MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform for PS5MockPlatform {
    fn platform(&self) -> Platform {
        self.base.platform()
    }

    fn console_platform(&self) -> ConsolePlatform {
        self.base.console_platform()
    }

    fn initialize(&mut self) -> Result<(), MockError> {
        self.base.initialize()?;
        // Add DualSense controller
        self.base.add_controller(0);
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<(), MockError> {
        if let PerformanceConstraint::MaxFrameTime(max_time) =
            *self.base.performance_constraint.lock().unwrap()
        {
            if delta_time > max_time {
                return Err(MockError::PerformanceConstraintViolation(format!(
                    "Frame time {delta_time}ms exceeds {max_time}ms"
                )));
            }
        }

        // Ray tracing has performance impact
        if self.ray_tracing_enabled && delta_time < 30.0 {
            return Err(MockError::PerformanceConstraintViolation(
                "Ray tracing enabled but frame time is too low".into(),
            ));
        }

        self.frame_time = delta_time;
        Ok(())
    }

    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::RayTracing => true,
            Feature::HDR => true,
            Feature::Vibration => true,
            Feature::Touchpad => true,
            Feature::MotionControls => true, // DualSense has gyroscope
            Feature::OnlineMultiplayer => true,
            Feature::CloudSave => true,
            Feature::Leaderboards => true,
            Feature::Achievements => true,
            Feature::SpatialAudio => true,
            Feature::VoiceChat => true,
            Feature::CrossPlatformPlay => true,
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
    fn test_ps5_mock() {
        let mut mock = PS5MockPlatform::new();
        mock.initialize().unwrap();

        assert_eq!(mock.platform(), Platform::PlayStation5);
        assert!(mock.supports_feature(Feature::RayTracing));
        assert!(mock.supports_feature(Feature::HDR));
        assert!(mock.supports_feature(Feature::Touchpad));

        // Enable ray tracing
        mock.enable_ray_tracing(true);
        assert!(mock.is_ray_tracing_enabled());
    }

    #[test]
    fn test_ps5_ray_tracing_performance() {
        let mut mock = PS5MockPlatform::new();
        mock.initialize().unwrap();

        // Without ray tracing
        mock.update(16.7).unwrap();

        // With ray tracing - should have higher frame time
        mock.enable_ray_tracing(true);
        assert!(mock.update(33.3).is_ok());
        assert!(mock.update(16.7).is_err()); // Too fast for ray tracing
    }
}
