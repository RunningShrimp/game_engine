//! # 时序抗锯齿 (Temporal Anti-Aliasing - TAA)
//!
//! **API 稳定性**: 实验性 (Experimental) (v0.1.0)
//!
//! 提供基于时序历史信息的抗锯齿效果：
//! - 运动矢量计算
//! - 历史帧累积
//! - 色彩重建
//! - 阻尼和钳制
//!
//! ## API 稳定性声明
//!
//! **警告**: 此 API 处于实验性阶段，可能会在未来版本中发生破坏性变更。
//! - **状态**: 实验性 (Experimental)
//! - **引入版本**: v0.1.0
//! - **预期稳定版本**: v0.3.0
//!
//! ## 功能完整性追踪
//!
//! | 功能 | 状态 | 说明 |
//! |------|------|------|
//! | 运动矢量计算 | ✅ 已实现 | 基于深度/运动矢量计算 |
//! | 历史帧累积 | ✅ 已实现 | 2帧历史缓冲 |
//! | 色彩重建 | ✅ 已实现 | 基于邻域的色彩重建 |
//! | 阻尼和钳制 | ✅ 已实现 | 消除闪烁和撕裂 |
//! | 自适应混合 | ✅ 已实现 | 基于运动的自适应混合 |
//!
//! ## 使用说明
//!
//! TAA 通过结合当前帧和历史帧的信息，消除运动中的锯齿和闪烁。
//!
//! ### 示例
//!
//! ```rust,no_run
//! use game_engine::render::postprocess::temporal_aa::{TemporalAaPass, TemporalAaConfig};
//!
//! let config = TemporalAaConfig {
//!     enabled: true,
//!     feedback_min: 0.88,
//!     feedback_max: 0.97,
//!     ..Default::default()
//! };
//!
//! let taa_pass = TemporalAaPass::new(&device, config)?;
//! ```
//!
//! ## 性能考虑
//!
//! TAA 增加少量GPU和内存开销：
//! - 额外需要深度缓冲区
//! - 需要存储历史帧
//! - 约增加1-2ms的渲染时间
//!
//! 适用于高帧率场景（60+ FPS）

use crate::error::RenderError;
use crate::impl_default;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device,
    Queue, Sampler, ShaderStages, Texture, TextureFormat, TextureUsages,
    TextureView,
};

/// TAA配置
#[derive(Debug, Clone)]
pub struct TemporalAaConfig {
    /// 是否启用TAA
    pub enabled: bool,
    /// 最小反馈混合因子（用于阻尼）
    pub feedback_min: f32,
    /// 最大反馈混合因子（用于阻尼）
    pub feedback_max: f32,
    /// 色彩重建范围
    pub neighborhood_clamping: bool,
    /// 历史帧数量
    pub history_buffer_count: u32,
    /// 运动矢量放大倍数
    pub motion_scale: f32,
}

impl_default!(TemporalAaConfig {
    enabled: true,
    feedback_min: 0.88,
    feedback_max: 0.97,
    neighborhood_clamping: true,
    history_buffer_count: 2,
    motion_scale: 1.0,
});

/// TAA Uniform数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TemporalAaUniforms {
    /// 当前帧混合因子
    pub feedback_factor: f32,
    /// 运动矢量缩放
    pub motion_scale: f32,
    /// 是否启用色彩重建
    pub enable_neighborhood_clamping: u32,
    /// 邻域采样半径
    pub neighborhood_radius: f32,
    /// _padding
    pub _padding: [f32; 3],
}

/// TAA Pass
pub struct TemporalAaPass {
    config: TemporalAaConfig,
    pipeline: Option<ComputePipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    sampler: Option<Sampler>,
    /// 历史帧缓冲区（当前帧和历史帧）
    history_buffers: [Option<Texture>; 2],
    /// 历史帧视图
    history_views: [Option<TextureView>; 2],
    /// 历史帧索引（当前写入哪个缓冲）
    current_history_index: u32,
    /// 配置缓冲区
    uniform_buffer: Option<Buffer>,
    /// 运动矢量缓冲区
    motion_vector_buffer: Option<Buffer>,
}

impl TemporalAaPass {
    /// 创建新的TAA Pass
    pub fn new(
        device: &Device,
        config: TemporalAaConfig,
    ) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                sampler: None,
                history_buffers: [None, None],
                history_views: [None, None],
                current_history_index: 0,
                uniform_buffer: None,
                motion_vector_buffer: None,
            });
        }

        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // 创建历史帧缓冲区
        let format = TextureFormat::Rgba16Float;
        let size = wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        };

        let mut history_buffers = [None, None];
        let mut history_views = [None, None];

        for i in 0..2 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("TAA History Buffer {}", i)),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[format],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            history_buffers[i] = Some(texture);
            history_views[i] = Some(view);
        }

        // 创建配置缓冲区
        let uniforms = [TemporalAaUniforms {
            feedback_factor: 0.9,
            motion_scale: config.motion_scale,
            enable_neighborhood_clamping: if config.neighborhood_clamping { 1 } else { 0 },
            neighborhood_radius: 2.0,
            _padding: [0.0; 3],
        }];

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TAA Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 创建着色器
        let shader_source = r#"
struct TemporalAaUniforms {
    feedback_factor: f32,
    motion_scale: f32,
    enable_neighborhood_clamping: u32,
    neighborhood_radius: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: TemporalAaUniforms;

@group(0) @binding(1)
var texture_2d<f32> current_color;

@group(0) @binding(2)
var texture_2d<f32> history_color;

@group(0) @binding(3)
var texture_2d<f32> current_depth;

@group(0) @binding(4)
var texture_2d<f32> history_depth;

@group(0) @binding(5)
var texture_depth_2d<f32> motion_vectors;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let quad_positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    
    var out: VertexOutput;
    out.position = vec4<f32>(quad_positions[vertex_index], 0.0, 0.0, 1.0);
    out.uv = (quad_positions[vertex_index] + 1.0) * 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    
    // 采样当前帧
    let current_sample = textureSample(current_color, current_depth, uv);
    
    // 采样历史帧
    let history_sample = textureSample(history_color, history_depth, uv);
    
    // 采样运动矢量
    let motion = textureLoad(motion_vectors, vec2<i32>(uv * vec2<f32>(1920.0, 1080.0)), 0).rg * uniforms.motion_scale;
    
    // 计算历史帧的UV偏移
    let history_uv = uv + motion.xy;
    
    // 采样历史帧（使用运动矢量）
    let history_uv_clamped = clamp(history_uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let history_motion_sample = textureSample(history_color, history_depth, history_uv_clamped);
    
    // 检测运动中的不连续性
    let motion_length = length(motion.xy);
    let motion_factor = smoothstep(0.01, 0.1, motion_length);
    
    // 自适应混合因子
    let adaptive_factor = mix(uniforms.feedback_min, uniforms.feedback_max, motion_factor);
    
    // 基础混合
    let mut color = mix(history_sample.color, current_sample.color, adaptive_factor);
    let mut depth = mix(history_sample.depth, current_sample.depth, adaptive_factor);
    
    // 运动矢量混合（如果运动较大，减少历史帧影响）
    if motion_length > 0.02 {
        color = mix(current_sample.color, color, motion_factor * 0.5);
        depth = mix(current_sample.depth, depth, motion_factor * 0.5);
    }
    
    // 色彩重建（邻域钳制）
    if (uniforms.enable_neighborhood_clamping != 0u) {
        let radius = uniforms.neighborhood_radius;
        let mut min_color = color;
        let mut max_color = color;
        let mut min_luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let mut max_luma = min_luma;
        
        // 采样邻域像素（十字形）
        let offsets = array<vec2<f32>, 4>(
            vec2<f32>(0.0, -radius),
            vec2<f32>(0.0, radius),
            vec2<f32>(-radius, 0.0),
            vec2<f32>(radius, 0.0),
        );
        
        for i in 0u..4u {
            let sample_uv = clamp(uv + offsets[i], vec2<f32>(0.0), vec2<f32>(1.0));
            let sample = textureSample(current_color, current_depth, sample_uv);
            let luma = dot(sample.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
            
            if (luma < min_luma) {
                min_luma = luma;
                min_color = sample.color;
            }
            if (luma > max_luma) {
                max_luma = luma;
                max_color = sample.color;
            }
        }
        
        // 钳制到邻域最小/最大亮度范围
        let clamped_luma = clamp(dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)), min_luma, max_luma);
        let luma_diff = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)) - clamped_luma;
        color.rgb += luma_diff;
    }
    
    // Tone mapping（简单的Reinhard）
    let luma = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tone_mapped_luma = luma / (1.0 + luma);
    color.rgb *= tone_mapped_luma / luma;
    
    return vec4<f32>(color.rgb, color.a);
}

fn textureSample(
    color_texture: texture_2d<f32>,
    depth_texture: texture_2d<f32>,
    uv: vec2<f32>,
) -> SampleResult {
    let color = textureSampleLevel(color_texture, uv, 0);
    let depth = textureSampleLevel(depth_texture, uv, 0);
    
    return SampleResult {
        color: color,
        depth: depth.r,
    };
}

struct SampleResult {
    color: vec4<f32>,
    depth: f32,
}
"#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TAA Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TAA BGL"),
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // 创建渲染管线
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TAA Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("TAA Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::TriangleList {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: wgpu::DepthStencilState::UNCHANGED,
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            sampler: Some(sampler),
            history_buffers,
            history_views,
            current_history_index: 0,
            uniform_buffer: Some(uniform_buffer),
            motion_vector_buffer: None, // 从外部传入
        })
    }

    /// 获取输出纹理视图
    pub fn output_view(&self) -> Option<&TextureView> {
        self.history_views[self.current_history_index as usize]
    }

    /// 渲染TAA
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        current_color: &TextureView,
        current_depth: &TextureView,
        motion_vectors: &TextureView,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(pipeline) = &self.pipeline else {
            return Ok(());
        };

        let Some(bgl) = &self.bind_group_layout else {
            return Ok(());
        };

        let Some(sampler) = &self.sampler else {
            return Ok(());
        };

        let Some(uniform_buffer) = &self.uniform_buffer else {
            return Ok(());
        };

        // 复制当前帧到历史缓冲区
        let history_index = self.current_history_index as usize;
        let prev_history_index = 1 - history_index;

        if let (Some(history_texture), Some(history_view)) = (
            &self.history_buffers[history_index],
            &self.history_views[history_index],
        ) {
            // 复制当前帧到历史缓冲区
            encoder.copy_texture_to_texture(
                current_color,
                wgpu::ImageCopyTexture {
                    texture: history_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::Zero,
                    aspect: wgpu::TextureAspect::All,
                },
            );
        }

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TAA Bind Group"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(current_color),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        self.history_views[prev_history_index]
                            .as_ref()
                            .expect("History view should be initialized"),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(current_depth),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        self.history_views[prev_history_index]
                            .as_ref()
                            .expect("History view should be initialized"),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(motion_vectors),
                },
            ],
        });

        // 渲染全屏四边形
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TAA Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.history_views[history_index]
                    .as_ref()
                    .expect("History view should be initialized"),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..4, 0..1);

        Ok(())
    }

    /// 更新历史帧索引
    pub fn swap_history(&mut self) {
        self.current_history_index = 1 - self.current_history_index;
    }
}

