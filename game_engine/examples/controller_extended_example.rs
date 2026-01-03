//! Extended Controller Features Example
//!
//! This example demonstrates how to use the extended controller features
//! for different console platforms.

use game_engine::platform::console::ConsolePlatform;
use game_engine::platform::console::controller_extended::*;

fn main() {
    println!("=== Extended Controller Features Example ===\n");

    // Example 1: Vibration on different platforms
    println!("1. Vibration Examples:");
    vibration_example();

    // Example 2: LED control
    println!("\n2. LED Control Examples:");
    led_example();

    // Example 3: PS5 Haptic Feedback
    println!("\n3. PS5 Haptic Feedback Examples:");
    ps5_haptic_example();

    // Example 4: PS5 Adaptive Triggers
    println!("\n4. PS5 Adaptive Triggers Examples:");
    ps5_triggers_example();

    // Example 5: Switch HD Rumble
    println!("\n5. Switch HD Rumble Examples:");
    switch_hd_rumble_example();

    // Example 6: Motion controls
    println!("\n6. Motion Control Examples:");
    motion_example();

    // Example 7: Touchpad input
    println!("\n7. Touchpad Input Examples:");
    touchpad_example();

    println!("\n=== Examples Complete ===");
}

/// Example 1: Vibration on different platforms
fn vibration_example() {
    // PlayStation 5
    let ps5_manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);
    let vibration = VibrationIntensity::new(0.7, 0.5);

    match ps5_manager.set_vibration(0, vibration) {
        Ok(_) => println!("  ✓ PS5 vibration set successfully"),
        Err(e) => println!("  ✗ PS5 vibration failed: {}", e),
    }

    // Nintendo Switch with HD Rumble
    let switch_manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);
    let gentle_vibration = VibrationIntensity::gentle();

    match switch_manager.set_vibration(0, gentle_vibration) {
        Ok(_) => println!("  ✓ Switch HD Rumble set successfully"),
        Err(e) => println!("  ✗ Switch HD Rumble failed: {}", e),
    }

    // Xbox
    let xbox_manager = ExtendedControllerManager::new(ConsolePlatform::XboxSeries);
    let strong_vibration = VibrationIntensity::strong();

    match xbox_manager.set_vibration(0, strong_vibration) {
        Ok(_) => println!("  ✓ Xbox vibration set successfully"),
        Err(e) => println!("  ✗ Xbox vibration failed: {}", e),
    }
}

/// Example 2: LED control
fn led_example() {
    // PlayStation 5
    let ps5_manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

    // Red color for danger/low health
    let red = LedColor::red();
    match ps5_manager.set_led_color(0, red) {
        Ok(_) => println!("  ✓ PS5 light bar set to red"),
        Err(e) => println!("  ✗ PS5 LED failed: {}", e),
    }

    // Green color for success/full health
    let green = LedColor::green();
    match ps5_manager.set_led_color(0, green) {
        Ok(_) => println!("  ✓ PS5 light bar set to green"),
        Err(e) => println!("  ✗ PS5 LED failed: {}", e),
    }

    // Breathing effect
    let breathing = LedEffect::breathing(1000);
    match ps5_manager.set_led_effect(0, LedColor::cyan(), breathing) {
        Ok(_) => println!("  ✓ PS5 breathing LED effect set"),
        Err(e) => println!("  ✗ PS5 LED effect failed: {}", e),
    }

    // Rainbow effect
    let rainbow = LedEffect::rainbow(2000);
    match ps5_manager.set_led_effect(0, LedColor::white(), rainbow) {
        Ok(_) => println!("  ✓ PS5 rainbow LED effect set"),
        Err(e) => println!("  ✗ PS5 LED effect failed: {}", e),
    }

    // HSV color (golden yellow)
    let gold = LedColor::from_hsv(50.0, 1.0, 1.0);
    match ps5_manager.set_led_color(0, gold) {
        Ok(_) => println!("  ✓ PS5 light bar set to gold (HSV)"),
        Err(e) => println!("  ✗ PS5 LED failed: {}", e),
    }

    // Nintendo Switch
    let switch_manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);
    match switch_manager.set_led_color(0, LedColor::blue()) {
        Ok(_) => println!("  ✓ Switch Pro Controller LED set to blue"),
        Err(e) => println!("  ✗ Switch LED failed: {}", e),
    }

    // Xbox (not supported)
    let xbox_manager = ExtendedControllerManager::new(ConsolePlatform::XboxSeries);
    match xbox_manager.set_led_color(0, LedColor::red()) {
        Ok(_) => println!("  ✓ Xbox LED set"),
        Err(e) => println!("  ✓ Xbox LED correctly returns error: {}", e),
    }
}

/// Example 3: PS5 Haptic Feedback
fn ps5_haptic_example() {
    let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

    // Light haptic feedback for UI interaction
    let light = HapticFeedback::light();
    match manager.set_haptic_feedback(0, light) {
        Ok(_) => println!("  ✓ Light haptic feedback set"),
        Err(e) => println!("  ✗ Haptic feedback failed: {}", e),
    }

    // Medium haptic feedback for picking up items
    let medium = HapticFeedback::medium();
    match manager.set_haptic_feedback(0, medium) {
        Ok(_) => println!("  ✓ Medium haptic feedback set"),
        Err(e) => println!("  ✗ Haptic feedback failed: {}", e),
    }

    // Strong haptic feedback for impacts
    let strong = HapticFeedback::strong();
    match manager.set_haptic_feedback(0, strong) {
        Ok(_) => println!("  ✓ Strong haptic feedback set"),
        Err(e) => println!("  ✗ Haptic feedback failed: {}", e),
    }

    // Asymmetric haptic feedback (left footstep, right silent)
    let asymmetric = HapticFeedback::new(0.6, 0.0);
    match manager.set_haptic_feedback(0, asymmetric) {
        Ok(_) => println!("  ✓ Asymmetric haptic feedback set"),
        Err(e) => println!("  ✗ Haptic feedback failed: {}", e),
    }
}

/// Example 4: PS5 Adaptive Triggers
fn ps5_triggers_example() {
    let manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

    // Weapon trigger effect
    let weapon_trigger = TriggerEffect::weapon();
    match manager.set_trigger_effect(0, weapon_trigger, TriggerEffect::none()) {
        Ok(_) => println!("  ✓ Weapon trigger effect set (L2)"),
        Err(e) => println!("  ✗ Trigger effect failed: {}", e),
    }

    // Bow draw effect
    let bow_trigger = TriggerEffect::bow();
    match manager.set_trigger_effect(0, TriggerEffect::none(), bow_trigger) {
        Ok(_) => println!("  ✓ Bow draw effect set (R2)"),
        Err(e) => println!("  ✗ Trigger effect failed: {}", e),
    }

    // Machine gun vibration
    let machine_gun = TriggerEffect::machine_gun();
    match manager.set_trigger_effect(0, machine_gun, TriggerEffect::none()) {
        Ok(_) => println!("  ✓ Machine gun effect set (L2)"),
        Err(e) => println!("  ✗ Trigger effect failed: {}", e),
    }

    // Constant resistance for car accelerator
    let gas_pedal = TriggerEffect::constant(0.3);
    match manager.set_trigger_effect(0, TriggerEffect::none(), gas_pedal) {
        Ok(_) => println!("  ✓ Gas pedal effect set (R2)"),
        Err(e) => println!("  ✗ Trigger effect failed: {}", e),
    }
}

/// Example 5: Switch HD Rumble
fn switch_hd_rumble_example() {
    let manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);

    // Gentle pulse
    let gentle = HDRumblePattern::gentle_pulse();
    match manager.set_hd_rumble(0, gentle) {
        Ok(_) => println!("  ✓ Gentle HD rumble pulse set"),
        Err(e) => println!("  ✗ HD rumble failed: {}", e),
    }

    // Sharp impact
    let impact = HDRumblePattern::sharp_impact();
    match manager.set_hd_rumble(0, impact) {
        Ok(_) => println!("  ✓ Sharp HD rumble impact set"),
        Err(e) => println!("  ✗ HD rumble failed: {}", e),
    }

    // Continuous rumble
    let continuous = HDRumblePattern::continuous(0.5);
    match manager.set_hd_rumble(0, continuous) {
        Ok(_) => println!("  ✓ Continuous HD rumble set"),
        Err(e) => println!("  ✗ HD rumble failed: {}", e),
    }

    // Custom pattern (rain-like effect)
    let rain = HDRumblePattern::new((0.1, 0.3), (0.3, 0.1));
    match manager.set_hd_rumble(0, rain) {
        Ok(_) => println!("  ✓ Custom rain pattern HD rumble set"),
        Err(e) => println!("  ✗ HD rumble failed: {}", e),
    }
}

/// Example 6: Motion controls
fn motion_example() {
    // Nintendo Switch Joy-Con motion
    let switch_manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);

    match switch_manager.get_motion_data(0) {
        Ok(motion) => {
            println!("  ✓ Switch motion data retrieved:");
            println!(
                "    Gyro: ({:.3}, {:.3}, {:.3}) rad/s",
                motion.gyro.0, motion.gyro.1, motion.gyro.2
            );
            println!(
                "    Accel: ({:.3}, {:.3}, {:.3}) m/s²",
                motion.accel.0, motion.accel.1, motion.accel.2
            );
            println!("    Gyro magnitude: {:.3} rad/s", motion.gyro_magnitude());
            println!("    Accel magnitude: {:.3} m/s²", motion.accel_magnitude());
        }
        Err(e) => println!("  ✗ Failed to get motion data: {}", e),
    }

    match switch_manager.get_orientation(0) {
        Ok(orientation) => {
            println!("  ✓ Switch orientation retrieved:");
            let (pitch, roll, yaw) = orientation.to_degrees();
            println!("    Pitch: {:.1}°", pitch);
            println!("    Roll: {:.1}°", roll);
            println!("    Yaw: {:.1}°", yaw);
        }
        Err(e) => println!("  ✗ Failed to get orientation: {}", e),
    }

    // PS5 DualSense motion
    let ps5_manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

    match ps5_manager.get_motion_data(0) {
        Ok(motion) => {
            println!("  ✓ PS5 motion data retrieved:");
            println!(
                "    Gyro: ({:.3}, {:.3}, {:.3}) rad/s",
                motion.gyro.0, motion.gyro.1, motion.gyro.2
            );
            println!(
                "    Accel: ({:.3}, {:.3}, {:.3}) m/s²",
                motion.accel.0, motion.accel.1, motion.accel.2
            );
        }
        Err(e) => println!("  ✗ Failed to get motion data: {}", e),
    }

    // Motion calibration
    let mut manager = ExtendedControllerManager::new(ConsolePlatform::NintendoSwitch);
    let calibration = MotionCalibration {
        gyro_offset: (0.01, 0.02, 0.03),
        accel_offset: (0.0, 0.0, 0.1),
    };
    manager.set_motion_calibration(0, calibration);
    println!("  ✓ Motion calibration data set for controller 0");

    // Check if shaken
    let shaken_motion = MotionData::new((0.0, 0.0, 0.0), (20.0, 0.0, 0.0));
    println!(
        "  ✓ Controller shaken detection: {}",
        shaken_motion.is_shaken(15.0)
    );
}

/// Example 7: Touchpad input
fn touchpad_example() {
    let ps5_manager = ExtendedControllerManager::new(ConsolePlatform::PlayStation5);

    // Get touch input
    match ps5_manager.get_touch_input(0) {
        Ok(touch_points) => {
            println!("  ✓ PS5 touchpad input retrieved:");
            for (i, point) in touch_points.iter().enumerate() {
                println!(
                    "    Touch {}: active={}, position=({:.2}, {:.2})",
                    i, point.touching, point.x, point.y
                );
            }
        }
        Err(e) => println!("  ✗ Failed to get touch input: {}", e),
    }

    // Simulate gesture detection
    let previous = [
        TouchPoint::new(false, 0.0, 0.0),
        TouchPoint::new(false, 0.0, 0.0),
    ];
    let current = [
        TouchPoint::new(true, 0.5, 0.5),
        TouchPoint::new(false, 0.0, 0.0),
    ];

    match ps5_manager.detect_touch_gesture(0, &previous, &current) {
        Ok(gesture) => {
            if let Some(g) = gesture {
                println!("  ✓ Gesture detected: {:?}", g);
            } else {
                println!("  ✓ No gesture detected (tap may need more analysis)");
            }
        }
        Err(e) => println!("  ✗ Failed to detect gesture: {}", e),
    }

    // Distance calculation
    let p1 = TouchPoint::new(true, 0.2, 0.3);
    let p2 = TouchPoint::new(true, 0.8, 0.3);
    let distance = p1.distance_to(&p2);
    println!("  ✓ Distance between two touch points: {:.3}", distance);
}

/// Example 8: Feature detection
fn feature_detection_example() {
    println!("  Platform Feature Support:");

    let platforms = [
        ConsolePlatform::NintendoSwitch,
        ConsolePlatform::PlayStation5,
        ConsolePlatform::PlayStation4,
        ConsolePlatform::XboxSeries,
    ];

    let features = [
        ControllerFeature::Vibration,
        ControllerFeature::HDRumble,
        ControllerFeature::HapticFeedback,
        ControllerFeature::Led,
        ControllerFeature::Touchpad,
        ControllerFeature::Motion,
        ControllerFeature::AdaptiveTriggers,
    ];

    for platform in platforms {
        println!("  {}:", platform.name());
        let manager = ExtendedControllerManager::new(platform);
        for feature in &features {
            let supported = manager.supports_feature(*feature);
            println!("    {:?}: {}", feature, if supported { "✓" } else { "✗" });
        }
    }
}
