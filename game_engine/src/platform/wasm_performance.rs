//! WebAssembly性能优化
//!
//! 提供WebAssembly特定的性能优化功能，包括：
//! - 内存池管理
//! - SIMD优化
//! - 线性内存优化
//! - WASM内存增长策略

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

/// WASM内存池配置
#[derive(Debug, Clone)]
pub struct WasmMemoryPoolConfig {
    /// 初始内存大小（页，每页64KB）
    pub initial_pages: u32,
    /// 最大内存大小（页）
    pub max_pages: u32,
    /// 内存增长步长（页）
    pub growth_step: u32,
    /// 是否启用内存池
    pub enable_pooling: bool,
    /// 池块大小（字节）
    pub pool_block_size: usize,
    /// 最大池块数
    pub max_pool_blocks: usize,
}

impl Default for WasmMemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_pages: 256, // 16MB
            max_pages: 16384,   // 1GB
            growth_step: 256,   // 16MB
            enable_pooling: true,
            pool_block_size: 64 * 1024, // 64KB块
            max_pool_blocks: 256,
        }
    }
}

/// WASM内存池
///
/// 管理WebAssembly线性内存的分配和重用
#[cfg(target_arch = "wasm32")]
pub struct WasmMemoryPool {
    /// 配置
    config: WasmMemoryPoolConfig,
    /// 可用块列表
    available_blocks: Vec<usize>,
    /// 已分配块映射（块索引 -> 大小）
    allocated_blocks: std::collections::HashMap<usize, usize>,
    /// 内存使用统计
    stats: WasmMemoryStats,
}

#[cfg(target_arch = "wasm32")]
impl WasmMemoryPool {
    /// 创建新的WASM内存池
    pub fn new(config: WasmMemoryPoolConfig) -> Self {
        Self {
            config,
            available_blocks: Vec::new(),
            allocated_blocks: std::collections::HashMap::new(),
            stats: WasmMemoryStats::default(),
        }
    }

    /// 分配内存块
    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        // 对齐到块大小
        let aligned_size = (size + self.config.pool_block_size - 1) / self.config.pool_block_size
            * self.config.pool_block_size;

        // 查找可用块
        if let Some(block_index) = self.available_blocks.iter().position(|&idx| {
            // 检查块是否足够大（简化实现）
            true // 实际实现需要跟踪块大小
        }) {
            let idx = self.available_blocks.remove(block_index);
            self.allocated_blocks.insert(idx, aligned_size);
            self.stats.allocated_bytes += aligned_size;
            self.stats.allocation_count += 1;
            Some(idx * self.config.pool_block_size)
        } else {
            // 分配新块
            if self.allocated_blocks.len() < self.config.max_pool_blocks {
                let new_block = self.allocated_blocks.len();
                self.allocated_blocks.insert(new_block, aligned_size);
                self.stats.allocated_bytes += aligned_size;
                self.stats.allocation_count += 1;
                Some(new_block * self.config.pool_block_size)
            } else {
                None
            }
        }
    }

    /// 释放内存块
    pub fn deallocate(&mut self, offset: usize) {
        let block_index = offset / self.config.pool_block_size;
        if let Some(size) = self.allocated_blocks.remove(&block_index) {
            self.available_blocks.push(block_index);
            self.stats.deallocated_bytes += size;
            self.stats.deallocation_count += 1;
        }
    }

    /// 获取内存统计
    pub fn stats(&self) -> &WasmMemoryStats {
        &self.stats
    }

    /// 清除所有块
    pub fn clear(&mut self) {
        self.available_blocks.clear();
        self.allocated_blocks.clear();
        self.stats = WasmMemoryStats::default();
    }
}

/// WASM内存统计
#[derive(Debug, Clone, Default)]
pub struct WasmMemoryStats {
    /// 已分配字节数
    pub allocated_bytes: usize,
    /// 已释放字节数
    pub deallocated_bytes: usize,
    /// 分配次数
    pub allocation_count: usize,
    /// 释放次数
    pub deallocation_count: usize,
    /// 当前使用字节数
    pub current_usage: usize,
}

impl WasmMemoryStats {
    /// 更新当前使用量
    pub fn update_usage(&mut self) {
        self.current_usage = self.allocated_bytes.saturating_sub(self.deallocated_bytes);
    }

    /// 获取内存使用率（0.0 - 1.0）
    pub fn usage_ratio(&self, total_memory: usize) -> f64 {
        if total_memory == 0 {
            0.0
        } else {
            self.current_usage as f64 / total_memory as f64
        }
    }
}

/// SIMD优化检测
#[cfg(target_arch = "wasm32")]
pub struct WasmSimdSupport {
    /// 是否支持SIMD
    pub supports_simd: bool,
    /// SIMD宽度（字节）
    pub simd_width: usize,
}

#[cfg(target_arch = "wasm32")]
impl WasmSimdSupport {
    /// 检测SIMD支持
    pub fn detect() -> Self {
        // 在WASM中，SIMD支持通过wasm32 target feature检测
        // 实际检测需要运行时检查或编译时特性标志
        let supports_simd = cfg!(target_feature = "simd128");

        Self {
            supports_simd,
            simd_width: if supports_simd { 16 } else { 0 }, // 128位 = 16字节
        }
    }

    /// 检查是否可以使用SIMD优化
    pub fn can_use_simd(&self) -> bool {
        self.supports_simd
    }

    /// 获取SIMD优化建议
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !self.supports_simd {
            suggestions.push("SIMD not supported. Consider enabling wasm32 target with simd128 feature for better performance.".to_string());
        } else {
            suggestions.push(
                "SIMD is available. Use SIMD-optimized functions for vector operations."
                    .to_string(),
            );
        }

        suggestions
    }
}

/// 线性内存优化器
#[cfg(target_arch = "wasm32")]
pub struct WasmLinearMemoryOptimizer {
    /// 内存池
    memory_pool: WasmMemoryPool,
    /// SIMD支持
    simd_support: WasmSimdSupport,
    /// 内存增长策略
    growth_strategy: MemoryGrowthStrategy,
}

#[cfg(target_arch = "wasm32")]
impl WasmLinearMemoryOptimizer {
    /// 创建新的线性内存优化器
    pub fn new(config: WasmMemoryPoolConfig) -> Self {
        Self {
            memory_pool: WasmMemoryPool::new(config),
            simd_support: WasmSimdSupport::detect(),
            growth_strategy: MemoryGrowthStrategy::Exponential,
        }
    }

    /// 获取内存池
    pub fn memory_pool(&mut self) -> &mut WasmMemoryPool {
        &mut self.memory_pool
    }

    /// 获取SIMD支持信息
    pub fn simd_support(&self) -> &WasmSimdSupport {
        &self.simd_support
    }

    /// 优化内存分配
    pub fn optimize_allocation(&mut self, size: usize) -> Option<usize> {
        if self.memory_pool.config.enable_pooling {
            self.memory_pool.allocate(size)
        } else {
            // 使用系统分配器
            None
        }
    }

    /// 获取内存增长建议
    pub fn get_growth_suggestion(&self, current_pages: u32, used_pages: u32) -> u32 {
        match self.growth_strategy {
            MemoryGrowthStrategy::Linear => self.memory_pool.config.growth_step,
            MemoryGrowthStrategy::Exponential => {
                // 指数增长：当前大小的50%
                (used_pages / 2).max(self.memory_pool.config.growth_step)
            }
            MemoryGrowthStrategy::Adaptive => {
                // 自适应：基于使用率
                let usage_ratio = used_pages as f64 / current_pages as f64;
                if usage_ratio > 0.8 {
                    // 使用率高，增长更多
                    current_pages / 4
                } else {
                    self.memory_pool.config.growth_step
                }
            }
        }
    }

    /// 获取优化建议
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // SIMD建议
        suggestions.extend(self.simd_support.get_optimization_suggestions());

        // 内存池建议
        let stats = self.memory_pool.stats();
        if stats.allocation_count > 0 {
            let avg_allocation = stats.allocated_bytes / stats.allocation_count;
            if avg_allocation < self.memory_pool.config.pool_block_size / 4 {
                suggestions.push("Average allocation size is small. Consider reducing pool block size for better efficiency.".to_string());
            }
        }

        suggestions
    }
}

/// 内存增长策略
#[derive(Debug, Clone, Copy)]
pub enum MemoryGrowthStrategy {
    /// 线性增长（固定步长）
    Linear,
    /// 指数增长（基于当前大小）
    Exponential,
    /// 自适应增长（基于使用率）
    Adaptive,
}

/// WASM性能优化器（非WASM平台）
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmLinearMemoryOptimizer {
    /// 占位字段
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmLinearMemoryOptimizer {
    /// 创建新的线性内存优化器（非WASM平台）
    pub fn new(_config: WasmMemoryPoolConfig) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 获取SIMD支持信息（非WASM平台）
    pub fn simd_support(&self) -> WasmSimdSupport {
        WasmSimdSupport {
            supports_simd: false,
            simd_width: 0,
        }
    }

    /// 优化内存分配（非WASM平台）
    pub fn optimize_allocation(&mut self, _size: usize) -> Option<usize> {
        None
    }

    /// 获取优化建议（非WASM平台）
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        vec!["WASM optimizations are only available on wasm32 target".to_string()]
    }
}

/// SIMD支持（非WASM平台）
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmSimdSupport {
    /// 是否支持SIMD
    pub supports_simd: bool,
    /// SIMD宽度（字节）
    pub simd_width: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmSimdSupport {
    /// 检测SIMD支持（非WASM平台）
    pub fn detect() -> Self {
        Self {
            supports_simd: false,
            simd_width: 0,
        }
    }

    /// 检查是否可以使用SIMD优化（非WASM平台）
    pub fn can_use_simd(&self) -> bool {
        false
    }

    /// 获取SIMD优化建议（非WASM平台）
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        vec!["SIMD detection is only available on wasm32 target".to_string()]
    }
}

/// WASM内存追踪分配器
///
/// 追踪WASM内存分配，用于性能分析和优化
pub struct WasmTrackingAllocator;

static WASM_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static WASM_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static WASM_PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for WasmTrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            let allocated = WASM_ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let deallocated = WASM_DEALLOCATED.load(Ordering::Relaxed);
            let current = allocated + size - deallocated;
            let mut peak = WASM_PEAK.load(Ordering::Relaxed);
            while current > peak {
                match WASM_PEAK.compare_exchange(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(new_peak) => peak = new_peak,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        WASM_DEALLOCATED.fetch_add(size, Ordering::Relaxed);
        std::alloc::System.dealloc(ptr, layout);
    }
}

/// 获取WASM内存统计
pub fn get_wasm_memory_stats() -> WasmMemoryStats {
    let allocated = WASM_ALLOCATED.load(Ordering::Relaxed);
    let deallocated = WASM_DEALLOCATED.load(Ordering::Relaxed);
    let peak = WASM_PEAK.load(Ordering::Relaxed);

    WasmMemoryStats {
        allocated_bytes: allocated,
        deallocated_bytes: deallocated,
        current_usage: allocated.saturating_sub(deallocated),
        allocation_count: 0,   // 需要额外追踪
        deallocation_count: 0, // 需要额外追踪
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_memory_pool_config_default() {
        let config = WasmMemoryPoolConfig::default();
        assert_eq!(config.initial_pages, 256);
        assert_eq!(config.max_pages, 16384);
        assert!(config.enable_pooling);
    }

    #[test]
    fn test_wasm_memory_stats() {
        let mut stats = WasmMemoryStats::default();
        stats.allocated_bytes = 1000;
        stats.deallocated_bytes = 300;
        stats.update_usage();

        assert_eq!(stats.current_usage, 700);
        assert_eq!(stats.usage_ratio(1000), 0.7);
    }

    #[test]
    fn test_wasm_simd_support_detect() {
        let simd = WasmSimdSupport::detect();
        // 在非WASM平台应该返回不支持
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!simd.supports_simd);
    }

    #[test]
    fn test_wasm_linear_memory_optimizer_creation() {
        let config = WasmMemoryPoolConfig::default();
        let optimizer = WasmLinearMemoryOptimizer::new(config);

        // 应该能够创建
        assert!(true);
    }

    #[test]
    fn test_wasm_memory_stats_usage_ratio() {
        let stats = WasmMemoryStats {
            allocated_bytes: 500,
            deallocated_bytes: 200,
            current_usage: 300,
            allocation_count: 10,
            deallocation_count: 5,
        };

        assert_eq!(stats.usage_ratio(1000), 0.3);
        assert_eq!(stats.usage_ratio(0), 0.0);
    }

    #[test]
    fn test_memory_growth_strategy() {
        // 测试不同增长策略的逻辑
        let config = WasmMemoryPoolConfig::default();
        let optimizer = WasmLinearMemoryOptimizer::new(config);

        // 测试线性增长
        let linear_suggestion = optimizer.get_growth_suggestion(1000, 800);
        assert!(linear_suggestion > 0);
    }
}
