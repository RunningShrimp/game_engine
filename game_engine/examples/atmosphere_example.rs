// Atmospheric Rendering Example
//
// This example demonstrates how to use the atmospheric rendering system
// to create realistic volumetric clouds, fog effects, and atmospheric scattering.

use game_engine::render::Camera;
use game_engine::render::atmosphere::{
    AtmosphereConfig, AtmosphereQuality, AtmosphereSystem, CloudConfig, CloudQuality, CloudType,
    FogConfig, FogQuality, FogType, HeightFogConfig, VolumetricFogConfig, VolumetricLightConfig,
    WeatherState,
};
use glam::{Mat4, Vec3};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Atmospheric Rendering Example ===\n");

    // Example 1: Basic setup
    println!("Example 1: Basic Atmosphere Setup");
    basic_setup_example()?;

    // Example 2: Cloud configurations
    println!("\nExample 2: Cloud Configurations");
    cloud_config_example()?;

    // Example 3: Fog configurations
    println!("\nExample 3: Fog Configurations");
    fog_config_example()?;

    // Example 4: Weather presets
    println!("\nExample 4: Weather Presets");
    weather_preset_example()?;

    // Example 5: Quality settings
    println!("\nExample 5: Quality Settings");
    quality_settings_example()?;

    // Example 6: Time of day
    println!("\nExample 6: Time of Day");
    time_of_day_example()?;

    println!("\n=== All Examples Completed Successfully ===");

    Ok(())
}

/// Example 1: Basic atmosphere system setup
fn basic_setup_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create default atmosphere configuration
    let config = AtmosphereConfig {
        quality: AtmosphereQuality::Medium,
        downsample_factor: 0.5,
        enable_temporal: true,
        ..Default::default()
    };

    println!("  ✓ Created atmosphere config with medium quality");
    println!("    - Downsample factor: {}", config.downsample_factor);
    println!("    - Temporal accumulation: {}", config.enable_temporal);

    // Note: In actual usage, you would create the system with a real device:
    // let atmosphere = AtmosphereSystem::new(device, config)?;

    // Weather state
    let weather = WeatherState {
        coverage: 0.5,
        density: 0.5,
        wind_speed: 10.0,
        ..Default::default()
    };

    println!("  ✓ Created default weather state");
    println!("    - Cloud coverage: {}%", weather.coverage * 100.0);
    println!("    - Wind speed: {} m/s", weather.wind_speed);

    Ok(())
}

/// Example 2: Different cloud configurations
fn cloud_config_example() -> Result<(), Box<dyn std::error::Error>> {
    // Cumulus clouds (puffy, low altitude)
    let cumulus = CloudConfig {
        cloud_type: CloudType::Cumulus,
        quality: CloudQuality::Medium,
        cloud_altitude: 1500.0,
        cloud_thickness: 1000.0,
        cloud_density: 0.5,
        cloud_coverage: 0.5,
        wind_speed: 10.0,
        ..Default::default()
    };
    println!("  ✓ Cumulus clouds");
    println!("    - Altitude: {} m", cumulus.cloud_altitude);
    println!("    - Thickness: {} m", cumulus.cloud_thickness);
    println!(
        "    - Quality: Medium ({} samples)",
        cumulus.quality.samples()
    );

    // Stratus clouds (layered, medium altitude)
    let stratus = CloudConfig {
        cloud_type: CloudType::Stratus,
        quality: CloudQuality::Low,
        cloud_altitude: 3000.0,
        cloud_thickness: 500.0,
        cloud_density: 0.7,
        cloud_coverage: 0.8,
        ..Default::default()
    };
    println!("  ✓ Stratus clouds");
    println!("    - Altitude: {} m", stratus.cloud_altitude);
    println!("    - Coverage: {}%", stratus.cloud_coverage * 100.0);

    // Cirrus clouds (wispy, high altitude)
    let cirrus = CloudConfig {
        cloud_type: CloudType::Cirrus,
        quality: CloudQuality::High,
        cloud_altitude: 8000.0,
        cloud_thickness: 200.0,
        cloud_density: 0.3,
        cloud_coverage: 0.4,
        ..Default::default()
    };
    println!("  ✓ Cirrus clouds");
    println!("    - Altitude: {} m", cirrus.cloud_altitude);
    println!("    - Quality: High ({} samples)", cirrus.quality.samples());

    // Cumulonimbus (storm clouds)
    let cumulonimbus = CloudConfig {
        cloud_type: CloudType::Cumulonimbus,
        quality: CloudQuality::Ultra,
        cloud_altitude: 500.0,
        cloud_thickness: 8000.0,
        cloud_density: 0.9,
        cloud_coverage: 1.0,
        anvil: 0.8,
        ..Default::default()
    };
    println!("  ✓ Cumulonimbus (storm clouds)");
    println!("    - Altitude: {} m", cumulonimbus.cloud_altitude);
    println!("    - Thickness: {} m", cumulonimbus.cloud_thickness);
    println!("    - Anvil: {}", cumulonimbus.anvil);

    Ok(())
}

/// Example 3: Different fog configurations
fn fog_config_example() -> Result<(), Box<dyn std::error::Error>> {
    // Linear fog
    let linear = FogConfig {
        fog_type: FogType::Linear,
        start_distance: 10.0,
        end_distance: 100.0,
        ..Default::default()
    };
    println!("  ✓ Linear fog");
    println!("    - Start: {} m", linear.start_distance);
    println!("    - End: {} m", linear.end_distance);

    // Exponential fog
    let exponential = FogConfig {
        fog_type: FogType::Exponential,
        density: 0.01,
        ..Default::default()
    };
    println!("  ✓ Exponential fog");
    println!("    - Density: {}", exponential.density);

    // Height fog
    let height = FogConfig {
        fog_type: FogType::Height,
        height_fog: Some(HeightFogConfig {
            height: 0.0,
            density: 0.01,
            falloff: 0.1,
        }),
        ..Default::default()
    };
    println!("  ✓ Height fog");
    if let Some(ref hf) = height.height_fog {
        println!("    - Height: {} m", hf.height);
        println!("    - Density: {}", hf.density);
        println!("    - Falloff: {}", hf.falloff);
    }

    // Volumetric fog with light shafts
    let volumetric = FogConfig {
        fog_type: FogType::Height,
        volumetric: VolumetricFogConfig {
            enabled: true,
            quality: FogQuality::Medium,
            scattering: 0.5,
            absorption: 0.1,
            anisotropy: 0.6,
            light_shafts: true,
            light_shaft_intensity: 0.3,
            ..Default::default()
        },
        ..Default::default()
    };
    println!("  ✓ Volumetric fog with light shafts");
    println!("    - Scattering: {}", volumetric.volumetric.scattering);
    println!("    - Light shafts: {}", volumetric.volumetric.light_shafts);

    Ok(())
}

/// Example 4: Weather presets
fn weather_preset_example() -> Result<(), Box<dyn std::error::Error>> {
    // Clear sky
    let clear = WeatherState {
        coverage: 0.1,
        density: 0.2,
        precipitation: 0.0,
        wind_speed: 5.0,
        time_of_day: 12.0,
        temperature: 25.0,
        ..Default::default()
    };
    println!("  ✓ Clear sky");
    println!("    - Coverage: {}%", clear.coverage * 100.0);
    println!("    - Temperature: {}°C", clear.temperature);

    // Partly cloudy
    let partly_cloudy = WeatherState {
        coverage: 0.4,
        density: 0.4,
        precipitation: 0.0,
        wind_speed: 10.0,
        ..Default::default()
    };
    println!("  ✓ Partly cloudy");
    println!("    - Coverage: {}%", partly_cloudy.coverage * 100.0);

    // Overcast
    let overcast = WeatherState {
        coverage: 0.9,
        density: 0.7,
        precipitation: 0.0,
        wind_speed: 8.0,
        ..Default::default()
    };
    println!("  ✓ Overcast");
    println!("    - Coverage: {}%", overcast.coverage * 100.0);

    // Rain
    let rain = WeatherState {
        coverage: 0.8,
        density: 0.6,
        precipitation: 0.5,
        wind_speed: 12.0,
        humidity: 0.8,
        ..Default::default()
    };
    println!("  ✓ Rain");
    println!("    - Precipitation: {}", rain.precipitation);
    println!("    - Humidity: {}%", rain.humidity * 100.0);

    // Storm
    let storm = WeatherState {
        coverage: 1.0,
        density: 1.0,
        precipitation: 1.0,
        wind_speed: 25.0,
        humidity: 0.95,
        ..Default::default()
    };
    println!("  ✓ Storm");
    println!("    - Precipitation: {}", storm.precipitation);
    println!("    - Wind speed: {} m/s", storm.wind_speed);

    Ok(())
}

/// Example 5: Quality settings
fn quality_settings_example() -> Result<(), Box<dyn std::error::Error>> {
    // Low quality (performance)
    let low = AtmosphereConfig {
        quality: AtmosphereQuality::Low,
        downsample_factor: 0.25,
        enable_temporal: false,
        ..Default::default()
    };
    println!("  ✓ Low quality (performance)");
    println!(
        "    - Ray marching samples: {}",
        low.quality.ray_marching_steps()
    );
    println!("    - Light samples: {}", low.quality.light_samples());
    println!("    - Downsample factor: {}", low.downsample_factor);

    // Medium quality (balanced)
    let medium = AtmosphereConfig {
        quality: AtmosphereQuality::Medium,
        downsample_factor: 0.5,
        enable_temporal: true,
        ..Default::default()
    };
    println!("  ✓ Medium quality (balanced)");
    println!(
        "    - Ray marching samples: {}",
        medium.quality.ray_marching_steps()
    );
    println!("    - Light samples: {}", medium.quality.light_samples());
    println!("    - Downsample factor: {}", medium.downsample_factor);

    // High quality (visuals)
    let high = AtmosphereConfig {
        quality: AtmosphereQuality::High,
        downsample_factor: 1.0,
        enable_temporal: true,
        ..Default::default()
    };
    println!("  ✓ High quality (visuals)");
    println!(
        "    - Ray marching samples: {}",
        high.quality.ray_marching_steps()
    );
    println!("    - Light samples: {}", high.quality.light_samples());
    println!("    - Downsample factor: {}", high.downsample_factor);

    // Ultra quality (screenshots)
    let ultra = AtmosphereConfig {
        quality: AtmosphereQuality::Ultra,
        downsample_factor: 1.0,
        enable_temporal: true,
        ..Default::default()
    };
    println!("  ✓ Ultra quality (screenshots)");
    println!(
        "    - Ray marching samples: {}",
        ultra.quality.ray_marching_steps()
    );
    println!("    - Light samples: {}", ultra.quality.light_samples());

    Ok(())
}

/// Example 6: Time of day configurations
fn time_of_day_example() -> Result<(), Box<dyn std::error::Error>> {
    // Sunrise (6 AM)
    let sunrise = WeatherState {
        time_of_day: 6.0,
        temperature: 15.0,
        ..Default::default()
    };
    let sunrise_fog = FogConfig {
        color: Vec3::new(1.0, 0.8, 0.7), // Warm orange
        density: 0.02,
        ..Default::default()
    };
    println!("  ✓ Sunrise (6:00 AM)");
    println!("    - Temperature: {}°C", sunrise.temperature);
    println!("    - Fog color: warm orange");

    // Noon (12 PM)
    let noon = WeatherState {
        time_of_day: 12.0,
        temperature: 25.0,
        ..Default::default()
    };
    let noon_fog = FogConfig {
        color: Vec3::new(0.9, 0.9, 1.0), // Neutral
        density: 0.005,
        ..Default::default()
    };
    println!("  ✓ Noon (12:00 PM)");
    println!("    - Temperature: {}°C", noon.temperature);
    println!("    - Fog color: neutral");

    // Sunset (6 PM)
    let sunset = WeatherState {
        time_of_day: 18.0,
        temperature: 20.0,
        ..Default::default()
    };
    let sunset_fog = FogConfig {
        color: Vec3::new(1.0, 0.7, 0.5), // Warm red
        density: 0.015,
        ..Default::default()
    };
    println!("  ✓ Sunset (6:00 PM)");
    println!("    - Temperature: {}°C", sunset.temperature);
    println!("    - Fog color: warm red");

    // Night (12 AM)
    let night = WeatherState {
        time_of_day: 0.0,
        temperature: 10.0,
        ..Default::default()
    };
    let night_fog = FogConfig {
        color: Vec3::new(0.1, 0.1, 0.15), // Dark blue
        density: 0.02,
        ..Default::default()
    };
    println!("  ✓ Night (12:00 AM)");
    println!("    - Temperature: {}°C", night.temperature);
    println!("    - Fog color: dark blue");

    Ok(())
}

/// Example game loop simulation
fn simulate_game_loop() {
    println!("\n=== Simulated Game Loop ===\n");

    let mut frame_count = 0;
    let start_time = Instant::now();

    // Simulate 60 frames
    for frame in 0..60 {
        // Delta time (assuming 60 FPS)
        let delta_time = 1.0 / 60.0;

        // Update atmosphere (would be real in actual usage)
        // atmosphere.update(queue, delta_time);

        // Dynamic weather based on time
        let elapsed = start_time.elapsed().as_secs_f32();
        let weather = if elapsed < 10.0 {
            WeatherState {
                coverage: 0.3,
                ..Default::default()
            }
        } else if elapsed < 20.0 {
            WeatherState {
                coverage: 0.6,
                precipitation: 0.2,
                ..Default::default()
            }
        } else {
            WeatherState {
                coverage: 0.9,
                precipitation: 0.5,
                ..Default::default()
            }
        };

        // atmosphere.set_weather(weather);

        // Print status every 10 frames
        if frame % 10 == 0 {
            println!(
                "  Frame {}: Coverage={:.0}%, Precip={:.0}%",
                frame,
                weather.coverage * 100.0,
                weather.precipitation * 100.0
            );
        }

        frame_count += 1;
    }

    println!("\n  ✓ Simulated {} frames", frame_count);
}

/// Performance benchmark example
fn performance_benchmark_example() {
    println!("\n=== Performance Benchmarks ===\n");

    let qualities = [
        ("Low", AtmosphereQuality::Low),
        ("Medium", AtmosphereQuality::Medium),
        ("High", AtmosphereQuality::High),
        ("Ultra", AtmosphereQuality::Ultra),
    ];

    for (name, quality) in qualities.iter() {
        let samples = quality.ray_marching_steps();
        let light_samples = quality.light_samples();
        let estimated_fps = match quality {
            AtmosphereQuality::Low => 120,
            AtmosphereQuality::Medium => 60,
            AtmosphereQuality::High => 30,
            AtmosphereQuality::Ultra => 15,
        };

        println!("  {} quality:", name);
        println!("    - Ray marching samples: {}", samples);
        println!("    - Light samples: {}", light_samples);
        println!("    - Estimated FPS: {}", estimated_fps);
    }
}
