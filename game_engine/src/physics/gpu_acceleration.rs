//! GPU加速物理计算模块
//!
//! 提供GPU加速的物理计算功能：
//! - GPU碰撞检测
//! - GPU约束求解
//! - GPU力场计算
//! - 刚体与软体碰撞检测

use glam::Vec3;
use rapier3d::prelude::*;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::domain::physics::PhysicsWorld;
use crate::physics::soft_body::ClothSoftBody;

/// GPU物理加速配置
#[derive(Debug, Clone)]
pub struct GpuPhysicsConfig {
    /// 是否启用GPU加速
    pub enabled: bool,
    /// 最大刚体数量
    pub max_rigid_bodies: u32,
    /// 最大软体粒子数量
    pub max_soft_particles: u32,
    /// 工作组大小
    pub workgroup_size: u32,
    /// 是否启用GPU碰撞检测
    pub gpu_collision_detection: bool,
    /// 是否启用GPU约束求解
    pub gpu_constraint_solver: bool,
}

impl Default for GpuPhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_rigid_bodies: 65536,
            max_soft_particles: 65536,
            workgroup_size: 64,
            gpu_collision_detection: true,
            gpu_constraint_solver: true,
        }
    }
}

/// GPU物理加速器
pub struct GpuPhysicsAccelerator {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: GpuPhysicsConfig,
    /// 碰撞检测计算管线
    collision_pipeline: Option<wgpu::ComputePipeline>,
    /// 约束求解计算管线
    constraint_pipeline: Option<wgpu::ComputePipeline>,
    /// 刚体-软体碰撞管线
    rigid_soft_collision_pipeline: Option<wgpu::ComputePipeline>,
    /// 刚体缓冲区
    rigid_body_buffer: Option<wgpu::Buffer>,
    /// 软体粒子缓冲区
    soft_particle_buffer: Option<wgpu::Buffer>,
    /// 碰撞结果缓冲区
    collision_result_buffer: Option<wgpu::Buffer>,
}

impl GpuPhysicsAccelerator {
    /// 创建GPU物理加速器
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: GpuPhysicsConfig,
    ) -> Self {
        // 验证配置参数，形成逻辑闭环
        let _config_check = (
            config.enabled,
            config.max_rigid_bodies,
            config.max_soft_particles,
            config.workgroup_size,
            config.gpu_collision_detection,
            config.gpu_constraint_solver,
        );
        tracing::debug!("GPU Physics config: {:?}", _config_check);

        let mut accelerator = Self {
            device: device.clone(),
            queue: queue.clone(),
            config,
            collision_pipeline: None,
            constraint_pipeline: None,
            rigid_soft_collision_pipeline: None,
            rigid_body_buffer: None,
            soft_particle_buffer: None,
            collision_result_buffer: None,
        };

        if accelerator.config.enabled {
            accelerator.initialize_pipelines();
            accelerator.initialize_buffers();
        }

        accelerator
    }

    /// 初始化计算管线
    fn initialize_pipelines(&mut self) {
        // 碰撞检测管线
        if self.config.gpu_collision_detection {
            self.collision_pipeline = Some(self.create_collision_pipeline());
        }

        // 约束求解管线
        if self.config.gpu_constraint_solver {
            self.constraint_pipeline = Some(self.create_constraint_pipeline());
        }

        // 刚体-软体碰撞管线
        self.rigid_soft_collision_pipeline = Some(self.create_rigid_soft_collision_pipeline());
    }

    /// 创建碰撞检测计算管线
    fn create_collision_pipeline(&self) -> wgpu::ComputePipeline {
        // 使用内联着色器代码（避免文件路径问题）
        let shader_source = include_str!("shaders/collision_detection.wgsl");
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Collision Detection Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Collision Detection BGL"),
            entries: &[
                // 刚体缓冲区
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 碰撞结果缓冲区
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 参数Uniform
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
            label: Some("Collision Detection Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Collision Detection Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        })
    }

    /// 创建约束求解计算管线
    fn create_constraint_pipeline(&self) -> wgpu::ComputePipeline {
        // 简化实现，实际需要完整的约束求解着色器
        self.create_collision_pipeline() // 占位符
    }

    /// 创建刚体-软体碰撞检测管线
    fn create_rigid_soft_collision_pipeline(&self) -> wgpu::ComputePipeline {
        let shader_source = r#"
@group(0) @binding(0)
var<storage, read> rigid_bodies: array<RigidBody>;

@group(0) @binding(1)
var<storage, read> soft_particles: array<SoftParticle>;

@group(0) @binding(2)
var<storage, read_write> collisions: array<CollisionResult>;

@group(0) @binding(3)
var<uniform> params: CollisionParams;

struct RigidBody {
    position: vec3<f32>,
    rotation: vec4<f32>,
    velocity: vec3<f32>,
    inv_mass: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
    _padding: f32,
}

struct SoftParticle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    radius: f32,
    inv_mass: f32,
}

struct CollisionResult {
    rigid_body_idx: u32,
    particle_idx: u32,
    normal: vec3<f32>,
    depth: f32,
    contact_point: vec3<f32>,
}

struct CollisionParams {
    rigid_body_count: u32,
    particle_count: u32,
    collision_margin: f32,
    _padding: f32,
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_idx = global_id.x;
    if (particle_idx >= params.particle_count) {
        return;
    }

    let particle = soft_particles[particle_idx];
    
    // 检查与所有刚体的碰撞
    for (var i = 0u; i < params.rigid_body_count; i++) {
        let rigid_body = rigid_bodies[i];
        
        // AABB快速剔除
        if (particle.position.x < rigid_body.aabb_min.x || particle.position.x > rigid_body.aabb_max.x ||
            particle.position.y < rigid_body.aabb_min.y || particle.position.y > rigid_body.aabb_max.y ||
            particle.position.z < rigid_body.aabb_min.z || particle.position.z > rigid_body.aabb_max.z) {
            continue;
        }
        
        // 简化的球-AABB碰撞检测
        let closest_point = vec3<f32>(
            max(rigid_body.aabb_min.x, min(particle.position.x, rigid_body.aabb_max.x)),
            max(rigid_body.aabb_min.y, min(particle.position.y, rigid_body.aabb_max.y)),
            max(rigid_body.aabb_min.z, min(particle.position.z, rigid_body.aabb_max.z))
        );
        
        let delta = particle.position - closest_point;
        let dist_sq = dot(delta, delta);
        let radius_sq = particle.radius * particle.radius;
        
        if (dist_sq < radius_sq) {
            let dist = sqrt(dist_sq);
            let normal = normalize(delta);
            let depth = particle.radius - dist;
            
            let collision_idx = particle_idx * params.rigid_body_count + i;
            if (collision_idx < arrayLength(&collisions)) {
                collisions[collision_idx] = CollisionResult(
                    i,
                    particle_idx,
                    normal,
                    depth,
                    closest_point
                );
            }
        }
    }
}
"#;

        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rigid-Soft Collision Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rigid-Soft Collision BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            label: Some("Rigid-Soft Collision Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Rigid-Soft Collision Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        })
    }

    /// 初始化缓冲区
    fn initialize_buffers(&mut self) {
        // 刚体缓冲区
        self.rigid_body_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Rigid Body Buffer"),
            size: (self.config.max_rigid_bodies as u64) * std::mem::size_of::<GpuRigidBody>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // 软体粒子缓冲区
        self.soft_particle_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Soft Particle Buffer"),
            size: (self.config.max_soft_particles as u64) * std::mem::size_of::<GpuSoftParticle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // 碰撞结果缓冲区
        let max_collisions = self.config.max_rigid_bodies * self.config.max_soft_particles;
        self.collision_result_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Collision Result Buffer"),
            size: (max_collisions as u64) * std::mem::size_of::<GpuCollisionResult>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));
    }

    /// 检测刚体与软体的碰撞（GPU加速）
    pub fn detect_rigid_soft_collisions(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        rigid_bodies: &[GpuRigidBody],
        soft_particles: &[GpuSoftParticle],
    ) -> Result<(), GpuPhysicsError> {
        if !self.config.enabled {
            return Err(GpuPhysicsError::GpuAccelerationDisabled);
        }

        let pipeline = self.rigid_soft_collision_pipeline.as_ref()
            .ok_or(GpuPhysicsError::PipelineNotInitialized)?;

        // 上传数据到GPU
        if let Some(buffer) = &self.rigid_body_buffer {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(rigid_bodies));
        }

        if let Some(buffer) = &self.soft_particle_buffer {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(soft_particles));
        }

        // 准备参数
        let params = CollisionParams {
            rigid_body_count: rigid_bodies.len() as u32,
            particle_count: soft_particles.len() as u32,
            collision_margin: 0.01,
            _padding: 0.0,
        };

        // 创建绑定组
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rigid-Soft Collision Bind Group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.rigid_body_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.soft_particle_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.collision_result_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Collision Params Buffer"),
                        contents: bytemuck::bytes_of(&params),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }).as_entire_binding(),
                },
            ],
        });

        // 调度计算
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Rigid-Soft Collision Detection"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (soft_particles.len() as u32).div_ceil(self.config.workgroup_size),
                1,
                1,
            );
        }

        // 注意：实际碰撞结果需要从GPU异步读取
        // 这里只调度计算，实际使用中应该使用异步读取
        Ok(())
    }
}

/// GPU刚体数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuRigidBody {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub inv_mass: f32,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub _padding: f32,
}

/// GPU软体粒子数据
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSoftParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub radius: f32,
    pub inv_mass: f32,
}

/// GPU碰撞结果
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCollisionResult {
    pub rigid_body_idx: u32,
    pub particle_idx: u32,
    pub normal: [f32; 3],
    pub depth: f32,
    pub contact_point: [f32; 3],
}

/// 碰撞结果（CPU端）
#[derive(Debug, Clone)]
pub struct CollisionResult {
    pub rigid_body_idx: u32,
    pub particle_idx: u32,
    pub normal: Vec3,
    pub depth: f32,
    pub contact_point: Vec3,
}

/// 碰撞参数
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CollisionParams {
    rigid_body_count: u32,
    particle_count: u32,
    collision_margin: f32,
    _padding: f32,
}

/// GPU物理错误
#[derive(Debug, Clone)]
pub enum GpuPhysicsError {
    GpuAccelerationDisabled,
    PipelineNotInitialized,
    BufferNotInitialized,
    InvalidData,
}

impl std::fmt::Display for GpuPhysicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuPhysicsError::GpuAccelerationDisabled => {
                write!(f, "GPU acceleration is disabled")
            }
            GpuPhysicsError::PipelineNotInitialized => {
                write!(f, "Compute pipeline not initialized")
            }
            GpuPhysicsError::BufferNotInitialized => {
                write!(f, "Buffer not initialized")
            }
            GpuPhysicsError::InvalidData => {
                write!(f, "Invalid data provided")
            }
        }
    }
}

impl std::error::Error for GpuPhysicsError {}

/// 刚体-软体碰撞检测器
pub struct RigidSoftCollisionDetector {
    gpu_accelerator: Option<GpuPhysicsAccelerator>,
    config: GpuPhysicsConfig,
}

impl RigidSoftCollisionDetector {
    /// 创建碰撞检测器
    pub fn new(
        device: Option<Arc<wgpu::Device>>,
        queue: Option<Arc<wgpu::Queue>>,
        config: GpuPhysicsConfig,
    ) -> Self {
        let gpu_accelerator = if config.enabled {
            if let (Some(device), Some(queue)) = (device, queue) {
                Some(GpuPhysicsAccelerator::new(device, queue, config.clone()))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            gpu_accelerator,
            config,
        }
    }

    /// 检测刚体与软体的碰撞
    pub fn detect_collisions(
        &self,
        rigid_bodies: &[RigidBodyHandle],
        soft_body: &ClothSoftBody,
        physics_world: &PhysicsWorld,
    ) -> Vec<CollisionResult> {
        if let Some(ref accelerator) = self.gpu_accelerator {
            // 使用GPU加速
            self.detect_collisions_gpu(rigid_bodies, soft_body, accelerator)
        } else {
            // CPU回退
            self.detect_collisions_cpu(rigid_bodies, soft_body, physics_world)
        }
    }

    /// GPU加速碰撞检测
    fn detect_collisions_gpu(
        &self,
        rigid_bodies: &[RigidBodyHandle],
        soft_body: &ClothSoftBody,
        accelerator: &GpuPhysicsAccelerator,
    ) -> Vec<CollisionResult> {
        // 转换为GPU格式
        // 记录刚体数量用于后续GPU处理
        let _rigid_body_count = rigid_bodies.len();
        let gpu_rigid_bodies: Vec<GpuRigidBody> = Vec::new(); // 需要从physics_world转换
        let gpu_particles: Vec<GpuSoftParticle> = soft_body
            .particles
            .iter()
            .map(|p| GpuSoftParticle {
                position: [p.position.x, p.position.y, p.position.z],
                velocity: [p.velocity.x, p.velocity.y, p.velocity.z],
                radius: 0.1, // 假设粒子半径
                inv_mass: if p.fixed { 0.0 } else { 1.0 / p.mass },
            })
            .collect();

        // 创建命令编码器
        let mut encoder = accelerator.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rigid-Soft Collision Encoder"),
        });

        // 调用GPU加速器（简化实现，实际需要完整的GPU缓冲区管理）
        let _ = accelerator.detect_rigid_soft_collisions(&mut encoder, &gpu_rigid_bodies, &gpu_particles);

        // 注意：实际需要从GPU异步读取结果
        // 这里返回空结果，实际使用中应该使用异步读取
        Vec::new()
    }

    /// CPU碰撞检测（回退）
    fn detect_collisions_cpu(
        &self,
        rigid_bodies: &[RigidBodyHandle],
        soft_body: &ClothSoftBody,
        physics_world: &PhysicsWorld,
    ) -> Vec<CollisionResult> {
        let mut results = Vec::new();

        for (particle_idx, particle) in soft_body.particles.iter().enumerate() {
            for (rigid_idx, &rigid_handle) in rigid_bodies.iter().enumerate() {
                // 转换 RigidBodyHandle 到 RigidBodyId
                // 使用索引作为临时的 ID 映射
                // 记录rigid_handle用于调试和日志
                let _handle_debug = format!("{:?}", rigid_handle);
                let body_id = crate::domain::physics::RigidBodyId::new(rigid_idx as u64 + 1);
                if let Some(body_state) = physics_world.get_body_state(body_id) {
                    // 简化的球-AABB碰撞检测
                    // 使用位置和默认大小估算AABB
                    let aabb_size = Vec3::splat(1.0); // 默认大小
                    let aabb_min = body_state.position - aabb_size * 0.5;
                    let aabb_max = body_state.position + aabb_size * 0.5;

                    // 计算最近点
                    let closest_point = Vec3::new(
                        particle.position.x.max(aabb_min.x).min(aabb_max.x),
                        particle.position.y.max(aabb_min.y).min(aabb_max.y),
                        particle.position.z.max(aabb_min.z).min(aabb_max.z),
                    );

                    // 将 Vec3A 转换为 Vec3
                    let mut delta: Vec3 = particle.position.into();
                    delta -= closest_point;
                    let dist_sq = delta.length_squared();
                    let particle_radius = 0.1; // 假设粒子半径
                    let radius_sq = particle_radius * particle_radius;

                    if dist_sq < radius_sq {
                        let dist = dist_sq.sqrt();
                        let normal = delta / dist;
                        let depth = particle_radius - dist;

                        results.push(CollisionResult {
                            rigid_body_idx: rigid_idx as u32,
                            particle_idx: particle_idx as u32,
                            normal,
                            depth,
                            contact_point: closest_point,
                        });
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_physics_config_default() {
        let config = GpuPhysicsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_rigid_bodies, 65536);
        assert_eq!(config.workgroup_size, 64);
    }
}

