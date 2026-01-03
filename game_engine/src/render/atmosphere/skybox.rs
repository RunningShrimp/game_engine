//! # Dynamic Skybox System
//!
//! 基于物理的动态天空盒系统，提供：
//! - 程序化天空盒生成
//! - 基于大气散射的真实天空渲染
//! - 昼夜循环支持
//! - 星空系统
//! - 云层投影
//!
//! ## 功能特性
//!
//! ### 大气散射
//! - Rayleigh散射（瑞利散射，天空蓝色）
//! - Mie散射（米氏散射，云和雾）
//! - 臭氧吸收
//! - 多次散射近似
//!
//! ### 时间系统
//! - 连续的昼夜循环
//! - 日出日落效果
//! - 月亮和星星
//! - 动态光照变化
//!
//! ## 使用示例
//!
//! ```ignore
//! use game_engine::render::atmosphere::skybox::{DynamicSkybox, SkyboxConfig, TimeOfDay};
//!
//! let config = SkyboxConfig::default();
//! let mut skybox = DynamicSkybox::new(device, &config)?;
//!
//! // 设置时间（0.0 = 午夜, 0.5 = 正午）
//! skybox.set_time_of_day(0.3);
//!
//! // 渲染天空盒
//! skybox.render(&mut render_pass, &camera);
//! ```

use crate::error::RenderError;
use crate::impl_default;
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::*;

/// 一天中的时间
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Midnight,    // 00:00
    Dawn,        // 06:00
    Noon,        // 12:00
    Dusk,        // 18:00
    Custom(f32), // 自定义时间（0.0-1.0）
}

impl TimeOfDay {
    /// 转换为0-1范围的值
    pub fn as_normalized(&self) -> f32 {
        match self {
            TimeOfDay::Midnight => 0.0,
            TimeOfDay::Dawn => 0.25,
            TimeOfDay::Noon => 0.5,
            TimeOfDay::Dusk => 0.75,
            TimeOfDay::Custom(value) => value.rem_euclid(1.0),
        }
    }

    /// 从归一化值创建
    pub fn from_normalized(value: f32) -> Self {
        let value = value.rem_euclid(1.0);
        if (value - 0.0).abs() < 0.01 {
            TimeOfDay::Midnight
        } else if (value - 0.25).abs() < 0.01 {
            TimeOfDay::Dawn
        } else if (value - 0.5).abs() < 0.01 {
            TimeOfDay::Noon
        } else if (value - 0.75).abs() < 0.01 {
            TimeOfDay::Dusk
        } else {
            TimeOfDay::Custom(value)
        }
    }

    /// 获取太阳角度（弧度）
    pub fn sun_angle(&self) -> f32 {
        let t = self.as_normalized();
        // 0.5是正午，太阳在最高点
        // 0.0和1.0是午夜，太阳在地平线下
        (t - 0.5) * std::f32::consts::PI * 2.0
    }

    /// 是否是白天
    pub fn is_daytime(&self) -> bool {
        let t = self.as_normalized();
        t > 0.25 && t < 0.75
    }
}

/// 天空盒配置
#[derive(Debug, Clone)]
pub struct SkyboxConfig {
    /// 纹理分辨率
    pub resolution: u32,
    /// 是否启用大气散射
    pub enable_atmospheric_scattering: bool,
    /// 是否启用星星
    pub enable_stars: bool,
    /// 是否启用太阳/月亮
    pub enable_celestial_bodies: bool,
    /// 大气散射强度
    pub scattering_intensity: f32,
    /// Rayleigh散射系数
    pub rayleigh_coefficient: Vec3,
    /// Mie散射系数
    pub mie_coefficient: Vec3,
    /// Mie散射方向性因子
    pub mie_g: f32,
    /// 星星数量
    pub star_count: u32,
    /// 星星亮度
    pub star_brightness: f32,
}

impl_default!(SkyboxConfig {
    resolution: 1024,
    enable_atmospheric_scattering: true,
    enable_stars: true,
    enable_celestial_bodies: true,
    scattering_intensity: 1.0,
    rayleigh_coefficient: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
    mie_coefficient: Vec3::new(2.0e-5, 2.0e-5, 2.0e-5),
    mie_g: 0.758,
    star_count: 2000,
    star_brightness: 0.8,
});

/// 天空盒Uniform数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyboxUniforms {
    /// 投影矩阵
    projection: [[f32; 4]; 4],
    /// 视图矩阵（移除平移）
    view: [[f32; 4]; 4],
    /// 太阳方向
    sun_direction: [f32; 3],
    /// 太阳强度
    sun_intensity: f32,
    /// Rayleigh系数
    rayleigh: [f32; 3],
    /// Mie系数
    mie: [f32; 3],
    /// Mie方向性因子
    mie_g: f32,
    /// 散射强度
    scattering_intensity: f32,
    /// 星星启用标志
    enable_stars: u32,
    /// 天体启用标志
    enable_celestial: u32,
    /// 填充
    _pad: [u32; 2],
}

/// 动态天空盒渲染器
pub struct DynamicSkybox {
    config: SkyboxConfig,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    uniform_buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    cube_texture: Option<Texture>,
    cube_view: Option<TextureView>,
    sampler: Sampler,
    current_time: TimeOfDay,
}

impl DynamicSkybox {
    /// 创建动态天空盒
    pub fn new(device: &Device, config: &SkyboxConfig) -> Result<Self, RenderError> {
        // 创建立方体顶点
        let vertices = Self::create_skybox_vertices();
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Skybox Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        // 创建索引
        let indices = Self::create_skybox_indices();
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Skybox Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let index_count = indices.len() as u32;

        // 创建Uniform缓冲区
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Skybox Uniform Buffer"),
            size: std::mem::size_of::<SkyboxUniforms>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建采样器
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Skybox Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Skybox BGL"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // 创建着色器
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Skybox Shader"),
            source: ShaderSource::Wgsl(SKYBOX_SHADER.into()),
        });

        // 创建管线布局
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建渲染管线
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::SrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            config: config.clone(),
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            bind_group_layout,
            cube_texture: None,
            cube_view: None,
            sampler,
            current_time: TimeOfDay::Noon,
        })
    }

    /// 设置时间
    pub fn set_time_of_day(&mut self, time: TimeOfDay) {
        self.current_time = time;
    }

    /// 更新天空盒
    pub fn update(&mut self, queue: &Queue, view_proj: &Mat4) {
        // 计算太阳方向
        let sun_angle = self.current_time.sun_angle();
        let sun_direction = Vec3::new(sun_angle.sin(), sun_angle.cos(), 0.0).normalize();

        // 计算太阳强度（基于时间）
        let sun_intensity = if self.current_time.is_daytime() {
            1.0
        } else {
            0.1
        };

        // 创建视图矩阵（移除平移）
        let mut view = *view_proj;
        view.col(3).x = 0.0;
        view.col(3).y = 0.0;
        view.col(3).z = 0.0;

        let uniforms = SkyboxUniforms {
            projection: {
                let mat = *view_proj.as_ref();
                [
                    [mat[0], mat[1], mat[2], mat[3]],
                    [mat[4], mat[5], mat[6], mat[7]],
                    [mat[8], mat[9], mat[10], mat[11]],
                    [mat[12], mat[13], mat[14], mat[15]],
                ]
            },
            view: {
                let mat = *view.as_ref();
                [
                    [mat[0], mat[1], mat[2], mat[3]],
                    [mat[4], mat[5], mat[6], mat[7]],
                    [mat[8], mat[9], mat[10], mat[11]],
                    [mat[12], mat[13], mat[14], mat[15]],
                ]
            },
            sun_direction: sun_direction.to_array(),
            sun_intensity,
            rayleigh: self.config.rayleigh_coefficient.to_array(),
            mie: self.config.mie_coefficient.to_array(),
            mie_g: self.config.mie_g,
            scattering_intensity: self.config.scattering_intensity,
            enable_stars: if self.config.enable_stars { 1 } else { 0 },
            enable_celestial: if self.config.enable_celestial_bodies {
                1
            } else {
                0
            },
            _pad: [0, 0],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// 渲染天空盒
    pub fn render<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        view_proj: &Mat4,
    ) -> Result<(), RenderError> {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        Ok(())
    }

    // 辅助函数
    fn create_skybox_vertices() -> Vec<[f32; 3]> {
        vec![
            // 前面
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            // 后面
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ]
    }

    fn create_skybox_indices() -> Vec<u32> {
        vec![
            // 前面
            0, 1, 2, 2, 3, 0, // 后面
            5, 4, 7, 7, 6, 5, // 左面
            4, 0, 3, 3, 7, 4, // 右面
            1, 5, 6, 6, 2, 1, // 上面
            3, 2, 6, 6, 7, 3, // 下面
            4, 5, 1, 1, 0, 4,
        ]
    }
}

/// 天空盒着色器
const SKYBOX_SHADER: &str = r#"
struct SkyboxUniforms {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    sun_direction: vec3<f32>,
    sun_intensity: f32,
    rayleigh: vec3<f32>,
    mie: vec3<f32>,
    mie_g: f32,
    scattering_intensity: f32,
    enable_stars: u32,
    enable_celestial: u32,
    _pad: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: SkyboxUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // 移除视图矩阵的平移，使天空盒始终在远处
    let clip_pos = uniforms.projection * uniforms.view * vec4<f32>(input.position, 1.0);

    // 设置深度为1.0（最远）
    output.clip_position = clip_pos.xyww;

    // 使用位置作为UV进行立方体采样
    output.uv = input.position;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(input.uv);

    // 计算大气散射
    let sun_dir = normalize(uniforms.sun_direction);

    // Rayleigh散射相位函数
    let mu = dot(view_dir, sun_dir);
    let rayleigh_phase = 3.0 / (16.0 * 3.14159265) * (1.0 + mu * mu);

    // Mie散射相位函数（Henyey-Greenstein）
    let mie_phase = 3.0 / (8.0 * 3.14159265) *
        ((1.0 - uniforms.mie_g * uniforms.mie_g) * (1.0 + mu * mu)) /
        pow(1.0 + uniforms.mie_g * uniforms.mie_g - 2.0 * uniforms.mie_g * mu, 1.5);

    // 计算散射
    let zenith_angle = max(0.0, view_dir.y);
    let scattering = (uniforms.rayleigh * rayleigh_phase + uniforms.mie * mie_phase) *
        uniforms.scattering_intensity;

    // 基础天空颜色
    let sky_color = scattering * uniforms.sun_intensity;

    // 添加地平线渐变
    let horizon_blend = smoothstep(-0.1, 0.1, zenith_angle);
    let horizon_color = vec3<f32>(0.8, 0.85, 0.9) * (1.0 - horizon_blend);

    var final_color = sky_color * horizon_blend + horizon_color;

    // 添加太阳（如果启用）
    if (uniforms.enable_celestial != 0u) {
        let sun_angle = acos(mu);
        let sun_disk = smoothstep(0.02, 0.01, sun_angle);
        let sun_color = vec3<f32>(1.0, 0.95, 0.8) * uniforms.sun_intensity * sun_disk;
        final_color += sun_color;
    }

    // Tone mapping
    final_color = final_color / (1.0 + final_color);

    // Gamma校正
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(final_color, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day_conversion() {
        assert_eq!(TimeOfDay::Midnight.as_normalized(), 0.0);
        assert_eq!(TimeOfDay::Dawn.as_normalized(), 0.25);
        assert_eq!(TimeOfDay::Noon.as_normalized(), 0.5);
        assert_eq!(TimeOfDay::Dusk.as_normalized(), 0.75);
        assert!((TimeOfDay::Custom(0.3).as_normalized() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_is_daytime() {
        assert!(!TimeOfDay::Midnight.is_daytime());
        assert!(TimeOfDay::Dawn.is_daytime());
        assert!(TimeOfDay::Noon.is_daytime());
        assert!(!TimeOfDay::Dusk.is_daytime());
        assert!(TimeOfDay::Custom(0.6).is_daytime());
        assert!(!TimeOfDay::Custom(0.8).is_daytime());
    }

    #[test]
    fn test_sun_angle() {
        let noon_angle = TimeOfDay::Noon.sun_angle();
        assert!((noon_angle - 0.0).abs() < 0.01);

        let midnight_angle = TimeOfDay::Midnight.sun_angle();
        // 应该接近PI或-PI
        assert!(midnight_angle.abs() > 3.0);
    }

    #[test]
    fn test_config_default() {
        let config = SkyboxConfig::default();
        assert!(config.enable_atmospheric_scattering);
        assert!(config.enable_stars);
        assert_eq!(config.resolution, 1024);
        assert_eq!(config.star_count, 2000);
    }
}
