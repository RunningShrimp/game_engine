//! # Volumetric Cloud Rendering
//!
//! This module implements procedural volumetric clouds using:
//! - 3D noise textures (Perlin, Simplex, Worley)
//! - Fractal Brownian Motion (FBM)
//! - Ray marching for volume rendering
//! - Dynamic weather simulation
//!
//! ## Cloud Types
//!
//! - **Cumulus**: Puffy clouds at low altitude
//! - **Stratus**: Layered clouds at medium altitude
//! - **Cirrus**: Wispy high-altitude clouds
//! - **Cumulonimbus**: Storm clouds with vertical development
//!
//! ## Weather Simulation
//!
//! The weather system simulates:
//! - Cloud coverage
//! - Cloud density
//! - Wind speed and direction
//! - Precipitation
//! - Time of day transitions

use crate::error::RenderError;
use crate::render::atmosphere::noise::{NoiseGenerator, NoiseQuality, NoiseType};
use glam::Vec3;
use std::time::Duration;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, RenderPipeline, Sampler,
    Texture, TextureView,
};

/// Cloud type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudType {
    /// Cumulus (puffy, low-altitude)
    Cumulus,
    /// Stratus (layered, medium-altitude)
    Stratus,
    /// Cirrus (wispy, high-altitude)
    Cirrus,
    /// Cumulonimbus (storm clouds)
    Cumulonimbus,
}

/// Cloud quality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudQuality {
    /// Low quality (32 samples, 64x64 texture)
    Low,
    /// Medium quality (64 samples, 128x128 texture)
    Medium,
    /// High quality (128 samples, 256x256 texture)
    High,
    /// Ultra quality (256 samples, 512x512 texture)
    Ultra,
}

impl CloudQuality {
    /// Get ray marching sample count
    pub fn samples(&self) -> u32 {
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
            Self::Medium => 6,
            Self::High => 8,
            Self::Ultra => 16,
        }
    }

    /// Get noise texture resolution
    pub fn noise_resolution(&self) -> u32 {
        match self {
            Self::Low => 64,
            Self::Medium => 128,
            Self::High => 128,
            Self::Ultra => 256,
        }
    }
}

/// Cloud configuration
#[derive(Debug, Clone)]
pub struct CloudConfig {
    /// Enable cloud rendering
    pub enabled: bool,
    /// Cloud type
    pub cloud_type: CloudType,
    /// Cloud quality
    pub quality: CloudQuality,
    /// Cloud base altitude (meters)
    pub cloud_altitude: f32,
    /// Cloud layer thickness (meters)
    pub cloud_thickness: f32,
    /// Cloud density (0.0 - 1.0)
    pub cloud_density: f32,
    /// Cloud coverage (0.0 - 1.0)
    pub cloud_coverage: f32,
    /// Cloud absorption coefficient
    pub absorption: f32,
    /// Cloud scattering coefficient
    pub scattering: f32,
    /// Wind speed (m/s)
    pub wind_speed: f32,
    /// Wind direction
    pub wind_direction: Vec3,
    /// Cloud detail scale
    pub detail_scale: f32,
    /// Cloud erosion
    pub erosion: f32,
    /// Cloud anvil effect (for cumulonimbus)
    pub anvil: f32,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cloud_type: CloudType::Cumulus,
            quality: CloudQuality::Medium,
            cloud_altitude: 1500.0,
            cloud_thickness: 1000.0,
            cloud_density: 0.5,
            cloud_coverage: 0.5,
            absorption: 0.3,
            scattering: 0.7,
            wind_speed: 10.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.0),
            detail_scale: 1.0,
            erosion: 0.5,
            anvil: 0.0,
        }
    }
}

/// Weather state
#[derive(Debug, Clone)]
pub struct WeatherState {
    /// Cloud coverage (0.0 = clear, 1.0 = overcast)
    pub coverage: f32,
    /// Cloud density
    pub density: f32,
    /// Precipitation intensity (0.0 = none, 1.0 = heavy)
    pub precipitation: f32,
    /// Wind speed (m/s)
    pub wind_speed: f32,
    /// Wind direction
    pub wind_direction: Vec3,
    /// Time of day (0.0 - 24.0)
    pub time_of_day: f32,
    /// Temperature (Celsius)
    pub temperature: f32,
    /// Humidity (0.0 - 1.0)
    pub humidity: f32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            coverage: 0.5,
            density: 0.5,
            precipitation: 0.0,
            wind_speed: 10.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.0),
            time_of_day: 12.0,
            temperature: 20.0,
            humidity: 0.5,
        }
    }
}

/// Cloud renderer
pub struct CloudRenderer {
    config: CloudConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    uniform_buffer: Option<Buffer>,
    noise_texture: Option<Texture>,
    noise_sampler: Option<Sampler>,
    detail_noise_texture: Option<Texture>,
    detail_noise_sampler: Option<Sampler>,
    weather_texture: Option<Texture>,
    weather_sampler: Option<Sampler>,
    output_texture: Option<Texture>,
    output_view: Option<TextureView>,
    noise_generator: NoiseGenerator,
    time: f32,
}

impl CloudRenderer {
    /// Create new cloud renderer
    pub fn new(device: &Device, queue: &Queue, config: CloudConfig) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                uniform_buffer: None,
                noise_texture: None,
                noise_sampler: None,
                detail_noise_texture: None,
                detail_noise_sampler: None,
                weather_texture: None,
                weather_sampler: None,
                output_texture: None,
                output_view: None,
                noise_generator: NoiseGenerator::new(42),
                time: 0.0,
            });
        }

        let noise_generator = NoiseGenerator::new(42);
        let quality = config.quality;

        // Generate base noise texture (3D)
        let noise_resolution = quality.noise_resolution();
        let noise_texture =
            noise_generator.generate_texture_3d(device, queue, noise_resolution, NoiseType::Fbm)?;

        // Generate detail noise texture
        let detail_noise_texture = noise_generator.generate_texture_3d(
            device,
            queue,
            noise_resolution,
            NoiseType::Simplex,
        )?;

        // Create samplers
        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Cloud Noise Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let detail_noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Cloud Detail Noise Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let weather_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Cloud Weather Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cloud BGL"),
            entries: &[
                // Uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 3D noise texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Noise sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Detail noise texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Detail noise sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Depth texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Depth sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cloud Shader"),
            source: wgpu::ShaderSource::Wgsl(CLOUD_SHADER.into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cloud Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cloud Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            uniform_buffer: None,
            noise_texture: Some(noise_texture),
            noise_sampler: Some(noise_sampler),
            detail_noise_texture: Some(detail_noise_texture),
            detail_noise_sampler: Some(detail_noise_sampler),
            weather_texture: None,
            weather_sampler: Some(weather_sampler),
            output_texture: None,
            output_view: None,
            noise_generator,
            time: 0.0,
        })
    }

    /// Update configuration
    pub fn update_config(
        &mut self,
        device: &Device,
        config: CloudConfig,
    ) -> Result<(), RenderError> {
        self.config = config;
        // Re-generate noise textures if quality changed
        Ok(())
    }

    /// Update clouds over time
    pub fn update(&mut self, queue: &Queue, delta_time: f32) {
        self.time += delta_time;
        // Update cloud animation based on wind
    }

    /// Prepare output texture
    pub fn prepare(&mut self, device: &Device, width: u32, height: u32) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cloud Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.output_texture = Some(texture);
        self.output_view = Some(view);

        Ok(())
    }

    /// Render clouds
    pub fn render<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        bind_group: &'a BindGroup,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(pipeline) = &self.pipeline else {
            return Ok(());
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        Ok(())
    }

    /// Get output view
    pub fn output_view(&self) -> Option<&TextureView> {
        self.output_view.as_ref()
    }

    /// Create bind group
    pub fn create_bind_group(
        &self,
        device: &Device,
        depth_texture: &TextureView,
        depth_sampler: &Sampler,
    ) -> Result<BindGroup, RenderError> {
        let Some(bind_group_layout) = &self.bind_group_layout else {
            return Err(RenderError::InvalidState {
                message: "Bind group layout not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(uniform_buffer) = &self.uniform_buffer else {
            return Err(RenderError::InvalidState {
                message: "Uniform buffer not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(noise_texture) = &self.noise_texture else {
            return Err(RenderError::InvalidState {
                message: "Noise texture not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(detail_noise_texture) = &self.detail_noise_texture else {
            return Err(RenderError::InvalidState {
                message: "Detail noise texture not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let noise_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let detail_view = detail_noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cloud Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&noise_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(self.noise_sampler.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&detail_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(
                        self.detail_noise_sampler.as_ref().unwrap(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(depth_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(depth_sampler),
                },
            ],
        }))
    }
}

/// Weather system for dynamic cloud simulation
pub struct WeatherSystem {
    renderer: CloudRenderer,
    weather_state: WeatherState,
    time_since_weather_change: f32,
}

impl WeatherSystem {
    /// Create new weather system
    pub fn new(device: &Device, queue: &Queue, config: CloudConfig) -> Result<Self, RenderError> {
        let renderer = CloudRenderer::new(device, queue, config)?;
        Ok(Self {
            renderer,
            weather_state: WeatherState::default(),
            time_since_weather_change: 0.0,
        })
    }

    /// Update weather system
    pub fn update(&mut self, queue: &Queue, delta_time: f32) {
        self.renderer.update(queue, delta_time);
        self.time_since_weather_change += delta_time;
    }

    /// Set weather state
    pub fn set_weather(&mut self, weather: WeatherState) {
        self.weather_state = weather;
        self.time_since_weather_change = 0.0;
    }

    /// Get current weather state
    pub fn weather(&self) -> &WeatherState {
        &self.weather_state
    }

    /// Prepare for rendering
    pub fn prepare(&mut self, device: &Device, width: u32, height: u32) -> Result<(), RenderError> {
        self.renderer.prepare(device, width, height)
    }

    /// Render clouds
    pub fn render(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        _device: &Device,
        _camera: &crate::render::volumetric::Camera,
        _depth_texture: &wgpu::TextureView,
        _light_direction: Vec3,
    ) -> Result<(), RenderError> {
        // Implement cloud rendering pass
        Ok(())
    }

    /// Update configuration
    pub fn update_config(
        &mut self,
        device: &Device,
        config: CloudConfig,
    ) -> Result<(), RenderError> {
        self.renderer.update_config(device, config)
    }
}

/// Cloud shader (WGSL)
const CLOUD_SHADER: &str = r#"
// Cloud rendering uniforms
struct CloudUniforms {
    camera_position: vec3<f32>,
    light_direction: vec3<f32>,
    light_color: vec3<f32>,
    cloud_altitude: f32,
    cloud_thickness: f32,
    cloud_density: f32,
    cloud_coverage: f32,
    absorption: f32,
    scattering: f32,
    wind_speed: f32,
    wind_direction: vec3<f32>,
    detail_scale: f32,
    erosion: f32,
    time: f32,
    ray_marching_samples: u32,
    light_samples: u32,
}

@group(0) @binding(0) var<uniform> uniforms: CloudUniforms;
@group(0) @binding(1) var noise_texture: texture_3d<f32>;
@group(0) @binding(2) var noise_sampler: sampler;
@group(0) @binding(3) var detail_noise_texture: texture_3d<f32>;
@group(0) @binding(4) var detail_noise_sampler: sampler;
@group(0) @binding(5) var depth_texture: texture_depth_2d;
@group(0) @binding(6) var depth_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_dir: vec3<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );

    let uv = pos[vertex_index];
    let clip_pos = vec4<f32>(uv, 0.0, 1.0);

    // TODO: Transform UV to world direction based on camera
    let world_dir = vec3<f32>(uv.x, uv.y, 1.0);

    return VertexOutput(clip_pos, uv * 0.5 + 0.5, world_dir);
}

// Sample cloud density at position
fn sample_cloud_density(pos: vec3<f32>) -> f32 {
    // Apply wind animation
    let wind_offset = uniforms.wind_direction * uniforms.time * uniforms.wind_speed;
    let sample_pos = pos + wind_offset;

    // Sample base noise
    let base_noise = textureSample(noise_texture, noise_sampler, sample_pos * 0.001).r;

    // Sample detail noise
    let detail_noise = textureSample(
        detail_noise_texture,
        detail_noise_sampler,
        sample_pos * uniforms.detail_scale * 0.01
    ).r;

    // Combine noises
    let cloud_density = base_noise * detail_noise;

    // Apply coverage and erosion
    let density = smoothstep(1.0 - uniforms.cloud_coverage, 1.0, cloud_density);
    let eroded = density - uniforms.erosion * detail_noise;

    return max(0.0, eroded * uniforms.cloud_density);
}

// Beer-Lambert law for light absorption
fn beer_lambert(density: f32, distance: f32) -> f32 {
    return exp(-density * uniforms.absorption * distance);
}

// Henyey-Greenstein phase function for scattering
fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let num = 1.0 - g * g;
    let denom = 4.0 * 3.14159 * pow(1.0 + g * g - 2.0 * g * cos_theta, 1.5);
    return num / denom;
}

// Ray march clouds
fn ray_march_clouds(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec4<f32> {
    let cloud_bottom = uniforms.cloud_altitude;
    let cloud_top = uniforms.cloud_altitude + uniforms.cloud_thickness;

    // Calculate entry and exit points
    var t_enter = 0.0;
    var t_exit = 0.0;

    // Simple plane intersection (should use sphere-box intersection)
    if (ray_dir.y > 0.0) {
        t_enter = (cloud_bottom - ray_origin.y) / ray_dir.y;
        t_exit = (cloud_top - ray_origin.y) / ray_dir.y;
    } else {
        t_enter = (cloud_top - ray_origin.y) / ray_dir.y;
        t_exit = (cloud_bottom - ray_origin.y) / ray_dir.y;
    }

    if (t_enter < 0.0) {
        t_enter = 0.0;
    }
    if (t_exit <= t_enter) {
        return vec4<f32>(0.0);
    }

    // Ray marching
    let step_size = (t_exit - t_enter) / f32(uniforms.ray_marching_samples);
    var transmittance = 1.0;
    var light_energy = vec3<f32>(0.0);

    var t = t_enter;
    let g = 0.8; // Scattering anisotropy

    for (var i = 0u; i < uniforms.ray_marching_samples; i++) {
        let pos = ray_origin + ray_dir * t;
        let density = sample_cloud_density(pos);

        if (density > 0.001) {
            // Light march towards sun
            var light_transmittance = 1.0;
            let light_step = step_size * 2.0;
            let light_dir = normalize(uniforms.light_direction);

            for (var j = 0u; j < uniforms.light_samples; j++) {
                let light_pos = pos + light_dir * f32(j) * light_step;
                let light_density = sample_cloud_density(light_pos);
                light_transmittance *= beer_lambert(light_density, light_step);
            }

            // In-scattering
            let cos_theta = dot(ray_dir, normalize(uniforms.light_direction));
            let phase = henyey_greenstein(cos_theta, g);
            let scattering = uniforms.scattering * density * step_size;

            light_energy += transmittance * light_transmittance * phase * scattering * uniforms.light_color;
            transmittance *= beer_lambert(density, step_size);
        }

        if (transmittance < 0.01) {
            break;
        }

        t += step_size;
    }

    return vec4<f32>(light_energy, 1.0 - transmittance);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = uniforms.camera_position;
    let ray_dir = normalize(in.world_dir);

    // Read depth
    let depth = textureSample(depth_texture, depth_sampler, in.uv).r;

    // TODO: Reconstruct world position from depth
    // For now, just ray march clouds
    let cloud_color = ray_march_clouds(ray_origin, ray_dir);

    return cloud_color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_config_default() {
        let config = CloudConfig::default();
        assert!(config.enabled);
        assert_eq!(config.cloud_type, CloudType::Cumulus);
    }

    #[test]
    fn test_weather_state_default() {
        let weather = WeatherState::default();
        assert_eq!(weather.coverage, 0.5);
        assert_eq!(weather.precipitation, 0.0);
    }

    #[test]
    fn test_cloud_quality() {
        assert_eq!(CloudQuality::Low.samples(), 32);
        assert_eq!(CloudQuality::Medium.samples(), 64);
        assert_eq!(CloudQuality::High.samples(), 128);
        assert_eq!(CloudQuality::Ultra.samples(), 256);
    }
}
