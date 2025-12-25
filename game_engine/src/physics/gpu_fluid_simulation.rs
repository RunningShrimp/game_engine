//! GPU加速流体模拟（SPH - Smoothed Particle Hydrodynamics）
//!
//! 提供GPU加速的流体模拟功能：
//! - SPH粒子模拟
//! - 密度计算
//! - 压力计算
//! - 粘性力计算
//! - 表面张力

use glam::Vec3;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU流体模拟配置
#[derive(Debug, Clone)]
pub struct GpuFluidSimulationConfig {
    /// 是否启用GPU加速
    pub enabled: bool,
    /// 最大粒子数量
    pub max_particles: u32,
    /// 工作组大小
    pub workgroup_size: u32,
    /// 平滑半径
    pub smoothing_radius: f32,
    /// 静止密度
    pub rest_density: f32,
    /// 压力常数
    pub pressure_constant: f32,
    /// 粘性系数
    pub viscosity: f32,
    /// 表面张力系数
    pub surface_tension: f32,
    /// 时间步长
    pub time_step: f32,
}

impl Default for GpuFluidSimulationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_particles: 16384,
            workgroup_size: 64,
            smoothing_radius: 0.2,
            rest_density: 1000.0,
            pressure_constant: 2000.0,
            viscosity: 0.018,
            surface_tension: 0.0728,
            time_step: 0.001,
        }
    }
}

/// GPU流体粒子数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuFluidParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
    pub _padding: [f32; 2],
}

/// GPU流体模拟器
pub struct GpuFluidSimulator {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: GpuFluidSimulationConfig,
    /// 密度计算管线
    density_pipeline: Option<wgpu::ComputePipeline>,
    /// 压力计算管线
    pressure_pipeline: Option<wgpu::ComputePipeline>,
    /// 力计算管线
    force_pipeline: Option<wgpu::ComputePipeline>,
    /// 更新管线
    update_pipeline: Option<wgpu::ComputePipeline>,
    /// 粒子缓冲区
    particle_buffer: Option<wgpu::Buffer>,
    /// 临时缓冲区（用于密度计算）
    temp_buffer: Option<wgpu::Buffer>,
}

impl GpuFluidSimulator {
    /// 创建GPU流体模拟器
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: GpuFluidSimulationConfig,
    ) -> Self {
        let mut simulator = Self {
            device: device.clone(),
            queue: queue.clone(),
            config,
            density_pipeline: None,
            pressure_pipeline: None,
            force_pipeline: None,
            update_pipeline: None,
            particle_buffer: None,
            temp_buffer: None,
        };

        if simulator.config.enabled {
            simulator.initialize_pipelines();
            simulator.initialize_buffers();
        }

        simulator
    }

    /// 初始化计算管线
    fn initialize_pipelines(&mut self) {
        self.density_pipeline = Some(self.create_density_pipeline());
        self.pressure_pipeline = Some(self.create_pressure_pipeline());
        self.force_pipeline = Some(self.create_force_pipeline());
        self.update_pipeline = Some(self.create_update_pipeline());
    }

    /// 创建密度计算管线
    fn create_density_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/fluid_density.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Fluid Density Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Density BGL"),
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
            label: Some("Fluid Density Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Fluid Density Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 创建压力计算管线
    fn create_pressure_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/fluid_pressure.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Fluid Pressure Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Pressure BGL"),
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
            label: Some("Fluid Pressure Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Fluid Pressure Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 创建力计算管线
    fn create_force_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/fluid_force.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Fluid Force Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Force BGL"),
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
            label: Some("Fluid Force Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Fluid Force Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 创建更新管线
    fn create_update_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = include_str!("shaders/fluid_update.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Fluid Update Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Update BGL"),
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
            label: Some("Fluid Update Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Fluid Update Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
        })
    }

    /// 初始化缓冲区
    fn initialize_buffers(&mut self) {
        self.particle_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Fluid Particle Buffer"),
            size: (self.config.max_particles as u64) * std::mem::size_of::<GpuFluidParticle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        self.temp_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Fluid Temp Buffer"),
            size: (self.config.max_particles as u64) * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    /// 模拟流体
    pub fn simulate(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        particles: &[GpuFluidParticle],
        delta_time: f32,
    ) -> Result<(), GpuFluidSimulationError> {
        if !self.config.enabled {
            return Err(GpuFluidSimulationError::GpuAccelerationDisabled);
        }

        // 上传粒子数据
        if let Some(buffer) = &self.particle_buffer {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(particles));
        }

        // 准备参数
        let params = FluidSimulationParams {
            particle_count: particles.len() as u32,
            delta_time: delta_time.min(self.config.time_step),
            smoothing_radius: self.config.smoothing_radius,
            rest_density: self.config.rest_density,
            pressure_constant: self.config.pressure_constant,
            viscosity: self.config.viscosity,
            surface_tension: self.config.surface_tension,
            gravity: [0.0, -9.81, 0.0],
            _padding: [0u32; 2],
        };

        let params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fluid Simulation Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 1. 计算密度
        if let Some(pipeline) = &self.density_pipeline {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fluid Density Bind Group"),
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
                label: Some("Fluid Density Pass"),
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                1,
                1,
            );
        }

        // 2. 计算压力
        if let Some(pipeline) = &self.pressure_pipeline {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fluid Pressure Bind Group"),
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
                label: Some("Fluid Pressure Pass"),
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                1,
                1,
            );
        }

        // 3. 计算力
        if let Some(pipeline) = &self.force_pipeline {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fluid Force Bind Group"),
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
                label: Some("Fluid Force Pass"),
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (particles.len() as u32 + self.config.workgroup_size - 1) / self.config.workgroup_size,
                1,
                1,
            );
        }

        // 4. 更新粒子
        if let Some(pipeline) = &self.update_pipeline {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fluid Update Bind Group"),
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
                label: Some("Fluid Update Pass"),
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

/// 流体模拟参数
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FluidSimulationParams {
    particle_count: u32,
    delta_time: f32,
    smoothing_radius: f32,
    rest_density: f32,
    pressure_constant: f32,
    viscosity: f32,
    surface_tension: f32,
    gravity: [f32; 3],
    _padding: [u32; 2],
}

/// GPU流体模拟错误
#[derive(Debug, Clone)]
pub enum GpuFluidSimulationError {
    GpuAccelerationDisabled,
    PipelineNotInitialized,
    BufferNotInitialized,
    InvalidData,
}

impl std::fmt::Display for GpuFluidSimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuFluidSimulationError::GpuAccelerationDisabled => {
                write!(f, "GPU fluid simulation acceleration is disabled")
            }
            GpuFluidSimulationError::PipelineNotInitialized => {
                write!(f, "Compute pipeline not initialized")
            }
            GpuFluidSimulationError::BufferNotInitialized => {
                write!(f, "Buffer not initialized")
            }
            GpuFluidSimulationError::InvalidData => {
                write!(f, "Invalid data provided")
            }
        }
    }
}

impl std::error::Error for GpuFluidSimulationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_fluid_simulation_config() {
        let config = GpuFluidSimulationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_particles, 16384);
    }
}

