//! 抗锯齿后处理模块
//!
//! 提供多种抗锯齿算法实现：
//! - FXAA (Fast Approximate Anti-Aliasing) - 快速近似抗锯齿
//! - TAA (Temporal Anti-Aliasing) - 时间抗锯齿
//! - SMAA (Subpixel Morphological Anti-Aliasing) - 子像素形态学抗锯齿
//!
//! # 示例
//!
//! ```ignore
//! let mut config = PostProcessConfig::default();
//! config.antialiasing = AntialiasingMode::FXAA;
//! config.fxaa_quality = FxaaQuality::High;
//! ```

/// 抗锯齿模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AntialiasingMode {
    /// 无抗锯齿
    #[default]
    None,
    /// 快速近似抗锯齿（低开销，适合性能敏感场景）
    FXAA,
    /// 时间抗锯齿（高质量，需要运动向量）
    TAA,
    /// 子像素形态学抗锯齿（平衡质量和性能）
    SMAA,
}

/// FXAA 质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxaaQuality {
    /// 低质量（最快）
    Low,
    /// 中等质量
    #[default]
    Medium,
    /// 高质量（最佳效果）
    High,
    /// 极致质量（用于截图）
    Ultra,
}

impl FxaaQuality {
    /// 获取对应的采样迭代次数
    pub fn iterations(&self) -> u32 {
        match self {
            FxaaQuality::Low => 4,
            FxaaQuality::Medium => 8,
            FxaaQuality::High => 12,
            FxaaQuality::Ultra => 16,
        }
    }
    
    /// 获取边缘检测阈值
    pub fn edge_threshold(&self) -> f32 {
        match self {
            FxaaQuality::Low => 0.250,
            FxaaQuality::Medium => 0.166,
            FxaaQuality::High => 0.125,
            FxaaQuality::Ultra => 0.063,
        }
    }
    
    /// 获取最小边缘阈值
    pub fn edge_threshold_min(&self) -> f32 {
        match self {
            FxaaQuality::Low => 0.0833,
            FxaaQuality::Medium => 0.0625,
            FxaaQuality::High => 0.0312,
            FxaaQuality::Ultra => 0.0156,
        }
    }
}

/// FXAA Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FxaaUniforms {
    /// 屏幕尺寸 (width, height)
    pub screen_size: [f32; 2],
    /// 像素大小 (1/width, 1/height)
    pub pixel_size: [f32; 2],
    /// 边缘检测阈值
    pub edge_threshold: f32,
    /// 最小边缘阈值
    pub edge_threshold_min: f32,
    /// 子像素质量
    pub subpix_quality: f32,
    /// 填充
    pub _pad: f32,
}

/// FXAA 渲染通道
pub struct FxaaPass {
    /// 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,
    /// 采样器
    sampler: wgpu::Sampler,
    /// 当前质量
    quality: FxaaQuality,
}

impl FxaaPass {
    /// 创建 FXAA 渲染通道
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FXAA Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_fxaa.wgsl").into()),
        });
        
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("FXAA Bind Group Layout"),
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
        
        // 创建管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("FXAA Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // 创建渲染管线
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("FXAA Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
        
        // 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("FXAA Uniform Buffer"),
            size: std::mem::size_of::<FxaaUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("FXAA Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            sampler,
            quality: FxaaQuality::default(),
        }
    }
    
    /// 设置 FXAA 质量
    pub fn set_quality(&mut self, quality: FxaaQuality) {
        self.quality = quality;
    }
    
    /// 获取当前质量
    pub fn quality(&self) -> FxaaQuality {
        self.quality
    }
    
    /// 执行 FXAA 渲染
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        // 更新 Uniforms
        let uniforms = FxaaUniforms {
            screen_size: [width as f32, height as f32],
            pixel_size: [1.0 / width as f32, 1.0 / height as f32],
            edge_threshold: self.quality.edge_threshold(),
            edge_threshold_min: self.quality.edge_threshold_min(),
            subpix_quality: 0.75,
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        
        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("FXAA Bind Group"),
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
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("FXAA Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1); // 全屏三角形
    }
}

// ============================================================================
// TAA (Temporal Anti-Aliasing)
// ============================================================================

/// TAA Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TaaUniforms {
    /// 屏幕尺寸
    pub screen_size: [f32; 2],
    /// 像素大小
    pub pixel_size: [f32; 2],
    /// 抖动偏移 (当前帧)
    pub jitter_offset: [f32; 2],
    /// 混合因子 (历史权重)
    pub blend_factor: f32,
    /// 填充
    pub _pad: f32,
}

/// TAA 渲染通道
pub struct TaaPass {
    /// 解析管线
    resolve_pipeline: wgpu::RenderPipeline,
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform 缓冲区
    uniform_buffer: wgpu::Buffer,
    /// 历史纹理 A
    history_texture_a: Option<wgpu::Texture>,
    /// 历史纹理 B
    history_texture_b: Option<wgpu::Texture>,
    /// 当前使用的历史纹理索引
    current_history: usize,
    /// 采样器
    sampler: wgpu::Sampler,
    /// 当前帧索引 (用于抖动序列)
    frame_index: u32,
    /// 抖动序列
    jitter_sequence: Vec<[f32; 2]>,
}

impl TaaPass {
    /// 创建 TAA 渲染通道
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        // Halton 序列生成抖动偏移
        let jitter_sequence = Self::generate_halton_sequence(16);
        
        // 创建着色器
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TAA Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_taa.wgsl").into()),
        });
        
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TAA Bind Group Layout"),
            entries: &[
                // 当前帧纹理
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
                // 历史帧纹理
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
                // 运动向量纹理
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
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
        
        // 创建管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TAA Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // 创建解析管线
        let resolve_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("TAA Resolve Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
        
        // 创建 Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TAA Uniform Buffer"),
            size: std::mem::size_of::<TaaUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            resolve_pipeline,
            bind_group_layout,
            uniform_buffer,
            history_texture_a: None,
            history_texture_b: None,
            current_history: 0,
            sampler,
            frame_index: 0,
            jitter_sequence,
        }
    }
    
    /// 生成 Halton 序列用于抖动
    fn generate_halton_sequence(count: usize) -> Vec<[f32; 2]> {
        let mut sequence = Vec::with_capacity(count);
        for i in 0..count {
            let x = Self::halton(i as u32 + 1, 2);
            let y = Self::halton(i as u32 + 1, 3);
            sequence.push([x - 0.5, y - 0.5]);
        }
        sequence
    }
    
    /// Halton 序列计算
    fn halton(mut index: u32, base: u32) -> f32 {
        let mut result = 0.0;
        let mut f = 1.0 / base as f32;
        while index > 0 {
            result += f * (index % base) as f32;
            index /= base;
            f /= base as f32;
        }
        result
    }
    
    /// 获取当前帧的抖动偏移
    pub fn get_jitter(&self) -> [f32; 2] {
        self.jitter_sequence[self.frame_index as usize % self.jitter_sequence.len()]
    }
    
    /// 推进到下一帧
    pub fn advance_frame(&mut self) {
        self.frame_index = self.frame_index.wrapping_add(1);
        self.current_history = 1 - self.current_history;
    }
    
    /// 确保历史纹理已创建
    pub fn ensure_history_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let need_recreate = self.history_texture_a.as_ref().map_or(true, |t| {
            t.size().width != width || t.size().height != height
        });
        
        if need_recreate {
            let create_texture = |label: &str| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
            };
            
            self.history_texture_a = Some(create_texture("TAA History A"));
            self.history_texture_b = Some(create_texture("TAA History B"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fxaa_quality_thresholds() {
        assert!(FxaaQuality::Low.edge_threshold() > FxaaQuality::High.edge_threshold());
        assert!(FxaaQuality::Low.iterations() < FxaaQuality::High.iterations());
    }
    
    #[test]
    fn test_halton_sequence() {
        let sequence = TaaPass::generate_halton_sequence(8);
        assert_eq!(sequence.len(), 8);
        
        // 验证值在 [-0.5, 0.5] 范围内
        for jitter in &sequence {
            assert!(jitter[0] >= -0.5 && jitter[0] <= 0.5);
            assert!(jitter[1] >= -0.5 && jitter[1] <= 0.5);
        }
    }
}
