/// 渲染管线优化
///
/// 优化 GPU 渲染管线性能：
/// - 绘制调用合并
/// - 状态缓存
/// - 延迟渲染管线
/// - GPU 命令缓冲区管理
use crate::impl_default;
use std::collections::HashMap;

/// 渲染命令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderCommandType {
    DrawIndexed,
    DrawInstanced,
    Dispatch,
    SetPipeline,
    SetBindGroup,
    SetScissor,
}

/// 渲染状态缓存键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderStateKey {
    pub pipeline_id: u32,
    pub bind_group_id: u32,
    pub blend_mode: u8,
    pub depth_test: bool,
}

/// GPU 绘制命令
#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub command_type: RenderCommandType,
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub index_count: u32,
    pub index_offset: u32,
}

/// 渲染状态缓存
#[derive(Default)]
pub struct RenderStateCache {
    /// 状态到批次的映射
    state_to_batch: HashMap<RenderStateKey, usize>,
    /// 状态访问频率统计
    state_frequency: HashMap<RenderStateKey, u32>,
    /// 最大缓存大小
    max_cache_size: usize,
}

impl RenderStateCache {
    /// 创建新的状态缓存
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            state_to_batch: HashMap::with_capacity(max_cache_size),
            state_frequency: HashMap::new(),
            max_cache_size,
        }
    }

    /// 获取状态对应的批次索引
    pub fn get_batch_index(&self, state: &RenderStateKey) -> Option<usize> {
        self.state_to_batch.get(state).copied()
    }

    /// 注册状态到批次映射
    pub fn register_state(&mut self, state: RenderStateKey, batch_index: usize) {
        // 更新访问频率
        *self.state_frequency.entry(state).or_insert(0) += 1;

        // 如果缓存未满，直接添加
        if self.state_to_batch.len() < self.max_cache_size {
            self.state_to_batch.insert(state, batch_index);
        } else {
            // 缓存已满，使用LRU策略：移除最少使用的状态
            if let Some(least_used) = self.find_least_used_state() {
                self.state_to_batch.remove(&least_used);
                self.state_frequency.remove(&least_used);
                self.state_to_batch.insert(state, batch_index);
            }
        }
    }

    /// 查找最少使用的状态
    fn find_least_used_state(&self) -> Option<RenderStateKey> {
        self.state_frequency
            .iter()
            .min_by_key(|&(_, &freq)| freq)
            .map(|(state, _)| *state)
    }

    /// 清除缓存
    pub fn clear(&mut self) {
        self.state_to_batch.clear();
        self.state_frequency.clear();
    }

    /// 获取缓存命中率（需要外部跟踪）
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.state_to_batch.len(), self.max_cache_size)
    }
}

/// 绘制调用优化器
#[derive(Default)]
pub struct DrawCallOptimizer {
    state_cache: RenderStateCache,
    command_batches: Vec<Vec<DrawCommand>>,
    current_state: Option<RenderStateKey>,
    state_changes: u32,
    /// 原始绘制调用数（用于计算优化率）
    original_draw_calls: u32,
}

impl DrawCallOptimizer {
    pub fn new() -> Self {
        Self {
            state_cache: RenderStateCache::new(256),
            command_batches: Vec::new(),
            current_state: None,
            state_changes: 0,
            original_draw_calls: 0,
        }
    }

    /// 添加绘制命令到批次
    pub fn add_command(&mut self, command: DrawCommand, state: RenderStateKey) {
        self.original_draw_calls += 1;

        // 检查状态缓存
        let batch_index = if let Some(idx) = self.state_cache.get_batch_index(&state) {
            // 缓存命中，使用现有批次
            idx
        } else {
            // 缓存未命中，创建新批次
            let new_index = self.command_batches.len();
            self.command_batches.push(Vec::new());
            self.state_cache.register_state(state, new_index);
            new_index
        };

        // 检查状态是否改变
        if self.current_state != Some(state) {
            self.current_state = Some(state);
            self.state_changes += 1;
        }

        // 添加命令到对应批次
        if let Some(batch) = self.command_batches.get_mut(batch_index) {
            batch.push(command);
        }
    }

    /// 获取优化后的批次数
    pub fn get_batch_count(&self) -> usize {
        self.command_batches.len()
    }

    /// 获取状态改变次数
    pub fn get_state_changes(&self) -> u32 {
        self.state_changes
    }

    /// 计算优化比率（原始vs优化后的批次数）
    pub fn get_optimization_ratio(&self) -> f32 {
        if self.original_draw_calls == 0 {
            0.0
        } else {
            1.0 - (self.command_batches.len() as f32) / (self.original_draw_calls as f32)
        }
    }

    /// 获取原始绘制调用数
    pub fn get_original_draw_calls(&self) -> u32 {
        self.original_draw_calls
    }

    /// 获取优化后的批次数
    pub fn get_optimized_batch_count(&self) -> usize {
        self.command_batches.len()
    }

    /// 获取状态缓存统计
    pub fn get_state_cache_stats(&self) -> (usize, usize) {
        self.state_cache.get_cache_stats()
    }

    pub fn clear(&mut self) {
        self.state_cache.clear();
        self.command_batches.clear();
        self.current_state = None;
        self.state_changes = 0;
        self.original_draw_calls = 0;
    }
}

/// GPU 命令缓冲区
#[derive(Default)]
pub struct CommandBuffer {
    commands: Vec<RenderCommand>,
    is_recording: bool,
}

#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub name: String,
    pub command_type: RenderCommandType,
    pub data: Vec<u8>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始录制命令
    pub fn begin_recording(&mut self) {
        self.is_recording = true;
        self.commands.clear();
    }

    /// 结束录制
    pub fn end_recording(&mut self) {
        self.is_recording = false;
    }

    /// 添加命令
    pub fn add_command(&mut self, command: RenderCommand) {
        if self.is_recording {
            self.commands.push(command);
        }
    }

    /// 获取命令数量
    pub fn get_command_count(&self) -> usize {
        self.commands.len()
    }

    /// 估算命令缓冲区大小
    pub fn estimate_size(&self) -> usize {
        self.commands.iter().map(|cmd| cmd.data.len()).sum()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// 延迟渲染信息
#[derive(Debug, Clone)]
pub struct DeferredRenderingInfo {
    pub g_buffer_count: u32,
    pub light_count: u32,
    pub geometry_pass_time_ms: f32,
    pub lighting_pass_time_ms: f32,
    pub composition_pass_time_ms: f32,
}

impl DeferredRenderingInfo {
    pub fn total_time_ms(&self) -> f32 {
        self.geometry_pass_time_ms + self.lighting_pass_time_ms + self.composition_pass_time_ms
    }

    pub fn geometry_time_ratio(&self) -> f32 {
        self.geometry_pass_time_ms / self.total_time_ms().max(0.001)
    }

    pub fn lighting_time_ratio(&self) -> f32 {
        self.lighting_pass_time_ms / self.total_time_ms().max(0.001)
    }
}

/// 渲染管线优化配置
#[derive(Debug, Clone)]
pub struct RenderPipelineOptimization {
    pub enable_draw_call_batching: bool,
    pub enable_state_caching: bool,
    pub enable_deferred_rendering: bool,
    pub max_batch_size: u32,
    pub max_state_changes_per_frame: u32,
}

impl_default!(RenderPipelineOptimization {
    enable_draw_call_batching: true,
    enable_state_caching: true,
    enable_deferred_rendering: false,
    max_batch_size: 1000,
    max_state_changes_per_frame: 100,
});

impl RenderPipelineOptimization {
    /// 创建延迟渲染配置
    pub fn deferred_rendering() -> Self {
        Self {
            enable_draw_call_batching: true,
            enable_state_caching: true,
            enable_deferred_rendering: true,
            max_batch_size: 2000,
            max_state_changes_per_frame: 200,
        }
    }

    /// 创建高性能配置（最大化批处理）
    pub fn high_performance() -> Self {
        Self {
            enable_draw_call_batching: true,
            enable_state_caching: true,
            enable_deferred_rendering: false,
            max_batch_size: 5000,
            max_state_changes_per_frame: 50,
        }
    }

    /// 创建低延迟配置（最小化状态切换）
    pub fn low_latency() -> Self {
        Self {
            enable_draw_call_batching: true,
            enable_state_caching: true,
            enable_deferred_rendering: false,
            max_batch_size: 500,
            max_state_changes_per_frame: 20,
        }
    }
}

/// GPU 栅栏管理
pub struct GPUFence {
    pub id: u64,
    pub frame_number: u64,
    pub is_signaled: bool,
}

impl GPUFence {
    pub fn new(id: u64, frame_number: u64) -> Self {
        Self {
            id,
            frame_number,
            is_signaled: false,
        }
    }

    pub fn signal(&mut self) {
        self.is_signaled = true;
    }

    pub fn is_ready(&self) -> bool {
        self.is_signaled
    }
}

/// GPU 内存管理
pub struct GPUMemoryManager {
    allocated_vram: u64,
    max_vram: u64,
    allocation_count: usize,
}

impl GPUMemoryManager {
    pub fn new(max_vram: u64) -> Self {
        Self {
            allocated_vram: 0,
            max_vram,
            allocation_count: 0,
        }
    }

    pub fn allocate(&mut self, size: u64) -> Result<u64, String> {
        if self.allocated_vram + size > self.max_vram {
            return Err(format!(
                "Not enough VRAM: requested {}, available {}",
                size,
                self.max_vram - self.allocated_vram
            ));
        }

        self.allocated_vram += size;
        self.allocation_count += 1;
        Ok(self.allocation_count as u64)
    }

    pub fn deallocate(&mut self, size: u64) {
        self.allocated_vram = self.allocated_vram.saturating_sub(size);
    }

    pub fn get_usage_ratio(&self) -> f32 {
        (self.allocated_vram as f32) / (self.max_vram as f32)
    }

    pub fn get_available_memory(&self) -> u64 {
        self.max_vram.saturating_sub(self.allocated_vram)
    }
}

/// 渲染性能指标
#[derive(Debug, Clone)]
pub struct RenderMetrics {
    pub total_draw_calls: u32,
    pub batched_draw_calls: u32,
    pub gpu_time_ms: f32,
    pub cpu_time_ms: f32,
    pub vertex_count: u64,
    pub triangle_count: u64,
    pub state_changes: u32,
}

impl RenderMetrics {
    pub fn get_draw_call_reduction(&self) -> f32 {
        if self.total_draw_calls == 0 {
            0.0
        } else {
            1.0 - (self.batched_draw_calls as f32) / (self.total_draw_calls as f32)
        }
    }

    pub fn get_vertices_per_ms(&self) -> f64 {
        (self.vertex_count as f64) / (self.gpu_time_ms.max(0.001) as f64)
    }

    pub fn get_triangles_per_ms(&self) -> f64 {
        (self.triangle_count as f64) / (self.gpu_time_ms.max(0.001) as f64)
    }

    pub fn print_report(&self) {
        tracing::info!(target: "render", "\n=== Render Performance Metrics ===");
        tracing::info!(target: "render", "Draw calls: {} -> {} (reduction: {:.1}%)",
            self.total_draw_calls,
            self.batched_draw_calls,
            self.get_draw_call_reduction() * 100.0
        );
        tracing::info!(target: "render", "GPU time: {:.2}ms", self.gpu_time_ms);
        tracing::info!(target: "render", "CPU time: {:.2}ms", self.cpu_time_ms);
        tracing::info!(target: "render", "Vertices: {} ({:.2}M/ms)", self.vertex_count, self.get_vertices_per_ms() / 1_000_000.0);
        tracing::info!(target: "render", "Triangles: {} ({:.2}M/ms)", self.triangle_count, self.get_triangles_per_ms() / 1_000_000.0);
        tracing::info!(target: "render", "State changes: {}", self.state_changes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_call_optimizer() {
        let mut optimizer = DrawCallOptimizer::new();

        let state1 = RenderStateKey {
            pipeline_id: 1,
            bind_group_id: 1,
            blend_mode: 0,
            depth_test: true,
        };

        let state2 = RenderStateKey {
            pipeline_id: 2,
            bind_group_id: 2,
            blend_mode: 0,
            depth_test: true,
        };

        let cmd = DrawCommand {
            command_type: RenderCommandType::DrawIndexed,
            vertex_count: 100,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            index_count: 300,
            index_offset: 0,
        };

        // 添加 10 个相同状态的命令
        for _ in 0..10 {
            optimizer.add_command(cmd.clone(), state1);
        }

        // 添加 5 个不同状态的命令
        for _ in 0..5 {
            optimizer.add_command(cmd.clone(), state2);
        }

        // 应该有 2 个批次
        assert_eq!(optimizer.get_batch_count(), 2);
        // 应该有 2 次状态改变
        assert_eq!(optimizer.get_state_changes(), 2);
    }

    #[test]
    fn test_command_buffer() {
        let mut buffer = CommandBuffer::new();

        buffer.begin_recording();
        buffer.add_command(RenderCommand {
            name: "draw_mesh".to_string(),
            command_type: RenderCommandType::DrawIndexed,
            data: vec![0; 64],
        });

        buffer.add_command(RenderCommand {
            name: "draw_particles".to_string(),
            command_type: RenderCommandType::DrawInstanced,
            data: vec![0; 32],
        });

        buffer.end_recording();

        assert_eq!(buffer.get_command_count(), 2);
        assert!(buffer.estimate_size() >= 96);
    }

    #[test]
    fn test_gpu_memory_manager() {
        let mut manager = GPUMemoryManager::new(1024 * 1024); // 1MB

        let alloc1 = manager.allocate(512 * 1024);
        assert!(alloc1.is_ok());

        let alloc2 = manager.allocate(512 * 1024);
        assert!(alloc2.is_ok());

        // 应该没有足够的空间
        let alloc3 = manager.allocate(1);
        assert!(alloc3.is_err());

        manager.deallocate(512 * 1024);
        let alloc4 = manager.allocate(256 * 1024);
        assert!(alloc4.is_ok());
    }

    #[test]
    fn test_render_metrics() {
        let metrics = RenderMetrics {
            total_draw_calls: 1000,
            batched_draw_calls: 100,
            gpu_time_ms: 16.0,
            cpu_time_ms: 8.0,
            vertex_count: 1_000_000,
            triangle_count: 333_333,
            state_changes: 50,
        };

        assert_eq!(metrics.get_draw_call_reduction(), 0.9);
        tracing::debug!(target: "render", "{:?}", metrics);
        metrics.print_report();
    }
}
