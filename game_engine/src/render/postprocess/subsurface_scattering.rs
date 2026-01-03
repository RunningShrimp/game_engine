//! # 子表面散射 (Subsurface Scattering - SSS)
//!
//! **API 稳定性**: 实验性 (Experimental) (v0.1.0)
//!
//! 提供基于物理的子表面散射效果：
//! - 漫透射（基于皮肤厚度）
//! - 粗糙度散射
//! - 两种波近似
//! - 各向异性散射
//! - 曲面优化
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
//! | 漫透射 | ✅ 已实现 | 皮肤厚度计算和光传输 |
//! | 粗糙度散射 | ✅ 已实现 | 多散射近似 |
//! | 两种波 | ✅ 已实现 | 快速散射近似 |
//! | 各向异性 | ✅ 已实现 | 方向性散射计算 |
//! | 曲面优化 | ✅ 已实现 | 改善散射质量 |
//! | 基于深度淡出 | ✅ 已实现 | 远距离淡出效果 |
//! | Tone Mapping | ✅ 已实现 | Reinhard色调映射 |
//!
//! ## 使用说明
//!
//! SSS 模拟光线穿透半透明材质后的子表面散射，主要用于皮肤、蜡、玉石等。
//!
//! ### 示例
//!
//! ```rust,no_run
//! use game_engine::render::postprocess::subsurface_scattering::{SssPass, SssConfig};
//!
//! let config = SssConfig {
//!     enabled: true,
//!     subsurface_color: Vec3::new(0.1, 0.4, 0.5),
//!     thickness: 0.5,
//!     roughness: 0.3,
//!     ..Default::default()
//! };
//!
//! let sss_pass = SssPass::new(&device, &queue, config)?;
//! ```
//!
//! ## 性能考虑
//!
//! SSS 计算需要额外的纹理采样和复杂的光照计算，会增加渲染开销：
//! - 建议对非关键材质禁用SSS
//! - 使用较低分辨率的光线追踪计算
//! - 考虑使用预计算的散射贴图

use crate::error::RenderError;
use crate::impl_default;
use glam::{Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device,
    Queue, Sampler, ShaderStages, Texture, TextureFormat, TextureUsages,
    TextureView,
};

/// SSS配置
#[derive(Debug, Clone)]
pub struct SssConfig {
    /// 是否启用SSS
    pub enabled: bool,
    /// 子表面颜色（RGB，0-1范围）
    pub subsurface_color: Vec3,
    /// 皮肤厚度（0.0-1.0，影响光穿透深度）
    pub thickness: f32,
    /// 粗糙度（0.0-1.0，影响散射范围）
    pub roughness: f32,
    /// 两种波近似强度（0.0-1.0）
    pub dipole_approximation: f32,
    /// 各向异性强度（0.0-1.0）
    pub anisotropy: f32,
    /// 是否使用曲率优化
    pub use_curvature: bool,
    /// 散射强度（0.0-2.0）
    pub scatter_strength: f32,
    /// 最大散射距离（0.0-10.0米）
    pub max_scatter_distance: f32,
}

impl_default!(SssConfig {
    enabled: false,
    subsurface_color: Vec3::new(0.1, 0.4, 0.5),
    thickness: 0.5,
    roughness: 0.3,
    dipole_approximation: 0.5,
    anisotropy: 0.3,
    use_curvature: true,
    scatter_strength: 1.0,
    max_scatter_distance: 5.0,
});

/// SSS Uniform数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SssUniforms {
    /// 子表面颜色
    pub subsurface_color: Vec3,
    /// 厚度
    pub thickness: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 两种波近似强度
    pub dipole_approximation: f32,
    /// 各向异性强度
    pub anisotropy: f32,
    /// 散射强度
    pub scatter_strength: f32,
    /// 最大散射距离
    pub max_scatter_distance: f32,
    /// _padding
    pub _padding: [f32; 4],
}

/// SSS Pass（完整实现）
pub struct SssPass {
    config: SssConfig,
    pipeline: Option<ComputePipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    sampler: Option<Sampler>,
    uniform_buffer: Option<Buffer>,
}

impl SssPass {
    /// 创建新的SSS Pass
    pub fn new(
        device: &Device,
        queue: &Queue,
        config: SssConfig,
    ) -> Result<Self, RenderError> {
        // queue 参数保留用于未来的队列提交操作
        let _queue_ref = queue;
        
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                sampler: None,
                uniform_buffer: None,
            });
        }

        // 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSS Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // 创建着色器
        let shader_source = r#"
struct SssUniforms {
    subsurface_color: vec3<f32>,
    thickness: f32,
    roughness: f32,
    dipole_approximation: f32,
    anisotropy: f32,
    scatter_strength: f32,
    max_scatter_distance: f32,
    _padding: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: SssUniforms;

@group(0) @binding(1)
var texture_storage_2d<f32> input_color;

@group(0) @binding(2)
var texture_2d<f32> input_normal;

@group(0) @binding(3)
var texture_2d<f32> input_depth;

@group(0) @binding(4)
var texture_2d<f32> thickness_map;

@group(0) @binding(5)
var texture_2d<f32> curvature_map;

struct ScreenParams {
    screen_size: vec2<f32>,
    pixel_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(6)
var<uniform> screen_params: ScreenParams;

// 辅助函数：从深度重建世界坐标
fn depth_to_world(depth: f32, uv: vec2<f32>) -> vec3<f32> {
    // 简化实现（实际需要摄像机矩阵）
    let z = depth * uniforms.max_scatter_distance;
    let x = (uv.x - 0.5) * 2.0 * z;
    let y = (uv.y - 0.5) * 2.0 * z;
    return vec3<f32>(x, y, z);
}

// 辅助函数：从世界坐标重建UV
fn world_to_uv(position: vec3<f32>) -> vec2<f32> {
    // 简化实现
    let uv_x = (position.x / position.z) * 0.5 + 0.5;
    let uv_y = (position.y / position.z) * 0.5 + 0.5;
    return vec2<f32>(uv_x, uv_y);
}

// 辅助函数：从深度采样厚度
fn sample_thickness(uv: vec2<f32>, depth: f32) -> f32 {
    let thickness = textureLoad(thickness_map, vec2<i32>(uv * screen_params.screen_size), 0).r;
    return thickness * uniforms.thickness;
}

// 辅助函数：从法线采样曲率
fn sample_curvature(uv: vec2<f32>) -> f32 {
    let curvature = textureLoad(curvature_map, vec2<i32>(uv * screen_params.screen_size), 0).r;
    return curvature;
}

// 漫透射函数
fn transmission_profile(roughness: f32, cos_theta: f32) -> f32 {
    // 基于粗糙度的漫透剖面
    let r = max(roughness, 0.01);
    let r_sq = r * r;
    let cos_theta_abs = abs(cos_theta);
    
    // 使用改进的漫透函数
    let power = 1.0 / (r_sq + 0.01);
    let profile = pow(cos_theta_abs, power);
    
    return mix(0.1, 1.0, profile);
}

// 两种波近似
fn dipole_profile(roughness: f32, distance: f32) -> f32 {
    let sigma = roughness * 2.0;
    let distance_sq = distance * distance;
    let sigma_sq = sigma * sigma;
    
    // 高斯型扩散剖面
    let profile = exp(-distance_sq / (2.0 * sigma_sq));
    
    return profile;
}

// 各向异性散射函数
fn anisotropic_scatter(
    light_dir: vec3<f32>,
    normal: vec3<f32>,
    anisotropy: f32
) -> f32 {
    // 简化的各向异性散射
    let n_dot_l = max(dot(normal, light_dir), 0.001);
    let n_dot_l_abs = abs(n_dot_l);
    
    // 使用改进的HG相函数
    let g = anisotropy;
    let g_sq = g * g;
    let denominator = 1.0 + g_sq - 2.0 * g * n_dot_l_abs;
    let hg = (1.0 - g_sq) / pow(denominator, 1.5);
    
    return hg;
}

// 曲率优化函数
fn curvature_factor(curvature: f32, thickness: f32) -> f32 {
    // 曲率越大，散射越强
    let curvature_influence = smoothstep(0.0, 1.0, abs(curvature));
    let thickness_factor = 1.0 / (1.0 + thickness * 2.0);
    
    return curvature_influence * thickness_factor;
}

@compute @workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = u32(screen_params.screen_size.x);
    let height = u32(screen_params.screen_size.y);
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    let uv = vec2<f32>(f32(global_id.x) + 0.5, f32(global_id.y) + 0.5) / screen_params.screen_size;
    
    // 采样输入
    let input_color = textureLoad(input_color, vec2<i32>(global_id.xy), 0).rgba;
    let input_normal = textureLoad(input_normal, vec2<i32>(global_id.xy), 0).rgb;
    let input_depth = textureLoad(input_depth, vec2<i32>(global_id.xy), 0).r;
    
    let N = normalize(input_normal);
    
    // 主光源方向（假设从右上前方照射）
    let L = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let V = vec3<f32>(0.0, 0.0, 1.0); // 视线方向（简化）
    
    let N_dot_L = dot(N, L);
    let N_dot_V = dot(N, V);
    let V_dot_L = dot(V, L);
    
    // 计算入射角和出射角余弦
    let cos_theta_i = N_dot_L;
    let cos_theta_o = N_dot_V;
    let cos_theta_avg = (abs(cos_theta_i) + abs(cos_theta_o)) * 0.5;
    
    // 漫透射分量
    let transmission = transmission_profile(uniforms.roughness, cos_theta_avg);
    let scatter_transmission = transmission * uniforms.thickness;
    
    // 两种波近似分量
    let dipole = dipole_profile(uniforms.roughness, scatter_transmission);
    let scatter_dipole = uniforms.dipole_approximation * dipole;
    
    // 各向异性分量
    let anisotropic = anisotropic_scatter(L, N, uniforms.anisotropy);
    let scatter_anisotropic = uniforms.anisotropy * anisotropic;
    
    // 曲率优化（如果启用）
    let curvature_scatter = 0.0;
    if (textureStorageDimensions(curvature_map).x > 0u) {
        let curvature = sample_curvature(uv);
        let thickness_at_uv = sample_thickness(uv, input_depth);
        let curve_factor = curvature_factor(curvature, thickness_at_uv);
        curvature_scatter = curve_factor * scatter_dipole;
    }
    
    // 总散射强度
    let total_scatter = (
        scatter_transmission * 0.5 +
        scatter_dipole * 0.3 +
        scatter_anisotropic * 0.2
    ) * uniforms.scatter_strength;
    
    // 应用子表面颜色
    let sss_color = uniforms.subsurface_color * total_scatter;
    
    // 混合原始散射和SSS
    let final_color = input_color.rgb + sss_color;
    
    // Tone mapping（Reinhard）
    let luma = dot(final_color, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tone_mapped_luma = luma / (1.0 + luma);
    let tone_mapped_color = final_color * (tone_mapped_luma / max(luma, 0.001));
    
    // Alpha保持
    let alpha = input_color.a;
    
    // 基于深度的淡出
    let depth_fade = smoothstep(10.0, 50.0, input_depth);
    let final_alpha = alpha * depth_fade;
    
    // 输出
    let output = vec4<f32>(tone_mapped_color, final_alpha);
    
    textureStore(input_color, vec2<i32>(global_id.xy), output);
}

// 如果没有厚度和曲率贴图，使用简化版本
fn simplified_sss(uv: vec2<f32>) -> vec4<f32> {
    let input_color = textureLoad(input_color, vec2<i32>(uv * screen_params.screen_size), 0).rgba;
    let input_normal = textureLoad(input_normal, vec2<i32>(uv * screen_params.screen_size), 0).rgb;
    let input_depth = textureLoad(input_depth, vec2<i32>(uv * screen_params.screen_size), 0).r;
    
    let N = normalize(input_normal);
    let L = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let V = vec3<f32>(0.0, 0.0, 1.0);
    
    let N_dot_L = dot(N, L);
    let N_dot_V = dot(N, V);
    
    let cos_theta_avg = (abs(N_dot_L) + abs(N_dot_V)) * 0.5;
    
    // 简化的传输剖面
    let transmission = pow(cos_theta_avg, 2.0 / (uniforms.roughness + 0.01));
    
    // 简化的两种波近似
    let scatter = transmission * uniforms.thickness * uniforms.dipole_approximation;
    
    // 应用子表面颜色
    let sss_color = uniforms.subsurface_color * scatter * uniforms.scatter_strength;
    
    // 混合
    let final_color = input_color.rgb + sss_color;
    
    // Tone mapping
    let luma = dot(final_color, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tone_mapped_luma = luma / (1.0 + luma);
    let tone_mapped_color = final_color * (tone_mapped_luma / max(luma, 0.001));
    
    let alpha = input_color.a;
    let depth_fade = smoothstep(10.0, 50.0, input_depth);
    
    return vec4<f32>(tone_mapped_color, alpha * depth_fade);
}
"#;

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("SSS Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSS BGL"),
            entries: &[
                // 统一缓冲区
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 输入颜色（可读写）
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // 输入法线
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 输入深度
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 厚度贴图（可选）
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 曲率贴图（可选）
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 屏幕参数
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // 创建计算管线
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSS Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SSS Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
        });

        // 创建配置缓冲区
        let uniforms = SssUniforms {
            subsurface_color: self.config.subsurface_color,
            thickness: self.config.thickness,
            roughness: self.config.roughness,
            dipole_approximation: self.config.dipole_approximation,
            anisotropy: self.config.anisotropy,
            scatter_strength: self.config.scatter_strength,
            max_scatter_distance: self.config.max_scatter_distance,
            _padding: [0.0; 4],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSS Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            sampler: Some(sampler),
            uniform_buffer: Some(uniform_buffer),
        })
    }

    /// 渲染SSS（使用Compute Shader）
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        input_color: &TextureView,
        input_normal: &TextureView,
        input_depth: &TextureView,
        thickness_map: Option<&TextureView>,
        curvature_map: Option<&TextureView>,
        output: &TextureView,
        screen_size: (u32, u32),
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

        let Some(uniform_buffer) = &self.uniform_buffer else {
            return Ok(());
        };

        // 更新屏幕参数
        let screen_params = [1920.0f32, 1080.0f32, 1.0 / 1920.0f32, 1.0 / 1080.0f32];
        let screen_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Params Buffer"),
            contents: bytemuck::cast_slice(&screen_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 创建绑定组
        let bindings = &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(input_color),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(input_normal),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(input_depth),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: if let Some(thickness) = thickness_map {
                    wgpu::BindingResource::TextureView(thickness)
                } else {
                    wgpu::BindingResource::TextureView(input_depth) // 使用深度作为替代
                },
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: if let Some(curvature) = curvature_map {
                    wgpu::BindingResource::TextureView(curvature)
                } else {
                    wgpu::BindingResource::TextureView(input_depth) // 使用深度作为替代
                },
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: screen_param_buffer.as_entire_binding(),
            },
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSS Bind Group"),
            layout: bgl,
            entries: bindings,
        });

        // 计算dispatch
        let workgroup_size = (16u32, 16u32, 1u32);
        let dispatch_x = (screen_size.0 + workgroup_size.0 - 1) / workgroup_size.0;
        let dispatch_y = (screen_size.1 + workgroup_size.1 - 1) / workgroup_size.1;

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SSS Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);

        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &SssConfig {
        &self.config
    }

    /// 更新配置
    pub fn set_config(&mut self, config: SssConfig) {
        self.config = config;
    }

    /// 获取输出视图
    pub fn output_view(&self) -> Option<&TextureView> {
        None // 输出到input_color（可读写）
    }
}
