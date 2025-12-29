//  WGPU GPU 计算集成
//
//  集成 WGPU 和 GPU 计算能力
//  - 计算管道管理
//  - WGSL 着色器编译
//  - GPU 资源管理
//  - 性能监控

use crate::impl_default;
use std::collections::HashMap;

/// GPU 计算设备
pub struct GPUComputeDevice {
    /// 设备特性
    features: GPUFeatures,
    /// 最大工作组大小
    max_workgroup_size: u32,
    /// 最大缓冲区大小
    max_buffer_size: u64,
    /// 支持的格式
    supported_formats: Vec<String>,
}

/// GPU 特性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GPUFeatures {
    /// 是否支持计算着色器
    pub compute_shader: bool,
    /// 是否支持原子操作
    pub atomic_operations: bool,
    /// 是否支持浮点原子
    pub float_atomic: bool,
    /// 是否支持动态索引
    pub dynamic_indexing: bool,
}

impl_default!(GPUFeatures {
    compute_shader: true,
    atomic_operations: true,
    float_atomic: false,
    dynamic_indexing: true,
});

impl GPUComputeDevice {
    /// 创建新的 GPU 计算设备
    pub fn new() -> Self {
        Self {
            features: GPUFeatures::default(),
            max_workgroup_size: 256,
            max_buffer_size: 256 * 1024 * 1024, // 256 MB
            supported_formats: vec![
                "r32float".to_string(),
                "rg32float".to_string(),
                "rgba32float".to_string(),
            ],
        }
    }

    /// 获取 GPU 特性
    pub fn get_features(&self) -> GPUFeatures {
        self.features
    }

    /// 获取最大工作组大小
    pub fn get_max_workgroup_size(&self) -> u32 {
        self.max_workgroup_size
    }

    /// 获取最大缓冲区大小
    pub fn get_max_buffer_size(&self) -> u64 {
        self.max_buffer_size
    }

    /// 检查格式支持
    pub fn supports_format(&self, format: &str) -> bool {
        self.supported_formats.contains(&format.to_string())
    }
}

impl Default for GPUComputeDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// WGSL 着色器源代码
#[derive(Debug, Clone)]
pub struct WGSLShader {
    /// 着色器名称
    pub name: String,
    /// 源代码
    pub source: String,
    /// 入口点
    pub entry_point: String,
}

impl WGSLShader {
    /// 创建新着色器
    pub fn new(name: String, source: String, entry_point: String) -> Self {
        Self {
            name,
            source,
            entry_point,
        }
    }

    /// 为物理计算生成着色器
    pub fn physics_compute() -> Self {
        let source = r#"
@group(0) @binding(0)
var<storage, read_write> positions: array<vec3<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities: array<vec3<f32>>;

@group(0) @binding(2)
var<uniform> gravity: f32;

@group(0) @binding(3)
var<uniform> delta_time: f32;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&positions)) {
        return;
    }
    
    // 应用重力
    velocities[idx].y -= gravity * delta_time;
    
    // 更新位置
    positions[idx] += velocities[idx] * delta_time;
}
"#
        .to_string();

        Self {
            name: "physics_compute".to_string(),
            source,
            entry_point: "main".to_string(),
        }
    }

    /// 为碰撞检测生成着色器
    pub fn collision_compute() -> Self {
        let source = r#"
@group(0) @binding(0)
var<storage, read> positions: array<vec3<f32>>;

@group(0) @binding(1)
var<storage, read_write> collisions: array<atomic<u32>>;

@group(0) @binding(2)
var<uniform> collision_radius: f32;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&positions)) {
        return;
    }
    
    let pos_i = positions[idx];
    var collision_count: u32 = 0u;
    
    // 检查与其他对象的碰撞
    for (var j = idx + 1u; j < arrayLength(&positions); j = j + 1u) {
        let pos_j = positions[j];
        let delta = pos_i - pos_j;
        let dist_sq = dot(delta, delta);
        
        if (dist_sq < collision_radius * collision_radius) {
            collision_count += 1u;
        }
    }
    
    atomicStore(&collisions[idx], collision_count);
}
"#
        .to_string();

        Self {
            name: "collision_compute".to_string(),
            source,
            entry_point: "main".to_string(),
        }
    }

    /// 为粒子模拟生成着色器
    pub fn particle_compute() -> Self {
        let source = r#"
@group(0) @binding(0)
var<storage, read_write> particles: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities: array<vec3<f32>>;

@group(0) @binding(2)
var<uniform> delta_time: f32;

@group(0) @binding(3)
var<uniform> lifetime_decay: f32;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&particles)) {
        return;
    }
    
    var particle = particles[idx];
    
    // 检查粒子是否还活跃
    if (particle.w <= 0.0) {
        return;
    }
    
    // 更新寿命
    particle.w -= lifetime_decay * delta_time;
    
    // 更新位置
    let vel = velocities[idx];
    particle.xyz += vel * delta_time;
    
    // 应用空气阻力
    velocities[idx] *= 0.99;
    
    particles[idx] = particle;
}
"#
        .to_string();

        Self {
            name: "particle_compute".to_string(),
            source,
            entry_point: "main".to_string(),
        }
    }

    /// 为路径规划生成着色器
    pub fn pathfinding_compute() -> Self {
        let source = r#"
@group(0) @binding(0)
var<storage, read> agents: array<vec3<f32>>;

@group(0) @binding(1)
var<storage, read> goals: array<vec3<f32>>;

@group(0) @binding(2)
var<storage, read_write> distances: array<f32>;

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&agents)) {
        return;
    }
    
    let agent_pos = agents[idx];
    let goal_pos = goals[idx];
    
    // 计算欧几里得距离
    let delta = goal_pos - agent_pos;
    distances[idx] = length(delta);
}
"#
        .to_string();

        Self {
            name: "pathfinding_compute".to_string(),
            source,
            entry_point: "main".to_string(),
        }
    }
}

/// GPU 缓冲区包装器
#[derive(Debug, Clone)]
pub struct GPUBuffer {
    /// 缓冲区名称
    pub name: String,
    /// 缓冲区大小（字节）
    pub size: u64,
    /// 是否可读
    pub readable: bool,
    /// 是否可写
    pub writable: bool,
}

impl GPUBuffer {
    /// 创建新缓冲区
    pub fn new(name: String, size: u64, readable: bool, writable: bool) -> Self {
        Self {
            name,
            size,
            readable,
            writable,
        }
    }

    /// 创建存储缓冲区
    pub fn storage(name: String, size: u64) -> Self {
        Self::new(name, size, true, true)
    }

    /// 创建只读缓冲区
    pub fn read_only(name: String, size: u64) -> Self {
        Self::new(name, size, true, false)
    }

    /// 创建统一缓冲区
    pub fn uniform(name: String, size: u64) -> Self {
        Self::new(name, size, true, false)
    }
}

/// 计算管道
pub struct ComputePipelineWGPU {
    /// 管道名称
    pub name: String,
    /// 着色器
    pub shader: WGSLShader,
    /// 绑定缓冲区
    pub buffers: HashMap<u32, GPUBuffer>,
    /// 工作组大小
    pub workgroup_size: (u32, u32, u32),
    /// 是否已编译
    pub compiled: bool,
}

impl ComputePipelineWGPU {
    /// 创建新管道
    pub fn new(name: String, shader: WGSLShader) -> Self {
        Self {
            name,
            shader,
            buffers: HashMap::new(),
            workgroup_size: (256, 1, 1),
            compiled: false,
        }
    }

    /// 绑定缓冲区
    pub fn bind_buffer(&mut self, binding: u32, buffer: GPUBuffer) {
        self.buffers.insert(binding, buffer);
    }

    /// 设置工作组大小
    pub fn set_workgroup_size(&mut self, x: u32, y: u32, z: u32) {
        self.workgroup_size = (x, y, z);
    }

    /// 编译管道
    pub fn compile(&mut self, device: &GPUComputeDevice) -> Result<(), String> {
        // 验证着色器
        if self.shader.source.is_empty() {
            return Err("着色器源代码为空".to_string());
        }

        // 检查工作组大小
        let total_workgroup = self.workgroup_size.0 * self.workgroup_size.1 * self.workgroup_size.2;
        if total_workgroup > device.get_max_workgroup_size() {
            return Err(format!(
                "工作组大小 {} 超过最大值 {}",
                total_workgroup,
                device.get_max_workgroup_size()
            ));
        }

        // 验证绑定
        for buffer in self.buffers.values() {
            if buffer.size > device.get_max_buffer_size() {
                return Err(format!("缓冲区 {} 大小超过最大值", buffer.name));
            }
        }

        self.compiled = true;
        Ok(())
    }

    /// 验证是否可执行
    pub fn can_execute(&self) -> bool {
        self.compiled && !self.buffers.is_empty()
    }

    /// 获取估计的执行时间
    pub fn estimate_execution_time_us(&self) -> f64 {
        let workgroup_size = self.workgroup_size.0 * self.workgroup_size.1 * self.workgroup_size.2;
        let buffer_size: u64 = self.buffers.values().map(|b| b.size).sum();

        // 粗略估计：每 MB 数据约 1 微秒
        let data_time = (buffer_size as f64) / (1024.0 * 1024.0);
        let compute_time = (workgroup_size as f64) / 1000.0;

        data_time + compute_time
    }
}

/// GPU 执行结果
#[derive(Debug, Clone)]
pub struct GPUExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 执行时间（微秒）
    pub duration_us: f64,
    /// 处理的元素数
    pub elements_processed: u64,
    /// 吞吐量（元素/秒）
    pub throughput_per_sec: f64,
    /// 错误消息
    pub error: Option<String>,
}

impl GPUExecutionResult {
    /// 创建成功结果
    pub fn success(duration_us: f64, elements_processed: u64) -> Self {
        let throughput = if duration_us > 0.0 {
            (elements_processed as f64) / (duration_us / 1_000_000.0)
        } else {
            0.0
        };

        Self {
            success: true,
            duration_us,
            elements_processed,
            throughput_per_sec: throughput,
            error: None,
        }
    }

    /// 创建失败结果
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            duration_us: 0.0,
            elements_processed: 0,
            throughput_per_sec: 0.0,
            error: Some(error),
        }
    }
}

/// GPU性能指标
#[derive(Debug, Clone)]
pub struct GPUPerformanceMetrics {
    /// GPU利用率（0.0-1.0）
    pub utilization: f32,
    /// Draw call数量
    pub draw_calls: u32,
    /// 三角形数量
    pub triangles: u64,
    /// 顶点数量
    pub vertices: u64,
    /// GPU内存使用（字节）
    pub memory_used: u64,
    /// GPU内存总量（字节）
    pub memory_total: u64,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// GPU时间（毫秒）
    pub gpu_time_ms: f32,
    /// 渲染管线瓶颈
    pub pipeline_bottleneck: PipelineBottleneck,
}

impl Default for GPUPerformanceMetrics {
    fn default() -> Self {
        Self {
            utilization: 0.0,
            draw_calls: 0,
            triangles: 0,
            vertices: 0,
            memory_used: 0,
            memory_total: 0,
            frame_time_ms: 0.0,
            gpu_time_ms: 0.0,
            pipeline_bottleneck: PipelineBottleneck::None,
        }
    }
}

/// 渲染管线瓶颈
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineBottleneck {
    /// 无瓶颈
    None,
    /// 顶点处理瓶颈
    VertexProcessing,
    /// 片段处理瓶颈
    FragmentProcessing,
    /// 内存带宽瓶颈
    MemoryBandwidth,
    /// 几何处理瓶颈
    GeometryProcessing,
    /// 计算着色器瓶颈
    ComputeShader,
}

/// GPU时间查询器
///
/// 使用wgpu的TIMESTAMP_QUERY特性查询GPU时间戳
pub struct GPUTimeQuery {
    /// 查询集合名称
    pub name: String,
    /// 开始时间戳（纳秒）
    pub start_timestamp_ns: Option<u64>,
    /// 结束时间戳（纳秒）
    pub end_timestamp_ns: Option<u64>,
    /// 历史时间记录
    pub history: Vec<f32>, // 毫秒
}

impl GPUTimeQuery {
    /// 创建新的GPU时间查询器
    pub fn new(name: String) -> Self {
        Self {
            name,
            start_timestamp_ns: None,
            end_timestamp_ns: None,
            history: Vec::with_capacity(100),
        }
    }

    /// 记录开始时间戳
    pub fn record_start(&mut self, timestamp_ns: u64) {
        self.start_timestamp_ns = Some(timestamp_ns);
    }

    /// 记录结束时间戳
    pub fn record_end(&mut self, timestamp_ns: u64) {
        self.end_timestamp_ns = Some(timestamp_ns);

        // 计算时间差并记录
        if let Some(start) = self.start_timestamp_ns {
            let duration_ns = timestamp_ns.saturating_sub(start);
            let duration_ms = duration_ns as f32 / 1_000_000.0;

            self.history.push(duration_ms);

            // 保持历史记录在合理大小
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
    }

    /// 获取平均时间（毫秒）
    pub fn average_time_ms(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f32>() / self.history.len() as f32
    }

    /// 获取最小时间（毫秒）
    pub fn min_time_ms(&self) -> f32 {
        self.history.iter().copied().fold(f32::INFINITY, f32::min)
    }

    /// 获取最大时间（毫秒）
    pub fn max_time_ms(&self) -> f32 {
        self.history.iter().copied().fold(0.0, f32::max)
    }

    /// 获取最后时间（毫秒）
    pub fn last_time_ms(&self) -> Option<f32> {
        self.history.last().copied()
    }

    /// 重置查询器
    pub fn reset(&mut self) {
        self.start_timestamp_ns = None;
        self.end_timestamp_ns = None;
        self.history.clear();
    }
}

/// GPU性能分析器
///
/// 收集和分析GPU性能指标
pub struct GPUProfiler {
    /// 时间查询器映射
    time_queries: std::collections::HashMap<String, GPUTimeQuery>,
    /// 当前性能指标
    metrics: GPUPerformanceMetrics,
    /// Draw call统计
    draw_call_count: u32,
    /// 三角形统计
    triangle_count: u64,
    /// 顶点统计
    vertex_count: u64,
}

impl GPUProfiler {
    /// 创建新的GPU性能分析器
    pub fn new() -> Self {
        Self {
            time_queries: std::collections::HashMap::new(),
            metrics: GPUPerformanceMetrics::default(),
            draw_call_count: 0,
            triangle_count: 0,
            vertex_count: 0,
        }
    }

    /// 开始时间查询
    pub fn start_query(&mut self, name: &str) {
        // 在实际实现中，这里应该调用wgpu的timestamp query
        // 这里使用占位实现
        let query = self
            .time_queries
            .entry(name.to_string())
            .or_insert_with(|| GPUTimeQuery::new(name.to_string()));

        // 模拟时间戳（实际应该从GPU获取）
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Test: operation should succeed")
            .as_nanos() as u64;
        query.record_start(timestamp_ns);
    }

    /// 结束时间查询
    pub fn end_query(&mut self, name: &str) {
        if let Some(query) = self.time_queries.get_mut(name) {
            let timestamp_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Test: operation should succeed")
                .as_nanos() as u64;
            query.record_end(timestamp_ns);
        }
    }

    /// 记录draw call
    pub fn record_draw_call(&mut self, count: u32) {
        self.draw_call_count += count;
    }

    /// 记录三角形
    pub fn record_triangles(&mut self, count: u64) {
        self.triangle_count += count;
    }

    /// 记录顶点
    pub fn record_vertices(&mut self, count: u64) {
        self.vertex_count += count;
    }

    /// 更新性能指标
    pub fn update_metrics(&mut self) {
        // 计算GPU利用率（简化实现）
        let gpu_time = self.time_queries.values().filter_map(|q| q.last_time_ms()).sum::<f32>();

        self.metrics.gpu_time_ms = gpu_time;
        self.metrics.draw_calls = self.draw_call_count;
        self.metrics.triangles = self.triangle_count;
        self.metrics.vertices = self.vertex_count;

        // 估算利用率（简化实现）
        if self.metrics.frame_time_ms > 0.0 {
            self.metrics.utilization =
                (self.metrics.gpu_time_ms / self.metrics.frame_time_ms).min(1.0);
        }

        // 检测瓶颈
        self.metrics.pipeline_bottleneck = self.detect_bottleneck();
    }

    /// 检测渲染管线瓶颈
    fn detect_bottleneck(&self) -> PipelineBottleneck {
        // 简化实现：基于时间查询检测瓶颈
        // 实际实现应该分析各个阶段的GPU时间

        if let Some(vertex_time) = self.time_queries.get("vertex_processing")
            && let Some(time) = vertex_time.last_time_ms()
            && time > 5.0
        {
            return PipelineBottleneck::VertexProcessing;
        }

        if let Some(fragment_time) = self.time_queries.get("fragment_processing")
            && let Some(time) = fragment_time.last_time_ms()
            && time > 5.0
        {
            return PipelineBottleneck::FragmentProcessing;
        }

        PipelineBottleneck::None
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> &GPUPerformanceMetrics {
        &self.metrics
    }

    /// 获取时间查询结果
    pub fn get_query_result(&self, name: &str) -> Option<&GPUTimeQuery> {
        self.time_queries.get(name)
    }

    /// 重置统计
    pub fn reset_frame(&mut self) {
        self.draw_call_count = 0;
        self.triangle_count = 0;
        self.vertex_count = 0;
    }

    /// 设置帧时间
    pub fn set_frame_time(&mut self, frame_time_ms: f32) {
        self.metrics.frame_time_ms = frame_time_ms;
    }
}

impl Default for GPUProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能对比结果
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// 操作名称
    pub operation: String,
    /// CPU 执行时间（微秒）
    pub cpu_time_us: f64,
    /// GPU 执行时间（微秒）
    pub gpu_time_us: f64,
    /// 加速比
    pub speedup: f64,
    /// 是否值得 GPU 加速
    pub recommended: bool,
}

impl PerformanceComparison {
    /// 创建性能对比
    pub fn new(operation: String, cpu_time_us: f64, gpu_time_us: f64) -> Self {
        let speedup = if gpu_time_us > 0.0 {
            cpu_time_us / gpu_time_us
        } else {
            0.0
        };

        // 加速比 > 2x 推荐使用 GPU
        let recommended = speedup > 2.0;

        Self {
            operation,
            cpu_time_us,
            gpu_time_us,
            speedup,
            recommended,
        }
    }

    /// 获取性能改进百分比
    pub fn improvement_percent(&self) -> f64 {
        if self.cpu_time_us > 0.0 {
            ((self.cpu_time_us - self.gpu_time_us) / self.cpu_time_us) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_device() {
        let device = GPUComputeDevice::new();
        assert!(device.get_features().compute_shader);
        assert_eq!(device.get_max_workgroup_size(), 256);
    }

    #[test]
    fn test_gpu_features() {
        let features = GPUFeatures::default();
        assert!(features.compute_shader);
        assert!(features.atomic_operations);
    }

    #[test]
    fn test_wgsl_shaders() {
        let physics = WGSLShader::physics_compute();
        assert!(!physics.source.is_empty());
        assert_eq!(physics.name, "physics_compute");

        let collision = WGSLShader::collision_compute();
        assert!(!collision.source.is_empty());

        let particle = WGSLShader::particle_compute();
        assert!(!particle.source.is_empty());

        let pathfinding = WGSLShader::pathfinding_compute();
        assert!(!pathfinding.source.is_empty());
    }

    #[test]
    fn test_gpu_buffer() {
        let buffer = GPUBuffer::storage("test".to_string(), 1024);
        assert!(buffer.writable);
        assert!(buffer.readable);
        assert_eq!(buffer.size, 1024);
    }

    #[test]
    fn test_compute_pipeline() {
        let device = GPUComputeDevice::new();
        let shader = WGSLShader::physics_compute();
        let mut pipeline = ComputePipelineWGPU::new("physics".to_string(), shader);

        pipeline.bind_buffer(0, GPUBuffer::storage("positions".to_string(), 1024 * 1024));
        pipeline.compile(&device).expect("Test: operation should succeed");

        assert!(pipeline.compiled);
        assert!(pipeline.can_execute());
    }

    #[test]
    fn test_execution_result() {
        let result = GPUExecutionResult::success(100.0, 1000);
        assert!(result.success);
        assert!(result.throughput_per_sec > 0.0);
    }

    #[test]
    fn test_performance_comparison() {
        let comp = PerformanceComparison::new("test".to_string(), 1000.0, 100.0);
        assert_eq!(comp.speedup, 10.0);
        assert!(comp.recommended);
        assert!(comp.improvement_percent() > 0.0);
    }
}
