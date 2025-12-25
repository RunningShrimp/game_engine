//! VXGI (Voxel Global Illumination) 全局光照系统
//!
//! 提供实时全局光照功能：
//! - 场景体素化
//! - 体素锥追踪 (Voxel Cone Tracing)
//! - 间接光照计算
//! - 动态更新支持

use crate::error::RenderError;
use crate::impl_default;
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline, Device, Queue, Texture,
    TextureView, TextureFormat, TextureDimension, TextureUsages,
};

/// VXGI配置
#[derive(Debug, Clone)]
pub struct VxgiConfig {
    /// 是否启用VXGI
    pub enabled: bool,
    /// 体素分辨率（每边体素数，必须是2的幂）
    pub voxel_resolution: u32,
    /// 体素世界空间大小（米）
    pub voxel_size: f32,
    /// 最大追踪距离
    pub max_trace_distance: f32,
    /// 锥追踪步数
    pub cone_trace_steps: u32,
    /// 间接光照强度
    pub indirect_intensity: f32,
    /// 是否启用动态更新
    pub dynamic_update: bool,
    /// 更新频率（每N帧更新一次）
    pub update_frequency: u32,
}

impl_default!(VxgiConfig {
    enabled: false,
    voxel_resolution: 256,
    voxel_size: 0.1,
    max_trace_distance: 10.0,
    cone_trace_steps: 8,
    indirect_intensity: 1.0,
    dynamic_update: false,
    update_frequency: 1,
});

/// 体素数据
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Voxel {
    /// 颜色 (RGB)
    pub color: [u8; 3],
    /// 法线 (编码为2个u8)
    pub normal: [u8; 2],
    /// 遮挡/空值标记
    pub occlusion: u8,
    /// 自发光
    pub emissive: u8,
}

unsafe impl bytemuck::Pod for Voxel {}
unsafe impl bytemuck::Zeroable for Voxel {}

/// VXGI渲染器
pub struct VxgiRenderer {
    config: VxgiConfig,
    /// 体素化管线
    voxelization_pipeline: Option<ComputePipeline>,
    /// 锥追踪管线
    cone_trace_pipeline: Option<ComputePipeline>,
    /// 体素纹理（3D纹理）
    voxel_texture: Option<Texture>,
    voxel_view: Option<TextureView>,
    /// 体素化绑定组布局
    voxelization_bgl: Option<BindGroupLayout>,
    /// 锥追踪绑定组布局
    cone_trace_bgl: Option<BindGroupLayout>,
    /// 场景缓冲区
    scene_buffer: Option<Buffer>,
    /// 配置缓冲区
    config_buffer: Option<Buffer>,
    /// 帧计数（用于动态更新）
    frame_count: u32,
}

impl VxgiRenderer {
    /// 创建新的VXGI渲染器
    pub fn new(device: &Device, config: VxgiConfig) -> Result<Self, RenderError> {
        if !config.enabled {
            return Ok(Self {
                config,
                voxelization_pipeline: None,
                cone_trace_pipeline: None,
                voxel_texture: None,
                voxel_view: None,
                voxelization_bgl: None,
                cone_trace_bgl: None,
                scene_buffer: None,
                config_buffer: None,
                frame_count: 0,
            });
        }

        // 验证体素分辨率是2的幂
        if !config.voxel_resolution.is_power_of_two() {
            return Err(RenderError::InvalidState {
                message: format!(
                    "Voxel resolution must be a power of 2, got {}",
                    config.voxel_resolution
                ),
                severity: crate::error::ErrorSeverity::Error,
            });
        }

        // 创建体素纹理（3D纹理）
        let voxel_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VXGI Voxel Texture"),
            size: wgpu::Extent3d {
                width: config.voxel_resolution,
                height: config.voxel_resolution,
                depth_or_array_layers: config.voxel_resolution,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let voxel_view = voxel_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("VXGI Voxel View"),
            ..Default::default()
        });

        // 创建体素化着色器
        let voxelization_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("VXGI Voxelization Shader"),
            source: wgpu::ShaderSource::Wgsl(VOXELIZATION_SHADER.into()),
        });

        // 创建体素化绑定组布局
        let voxelization_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VXGI Voxelization BGL"),
            entries: &[
                // 体素纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
                // 场景数据
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
                // 配置
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
            ],
        });

        // 创建体素化管线
        let voxelization_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("VXGI Voxelization Pipeline Layout"),
                bind_group_layouts: &[&voxelization_bgl],
                push_constant_ranges: &[],
            });

        let voxelization_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("VXGI Voxelization Pipeline"),
                layout: Some(&voxelization_pipeline_layout),
                module: &voxelization_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        // 创建锥追踪着色器
        let cone_trace_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("VXGI Cone Trace Shader"),
            source: wgpu::ShaderSource::Wgsl(CONE_TRACE_SHADER.into()),
        });

        // 创建锥追踪绑定组布局
        let cone_trace_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VXGI Cone Trace BGL"),
            entries: &[
                // 输出纹理（屏幕空间）
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // 体素纹理
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // 采样器
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // G-Buffer位置
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
                // G-Buffer法线
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
                // 配置
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
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

        // 创建锥追踪管线
        let cone_trace_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("VXGI Cone Trace Pipeline Layout"),
                bind_group_layouts: &[&cone_trace_bgl],
                push_constant_ranges: &[],
            });

        let cone_trace_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("VXGI Cone Trace Pipeline"),
                layout: Some(&cone_trace_pipeline_layout),
                module: &cone_trace_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        // 创建配置缓冲区
        let config_uniform = VxgiUniforms {
            voxel_resolution: config.voxel_resolution,
            voxel_size: config.voxel_size,
            max_trace_distance: config.max_trace_distance,
            cone_trace_steps: config.cone_trace_steps,
            indirect_intensity: config.indirect_intensity,
            _padding: [0u32; 3],
        };
        let config_data = bytemuck::cast_slice(&[config_uniform]);
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VXGI Config Buffer"),
            contents: config_data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            config,
            voxelization_pipeline: Some(voxelization_pipeline),
            cone_trace_pipeline: Some(cone_trace_pipeline),
            voxel_texture: Some(voxel_texture),
            voxel_view: Some(voxel_view),
            voxelization_bgl: Some(voxelization_bgl),
            cone_trace_bgl: Some(cone_trace_bgl),
            scene_buffer: None,
            config_buffer: Some(config_buffer),
            frame_count: 0,
        })
    }

    /// 更新配置
    pub fn update_config(
        &mut self,
        device: &Device,
        queue: &Queue,
        config: VxgiConfig,
    ) -> Result<(), RenderError> {
        self.config = config.clone();
        if config.enabled && self.voxelization_pipeline.is_none() {
            *self = Self::new(device, config)?;
        } else if config.enabled {
            // 更新配置缓冲区
            let config_uniform = VxgiUniforms {
                voxel_resolution: config.voxel_resolution,
                voxel_size: config.voxel_size,
                max_trace_distance: config.max_trace_distance,
                cone_trace_steps: config.cone_trace_steps,
                indirect_intensity: config.indirect_intensity,
                _padding: [0u32; 3],
            };
            let config_data = bytemuck::cast_slice(&[config_uniform]);
            if let Some(config_buffer) = &self.config_buffer {
                queue.write_buffer(config_buffer, 0, config_data);
            }
        }
        Ok(())
    }

    /// 体素化场景
    pub fn voxelize_scene(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        scene_data: &[u8],
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        // 检查是否需要更新
        if !self.config.dynamic_update {
            // 静态场景，只更新一次
            if self.scene_buffer.is_some() {
                return Ok(());
            }
        } else {
            // 动态更新
            self.frame_count += 1;
            if self.frame_count % self.config.update_frequency != 0 {
                return Ok(());
            }
        }

        let Some(pipeline) = &self.voxelization_pipeline else {
            return Ok(());
        };

        // 创建场景缓冲区
        let scene_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VXGI Scene Buffer"),
            contents: scene_data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.scene_buffer = Some(scene_buffer);

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VXGI Voxelization Bind Group"),
            layout: self.voxelization_bgl.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.voxel_view.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.scene_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.config_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });

        // 执行体素化
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VXGI Voxelization Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // 计算工作组数量
        let resolution = self.config.voxel_resolution;
        let workgroup_size = 8;
        let workgroups = (resolution + workgroup_size - 1) / workgroup_size;

        compute_pass.dispatch_workgroups(workgroups, workgroups, workgroups);

        Ok(())
    }

    /// 执行锥追踪
    pub fn cone_trace(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        output_texture_view: &TextureView,
        gbuffer_position: &TextureView,
        gbuffer_normal: &TextureView,
        sampler: &wgpu::Sampler,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let Some(pipeline) = &self.cone_trace_pipeline else {
            return Ok(());
        };

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VXGI Cone Trace Bind Group"),
            layout: self.cone_trace_bgl.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(output_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(self.voxel_view.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(gbuffer_position),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(gbuffer_normal),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.config_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });

        // 执行锥追踪
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VXGI Cone Trace Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // 计算工作组数量
        let workgroup_size = 8;
        let workgroups_x = (width + workgroup_size - 1) / workgroup_size;
        let workgroups_y = (height + workgroup_size - 1) / workgroup_size;

        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);

        Ok(())
    }

    /// 获取体素纹理视图
    pub fn voxel_view(&self) -> Option<&TextureView> {
        self.voxel_view.as_ref()
    }
}

/// VXGI统一缓冲区
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VxgiUniforms {
    voxel_resolution: u32,
    voxel_size: f32,
    max_trace_distance: f32,
    cone_trace_steps: u32,
    indirect_intensity: f32,
    _padding: [u32; 3],
}

/// 体素化着色器
const VOXELIZATION_SHADER: &str = r#"
@group(0) @binding(0) var voxel_texture: texture_storage_3d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(0) @binding(2) var<uniform> config: VxgiConfig;

struct VxgiConfig {
    voxel_resolution: u32,
    voxel_size: f32,
    max_trace_distance: f32,
    cone_trace_steps: u32,
    indirect_intensity: f32,
    _padding: vec3<u32>,
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = config.voxel_resolution;
    
    if (global_id.x >= resolution || global_id.y >= resolution || global_id.z >= resolution) {
        return;
    }
    
    // 计算世界空间位置
    let world_pos = (vec3<f32>(global_id) / f32(resolution) - 0.5) * config.voxel_size * f32(resolution);
    
    // 体素化场景（简化实现）
    // 实际实现需要从场景数据中读取几何体并体素化
    
    // 写入体素数据
    let color = vec4<f32>(0.5, 0.5, 0.5, 1.0); // 默认颜色
    textureStore(voxel_texture, vec3<i32>(global_id), color);
}
"#;

/// 锥追踪着色器
const CONE_TRACE_SHADER: &str = r#"
@group(0) @binding(0) var output_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1) var voxel_texture: texture_3d<f32>;
@group(0) @binding(2) var voxel_sampler: sampler;
@group(0) @binding(3) var gbuffer_position: texture_2d<f32>;
@group(0) @binding(4) var gbuffer_normal: texture_2d<f32>;
@group(0) @binding(5) var<uniform> config: VxgiConfig;

struct VxgiConfig {
    voxel_resolution: u32,
    voxel_size: f32,
    max_trace_distance: f32,
    cone_trace_steps: u32,
    indirect_intensity: f32,
    _padding: vec3<u32>,
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec2<u32>) {
    let width = textureDimensions(output_texture).x;
    let height = textureDimensions(output_texture).y;
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    let uv = vec2<f32>(global_id) / vec2<f32>(width, height);
    
    // 读取G-Buffer
    let position = textureLoad(gbuffer_position, vec2<i32>(global_id), 0).xyz;
    let normal = normalize(textureLoad(gbuffer_normal, vec2<i32>(global_id), 0).xyz);
    
    // 体素空间坐标
    let voxel_pos = (position / config.voxel_size + f32(config.voxel_resolution) * 0.5) / f32(config.voxel_resolution);
    
    // 锥追踪（简化实现）
    var indirect_light = vec3<f32>(0.0);
    
    // 追踪多个方向
    let num_cones = 6u;
    for (var i = 0u; i < num_cones; i++) {
        // 计算锥方向（基于法线）
        let angle = f32(i) * 2.0 * 3.14159 / f32(num_cones);
        let cone_dir = normalize(normal + vec3<f32>(cos(angle), sin(angle), 0.0) * 0.5);
        
        // 追踪锥
        var trace_pos = voxel_pos;
        var accumulated = vec3<f32>(0.0);
        var occlusion = 1.0;
        
        for (var step = 0u; step < config.cone_trace_steps; step++) {
            let step_size = f32(step + 1u) * config.voxel_size;
            if (step_size > config.max_trace_distance) {
                break;
            }
            
            trace_pos = voxel_pos + cone_dir * step_size / config.voxel_size;
            
            // 采样体素
            let voxel_sample = textureSampleLevel(voxel_texture, voxel_sampler, trace_pos, 0.0);
            
            // 累积光照
            accumulated += voxel_sample.rgb * occlusion;
            occlusion *= 0.9; // 衰减
        }
        
        indirect_light += accumulated / f32(num_cones);
    }
    
    // 写入输出
    let result = vec4<f32>(indirect_light * config.indirect_intensity, 1.0);
    textureStore(output_texture, vec2<i32>(global_id), result);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vxgi_config() {
        let config = VxgiConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.voxel_resolution, 256);
    }

    #[test]
    fn test_voxel() {
        let voxel = Voxel {
            color: [128, 128, 128],
            normal: [64, 64],
            occlusion: 255,
            emissive: 0,
        };
        assert_eq!(voxel.color[0], 128);
    }
}

