//! WGPU 管线创建
//!
//! 包含渲染管线的创建和管理。

use super::types::{Instance, UiInstance, Vertex};

/// 管线构建器
///
/// 用于创建各种渲染管线。
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// 创建 2D 精灵渲染管线
    pub fn create_sprite_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniform_bgl: &wgpu::BindGroupLayout,
        texture_bgl: &wgpu::BindGroupLayout,
        lights_bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[uniform_bgl, texture_bgl, lights_bgl],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    Vertex::vertex_buffer_layout(),
                    Instance::vertex_buffer_layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    /// 创建 UI 渲染管线
    pub fn create_ui_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniform_bgl: &wgpu::BindGroupLayout,
        texture_bgl: &wgpu::BindGroupLayout,
        lights_bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(UI_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[uniform_bgl, texture_bgl, lights_bgl],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "ui_vs",
                buffers: &[
                    Vertex::vertex_buffer_layout(),
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<UiInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: 8,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: 20,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: 24,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 40,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 56,
                                shader_location: 7,
                                format: wgpu::VertexFormat::Float32,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "ui_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    /// 创建 Uniform 绑定组布局
    pub fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(16),
                },
                count: None,
            }],
        })
    }

    /// 创建纹理绑定组布局
    pub fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture BGL"),
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
        })
    }

    /// 创建光源绑定组布局
    pub fn create_lights_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        })
    }
}

/// 精灵着色器
const SPRITE_SHADER: &str = r#"
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
    @location(13) i_chunk: u32,
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
    
    var light_accum = vec3<f32>(0.1, 0.1, 0.1);
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

/// UI 着色器
const UI_SHADER: &str = r#"
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

@vertex 
fn ui_vs(
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

@fragment 
fn ui_fs(
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
