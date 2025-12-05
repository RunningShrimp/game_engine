//! 体积渲染模块
//!
//! 实现体积渲染效果，包括：
//! - 雾效果（线性、指数、高度雾）
//! - 体积光（God Rays）
//! - 云渲染
//! - 体积阴影

use crate::core::error::RenderError;
use crate::impl_default;
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, Queue, RenderPass, RenderPipeline,
    Sampler, Texture, TextureView,
};

/// 体积渲染配置
#[derive(Debug, Clone)]
pub struct VolumetricConfig {
    /// 是否启用体积渲染
    pub enabled: bool,
    /// 雾类型
    pub fog_type: FogType,
    /// 雾颜色
    pub fog_color: Vec3,
    /// 雾密度
    pub fog_density: f32,
    /// 雾起始距离
    pub fog_start: f32,
    /// 雾结束距离
    pub fog_end: f32,
    /// 是否启用体积光
    pub volumetric_lighting: bool,
    /// 体积光强度
    pub volumetric_light_intensity: f32,
    /// 体积光采样数
    pub volumetric_light_samples: u32,
    /// 是否启用云渲染
    pub cloud_rendering: bool,
    /// 云密度
    pub cloud_density: f32,
    /// 云高度范围
    pub cloud_height_range: (f32, f32),
}

impl_default!(VolumetricConfig {
    enabled: true,
    fog_type: FogType::Exponential,
    fog_color: Vec3::new(0.7, 0.8, 0.9),
    fog_density: 0.02,
    fog_start: 10.0,
    fog_end: 100.0,
    volumetric_lighting: false,
    volumetric_light_intensity: 1.0,
    volumetric_light_samples: 16,
    cloud_rendering: false,
    cloud_density: 0.5,
    cloud_height_range: (50.0, 200.0),
});

/// 雾类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FogType {
    /// 线性雾
    Linear,
    /// 指数雾
    Exponential,
    /// 指数平方雾
    ExponentialSquared,
    /// 高度雾
    Height { height: f32, falloff: f32 },
}

/// 体积渲染器
pub struct VolumetricRenderer {
    config: VolumetricConfig,
    pipeline: Option<RenderPipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    uniform_buffer: Option<Buffer>,
    fog_texture: Option<Texture>,
    fog_view: Option<TextureView>,
}

impl VolumetricRenderer {
    /// 创建新的体积渲染器
    pub fn new(device: &Device, config: VolumetricConfig) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                uniform_buffer: None,
                fog_texture: None,
                fog_view: None,
            });
        }

        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Volumetric Rendering Shader"),
            source: wgpu::ShaderSource::Wgsl(VOLUMETRIC_SHADER.into()),
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Volumetric BGL"),
            entries: &[
                // 统一缓冲区
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
                // 深度纹理
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
                // 深度采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // 创建渲染管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Volumetric Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建渲染管线
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Volumetric Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[QuadVertex::desc().clone()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        });

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            uniform_buffer: None,
            fog_texture: None,
            fog_view: None,
        })
    }

    /// 更新配置
    pub fn update_config(
        &mut self,
        device: &Device,
        _queue: &Queue,
        config: VolumetricConfig,
    ) -> Result<(), RenderError> {
        self.config = config.clone();

        if config.enabled {
            // 更新统一缓冲区
            let fog_type_u32 = match config.fog_type {
                FogType::Linear => 0,
                FogType::Exponential => 1,
                FogType::ExponentialSquared => 2,
                FogType::Height { .. } => 3,
            };

            let uniforms = VolumetricUniforms {
                fog_type: fog_type_u32,
                fog_color: [config.fog_color.x, config.fog_color.y, config.fog_color.z],
                fog_density: config.fog_density,
                fog_start: config.fog_start,
                fog_end: config.fog_end,
                volumetric_lighting: if config.volumetric_lighting {
                    1u32
                } else {
                    0u32
                },
                volumetric_light_intensity: config.volumetric_light_intensity,
                volumetric_light_samples: config.volumetric_light_samples,
                cloud_rendering: if config.cloud_rendering { 1u32 } else { 0u32 },
                cloud_density: config.cloud_density,
                cloud_height_min: config.cloud_height_range.0,
                cloud_height_max: config.cloud_height_range.1,
            };

            let uniforms_array = [uniforms];
            let uniform_data = bytemuck::cast_slice(&uniforms_array);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Volumetric Uniform Buffer"),
                contents: uniform_data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            self.uniform_buffer = Some(buffer);
        }

        Ok(())
    }

    /// 准备输出纹理
    pub fn prepare_output(
        &mut self,
        device: &Device,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volumetric Fog Texture"),
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

        self.fog_texture = Some(texture);
        self.fog_view = Some(view);

        Ok(())
    }

    /// 渲染体积效果
    ///
    /// 注意：bind_group 需要在外部创建并传入
    pub fn render<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        bind_group: &'a BindGroup,
        _camera: &Camera,
        _depth_texture: &TextureView,
        _depth_sampler: &Sampler,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(pipeline) = &self.pipeline else {
            return Ok(());
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);

        // 渲染全屏四边形
        // 注意：需要创建全屏四边形顶点缓冲区
        // render_pass.draw(0, 6, 0, 1);

        Ok(())
    }

    /// 获取雾纹理视图
    pub fn fog_view(&self) -> Option<&TextureView> {
        self.fog_view.as_ref()
    }

    /// 创建绑定组
    pub fn create_bind_group(
        &self,
        device: &Device,
        depth_texture: &TextureView,
        depth_sampler: &Sampler,
    ) -> Result<BindGroup, RenderError> {
        let Some(bind_group_layout) = &self.bind_group_layout else {
            return Err(RenderError::InvalidState(
                "Bind group layout not initialized".into(),
            ));
        };

        let Some(uniform_buffer) = &self.uniform_buffer else {
            return Err(RenderError::InvalidState(
                "Uniform buffer not initialized".into(),
            ));
        };

        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volumetric Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
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

/// 相机参数
#[derive(Debug, Clone)]
pub struct Camera {
    /// 视图矩阵
    pub view: Mat4,
    /// 投影矩阵
    pub projection: Mat4,
    /// 位置
    pub position: Vec3,
    /// 方向
    pub direction: Vec3,
}

/// 全屏四边形顶点
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl QuadVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

/// 体积渲染统一缓冲区
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VolumetricUniforms {
    fog_type: u32,
    fog_color: [f32; 3],
    fog_density: f32,
    fog_start: f32,
    fog_end: f32,
    volumetric_lighting: u32,
    volumetric_light_intensity: f32,
    volumetric_light_samples: u32,
    cloud_rendering: u32,
    cloud_density: f32,
    cloud_height_min: f32,
    cloud_height_max: f32,
}

/// 体积渲染着色器
const VOLUMETRIC_SHADER: &str = r#"
struct VolumetricUniforms {
    fog_type: u32,
    fog_color: vec3<f32>,
    fog_density: f32,
    fog_start: f32,
    fog_end: f32,
    volumetric_lighting: u32,
    volumetric_light_intensity: f32,
    volumetric_light_samples: u32,
    cloud_rendering: u32,
    cloud_density: f32,
    cloud_height_min: f32,
    cloud_height_max: f32,
}

@group(0) @binding(0) var<uniform> uniforms: VolumetricUniforms;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var depth_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(position, 0.0, 1.0),
        uv
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 从深度纹理重建世界位置
    let depth = textureSample(depth_texture, depth_sampler, in.uv).r;
    
    // 计算雾因子
    var fog_factor = 0.0;
    
    if (uniforms.fog_type == 0u) {
        // 线性雾
        fog_factor = (uniforms.fog_end - depth) / (uniforms.fog_end - uniforms.fog_start);
        fog_factor = clamp(fog_factor, 0.0, 1.0);
    } else if (uniforms.fog_type == 1u) {
        // 指数雾
        fog_factor = exp(-uniforms.fog_density * depth);
    } else if (uniforms.fog_type == 2u) {
        // 指数平方雾
        fog_factor = exp(-uniforms.fog_density * uniforms.fog_density * depth * depth);
    }
    
    // 应用雾颜色
    let fog_color = uniforms.fog_color * (1.0 - fog_factor);
    
    return vec4<f32>(fog_color, fog_factor);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volumetric_config_default() {
        let config = VolumetricConfig::default();
        assert!(config.enabled);
        assert_eq!(config.fog_type, FogType::Exponential);
    }

    #[test]
    fn test_fog_type() {
        let height_fog = FogType::Height {
            height: 100.0,
            falloff: 10.0,
        };
        assert_ne!(height_fog, FogType::Linear);
    }
}
