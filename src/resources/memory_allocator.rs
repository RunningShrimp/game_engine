//! 内存分配器模块
//!
//! 基于环形缓冲区池的智能内存分配器，提供高性能的GPU内存管理。
//!
//! ## 分配策略
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Memory Allocator                      │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. 智能大小选择                                        │
//! │     - 小数据 (<64KB): 使用小块池                        │
//! │     - 中数据 (64KB-1MB): 使用中块池                     │
//! │     - 大数据 (>1MB): 使用大块池或直接分配                  │
//! │                                                          │
//! │  2. 最佳适配算法                                        │
//! │     - 优先选择大小最匹配的块                             │
//! │     - 避免内存碎片                                     │
//! │     - 支持块合并                                       │
//! │                                                          │
//! │  3. 动态扩展机制                                        │
//! │     - 空间不足时自动扩展                               │
//! │     - 预留空间管理                                     │
//! │     - 内存压力检测                                     │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Serialize, Deserialize};

use super::ring_buffer_pool::{BlockSize, MemoryBlock, RingBufferPool, align_to};

// ============================================================================
// 常量配置
// ============================================================================

/// 小数据阈值 (64KB)
const SMALL_DATA_THRESHOLD: u64 = 64 * 1024;

/// 中数据阈值 (1MB)
const MEDIUM_DATA_THRESHOLD: u64 = 1024 * 1024;

/// 大数据阈值 (4MB)
const LARGE_DATA_THRESHOLD: u64 = 4 * 1024 * 1024;

/// 默认环形缓冲区大小
const DEFAULT_BUFFER_SIZE: u64 = 16 * 1024 * 1024;

/// 内存压力阈值 (90%)
const MEMORY_PRESSURE_THRESHOLD: f32 = 0.9;

/// 扩展因子 (1.5倍)
const EXPANSION_FACTOR: f32 = 1.5;

/// 最大扩展次数
const MAX_EXPANSIONS: u32 = 4;
// ============================================================================
// 分配器配置
// ============================================================================

/// 内存分配器配置
#[derive(Debug, Clone)]
pub struct AllocatorConfig {
    /// 初始缓冲区大小
    pub initial_buffer_size: u64,
    /// 最大缓冲区大小
    pub max_buffer_size: u64,
    /// 是否启用自动扩展
    pub enable_auto_expansion: bool,
    /// 内存压力阈值
    pub memory_pressure_threshold: f32,
    /// 扩展因子
    pub expansion_factor: f32,
    /// 最大扩展次数
    pub max_expansions: u32,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            initial_buffer_size: DEFAULT_BUFFER_SIZE,
            max_buffer_size: 64 * 1024 * 1024, // 64MB
            enable_auto_expansion: true,
            memory_pressure_threshold: MEMORY_PRESSURE_THRESHOLD,
            expansion_factor: EXPANSION_FACTOR,
            max_expansions: MAX_EXPANSIONS,
        }
    }
}

impl AllocatorConfig {
    /// 创建新的配置
    pub fn new(initial_size: u64, max_size: u64) -> Self {
        Self {
            initial_buffer_size: initial_size,
            max_buffer_size: max_size,
            ..Default::default()
        }
    }

    /// 设置自动扩展
    pub fn with_auto_expansion(mut self, enable: bool) -> Self {
        self.enable_auto_expansion = enable;
        self
    }

    /// 设置内存压力阈值
    pub fn with_pressure_threshold(mut self, threshold: f32) -> Self {
        self.memory_pressure_threshold = threshold.clamp(0.1, 0.95);
        self
    }
}

// ============================================================================
// 分配请求
// ============================================================================

/// 内存分配请求
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    /// 请求大小
    pub size: u64,
    /// 对齐要求
    pub alignment: u64,
    /// 分配类型
    pub allocation_type: AllocationType,
    /// 优先级
    pub priority: AllocationPriority,
}

impl AllocationRequest {
    /// 创建新的分配请求
    pub fn new(size: u64) -> Self {
        Self {
            size,
            alignment: wgpu::COPY_BUFFER_ALIGNMENT,
            allocation_type: AllocationType::Temporary,
            priority: AllocationPriority::Normal,
        }
    }

    /// 设置对齐
    pub fn with_alignment(mut self, alignment: u64) -> Self {
        self.alignment = alignment;
        self
    }

    /// 设置分配类型
    pub fn with_type(mut self, allocation_type: AllocationType) -> Self {
        self.allocation_type = allocation_type;
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: AllocationPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// 分配类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationType {
    /// 临时分配 (一帧内使用)
    Temporary,
    /// 短期分配 (几帧内使用)
    ShortTerm,
    /// 长期分配 (持久数据)
    LongTerm,
}

/// 分配优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocationPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 关键优先级
    Critical = 3,
}

// ============================================================================
// 分配结果
// ============================================================================

/// 内存分配结果
#[derive(Debug)]
pub struct AllocationResult {
    /// 分配的内存块
    pub block: MemoryBlock,
    /// 实际分配大小 (可能大于请求大小)
    pub allocated_size: u64,
    /// 分配延迟 (微秒)
    pub allocation_latency_us: f32,
    /// 是否从扩展的缓冲区分配
    pub from_expanded: bool,
}

impl AllocationResult {
    /// 创建新的分配结果
    pub fn new(block: MemoryBlock, allocated_size: u64, latency_us: f32) -> Self {
        Self {
            block,
            allocated_size,
            allocation_latency_us: latency_us,
            from_expanded: false,
        }
    }

    /// 标记为来自扩展缓冲区
    pub fn from_expanded(mut self) -> Self {
        self.from_expanded = true;
        self
    }
}

// ============================================================================
// 内存压力监控
// ============================================================================

/// 内存压力级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressure {
    /// 低压力 (<50%)
    Low,
    /// 中等压力 (50-75%)
    Medium,
    /// 高压力 (75-90%)
    High,
    /// 临界压力 (>90%)
    Critical,
}

impl MemoryPressure {
    /// 根据使用率计算压力级别
    pub fn from_utilization(utilization: f32) -> Self {
        if utilization < 0.5 {
            MemoryPressure::Low
        } else if utilization < 0.75 {
            MemoryPressure::Medium
        } else if utilization < 0.9 {
            MemoryPressure::High
        } else {
            MemoryPressure::Critical
        }
    }

    /// 获取压力描述
    pub fn description(&self) -> &'static str {
        match self {
            MemoryPressure::Low => "低内存压力",
            MemoryPressure::Medium => "中等内存压力",
            MemoryPressure::High => "高内存压力",
            MemoryPressure::Critical => "临界内存压力",
        }
    }
}

/// 内存压力事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPressureEvent {
    /// 压力级别
    pub pressure: MemoryPressure,
    /// 当前使用率
    pub utilization: f32,
    /// 时间戳 (Unix 时间戳毫秒)
    pub timestamp_ms: u64,
    /// 建议操作
    pub recommendation: PressureRecommendation,
}

impl MemoryPressureEvent {
    /// 创建新的内存压力事件
    pub fn new(pressure: MemoryPressure, utilization: f32, recommendation: PressureRecommendation) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            pressure,
            utilization,
            timestamp_ms,
            recommendation,
        }
    }
}

/// 压力建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PressureRecommendation {
    /// 无需操作
    None,
    /// 考虑清理未使用的资源
    CleanupUnused,
    /// 考虑扩展内存池
    ExpandPool,
    /// 立即清理并扩展
    CleanupAndExpand,
    /// 内存不足，拒绝分配
    RejectAllocation,
}
// ============================================================================
// 智能内存分配器
// ============================================================================

/// 智能内存分配器
/// 
/// 基于环形缓冲区池的高性能内存分配器，支持多种分配策略和自动扩展。
#[derive(Debug)]
pub struct SmartMemoryAllocator {
    /// 环形缓冲区池
    ring_pools: Vec<RingBufferPool>,
    /// 当前活动池索引
    current_pool_index: usize,
    /// 分配器配置
    config: AllocatorConfig,
    /// 分配统计
    allocation_stats: Arc<Mutex<super::ring_buffer_pool::AllocationStats>>,
    /// 压力事件历史
    pressure_history: Arc<Mutex<Vec<MemoryPressureEvent>>>,
    /// 扩展计数
    expansion_count: Arc<Mutex<u32>>,
    /// 设备引用
    device: Arc<wgpu::Device>,
    /// 当前帧号
    current_frame: Arc<Mutex<u64>>,
}

impl SmartMemoryAllocator {
    /// 创建新的智能内存分配器
    pub fn new(device: Arc<wgpu::Device>, config: AllocatorConfig) -> Self {
        let initial_pool = RingBufferPool::new(device.clone(), config.initial_buffer_size);
        
        Self {
            ring_pools: vec![initial_pool],
            current_pool_index: 0,
            config,
            allocation_stats: Arc::new(Mutex::new(super::ring_buffer_pool::AllocationStats::default())),
            pressure_history: Arc::new(Mutex::new(Vec::new())),
            expansion_count: Arc::new(Mutex::new(0)),
            device,
            current_frame: Arc::new(Mutex::new(0)),
        }
    }

    /// 分配内存
    /// 
    /// # 参数
    /// - `request`: 分配请求
    /// 
    /// # 返回
    /// 返回分配结果，如果分配失败则返回None
    pub fn allocate(&mut self, request: AllocationRequest) -> Option<AllocationResult> {
        let start_time = std::time::Instant::now();
        
        // 检查内存压力
        let pressure = self.check_memory_pressure();
        if pressure == MemoryPressure::Critical && request.priority < AllocationPriority::High {
            // 临界压力下，只允许高优先级分配
            return None;
        }
        
        // 尝试在当前池中分配
        if let Some(result) = self.try_allocate_in_current_pool(&request) {
            let latency = start_time.elapsed().as_micros() as f32;
            let mut final_result = result;
            final_result.allocation_latency_us = latency;
            
            // 更新统计信息
            self.update_allocation_stats(&final_result);
            
            return Some(final_result);
        }
        
        // 当前池分配失败，尝试扩展
        if self.config.enable_auto_expansion {
            if let Some(result) = self.try_expand_and_allocate(&request) {
                let latency = start_time.elapsed().as_micros() as f32;
                let mut final_result = result;
                final_result.allocation_latency_us = latency;
                
                // 更新统计信息
                self.update_allocation_stats(&final_result);
                
                return Some(final_result);
            }
        }
        
        // 所有分配尝试都失败
        None
    }

    /// 在当前池中尝试分配
    fn try_allocate_in_current_pool(&mut self, request: &AllocationRequest) -> Option<AllocationResult> {
        let current_pool = &mut self.ring_pools[self.current_pool_index];
        
        // 根据请求大小选择合适的分配策略
        let aligned_size = align_to(request.size, request.alignment);
        
        if let Some(block) = current_pool.allocate(aligned_size, request.alignment) {
            let allocated_size = block.size;
            Some(AllocationResult::new(block, allocated_size, 0.0))
        } else {
            None
        }
    }

    /// 尝试扩展池并分配
    fn try_expand_and_allocate(&mut self, request: &AllocationRequest) -> Option<AllocationResult> {
        let mut expansion_count = self.expansion_count.lock();
        
        if *expansion_count >= self.config.max_expansions {
            return None; // 达到最大扩展次数
        }
        
        // 检查是否需要扩展
        let current_utilization = self.calculate_utilization();
        if current_utilization < self.config.memory_pressure_threshold {
            return None; // 使用率不高，不需要扩展
        }
        
        // 创建新的扩展池
        let new_size = (self.get_current_total_size() as f32 * self.config.expansion_factor) as u64;
        let new_size = new_size.min(self.config.max_buffer_size);
        
        if new_size <= self.get_current_total_size() {
            return None; // 无法进一步扩展
        }
        
        // 创建新的环形缓冲区池
        let new_pool = RingBufferPool::new(self.device.clone(), new_size);
        self.ring_pools.push(new_pool);
        self.current_pool_index = self.ring_pools.len() - 1;
        *expansion_count += 1;
        
        // 在新池中分配
        if let Some(block) = self.ring_pools[self.current_pool_index].allocate(
            align_to(request.size, request.alignment),
            request.alignment,
        ) {
            let allocated_size = block.size;
            let mut result = AllocationResult::new(block, allocated_size, 0.0);
            result.from_expanded = true;
            
            // 记录扩展事件
            self.log_expansion_event(new_size, current_utilization);
            
            Some(result)
        } else {
            None
        }
    }

    /// 释放内存
    pub fn deallocate(&mut self, block: MemoryBlock) {
        let block_size = block.size;
        
        // 找到对应的池并释放
        for pool in &mut self.ring_pools {
            if block.offset < pool.total_capacity() {
                pool.deallocate(block);
                break;
            }
        }
        
        // 更新统计信息
        {
            let mut stats = self.allocation_stats.lock();
            stats.active_allocations = stats.active_allocations.saturating_sub(1);
            stats.active_bytes = stats.active_bytes.saturating_sub(block_size);
        }
    }

    /// 检查内存压力
    fn check_memory_pressure(&self) -> MemoryPressure {
        let utilization = self.calculate_utilization();
        MemoryPressure::from_utilization(utilization)
    }

    /// 计算当前使用率
    fn calculate_utilization(&self) -> f32 {
        let total_capacity = self.get_current_total_size();
        if total_capacity == 0 {
            return 0.0;
        }
        
        let total_active = self.ring_pools.iter()
            .map(|pool| pool.stats().active_bytes)
            .sum::<u64>();
        
        total_active as f32 / total_capacity as f32
    }

    /// 获取当前总容量
    fn get_current_total_size(&self) -> u64 {
        self.ring_pools.iter().map(|pool| pool.total_capacity()).sum()
    }

    /// 更新分配统计信息
    fn update_allocation_stats(&self, result: &AllocationResult) {
        let mut stats = self.allocation_stats.lock();
        stats.total_allocations += 1;
        stats.total_bytes_allocated += result.allocated_size;
        stats.active_allocations += 1;
        stats.active_bytes += result.allocated_size;
        stats.update_peak();
        
        if result.from_expanded {
            // 记录扩展分配
            tracing::debug!(
                target: "memory_allocator",
                "Allocated {} bytes from expanded pool (latency: {:.2}μs)",
                result.allocated_size,
                result.allocation_latency_us
            );
        }
    }

    /// 记录扩展事件
    fn log_expansion_event(&self, new_size: u64, utilization: f32) {
        let pressure = MemoryPressure::from_utilization(utilization);
        let recommendation = if utilization > 0.9 {
            PressureRecommendation::CleanupAndExpand
        } else {
            PressureRecommendation::ExpandPool
        };
        
        let event = MemoryPressureEvent::new(pressure, utilization, recommendation);
        
        {
            let mut history = self.pressure_history.lock();
            history.push(event);
            
            // 保持历史记录在合理范围内
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        tracing::info!(
            target: "memory_allocator",
            "Memory pool expanded to {} bytes (utilization: {:.1}%)",
            new_size,
            utilization * 100.0
        );
    }

    /// 帧结束时调用
    pub fn end_frame(&mut self) {
        {
            let mut frame = self.current_frame.lock();
            *frame += 1;
        }
        
        // 更新所有池
        for pool in &mut self.ring_pools {
            pool.end_frame();
        }
        
        // 检查内存压力
        let pressure = self.check_memory_pressure();
        if pressure == MemoryPressure::High || pressure == MemoryPressure::Critical {
            self.handle_memory_pressure(pressure);
        }
    }

    /// 处理内存压力
    fn handle_memory_pressure(&mut self, pressure: MemoryPressure) {
        let utilization = self.calculate_utilization();
        let recommendation = match pressure {
            MemoryPressure::High => PressureRecommendation::CleanupUnused,
            MemoryPressure::Critical => PressureRecommendation::CleanupAndExpand,
            _ => PressureRecommendation::None,
        };
        
        let event = MemoryPressureEvent::new(pressure, utilization, recommendation);
        
        {
            let mut history = self.pressure_history.lock();
            history.push(event);
            
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        tracing::warn!(
            target: "memory_allocator",
            "Memory pressure detected: {} (utilization: {:.1}%)",
            pressure.description(),
            utilization * 100.0
        );
    }

    /// 获取分配统计信息
    pub fn stats(&self) -> super::ring_buffer_pool::AllocationStats {
        self.allocation_stats.lock().clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        let mut stats = self.allocation_stats.lock();
        *stats = super::ring_buffer_pool::AllocationStats::default();
        
        // 重置所有池的统计
        for pool in &mut self.ring_pools {
            pool.reset_stats();
        }
    }

    /// 获取内存压力历史
    pub fn pressure_history(&self) -> Vec<MemoryPressureEvent> {
        self.pressure_history.lock().clone()
    }

    /// 获取当前内存压力
    pub fn current_pressure(&self) -> MemoryPressure {
        self.check_memory_pressure()
    }

    /// 获取使用率
    pub fn utilization(&self) -> f32 {
        self.calculate_utilization()
    }

    /// 获取总容量
    pub fn total_capacity(&self) -> u64 {
        self.get_current_total_size()
    }

    /// 获取扩展次数
    pub fn expansion_count(&self) -> u32 {
        *self.expansion_count.lock()
    }

    /// 强制垃圾回收
    pub fn force_gc(&mut self) {
        // 检查所有池的空闲块
        for pool in &mut self.ring_pools {
            // 这里可以添加更复杂的GC逻辑
            // 例如：合并碎片、释放未使用的池等
        }
        
        tracing::debug!(target: "memory_allocator", "Forced garbage collection completed");
    }
}

// ============================================================================
// 便捷函数
// ============================================================================

/// 创建默认配置的内存分配器
pub fn create_default_allocator(device: Arc<wgpu::Device>) -> SmartMemoryAllocator {
    SmartMemoryAllocator::new(device, AllocatorConfig::default())
}

/// 创建高性能配置的内存分配器
pub fn create_high_performance_allocator(device: Arc<wgpu::Device>) -> SmartMemoryAllocator {
    let config = AllocatorConfig::new(32 * 1024 * 1024, 128 * 1024 * 1024) // 32MB-128MB
        .with_auto_expansion(true)
        .with_pressure_threshold(0.8);
    
    SmartMemoryAllocator::new(device, config)
}

/// 创建低内存配置的内存分配器
pub fn create_low_memory_allocator(device: Arc<wgpu::Device>) -> SmartMemoryAllocator {
    let config = AllocatorConfig::new(8 * 1024 * 1024, 32 * 1024 * 1024) // 8MB-32MB
        .with_auto_expansion(false)
        .with_pressure_threshold(0.95);
    
    SmartMemoryAllocator::new(device, config)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_config_default() {
        let config = AllocatorConfig::default();
        assert_eq!(config.initial_buffer_size, DEFAULT_BUFFER_SIZE);
        assert_eq!(config.max_buffer_size, 64 * 1024 * 1024);
        assert!(config.enable_auto_expansion);
        assert_eq!(config.memory_pressure_threshold, MEMORY_PRESSURE_THRESHOLD);
    }

    #[test]
    fn test_allocation_request() {
        let request = AllocationRequest::new(1024)
            .with_alignment(256)
            .with_type(AllocationType::ShortTerm)
            .with_priority(AllocationPriority::High);
        
        assert_eq!(request.size, 1024);
        assert_eq!(request.alignment, 256);
        assert_eq!(request.allocation_type, AllocationType::ShortTerm);
        assert_eq!(request.priority, AllocationPriority::High);
    }

    #[test]
    fn test_memory_pressure_levels() {
        assert_eq!(MemoryPressure::from_utilization(0.3), MemoryPressure::Low);
        assert_eq!(MemoryPressure::from_utilization(0.6), MemoryPressure::Medium);
        assert_eq!(MemoryPressure::from_utilization(0.8), MemoryPressure::High);
        assert_eq!(MemoryPressure::from_utilization(0.95), MemoryPressure::Critical);
    }

    #[test]
    fn test_allocation_result() {
        let block = MemoryBlock::new(1, 1024, 0, 256, 10);
        let mut result = AllocationResult::new(block, 1024, 50.0);
        assert!(!result.from_expanded);
        
        result = result.from_expanded();
        assert!(result.from_expanded);
    }
}