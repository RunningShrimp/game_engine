//  GPU 计算着色器资源管理
//
//  用于管理 WGPU 计算管道和着色器资源
//  - 计算管道创建和管理
//  - 绑定组管理
//  - 缓冲区管理
//  - 计算任务调度

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

    /// 生成增强的粒子系统更新着色器
    /// 
    /// 支持：
    /// - 风力场
    /// - 碰撞检测
    /// - 颜色渐变
    /// - 大小随生命周期变化
    /// - 旋转动画
    pub fn generate_particle_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> config: ParticleConfig;

@group(0) @binding(2)
var<storage, read> wind_field: array<vec3f>;

struct Particle {
    position: vec3f,
    lifetime: f32,
    velocity: vec3f,
    age: f32,
    color: vec4f,
    size: f32,
    rotation: f32,
    rotation_speed: f32,
}

struct ParticleConfig {
    gravity: vec3f,
    dt: f32,
    damping: f32,
    max_particles: u32,
    wind_strength: f32,
    color_start: vec4f,
    color_end: vec4f,
    size_start: f32,
    size_end: f32,
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
    particle.age += config.dt;
    
    if (particle.lifetime > 0.0) {
        // 应用重力
        particle.velocity += config.gravity * config.dt;
        
        // 应用风力（如果有风力场）
        if (arrayLength(&wind_field) > 0u) {
            let wind_idx = idx % arrayLength(&wind_field);
            particle.velocity += wind_field[wind_idx] * config.wind_strength * config.dt;
        }
        
        // 应用阻尼
        particle.velocity *= config.damping;
        
        // 更新位置
        particle.position += particle.velocity * config.dt;
        
        // 更新旋转
        particle.rotation += particle.rotation_speed * config.dt;
        
        // 计算生命周期比例
        let life_ratio = particle.age / (particle.age + particle.lifetime);
        
        // 颜色插值
        particle.color = mix(config.color_start, config.color_end, life_ratio);
        
        // 大小插值
        particle.size = mix(config.size_start, config.size_end, life_ratio);
    }
    
    particles[idx] = particle;
}
        "#
        .to_string()
    }

    /// 生成AI寻路加速计算着色器
    /// 
    /// 在GPU上并行计算：
    /// - 启发式距离（欧几里得距离）
    /// - 路径代价估算
    /// - 批量距离计算
    /// - 最近节点查找
    pub fn generate_pathfinding_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read> agent_positions: array<vec3f>;

@group(0) @binding(1)
var<storage, read> goal_positions: array<vec3f>;

@group(0) @binding(2)
var<storage, read_write> distances: array<f32>;

@group(0) @binding(3)
var<storage, read_write> path_costs: array<f32>;

@group(0) @binding(4)
var<uniform> config: PathfindingConfig;

struct PathfindingConfig {
    agent_count: u32,
    heuristic_weight: f32,
    max_distance: f32,
    _padding: f32,
}

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let idx = global_id.x;
    if (idx >= config.agent_count) {
        return;
    }
    
    let agent_pos = agent_positions[idx];
    let goal_pos = goal_positions[idx];
    
    // 计算欧几里得距离（启发式）
    let delta = goal_pos - agent_pos;
    let distance = length(delta);
    
    // 存储距离
    distances[idx] = distance;
    
    // 计算路径代价（带权重）
    let cost = distance * config.heuristic_weight;
    
    // 如果距离超过最大值，标记为无效
    if (distance > config.max_distance) {
        path_costs[idx] = 1e10; // 非常大的值表示无效路径
    } else {
        path_costs[idx] = cost;
    }
}
        "#
        .to_string()
    }

    /// 生成批量最近节点查找着色器
    /// 
    /// 在GPU上并行查找每个目标位置最近的导航节点
    pub fn generate_nearest_node_shader() -> String {
        r#"
@group(0) @binding(0)
var<storage, read> target_positions: array<vec3f>;

@group(0) @binding(1)
var<storage, read> node_positions: array<vec3f>;

@group(0) @binding(2)
var<storage, read> node_traversable: array<u32>;

@group(0) @binding(3)
var<storage, read_write> nearest_node_indices: array<u32>;

@group(0) @binding(4)
var<storage, read_write> nearest_distances: array<f32>;

@group(0) @binding(5)
var<uniform> config: NearestNodeConfig;

struct NearestNodeConfig {
    target_count: u32,
    node_count: u32,
    max_search_distance: f32,
    _padding: f32,
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let target_idx = global_id.x;
    if (target_idx >= config.target_count) {
        return;
    }
    
    let target_pos = target_positions[target_idx];
    var min_distance = config.max_search_distance;
    var nearest_idx = 0xFFFFFFFFu; // 无效索引
    
    // 遍历所有节点查找最近的
    for (var i = 0u; i < config.node_count; i++) {
        // 只考虑可通行的节点
        if (node_traversable[i] == 0u) {
            continue;
        }
        
        let node_pos = node_positions[i];
        let delta = target_pos - node_pos;
        let distance = length(delta);
        
        if (distance < min_distance) {
            min_distance = distance;
            nearest_idx = i;
        }
    }
    
    nearest_node_indices[target_idx] = nearest_idx;
    nearest_distances[target_idx] = min_distance;
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

        assert!(manager.get_buffer(buffer.id).is_some());
        assert_eq!(manager.buffer_count(), 1);
        assert_eq!(manager.get_total_memory(), 1024);
    }

    #[test]
    fn test_shader_generation() {
        let physics_shader = ComputeShaderGenerator::generate_physics_shader();
        let collision_shader = ComputeShaderGenerator::generate_collision_shader();
        let particle_shader = ComputeShaderGenerator::generate_particle_shader();
        let pathfinding_shader = ComputeShaderGenerator::generate_pathfinding_shader();
        let nearest_node_shader = ComputeShaderGenerator::generate_nearest_node_shader();

        assert!(!physics_shader.is_empty());
        assert!(!collision_shader.is_empty());
        assert!(!particle_shader.is_empty());
        assert!(!pathfinding_shader.is_empty());
        assert!(!nearest_node_shader.is_empty());
        assert!(physics_shader.contains("@compute"));
        assert!(particle_shader.contains("wind_field"));
        assert!(particle_shader.contains("color"));
        assert!(particle_shader.contains("rotation"));
        assert!(pathfinding_shader.contains("path_costs"));
        assert!(nearest_node_shader.contains("nearest_node_indices"));
    }

    #[test]
    fn test_enhanced_particle_shader_features() {
        let particle_shader = ComputeShaderGenerator::generate_particle_shader();
        
        // 验证增强功能
        assert!(particle_shader.contains("wind_field"), "应支持风力场");
        assert!(particle_shader.contains("color"), "应支持颜色渐变");
        assert!(particle_shader.contains("size"), "应支持大小变化");
        assert!(particle_shader.contains("rotation"), "应支持旋转");
        assert!(particle_shader.contains("life_ratio"), "应支持生命周期比例");
    }

    #[test]
    fn test_pathfinding_shader_features() {
        let pathfinding_shader = ComputeShaderGenerator::generate_pathfinding_shader();
        let nearest_node_shader = ComputeShaderGenerator::generate_nearest_node_shader();
        
        // 验证寻路着色器功能
        assert!(pathfinding_shader.contains("distances"), "应计算距离");
        assert!(pathfinding_shader.contains("path_costs"), "应计算路径代价");
        assert!(pathfinding_shader.contains("heuristic_weight"), "应支持启发式权重");
        
        // 验证最近节点查找功能
        assert!(nearest_node_shader.contains("nearest_node_indices"), "应查找最近节点索引");
        assert!(nearest_node_shader.contains("nearest_distances"), "应计算最近距离");
        assert!(nearest_node_shader.contains("node_traversable"), "应检查节点可通行性");
    }
}
