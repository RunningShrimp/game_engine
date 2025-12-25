//! GPU加速粒子物理模拟
//!
//! 提供GPU加速的粒子物理功能：
//! - 粒子碰撞检测
//! - 粒子力场计算
//! - 粒子约束求解
//! - 大规模粒子系统支持

use glam::Vec3;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU粒子物理配置
#[derive(Debug, Clone)]
pub struct GpuParticlePhysicsConfig {
    /// 是否启用GPU加速
    pub enabled: bool,
    /// 最大粒子数量
    pub max_particles: u32,
    /// 工作组大小
    pub workgroup_size: u32,
    /// 碰撞检测半径
    pub collision_radius: f32,
    /// 粒子间相互作用范围
    pub interaction_radius: f32,
    /// 是否启用粒子碰撞
    pub enable_collision: bool,
    /// 是否启用力场
    pub enable_force_fields: bool,
}

impl Default for GpuParticlePhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_particles: 65536,
            workgroup_size: 64,
            collision_radius: 0.1,
            interaction_radius: 0.5,
            enable_collision: true,
            enable_force_fields: true,
        }
    }
}

/// GPU粒子数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub force: [f32; 3],
    pub mass: f32,
    pub radius: f32,
    pub _padding: [f32; 2],
}

/// GPU粒子物理加速器
pub struct GpuParticlePhysicsAccelerator {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: GpuParticlePhysicsConfig,
    /// 粒子碰撞检测管线
    collision_pipeline: Option<wgpu::ComputePipeline>,
    /// 力场计算管线
    force_field_pipeline: Option<wgpu::ComputePipeline>,
    /// 粒子更新管线
    update_pipeline: Option<wgpu::ComputePipeline>,
    /// 粒子缓冲区
    particle_buffer: Option<wgpu::Buffer>,
    /// 力场缓冲区
    force_field_buffer: Option<wgpu::Buffer>,
}

impl GpuParticlePhysicsAccelerator {
    /// 创建GPU粒子物理加速器
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: GpuParticlePhysicsConfig,
    ) -> Self {
        let mut accelerator = Self {
            device: device.clone(),
            queue: queue.clone(),
            config,
            collision_pipeline: None,
            force_field_pipeline: None,
            update_pipeline: None,
            particle_buffer: None,
            force_field_buffer: None,
        };

        if accelerator.config.enabled {
            accelerator.initialize_pipelines();
            accelerator.initialize_buffers();
        }

        accelerator
    }

    /// 初始化计算管线
    fn initialize_pipelines(&mut self) {
        if self.config.enable_collision {
            self.collision_pipeline = Some(self.create_collision_pipeline());
        }

        if self.config.enable_force_fields {
            self.force_field_pipeline = Some(self.create_force_field_pipeline());
        }

        self.update_pipeline = Some(self.create_update_pipeline());
    }

    /// 创建碰撞检测管线
    fn create_collision_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/particle_collision.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Particle Collision Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Collision BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Collision Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Collision Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 创建力场计算管线
    fn create_force_field_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/particle_force_field.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Particle Force Field Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Force Field BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
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

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Force Field Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Force Field Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 创建粒子更新管线
    fn create_update_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/particle_update.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Particle Update Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Update BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Update Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Update Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 初始化缓冲区
    fn initialize_buffers(&mut self) {
        self.particle_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Particle Buffer"),
            size: (self.config.max_particles as u64) * std::mem::size_of::<GpuParticle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        // 力场缓冲区（用于存储力场数据）
        self.force_field_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Force Field Buffer"),
            size: 1024 * 1024, // 1MB，可根据需要调整
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    /// 更新粒子物理
    pub fn update_particles(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        particles: &[GpuParticle],
        delta_time: f32,
    ) -> Result<(), GpuParticlePhysicsError> {
        if !self.config.enabled {
            return Err(GpuParticlePhysicsError::GpuAccelerationDisabled);
        }

        // 上传粒子数据
        if let Some(buffer) = &self.particle_buffer {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(particles));
        }

        // 准备参数
        let params = ParticlePhysicsParams {
            particle_count: particles.len() as u32,
            delta_time,
            collision_radius: self.config.collision_radius,
            interaction_radius: self.config.interaction_radius,
            gravity: [0.0, -9.81, 0.0],
            _padding: [0u32; 2],
        };

        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Physics Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 执行力场计算
        if self.config.enable_force_fields {
            if let Some(pipeline) = &self.force_field_pipeline {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Particle Force Field Bind Group"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.particle_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.force_field_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: params_buffer.as_entire_binding(),
                        },
                    ],
                });

                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Particle Force Field Pass"),
                });
                cpass.set_pipeline(pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(
                    (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                    1,
                    1,
                );
            }
        }

        // 执行碰撞检测
        if self.config.enable_collision {
            if let Some(pipeline) = &self.collision_pipeline {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Particle Collision Bind Group"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.particle_buffer.as_ref().unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: params_buffer.as_entire_binding(),
                        },
                    ],
                });

                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Particle Collision Pass"),
                });
                cpass.set_pipeline(pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(
                    (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                    1,
                    1,
                );
            }
        }

        // 执行粒子更新
        if let Some(pipeline) = &self.update_pipeline {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Particle Update Bind Group"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.particle_buffer.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Update Pass"),
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                1,
                1,
            );
        }

        Ok(())
    }

    /// 获取粒子缓冲区
    pub fn particle_buffer(&self) -> Option<&wgpu::Buffer> {
        self.particle_buffer.as_ref()
    }
}

/// 粒子物理参数
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticlePhysicsParams {
    particle_count: u32,
    delta_time: f32,
    collision_radius: f32,
    interaction_radius: f32,
    gravity: [f32; 3],
    _padding: [u32; 2],
}

/// GPU粒子物理错误
#[derive(Debug, Clone)]
pub enum GpuParticlePhysicsError {
    GpuAccelerationDisabled,
    PipelineNotInitialized,
    BufferNotInitialized,
    InvalidData,
}

impl std::fmt::Display for GpuParticlePhysicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuParticlePhysicsError::GpuAccelerationDisabled => {
                write!(f, "GPU particle physics acceleration is disabled")
            }
            GpuParticlePhysicsError::PipelineNotInitialized => {
                write!(f, "Compute pipeline not initialized")
            }
            GpuParticlePhysicsError::BufferNotInitialized => {
                write!(f, "Buffer not initialized")
            }
            GpuParticlePhysicsError::InvalidData => {
                write!(f, "Invalid data provided")
            }
        }
    }
}

impl std::error::Error for GpuParticlePhysicsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_particle_physics_config() {
        let config = GpuParticlePhysicsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_particles, 65536);
    }
}

