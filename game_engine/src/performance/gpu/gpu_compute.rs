//! GPU 计算着色器资源管理
//!
//! 用于管理 WGPU 计算管道和着色器资源
//! - 计算管道创建和管理
//! - 绑定组管理
//! - 缓冲区管理
//! - 计算任务调度

use std::fmt;
use std::sync::Arc;

/// 计算着色器配置
#[derive(Debug, Clone)]
pub struct ComputeShaderConfig {
    /// 着色器源代码
    pub shader_code: String,
    /// 着色器入口点
    pub entry_point: String,
    /// 工作组大小 (x, y, z)
    pub workgroup_size: (u32, u32, u32),
    /// 工作组数量 (x, y, z)
    pub workgroup_count: (u32, u32, u32),
}

impl ComputeShaderConfig {
    /// 创建新的计算着色器配置
    pub fn new(shader_code: String) -> Self {
        Self {
            shader_code,
            entry_point: "main".to_string(),
            workgroup_size: (8, 8, 1),
            workgroup_count: (1, 1, 1),
        }
    }

    /// 设置入口点
    pub fn with_entry_point(mut self, entry_point: String) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// 设置工作组大小
    pub fn with_workgroup_size(mut self, x: u32, y: u32, z: u32) -> Self {
        self.workgroup_size = (x, y, z);
        self
    }

    /// 设置工作组数量
    pub fn with_workgroup_count(mut self, x: u32, y: u32, z: u32) -> Self {
        self.workgroup_count = (x, y, z);
        self
    }
}

/// 绑定组条目
#[derive(Debug, Clone)]
pub struct BindGroupEntry {
    /// 绑定点位置
    pub binding: u32,
    /// 缓冲区大小 (字节)
    pub buffer_size: u64,
    /// 缓冲区类型 (0=均匀, 1=存储读, 2=存储读写)
    pub buffer_type: u32,
}

/// GPU 缓冲区
pub struct GPUBuffer {
    /// 缓冲区标识
    pub id: u32,
    /// 缓冲区大小
    pub size: u64,
    /// 缓冲区类型
    pub buffer_type: u32,
    /// 是否需要同步
    pub needs_sync: bool,
}

impl GPUBuffer {
    /// 创建新的 GPU 缓冲区
    pub fn new(id: u32, size: u64, buffer_type: u32) -> Self {
        Self {
            id,
            size,
            buffer_type,
            needs_sync: true,
        }
    }
}

/// GPU 计算管道
pub struct ComputePipeline {
    /// 管道标识
    pub id: u32,
    /// 着色器配置
    pub config: ComputeShaderConfig,
    /// 绑定组条目
    pub bind_groups: Vec<BindGroupEntry>,
    /// 缓冲区
    pub buffers: Vec<Arc<GPUBuffer>>,
    /// 是否已编译
    pub compiled: bool,
}

impl ComputePipeline {
    /// 创建新的计算管道
    pub fn new(id: u32, config: ComputeShaderConfig) -> Self {
        Self {
            id,
            config,
            bind_groups: Vec::new(),
            buffers: Vec::new(),
            compiled: false,
        }
    }

    /// 添加绑定组条目
    pub fn add_bind_group(&mut self, binding: u32, buffer_size: u64, buffer_type: u32) {
        self.bind_groups.push(BindGroupEntry {
            binding,
            buffer_size,
            buffer_type,
        });
    }

    /// 编译管道 (占位符)
    pub fn compile(&mut self) -> Result<(), String> {
        // 在实际实现中会使用 WGPU 编译着色器
        self.compiled = true;
        Ok(())
    }

    /// 执行计算 (占位符)
    pub fn execute(&self) -> Result<(), String> {
        if !self.compiled {
            return Err("Pipeline not compiled".to_string());
        }

        // 在实际实现中会调用 GPU 计算
        Ok(())
    }
}

impl fmt::Debug for ComputePipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComputePipeline")
            .field("id", &self.id)
            .field("compiled", &self.compiled)
            .field("buffers", &self.buffers.len())
            .field("bind_groups", &self.bind_groups.len())
            .finish()
    }
}

/// GPU 计算资源管理器
pub struct ComputeResourceManager {
    /// 已创建的管道
    pipelines: Vec<Arc<ComputePipeline>>,
    /// 已创建的缓冲区
    buffers: Vec<Arc<GPUBuffer>>,
    /// 下一个管道 ID
    next_pipeline_id: u32,
    /// 下一个缓冲区 ID
    next_buffer_id: u32,
    /// 总 GPU 内存使用量
    total_gpu_memory: u64,
}

impl Default for ComputeResourceManager {
    fn default() -> Self {
        Self {
            pipelines: Vec::new(),
            buffers: Vec::new(),
            next_pipeline_id: 0,
            next_buffer_id: 0,
            total_gpu_memory: 0,
        }
    }
}

impl ComputeResourceManager {
    /// 创建新的资源管理器
    pub fn new() -> Self {
        Self {
            next_pipeline_id: 1,
            next_buffer_id: 1,
            ..Default::default()
        }
    }

    /// 创建计算管道
    pub fn create_pipeline(&mut self, config: ComputeShaderConfig) -> Arc<ComputePipeline> {
        let id = self.next_pipeline_id;
        self.next_pipeline_id += 1;

        let pipeline = Arc::new(ComputePipeline::new(id, config));
        self.pipelines.push(pipeline.clone());
        pipeline
    }

    /// 创建 GPU 缓冲区
    pub fn create_buffer(&mut self, size: u64, buffer_type: u32) -> Arc<GPUBuffer> {
        let id = self.next_buffer_id;
        self.next_buffer_id += 1;

        let buffer = Arc::new(GPUBuffer::new(id, size, buffer_type));
        self.total_gpu_memory += size;
        self.buffers.push(buffer.clone());
        buffer
    }

    /// 获取管道
    pub fn get_pipeline(&self, id: u32) -> Option<Arc<ComputePipeline>> {
        self.pipelines.iter().find(|p| p.id == id).cloned()
    }

    /// 获取缓冲区
    pub fn get_buffer(&self, id: u32) -> Option<Arc<GPUBuffer>> {
        self.buffers.iter().find(|b| b.id == id).cloned()
    }

    /// 获取总 GPU 内存使用量
    pub fn get_total_memory(&self) -> u64 {
        self.total_gpu_memory
    }

    /// 获取管道数量
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }

    /// 获取缓冲区数量
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

/// WGSL (WebGPU Shading Language) 计算着色器生成器
pub struct ComputeShaderGenerator;

impl ComputeShaderGenerator {
    /// 生成物理模拟计算着色器
    pub fn generate_physics_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read_write> bodies: array<PhysicsBody>;

@group(0) @binding(1)
var<uniform> config: PhysicsConfig;

struct PhysicsBody {
    position: vec3f,
    inv_mass: f32,
    velocity: vec3f,
    angular_velocity: f32,
    force: vec3f,
    _padding: f32,
}

struct PhysicsConfig {
    gravity: vec3f,
    dt: f32,
    damping: f32,
    _padding: [f32; 3],
}

@compute @workgroup_size(8, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let idx = global_id.x;
    if (idx >= arrayLength(&bodies)) {
        return;
    }
    
    var body = bodies[idx];
    
    // 跳过固定物体
    if (body.inv_mass <= 0.0) {
        return;
    }
    
    // 应用力和重力
    let accel = (body.force + config.gravity) * body.inv_mass;
    body.velocity += accel * config.dt;
    body.velocity *= config.damping;
    body.position += body.velocity * config.dt;
    body.force = vec3f(0.0);
    
    bodies[idx] = body;
}
        "#
        .to_string()
    }

    /// 生成碰撞检测计算着色器
    pub fn generate_collision_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read> bodies: array<PhysicsBody>;

@group(0) @binding(1)
var<storage, read_write> collisions: array<CollisionInfo>;

@group(0) @binding(2)
var<uniform> params: CollisionParams;

struct PhysicsBody {
    position: vec3f,
    inv_mass: f32,
    velocity: vec3f,
    angular_velocity: f32,
    force: vec3f,
    _padding: f32,
}

struct CollisionInfo {
    body_a: u32,
    body_b: u32,
    normal: vec3f,
    depth: f32,
}

struct CollisionParams {
    body_count: u32,
    collision_margin: f32,
    _padding: [f32; 2],
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let idx_a = global_id.x;
    let idx_b = global_id.y;
    
    if (idx_a >= params.body_count || idx_b >= params.body_count || idx_a >= idx_b) {
        return;
    }
    
    let body_a = bodies[idx_a];
    let body_b = bodies[idx_b];
    
    let delta = body_b.position - body_a.position;
    let dist = length(delta);
    let min_dist = 1.0 + params.collision_margin;
    
    if (dist < min_dist) {
        let normal = normalize(delta);
        let depth = min_dist - dist;
        
        let collision_idx = idx_a * params.body_count + idx_b;
        if (collision_idx < arrayLength(&collisions)) {
            collisions[collision_idx] = CollisionInfo(
                idx_a,
                idx_b,
                normal,
                depth
            );
        }
    }
}
        "#
        .to_string()
    }

    /// 生成粒子系统更新着色器
    pub fn generate_particle_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> config: ParticleConfig;

struct Particle {
    position: vec3f,
    lifetime: f32,
    velocity: vec3f,
    _padding: f32,
}

struct ParticleConfig {
    gravity: vec3f,
    dt: f32,
    damping: f32,
    max_particles: u32,
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let idx = global_id.x;
    if (idx >= config.max_particles) {
        return;
    }
    
    var particle = particles[idx];
    
    // 更新生命周期
    particle.lifetime -= config.dt;
    
    if (particle.lifetime > 0.0) {
        // 应用重力
        particle.velocity += config.gravity * config.dt;
        particle.velocity *= config.damping;
        particle.position += particle.velocity * config.dt;
    }
    
    particles[idx] = particle;
}
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_pipeline_creation() {
        let config = ComputeShaderConfig::new("shader code".to_string());
        let pipeline = ComputePipeline::new(1, config);

        assert_eq!(pipeline.id, 1);
        assert!(!pipeline.compiled);
    }

    #[test]
    fn test_compute_resource_manager() {
        let mut manager = ComputeResourceManager::new();

        let config = ComputeShaderConfig::new("shader".to_string());
        let pipeline = manager.create_pipeline(config);

        assert_eq!(manager.pipeline_count(), 1);
        assert!(manager.get_pipeline(pipeline.id).is_some());
    }

    #[test]
    fn test_buffer_creation() {
        let mut manager = ComputeResourceManager::new();

        let buffer = manager.create_buffer(1024, 0);

        assert_eq!(manager.buffer_count(), 1);
        assert_eq!(manager.get_total_memory(), 1024);
    }

    #[test]
    fn test_shader_generation() {
        let physics_shader = ComputeShaderGenerator::generate_physics_shader();
        let collision_shader = ComputeShaderGenerator::generate_collision_shader();
        let particle_shader = ComputeShaderGenerator::generate_particle_shader();

        assert!(!physics_shader.is_empty());
        assert!(!collision_shader.is_empty());
        assert!(!particle_shader.is_empty());
        assert!(physics_shader.contains("@compute"));
    }
}
