//! # Extended Controller Features
//!
//! Advanced controller features like vibration, LED, motion controls, and touchpad.
//!
//! ## Features
//!
//! - **Vibration**: Nintendo Switch HD rumble, PS5 haptic feedback, Xbox standard vibration
//! - **LED**: PS5 light bar, PS4 light bar, custom color effects
//! - **Touchpad**: PS4/PS5 touchpad input tracking
//! - **Motion**: Gyroscope and accelerometer support for Switch and PlayStation
//! - **Adaptive Triggers**: PS5 DualSense adaptive trigger resistance
//!
//! ## Platform Support
//!
//! | Feature | Switch | PS5 | PS4 | Xbox |
//! |---------|--------|-----|-----|------|
//! | HD Rumble | ✅ | ❌ | ❌ | ❌ |
//! | Haptic Feedback | ❌ | ✅ | ❌ | ❌ |
//! | Vibration | ❌ | ✅ | ✅ | ✅ |
//! | LED | ✅ | ✅ | ✅ | ❌ |
//! | Touchpad | ❌ | ✅ | ✅ | ❌ |
//! | Motion | ✅ | ✅ | ✅ | ❌ |
//! | Adaptive Triggers | ❌ | ✅ | ❌ | ❌ |
//!
//! ## Examples
//!
//! ```rust,ignore
//! use game_engine::platform::console::controller_extended::*;
//!
//! let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
//!
//! // Set vibration
//! let vibration = VibrationIntensity::new(0.7, 0.5);
//! manager.set_vibration(0, vibration)?;
//!
//! // Set LED color
//! let color = LedColor::red();
//! manager.set_led_color(0, color)?;
//!
//! // Get motion data
//! let motion = manager.get_motion_data(0)?;
//! println!("Gyro: {:?}", motion.gyro);
//! ```

use super::ConsolePlatform;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Vibration intensity for controller motors
///
/// - `weak_motor`: High-frequency, low-amplitude motor (typically right)
/// - `strong_motor`: Low-frequency, high-amplitude motor (typically left)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VibrationIntensity {
    pub weak_motor: f32,
    pub strong_motor: f32,
}

impl VibrationIntensity {
    /// Create new vibration intensity with clamped values [0.0, 1.0]
    pub fn new(weak: f32, strong: f32) -> Self {
        Self {
            weak_motor: weak.clamp(0.0, 1.0),
            strong_motor: strong.clamp(0.0, 1.0),
        }
    }

    /// No vibration
    pub fn off() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Maximum vibration
    pub fn max() -> Self {
        Self::new(1.0, 1.0)
    }

    /// Gentle vibration (30% intensity)
    pub fn gentle() -> Self {
        Self::new(0.3, 0.3)
    }

    /// Medium vibration (60% intensity)
    pub fn medium() -> Self {
        Self::new(0.6, 0.6)
    }

    /// Strong vibration (90% intensity)
    pub fn strong() -> Self {
        Self::new(0.9, 0.9)
    }

    /// Check if vibration is off
    pub fn is_off(&self) -> bool {
        self.weak_motor == 0.0 && self.strong_motor == 0.0
    }

    /// Linear interpolation between two vibration intensities
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self::new(
            self.weak_motor + (other.weak_motor - self.weak_motor) * t,
            self.strong_motor + (other.strong_motor - self.strong_motor) * t,
        )
    }
}

/// LED color for controller light bars and indicators
///
/// RGB values in range [0, 255]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl LedColor {
    /// Create new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Red color
    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    /// Green color
    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    /// Blue color
    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

    /// White color
    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    /// LED off
    pub fn off() -> Self {
        Self::new(0, 0, 0)
    }

    /// Yellow color
    pub fn yellow() -> Self {
        Self::new(255, 255, 0)
    }

    /// Cyan color
    pub fn cyan() -> Self {
        Self::new(0, 255, 255)
    }

    /// Magenta color
    pub fn magenta() -> Self {
        Self::new(255, 0, 255)
    }

    /// Create from HSV color space (hue in [0, 360], saturation and value in [0, 1])
    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let h = hue / 60.0;
        let s = saturation.clamp(0.0, 1.0);
        let v = value.clamp(0.0, 1.0);

        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::new(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// Linear interpolation between two colors
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self::new(
            (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        )
    }

    /// Convert to RGB tuple
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// LED effect pattern
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LedEffect {
    /// Static color
    Static,
    /// Breathing effect (fade in and out)
    Breathing { speed_ms: u32 },
    /// Blinking effect
    Blinking {
        on_duration_ms: u32,
        off_duration_ms: u32,
    },
    /// Rainbow cycle effect
    Rainbow { speed_ms: u32 },
}

impl LedEffect {
    /// Static effect
    pub fn static_effect() -> Self {
        LedEffect::Static
    }

    /// Breathing effect with speed in milliseconds
    pub fn breathing(speed_ms: u32) -> Self {
        LedEffect::Breathing { speed_ms }
    }

    /// Blinking effect with on/off durations
    pub fn blinking(on_duration_ms: u32, off_duration_ms: u32) -> Self {
        LedEffect::Blinking {
            on_duration_ms,
            off_duration_ms,
        }
    }

    /// Rainbow effect with speed
    pub fn rainbow(speed_ms: u32) -> Self {
        LedEffect::Rainbow { speed_ms }
    }
}

/// Touch point for DualShock/DualSense touchpad
///
/// Coordinates are normalized in range [0, 1]
/// - (0, 0) = top-left
/// - (1, 1) = bottom-right
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    /// Whether the touchpad is being touched
    pub touching: bool,
    /// X coordinate [0.0, 1.0]
    pub x: f32,
    /// Y coordinate [0.0, 1.0]
    pub y: f32,
}

impl TouchPoint {
    /// Create new touch point
    pub fn new(touching: bool, x: f32, y: f32) -> Self {
        Self {
            touching,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    /// Check if touch point is active
    pub fn is_active(&self) -> bool {
        self.touching
    }

    /// Get distance to another touch point
    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Touchpad gesture
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TouchGesture {
    /// Single tap
    Tap,
    /// Double tap
    DoubleTap,
    /// Swipe (direction_x, direction_y) normalized [-1, 1]
    Swipe { direction_x: f32, direction_y: f32 },
    /// Pinch (positive = zoom in, negative = zoom out)
    Pinch { scale: f32 },
    /// Two-finger pan
    Pan { delta_x: f32, delta_y: f32 },
}

/// Motion data from gyroscope and accelerometer
///
/// All values are in the controller's local coordinate system
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionData {
    /// Angular velocity (rad/s) around x, y, z axes
    pub gyro: (f32, f32, f32),
    /// Acceleration (m/s²) along x, y, z axes
    pub accel: (f32, f32, f32),
}

impl MotionData {
    /// Create new motion data
    pub fn new(gyro: (f32, f32, f32), accel: (f32, f32, f32)) -> Self {
        Self { gyro, accel }
    }

    /// Get magnitude of angular velocity
    pub fn gyro_magnitude(&self) -> f32 {
        (self.gyro.0.powi(2) + self.gyro.1.powi(2) + self.gyro.2.powi(2)).sqrt()
    }

    /// Get magnitude of acceleration
    pub fn accel_magnitude(&self) -> f32 {
        (self.accel.0.powi(2) + self.accel.1.powi(2) + self.accel.2.powi(2)).sqrt()
    }

    /// Check if controller is being shaken
    pub fn is_shaken(&self, threshold: f32) -> bool {
        self.accel_magnitude() > threshold
    }
}

/// Orientation from motion sensors (pitch, roll, yaw in radians)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Orientation {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

impl Orientation {
    /// Create new orientation
    pub fn new(pitch: f32, roll: f32, yaw: f32) -> Self {
        Self { pitch, roll, yaw }
    }

    /// Convert to degrees
    pub fn to_degrees(&self) -> (f32, f32, f32) {
        (
            self.pitch.to_degrees(),
            self.roll.to_degrees(),
            self.yaw.to_degrees(),
        )
    }
}

/// Motion calibration data
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionCalibration {
    /// Gyroscope offset (rad/s)
    pub gyro_offset: (f32, f32, f32),
    /// Accelerometer offset (m/s²)
    pub accel_offset: (f32, f32, f32),
}

impl MotionCalibration {
    /// No calibration
    pub fn none() -> Self {
        Self {
            gyro_offset: (0.0, 0.0, 0.0),
            accel_offset: (0.0, 0.0, 0.0),
        }
    }

    /// Apply calibration to motion data
    pub fn apply(&self, data: &MotionData) -> MotionData {
        MotionData {
            gyro: (
                data.gyro.0 - self.gyro_offset.0,
                data.gyro.1 - self.gyro_offset.1,
                data.gyro.2 - self.gyro_offset.2,
            ),
            accel: (
                data.accel.0 - self.accel_offset.0,
                data.accel.1 - self.accel_offset.1,
                data.accel.2 - self.accel_offset.2,
            ),
        }
    }
}

/// Haptic feedback for DualSense controller
///
/// DualSense provides precise haptic feedback with multiple actuators
/// - Values in range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HapticFeedback {
    pub left: f32,
    pub right: f32,
}

impl HapticFeedback {
    /// Create new haptic feedback with clamped values
    pub fn new(left: f32, right: f32) -> Self {
        Self {
            left: left.clamp(0.0, 1.0),
            right: right.clamp(0.0, 1.0),
        }
    }

    /// No feedback
    pub fn off() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Uniform intensity on both sides
    pub fn uniform(intensity: f32) -> Self {
        Self::new(intensity, intensity)
    }

    /// Light feedback
    pub fn light() -> Self {
        Self::uniform(0.3)
    }

    /// Medium feedback
    pub fn medium() -> Self {
        Self::uniform(0.6)
    }

    /// Strong feedback
    pub fn strong() -> Self {
        Self::uniform(0.9)
    }

    /// Check if feedback is off
    pub fn is_off(&self) -> bool {
        self.left == 0.0 && self.right == 0.0
    }
}

/// Trigger effect types for DualSense adaptive triggers
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TriggerEffect {
    /// No resistance
    None,
    /// Constant resistance
    Constant { strength: f32 },
    /// Resistance increases as trigger is pressed
    Linear {
        start_strength: f32,
        end_strength: f32,
    },
    /// Multiple zones of resistance
    Zones {
        zones: [(f32, f32); 4], // (position_start, strength)
    },
    /// Vibration effect
    Vibration { strength: f32, frequency: f32 },
}

impl TriggerEffect {
    /// No resistance
    pub fn none() -> Self {
        TriggerEffect::None
    }

    /// Constant resistance
    pub fn constant(strength: f32) -> Self {
        TriggerEffect::Constant {
            strength: strength.clamp(0.0, 1.0),
        }
    }

    /// Linear resistance
    pub fn linear(start: f32, end: f32) -> Self {
        TriggerEffect::Linear {
            start_strength: start.clamp(0.0, 1.0),
            end_strength: end.clamp(0.0, 1.0),
        }
    }

    /// Weapon trigger effect (soft then hard stop)
    pub fn weapon() -> Self {
        TriggerEffect::Zones {
            zones: [(0.0, 0.2), (0.5, 0.3), (0.7, 0.8), (0.9, 1.0)],
        }
    }

    /// Bow draw effect
    pub fn bow() -> Self {
        TriggerEffect::Linear {
            start_strength: 0.1,
            end_strength: 0.9,
        }
    }

    /// Machine gun vibration
    pub fn machine_gun() -> Self {
        TriggerEffect::Vibration {
            strength: 0.7,
            frequency: 15.0,
        }
    }
}

/// HD Rumble pattern for Nintendo Switch
///
/// Switch HD Rumble uses high-frequency vibration for detailed effects
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HDRumblePattern {
    /// Low-frequency band amplitude
    pub low_band: (f32, f32),
    /// High-frequency band amplitude
    pub high_band: (f32, f32),
}

impl HDRumblePattern {
    /// Create new HD rumble pattern
    pub fn new(low_band: (f32, f32), high_band: (f32, f32)) -> Self {
        Self {
            low_band: (low_band.0.clamp(0.0, 1.0), low_band.1.clamp(0.0, 1.0)),
            high_band: (high_band.0.clamp(0.0, 1.0), high_band.1.clamp(0.0, 1.0)),
        }
    }

    /// Gentle pulse
    pub fn gentle_pulse() -> Self {
        Self::new((0.2, 0.1), (0.1, 0.2))
    }

    /// Sharp impact
    pub fn sharp_impact() -> Self {
        Self::new((0.8, 0.6), (0.6, 0.8))
    }

    /// Continuous rumble
    pub fn continuous(intensity: f32) -> Self {
        Self::new((intensity, intensity), (intensity, intensity))
    }
}

/// Extended controller manager
///
/// Provides advanced controller features for different console platforms
pub struct ExtendedControllerManager {
    platform: ConsolePlatform,
    calibration_data: std::collections::HashMap<u32, MotionCalibration>,
}

impl ExtendedControllerManager {
    /// Create new extended controller manager
    pub fn new(platform: ConsolePlatform) -> Self {
        Self {
            platform,
            calibration_data: std::collections::HashMap::new(),
        }
    }

    /// Set motion calibration data for a controller
    pub fn set_motion_calibration(&mut self, controller_id: u32, calibration: MotionCalibration) {
        self.calibration_data.insert(controller_id, calibration);
    }

    /// Get motion calibration data for a controller
    pub fn get_motion_calibration(&self, controller_id: u32) -> Option<MotionCalibration> {
        self.calibration_data.get(&controller_id).copied()
    }

    /// Set vibration for controller
    ///
    /// Supports:
    /// - Nintendo Switch: HD rumble with frequency bands
    /// - PlayStation 5/4: Standard vibration
    /// - Xbox: Standard vibration
    pub fn set_vibration(
        &self,
        controller_id: u32,
        vibration: VibrationIntensity,
    ) -> Result<(), ControllerError> {
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                // Nintendo Switch HD Rumble implementation
                // HD Rumble uses frequency bands for detailed haptic effects
                tracing::info!(
                    "Switch HD Rumble for controller {}: weak={:.2}, strong={:.2}",
                    controller_id,
                    vibration.weak_motor,
                    vibration.strong_motor
                );

                // In a real implementation, this would call the Switch SDK
                // nn::hid::SetVibrationMotorFct(controller_id, ...)

                Ok(())
            }
            ConsolePlatform::PlayStation5 => {
                // DualSense vibration implementation
                tracing::info!(
                    "DualSense vibration for controller {}: weak={:.2}, strong={:.2}",
                    controller_id,
                    vibration.weak_motor,
                    vibration.strong_motor
                );

                // Real implementation would use DualSense SDK
                // DS5 vibration supports both weak and strong motors

                Ok(())
            }
            ConsolePlatform::PlayStation4 => {
                // DualShock 4 vibration implementation
                tracing::info!(
                    "DualShock 4 vibration for controller {}: weak={:.2}, strong={:.2}",
                    controller_id,
                    vibration.weak_motor,
                    vibration.strong_motor
                );

                // Real implementation would use PS4 SDK
                // scePadSetVibration

                Ok(())
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                // Xbox controller vibration implementation
                tracing::info!(
                    "Xbox vibration for controller {}: weak={:.2}, strong={:.2}",
                    controller_id,
                    vibration.weak_motor,
                    vibration.strong_motor
                );

                // Real implementation would use XInput
                // XInputSetState

                Ok(())
            }
        }
    }

    /// Set HD rumble pattern for Switch (enhanced vibration)
    pub fn set_hd_rumble(
        &self,
        controller_id: u32,
        pattern: HDRumblePattern,
    ) -> Result<(), ControllerError> {
        if self.platform != ConsolePlatform::NintendoSwitch {
            return Err(ControllerError::NotSupported);
        }

        tracing::info!(
            "Switch HD Rumble pattern for controller {}: low_band=({:.2},{:.2}), high_band=({:.2},{:.2})",
            controller_id,
            pattern.low_band.0,
            pattern.low_band.1,
            pattern.high_band.0,
            pattern.high_band.1
        );

        // Real implementation: nn::hid::SetVibrationMotorFct
        Ok(())
    }

    /// Set LED color for controller
    ///
    /// Supports:
    /// - Nintendo Switch: Pro Controller player LEDs
    /// - PlayStation 5/4: Light bar color
    /// - Xbox: Not supported
    pub fn set_led_color(
        &self,
        controller_id: u32,
        color: LedColor,
    ) -> Result<(), ControllerError> {
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                // Switch Pro Controller LED
                tracing::info!(
                    "Switch Pro Controller LED for controller {}: RGB=({},{},{})",
                    controller_id,
                    color.r,
                    color.g,
                    color.b
                );

                // Real implementation: nn::hid::SetPlayerLedPattern
                Ok(())
            }
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                // DualShock/DualSense light bar
                tracing::info!(
                    "PlayStation light bar for controller {}: RGB=({},{},{})",
                    controller_id,
                    color.r,
                    color.g,
                    color.b
                );

                // Real implementation: scePadSetLightBar
                Ok(())
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                // Xbox controllers don't have programmable LEDs
                // (Xbox Series X controller has LED but it's not programmable)
                Err(ControllerError::NotSupported)
            }
        }
    }

    /// Set LED effect pattern
    pub fn set_led_effect(
        &self,
        controller_id: u32,
        color: LedColor,
        effect: LedEffect,
    ) -> Result<(), ControllerError> {
        tracing::info!(
            "Setting LED effect for controller {}: color=({},{},{}), effect={:?}",
            controller_id,
            color.r,
            color.g,
            color.b,
            effect
        );

        // Real implementation would set up continuous LED effects
        Ok(())
    }

    /// Get touch input from PlayStation controllers
    ///
    /// Returns up to 2 touch points from the touchpad
    pub fn get_touch_input(&self, controller_id: u32) -> Result<[TouchPoint; 2], ControllerError> {
        match self.platform {
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                // Real implementation: scePadRead
                tracing::trace!("Reading touchpad input for controller {}", controller_id);

                // Mock data - real implementation would read from hardware
                Ok([TouchPoint::default(), TouchPoint::default()])
            }
            _ => Err(ControllerError::NotSupported),
        }
    }

    /// Detect touch gesture from touchpad input
    pub fn detect_touch_gesture(
        &self,
        controller_id: u32,
        previous: &[TouchPoint; 2],
        current: &[TouchPoint; 2],
    ) -> Result<Option<TouchGesture>, ControllerError> {
        match self.platform {
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                // Analyze touch points to detect gestures
                // This is a simplified implementation

                // Check for tap
                if current[0].touching && !previous[0].touching {
                    // Further analysis needed to distinguish tap from swipe
                }

                // Real implementation would track touch history and detect gestures
                Ok(None)
            }
            _ => Err(ControllerError::NotSupported),
        }
    }

    /// Get motion data (gyro and accelerometer)
    ///
    /// Supports:
    /// - Nintendo Switch: Joy-Con motion sensors
    /// - PlayStation 5/4: DualShock/DualSense motion sensors
    /// - Xbox: Not supported
    pub fn get_motion_data(&self, controller_id: u32) -> Result<MotionData, ControllerError> {
        match self.platform {
            ConsolePlatform::NintendoSwitch => {
                tracing::trace!(
                    "Reading Joy-Con motion data for controller {}",
                    controller_id
                );

                // Real implementation: nn::hid::GetSixAxisSensor
                let raw_data = MotionData::new(
                    (0.01, 0.02, 0.03), // Mock gyro data (rad/s)
                    (0.0, 0.0, 9.8),    // Mock accel data (m/s²)
                );

                // Apply calibration if available
                if let Some(calibration) = self.get_motion_calibration(controller_id) {
                    Ok(calibration.apply(&raw_data))
                } else {
                    Ok(raw_data)
                }
            }
            ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4 => {
                tracing::trace!(
                    "Reading DualSense/DualShock motion data for controller {}",
                    controller_id
                );

                // Real implementation: scePadGetMotionSensor
                let raw_data = MotionData::new(
                    (0.015, 0.025, 0.035), // Mock gyro data (rad/s)
                    (0.1, 0.2, 9.7),       // Mock accel data (m/s²)
                );

                // Apply calibration if available
                if let Some(calibration) = self.get_motion_calibration(controller_id) {
                    Ok(calibration.apply(&raw_data))
                } else {
                    Ok(raw_data)
                }
            }
            ConsolePlatform::XboxSeries | ConsolePlatform::XboxOne => {
                // Xbox controllers don't have motion sensors
                Err(ControllerError::NotSupported)
            }
        }
    }

    /// Get controller orientation from motion sensors
    pub fn get_orientation(&self, controller_id: u32) -> Result<Orientation, ControllerError> {
        match self.platform {
            ConsolePlatform::NintendoSwitch
            | ConsolePlatform::PlayStation5
            | ConsolePlatform::PlayStation4 => {
                // Real implementation would integrate gyro data over time
                // For now, return a mock orientation
                Ok(Orientation::new(0.1, 0.2, 0.3))
            }
            _ => Err(ControllerError::NotSupported),
        }
    }

    /// Set haptic feedback for DualSense (PS5 only)
    ///
    /// DualSense provides precise haptic feedback with multiple actuators
    pub fn set_haptic_feedback(
        &self,
        controller_id: u32,
        feedback: HapticFeedback,
    ) -> Result<(), ControllerError> {
        match self.platform {
            ConsolePlatform::PlayStation5 => {
                tracing::info!(
                    "DualSense haptic feedback for controller {}: left={:.2}, right={:.2}",
                    controller_id,
                    feedback.left,
                    feedback.right
                );

                // Real implementation: scePadSetVibration (with DualSense extensions)
                Ok(())
            }
            _ => Err(ControllerError::NotSupported),
        }
    }

    /// Set trigger resistance (DualSense adaptive triggers)
    ///
    /// Supports various trigger effects for immersive gameplay
    pub fn set_trigger_effect(
        &self,
        controller_id: u32,
        left: TriggerEffect,
        right: TriggerEffect,
    ) -> Result<(), ControllerError> {
        match self.platform {
            ConsolePlatform::PlayStation5 => {
                tracing::info!(
                    "DualSense trigger effect for controller {}: left={:?}, right={:?}",
                    controller_id,
                    left,
                    right
                );

                // Real implementation: scePadSetTriggerEffect
                Ok(())
            }
            _ => Err(ControllerError::NotSupported),
        }
    }

    /// Set trigger resistance (deprecated - use set_trigger_effect)
    pub fn set_trigger_resistance(
        &self,
        controller_id: u32,
        left: f32,
        right: f32,
    ) -> Result<(), ControllerError> {
        self.set_trigger_effect(
            controller_id,
            TriggerEffect::Constant { strength: left },
            TriggerEffect::Constant { strength: right },
        )
    }

    /// Check if feature is supported on current platform
    pub fn supports_feature(&self, feature: ControllerFeature) -> bool {
        match feature {
            ControllerFeature::Vibration => true,
            ControllerFeature::HDRumble => self.platform == ConsolePlatform::NintendoSwitch,
            ControllerFeature::HapticFeedback => self.platform == ConsolePlatform::PlayStation5,
            ControllerFeature::Led => matches!(
                self.platform,
                ConsolePlatform::NintendoSwitch
                    | ConsolePlatform::PlayStation5
                    | ConsolePlatform::PlayStation4
            ),
            ControllerFeature::Touchpad => matches!(
                self.platform,
                ConsolePlatform::PlayStation5 | ConsolePlatform::PlayStation4
            ),
            ControllerFeature::Motion => matches!(
                self.platform,
                ConsolePlatform::NintendoSwitch
                    | ConsolePlatform::PlayStation5
                    | ConsolePlatform::PlayStation4
            ),
            ControllerFeature::AdaptiveTriggers => self.platform == ConsolePlatform::PlayStation5,
        }
    }
}

/// Controller features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerFeature {
    Vibration,
    HDRumble,
    HapticFeedback,
    Led,
    Touchpad,
    Motion,
    AdaptiveTriggers,
}

/// Controller errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerError {
    NotConnected,
    NotSupported,
    InvalidControllerId,
    CalibrationFailed,
    PlatformError(String),
}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerError::NotConnected => write!(f, "Controller not connected"),
            ControllerError::NotSupported => write!(f, "Feature not supported on this platform"),
            ControllerError::InvalidControllerId => write!(f, "Invalid controller ID"),
            ControllerError::CalibrationFailed => write!(f, "Motion calibration failed"),
            ControllerError::PlatformError(msg) => write!(f, "Platform error: {msg}"),
        }
    }
}

impl std::error::Error for ControllerError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for VibrationIntensity
    #[test]
    fn test_vibration_intensity_new() {
        let vibration = VibrationIntensity::new(0.5, 0.7);
        assert_eq!(vibration.weak_motor, 0.5);
        assert_eq!(vibration.strong_motor, 0.7);
    }

    #[test]
    fn test_vibration_intensity_clamping() {
        let clamped = VibrationIntensity::new(1.5, -0.5);
        assert_eq!(clamped.weak_motor, 1.0);
        assert_eq!(clamped.strong_motor, 0.0);
    }

    #[test]
    fn test_vibration_intensity_presets() {
        assert!(VibrationIntensity::off().is_off());
        assert!(!VibrationIntensity::gentle().is_off());
        assert_eq!(VibrationIntensity::max().weak_motor, 1.0);
    }

    #[test]
    fn test_vibration_intensity_lerp() {
        let v1 = VibrationIntensity::new(0.0, 0.0);
        let v2 = VibrationIntensity::new(1.0, 1.0);
        let v_mid = v1.lerp(&v2, 0.5);
        assert_eq!(v_mid.weak_motor, 0.5);
        assert_eq!(v_mid.strong_motor, 0.5);
    }

    // Tests for LedColor
    #[test]
    fn test_led_color_presets() {
        let red = LedColor::red();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        let green = LedColor::green();
        assert_eq!(green.r, 0);
        assert_eq!(green.g, 255);
        assert_eq!(green.b, 0);

        let blue = LedColor::blue();
        assert_eq!(blue.r, 0);
        assert_eq!(blue.g, 0);
        assert_eq!(blue.b, 255);
    }

    #[test]
    fn test_led_color_hsv() {
        // Red from HSV (hue=0)
        let red = LedColor::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }

    #[test]
    fn test_led_color_lerp() {
        let c1 = LedColor::new(0, 0, 0);
        let c2 = LedColor::new(255, 255, 255);
        let c_mid = c1.lerp(&c2, 0.5);
        assert_eq!(c_mid.r, 127);
        assert_eq!(c_mid.g, 127);
        assert_eq!(c_mid.b, 127);
    }

    // Tests for TouchPoint
    #[test]
    fn test_touch_point_new() {
        let point = TouchPoint::new(true, 0.5, 0.7);
        assert!(point.touching);
        assert_eq!(point.x, 0.5);
        assert_eq!(point.y, 0.7);
    }

    #[test]
    fn test_touch_point_clamping() {
        let point = TouchPoint::new(true, 1.5, -0.5);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 0.0);
    }

    #[test]
    fn test_touch_point_distance() {
        let p1 = TouchPoint::new(true, 0.0, 0.0);
        let p2 = TouchPoint::new(true, 1.0, 0.0);
        let distance = p1.distance_to(&p2);
        assert!((distance - 1.0).abs() < 0.001);
    }

    // Tests for MotionData
    #[test]
    fn test_motion_data_new() {
        let motion = MotionData::new((0.1, 0.2, 0.3), (1.0, 2.0, 3.0));
        assert_eq!(motion.gyro, (0.1, 0.2, 0.3));
        assert_eq!(motion.accel, (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_motion_data_magnitude() {
        let motion = MotionData::new((3.0, 4.0, 0.0), (1.0, 0.0, 0.0));
        assert!((motion.gyro_magnitude() - 5.0).abs() < 0.001);
        assert!((motion.accel_magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_motion_data_shaken() {
        let motion = MotionData::new((0.0, 0.0, 0.0), (20.0, 0.0, 0.0));
        assert!(motion.is_shaken(15.0));
        assert!(!motion.is_shaken(25.0));
    }

    // Tests for HapticFeedback
    #[test]
    fn test_haptic_feedback_new() {
        let feedback = HapticFeedback::new(0.3, 0.7);
        assert_eq!(feedback.left, 0.3);
        assert_eq!(feedback.right, 0.7);
    }

    #[test]
    fn test_haptic_feedback_presets() {
        assert!(HapticFeedback::off().is_off());
        assert!(HapticFeedback::light().left > 0.0);
        assert!(HapticFeedback::medium().left > HapticFeedback::light().left);
        assert!(HapticFeedback::strong().left > HapticFeedback::medium().left);
    }

    #[test]
    fn test_haptic_feedback_uniform() {
        let uniform = HapticFeedback::uniform(0.5);
        assert_eq!(uniform.left, 0.5);
        assert_eq!(uniform.right, 0.5);
    }

    // Tests for MotionCalibration
    #[test]
    fn test_motion_calibration_none() {
        let calibration = MotionCalibration::none();
        let data = MotionData::new((0.1, 0.2, 0.3), (1.0, 2.0, 3.0));
        let calibrated = calibration.apply(&data);
        assert_eq!(calibrated.gyro, data.gyro);
        assert_eq!(calibrated.accel, data.accel);
    }

    #[test]
    fn test_motion_calibration_apply() {
        let calibration = MotionCalibration {
            gyro_offset: (0.01, 0.02, 0.03),
            accel_offset: (0.1, 0.2, 0.3),
        };
        let data = MotionData::new((0.11, 0.22, 0.33), (1.1, 2.2, 3.3));
        let calibrated = calibration.apply(&data);
        assert_eq!(calibrated.gyro, (0.1, 0.2, 0.3));
        assert_eq!(calibrated.accel, (1.0, 2.0, 3.0));
    }

    // Tests for ExtendedControllerManager
    #[test]
    fn test_controller_manager_new() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
        assert!(manager.supports_feature(ControllerFeature::HapticFeedback));
        assert!(manager.supports_feature(ControllerFeature::AdaptiveTriggers));
    }

    #[test]
    fn test_controller_manager_vibration() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
        let vibration = VibrationIntensity::new(0.7, 0.5);
        assert!(manager.set_vibration(0, vibration).is_ok());
    }

    #[test]
    fn test_controller_manager_led() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
        let color = LedColor::red();
        assert!(manager.set_led_color(0, color).is_ok());
    }

    #[test]
    fn test_controller_manager_haptic() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
        let haptic = HapticFeedback::new(0.5, 0.5);
        assert!(manager.set_haptic_feedback(0, haptic).is_ok());
    }

    #[test]
    fn test_controller_manager_xbox_led_not_supported() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::XboxSeries);
        let color = LedColor::red();
        assert!(matches!(
            manager.set_led_color(0, color),
            Err(ControllerError::NotSupported)
        ));
    }

    #[test]
    fn test_controller_manager_motion_not_supported_xbox() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::XboxOne);
        assert!(matches!(
            manager.get_motion_data(0),
            Err(ControllerError::NotSupported)
        ));
    }

    #[test]
    fn test_controller_manager_motion_calibrated() {
        let mut manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);
        let calibration = MotionCalibration {
            gyro_offset: (0.01, 0.02, 0.03),
            accel_offset: (0.0, 0.0, 0.0),
        };
        manager.set_motion_calibration(0, calibration);

        let motion = manager.get_motion_data(0);
        assert!(motion.is_ok());
    }

    #[test]
    fn test_controller_manager_features() {
        let switch_manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);
        assert!(switch_manager.supports_feature(ControllerFeature::HDRumble));
        assert!(!switch_manager.supports_feature(ControllerFeature::HapticFeedback));
        assert!(switch_manager.supports_feature(ControllerFeature::Motion));

        let ps5_manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
        assert!(!ps5_manager.supports_feature(ControllerFeature::HDRumble));
        assert!(ps5_manager.supports_feature(ControllerFeature::HapticFeedback));
        assert!(ps5_manager.supports_feature(ControllerFeature::AdaptiveTriggers));

        let xbox_manager = ExtendedControllerManager::new(ConsolePlatform::XboxSeries);
        assert!(!xbox_manager.supports_feature(ControllerFeature::Motion));
        assert!(!xbox_manager.supports_feature(ControllerFeature::Touchpad));
    }

    #[test]
    fn test_trigger_effects() {
        let weapon = TriggerEffect::weapon();
        assert!(matches!(weapon, TriggerEffect::Zones { .. }));

        let bow = TriggerEffect::bow();
        assert!(matches!(bow, TriggerEffect::Linear { .. }));

        let machine_gun = TriggerEffect::machine_gun();
        assert!(matches!(machine_gun, TriggerEffect::Vibration { .. }));
    }

    #[test]
    fn test_hd_rumble_patterns() {
        let gentle = HDRumblePattern::gentle_pulse();
        assert!(gentle.low_band.0 < 0.5);

        let impact = HDRumblePattern::sharp_impact();
        assert!(impact.low_band.0 > 0.5);

        let continuous = HDRumblePattern::continuous(0.7);
        assert_eq!(continuous.low_band.0, 0.7);
    }

    #[test]
    fn test_led_effects() {
        let breathing = LedEffect::breathing(1000);
        assert!(matches!(breathing, LedEffect::Breathing { .. }));

        let blinking = LedEffect::blinking(500, 500);
        assert!(matches!(blinking, LedEffect::Blinking { .. }));

        let rainbow = LedEffect::rainbow(2000);
        assert!(matches!(rainbow, LedEffect::Rainbow { .. }));
    }

    #[test]
    fn test_controller_manager_trigger_effects() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

        let left = TriggerEffect::weapon();
        let right = TriggerEffect::bow();
        assert!(manager.set_trigger_effect(0, left, right).is_ok());
    }

    #[test]
    fn test_controller_manager_hd_rumble() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);

        let pattern = HDRumblePattern::sharp_impact();
        assert!(manager.set_hd_rumble(0, pattern).is_ok());
    }

    #[test]
    fn test_controller_manager_hd_rumble_not_supported_on_ps5() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

        let pattern = HDRumblePattern::sharp_impact();
        assert!(matches!(
            manager.set_hd_rumble(0, pattern),
            Err(ControllerError::NotSupported)
        ));
    }

    #[test]
    fn test_controller_manager_touch_input() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

        let touch = manager.get_touch_input(0);
        assert!(touch.is_ok());
        assert_eq!(touch.unwrap().len(), 2);
    }

    #[test]
    fn test_controller_manager_touch_input_not_supported_on_switch() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);

        assert!(matches!(
            manager.get_touch_input(0),
            Err(ControllerError::NotSupported)
        ));
    }

    #[test]
    fn test_controller_manager_orientation() {
        let manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);

        let orientation = manager.get_orientation(0);
        assert!(orientation.is_ok());
        let ori = orientation.unwrap();
        assert!(ori.pitch.abs() < 1.0);
        assert!(ori.roll.abs() < 1.0);
        assert!(ori.yaw.abs() < 1.0);
    }
}
