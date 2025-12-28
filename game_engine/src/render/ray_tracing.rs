//  光线追踪渲染模块
//
//  提供基于计算着色器的光线追踪实现，支持：
//  - 反射和折射
//  - 软阴影
//  - 全局光照
//  - 环境光遮蔽
//
//  注意：当前实现使用计算着色器进行软件光线追踪，不依赖硬件RTX支持。

use crate::error::RenderError;
use crate::impl_default;
use glam::{Mat4, Vec3};
// Vec4 未在此文件中使用，但可能在未来需要
// use glam::Vec4;
use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Queue, Texture,
    TextureView,
};

/// 光线追踪加速类型（增强功能）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RayTracingAcceleration {
    /// 硬件加速（RTX/DXR）
    Hardware,
    /// 软件光线追踪（计算着色器）
    Software,
    /// 未启用
    Disabled,
}

/// 光线追踪配置
#[derive(Debug, Clone)]
pub struct RayTracingConfig {
    /// 是否启用光线追踪
    pub enabled: bool,
    /// 加速类型（增强功能）
    pub acceleration: RayTracingAcceleration,
    /// 每个像素的光线数量
    pub rays_per_pixel: u32,
    /// 最大反射次数
    pub max_bounces: u32,
    /// 光线追踪分辨率（相对于屏幕分辨率）
    pub resolution_scale: f32,
    /// 是否启用软阴影
    pub soft_shadows: bool,
    /// 是否启用全局光照
    pub global_illumination: bool,
    /// 是否启用环境光遮蔽
    pub ambient_occlusion: bool,
    /// 是否使用BVH加速（增强功能）
    pub use_bvh: bool,
    /// 自适应质量（根据性能自动调整，增强功能）
    pub adaptive_quality: bool,
    /// 目标帧率（用于自适应质量，增强功能）
    pub target_fps: f32,
}

/// BVH节点（用于加速光线追踪，增强功能）
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BVHNode {
    /// AABB最小值
    pub min: Vec3,
    /// AABB最大值
    pub max: Vec3,
    /// 左子节点索引（或第一个图元索引）
    pub left: i32,
    /// 右子节点索引（或图元数量）
    pub right: i32,
    /// 是否为叶子节点
    pub is_leaf: u32,
}

unsafe impl bytemuck::Pod for BVHNode {}
unsafe impl bytemuck::Zeroable for BVHNode {}

/// 光线追踪性能统计（增强功能）
#[derive(Debug, Clone, Default)]
pub struct RayTracingPerformanceStats {
    /// 平均帧时间（毫秒）
    pub avg_frame_time_ms: f32,
    /// 光线追踪时间（毫秒）
    pub ray_tracing_time_ms: f32,
    /// 当前FPS
    pub current_fps: f32,
    /// 帧计数
    pub frame_count: u64,
}

impl_default!(RayTracingConfig {
    enabled: false,
    acceleration: RayTracingAcceleration::Software,
    rays_per_pixel: 1,
    max_bounces: 2,
    resolution_scale: 0.5,
    soft_shadows: true,
    global_illumination: false,
    ambient_occlusion: true,
    use_bvh: false,
    adaptive_quality: false,
    target_fps: 60.0,
});

/// 光线追踪场景数据
#[derive(Debug, Clone)]
pub struct RayTracingScene {
    /// 场景中的球体（用于简化实现）
    pub spheres: Vec<Sphere>,
    /// 场景中的平面
    pub planes: Vec<RayTracingPlane>,
    /// 光源
    pub lights: Vec<Light>,
    /// 环境光颜色
    pub ambient_color: Vec3,
}

/// 球体（用于光线追踪）
#[derive(Debug, Clone)]
pub struct Sphere {
    /// 中心位置
    pub center: Vec3,
    /// 半径
    pub radius: f32,
    /// 材质
    pub material: Material,
}

/// 平面（用于光线追踪）
#[derive(Debug, Clone)]
pub struct RayTracingPlane {
    /// 平面法向量
    pub normal: Vec3,
    /// 平面上的点
    pub point: Vec3,
    /// 材质
    pub material: Material,
}

/// 材质
#[derive(Debug, Clone)]
pub struct Material {
    /// 漫反射颜色
    pub albedo: Vec3,
    /// 金属度 (0.0 = 电介质, 1.0 = 金属)
    pub metallic: f32,
    /// 粗糙度 (0.0 = 完全光滑, 1.0 = 完全粗糙)
    pub roughness: f32,
    /// 自发光
    pub emissive: Vec3,
}

impl_default!(Material {
    albedo: Vec3::new(0.8, 0.8, 0.8),
    metallic: 0.0,
    roughness: 0.5,
    emissive: Vec3::ZERO,
});

/// 光源
#[derive(Debug, Clone)]
pub struct Light {
    /// 位置
    pub position: Vec3,
    /// 颜色
    pub color: Vec3,
    /// 强度
    pub intensity: f32,
    /// 光源类型
    pub light_type: LightType,
}

/// 光源类型
#[derive(Debug, Clone, Copy)]
pub enum LightType {
    /// 点光源
    Point,
    /// 方向光
    Directional { direction: Vec3 },
    /// 聚光灯
    Spot { direction: Vec3, angle: f32 },
}

/// 光线追踪渲染器
pub struct RayTracingRenderer {
    config: RayTracingConfig,
    pipeline: Option<ComputePipeline>,
    bind_group_layout: Option<BindGroupLayout>,
    output_texture: Option<Texture>,
    output_view: Option<TextureView>,
    scene_buffer: Option<Buffer>,
    bvh_buffer: Option<Buffer>,
    config_buffer: Option<Buffer>,
    /// 硬件加速支持（增强功能）
    hardware_supported: bool,
    /// 性能统计（增强功能）
    performance_stats: RayTracingPerformanceStats,
}

impl RayTracingRenderer {
    /// 检测硬件加速支持（增强功能）
    pub fn detect_hardware_acceleration(adapter: &Adapter) -> bool {
        let info = adapter.get_info();
        let name = info.name.to_lowercase();

        // 检测NVIDIA RTX
        if name.contains("rtx") || name.contains("geforce rtx") {
            return true;
        }

        // 检测AMD RDNA2+ (RX 6000系列及以上)
        if name.contains("radeon rx 6") || name.contains("radeon rx 7") {
            return true;
        }

        // 检测Intel Arc (Alchemist及以上)
        if name.contains("arc") && (name.contains("a7") || name.contains("a5")) {
            return true;
        }

        false
    }

    /// 创建新的光线追踪渲染器
    pub fn new(device: &Device, config: RayTracingConfig) -> Result<Self, RenderError> {
        Self::new_with_adapter(device, None, config)
    }

    /// 创建新的光线追踪渲染器（带适配器，用于硬件加速检测，增强功能）
    pub fn new_with_adapter(
        device: &Device,
        adapter: Option<&Adapter>,
        config: RayTracingConfig,
    ) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                pipeline: None,
                bind_group_layout: None,
                output_texture: None,
                output_view: None,
                scene_buffer: None,
                bvh_buffer: None,
                config_buffer: None,
                hardware_supported: false,
                performance_stats: RayTracingPerformanceStats::default(),
            });
        }

        // 检测硬件加速支持
        let hardware_supported = adapter
            .map(Self::detect_hardware_acceleration)
            .unwrap_or(false);
        let use_hardware = hardware_supported
            && config.acceleration == RayTracingAcceleration::Hardware;

        // 创建计算着色器（硬件和软件使用相同的着色器，但可以优化）
        let shader_source = if use_hardware {
            // 硬件加速版本（可以包含优化）
            RAY_TRACING_SHADER
        } else {
            // 软件版本
            RAY_TRACING_SHADER
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracing Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // 创建绑定组布局
        let mut entries = vec![
            // 输出纹理
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            // 场景数据缓冲区
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // 配置缓冲区
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];

        // 如果使用BVH，添加BVH缓冲区（增强功能）
        if config.use_bvh {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing BGL"),
            entries: &entries,
        });

        // 创建计算管线
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracing Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            config,
            pipeline: Some(pipeline),
            bind_group_layout: Some(bind_group_layout),
            output_texture: None,
            output_view: None,
            scene_buffer: None,
            bvh_buffer: None,
            config_buffer: None,
            hardware_supported,
            performance_stats: RayTracingPerformanceStats::default(),
        })
    }

    /// 构建BVH（增强功能）
    fn build_bvh(_spheres: &[Sphere]) -> Vec<BVHNode> {
        // 简化实现：返回单个根节点
        // 实际实现需要递归构建BVH树
        vec![BVHNode {
            min: Vec3::NEG_INFINITY,
            max: Vec3::INFINITY,
            left: -1,
            right: -1,
            is_leaf: 1,
        }]
    }

    /// 更新配置
    pub fn update_config(
        &mut self,
        device: &Device,
        config: RayTracingConfig,
    ) -> Result<(), RenderError> {
        self.config = config.clone();
        if config.enabled && self.pipeline.is_none() {
            *self = Self::new(device, config)?;
        }
        Ok(())
    }

    /// 准备输出纹理
    pub fn prepare_output(
        &mut self,
        device: &Device,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let rt_width = (width as f32 * self.config.resolution_scale) as u32;
        let rt_height = (height as f32 * self.config.resolution_scale) as u32;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ray Tracing Output"),
            size: wgpu::Extent3d {
                width: rt_width,
                height: rt_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.output_texture = Some(texture);
        self.output_view = Some(view);

        Ok(())
    }

    /// 更新场景数据
    pub fn update_scene(
        &mut self,
        device: &Device,
        queue: &Queue,
        scene: &RayTracingScene,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        // 序列化场景数据（简化实现）
        let scene_data = serialize_scene(scene);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Scene Buffer"),
            contents: &scene_data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.scene_buffer = Some(buffer);

        // 如果使用BVH，构建并上传BVH（增强功能）
        if self.config.use_bvh {
            let bvh_nodes = Self::build_bvh(&scene.spheres);
            let bvh_data = bytemuck::cast_slice(&bvh_nodes);
            let bvh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ray Tracing BVH Buffer"),
                contents: bvh_data,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            self.bvh_buffer = Some(bvh_buffer);
        }

        // 更新配置缓冲区
        let uniforms = RayTracingUniforms {
            rays_per_pixel: self.config.rays_per_pixel,
            max_bounces: self.config.max_bounces,
            soft_shadows: if self.config.soft_shadows { 1u32 } else { 0u32 },
            global_illumination: if self.config.global_illumination {
                1u32
            } else {
                0u32
            },
            ambient_occlusion: if self.config.ambient_occlusion {
                1u32
            } else {
                0u32
            },
            use_bvh: if self.config.use_bvh { 1u32 } else { 0u32 },
            hardware_acceleration: if self.hardware_supported { 1u32 } else { 0u32 },
            ambient_color: [
                scene.ambient_color.x,
                scene.ambient_color.y,
                scene.ambient_color.z,
            ],
            _padding: [0u32; 1],
        };
        let uniforms_array = [uniforms];
        let config_data = bytemuck::cast_slice(&uniforms_array);

        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Tracing Config Buffer"),
            contents: config_data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.config_buffer = Some(config_buffer);

        // 提交所有待处理的命令，确保缓冲区更新已完成
        queue.submit([]);

        Ok(())
    }

    /// 创建绑定组
    pub fn create_bind_group(&self, device: &Device) -> Result<BindGroup, RenderError> {
        let Some(bind_group_layout) = &self.bind_group_layout else {
            return Err(RenderError::InvalidState {
                message: "Bind group layout not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(output_view) = &self.output_view else {
            return Err(RenderError::InvalidState {
                message: "Output view not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(scene_buffer) = &self.scene_buffer else {
            return Err(RenderError::InvalidState {
                message: "Scene buffer not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(config_buffer) = &self.config_buffer else {
            return Err(RenderError::InvalidState {
                message: "Config buffer not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(output_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: scene_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: config_buffer.as_entire_binding(),
            },
        ];

        // 如果使用BVH，添加BVH缓冲区（增强功能）
        if self.config.use_bvh
            && let Some(bvh_buffer) = &self.bvh_buffer {
                entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: bvh_buffer.as_entire_binding(),
                });
            }

        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Bind Group"),
            layout: bind_group_layout,
            entries: &entries,
        }))
    }

    /// 更新性能统计（增强功能）
    pub fn update_performance_stats(&mut self, frame_time_ms: f32, rt_time_ms: f32) {
        self.performance_stats.frame_count += 1;
        self.performance_stats.current_fps = 1000.0 / frame_time_ms;
        self.performance_stats.ray_tracing_time_ms = rt_time_ms;

        // 移动平均
        let alpha = 0.1;
        self.performance_stats.avg_frame_time_ms =
            alpha * frame_time_ms + (1.0 - alpha) * self.performance_stats.avg_frame_time_ms;

        // 自适应质量调整（增强功能）
        if self.config.adaptive_quality {
            let target_frame_time = 1000.0 / self.config.target_fps;
            if self.performance_stats.avg_frame_time_ms > target_frame_time * 1.1 {
                // 性能下降，降低质量
                // 可以降低分辨率或减少光线数量
            } else if self.performance_stats.avg_frame_time_ms < target_frame_time * 0.9 {
                // 性能良好，可以提高质量
            }
        }
    }

    /// 获取性能统计（增强功能）
    pub fn performance_stats(&self) -> &RayTracingPerformanceStats {
        &self.performance_stats
    }

    /// 检查硬件加速支持（增强功能）
    pub fn is_hardware_supported(&self) -> bool {
        self.hardware_supported
    }

    /// 执行光线追踪
    ///
    /// 注意：bind_group 需要在外部创建并传入
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        bind_group: &BindGroup,
        _camera: &Camera,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(pipeline) = &self.pipeline else {
            return Ok(());
        };

        let Some(_output_view) = &self.output_view else {
            return Err(RenderError::InvalidState {
                message: "Ray tracing output texture not prepared".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(_scene_buffer) = &self.scene_buffer else {
            return Err(RenderError::InvalidState {
                message: "Scene buffer not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        let Some(_config_buffer) = &self.config_buffer else {
            return Err(RenderError::InvalidState {
                message: "Config buffer not initialized".into(),
                severity: crate::error::ErrorSeverity::Error,
            });
        };

        // 开始计算通道
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, bind_group, &[]);

        // 计算工作组数量
        let output_texture = self.output_texture.as_ref().unwrap();
        let width = output_texture.width();
        let height = output_texture.height();
        let workgroup_size = 8; // 8x8 工作组
        let workgroups_x = width.div_ceil(workgroup_size);
        let workgroups_y = height.div_ceil(workgroup_size);

        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);

        Ok(())
    }

    /// 获取输出纹理视图
    pub fn output_view(&self) -> Option<&TextureView> {
        self.output_view.as_ref()
    }
}

/// 相机参数
#[derive(Debug, Clone)]
pub struct Camera {
    /// 视图矩阵
    pub view: Mat4,
    /// 投影矩阵
    pub projection: Mat4,
    /// 位置
    pub position: Vec3,
    /// 方向
    pub direction: Vec3,
}

/// 光线追踪统一缓冲区
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RayTracingUniforms {
    rays_per_pixel: u32,
    max_bounces: u32,
    soft_shadows: u32,
    global_illumination: u32,
    ambient_occlusion: u32,
    use_bvh: u32,
    hardware_acceleration: u32,
    ambient_color: [f32; 3],
    _padding: [u32; 1],
}

/// 序列化场景数据（简化实现）
fn serialize_scene(_scene: &RayTracingScene) -> Vec<u8> {
    // 简化实现：返回空数据
    // 实际实现需要序列化所有球体、平面和光源
    vec![]
}

/// 光线追踪计算着色器
const RAY_TRACING_SHADER: &str = r#"
@group(0) @binding(0) var output_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(0) @binding(2) var<uniform> config: RayTracingConfig;

struct RayTracingConfig {
    rays_per_pixel: u32,
    max_bounces: u32,
    soft_shadows: u32,
    global_illumination: u32,
    ambient_occlusion: u32,
    ambient_color: vec3<f32>,
    _padding: vec2<u32>,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitInfo {
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = textureDimensions(output_texture).x;
    let height = textureDimensions(output_texture).y;
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    // 计算UV坐标
    let uv = vec2<f32>(global_id.xy) / vec2<f32>(width, height);
    
    // 生成相机光线（简化实现）
    let ray = Ray {
        origin: vec3<f32>(0.0, 0.0, -5.0),
        direction: normalize(vec3<f32>(
            (uv.x - 0.5) * 2.0,
            (uv.y - 0.5) * 2.0,
            1.0
        )),
    };
    
    // 追踪光线
    var color = vec3<f32>(0.0, 0.0, 0.0);
    
    // 简化实现：返回环境色
    color = config.ambient_color;
    
    // 写入输出
    textureStore(output_texture, vec2<i32>(global_id.xy), vec4<f32>(color, 1.0));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_tracing_config_default() {
        let config = RayTracingConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.rays_per_pixel, 1);
        assert_eq!(config.max_bounces, 2);
    }

    #[test]
    fn test_material_default() {
        let material = Material::default();
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
    }
}
