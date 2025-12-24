//! 预分配管理器模块
//!
//! 管理Staging Buffer的预分配和回收机制，优化内存使用效率。
//!
//! ## 预分配策略
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              Preallocation Manager                   │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. 启动时预分配                                        │
//! │     - 根据历史使用模式预分配                            │
//! │     - 分级预分配 (小/中/大块)                         │
//! │     - 预留缓冲区                                      │
//! │                                                          │
//! │  2. 动态预分配                                        │
//! │     - 根据当前使用率调整                                │
//! │     - 预测性预分配                                    │
//! │     - 智能扩容                                        │
//! │                                                          │
//! │  3. 回收机制                                            │
//! │     - 基于Fence的安全回收                              │
//! │     - 延迟回收策略                                    │
//! │     - 内存碎片整理                                    │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use super::memory_allocator::AllocationRequest;
use super::ring_buffer_pool::{BlockSize, MemoryBlock, RingBufferPool, align_to};

// ============================================================================
// 常量配置
// ============================================================================

/// 默认预分配大小 (8MB)
const DEFAULT_PREALLOCATION_SIZE: u64 = 8 * 1024 * 1024;

/// 小块预分配数量
const SMALL_BLOCK_PREALLOC_COUNT: usize = 32;

/// 中块预分配数量
const MEDIUM_BLOCK_PREALLOC_COUNT: usize = 8;

/// 大块预分配数量
const LARGE_BLOCK_PREALLOC_COUNT: usize = 2;

/// 预分配池大小倍数
const PREALLOCATION_MULTIPLIER: f32 = 1.5;

/// 回收延迟帧数
const RECLAIM_DELAY_FRAMES: u64 = 3;

/// 预分配历史记录长度
const PREALLOCATION_HISTORY_LENGTH: usize = 60;

// ============================================================================
// 预分配配置
// ============================================================================

/// 预分配配置
#[derive(Debug, Clone)]
pub struct PreallocationConfig {
    /// 启用预分配
    pub enable_preallocation: bool,
    /// 初始预分配大小
    pub initial_preallocation_size: u64,
    /// 小块预分配数量
    pub small_block_count: usize,
    /// 中块预分配数量
    pub medium_block_count: usize,
    /// 大块预分配数量
    pub large_block_count: usize,
    /// 启用动态预分配
    pub enable_dynamic_preallocation: bool,
    /// 预分配阈值 (使用率)
    pub preallocation_threshold: f32,
    /// 回收延迟帧数
    pub reclaim_delay_frames: u64,
}

impl Default for PreallocationConfig {
    fn default() -> Self {
        Self {
            enable_preallocation: true,
            initial_preallocation_size: DEFAULT_PREALLOCATION_SIZE,
            small_block_count: SMALL_BLOCK_PREALLOC_COUNT,
            medium_block_count: MEDIUM_BLOCK_PREALLOC_COUNT,
            large_block_count: LARGE_BLOCK_PREALLOC_COUNT,
            enable_dynamic_preallocation: true,
            preallocation_threshold: 0.8,
            reclaim_delay_frames: RECLAIM_DELAY_FRAMES,
        }
    }
}

impl PreallocationConfig {
    /// 创建新的配置
    pub fn new(initial_size: u64) -> Self {
        Self {
            initial_preallocation_size: initial_size,
            ..Default::default()
        }
    }

    /// 设置预分配数量
    pub fn with_block_counts(mut self, small: usize, medium: usize, large: usize) -> Self {
        self.small_block_count = small;
        self.medium_block_count = medium;
        self.large_block_count = large;
        self
    }

    /// 设置动态预分配
    pub fn with_dynamic_preallocation(mut self, enable: bool) -> Self {
        self.enable_dynamic_preallocation = enable;
        self
    }
}

// ============================================================================
// 使用模式分析
// ============================================================================

/// 分配模式统计
#[derive(Debug, Clone, Default)]
pub struct AllocationPattern {
    /// 小块分配次数
    pub small_allocations: u64,
    /// 中块分配次数
    pub medium_allocations: u64,
    /// 大块分配次数
    pub large_allocations: u64,
    /// 总分配字节数
    pub total_bytes: u64,
    /// 峰值使用量
    pub peak_usage: u64,
    /// 平均分配大小
    pub average_size: f32,
    /// 分配频率 (每帧)
    pub allocation_frequency: f32,
}

impl AllocationPattern {
    /// 记录分配
    pub fn record_allocation(&mut self, size: u64) {
        self.total_bytes += size;

        let block_size = BlockSize::for_request(size);
        match block_size {
            BlockSize::Small => self.small_allocations += 1,
            BlockSize::Medium => self.medium_allocations += 1,
            BlockSize::Large => self.large_allocations += 1,
            BlockSize::Custom(_) => self.large_allocations += 1,
        }

        self.update_statistics();
    }

    /// 更新统计信息
    fn update_statistics(&mut self) {
        let total_allocs =
            self.small_allocations + self.medium_allocations + self.large_allocations;
        if total_allocs > 0 {
            self.average_size = self.total_bytes as f32 / total_allocs as f32;
        }
    }

    /// 预测下一帧需求
    pub fn predict_next_frame需求(&self) -> HashMap<BlockSize, u32> {
        let mut prediction = HashMap::new();

        // 简单的线性预测：基于历史平均值
        if self.small_allocations > 0 {
            prediction.insert(
                BlockSize::Small,
                (self.small_allocations / PREALLOCATION_HISTORY_LENGTH as u64) as u32,
            );
        }
        if self.medium_allocations > 0 {
            prediction.insert(
                BlockSize::Medium,
                (self.medium_allocations / PREALLOCATION_HISTORY_LENGTH as u64) as u32,
            );
        }
        if self.large_allocations > 0 {
            prediction.insert(
                BlockSize::Large,
                (self.large_allocations / PREALLOCATION_HISTORY_LENGTH as u64) as u32,
            );
        }

        prediction
    }
}

// ============================================================================
// 预分配块
// ============================================================================

/// 预分配块
#[derive(Debug)]
pub struct PreallocatedBlock {
    /// 内存块
    pub block: MemoryBlock,
    /// 块大小类型
    pub block_size: BlockSize,
    /// 创建时间
    pub created_at: Instant,
    /// 最后使用帧
    pub last_used: Option<u64>,
    /// 使用次数
    pub use_count: u64,
    /// 是否空闲
    pub is_free: bool,
}

impl PreallocatedBlock {
    /// 创建新的预分配块
    pub fn new(block: MemoryBlock, block_size: BlockSize) -> Self {
        Self {
            block,
            block_size,
            created_at: Instant::now(),
            last_used: None,
            use_count: 0,
            is_free: true,
        }
    }

    /// 标记为使用
    pub fn mark_used(&mut self, current_frame: u64) {
        self.is_free = false;
        self.last_used = Some(current_frame);
        self.use_count += 1;
    }

    /// 标记为空闲
    pub fn mark_free(&mut self) {
        self.is_free = true;
    }

    /// 检查是否可以回收
    pub fn can_reclaim(&self, current_frame: u64, reclaim_delay: u64) -> bool {
        if !self.is_free {
            return false;
        }

        // 检查是否超过回收延迟
        if let Some(last_used) = self.last_used {
            let frames_since_use = (current_frame - self.block.created_frame).max(1);
            let idle_frames = current_frame.saturating_sub(last_used);
            // 考虑实际使用时间和创建时间，取较大值
            let effective_idle_frames = frames_since_use.max(idle_frames);
            effective_idle_frames >= reclaim_delay
        } else {
            // 从未使用过的块可以立即回收
            true
        }
    }

    /// 获取使用效率
    pub fn efficiency(&self) -> f32 {
        if self.created_at.elapsed().as_secs() == 0 {
            0.0
        } else {
            self.use_count as f32 / self.created_at.elapsed().as_secs() as f32
        }
    }
}

// ============================================================================
// 预分配管理器
// ============================================================================

/// 预分配管理器
#[derive(Debug)]
pub struct PreallocationManager {
    /// 预分配配置
    config: PreallocationConfig,
    /// 环形缓冲区池
    ring_pool: RingBufferPool,
    /// 预分配块池 (按大小分级)
    preallocated_pools: HashMap<BlockSize, VecDeque<PreallocatedBlock>>,
    /// 分配模式历史
    allocation_history: VecDeque<AllocationPattern>,
    /// 当前帧模式
    current_pattern: AllocationPattern,
    /// 当前帧号
    current_frame: u64,
    /// 预分配统计
    stats: Arc<Mutex<PreallocationStats>>,
    /// 设备引用
    device: Arc<wgpu::Device>,
}

/// 预分配统计信息
#[derive(Debug, Clone, Default)]
pub struct PreallocationStats {
    /// 预分配总次数
    pub total_preallocations: u64,
    /// 预分配总字节数
    pub total_preallocated_bytes: u64,
    /// 预分配命中次数
    pub preallocation_hits: u64,
    /// 预分配未命中次数
    pub preallocation_misses: u64,
    /// 回收次数
    pub reclamation_count: u64,
    /// 当前预分配块数
    pub current_preallocated_blocks: u64,
    /// 预分配命中率
    pub hit_rate: f32,
    /// 内存节省率
    pub memory_savings_rate: f32,
}

impl PreallocationManager {
    /// 创建新的预分配管理器
    pub fn new(device: Arc<wgpu::Device>, config: PreallocationConfig) -> Self {
        let ring_pool = RingBufferPool::new(device.clone(), config.initial_preallocation_size);

        let mut manager = Self {
            config,
            ring_pool,
            preallocated_pools: HashMap::new(),
            allocation_history: VecDeque::with_capacity(PREALLOCATION_HISTORY_LENGTH),
            current_pattern: AllocationPattern::default(),
            current_frame: 0,
            stats: Arc::new(Mutex::new(PreallocationStats::default())),
            device,
        };

        // 初始化预分配池
        manager.initialize_preallocated_pools();

        manager
    }

    /// 初始化预分配池
    fn initialize_preallocated_pools(&mut self) {
        if !self.config.enable_preallocation {
            return;
        }

        // 创建不同大小的预分配池
        self.preallocated_pools.insert(BlockSize::Small, VecDeque::new());
        self.preallocated_pools.insert(BlockSize::Medium, VecDeque::new());
        self.preallocated_pools.insert(BlockSize::Large, VecDeque::new());

        // 预分配块
        self.perform_initial_preallocation();
    }

    /// 执行初始预分配
    fn perform_initial_preallocation(&mut self) {
        // 预分配小块
        for _ in 0..self.config.small_block_count {
            if let Some(block) = self.preallocate_block(BlockSize::Small) {
                self.add_to_pool(BlockSize::Small, block);
            }
        }

        // 预分配中块
        for _ in 0..self.config.medium_block_count {
            if let Some(block) = self.preallocate_block(BlockSize::Medium) {
                self.add_to_pool(BlockSize::Medium, block);
            }
        }

        // 预分配大块
        for _ in 0..self.config.large_block_count {
            if let Some(block) = self.preallocate_block(BlockSize::Large) {
                self.add_to_pool(BlockSize::Large, block);
            }
        }

        tracing::info!(
            target: "preallocation_manager",
            "Initial preallocation completed: {} small, {} medium, {} large blocks",
            self.config.small_block_count,
            self.config.medium_block_count,
            self.config.large_block_count
        );
    }

    /// 预分配单个块
    fn preallocate_block(&mut self, block_size: BlockSize) -> Option<PreallocatedBlock> {
        let size = block_size.size();
        let alignment = 256; // 常用对齐

        if let Some(block) = self.ring_pool.allocate(size, alignment) {
            let preallocated_block = PreallocatedBlock::new(block, block_size);

            // 更新统计信息
            {
                let mut stats = self.stats.lock();
                stats.total_preallocations += 1;
                stats.total_preallocated_bytes += size;
                stats.current_preallocated_blocks += 1;
            }

            Some(preallocated_block)
        } else {
            None
        }
    }

    /// 添加块到预分配池
    fn add_to_pool(&mut self, block_size: BlockSize, block: PreallocatedBlock) {
        if let Some(pool) = self.preallocated_pools.get_mut(&block_size) {
            pool.push_back(block);
        }
    }

    /// 分配预分配块
    pub fn allocate(
        &mut self,
        request: AllocationRequest,
        current_frame: u64,
    ) -> Option<MemoryBlock> {
        let block_size = BlockSize::for_request(request.size);

        // 记录分配模式
        self.current_pattern.record_allocation(request.size);

        // 尝试从预分配池中获取
        if let Some(mut preallocated_block) = self.try_get_from_pool(block_size) {
            preallocated_block.mark_used(current_frame);

            // 更新统计信息
            {
                let mut stats = self.stats.lock();
                stats.preallocation_hits += 1;
                stats.update_hit_rate();
            }

            return Some(preallocated_block.block);
        }

        // 预分配池中没有合适的块，尝试直接分配
        if let Some(block) = self
            .ring_pool
            .allocate(align_to(request.size, request.alignment), request.alignment)
        {
            // 更新统计信息
            {
                let mut stats = self.stats.lock();
                stats.preallocation_misses += 1;
                stats.update_hit_rate();
            }

            Some(block)
        } else {
            // 环形缓冲池分配失败，尝试扩展池
            if self.expand_preallocation_pool(block_size) {
                // 扩展成功后重试分配
                if let Some(block) = self
                    .ring_pool
                    .allocate(align_to(request.size, request.alignment), request.alignment)
                {
                    // 更新统计信息
                    {
                        let mut stats = self.stats.lock();
                        stats.preallocation_misses += 1;
                        stats.update_hit_rate();
                    }

                    Some(block)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// 从预分配池中获取块
    fn try_get_from_pool(&mut self, block_size: BlockSize) -> Option<PreallocatedBlock> {
        if let Some(pool) = self.preallocated_pools.get_mut(&block_size) {
            // 寻找大小合适的块
            for i in 0..pool.len() {
                if let Some(preallocated_block) = pool.get_mut(i) {
                    if preallocated_block.is_free
                        && preallocated_block.block.size >= block_size.size()
                    {
                        return Some(pool.remove(i).unwrap());
                    }
                }
            }
        }

        None
    }

    /// 释放块回预分配池
    pub fn deallocate(&mut self, block: MemoryBlock) {
        let block_size = BlockSize::for_request(block.size);

        // 尝试放回预分配池
        if self.can_return_to_pool(&block, block_size) {
            let mut preallocated_block = PreallocatedBlock::new(block, block_size);
            preallocated_block.mark_free();

            self.add_to_pool(block_size, preallocated_block);
        } else {
            // 直接释放到环形池
            self.ring_pool.deallocate(block);
        }
    }

    /// 检查是否可以返回预分配池
    fn can_return_to_pool(&self, block: &MemoryBlock, block_size: BlockSize) -> bool {
        // 检查块大小是否匹配预分配大小
        match block_size {
            BlockSize::Small => block.size <= BlockSize::Small.size() * 2,
            BlockSize::Medium => block.size <= BlockSize::Medium.size() * 2,
            BlockSize::Large => block.size <= BlockSize::Large.size() * 2,
            BlockSize::Custom(_) => false, // 自定义大小不返回预分配池
        }
    }

    /// 执行动态预分配
    fn perform_dynamic_preallocation(&mut self) {
        if !self.config.enable_dynamic_preallocation {
            return;
        }

        // 基于使用模式预测需求
        let prediction = self.current_pattern.predict_next_frame需求();

        for (block_size, predicted_count) in prediction {
            let current_count = self.get_pool_count(block_size);
            let target_count = (predicted_count as f32 * PREALLOCATION_MULTIPLIER) as usize;

            if current_count < target_count {
                // 需要预分配更多块
                let additional_count = target_count - current_count;
                for _ in 0..additional_count {
                    if let Some(block) = self.preallocate_block(block_size) {
                        self.add_to_pool(block_size, block);
                    } else {
                        break; // 空间不足
                    }
                }
            }
        }
    }

    /// 获取指定大小的池中块数量
    fn get_pool_count(&self, block_size: BlockSize) -> usize {
        self.preallocated_pools.get(&block_size).map(|pool| pool.len()).unwrap_or(0)
    }

    /// 回收未使用的预分配块
    fn reclaim_unused_blocks(&mut self) {
        let current_frame = self.current_frame;
        let reclaim_delay = self.config.reclaim_delay_frames;

        for (block_size, pool) in &mut self.preallocated_pools {
            let mut to_reclaim = Vec::new();

            // 标记可以回收的块
            for (i, block) in pool.iter().enumerate() {
                if block.can_reclaim(current_frame, reclaim_delay) {
                    to_reclaim.push(i);
                }
            }

            // 记录回收统计信息
            if !to_reclaim.is_empty() {
                tracing::debug!(target: "memory_allocator", "Reclaiming {} blocks of size {}", to_reclaim.len(), block_size);
            }

            // 回收块 (从后往前删除，避免索引问题)
            for &i in to_reclaim.iter().rev() {
                if let Some(block) = pool.remove(i) {
                    // 记录块的使用效率
                    let efficiency = block.efficiency();
                    if efficiency < 0.1 {
                        tracing::debug!(target: "memory_allocator", "Low efficiency block reclaimed: {:.2}%", efficiency * 100.0);
                    }

                    // 释放到环形池
                    self.ring_pool.deallocate(block.block);

                    // 更新统计信息
                    {
                        let mut stats = self.stats.lock();
                        stats.reclamation_count += 1;
                        stats.current_preallocated_blocks =
                            stats.current_preallocated_blocks.saturating_sub(1);
                    }
                }
            }
        }

        if !self.preallocated_pools.is_empty() {
            tracing::debug!(target: "preallocation_manager", "Reclaimed unused preallocated blocks");
        }
    }

    /// 整理预分配池
    fn defragment_pools(&mut self) {
        for (block_size, pool) in &mut self.preallocated_pools {
            // 按使用效率排序，优先保留高效率的块
            pool.make_contiguous().sort_by(|a, b| {
                b.efficiency().partial_cmp(&a.efficiency()).unwrap_or(std::cmp::Ordering::Equal)
            });

            // 移除效率过低的块
            let initial_len = pool.len();
            pool.retain(|block| {
                block.efficiency() > 0.1 || // 使用效率大于10%
                block.use_count > 0 || // 至少使用过一次
                block.created_at.elapsed() < Duration::from_secs(60) // 创建时间小于1分钟
            });

            let removed_count = initial_len - pool.len();
            if removed_count > 0 {
                tracing::debug!(
                    target: "preallocation_manager",
                    "Removed {} low-efficiency {:?} blocks",
                    removed_count,
                    block_size
                );
            }
        }
    }

    /// 帧结束时调用
    pub fn end_frame(&mut self) {
        self.current_frame += 1;

        // 更新环形池
        self.ring_pool.end_frame();

        // 保存当前分配模式到历史
        self.allocation_history.push_back(self.current_pattern.clone());
        if self.allocation_history.len() > PREALLOCATION_HISTORY_LENGTH {
            self.allocation_history.pop_front();
        }

        // 重置当前模式
        self.current_pattern = AllocationPattern::default();

        // 执行动态预分配
        self.perform_dynamic_preallocation();

        // 回收未使用的块
        self.reclaim_unused_blocks();

        // 定期整理 (每60帧一次)
        if self.current_frame % 60 == 0 {
            self.defragment_pools();
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> PreallocationStats {
        self.stats.lock().clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        let mut stats = self.stats.lock();
        *stats = PreallocationStats::default();
    }

    /// 获取当前预分配块数量
    pub fn current_preallocated_count(&self) -> usize {
        self.preallocated_pools.values().map(|pool| pool.len()).sum()
    }

    /// 获取使用率
    pub fn utilization(&self) -> f32 {
        self.ring_pool.utilization()
    }

    /// 强制回收所有预分配块
    pub fn force_reclaim_all(&mut self) {
        for (block_size, pool) in &mut self.preallocated_pools {
            let count = pool.len();
            for block in pool.drain(..) {
                // 记录块的使用统计
                let efficiency = block.efficiency();
                let use_count = block.use_count;

                // 释放到环形池
                self.ring_pool.deallocate(block.block);

                // 记录低效块
                if efficiency < 0.1 && use_count > 0 {
                    tracing::debug!(
                        target: "preallocation_manager",
                        "Force reclaimed low efficiency block: {:.2}% efficiency, {} uses",
                        efficiency * 100.0,
                        use_count
                    );
                }
            }

            if count > 0 {
                tracing::debug!(
                    target: "preallocation_manager",
                    "Force reclaimed {} {:?} blocks",
                    count,
                    block_size
                );
            }
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock();
            stats.current_preallocated_blocks = 0;
        }
    }

    /// 获取设备引用（用于调试和扩展）
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// 扩展预分配池（当现有池容量不足时）
    fn expand_preallocation_pool(&mut self, block_size: BlockSize) -> bool {
        if !self.config.enable_dynamic_preallocation {
            return false;
        }

        // 计算扩展大小
        let expand_size = match block_size {
            BlockSize::Small => 1024 * 1024,      // 1MB
            BlockSize::Medium => 4 * 1024 * 1024, // 4MB
            BlockSize::Large => 16 * 1024 * 1024, // 16MB
            BlockSize::Custom(size) => size * 2,  // 自定义大小的两倍
        };

        // 创建新的更大的环形缓冲池
        let new_pool = RingBufferPool::new(self.device.clone(), expand_size);

        // 替换现有的环形缓冲池
        self.ring_pool = new_pool;

        // 记录扩展操作
        tracing::debug!(
            target: "preallocation_manager",
            "Expanded preallocation pool for {:?} blocks by {} bytes",
            block_size,
            expand_size
        );

        true
    }
}

impl PreallocationStats {
    /// 更新命中率
    pub fn update_hit_rate(&mut self) {
        let total_attempts = self.preallocation_hits + self.preallocation_misses;
        if total_attempts > 0 {
            self.hit_rate = self.preallocation_hits as f32 / total_attempts as f32;
        }
    }

    /// 更新内存节省率
    pub fn update_memory_savings_rate(&mut self, total_allocated: u64, preallocated_used: u64) {
        if total_allocated > 0 {
            self.memory_savings_rate = preallocated_used as f32 / total_allocated as f32;
        }
    }
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 创建默认配置的预分配管理器
pub fn create_default_preallocation_manager(device: Arc<wgpu::Device>) -> PreallocationManager {
    PreallocationManager::new(device, PreallocationConfig::default())
}

/// 创建高性能配置的预分配管理器
pub fn create_high_performance_preallocation_manager(
    device: Arc<wgpu::Device>,
) -> PreallocationManager {
    let config = PreallocationConfig::new(16 * 1024 * 1024) // 16MB
        .with_block_counts(64, 16, 4) // 更多预分配块
        .with_dynamic_preallocation(true);

    PreallocationManager::new(device, config)
}

/// 创建低内存配置的预分配管理器
pub fn create_low_memory_preallocation_manager(device: Arc<wgpu::Device>) -> PreallocationManager {
    let config = PreallocationConfig::new(4 * 1024 * 1024) // 4MB
        .with_block_counts(16, 4, 1) // 较少预分配块
        .with_dynamic_preallocation(false);

    PreallocationManager::new(device, config)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preallocation_config_default() {
        let config = PreallocationConfig::default();
        assert!(config.enable_preallocation);
        assert_eq!(
            config.initial_preallocation_size,
            DEFAULT_PREALLOCATION_SIZE
        );
        assert_eq!(config.small_block_count, SMALL_BLOCK_PREALLOC_COUNT);
        assert_eq!(config.medium_block_count, MEDIUM_BLOCK_PREALLOC_COUNT);
        assert_eq!(config.large_block_count, LARGE_BLOCK_PREALLOC_COUNT);
    }

    #[test]
    fn test_allocation_pattern() {
        let mut pattern = AllocationPattern::default();

        pattern.record_allocation(32 * 1024); // 小块
        pattern.record_allocation(512 * 1024); // 中块
        pattern.record_allocation(2 * 1024 * 1024); // 大块

        assert_eq!(pattern.small_allocations, 1);
        assert_eq!(pattern.medium_allocations, 1);
        assert_eq!(pattern.large_allocations, 1);
        assert!(pattern.total_bytes > 0);
        assert!(pattern.average_size > 0.0);
    }

    #[test]
    fn test_preallocated_block() {
        let block = MemoryBlock::new(1, 1024, 0, 256, 10);
        let mut preallocated_block = PreallocatedBlock::new(block, BlockSize::Small);

        assert!(preallocated_block.is_free);
        assert_eq!(preallocated_block.use_count, 0);
        assert!(preallocated_block.last_used.is_none());

        preallocated_block.mark_used(0);
        assert!(!preallocated_block.is_free);
        assert_eq!(preallocated_block.use_count, 1);
        assert!(preallocated_block.last_used.is_some());

        preallocated_block.mark_free();
        assert!(preallocated_block.is_free);
    }

    #[test]
    fn test_preallocation_stats() {
        let mut stats = PreallocationStats::default();

        stats.preallocation_hits = 80;
        stats.preallocation_misses = 20;
        stats.update_hit_rate();

        assert_eq!(stats.hit_rate, 0.8);

        stats.update_memory_savings_rate(1000, 800);
        assert_eq!(stats.memory_savings_rate, 0.8);
    }
}
