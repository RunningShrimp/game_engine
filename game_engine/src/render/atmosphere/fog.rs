//! # Fog Effects System
//!
//! This module implements various fog effects:
//! - Volumetric fog (height-based)
//! - Exponential distance fog
//! - Layered fog
//! - Ground fog
//! - Atmospheric scattering

use crate::error::RenderError;
use glam::Vec3;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Device, RenderPass, RenderPipeline, Sampler, Texture,
    TextureView,
};

/// Fog type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogType {
    /// Linear distance fog
    Linear,
    /// Exponential distance fog
    Exponential,
    /// Exponential squared distance fog
    ExponentialSquared,
    /// Height-based volumetric fog
    Height,
    /// Layered fog
    Layered,
    /// Ground fog
    Ground,
}

/// Fog quality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogQuality {
    /// Low quality (16 samples, quarter resolution)
    Low,
    /// Medium quality (32 samples, half resolution)
    Medium,
    /// High quality (64 samples, full resolution)
    High,
    /// Ultra quality (128 samples, full resolution)
    Ultra,
}

impl FogQuality {
    /// Get ray marching sample count
    pub fn samples(&self) -> u32 {
        match self {
            Self::Low => 16,
            Self::Medium => 32,
            Self::High => 64,
            Self::Ultra => 128,
        }
    }

    /// Get resolution scale
    pub fn resolution_scale(&self) -> f32 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.5,
            Self::High => 1.0,
            Self::Ultra => 1.0,
        }
    }
}

/// Height fog configuration
#[derive(Debug, Clone)]
pub struct HeightFogConfig {
    /// Fog height (meters)
    pub height: f32,
    /// Fog density at height
    pub density: f32,
    /// Fog falloff (how quickly density decreases with height)
    pub falloff: f32,
}

impl Default for HeightFogConfig {
    fn default() -> Self {
        Self {
            height: 0.0,
            density: 0.01,
            falloff: 0.1,
        }
    }
}

/// Ground fog configuration
#[derive(Debug, Clone)]
pub struct GroundFogConfig {
    /// Ground height
    pub ground_height: f32,
    /// Maximum fog height
    pub max_height: f32,
    /// Fog density
    pub density: f32,
}

impl Default for GroundFogConfig {
    fn default() -> Self {
        Self {
            ground_height: 0.0,
            max_height: 10.0,
            density: 0.02,
        }
    }
}

/// Volumetric fog configuration
#[derive(Debug, Clone)]
pub struct VolumetricFogConfig {
    /// Enable volumetric fog
    pub enabled: bool,
    /// Fog quality
    pub quality: FogQuality,
    /// Fog color
    pub color: Vec3,
    /// Fog density
    pub density: f32,
    /// Light scattering coefficient
    pub scattering: f32,
    /// Light absorption coefficient
    pub absorption: f32,
    /// Anisotropy (-1 to 1)
    pub anisotropy: f32,
    /// Enable light shafts
    pub light_shafts: bool,
    /// Light shaft intensity
    pub light_shaft_intensity: f32,
}

impl Default for VolumetricFogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            quality: FogQuality::Medium,
            color: Vec3::new(0.7, 0.8, 0.9),
            density: 0.01,
            scattering: 0.5,
            absorption: 0.1,
            anisotropy: 0.6,
            light_shafts: false,
            light_shaft_intensity: 0.3,
        }
    }
}

/// Fog configuration
#[derive(Debug, Clone)]
pub struct FogConfig {
    /// Enable fog
    pub enabled: bool,
    /// Fog type
    pub fog_type: FogType,
    /// Fog quality
    pub quality: FogQuality,
    /// Fog color
    pub color: Vec3,
    /// Fog density
    pub density: f32,
    /// Start distance
    pub start_distance: f32,
    /// End distance
    pub end_distance: f32,
    /// Height fog configuration
    pub height_fog: Option<HeightFogConfig>,
    /// Ground fog configuration
    pub ground_fog: Option<GroundFogConfig>,
    /// Volumetric fog configuration
    pub volumetric: VolumetricFogConfig,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fog_type: FogType::Exponential,
            quality: FogQuality::Medium,
            color: Vec3::new(0.7, 0.8, 0.9),
            density: 0.01,
            start_distance: 10.0,
            end_distance: 100.0,
            height_fog: Some(HeightFogConfig::default()),
            ground_fog: None,
            volumetric: VolumetricFogConfig::default(),
        }
    }
}

/// Fog renderer
pub struct FogRenderer {
    config: FogConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    uniform_buffer: Option<Buffer>,
    volumetric_pipeline: Option<RenderPipeline>,
    volumetric_bind_group_layout: Option<BindGroupLayout>,
    volumetric_uniform_buffer: Option<Buffer>,
    output_texture: Option<Texture>,
    output_view: Option<TextureView>,
    volumetric_output_texture: Option<Texture>,
    volumetric_output_view: Option<TextureView>,
}

impl FogRenderer {
    /// Create new fog renderer
    pub fn new(device: &Device, config: FogConfig) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                uniform_buffer: None,
                volumetric_pipeline: None,
                volumetric_bind_group_layout: None,
                volumetric_uniform_buffer: None,
                output_texture: None,
                output_view: None,
                volumetric_output_texture: None,
                volumetric_output_view: None,
            });
        }

        // Create basic fog pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fog BGL"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let fog_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fog Shader"),
            source: wgpu::ShaderSource::Wgsl(FOG_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fog Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fog Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fog_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fog_shader,
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

        // Create volumetric fog pipeline if enabled
        let (volumetric_pipeline, volumetric_bind_group_layout) = if config.volumetric.enabled {
            let v_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Volumetric Fog BGL"),
                entries: &[
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

            let v_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Volumetric Fog Shader"),
                source: wgpu::ShaderSource::Wgsl(VOLUMETRIC_FOG_SHADER.into()),
            });

            let v_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Volumetric Fog Pipeline Layout"),
                bind_group_layouts: &[&v_bgl],
                push_constant_ranges: &[],
            });

            let v_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Volumetric Fog Pipeline"),
                layout: Some(&v_layout),
                vertex: wgpu::VertexState {
                    module: &v_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &v_shader,
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

            (Some(v_pipeline), Some(v_bgl))
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            uniform_buffer: None,
            volumetric_pipeline,
            volumetric_bind_group_layout,
            volumetric_uniform_buffer: None,
            output_texture: None,
            output_view: None,
            volumetric_output_texture: None,
            volumetric_output_view: None,
        })
    }

    /// Update configuration
    pub fn update_config(&mut self, device: &Device, config: FogConfig) -> Result<(), RenderError> {
        self.config = config;
        Ok(())
    }

    /// Prepare output textures
    pub fn prepare(&mut self, device: &Device, width: u32, height: u32) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let scale = self.config.quality.resolution_scale();
        let fog_width = (width as f32 * scale) as u32;
        let fog_height = (height as f32 * scale) as u32;

        // Basic fog texture
        let fog_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fog Output Texture"),
            size: wgpu::Extent3d {
                width: fog_width,
                height: fog_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let fog_view = fog_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.output_texture = Some(fog_texture);
        self.output_view = Some(fog_view);

        // Volumetric fog texture
        if self.config.volumetric.enabled {
            let vol_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Volumetric Fog Output Texture"),
                size: wgpu::Extent3d {
                    width: fog_width,
                    height: fog_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let vol_view = vol_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.volumetric_output_texture = Some(vol_texture);
            self.volumetric_output_view = Some(vol_view);
        }

        Ok(())
    }

    /// Render fog
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &Device,
        _camera: &crate::render::volumetric::Camera,
        depth_texture: &TextureView,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Render basic fog
        if let (Some(pipeline), Some(output_view)) = (&self.pipeline, &self.output_view) {
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

            // Create bind group (would be cached in production)
            let bind_group = self.create_bind_group(device, depth_texture, &sampler)?;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Fog Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None, // Required for wgpu 27+
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);

            drop(render_pass);
        }

        Ok(())
    }

    /// Get output view
    pub fn output_view(&self) -> Option<&TextureView> {
        self.output_view.as_ref()
    }

    /// Get volumetric output view
    pub fn volumetric_output_view(&self) -> Option<&TextureView> {
        self.volumetric_output_view.as_ref()
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

        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fog Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.uniform_buffer.as_ref().ok_or(RenderError::InvalidState {
                            message: "Uniform buffer not initialized".into(),
                            severity: crate::error::ErrorSeverity::Error,
                        })?,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(depth_sampler),
                },
            ],
        }))
    }
}

/// Basic fog shader
const FOG_SHADER: &str = r#"
struct FogUniforms {
    fog_type: u32,
    fog_color: vec3<f32>,
    fog_density: f32,
    fog_start: f32,
    fog_end: f32,
    camera_position: vec3<f32>,
    height: f32,
    height_falloff: f32,
}

@group(0) @binding(0) var<uniform> uniforms: FogUniforms;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var depth_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

    return VertexOutput(clip_pos, uv * 0.5 + 0.5);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_texture, depth_sampler, in.uv).r;

    // Simple fog based on depth
    var fog_factor = 0.0;

    if (uniforms.fog_type == 0u) {
        // Linear fog
        fog_factor = (uniforms.fog_end - depth) / (uniforms.fog_end - uniforms.fog_start);
    } else if (uniforms.fog_type == 1u) {
        // Exponential fog
        fog_factor = exp(-uniforms.fog_density * depth);
    } else if (uniforms.fog_type == 2u) {
        // Exponential squared fog
        fog_factor = exp(-uniforms.fog_density * uniforms.fog_density * depth * depth);
    }

    fog_factor = clamp(fog_factor, 0.0, 1.0);

    let fog_color = uniforms.fog_color * (1.0 - fog_factor);

    return vec4<f32>(fog_color, 1.0 - fog_factor);
}
"#;

/// Volumetric fog shader
const VOLUMETRIC_FOG_SHADER: &str = r#"
struct VolumetricFogUniforms {
    fog_color: vec3<f32>,
    fog_density: f32,
    scattering: f32,
    absorption: f32,
    anisotropy: f32,
    camera_position: vec3<f32>,
    light_direction: vec3<f32>,
    light_color: vec3<f32>,
    light_shaft_intensity: f32,
    samples: u32,
}

@group(0) @binding(0) var<uniform> uniforms: VolumetricFogUniforms;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var depth_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

    return VertexOutput(clip_pos, uv * 0.5 + 0.5);
}

// Henyey-Greenstein phase function
fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let num = 1.0 - g * g;
    let denom = 4.0 * 3.14159 * pow(1.0 + g * g - 2.0 * g * cos_theta, 1.5);
    return num / denom;
}

// Sample fog density
fn sample_fog_density(position: vec3<f32>) -> f32 {
    return uniforms.fog_density;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_texture, depth_sampler, in.uv).r;

    // Reconstruct world position (simplified)
    let ray_dir = vec3<f32>(0.0, 0.0, 1.0); // Should be reconstructed from UV
    let ray_origin = uniforms.camera_position;

    let max_dist = depth * 100.0;
    let step_size = max_dist / f32(uniforms.samples);

    var transmittance = 1.0;
    var scattered_light = vec3<f32>(0.0);

    var t = 0.0;
    for (var i = 0u; i < uniforms.samples; i++) {
        let pos = ray_origin + ray_dir * t;
        let density = sample_fog_density(pos);

        if (density > 0.001) {
            // Light integration towards light source
            let light_dir = normalize(uniforms.light_direction);
            let cos_theta = dot(ray_dir, light_dir);
            let phase = henyey_greenstein(cos_theta, uniforms.anisotropy);

            // Scattering
            let scattering_coeff = uniforms.scattering * density * step_size;
            scattered_light += transmittance * phase * scattering_coeff * uniforms.light_color;
            transmittance *= exp(-density * uniforms.absorption * step_size);
        }

        if (transmittance < 0.01) {
            break;
        }

        t += step_size;
    }

    let final_color = uniforms.fog_color * scattered_light + uniforms.fog_color * (1.0 - transmittance);

    return vec4<f32>(final_color, 1.0 - transmittance);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fog_config_default() {
        let config = FogConfig::default();
        assert!(config.enabled);
        assert_eq!(config.fog_type, FogType::Exponential);
    }

    #[test]
    fn test_height_fog_config_default() {
        let config = HeightFogConfig::default();
        assert_eq!(config.height, 0.0);
        assert_eq!(config.density, 0.01);
    }

    #[test]
    fn test_fog_quality() {
        assert_eq!(FogQuality::Low.samples(), 16);
        assert_eq!(FogQuality::Medium.samples(), 32);
        assert_eq!(FogQuality::High.samples(), 64);
    }
}
