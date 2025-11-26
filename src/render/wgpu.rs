use winit::window::Window;
use wgpu::util::DeviceExt;

use crate::render::mesh::Vertex3D;


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],
    pub scale: [f32; 2],
    pub rot: f32,
    pub target: u32,
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub layer: f32,
    pub tex_index: u32,
    pub normal_tex_index: u32,
    pub msdf: f32,
    pub px_range: f32,
    pub _pad: [f32; 3], // Padding to align to 16 bytes if needed, or just ensure total size is correct.
    // Current size: 2*4 + 2*4 + 4 + 4 + 4*4 + 2*4 + 2*4 + 4 + 4 + 4 + 4 + 4 = 8+8+4+4+16+8+8+4+4+4+4+4 = 76 bytes.
    // Let's check alignment. 76 is not a multiple of 16. 80 is.
    // _pad: f32 is 4 bytes. Total 80.
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub radius: f32,
    pub stroke_width: f32,
    pub color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub rotation: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    screen_size: [f32; 2],
    scale_factor: f32,
    _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms3D {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    _pad1: [f32; 32],
    _pad2: [f32; 12],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPointLight {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
    pub falloff: f32,
    pub _pad: [f32; 2], // Align to 16 bytes (vec4)
}

pub struct WgpuRenderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_buffer: wgpu::Buffer,
    instance_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bgl: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    textures_size: Vec<[u32; 2]>,
    layer_ranges: Vec<(u32, u32)>,
    draw_groups: Vec<DrawGroup>,
    scale_factor: f32,
    scissor: Option<[u32;4]>,
    group_cache: Vec<(f32, usize, u32)>,
    groups_dirty: bool,
    ui_pipeline: wgpu::RenderPipeline,
    ui_instance_buffer: wgpu::Buffer,
    ui_count: u32,
    commands: Vec<crate::render::graph::RenderCommand>,
    pub offscreen_views: std::collections::HashMap<u32, wgpu::TextureView>,
    
    // Lighting
    lights_buffer: wgpu::Buffer,
    lights_bind_group: wgpu::BindGroup,
    pub lights: Vec<GpuPointLight>,

    // 3D
    pub depth_texture: wgpu::TextureView,
    pub pipeline_3d: wgpu::RenderPipeline,
    pub uniform_buffer_3d: wgpu::Buffer,
    pub uniform_bind_group_3d: wgpu::BindGroup,
    pub model_uniform_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
}

pub struct DrawGroup {
    pub start: u32,
    pub end: u32,
    pub tex_idx: usize,
    pub layer: f32,
    pub scissor: Option<[u32;4]>,
}

impl<'a> WgpuRenderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let present_mode = caps.present_modes[0];
        let alpha_mode = caps.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let vs_src = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    scale_factor: f32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct PointLight {
    pos: vec2<f32>,
    color: vec3<f32>,
    radius: f32,
    intensity: f32,
    falloff: f32,
    _pad: vec2<f32>,
};
@group(2) @binding(0) var<storage, read> lights: array<PointLight>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) msdf: f32,
    @location(3) pxr: f32,
    @location(4) world_pos: vec2<f32>,
    @location(5) normal_idx: u32,
};

@vertex
fn vs(
    @location(0) v_pos: vec2<f32>,
    @location(1) i_pos: vec2<f32>,
    @location(2) i_scale: vec2<f32>,
    @location(3) i_rot: f32,
    @location(4) i_target: u32,
    @location(5) i_color: vec4<f32>,
    @location(6) i_uv_offset: vec2<f32>,
    @location(7) i_uv_scale: vec2<f32>,
    @location(8) i_layer: f32,
    @location(9) i_tex_index: u32,
    @location(10) i_normal_idx: u32,
    @location(11) i_msdf: f32,
    @location(12) i_pxr: f32,
) -> VsOut {
    let c = cos(i_rot);
    let s = sin(i_rot);
    let rot = mat2x2<f32>(c, -s, s, c);
    let local = rot * (v_pos * i_scale);
    let world = local + i_pos;
    let ndc = vec2<f32>( (world.x / uniforms.screen_size.x) * 2.0 - 1.0,
                         -(world.y / uniforms.screen_size.y) * 2.0 + 1.0);
    let base_uv = v_pos + vec2<f32>(0.5, 0.5);
    let uv = i_uv_offset + base_uv * i_uv_scale;
    return VsOut(vec4<f32>(ndc, 0.0, 1.0), i_color.xyz, uv, i_msdf, i_pxr, world, i_normal_idx);
}

@fragment
fn fs(@location(0) color: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) msdf: f32, @location(3) pxr: f32, @location(4) world_pos: vec2<f32>, @location(5) normal_idx: u32) -> @location(0) vec4<f32> {
    let texc = textureSample(tex, samp, uv);
    
    var light_accum = vec3<f32>(0.1, 0.1, 0.1); // Ambient
    let num_lights = arrayLength(&lights);
    for (var i = 0u; i < num_lights; i++) {
        let light = lights[i];
        let dist = distance(world_pos, light.pos);
        if (dist < light.radius) {
            let att = 1.0 - smoothstep(0.0, light.radius, dist);
            light_accum += light.color * light.intensity * att;
        }
    }
    
    let final_color = texc.rgb * color * light_accum;

    if (msdf < 0.0) {
        let p = uv - vec2<f32>(0.5, 0.5);
        let r = clamp(pxr, 0.0, 0.45);
        let b = vec2<f32>(0.5 - r, 0.5 - r);
        let q = abs(p) - b;
        let k = max(q, vec2<f32>(0.0, 0.0));
        let dist = length(k) - r;
        let w = fwidth(dist);
        let alpha = smoothstep(0.0 - w, 0.0 + w, -dist);
        return vec4<f32>(final_color, alpha);
    } else if (msdf > 0.5) {
        let r = texc.r; let g = texc.g; let b = texc.b;
        let sd = max(min(r, g), min(max(r, g), b));
        let w = fwidth(sd) * max(pxr * uniforms.scale_factor, 0.0001);
        let alpha = smoothstep(0.5 - w, 0.5 + w, sd);
        return vec4<f32>(final_color, alpha);
    } else {
        return vec4<f32>(final_color, texc.a);
    }
}
"#;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(vs_src.into()),
        });
        let ui_wgsl = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    scale_factor: f32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
struct UiOut { 
    @builtin(position) pos: vec4<f32>, 
    @location(0) uv: vec2<f32>, 
    @location(1) color: vec4<f32>, 
    @location(2) radius: f32,
    @location(3) stroke_width: f32,
    @location(4) stroke_color: vec4<f32>,
    @location(5) size: vec2<f32>,
};
@vertex fn ui_vs(
    @location(0) v_pos: vec2<f32>, 
    @location(1) i_pos: vec2<f32>, 
    @location(2) i_size: vec2<f32>, 
    @location(3) i_radius: f32, 
    @location(4) i_stroke_width: f32,
    @location(5) i_color: vec4<f32>,
    @location(6) i_stroke_color: vec4<f32>,
    @location(7) i_rotation: f32
) -> UiOut {
    let c = cos(i_rotation);
    let s = sin(i_rotation);
    let rot_mat = mat2x2<f32>(c, -s, s, c);
    let local = rot_mat * (v_pos * i_size);
    let world = local + i_pos;
    let ndc = vec2<f32>( (world.x / uniforms.screen_size.x) * 2.0 - 1.0,
                         -(world.y / uniforms.screen_size.y) * 2.0 + 1.0);
    let uv = v_pos + vec2<f32>(0.5, 0.5);
    return UiOut(vec4<f32>(ndc, 0.0, 1.0), uv, i_color, i_radius, i_stroke_width, i_stroke_color, i_size);
}
@fragment fn ui_fs(
    @location(0) uv: vec2<f32>, 
    @location(1) color: vec4<f32>, 
    @location(2) radius: f32,
    @location(3) stroke_width: f32,
    @location(4) stroke_color: vec4<f32>,
    @location(5) size: vec2<f32>
) -> @location(0) vec4<f32> {
    let p = (uv - 0.5) * size;
    let r = radius;
    let b = size * 0.5 - vec2<f32>(r, r);
    let q = abs(p) - b;
    let k = max(q, vec2<f32>(0.0, 0.0));
    let dist = length(k) - r;
    let w = fwidth(dist);
    
    let inside = smoothstep(0.0, -w, dist);
    let inner_edge = smoothstep(-stroke_width, -stroke_width - w, dist);
    let border = inside - inner_edge;
    
    let out_color = mix(color, stroke_color, border);
    return vec4<f32>(out_color.rgb, out_color.a * inside);
}
"#;
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(ui_wgsl.into()) });
        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<ScreenUniform>() as u64),
                },
                count: None,
            }],
        });
        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
            ],
        });
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lights Buffer"),
            size: 1024 * std::mem::size_of::<GpuPointLight>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lights BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lights BG"),
            layout: &lights_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bgl, &texture_bgl, &lights_bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 20, shader_location: 4, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 24, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 40, shader_location: 6, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 48, shader_location: 7, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 56, shader_location: 8, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 60, shader_location: 9, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 64, shader_location: 10, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 68, shader_location: 11, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 72, shader_location: 12, format: wgpu::VertexFormat::Float32 },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: "ui_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<UiInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 20, shader_location: 4, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 24, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 40, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 56, shader_location: 7, format: wgpu::VertexFormat::Float32 },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState { module: &ui_shader, entry_point: "ui_fs", targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let quad: [Vertex; 6] = [
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [ 0.5, -0.5] },
            Vertex { pos: [ 0.5,  0.5] },
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [ 0.5,  0.5] },
            Vertex { pos: [-0.5,  0.5] },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&quad),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * std::mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ui_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 1024 * std::mem::size_of::<UiInstance>() as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        // screen uniform
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&ScreenUniform { screen_size: [size.width as f32, size.height as f32], scale_factor: 1.0, _pad: 0.0 }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &uniform_buffer, offset: 0, size: None }) }],
        });

        // checkerboard texture
        let tex_size = 256u32;
        let mut data = vec![0u8; (tex_size * tex_size * 4) as usize];
        for y in 0..tex_size {
            for x in 0..tex_size {
                let idx = ((y * tex_size + x) * 4) as usize;
                let c = if ((x / 32) % 2) ^ ((y / 32) % 2) == 0 { 220 } else { 60 };
                data[idx] = c; // r
                data[idx + 1] = c; // g
                data[idx + 2] = c; // b
                data[idx + 3] = 255; // a
            }
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * tex_size), rows_per_image: Some(tex_size) },
            wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // --- 3D Setup ---
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
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

        let shader_3d = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct ModelUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
};
@group(1) @binding(0) var<uniform> mesh_data: ModelUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.normal = model.normal;
    out.clip_position = uniforms.view_proj * mesh_data.model * vec4<f32>(model.pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let color = vec3<f32>(1.0, 1.0, 1.0) * (diffuse + 0.1);
    return vec4<f32>(color, 1.0);
}
"#)),
        });

        let uniform_buffer_3d = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("3D Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms3D { view_proj: [[0.0; 4]; 4] }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout_3d = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("3D Uniform Bind Group Layout"),
        });

        let uniform_bind_group_3d = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_3d,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer_3d.as_entire_binding(),
            }],
            label: Some("3D Uniform Bind Group"),
        });

        let model_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Uniform Buffer"),
            size: (256 * 1000) as wgpu::BufferAddress, // Support 1000 meshes
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(256),
                },
                count: None,
            }],
            label: Some("Model Bind Group Layout"),
        });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &model_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(256),
                }),
            }],
            label: Some("Model Bind Group"),
        });

        let pipeline_layout_3d = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout_3d, &model_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_3d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Pipeline"),
            layout: Some(&pipeline_layout_3d),
            vertex: wgpu::VertexState {
                module: &shader_3d,
                entry_point: "vs_main",
                buffers: &[Vertex3D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_3d,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            instance_buffer,
            vertex_count: 6,
            index_buffer,
            instance_count: 0,
            uniform_buffer,
            uniform_bind_group,
            texture_bgl,
            texture_bind_groups: vec![texture_bind_group],
            textures_size: vec![[tex_size, tex_size]],
            layer_ranges: Vec::new(),
            draw_groups: Vec::new(),
            scale_factor: 1.0,
            scissor: None,
            group_cache: Vec::new(),
            groups_dirty: true,
            ui_pipeline,
            ui_instance_buffer,
            ui_count: 0,
            commands: Vec::new(),
            offscreen_views: std::collections::HashMap::new(),
            lights_buffer,
            lights_bind_group,
            lights: Vec::new(),
            depth_texture: depth_view,
            pipeline_3d,
            uniform_buffer_3d,
            uniform_bind_group_3d,
            model_uniform_buffer,
            model_bind_group,
        }
    }

    pub fn create_offscreen_target(&mut self, id: u32, width: u32, height: u32) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Offscreen Target {}", id)),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.offscreen_views.insert(id, view);
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);

            // Resize depth texture
            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.depth_texture = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn render(
        &mut self, 
        instances: &[Instance], 
        meshes: &[(crate::render::mesh::GpuMesh, crate::ecs::Transform)], 
        camera_view_proj: [[f32; 4]; 4],
        mut egui_renderer: Option<&mut egui_wgpu::Renderer>, 
        egui_shapes: &[egui::ClippedPrimitive], 
        egui_pixels_per_point: f32
    ) {
        self.update_instances_grouped(&mut instances.to_vec());

        // Update 3D uniforms
        self.queue.write_buffer(&self.uniform_buffer_3d, 0, bytemuck::cast_slice(&[Uniforms3D { view_proj: camera_view_proj }]));
        
        // Update Model Uniforms
        // Note: This is a simple implementation. For production, use a staging buffer or write_buffer_with.
        // Also, we assume meshes.len() < 1000.
        let mut model_data = Vec::with_capacity(meshes.len());
        for (_, transform) in meshes {
            let mat = glam::Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
            model_data.push(ModelUniform {
                model: mat.to_cols_array_2d(),
                color: [1.0, 1.0, 1.0, 1.0], // Placeholder color
                _pad1: [0.0; 32],
                _pad2: [0.0; 12],
            });
        }
        if !model_data.is_empty() {
             self.queue.write_buffer(&self.model_uniform_buffer, 0, bytemuck::cast_slice(&model_data));
        }
        
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().unwrap()
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        if let Some(renderer) = egui_renderer.as_mut() {
             let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: egui_pixels_per_point,
            };
            renderer.update_buffers(&self.device, &self.queue, &mut encoder, egui_shapes, &screen_desc);
        }

        let graph = crate::render::graph::build_commands(instances);
        let mut cleared_targets = std::collections::HashSet::new();
        let mut i = 0;
        
        while i < graph.commands.len() {
            if let crate::render::graph::RenderCommand::SetTarget(t) = &graph.commands[i] {
                let (target_view, target_id) = match t {
                    crate::render::graph::Target::Main | crate::render::graph::Target::Ui => (&view, 0),
                    crate::render::graph::Target::Offscreen(id) => (self.offscreen_views.get(id).unwrap_or(&view), *id + 1),
                };
                
                let load_op = if cleared_targets.contains(&target_id) {
                    wgpu::LoadOp::Load
                } else {
                    cleared_targets.insert(target_id);
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.04, b: 0.06, a: 1.0 })
                };

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_view,
                            resolve_target: None,
                            ops: wgpu::Operations { load: load_op, store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: if target_id == 0 { // Main target only
                            Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &self.depth_texture,
                                depth_ops: Some(wgpu::Operations {
                                    load: if load_op == wgpu::LoadOp::Load { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(1.0) },
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            })
                        } else {
                            None
                        },
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    if target_id == 0 {
                        // Draw 3D Meshes
                        if !meshes.is_empty() {
                            rpass.set_pipeline(&self.pipeline_3d);
                            rpass.set_bind_group(0, &self.uniform_bind_group_3d, &[]);
                            for (i, (mesh, _)) in meshes.iter().enumerate() {
                                let offset = (i * 256) as u32;
                                rpass.set_bind_group(1, &self.model_bind_group, &[offset]);
                                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                            }
                        }
                    }

                    rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    
                    i += 1;
                    while i < graph.commands.len() {
                        match &graph.commands[i] {
                            crate::render::graph::RenderCommand::SetTarget(_) => break,
                            crate::render::graph::RenderCommand::Draw { start, end, tex_idx, scissor } => {
                                rpass.set_pipeline(&self.pipeline);
                                rpass.set_bind_group(1, &self.texture_bind_groups[*tex_idx], &[]);
                                rpass.set_bind_group(2, &self.lights_bind_group, &[]);
                                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                                rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                                if let Some([x,y,w,h]) = scissor { rpass.set_scissor_rect(*x, *y, *w, *h); }
                                if end > start { rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); rpass.draw_indexed(0..self.vertex_count, 0, *start..*end); }
                            }
                            crate::render::graph::RenderCommand::DrawUi { count: _ } => {
                                rpass.set_pipeline(&self.ui_pipeline);
                                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                                rpass.set_vertex_buffer(1, self.ui_instance_buffer.slice(..));
                                rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                                rpass.draw_indexed(0..self.vertex_count, 0, 0..self.ui_count);
                            }
                        }
                        i += 1;
                    }
                    
                    if target_id == 0 {
                        if let Some(renderer) = egui_renderer.as_mut() {
                             let screen_desc = egui_wgpu::ScreenDescriptor {
                                size_in_pixels: [self.config.width, self.config.height],
                                pixels_per_point: egui_pixels_per_point,
                            };
                            renderer.render(&mut rpass, egui_shapes, &screen_desc);
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    pub fn set_lights(&mut self, lights: Vec<GpuPointLight>) {
        self.update_lights(&lights);
    }
    pub fn gpu_timings_ms(&self) -> Option<(f32, f32)> { None }
    pub fn draw_stats(&self) -> (u32, u32) { (0, 0) }
    pub fn pass_count(&self) -> u32 { 1 }
    pub fn stage_timings_ms(&self) -> (Option<f32>, Option<f32>, Option<f32>) { (None, None, None) }
    pub fn offscreen_timing_ms(&self) -> Option<f32> { None }

    pub fn update_instances(&mut self, instances: &[Instance]) {
        self.instance_count = instances.len() as u32;
        self.layer_ranges.clear();
        self.layer_ranges.push((0, self.instance_count));
        self.draw_groups.clear();
        self.draw_groups.push(DrawGroup { start: 0, end: self.instance_count, tex_idx: 0, layer: 0.0, scissor: None });
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        self.group_cache.clear();
        self.groups_dirty = false;
    }

    pub fn update_instances_grouped(&mut self, instances: &mut [Instance]) {
        instances.sort_by(|a, b| a.layer.partial_cmp(&b.layer).unwrap_or(std::cmp::Ordering::Equal));
        self.instance_count = instances.len() as u32;
        self.layer_ranges.clear();
        // Build cache of (layer, tex_idx) for fast diff; if unchanged and not dirty, skip rebuilding groups
        let mut runs: Vec<(f32, usize, u32)> = Vec::new();
        if !instances.is_empty() {
            let mut cur_layer = instances[0].layer;
            let mut cur_tex = inst_tex_index(&instances[0]);
            let mut cnt: u32 = 0;
            for inst in instances.iter() {
                let t = inst_tex_index(inst);
                if inst.layer == cur_layer && t == cur_tex { cnt += 1; } else { runs.push((cur_layer, cur_tex, cnt)); cur_layer = inst.layer; cur_tex = t; cnt = 1; }
            }
            runs.push((cur_layer, cur_tex, cnt));
        }
        let need_rebuild = self.groups_dirty || runs.len() != self.group_cache.len() || runs != self.group_cache;
        if need_rebuild {
            // sliding-window增量重建：定位首个差异段，仅重建其后的分段
            let mut prefix = 0usize;
            let minl = std::cmp::min(runs.len(), self.group_cache.len());
            while prefix < minl && runs[prefix] == self.group_cache[prefix] { prefix += 1; }
            // 计算起始offset
            let mut start = if prefix > 0 { self.draw_groups.get(prefix - 1).map(|g| g.end).unwrap_or(0) } else { 0 };
            // 截断旧分段到前缀长度
            if self.draw_groups.len() > prefix { self.draw_groups.truncate(prefix); }
            if self.layer_ranges.len() > prefix { self.layer_ranges.truncate(prefix); }
            // 追加新分段，并尽量复用尾部分段的裁剪属性
            for i in prefix..runs.len() {
                let (layer, tex_idx, cnt) = runs[i];
                let end = start + cnt;
                let mut scissor = None;
                let suffix = {
                    let mut s = 0usize;
                    let minl = std::cmp::min(runs.len(), self.group_cache.len());
                    while s < minl && runs[runs.len()-1-s] == self.group_cache[self.group_cache.len()-1-s] { s += 1; }
                    s
                };
                if suffix > 0 {
                    let _new_ix_tail = (prefix..runs.len()).count();
                    let k = runs.len() - i;
                    if k <= suffix {
                        let old_ix = self.draw_groups.len().saturating_sub(k);
                        scissor = self.draw_groups.get(old_ix).and_then(|g| g.scissor);
                    }
                }
                self.layer_ranges.push((start, end));
                self.draw_groups.push(DrawGroup { start, end, tex_idx, layer, scissor });
                start = end;
            }
            self.group_cache = runs;
            self.groups_dirty = false;
        }
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }

    pub fn set_scale_factor(&mut self, scale: f32) { self.scale_factor = scale; }

    pub fn set_scissor(&mut self, rect: Option<[u32;4]>) { self.scissor = rect; }

    pub fn set_scissor_for_instances(&mut self, start: u32, end: u32, rect: Option<[u32;4]>) {
        for g in &mut self.draw_groups {
            if !(end <= g.start || start >= g.end) { g.scissor = rect; }
        }
    }

    pub fn mark_groups_dirty(&mut self) { self.groups_dirty = true; }

    pub fn load_texture_file(&mut self, path: &std::path::Path) -> Option<u32> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn load_texture_file_linear(&mut self, path: &std::path::Path) -> Option<u32> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn reload_texture_file_at(&mut self, index: u32, path: &std::path::Path, linear: bool) -> Option<()> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let format = if linear { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Rgba8UnormSrgb };
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = index as usize;
            if idx < self.texture_bind_groups.len() {
                self.texture_bind_groups[idx] = bg;
                self.textures_size[idx] = [w, h];
                return Some(());
            }
        }
        None
    }

    pub fn load_texture_from_bytes(&mut self, bytes: &[u8], is_linear: bool) -> Option<u32> {
        if let Ok(img) = image::load_from_memory(bytes) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let format = if is_linear { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Rgba8UnormSrgb };
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn update_screen(&mut self) {
        let u = ScreenUniform { screen_size: [self.size.width as f32, self.size.height as f32], scale_factor: self.scale_factor, _pad: 0.0 };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&u));
    }

    pub fn set_scissor_for_layer(&mut self, layer: f32, rect: Option<[u32;4]>) {
        for g in &mut self.draw_groups {
            if (g.layer - layer).abs() < f32::EPSILON { g.scissor = rect; }
        }
    }

    pub fn ui_set(&mut self, items: &[UiInstance]) {
        self.ui_count = items.len() as u32;
        if self.ui_count > 0 { self.queue.write_buffer(&self.ui_instance_buffer, 0, bytemuck::cast_slice(items)); }
    }

    pub fn set_graph(&mut self, graph: crate::render::graph::RenderGraph, instances: &[Instance]) {
        self.commands = graph.commands;
        self.instance_count = instances.len() as u32;
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }

    pub fn device(&self) -> &wgpu::Device { &self.device }
    pub fn queue(&self) -> &wgpu::Queue { &self.queue }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration { &self.config }
    
    pub fn update_lights(&mut self, lights: &[GpuPointLight]) {
        self.lights.clear();
        self.lights.extend_from_slice(lights);
        
        let mut data = vec![GpuPointLight {
            pos: [0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            radius: 0.0,
            intensity: 0.0,
            falloff: 0.0,
            _pad: [0.0, 0.0],
        }; 1024];
        
        for (i, light) in lights.iter().enumerate().take(1024) {
            data[i] = *light;
        }
        
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::cast_slice(&data));
    }
}

fn inst_tex_index(inst: &Instance) -> usize { inst.tex_index as usize }
