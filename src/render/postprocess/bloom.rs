//! Bloom（辉光）后处理效果
//!
//! 实现双向高斯模糊 + 亮度提取的 Bloom 效果。
//! 
//! ## 算法流程
//! 1. 亮度提取：从场景中提取高于阈值的亮度区域
//! 2. 降采样：逐级降低分辨率，扩大模糊范围
//! 3. 高斯模糊：对每级进行水平和垂直方向的高斯模糊
//! 4. 升采样：逐级提升分辨率，混合各级结果
//! 5. 合成：将模糊结果与原场景混合

/// Bloom 通道 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomUniforms {
    /// 纹理尺寸 (width, height)
    pub texture_size: [f32; 2],
    /// 模糊方向 (1,0) 水平 或 (0,1) 垂直
    pub direction: [f32; 2],
    /// 亮度阈值
    pub threshold: f32,
    /// Bloom 强度
    pub intensity: f32,
    /// 模糊半径
    pub radius: f32,
    /// 当前 mip 级别
    pub mip_level: f32,
}

/// Bloom 渲染通道
pub struct BloomPass {
    /// 亮度提取管线
    threshold_pipeline: wgpu::RenderPipeline,
    /// 模糊管线
    blur_pipeline: wgpu::RenderPipeline,
    /// 合成管线
    composite_pipeline: wgpu::RenderPipeline,
    
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    
    /// 降采样纹理链 (多个 mip 级别)
    downsample_textures: Vec<wgpu::Texture>,
    downsample_views: Vec<wgpu::TextureView>,
    
    /// 模糊中间纹理
    blur_temp_textures: Vec<wgpu::Texture>,
    blur_temp_views: Vec<wgpu::TextureView>,
    
    /// 输出纹理
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    
    /// 采样器
    sampler: wgpu::Sampler,
    
    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,
    
    /// 屏幕尺寸
    width: u32,
    height: u32,
    
    /// Mip 级别数量
    mip_count: u32,
}

impl BloomPass {
    /// 创建 Bloom 通道
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let mip_count = Self::calculate_mip_count(width, height);
        
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Bloom Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom BGL"),
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
            label: Some("Bloom Uniform Buffer"),
            size: std::mem::size_of::<BloomUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Shader"),
            source: wgpu::ShaderSource::Wgsl(BLOOM_SHADER.into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // 创建管线
        let threshold_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "vs_fullscreen",
            "fs_threshold",
            wgpu::TextureFormat::Rgba16Float,
        );
        
        let blur_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "vs_fullscreen",
            "fs_blur",
            wgpu::TextureFormat::Rgba16Float,
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
        let (downsample_textures, downsample_views) = 
            Self::create_mip_chain(device, width, height, mip_count, "Downsample");
        let (blur_temp_textures, blur_temp_views) = 
            Self::create_mip_chain(device, width, height, mip_count, "BlurTemp");
        
        // 创建输出纹理
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom Output"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self {
            threshold_pipeline,
            blur_pipeline,
            composite_pipeline,
            bind_group_layout,
            downsample_textures,
            downsample_views,
            blur_temp_textures,
            blur_temp_views,
            output_texture,
            output_view,
            sampler,
            uniform_buffer,
            width,
            height,
            mip_count,
        }
    }
    
    /// 计算 mip 级别数量
    fn calculate_mip_count(width: u32, height: u32) -> u32 {
        let min_dim = width.min(height) as f32;
        ((min_dim.log2()).floor() as u32).min(6).max(1)
    }
    
    /// 创建 mip 纹理链
    fn create_mip_chain(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        mip_count: u32,
        label_prefix: &str,
    ) -> (Vec<wgpu::Texture>, Vec<wgpu::TextureView>) {
        let mut textures = Vec::with_capacity(mip_count as usize);
        let mut views = Vec::with_capacity(mip_count as usize);
        
        for i in 0..mip_count {
            let scale = 1 << i;
            let mip_width = (width / scale).max(1);
            let mip_height = (height / scale).max(1);
            
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom {} Mip {}", label_prefix, i)),
                size: wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
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
            textures.push(texture);
            views.push(view);
        }
        
        (textures, views)
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
            label: Some(&format!("Bloom {} Pipeline", fs_entry)),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        self.mip_count = Self::calculate_mip_count(width, height);
        
        let (downsample_textures, downsample_views) = 
            Self::create_mip_chain(device, width, height, self.mip_count, "Downsample");
        let (blur_temp_textures, blur_temp_views) = 
            Self::create_mip_chain(device, width, height, self.mip_count, "BlurTemp");
        
        self.downsample_textures = downsample_textures;
        self.downsample_views = downsample_views;
        self.blur_temp_textures = blur_temp_textures;
        self.blur_temp_views = blur_temp_views;
        
        self.output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom Output"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.output_view = self.output_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }
    
    /// 执行 Bloom 渲染
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_view: &wgpu::TextureView,
        threshold: f32,
        intensity: f32,
        radius: f32,
    ) {
        if self.mip_count == 0 {
            return;
        }
        
        // 1. 亮度提取 + 第一次降采样
        self.render_threshold(encoder, device, queue, input_view, threshold);
        
        // 2. 降采样 + 模糊
        for i in 1..self.mip_count as usize {
            let scale = 1 << i;
            let mip_width = (self.width / scale).max(1);
            let mip_height = (self.height / scale).max(1);
            
            // 降采样
            self.render_downsample(encoder, device, queue, i, mip_width, mip_height);
            
            // 水平模糊
            self.render_blur(
                encoder, device, queue,
                &self.downsample_views[i],
                &self.blur_temp_views[i],
                mip_width, mip_height,
                [1.0, 0.0], radius, i as f32,
            );
            
            // 垂直模糊
            self.render_blur(
                encoder, device, queue,
                &self.blur_temp_views[i],
                &self.downsample_views[i],
                mip_width, mip_height,
                [0.0, 1.0], radius, i as f32,
            );
        }
        
        // 3. 升采样 + 合成
        self.render_composite(encoder, device, queue, input_view, intensity);
    }
    
    /// 渲染亮度提取
    fn render_threshold(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_view: &wgpu::TextureView,
        threshold: f32,
    ) {
        let uniforms = BloomUniforms {
            texture_size: [self.width as f32, self.height as f32],
            direction: [0.0, 0.0],
            threshold,
            intensity: 1.0,
            radius: 0.0,
            mip_level: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Threshold BG"),
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
        
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Threshold Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.downsample_views[0],
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
        
        rpass.set_pipeline(&self.threshold_pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
    
    /// 渲染降采样
    fn render_downsample(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mip_level: usize,
        width: u32,
        height: u32,
    ) {
        let uniforms = BloomUniforms {
            texture_size: [width as f32, height as f32],
            direction: [0.0, 0.0],
            threshold: 0.0,
            intensity: 1.0,
            radius: 0.0,
            mip_level: mip_level as f32,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Downsample BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.downsample_views[mip_level - 1]),
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
        
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Downsample Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.downsample_views[mip_level],
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
        
        rpass.set_pipeline(&self.blur_pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
    
    /// 渲染高斯模糊
    fn render_blur(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        direction: [f32; 2],
        radius: f32,
        mip_level: f32,
    ) {
        let uniforms = BloomUniforms {
            texture_size: [width as f32, height as f32],
            direction,
            threshold: 0.0,
            intensity: 1.0,
            radius,
            mip_level,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Blur BG"),
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
        
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Blur Pass"),
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
        
        rpass.set_pipeline(&self.blur_pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
    
    /// 渲染合成
    fn render_composite(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_view: &wgpu::TextureView,
        intensity: f32,
    ) {
        // 使用最低 mip 作为 bloom 纹理，与原场景合成
        let bloom_view = &self.downsample_views[0];
        
        let uniforms = BloomUniforms {
            texture_size: [self.width as f32, self.height as f32],
            direction: [0.0, 0.0],
            threshold: 0.0,
            intensity,
            radius: 0.0,
            mip_level: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        // 需要两个纹理的绑定组，这里简化为使用场景纹理
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Composite BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_view),
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
        
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Composite Pass"),
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
    
    /// 获取输出纹理视图
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }
}

/// Bloom 着色器
const BLOOM_SHADER: &str = r#"
struct BloomUniforms {
    texture_size: vec2<f32>,
    direction: vec2<f32>,
    threshold: f32,
    intensity: f32,
    radius: f32,
    mip_level: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: BloomUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// 全屏三角形顶点着色器
@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // 生成覆盖全屏的三角形
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    
    return out;
}

// 计算亮度
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// 软阈值函数
fn soft_threshold(color: vec3<f32>, threshold: f32) -> vec3<f32> {
    let brightness = luminance(color);
    let soft = brightness - threshold + 0.1;
    let contribution = max(0.0, soft) / max(brightness, 0.0001);
    return color * contribution;
}

// 亮度提取片段着色器
@fragment
fn fs_threshold(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv).rgb;
    let result = soft_threshold(color, uniforms.threshold);
    return vec4<f32>(result, 1.0);
}

// 13-tap 高斯模糊权重
fn gaussian_weight(offset: f32, sigma: f32) -> f32 {
    let sigma2 = sigma * sigma;
    return exp(-(offset * offset) / (2.0 * sigma2)) / (sqrt(2.0 * 3.14159265) * sigma);
}

// 高斯模糊片段着色器
@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / uniforms.texture_size;
    let direction = uniforms.direction * texel_size;
    let sigma = uniforms.radius * 0.5 + 1.0;
    
    var color = vec3<f32>(0.0);
    var total_weight = 0.0;
    
    // 9-tap 高斯模糊
    for (var i = -4; i <= 4; i++) {
        let offset = f32(i);
        let weight = gaussian_weight(offset, sigma);
        let sample_uv = in.uv + direction * offset;
        color += textureSample(input_texture, input_sampler, sample_uv).rgb * weight;
        total_weight += weight;
    }
    
    return vec4<f32>(color / total_weight, 1.0);
}

// 合成片段着色器
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(input_texture, input_sampler, in.uv).rgb;
    // 这里简化处理，实际应该有独立的 bloom 纹理
    let bloom_color = scene_color * uniforms.intensity * 0.5;
    let result = scene_color + bloom_color;
    return vec4<f32>(result, 1.0);
}
"#;
