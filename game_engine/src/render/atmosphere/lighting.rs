//! # Atmospheric Lighting System
//!
//! This module implements atmospheric lighting effects:
//! - Atmospheric scattering (Rayleigh and Mie)
//! - Light scattering
//! - Volumetric light integration
//! - Multiple scattering approximation

use glam::Vec3;

/// Atmospheric scattering configuration
#[derive(Debug, Clone)]
pub struct LightScatteringConfig {
    /// Enable atmospheric scattering
    pub enabled: bool,
    /// Rayleigh scattering coefficient
    pub rayleigh_coefficient: Vec3,
    /// Mie scattering coefficient
    pub mie_coefficient: f32,
    /// Mie phase function anisotropy
    pub mie_anisotropy: f32,
    /// Atmosphere thickness
    pub atmosphere_thickness: f32,
    /// Planet radius
    pub planet_radius: f32,
    /// Sun intensity
    pub sun_intensity: f32,
}

impl Default for LightScatteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rayleigh_coefficient: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
            mie_coefficient: 2.0e-5,
            mie_anisotropy: 0.758,
            atmosphere_thickness: 8000.0,
            planet_radius: 6360000.0,
            sun_intensity: 20.0,
        }
    }
}

/// Volumetric light configuration
#[derive(Debug, Clone)]
pub struct VolumetricLightConfig {
    /// Enable volumetric lighting
    pub enabled: bool,
    /// Light intensity
    pub intensity: f32,
    /// Light color
    pub color: Vec3,
    /// Number of light samples
    pub samples: u32,
    /// Scattering coefficient
    pub scattering_coefficient: f32,
}

impl Default for VolumetricLightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            color: Vec3::new(1.0, 0.9, 0.8),
            samples: 32,
            scattering_coefficient: 0.3,
        }
    }
}

/// Atmospheric scattering model
pub struct AtmosphericScattering {
    config: LightScatteringConfig,
}

impl AtmosphericScattering {
    /// Create new atmospheric scattering model
    pub fn new(config: LightScatteringConfig) -> Self {
        Self { config }
    }

    /// Calculate Rayleigh scattering
    pub fn rayleigh_scattering(&self, cos_theta: f32) -> Vec3 {
        // Rayleigh phase function: 3/16π * (1 + cos²θ)
        let phase = 3.0 / (16.0 * std::f32::consts::PI) * (1.0 + cos_theta * cos_theta);
        self.config.rayleigh_coefficient * phase
    }

    /// Calculate Mie scattering
    pub fn mie_scattering(&self, cos_theta: f32) -> Vec3 {
        // Henyey-Greenstein phase function
        let g = self.config.mie_anisotropy;
        let num = 1.0 - g * g;
        let denom = 4.0 * std::f32::consts::PI * (1.0 + g * g - 2.0 * g * cos_theta).powf(1.5);
        let phase = num / denom;

        Vec3::splat(self.config.mie_coefficient * phase)
    }

    /// Calculate total scattering
    pub fn total_scattering(&self, cos_theta: f32) -> Vec3 {
        let rayleigh = self.rayleigh_scattering(cos_theta);
        let mie = self.mie_scattering(cos_theta);
        rayleigh + mie
    }

    /// Calculate sky color at zenith
    pub fn sky_color_zenith(&self) -> Vec3 {
        // Simplified calculation
        self.config.rayleigh_coefficient * self.config.sun_intensity
    }

    /// Calculate sky color at horizon
    pub fn sky_color_horizon(&self) -> Vec3 {
        // More scattering at horizon due to longer path length
        let path_length = 10.0;
        let rayleigh = self.config.rayleigh_coefficient * path_length * self.config.sun_intensity;
        let mie =
            Vec3::splat(self.config.mie_coefficient * path_length * self.config.sun_intensity);
        rayleigh + mie
    }

    /// Get configuration
    pub fn config(&self) -> &LightScatteringConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LightScatteringConfig) {
        self.config = config;
    }
}

impl Default for AtmosphericScattering {
    fn default() -> Self {
        Self::new(LightScatteringConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atmospheric_scattering_default() {
        let scattering = AtmosphericScattering::default();
        let config = scattering.config();
        assert!(config.enabled);
        assert_eq!(config.mie_anisotropy, 0.758);
    }

    #[test]
    fn test_rayleigh_scattering() {
        let scattering = AtmosphericScattering::default();
        let cos_theta = 1.0;
        let rayleigh = scattering.rayleigh_scattering(cos_theta);
        assert!(rayleigh.x > 0.0);
        assert!(rayleigh.y > 0.0);
        assert!(rayleigh.z > 0.0);
    }

    #[test]
    fn test_mie_scattering() {
        let scattering = AtmosphericScattering::default();
        let cos_theta = 1.0;
        let mie = scattering.mie_scattering(cos_theta);
        assert!(mie.x > 0.0);
    }

    #[test]
    fn test_total_scattering() {
        let scattering = AtmosphericScattering::default();
        let cos_theta = 0.5;
        let total = scattering.total_scattering(cos_theta);
        assert!(total.x > 0.0);
    }
}
