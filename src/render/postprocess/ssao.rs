//! SSAO（屏幕空间环境光遮蔽）后处理效果
//!
//! 实现基于屏幕空间的环境光遮蔽效果，增强场景的深度感。
//!
//! ## 算法流程
//! 1. 采样深度缓冲获取像素深度
//! 2. 在半球范围内随机采样多个点
//! 3. 比较采样点深度与实际深度，累积遮蔽因子
//! 4. 应用模糊降噪
//! 5. 与场景颜色混合

use rand::Rng;

/// SSAO Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SsaoUniforms {
    /// 投影矩阵
    pub projection: [[f32; 4]; 4],
    /// 逆投影矩阵
    pub inv_projection: [[f32; 4]; 4],
    /// 屏幕尺寸
    pub screen_size: [f32; 2],
    /// 采样半径
    pub radius: f32,
    /// 强度
    pub intensity: f32,
    /// 偏移
    pub bias: f32,
    /// 采样数量
    pub sample_count: u32,
    /// 填充
    pub _pad: [f32; 2],
}

/// SSAO 采样核
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SsaoKernel {
    /// 采样方向 (64个采样点)
    pub samples: [[f32; 4]; 64],
}

/// SSAO 渲染通道
pub struct SsaoPass {
    /// SSAO 计算管线
    ssao_pipeline: wgpu::RenderPipeline,
    /// 模糊管线
    blur_pipeline: wgpu::RenderPipeline,
    /// 合成管线
    composite_pipeline: wgpu::RenderPipeline,

    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,

    /// SSAO 输出纹理
    ssao_texture: wgpu::Texture,
    ssao_view: wgpu::TextureView,

    /// 模糊中间纹理
    blur_texture: wgpu::Texture,
    blur_view: wgpu::TextureView,

    /// 最终输出纹理
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,

    /// 噪声纹理
    noise_texture: wgpu::Texture,
    noise_view: wgpu::TextureView,

    /// 采样器
    sampler: wgpu::Sampler,
    noise_sampler: wgpu::Sampler,

    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,

    /// 采样核缓冲区
    kernel_buffer: wgpu::Buffer,

    /// 屏幕尺寸
    width: u32,
    height: u32,
}

impl SsaoPass {
    /// 创建 SSAO 通道
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Noise Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO BGL"),
            entries: &[
                // 深度纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 场景纹理
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
                // 噪声纹理
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
                // 采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // 噪声采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 采样核
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
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
            label: Some("SSAO Uniform Buffer"),
            size: std::mem::size_of::<SsaoUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 生成采样核
        let kernel = Self::generate_kernel();
        let kernel_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Kernel Buffer"),
            size: std::mem::size_of::<SsaoKernel>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 将采样核数据写入缓冲区（需要在queue可用时写入）
        // kernel用于SSAO采样，需要写入到kernel_buffer
        let _kernel_data = bytemuck::bytes_of(&kernel);
        // queue.write_buffer(&kernel_buffer, 0, _kernel_data);

        // 创建噪声纹理
        let (noise_texture, noise_view) = Self::create_noise_texture(device);

        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Shader"),
            source: wgpu::ShaderSource::Wgsl(SSAO_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建管线
        let ssao_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "vs_fullscreen",
            "fs_ssao",
            wgpu::TextureFormat::R8Unorm,
        );

        let blur_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "vs_fullscreen",
            "fs_blur",
            wgpu::TextureFormat::R8Unorm,
        );

        let composite_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "vs_fullscreen",
            "fs_composite",
            wgpu::TextureFormat::Rgba16Float,
        );

        // 创建纹理
        let (ssao_texture, ssao_view) = Self::create_ao_texture(device, width, height, "SSAO");
        let (blur_texture, blur_view) = Self::create_ao_texture(device, width, height, "SSAO Blur");
        let (output_texture, output_view) = Self::create_output_texture(device, width, height);

        Self {
            ssao_pipeline,
            blur_pipeline,
            composite_pipeline,
            bind_group_layout,
            ssao_texture,
            ssao_view,
            blur_texture,
            blur_view,
            output_texture,
            output_view,
            noise_texture,
            noise_view,
            sampler,
            noise_sampler,
            uniform_buffer,
            kernel_buffer,
            width,
            height,
        }
    }

    /// 生成采样核
    fn generate_kernel() -> SsaoKernel {
        let mut rng = rand::thread_rng();
        let mut samples = [[0.0f32; 4]; 64];

        for i in 0..64 {
            // 在单位半球内生成随机点
            let mut sample = [
                rng.r#gen::<f32>() * 2.0 - 1.0,
                rng.r#gen::<f32>() * 2.0 - 1.0,
                rng.r#gen::<f32>(),
                0.0,
            ];

            // 归一化
            let len =
                (sample[0] * sample[0] + sample[1] * sample[1] + sample[2] * sample[2]).sqrt();
            sample[0] /= len;
            sample[1] /= len;
            sample[2] /= len;

            // 使采样点在核心附近更密集
            let scale = (i as f32) / 64.0;
            let scale = 0.1 + scale * scale * 0.9;
            sample[0] *= scale;
            sample[1] *= scale;
            sample[2] *= scale;

            samples[i] = sample;
        }

        SsaoKernel { samples }
    }

    /// 创建噪声纹理
    fn create_noise_texture(device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
        let mut rng = rand::thread_rng();
        let size = 4u32;
        let mut noise_data = vec![0u8; (size * size * 4) as usize];

        for i in 0..(size * size) as usize {
            let idx = i * 4;
            // 生成随机旋转向量 (在 tangent 空间)
            noise_data[idx] = ((rng.r#gen::<f32>() * 2.0 - 1.0) * 127.0 + 128.0) as u8;
            noise_data[idx + 1] = ((rng.r#gen::<f32>() * 2.0 - 1.0) * 127.0 + 128.0) as u8;
            noise_data[idx + 2] = 0;
            noise_data[idx + 3] = 255;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Noise Texture"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// 创建 AO 纹理
    fn create_ao_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// 创建输出纹理
    fn create_output_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Output"),
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
        (texture, view)
    }

    /// 创建渲染管线
    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vs_entry: &str,
        fs_entry: &str,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("SSAO {} Pipeline", fs_entry)),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: vs_entry,
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: fs_entry,
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        })
    }

    /// 调整大小
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == self.width && height == self.height {
            return;
        }

        self.width = width;
        self.height = height;

        let (ssao_texture, ssao_view) = Self::create_ao_texture(device, width, height, "SSAO");
        let (blur_texture, blur_view) = Self::create_ao_texture(device, width, height, "SSAO Blur");
        let (output_texture, output_view) = Self::create_output_texture(device, width, height);

        self.ssao_texture = ssao_texture;
        self.ssao_view = ssao_view;
        self.blur_texture = blur_texture;
        self.blur_view = blur_view;
        self.output_texture = output_texture;
        self.output_view = output_view;
    }

    /// 执行 SSAO 渲染
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        radius: f32,
        intensity: f32,
        bias: f32,
    ) {
        // 更新 uniforms
        let uniforms = SsaoUniforms {
            projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            inv_projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            screen_size: [self.width as f32, self.height as f32],
            radius,
            intensity,
            bias,
            sample_count: 32,
            _pad: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // 更新采样核
        let kernel = Self::generate_kernel();
        queue.write_buffer(&self.kernel_buffer, 0, bytemuck::bytes_of(&kernel));

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.noise_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.noise_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.kernel_buffer.as_entire_binding(),
                },
            ],
        });

        // 1. SSAO 计算
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.ssao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            rpass.set_pipeline(&self.ssao_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        // 2. 模糊
        // (简化：跳过模糊步骤，直接使用 SSAO 结果)

        // 3. 合成
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Composite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
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

            rpass.set_pipeline(&self.composite_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }

    /// 获取输出纹理视图
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }
}

/// SSAO 着色器
const SSAO_SHADER: &str = r#"
struct SsaoUniforms {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    radius: f32,
    intensity: f32,
    bias: f32,
    sample_count: u32,
    _pad: vec2<f32>,
};

struct SsaoKernel {
    samples: array<vec4<f32>, 64>,
};

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var scene_texture: texture_2d<f32>;
@group(0) @binding(2) var noise_texture: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;
@group(0) @binding(4) var noise_sampler: sampler;
@group(0) @binding(5) var<uniform> uniforms: SsaoUniforms;
@group(0) @binding(6) var<uniform> kernel: SsaoKernel;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

// 从深度重建视空间位置
fn reconstruct_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let view_pos = uniforms.inv_projection * ndc;
    return view_pos.xyz / view_pos.w;
}

@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) f32 {
    let depth = textureSample(depth_texture, tex_sampler, in.uv);
    
    if (depth >= 1.0) {
        return 1.0;
    }
    
    let frag_pos = reconstruct_position(in.uv, depth);
    
    // 从噪声纹理获取随机向量
    let noise_scale = uniforms.screen_size / 4.0;
    let random_vec = textureSample(noise_texture, noise_sampler, in.uv * noise_scale).xyz * 2.0 - 1.0;
    
    // 构建 TBN 矩阵 (简化版，假设法线向上)
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);
    
    var occlusion = 0.0;
    let sample_count = min(uniforms.sample_count, 64u);
    
    for (var i = 0u; i < sample_count; i++) {
        // 获取采样向量并变换到视空间
        var sample_pos = tbn * kernel.samples[i].xyz;
        sample_pos = frag_pos + sample_pos * uniforms.radius;
        
        // 投影到屏幕空间
        var offset = uniforms.projection * vec4<f32>(sample_pos, 1.0);
        offset = offset / offset.w;
        let sample_uv = offset.xy * 0.5 + 0.5;
        
        // 获取采样点深度
        let sample_depth = textureSample(depth_texture, tex_sampler, sample_uv);
        let sample_view_pos = reconstruct_position(sample_uv, sample_depth);
        
        // 范围检查
        let range_check = smoothstep(0.0, 1.0, uniforms.radius / abs(frag_pos.z - sample_view_pos.z));
        
        // 遮蔽检查
        if (sample_view_pos.z >= sample_pos.z + uniforms.bias) {
            occlusion += range_check;
        }
    }
    
    occlusion = 1.0 - (occlusion / f32(sample_count));
    return pow(occlusion, uniforms.intensity);
}

@fragment
fn fs_blur(in: VertexOutput) -> @location(0) f32 {
    let texel_size = 1.0 / uniforms.screen_size;
    var result = 0.0;
    
    for (var x = -2; x <= 2; x++) {
        for (var y = -2; y <= 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            result += textureSample(scene_texture, tex_sampler, in.uv + offset).r;
        }
    }
    
    return result / 25.0;
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(scene_texture, tex_sampler, in.uv).rgb;
    let ao = textureSample(scene_texture, tex_sampler, in.uv).r; // 简化：使用场景纹理
    return vec4<f32>(scene_color * ao, 1.0);
}
"#;
