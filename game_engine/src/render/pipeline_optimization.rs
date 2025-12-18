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

/// 绘制调用优化器
#[derive(Default)]
pub struct DrawCallOptimizer {
    state_cache: HashMap<RenderStateKey, u32>,
    command_batches: Vec<Vec<DrawCommand>>,
    current_state: Option<RenderStateKey>,
    state_changes: u32,
}

impl DrawCallOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加绘制命令到批次
    pub fn add_command(&mut self, command: DrawCommand, state: RenderStateKey) {
        // 检查状态是否改变
        if self.current_state != Some(state) {
            if !self.command_batches.is_empty() && self.command_batches.last().unwrap().is_empty() {
                // 新建批次
            } else if self.command_batches.is_empty() {
                self.command_batches.push(Vec::new());
            } else {
                self.command_batches.push(Vec::new());
            }
            self.current_state = Some(state);
            self.state_changes += 1;
        }

        if let Some(batch) = self.command_batches.last_mut() {
            batch.push(command);
        } else {
            self.command_batches.push(vec![command]);
        }

        self.state_cache.insert(state, self.state_changes);
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
    pub fn get_optimization_ratio(&self, original_count: u32) -> f32 {
        if original_count == 0 {
            0.0
        } else {
            1.0 - (self.command_batches.len() as f32) / (original_count as f32)
        }
    }

    pub fn clear(&mut self) {
        self.state_cache.clear();
        self.command_batches.clear();
        self.current_state = None;
        self.state_changes = 0;
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
