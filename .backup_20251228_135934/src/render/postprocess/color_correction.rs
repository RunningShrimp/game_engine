use wgpu;

/// 色彩校正 Uniform 数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorCorrectionUniforms {
    /// 亮度调整 (-1.0 到 1.0)
    pub brightness: f32,
    /// 对比度调整 (0.0 到 2.0)
    pub contrast: f32,
    /// 饱和度调整 (0.0 到 2.0)
    pub saturation: f32,
    /// 色调偏移 (-180.0 到 180.0 度)
    pub hue_shift: f32,
    /// 色差强度 (0.0 到 1.0)
    pub chromatic_aberration: f32,
    /// 暗角强度 (0.0 到 1.0)
    pub vignette_intensity: f32,
    /// 暗角圆度 (0.0 到 1.0)
    pub vignette_roundness: f32,
    /// 填充
    pub _pad: f32,
}

/// 色彩校正渲染通道
pub struct ColorCorrectionPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    output_format: wgpu::TextureFormat,
}

impl ColorCorrectionPass {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Color Correction Shader"),
            source: wgpu::ShaderSource::Wgsl(COLOR_CORRECTION_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Correction BGL"),
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
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Correction Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Color Correction Pipeline"),
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
            label: Some("Color Correction Uniform Buffer"),
            size: std::mem::size_of::<ColorCorrectionUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Color Correction Sampler"),
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
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        brightness: f32,
        contrast: f32,
        saturation: f32,
        hue_shift: f32,
        chromatic_aberration: f32,
        vignette_intensity: f32,
        vignette_roundness: f32,
    ) {
        let uniforms = ColorCorrectionUniforms {
            brightness: brightness.clamp(-1.0, 1.0),
            contrast: contrast.clamp(0.0, 2.0),
            saturation: saturation.clamp(0.0, 2.0),
            hue_shift: hue_shift.clamp(-180.0, 180.0),
            chromatic_aberration: chromatic_aberration.clamp(0.0, 1.0),
            vignette_intensity: vignette_intensity.clamp(0.0, 1.0),
            vignette_roundness: vignette_roundness.clamp(0.0, 1.0),
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Correction BG"),
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
            label: Some("Color Correction Pass"),
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

const COLOR_CORRECTION_SHADER: &str = r#"
struct ColorCorrectionUniforms {
    brightness: f32,
    contrast: f32,
    saturation: f32,
    hue_shift: f32,
    chromatic_aberration: f32,
    vignette_intensity: f32,
    vignette_roundness: f32,
    _pad: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: ColorCorrectionUniforms;

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

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(max(rgb.r, rgb.g), rgb.b);
    let cmin = min(min(rgb.r, rgb.g), rgb.b);
    let delta = cmax - cmin;
    
    var h = 0.0;
    if (delta > 0.0) {
        if (cmax == rgb.r) {
            h = 60.0 * mod((rgb.g - rgb.b) / delta + 6.0, 6.0);
        } else if (cmax == rgb.g) {
            h = 60.0 * ((rgb.b - rgb.r) / delta + 2.0);
        } else {
            h = 60.0 * ((rgb.r - rgb.g) / delta + 4.0);
        }
    }
    
    let s = if (cmax > 0.0) { delta / cmax } else { 0.0 };
    let v = cmax;
    
    return vec3<f32>(h, s, v);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let c = hsv.z * hsv.y;
    let x = c * (1.0 - abs(mod(hsv.x / 60.0, 2.0) - 1.0));
    let m = hsv.z - c;
    
    var rgb = vec3<f32>(0.0);
    
    if (hsv.x < 60.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (hsv.x < 120.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (hsv.x < 180.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (hsv.x < 240.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (hsv.x < 300.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + vec3<f32>(m);
}

fn apply_vignette(color: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);
    let radius = 0.5 * uniforms.vignette_roundness;
    let vignette = smoothstep(radius, radius * 0.3, dist);
    let factor = mix(1.0 - uniforms.vignette_intensity, 1.0, vignette);
    return color * factor;
}

fn apply_chromatic_aberration(uv: vec2<f32>) -> vec3<f32> {
    if (uniforms.chromatic_aberration <= 0.0) {
        return textureSample(input_texture, input_sampler, uv).rgb;
    }
    
    let center = vec2<f32>(0.5, 0.5);
    let dir = normalize(uv - center);
    let dist = distance(uv, center);
    let offset = dir * dist * uniforms.chromatic_aberration * 0.02;
    
    let r = textureSample(input_texture, input_sampler, uv + offset).r;
    let g = textureSample(input_texture, input_sampler, uv).g;
    let b = textureSample(input_texture, input_sampler, uv - offset).b;
    
    return vec3<f32>(r, g, b);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = apply_chromatic_aberration(in.uv);
    
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    
    color = mix(vec3<f32>(luminance), color, uniforms.saturation);
    
    color = (color - 0.5) * uniforms.contrast + 0.5;
    
    color = color + uniforms.brightness;
    
    var hsv = rgb_to_hsv(color);
    hsv.x = mod(hsv.x + uniforms.hue_shift, 360.0);
    color = hsv_to_rgb(hsv);
    
    color = apply_vignette(color, in.uv);
    
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    
    return vec4<f32>(color, 1.0);
}
"#;
