//! # Xbox Mock Platform
//!
//! Mock implementation of Xbox Series X/S and Xbox One platforms for testing.

use crate::platform::console::{ConsolePlatform, ControllerState};
use crate::platform::detection_extended::{Feature, Platform};
use crate::platform::mock::base_mock::{
    BaseMockPlatform, MockError, MockPlatform, PerformanceConstraint,
};

/// Xbox mock platform (supports both Series and One)
pub struct XboxMockPlatform {
    base: BaseMockPlatform,
    is_series: bool,
    is_series_x: bool,
    frame_time: f32,
}

impl XboxMockPlatform {
    /// Create a new Xbox Series X mock
    pub fn new_series_x() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::XboxSeries),
            is_series: true,
            is_series_x: true,
            frame_time: 16.7, // 60 FPS target
        }
    }

    /// Create a new Xbox Series S mock
    pub fn new_series_s() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::XboxSeries),
            is_series: true,
            is_series_x: false,
            frame_time: 16.7,
        }
    }

    /// Create a new Xbox One mock
    pub fn new_xbox_one() -> Self {
        Self {
            base: BaseMockPlatform::new(ConsolePlatform::XboxOne),
            is_series: false,
            is_series_x: false,
            frame_time: 16.7,
        }
    }

    /// Check if this is Xbox Series
    pub fn is_series(&self) -> bool {
        self.is_series
    }

    /// Check if this is Xbox Series X (vs Series S)
    pub fn is_series_x(&self) -> bool {
        self.is_series_x
    }

    /// Check if this is Xbox Series S
    pub fn is_series_s(&self) -> bool {
        self.is_series && !self.is_series_x
    }
}

impl Default for XboxMockPlatform {
    fn default() -> Self {
        Self::new_series_x()
    }
}

impl MockPlatform for XboxMockPlatform {
    fn platform(&self) -> Platform {
        self.base.platform()
    }

    fn console_platform(&self) -> ConsolePlatform {
        self.base.console_platform()
    }

    fn initialize(&mut self) -> Result<(), MockError> {
        self.base.initialize()?;
        // Add Xbox controller
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

        // Series S has lower resolution target, may have better performance
        if self.is_series_s() && delta_time < 20.0 {
            // Series S often runs at lower resolution
        }

        self.frame_time = delta_time;
        Ok(())
    }

    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::RayTracing => self.is_series && self.is_series_x, // Only Series X
            Feature::HDR => true,
            Feature::Vibration => true,
            Feature::OnlineMultiplayer => true,
            Feature::LanMultiplayer => true,
            Feature::CloudSave => true,
            Feature::Leaderboards => true,
            Feature::Achievements => true,
            Feature::SpatialAudio => true,
            Feature::VoiceChat => true,
            Feature::CrossPlatformPlay => true,
            Feature::RemotePlay => true,
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

    fn get_controller(&self, id: u32) -> Option<ControllerState> {
        self.base.get_controller(id)
    }

    fn set_controller(&mut self, id: u32, state: ControllerState) {
        self.base.set_controller(id, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xbox_series_x() {
        let mut mock = XboxMockPlatform::new_series_x();
        mock.initialize().unwrap();

        assert_eq!(mock.platform(), Platform::XboxSeries);
        assert!(mock.is_series());
        assert!(mock.is_series_x());
        assert!(mock.supports_feature(Feature::RayTracing));
        assert!(mock.supports_feature(Feature::HDR));
    }

    #[test]
    fn test_xbox_series_s() {
        let mut mock = XboxMockPlatform::new_series_s();
        mock.initialize().unwrap();

        assert!(mock.is_series());
        assert!(mock.is_series_s());
        assert!(!mock.is_series_x());
        assert!(!mock.supports_feature(Feature::RayTracing)); // Series S doesn't have full RT
        assert!(mock.supports_feature(Feature::HDR));
    }

    #[test]
    fn test_xbox_one() {
        let mut mock = XboxMockPlatform::new_xbox_one();
        mock.initialize().unwrap();

        assert_eq!(mock.platform(), Platform::XboxOne);
        assert!(!mock.is_series());
        assert!(!mock.supports_feature(Feature::RayTracing));
        assert!(mock.supports_feature(Feature::HDR));
        assert!(mock.supports_feature(Feature::LanMultiplayer));
    }
}
