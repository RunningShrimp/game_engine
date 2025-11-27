use super::pbr::{PbrMaterial, PointLight3D, DirectionalLight};
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
struct MaterialUniformPBR {

    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    ao: f32,
    normal_scale: f32,
    emissive: [f32; 3],
    _pad: f32,
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
    pub material_bind_group: wgpu::BindGroup,
    pub lights_buffer: wgpu::Buffer,
    pub lights_bind_group: wgpu::BindGroup,
}

impl PbrRenderer {
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
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<Uniforms3DPBR>() as wgpu::BufferAddress as u64),
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
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<MaterialUniformPBR>() as wgpu::BufferAddress as u64),
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

        let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PBR Material BG"),
            layout: &material_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
        });

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

        // 创建管线
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[&uniform_bgl, &material_bgl, &lights_bgl],
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
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
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
            lights_buffer,
            lights_bind_group,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4], camera_pos: [f32; 3]) {
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
            _pad: 0.0,
        };
        queue.write_buffer(&self.material_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn update_lights(&self, queue: &wgpu::Queue, point_lights: &[PointLight3D], dir_lights: &[DirectionalLight]) {
        // 更新点光源
        let gpu_point_lights: Vec<GpuPointLight3D> = point_lights.iter().map(|light| {
            GpuPointLight3D {
                position: light.position.to_array(),
                _pad1: 0.0,
                color: light.color.to_array(),
                intensity: light.intensity,
                radius: light.radius,
                _pad2: [0.0; 3],
            }
        }).collect();
        
        if !gpu_point_lights.is_empty() {
            queue.write_buffer(&self.lights_buffer, 0, bytemuck::cast_slice(&gpu_point_lights));
        }

        // 更新方向光
        let gpu_dir_lights: Vec<GpuDirectionalLight> = dir_lights.iter().map(|light| {
            GpuDirectionalLight {
                direction: light.direction.to_array(),
                _pad1: 0.0,
                color: light.color.to_array(),
                intensity: light.intensity,
            }
        }).collect();
        
        if !gpu_dir_lights.is_empty() {
            queue.write_buffer(&self.lights_buffer, std::mem::size_of::<GpuPointLight3D>() as u64 * 256, bytemuck::cast_slice(&gpu_dir_lights));
        }
    }
}
