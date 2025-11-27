//! Tonemap（色调映射）后处理效果
//!
//! 将 HDR 颜色映射到 SDR 显示范围，支持多种色调映射算法。
//!
//! ## 支持的算法
//! - None: 无色调映射（直接裁剪）
//! - Reinhard: 简单的 Reinhard 算法
//! - ACES: Academy Color Encoding System，电影级色调映射
//! - Filmic: 类似胶片的色调映射

/// 色调映射算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TonemapOperator {
    /// 无色调映射
    None = 0,
    /// Reinhard 算法
    Reinhard = 1,
    /// ACES 算法（默认）
    #[default]
    ACES = 2,
    /// Filmic 算法
    Filmic = 3,
}

/// Tonemap Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TonemapUniforms {
    /// 曝光值
    pub exposure: f32,
    /// Gamma 校正值
    pub gamma: f32,
    /// 色调映射算法 (0=None, 1=Reinhard, 2=ACES, 3=Filmic)
    pub tonemap_mode: u32,
    /// 填充
    pub _pad: u32,
}

/// Tonemap 渲染通道
pub struct TonemapPass {
    /// 渲染管线
    pipeline: wgpu::RenderPipeline,
    
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    
    /// 采样器
    sampler: wgpu::Sampler,
    
    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,
    
    /// 输出格式
    output_format: wgpu::TextureFormat,
}

impl TonemapPass {
    /// 创建 Tonemap 通道
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Tonemap Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tonemap BGL"),
            entries: &[
                // 输入纹理
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
                // 采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tonemap Uniform Buffer"),
            size: std::mem::size_of::<TonemapUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tonemap Shader"),
            source: wgpu::ShaderSource::Wgsl(TONEMAP_SHADER.into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tonemap Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // 创建渲染管线
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tonemap Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_fullscreen",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_tonemap",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        Self {
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            output_format,
        }
    }
    
    /// 执行色调映射渲染
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        exposure: f32,
        gamma: f32,
        operator: TonemapOperator,
    ) {
        // 更新 uniforms
        let uniforms = TonemapUniforms {
            exposure,
            gamma,
            tonemap_mode: operator as u32,
            _pad: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tonemap BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });
        
        // 渲染
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Tonemap Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}

/// Tonemap 着色器
const TONEMAP_SHADER: &str = r#"
struct TonemapUniforms {
    exposure: f32,
    gamma: f32,
    tonemap_mode: u32,
    _pad: u32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: TonemapUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// 全屏三角形顶点着色器
@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    
    return out;
}

// Reinhard 色调映射
fn tonemap_reinhard(color: vec3<f32>) -> vec3<f32> {
    return color / (color + vec3<f32>(1.0));
}

// ACES 色调映射
fn tonemap_aces(color: vec3<f32>) -> vec3<f32> {
    // ACES 输入变换矩阵
    let aces_input = mat3x3<f32>(
        vec3<f32>(0.59719, 0.07600, 0.02840),
        vec3<f32>(0.35458, 0.90834, 0.13383),
        vec3<f32>(0.04823, 0.01566, 0.83777)
    );
    
    // ACES 输出变换矩阵
    let aces_output = mat3x3<f32>(
        vec3<f32>(1.60475, -0.10208, -0.00327),
        vec3<f32>(-0.53108, 1.10813, -0.07276),
        vec3<f32>(-0.07367, -0.00605, 1.07602)
    );
    
    var c = aces_input * color;
    
    // RRT and ODT fit
    let a = c * (c + vec3<f32>(0.0245786)) - vec3<f32>(0.000090537);
    let b = c * (vec3<f32>(0.983729) * c + vec3<f32>(0.4329510)) + vec3<f32>(0.238081);
    c = a / b;
    
    return aces_output * c;
}

// Filmic 色调映射 (Uncharted 2 风格)
fn tonemap_filmic(color: vec3<f32>) -> vec3<f32> {
    let A = 0.15;  // Shoulder Strength
    let B = 0.50;  // Linear Strength
    let C = 0.10;  // Linear Angle
    let D = 0.20;  // Toe Strength
    let E = 0.02;  // Toe Numerator
    let F = 0.30;  // Toe Denominator
    let W = 11.2;  // Linear White Point
    
    let x = color;
    let result = ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
    
    let white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
    
    return result / white;
}

// 主色调映射片段着色器
@fragment
fn fs_tonemap(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(input_texture, input_sampler, in.uv).rgb;
    
    // 应用曝光
    color = color * uniforms.exposure;
    
    // 应用色调映射
    switch (uniforms.tonemap_mode) {
        case 0u: {
            // None - 直接裁剪
            color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
        }
        case 1u: {
            // Reinhard
            color = tonemap_reinhard(color);
        }
        case 2u: {
            // ACES
            color = tonemap_aces(color);
        }
        case 3u: {
            // Filmic
            color = tonemap_filmic(color);
        }
        default: {
            color = tonemap_aces(color);
        }
    }
    
    // 裁剪到 [0, 1]
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // Gamma 校正
    color = pow(color, vec3<f32>(1.0 / uniforms.gamma));
    
    return vec4<f32>(color, 1.0);
}
"#;
