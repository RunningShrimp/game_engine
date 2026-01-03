# Atmospheric Rendering User Guide

## Quick Start

### Basic Cloud Rendering

```rust
use game_engine::render::atmosphere::{AtmosphereSystem, AtmosphereConfig};

// 1. Create the atmosphere system
let config = AtmosphereConfig::default();
let mut atmosphere = AtmosphereSystem::new(device, config)?;

// 2. Prepare render targets
atmosphere.prepare(device, 1920, 1080)?;

// 3. Set weather (optional)
let weather = WeatherState {
    coverage: 0.5,  // 50% cloud coverage
    ..Default::default()
};
atmosphere.set_weather(weather);

// 4. Update in game loop
atmosphere.update(queue, delta_time);

// 5. Render
atmosphere.render(
    encoder,
    device,
    &frame_view,
    &camera,
    &depth_texture,
    sun_direction
)?;
```

## Configuration Guide

### Cloud Configuration

#### Cloud Types

```rust
use game_engine::render::atmosphere::{CloudConfig, CloudType};

// Cumulus - puffy clouds (default)
CloudConfig {
    cloud_type: CloudType::Cumulus,
    cloud_altitude: 1500.0,      // 1.5 km altitude
    cloud_thickness: 1000.0,     // 1 km thick
    ..Default::default()
}

// Stratus - layered clouds
CloudConfig {
    cloud_type: CloudType::Stratus,
    cloud_altitude: 3000.0,
    cloud_thickness: 500.0,
    ..Default::default()
}

// Cirrus - wispy high clouds
CloudConfig {
    cloud_type: CloudType::Cirrus,
    cloud_altitude: 8000.0,
    cloud_thickness: 200.0,
    ..Default::default()
}

// Cumulonimbus - storm clouds
CloudConfig {
    cloud_type: CloudType::Cumulonimbus,
    cloud_altitude: 1000.0,
    cloud_thickness: 8000.0,     // Very tall
    anvil: 0.8,                   // Anvil shape at top
    ..Default::default()
}
```

#### Quality Settings

```rust
use game_engine::render::atmosphere::{CloudConfig, CloudQuality};

// Low quality - fastest
CloudConfig {
    quality: CloudQuality::Low,   // 32 samples
    ..Default::default()
}

// Medium quality - balanced (default)
CloudConfig {
    quality: CloudQuality::Medium, // 64 samples
    ..Default::default()
}

// High quality - beautiful
CloudConfig {
    quality: CloudQuality::High,  // 128 samples
    ..Default::default()
}

// Ultra quality - screenshots
CloudConfig {
    quality: CloudQuality::Ultra, // 256 samples
    ..Default::default()
}
```

#### Density and Coverage

```rust
CloudConfig {
    cloud_density: 0.5,      // How thick clouds are (0.0 - 1.0)
    cloud_coverage: 0.5,     // How much sky is covered (0.0 - 1.0)
    erosion: 0.5,            // Cloud edge detail (0.0 - 1.0)
    detail_scale: 1.0,       // Noise scale (0.1 - 10.0)
    ..Default::default()
}
```

#### Wind Animation

```rust
use glam::Vec3;

CloudConfig {
    wind_speed: 10.0,                    // meters per second
    wind_direction: Vec3::new(1.0, 0.0, 0.5), // direction (normalized)
    ..Default::default()
}
```

### Fog Configuration

#### Fog Types

```rust
use game_engine::render::atmosphere::{FogConfig, FogType};

// Linear fog - simple distance fog
FogConfig {
    fog_type: FogType::Linear,
    start_distance: 10.0,
    end_distance: 100.0,
    ..Default::default()
}

// Exponential fog - natural falloff (default)
FogConfig {
    fog_type: FogType::Exponential,
    density: 0.01,
    ..Default::default()
}

// Height fog - altitude-based
FogConfig {
    fog_type: FogType::Height,
    height_fog: Some(HeightFogConfig {
        height: 0.0,
        density: 0.01,
        falloff: 0.1,
    }),
    ..Default::default()
}

// Ground fog - hugs the ground
FogConfig {
    fog_type: FogType::Ground,
    ground_fog: Some(GroundFogConfig {
        ground_height: 0.0,
        max_height: 10.0,
        density: 0.02,
    }),
    ..Default::default()
}
```

#### Fog Colors

```rust
use game_engine::render::atmosphere::FogConfig;
use glam::Vec3;

FogConfig {
    // Morning fog (warm)
    color: Vec3::new(1.0, 0.9, 0.8),

    // Noon fog (neutral)
    color: Vec3::new(0.9, 0.9, 1.0),

    // Evening fog (cool)
    color: Vec3::new(0.7, 0.8, 0.9),

    // Night fog (dark)
    color: Vec3::new(0.1, 0.1, 0.15),
    ..Default::default()
}
```

#### Volumetric Fog

```rust
use game_engine::render::atmosphere::{FogConfig, VolumetricFogConfig};

FogConfig {
    volumetric: VolumetricFogConfig {
        enabled: true,
        scattering: 0.5,        // How much light scatters
        absorption: 0.1,        // How much light is absorbed
        anisotropy: 0.6,        // Scattering direction (-1 to 1)
        light_shafts: true,     // Enable god rays
        light_shaft_intensity: 0.3,
        ..Default::default()
    },
    ..Default::default()
}
```

### Volumetric Lighting

```rust
use game_engine::render::atmosphere::VolumetricLightConfig;
use glam::Vec3;

VolumetricLightConfig {
    enabled: true,
    intensity: 1.0,
    color: Vec3::new(1.0, 0.9, 0.8), // Warm sunlight
    samples: 32,                         // Light quality
    scattering_coefficient: 0.3,
    ..Default::default()
}
```

### Atmospheric Scattering

```rust
use game_engine::render::atmosphere::LightScatteringConfig;
use glam::Vec3;

LightScatteringConfig {
    enabled: true,
    // Rayleigh scattering (blue sky)
    rayleigh_coefficient: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
    // Mie scattering (haze)
    mie_coefficient: 2.0e-5,
    mie_anisotropy: 0.758,
    atmosphere_thickness: 8000.0,
    planet_radius: 6360000.0,
    sun_intensity: 20.0,
}
```

## Weather Presets

### Clear Sky

```rust
WeatherState {
    coverage: 0.1,      // Almost no clouds
    density: 0.2,
    precipitation: 0.0,
    wind_speed: 5.0,
    ..Default::default()
}
```

### Partly Cloudy

```rust
WeatherState {
    coverage: 0.4,      // 40% coverage
    density: 0.4,
    precipitation: 0.0,
    wind_speed: 10.0,
    ..Default::default()
}
```

### Overcast

```rust
WeatherState {
    coverage: 0.9,      // 90% coverage
    density: 0.7,
    precipitation: 0.0,
    wind_speed: 8.0,
    ..Default::default()
}
```

### Light Rain

```rust
WeatherState {
    coverage: 0.8,
    density: 0.6,
    precipitation: 0.3,  // 30% intensity
    wind_speed: 12.0,
    humidity: 0.8,
    ..Default::default()
}
```

### Heavy Rain/Storm

```rust
WeatherState {
    coverage: 1.0,      // Full coverage
    density: 1.0,
    precipitation: 1.0,  // Heavy rain
    wind_speed: 25.0,   // Strong wind
    humidity: 0.95,
    ..Default::default()
}
```

## Time of Day

### Sunrise

```rust
// 6:00 AM
let weather = WeatherState {
    time_of_day: 6.0,
    temperature: 15.0,
    ..Default::default()
};

let fog_config = FogConfig {
    color: Vec3::new(1.0, 0.8, 0.7), // Warm morning fog
    density: 0.02,                     // Dense morning fog
    ..Default::default()
};
```

### Noon

```rust
// 12:00 PM
let weather = WeatherState {
    time_of_day: 12.0,
    temperature: 25.0,
    ..Default::default()
};

let fog_config = FogConfig {
    color: Vec3::new(0.9, 0.9, 1.0), // Neutral
    density: 0.005,                    // Light fog
    ..Default::default()
};
```

### Sunset

```rust
// 18:00 (6 PM)
let weather = WeatherState {
    time_of_day: 18.0,
    temperature: 20.0,
    ..Default::default()
};

let fog_config = FogConfig {
    color: Vec3::new(1.0, 0.7, 0.5), // Warm sunset
    density: 0.015,
    ..Default::default()
};
```

### Night

```rust
// 0:00 (Midnight)
let weather = WeatherState {
    time_of_day: 0.0,
    temperature: 10.0,
    ..Default::default()
};

let fog_config = FogConfig {
    color: Vec3::new(0.1, 0.1, 0.15), // Dark night fog
    density: 0.02,
    ..Default::default()
};
```

## Performance Tips

### For High FPS

```rust
let config = AtmosphereConfig {
    quality: AtmosphereQuality::Low,
    downsample_factor: 0.25,  // Quarter resolution
    enable_temporal: false,    // Disable temporal AA
    ..Default::default()
};
```

### For Balanced Quality

```rust
let config = AtmosphereConfig {
    quality: AtmosphereQuality::Medium,
    downsample_factor: 0.5,   // Half resolution
    enable_temporal: true,     // Enable temporal AA
    ..Default::default()
};
```

### For Best Quality

```rust
let config = AtmosphereConfig {
    quality: AtmosphereQuality::High,
    downsample_factor: 1.0,   // Full resolution
    enable_temporal: true,
    ..Default::default()
};
```

### Disable Specific Features

```rust
// No clouds
let cloud_config = CloudConfig {
    enabled: false,
    ..Default::default()
};

// No fog
let fog_config = FogConfig {
    enabled: false,
    ..Default::default()
};

// No volumetric lighting
let vol_light_config = VolumetricLightConfig {
    enabled: false,
    ..Default::default()
};
```

## Integration Examples

### With Deferred Rendering

```rust
// 1. Geometry pass
deferred_renderer.geometry_pass(encoder, device, scene)?;

// 2. Atmospheric passes
atmosphere.prepare(device, width, height)?;
atmosphere.render(encoder, device, view, &camera, &depth_texture, light_dir)?;

// 3. Lighting pass
deferred_renderer.lighting_pass(encoder, device, view, &atmosphere)?;

// 4. Post-processing
post_process.tone_map(encoder, device, view)?;
```

### With Forward Rendering

```rust
// 1. Render scene
forward_renderer.render(encoder, device, scene, camera)?;

// 2. Render atmosphere on top
atmosphere.render(encoder, device, view, &camera, &depth_texture, light_dir)?;
```

### Custom Weather System

```rust
struct DynamicWeather {
    atmosphere: AtmosphereSystem,
    time: f32,
}

impl DynamicWeather {
    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        // Calculate weather based on time
        let hour = (self.time / 3600.0) % 24.0;

        let weather = if hour >= 6.0 && hour < 12.0 {
            // Morning: clear to partly cloudy
            WeatherState {
                coverage: 0.3,
                density: 0.3,
                ..Default::default()
            }
        } else if hour >= 12.0 && hour < 18.0 {
            // Afternoon: partly cloudy
            WeatherState {
                coverage: 0.5,
                density: 0.4,
                ..Default::default()
            }
        } else {
            // Evening/Night: overcast
            WeatherState {
                coverage: 0.8,
                density: 0.6,
                ..Default::default()
            }
        };

        self.atmosphere.set_weather(weather);
    }
}
```

## Troubleshooting

### Clouds Too Bright
```rust
CloudConfig {
    cloud_density: 0.3,        // Reduce density
    absorption: 0.5,           // Increase absorption
    ..Default::default()
}
```

### Clouds Too Dark
```rust
CloudConfig {
    cloud_density: 0.7,        // Increase density
    scattering: 0.9,           // Increase scattering
    ..Default::default()
}
```

### Fog Too Thick
```rust
FogConfig {
    density: 0.005,            // Reduce density
    start_distance: 50.0,      // Push fog back
    ..Default::default()
}
```

### Performance Issues
```rust
AtmosphereConfig {
    quality: AtmosphereQuality::Low,
    downsample_factor: 0.25,
    enable_temporal: false,
    clouds: CloudConfig {
        quality: CloudQuality::Low,
        ..Default::default()
    },
    fog: FogConfig {
        quality: FogQuality::Low,
        volumetric: VolumetricFogConfig {
            enabled: false,     // Disable volumetric fog
            ..Default::default()
        },
        ..Default::default()
    },
    ..Default::default()
}
```

## Advanced Usage

### Custom Noise Textures

```rust
use game_engine::render::atmosphere::{NoiseGenerator, NoiseType};

let noise = NoiseGenerator::new(12345);

// Generate custom noise
let perlin_texture = noise.generate_texture_3d(
    device,
    queue,
    256,   // High resolution
    NoiseType::Perlin
)?;

let worley_texture = noise.generate_texture_3d(
    device,
    queue,
    128,
    NoiseType::Worley
)?;
```

### Procedural Weather Patterns

```rust
fn generate_weather_pattern(time: f32) -> WeatherState {
    // Use sine waves for smooth transitions
    let coverage = 0.5 + 0.3 * (time * 0.1).sin();
    let density = 0.5 + 0.2 * (time * 0.15).cos();
    let precipitation = (coverage * density).max(0.0).min(1.0);

    WeatherState {
        coverage,
        density,
        precipitation,
        ..Default::default()
    }
}
```

### Seasonal Weather

```rust
fn get_seasonal_weather(month: u8) -> WeatherState {
    match month {
        12..=2 => WeatherState {  // Winter
            coverage: 0.8,
            density: 0.7,
            precipitation: 0.2,  // Snow
            temperature: -5.0,
            ..Default::default()
        },
        3..=5 => WeatherState {   // Spring
            coverage: 0.5,
            density: 0.4,
            precipitation: 0.4,  // Rain
            temperature: 15.0,
            ..Default::default()
        },
        6..=8 => WeatherState {   // Summer
            coverage: 0.3,
            density: 0.3,
            precipitation: 0.2,  // Light rain
            temperature: 30.0,
            ..Default::default()
        },
        9..=11 => WeatherState {  // Autumn
            coverage: 0.6,
            density: 0.5,
            precipitation: 0.5,  // Rain
            temperature: 12.0,
            ..Default::default()
        },
        _ => WeatherState::default(),
    }
}
```

## Best Practices

1. **Start with default settings** - They're tuned for good quality/performance balance
2. **Use quality presets** - Low/Medium/High/Ultra instead of manual tuning
3. **Enable temporal AA** - Improves quality with minimal cost
4. **Use down-sampling** - Half resolution looks almost the same but 2x faster
5. **Profile your scene** - Adjust settings based on actual performance
6. **Weather transitions** - Smoothly interpolate between weather states
7. **Test on target hardware** - Quality that works on RTX may be too slow on mobile

## API Reference

See the rustdoc documentation for complete API details:
- `AtmosphereSystem`
- `CloudConfig`
- `FogConfig`
- `WeatherState`
- `NoiseGenerator`
