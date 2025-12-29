//! 延迟渲染管线
//!
//! 实现延迟渲染（Deferred Rendering）管线，包括：
//! - G-Buffer生成（几何阶段）
//! - 延迟光照计算（光照阶段）
//! - 支持多光源和CSM阴影
//!
//! ## 延迟渲染流程
//!
//! 1. **几何阶段（Geometry Pass）**: 渲染场景几何到G-Buffer
//!    - 位置 + 深度
//!    - 法线 + 粗糙度
//!    - 反照率 + 金属度
//!
//! 2. **光照阶段（Lighting Pass）**: 从G-Buffer读取数据，计算光照
//!    - 读取G-Buffer数据
//!    - 应用PBR光照模型
//!    - 支持方向光、点光源、CSM阴影
//!
//! ## 优势
//!
//! - 支持大量动态光源（不受几何复杂度影响）
//! - 光照计算与几何分离，便于优化
//! - 适合复杂光照场景

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
// Vec4 未在此文件中使用，但可能在未来需要
// use glam::Vec4;
use bytemuck::{Pod, Zeroable};

/// G-Buffer纹理
pub struct GBuffer {
    /// 位置 + 深度 (RGB = 世界坐标, A = 深度)
    pub position_texture: wgpu::Texture,
    pub position_view: wgpu::TextureView,

    /// 法线 + 粗糙度 (RGB = 法线, A = 粗糙度)
    pub normal_texture: wgpu::Texture,
    pub normal_view: wgpu::TextureView,

    /// 反照率 + 金属度 (RGB = 反照率, A = 金属度)
    pub albedo_texture: wgpu::Texture,
    pub albedo_view: wgpu::TextureView,

    /// 深度缓冲
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    /// G-Buffer绑定组
    pub bind_group: wgpu::BindGroup,
}

impl GBuffer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // 位置纹理 (RGBA32Float)
        let position_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Position"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let position_view = position_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 法线纹理 (RGBA16Float)
        let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Normal"),
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
        let normal_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 反照率纹理 (RGBA8UnormSrgb)
        let albedo_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Albedo"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let albedo_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 深度纹理
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G-Buffer Depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("G-Buffer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("G-Buffer Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&position_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            position_texture,
            position_view,
            normal_texture,
            normal_view,
            albedo_texture,
            albedo_view,
            depth_texture,
            depth_view,
            bind_group,
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        *self = Self::new(device, width, height, bind_group_layout);
    }
}

/// 延迟渲染器
pub struct DeferredRenderer {
    pub gbuffer: GBuffer,
    pub geometry_pipeline: wgpu::RenderPipeline,
    pub lighting_pipeline: wgpu::RenderPipeline,
    pub gbuffer_bind_group_layout: wgpu::BindGroupLayout,
    pub fullscreen_vertex_buffer: wgpu::Buffer,
}

impl DeferredRenderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // 创建G-Buffer绑定组布局
        let gbuffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("G-Buffer BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        // 创建G-Buffer
        let gbuffer = GBuffer::new(device, width, height, &gbuffer_bind_group_layout);

        // 创建几何阶段着色器
        let geometry_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Geometry Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_deferred_geometry.wgsl").into()),
        });

        // 创建光照阶段着色器
        let lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Lighting Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_deferred_lighting.wgsl").into()),
        });

        // 创建几何阶段管线 (写入G-Buffer)
        let geometry_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Deferred Geometry Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Geometry Pipeline"),
            layout: Some(&geometry_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &geometry_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::render::mesh::Vertex3D>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &geometry_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float, // Position
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float, // Normal
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb, // Albedo
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            cache: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 创建光照阶段管线 (读取G-Buffer,输出到屏幕)
        // 注意: 这里假设CSM绑定组布局已经在其他地方定义
        // 实际使用时需要传入CSM绑定组布局
        let lighting_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Deferred Lighting Pipeline Layout"),
                bind_group_layouts: &[&gbuffer_bind_group_layout],
                push_constant_ranges: &[],
            });

        let lighting_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Lighting Pipeline"),
            layout: Some(&lighting_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &lighting_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &lighting_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            cache: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 创建全屏四边形顶点缓冲
        let fullscreen_vertices: &[[f32; 2]] = &[
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.0],
            [-1.0, -1.0],
            [1.0, 1.0],
            [-1.0, 1.0],
        ];

        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(fullscreen_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Self {
            gbuffer,
            geometry_pipeline,
            lighting_pipeline,
            gbuffer_bind_group_layout,
            fullscreen_vertex_buffer,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.gbuffer.resize(device, width, height, &self.gbuffer_bind_group_layout);
    }
}

// ============================================================================
// 统一缓冲区结构
// ============================================================================

/// 相机统一缓冲区
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    /// 视图投影矩阵
    pub view_proj: [[f32; 4]; 4],
    /// 相机位置
    pub position: [f32; 3],
    /// 填充对齐
    pub _pad1: f32,
    /// 视图矩阵
    pub view: [[f32; 4]; 4],
    /// 投影矩阵
    pub projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(view: Mat4, proj: Mat4, position: Vec3) -> Self {
        Self {
            view_proj: (proj * view).to_cols_array_2d(),
            position: position.to_array(),
            _pad1: 0.0,
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
        }
    }
}

/// 光照统一缓冲区
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightingUniform {
    /// 方向光方向
    pub light_direction: [f32; 3],
    /// 方向光强度
    pub light_intensity: f32,
    /// 方向光颜色
    pub light_color: [f32; 3],
    /// 环境光强度
    pub ambient_intensity: f32,
    /// 点光源数量
    pub point_light_count: u32,
    /// 填充对齐
    pub _pad: [u32; 3],
}

impl Default for LightingUniform {
    fn default() -> Self {
        Self {
            light_direction: [0.0, -1.0, 0.0],
            light_intensity: 1.0,
            light_color: [1.0, 1.0, 1.0],
            ambient_intensity: 0.03,
            point_light_count: 0,
            _pad: [0; 3],
        }
    }
}

/// 延迟渲染配置
#[derive(Debug, Clone)]
pub struct DeferredConfig {
    /// 启用延迟渲染
    pub enabled: bool,
    /// 启用CSM阴影
    pub enable_csm: bool,
    /// 最大点光源数量
    pub max_point_lights: u32,
    /// 启用SSAO
    pub enable_ssao: bool,
}

impl Default for DeferredConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_csm: true,
            max_point_lights: 64,
            enable_ssao: false,
        }
    }
}

/// 延迟渲染器（增强版）
pub struct DeferredRendererEnhanced {
    /// 基础延迟渲染器
    pub renderer: DeferredRenderer,
    /// 相机统一缓冲区
    pub camera_uniform: wgpu::Buffer,
    /// 相机绑定组
    pub camera_bind_group: wgpu::BindGroup,
    /// 相机绑定组布局
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    /// 光照统一缓冲区
    pub lighting_uniform: wgpu::Buffer,
    /// 光照绑定组
    pub lighting_bind_group: wgpu::BindGroup,
    /// 光照绑定组布局
    pub lighting_bind_group_layout: wgpu::BindGroupLayout,
    /// 配置
    pub config: DeferredConfig,
}

impl DeferredRendererEnhanced {
    /// 创建增强的延迟渲染器
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let renderer = DeferredRenderer::new(device, width, height, surface_format);

        // 创建相机统一缓冲区
        let camera_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Deferred Camera Uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建相机绑定组布局
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Deferred Camera BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<CameraUniform>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        // 创建相机绑定组
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Deferred Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform.as_entire_binding(),
            }],
        });

        // 创建光照统一缓冲区
        let lighting_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Deferred Lighting Uniform"),
            size: std::mem::size_of::<LightingUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建光照绑定组布局
        let lighting_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Deferred Lighting BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                            LightingUniform,
                        >()
                            as u64),
                    },
                    count: None,
                }],
            });

        // 创建光照绑定组
        let lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Deferred Lighting Bind Group"),
            layout: &lighting_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lighting_uniform.as_entire_binding(),
            }],
        });

        Self {
            renderer,
            camera_uniform,
            camera_bind_group,
            camera_bind_group_layout,
            lighting_uniform,
            lighting_bind_group,
            lighting_bind_group_layout,
            config: DeferredConfig::default(),
        }
    }

    /// 更新相机参数
    pub fn update_camera(&self, queue: &wgpu::Queue, view: Mat4, proj: Mat4, position: Vec3) {
        let uniform = CameraUniform::new(view, proj, position);
        queue.write_buffer(&self.camera_uniform, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// 更新光照参数
    pub fn update_lighting(
        &self,
        queue: &wgpu::Queue,
        light_direction: Vec3,
        light_color: Vec3,
        light_intensity: f32,
        ambient_intensity: f32,
    ) {
        let uniform = LightingUniform {
            light_direction: light_direction.to_array(),
            light_color: light_color.to_array(),
            light_intensity,
            ambient_intensity,
            point_light_count: 0,
            _pad: [0; 3],
        };
        queue.write_buffer(&self.lighting_uniform, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// 调整大小
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.renderer.resize(device, width, height);
    }
}

// ============================================================================
// 向后兼容导出
// ============================================================================

/// 延迟渲染器（基础版本）
///
/// 注意：推荐使用`DeferredRendererEnhanced`以获得完整功能
pub type DeferredRendererBase = DeferredRenderer;
