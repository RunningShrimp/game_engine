use super::pbr::{DirectionalLight, PbrMaterial, PointLight3D};
use crate::render::mesh::Vertex3D;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance3D {
    pub model: [[f32; 4]; 4],
}

impl Instance3D {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model Matrix (4x4) takes 4 slots
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms3DPBR {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniformPBR {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    ao: f32,
    normal_scale: f32,
    emissive: [f32; 3],
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
    uv_rotation: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    anisotropy: f32,
    anisotropy_direction: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuPointLight3D {
    position: [f32; 3],
    _pad1: f32,
    color: [f32; 3],
    intensity: f32,
    radius: f32,
    _pad2: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuDirectionalLight {
    direction: [f32; 3],
    _pad1: f32,
    color: [f32; 3],
    intensity: f32,
}

pub struct PbrRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub material_buffer: wgpu::Buffer,
    pub material_bind_group: std::sync::Arc<wgpu::BindGroup>,
    pub material_bgl: wgpu::BindGroupLayout,
    pub lights_buffer: wgpu::Buffer,
    pub lights_bind_group: wgpu::BindGroup,
    pub textures_bind_group: wgpu::BindGroup,
    pub textures_bgl: wgpu::BindGroupLayout,
}

pub struct PbrTextureSet {
    pub textures: [wgpu::Texture; 5],
    pub views: [wgpu::TextureView; 5],
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl PbrRenderer {
    pub fn create_textures_bind_group_from_views(
        &self,
        device: &wgpu::Device,
        views: [&wgpu::TextureView; 5],
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Textures BG (Custom)"),
            layout: &self.textures_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(views[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(views[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(views[3]),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(views[4]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    pub fn create_texture_set_from_images(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: [image::RgbaImage; 5],
        srgb_flags: [bool; 5],
    ) -> PbrTextureSet {
        let mut textures: [wgpu::Texture; 5] = [
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }),
        ];
        let mut views: [wgpu::TextureView; 5] = [
            textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[2].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[3].create_view(&wgpu::TextureViewDescriptor::default()),
            textures[4].create_view(&wgpu::TextureViewDescriptor::default()),
        ];
        for i in 0..5 {
            let (w, h) = images[i].dimensions();
            let format = if srgb_flags[i] {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            };
            textures[i] = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("PBR Texture Imported"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &textures[i],
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                images[i].as_raw(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
            views[i] = textures[i].create_view(&wgpu::TextureViewDescriptor::default());
        }
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = self.create_textures_bind_group_from_views(
            device,
            [&views[0], &views[1], &views[2], &views[3], &views[4]],
            &sampler,
        );
        PbrTextureSet {
            textures,
            views,
            sampler,
            bind_group,
        }
    }
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_pbr.wgsl").into()),
        });

        // 创建Uniform缓冲区和绑定组布局
        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PBR Uniform BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<Uniforms3DPBR>() as wgpu::BufferAddress as u64,
                    ),
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PBR Uniform Buffer"),
            size: std::mem::size_of::<Uniforms3DPBR>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Uniform BG"),
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 材质
        let material_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PBR Material BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        MaterialUniformPBR,
                    >()
                        as wgpu::BufferAddress
                        as u64),
                },
                count: None,
            }],
        });

        let material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PBR Material Buffer"),
            size: std::mem::size_of::<MaterialUniformPBR>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let material_bind_group =
            std::sync::Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("PBR Material BG"),
                layout: &material_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: material_buffer.as_entire_binding(),
                }],
            }));

        // 光源
        let lights_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PBR Lights BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PBR Lights Buffer"),
            size: 256 * std::mem::size_of::<GpuPointLight3D>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dir_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PBR Dir Lights Buffer"),
            size: 16 * std::mem::size_of::<GpuDirectionalLight>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Lights BG"),
            layout: &lights_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: dir_lights_buffer.as_entire_binding(),
                },
            ],
        });

        let textures_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PBR Textures BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let dummy_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PBR Dummy Tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let dummy_view = dummy_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Textures BG"),
            layout: &textures_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[&uniform_bgl, &material_bgl, &lights_bgl, &textures_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex3D>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4],
                    },
                    Instance3D::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
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

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            material_buffer,
            material_bind_group,
            material_bgl,
            lights_buffer,
            lights_bind_group,
            textures_bind_group,
            textures_bgl,
        }
    }

    pub fn create_material_bind_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mat: &PbrMaterial,
    ) -> (
        std::sync::Arc<wgpu::BindGroup>,
        std::sync::Arc<wgpu::Buffer>,
    ) {
        let uniform = MaterialUniformPBR {
            base_color: mat.base_color.to_array(),
            metallic: mat.metallic,
            roughness: mat.roughness,
            ao: mat.ambient_occlusion,
            normal_scale: mat.normal_scale,
            emissive: mat.emissive.to_array(),
            uv_offset: mat.uv_offset,
            uv_scale: mat.uv_scale,
            uv_rotation: mat.uv_rotation,
            clearcoat: mat.clearcoat,
            clearcoat_roughness: mat.clearcoat_roughness,
            anisotropy: mat.anisotropy,
            anisotropy_direction: mat.anisotropy_direction,
        };
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PBR Material Buffer (Per-Material)"),
            size: std::mem::size_of::<MaterialUniformPBR>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buf, 0, bytemuck::bytes_of(&uniform));
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Material BG (Per-Material)"),
            layout: &self.material_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buf.as_entire_binding(),
            }],
        });
        (std::sync::Arc::new(bg), std::sync::Arc::new(buf))
    }

    pub fn encode_material_uniform(mat: &PbrMaterial) -> MaterialUniformPBR {
        MaterialUniformPBR {
            base_color: mat.base_color.to_array(),
            metallic: mat.metallic,
            roughness: mat.roughness,
            ao: mat.ambient_occlusion,
            normal_scale: mat.normal_scale,
            emissive: mat.emissive.to_array(),
            uv_offset: mat.uv_offset,
            uv_scale: mat.uv_scale,
            uv_rotation: mat.uv_rotation,
            clearcoat: mat.clearcoat,
            clearcoat_roughness: mat.clearcoat_roughness,
            anisotropy: mat.anisotropy,
            anisotropy_direction: mat.anisotropy_direction,
        }
    }

    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
    ) {
        let uniforms = Uniforms3DPBR {
            view_proj,
            camera_pos,
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn update_material(&self, queue: &wgpu::Queue, material: &PbrMaterial) {
        let uniform = MaterialUniformPBR {
            base_color: material.base_color.to_array(),
            metallic: material.metallic,
            roughness: material.roughness,
            ao: material.ambient_occlusion,
            normal_scale: material.normal_scale,
            emissive: material.emissive.to_array(),
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            uv_rotation: 0.0,
            clearcoat: 0.0,
            clearcoat_roughness: 0.5,
            anisotropy: 0.0,
            anisotropy_direction: [1.0, 0.0],
        };
        queue.write_buffer(&self.material_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn update_lights(
        &self,
        queue: &wgpu::Queue,
        point_lights: &[PointLight3D],
        dir_lights: &[DirectionalLight],
    ) {
        // 更新点光源
        let gpu_point_lights: Vec<GpuPointLight3D> = point_lights
            .iter()
            .map(|light| GpuPointLight3D {
                position: light.position.to_array(),
                _pad1: 0.0,
                color: light.color.to_array(),
                intensity: light.intensity,
                radius: light.radius,
                _pad2: [0.0; 3],
            })
            .collect();

        if !gpu_point_lights.is_empty() {
            queue.write_buffer(
                &self.lights_buffer,
                0,
                bytemuck::cast_slice(&gpu_point_lights),
            );
        }

        // 更新方向光
        let gpu_dir_lights: Vec<GpuDirectionalLight> = dir_lights
            .iter()
            .map(|light| GpuDirectionalLight {
                direction: light.direction.to_array(),
                _pad1: 0.0,
                color: light.color.to_array(),
                intensity: light.intensity,
            })
            .collect();

        if !gpu_dir_lights.is_empty() {
            queue.write_buffer(
                &self.lights_buffer,
                std::mem::size_of::<GpuPointLight3D>() as u64 * 256,
                bytemuck::cast_slice(&gpu_dir_lights),
            );
        }
    }

    /// 渲染单个网格实例
    pub fn render_mesh<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a super::mesh::GpuMesh,
        instance_buffer: &'a wgpu::Buffer,
        instance_count: u32,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.material_bind_group, &[]);
        render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
        render_pass.set_bind_group(3, &self.textures_bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..mesh.index_count, 0, 0..instance_count);
    }

    /// 渲染实例化批次
    pub fn render_batch<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        batch: &'a super::instance_batch::InstanceBatch,
    ) {
        if batch.instances.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &batch.material_bind_group, &[]);
        render_pass.set_bind_group(2, &self.lights_bind_group, &[]);
        if let Some(bg) = batch.extra_material_bind_groups.get(0) {
            render_pass.set_bind_group(3, bg, &[]);
        } else {
            render_pass.set_bind_group(3, &self.textures_bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));
        if let Some(instance_buffer) = &batch.instance_buffer {
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        }
        render_pass.set_index_buffer(batch.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
    }

    /// 渲染所有可见批次
    pub fn render_all_batches<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        batch_manager: &'a super::instance_batch::BatchManager,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.lights_bind_group, &[]);

        for batch in batch_manager.visible_batches() {
            if batch.instances.is_empty() {
                continue;
            }

            render_pass.set_bind_group(1, &batch.material_bind_group, &[]);
            if let Some(bg) = batch.extra_material_bind_groups.get(0) {
                render_pass.set_bind_group(3, bg, &[]);
            } else {
                render_pass.set_bind_group(3, &self.textures_bind_group, &[]);
            }

            // 绑定顶点缓冲区
            render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));
            if let Some(instance_buffer) = &batch.instance_buffer {
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            }
            render_pass
                .set_index_buffer(batch.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // 实例化绘制
            render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
        }
    }
}
