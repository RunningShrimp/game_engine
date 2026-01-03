//! # Atmospheric Rendering System
//!
//! This module provides a comprehensive atmospheric rendering system including:
//! - Procedural volumetric clouds
//! - Advanced fog effects
//! - Volumetric lighting
//! - Atmospheric scattering
//!
//! ## Module Structure
//!
//! - [`noise`][]: Noise generation algorithms (Perlin, Simplex, Worley, FBM)
//! - [`clouds`][]: Volumetric cloud simulation and rendering
//! - [`fog`][]: Fog effects (volumetric, height, distance, layered)
//! - [`volumetric`][]: Volumetric rendering techniques (ray marching, light scattering)
//! - [`lighting`][]: Atmospheric lighting (scattering, shadows, multiple scattering)
//! - [`integration`][]: Post-processing integration with deferred rendering
//!
//! ## Features
//!
//! ### Cloud Simulation
//! - 3D noise texture generation
//! - Perlin/Simplex noise for cloud shapes
//! - Worley noise for cloud details
//! - Fractal Brownian Motion (FBM) for realism
//! - Dynamic weather simulation
//!
//! ### Fog Effects
//! - Volumetric fog (height-based)
//! - Exponential distance fog
//! - Layered fog
//! - Ground fog
//! - Atmospheric scattering
//!
//! ### Rendering Techniques
//! - Ray marching for volume rendering
//! - Volumetric shadows
//! - Light scattering (single and multiple)
//! - Down-sampling and up-sampling for performance
//! - Temporal reprojection for quality
//!
//! ## Performance
//!
//! Target performance metrics:
//! - Cloud rendering: >60 FPS (medium quality)
//! - Fog effects: >60 FPS
//! - Volumetric shadows: >45 FPS
//! - Reasonable memory usage
//!
//! ## Usage
//!
//! ```rust
//! use game_engine::render::atmosphere::AtmosphereSystem;
//!
//! // Create atmosphere system
//! let mut atmosphere = AtmosphereSystem::new(device, config)?;
//!
//! // Update weather
//! atmosphere.set_weather(&weather_config);
//!
//! // Render
//! atmosphere.render(&mut render_pass, &camera, &depth_texture)?;
//! ```

pub mod clouds;
pub mod fog;
pub mod integration;
pub mod lighting;
pub mod noise;
pub mod skybox;
pub mod volumetric;

pub use clouds::{
    CloudConfig, CloudQuality, CloudRenderer, CloudType, WeatherState, WeatherSystem,
};
pub use fog::{
    FogConfig, FogQuality, FogRenderer, FogType, GroundFogConfig, HeightFogConfig,
    VolumetricFogConfig,
};
pub use integration::{AtmosphereComposePass, AtmosphereIntegrator};
pub use lighting::{AtmosphericScattering, LightScatteringConfig};
pub use noise::{
    FbmConfig, NoiseGenerator, NoiseQuality, NoiseType, PerlinNoise, SimplexNoise, WorleyNoise,
};
pub use skybox::{DynamicSkybox, SkyboxConfig, TimeOfDay};
pub use volumetric::{
    RayMarchConfig, VolumetricLightConfig, VolumetricRenderer, VolumetricScattering,
    VolumetricShadowConfig,
};

/// Atmospheric rendering system configuration
#[derive(Debug, Clone)]
pub struct AtmosphereConfig {
    /// Cloud configuration
    pub clouds: CloudConfig,
    /// Fog configuration
    pub fog: FogConfig,
    /// Volumetric lighting configuration
    pub volumetric_light: VolumetricLightConfig,
    /// Atmospheric scattering configuration
    pub scattering: LightScatteringConfig,
    /// Quality preset
    pub quality: AtmosphereQuality,
    /// Enable temporal accumulation
    pub enable_temporal: bool,
    /// Down-sampling factor (1.0 = full resolution, 0.5 = half resolution)
    pub downsample_factor: f32,
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            clouds: CloudConfig::default(),
            fog: FogConfig::default(),
            volumetric_light: VolumetricLightConfig::default(),
            scattering: LightScatteringConfig::default(),
            quality: AtmosphereQuality::Medium,
            enable_temporal: true,
            downsample_factor: 0.5,
        }
    }
}

/// Atmospheric rendering quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosphereQuality {
    /// Low quality (fastest)
    Low,
    /// Medium quality (balanced)
    Medium,
    /// High quality (best visuals)
    High,
    /// Ultra quality (maximum quality)
    Ultra,
}

impl AtmosphereQuality {
    /// Get ray marching step count
    pub fn ray_marching_steps(&self) -> u32 {
        match self {
            Self::Low => 32,
            Self::Medium => 64,
            Self::High => 128,
            Self::Ultra => 256,
        }
    }

    /// Get light sampling count
    pub fn light_samples(&self) -> u32 {
        match self {
            Self::Low => 4,
            Self::Medium => 8,
            Self::High => 16,
            Self::Ultra => 32,
        }
    }

    /// Get cloud quality
    pub fn cloud_quality(&self) -> CloudQuality {
        match self {
            Self::Low => CloudQuality::Low,
            Self::Medium => CloudQuality::Medium,
            Self::High => CloudQuality::High,
            Self::Ultra => CloudQuality::Ultra,
        }
    }

    /// Get fog quality
    pub fn fog_quality(&self) -> FogQuality {
        match self {
            Self::Low => FogQuality::Low,
            Self::Medium => FogQuality::Medium,
            Self::High => FogQuality::High,
            Self::Ultra => FogQuality::Ultra,
        }
    }
}

/// Main atmospheric rendering system
///
/// This system integrates all atmospheric rendering components:
/// - Cloud simulation and rendering
/// - Fog effects
/// - Volumetric lighting
/// - Atmospheric scattering
pub struct AtmosphereSystem {
    config: AtmosphereConfig,
    clouds: WeatherSystem,
    fog: FogRenderer,
    volumetric: VolumetricRenderer,
    integrator: AtmosphereIntegrator,
}

impl AtmosphereSystem {
    /// Create a new atmospheric rendering system
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: AtmosphereConfig,
    ) -> Result<Self, crate::error::RenderError> {
        let clouds = WeatherSystem::new(device, queue, config.clouds.clone())?;
        let fog = FogRenderer::new(device, config.fog.clone())?;
        let volumetric = VolumetricRenderer::new(device, config.volumetric_light.clone())?;
        let integrator = AtmosphereIntegrator::new(device)?;

        Ok(Self {
            config,
            clouds,
            fog,
            volumetric,
            integrator,
        })
    }

    /// Update atmospheric conditions
    pub fn update(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        self.clouds.update(queue, delta_time);
    }

    /// Set weather state
    pub fn set_weather(&mut self, weather: WeatherState) {
        self.clouds.set_weather(weather);
    }

    /// Prepare render targets
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), crate::error::RenderError> {
        let clouds_width = (width as f32 * self.config.downsample_factor) as u32;
        let clouds_height = (height as f32 * self.config.downsample_factor) as u32;

        self.clouds.prepare(device, clouds_width, clouds_height)?;
        self.fog.prepare(device, clouds_width, clouds_height)?;
        self.volumetric.prepare(device, clouds_width, clouds_height)?;
        self.integrator.prepare(device, width, height)?;

        Ok(())
    }

    /// Render atmospheric effects
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        camera: &crate::render::volumetric::Camera,
        depth_texture: &wgpu::TextureView,
        light_direction: glam::Vec3,
    ) -> Result<(), crate::error::RenderError> {
        // Render clouds
        self.clouds.render(encoder, device, camera, depth_texture, light_direction)?;

        // Render fog
        self.fog.render(encoder, device, camera, depth_texture)?;

        // Render volumetric lighting
        self.volumetric
            .render(encoder, device, camera, depth_texture, light_direction)?;

        // Compose final result
        self.integrator.compose(encoder, device, view)?;

        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &AtmosphereConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(
        &mut self,
        device: &wgpu::Device,
        config: AtmosphereConfig,
    ) -> Result<(), crate::error::RenderError> {
        self.clouds.update_config(device, config.clouds.clone())?;
        self.fog.update_config(device, config.fog.clone())?;
        self.volumetric.update_config(device, config.volumetric_light.clone())?;
        self.config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atmosphere_config_default() {
        let config = AtmosphereConfig::default();
        assert_eq!(config.quality, AtmosphereQuality::Medium);
        assert!(config.enable_temporal);
        assert_eq!(config.downsample_factor, 0.5);
    }

    #[test]
    fn test_quality_presets() {
        assert_eq!(AtmosphereQuality::Low.ray_marching_steps(), 32);
        assert_eq!(AtmosphereQuality::Medium.ray_marching_steps(), 64);
        assert_eq!(AtmosphereQuality::High.ray_marching_steps(), 128);
        assert_eq!(AtmosphereQuality::Ultra.ray_marching_steps(), 256);
    }

    #[test]
    fn test_quality_samples() {
        assert_eq!(AtmosphereQuality::Low.light_samples(), 4);
        assert_eq!(AtmosphereQuality::Medium.light_samples(), 8);
        assert_eq!(AtmosphereQuality::High.light_samples(), 16);
        assert_eq!(AtmosphereQuality::Ultra.light_samples(), 32);
    }
}
