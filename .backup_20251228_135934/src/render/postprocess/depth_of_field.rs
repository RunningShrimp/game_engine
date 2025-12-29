use wgpu;

/// 景深效果 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DepthOfFieldUniforms {
    /// 屏幕尺寸
    pub screen_size: [f32; 2],
    /// 像素大小
    pub pixel_size: [f32; 2],
    /// 焦点距离 (0.0 - 1.0)
    pub focus_distance: f32,
    /// 光圈大小 (模糊范围)
    pub aperture: f32,
    /// 近平面模糊
    pub near_blur: f32,
    /// 远平面模糊
    pub far_blur: f32,
    /// 最大模糊半径
    pub max_blur_radius: f32,
    /// 填充
    pub _pad: f32,
}

/// 景深渲染通道
pub struct DepthOfFieldPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    output_format: wgpu::TextureFormat,
}

impl DepthOfFieldPass {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Depth of Field Shader"),
            source: wgpu::ShaderSource::Wgsl(DOF_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DOF BGL"),
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
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("DOF Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("DOF Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DOF Uniform Buffer"),
            size: std::mem::size_of::<DepthOfFieldUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("DOF Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            sampler,
            output_format,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        focus_distance: f32,
        aperture: f32,
        near_blur: f32,
        far_blur: f32,
        max_blur_radius: f32,
    ) {
        let uniforms = DepthOfFieldUniforms {
            screen_size: [width as f32, height as f32],
            pixel_size: [1.0 / width as f32, 1.0 / height as f32],
            focus_distance: focus_distance.clamp(0.0, 1.0),
            aperture: aperture.clamp(0.0, 10.0),
            near_blur: near_blur.clamp(0.0, 1.0),
            far_blur: far_blur.clamp(0.0, 1.0),
            max_blur_radius: max_blur_radius.clamp(0.0, 20.0),
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DOF BG"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DOF Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
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

const DOF_SHADER: &str = r#"
struct DepthOfFieldUniforms {
    screen_size: vec2<f32>,
    pixel_size: vec2<f32>,
    focus_distance: f32,
    aperture: f32,
    near_blur: f32,
    far_blur: f32,
    max_blur_radius: f32,
    _pad: f32,
};

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var input_sampler: sampler;
@group(0) @binding(3) var<uniform> uniforms: DepthOfFieldUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

fn get_blur_radius(depth: f32) -> f32 {
    let dist = abs(depth - uniforms.focus_distance);
    
    if (depth < uniforms.focus_distance) {
        return dist * uniforms.near_blur * uniforms.aperture;
    } else {
        return dist * uniforms.far_blur * uniforms.aperture;
    }
}

fn gaussian_weight(offset: f32, sigma: f32) -> f32 {
    let sigma2 = sigma * sigma;
    return exp(-(offset * offset) / (2.0 * sigma2)) / (sqrt(2.0 * 3.14159265) * sigma);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_texture, input_sampler, in.uv);
    let blur_radius = get_blur_radius(depth);
    
    if (blur_radius < 0.5) {
        return textureSample(color_texture, input_sampler, in.uv);
    }
    
    let radius = min(blur_radius, uniforms.max_blur_radius);
    let sigma = radius * 0.5;
    var result = vec3<f32>(0.0);
    var total_weight = 0.0;
    
    let taps = i32(radius * 2.0 + 1.0);
    let half_taps = taps / 2;
    
    for (var y = -half_taps; y <= half_taps; y++) {
        for (var x = -half_taps; x <= half_taps; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * uniforms.pixel_size;
            let sample_uv = in.uv + offset;
            
            if (sample_uv.x >= 0.0 && sample_uv.x <= 1.0 && 
                sample_uv.y >= 0.0 && sample_uv.y <= 1.0) {
                let weight = gaussian_weight(length(vec2<f32>(f32(x), f32(y))), sigma);
                result += textureSample(color_texture, input_sampler, sample_uv).rgb * weight;
                total_weight += weight;
            }
        }
    }
    
    return vec4<f32>(result / total_weight, 1.0);
}
"#;
