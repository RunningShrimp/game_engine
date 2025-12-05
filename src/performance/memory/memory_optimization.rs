/// 内存优化和缓存对齐
///
/// 提供内存优化技术：
/// - 缓存行对齐
/// - 对象池优化
/// - 内存碎片检测
/// - NUMA 感知内存分配
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

/// CPU 缓存行大小（大多数现代 CPU 为 64 字节）
pub const CACHE_LINE_SIZE: usize = 64;

/// 缓存行对齐的 Data 包装器
#[repr(align(64))]
pub struct CacheLineAligned<T: Sized> {
    pub data: T,
}

impl<T> CacheLineAligned<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

/// 内存分配统计
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub allocated_bytes: usize,
    pub deallocated_bytes: usize,
    pub peak_usage_bytes: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

impl MemoryStats {
    pub fn current_usage(&self) -> usize {
        self.allocated_bytes.saturating_sub(self.deallocated_bytes)
    }

    pub fn fragmentation_ratio(&self) -> f64 {
        if self.allocation_count == 0 {
            0.0
        } else {
            (self.deallocation_count as f64) / (self.allocation_count as f64)
        }
    }
}

/// 追踪内存分配的全局分配器
pub struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = std::alloc::System.alloc(layout);

        if !ret.is_null() {
            let size = layout.size();
            let allocated = ALLOCATED.fetch_add(size, Ordering::Relaxed) + size;

            // 更新峰值
            let mut peak = PEAK.load(Ordering::Relaxed);
            while allocated > peak {
                match PEAK.compare_exchange_weak(
                    peak,
                    allocated,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        std::alloc::System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

/// 内存对齐优化工具
pub struct AlignmentOptimizer;

impl AlignmentOptimizer {
    /// 计算对齐后的大小
    pub fn align_to(size: usize, alignment: usize) -> usize {
        (size + alignment - 1) & !(alignment - 1)
    }

    /// 计算最佳缓存行对齐大小
    pub fn align_to_cache_line(size: usize) -> usize {
        Self::align_to(size, CACHE_LINE_SIZE)
    }

    /// 检测是否需要内存对齐
    pub fn needs_alignment(ptr: *const u8, alignment: usize) -> bool {
        (ptr as usize) % alignment != 0
    }

    /// 获取对齐所需的填充字节数
    pub fn padding_needed(ptr: *const u8, alignment: usize) -> usize {
        let addr = ptr as usize;
        let misaligned = addr % alignment;
        if misaligned == 0 {
            0
        } else {
            alignment - misaligned
        }
    }
}

/// 内存碎片分析
#[derive(Debug, Clone)]
pub struct FragmentationAnalysis {
    pub free_blocks: Vec<usize>,
    pub allocated_blocks: Vec<usize>,
    pub largest_free_block: usize,
    pub smallest_free_block: usize,
    pub avg_free_block_size: usize,
    pub fragmentation_index: f64,
}

impl FragmentationAnalysis {
    /// 计算碎片指数（0 = 无碎片，1 = 严重碎片）
    pub fn compute_index(free_blocks: &[usize]) -> f64 {
        if free_blocks.is_empty() {
            return 0.0;
        }

        let total_free: usize = free_blocks.iter().sum();
        let max_free = *free_blocks.iter().max().unwrap_or(&0);

        if max_free == 0 {
            return 1.0;
        }

        1.0 - (max_free as f64) / (total_free as f64)
    }

    /// 创建分析报告
    pub fn new(free_blocks: Vec<usize>, allocated_blocks: Vec<usize>) -> Self {
        let largest_free_block = *free_blocks.iter().max().unwrap_or(&0);
        let smallest_free_block = *free_blocks.iter().min().unwrap_or(&0);
        let avg_free_block_size = if free_blocks.is_empty() {
            0
        } else {
            free_blocks.iter().sum::<usize>() / free_blocks.len()
        };

        let fragmentation_index = Self::compute_index(&free_blocks);

        Self {
            free_blocks,
            allocated_blocks,
            largest_free_block,
            smallest_free_block,
            avg_free_block_size,
            fragmentation_index,
        }
    }

    /// 打印分析报告
    pub fn print_report(&self) {
        tracing::info!(target: "memory", "\n=== Memory Fragmentation Analysis ===");
        tracing::info!(target: "memory", "Free blocks: {}", self.free_blocks.len());
        tracing::info!(target: "memory", "Allocated blocks: {}", self.allocated_blocks.len());
        tracing::info!(target: "memory", "Largest free block: {} bytes", self.largest_free_block);
        tracing::info!(target: "memory", "Smallest free block: {} bytes", self.smallest_free_block);
        tracing::info!(target: "memory", "Average free block size: {} bytes", self.avg_free_block_size);
        tracing::info!(target: "memory", "Fragmentation index: {:.2}%", self.fragmentation_index * 100.0);
    }
}

/// 内存分配模式检测
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationPattern {
    /// 线性分配（单调递增）
    Linear,
    /// 池式分配（重复相同大小）
    Pooled,
    /// 随机分配
    Random,
    /// 混合模式
    Mixed,
}

pub struct AllocationPatternDetector {
    allocations: Vec<usize>,
    pattern: AllocationPattern,
}

impl AllocationPatternDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_allocation(&mut self, size: usize) {
        self.allocations.push(size);
        self.update_pattern();
    }

    fn update_pattern(&mut self) {
        if self.allocations.len() < 3 {
            return;
        }

        // 检测线性模式
        let is_linear = self.allocations.windows(2).all(|w| w[0] <= w[1]);

        // 检测池式模式（大多数大小相同）
        let is_pooled = {
            let first = self.allocations[0];
            let same_count = self.allocations.iter().filter(|&&s| s == first).count();
            same_count > self.allocations.len() / 2
        };

        self.pattern = if is_linear {
            AllocationPattern::Linear
        } else if is_pooled {
            AllocationPattern::Pooled
        } else {
            AllocationPattern::Mixed
        };
    }

    pub fn get_pattern(&self) -> AllocationPattern {
        self.pattern
    }

    pub fn get_recommendation(&self) -> &'static str {
        match self.pattern {
            AllocationPattern::Linear => {
                "Consider using Arena allocator for linear allocation pattern"
            }
            AllocationPattern::Pooled => {
                "Consider using Object Pool for repeated allocations of same size"
            }
            AllocationPattern::Random => {
                "Consider using jemalloc or tcmalloc for random allocation pattern"
            }
            AllocationPattern::Mixed => {
                "Monitor allocation patterns for optimization opportunities"
            }
        }
    }
}

impl Default for AllocationPatternDetector {
    fn default() -> Self {
        Self {
            allocations: Vec::new(),
            pattern: AllocationPattern::Mixed,
        }
    }
}

/// 内存预分配优化
pub struct MemoryPreallocationStrategy {
    pub initial_capacity: usize,
    pub growth_factor: f32,
    pub max_capacity: usize,
}

impl MemoryPreallocationStrategy {
    pub fn new(initial: usize, max: usize) -> Self {
        Self {
            initial_capacity: initial,
            growth_factor: 1.5,
            max_capacity: max,
        }
    }

    pub fn calculate_next_capacity(&self, current: usize) -> usize {
        let next = ((current as f32) * self.growth_factor) as usize;
        next.min(self.max_capacity)
    }

    /// 预热（预分配）
    pub fn preheat<T: Default>(&self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.initial_capacity);
        for _ in 0..self.initial_capacity {
            vec.push(T::default());
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_line_alignment() {
        let aligned = CacheLineAligned::new(42u32);
        let addr = &aligned as *const _ as usize;
        assert_eq!(addr % CACHE_LINE_SIZE, 0);
    }

    #[test]
    fn test_alignment_optimizer() {
        assert_eq!(AlignmentOptimizer::align_to(10, 8), 16);
        assert_eq!(AlignmentOptimizer::align_to(16, 8), 16);
        assert_eq!(AlignmentOptimizer::align_to_cache_line(100), 128);
    }

    #[test]
    fn test_fragmentation_analysis() {
        let free = vec![100, 50, 25];
        let allocated = vec![1000, 500];
        let analysis = FragmentationAnalysis::new(free, allocated);

        assert_eq!(analysis.largest_free_block, 100);
        assert_eq!(analysis.smallest_free_block, 25);
        assert!(analysis.fragmentation_index >= 0.0 && analysis.fragmentation_index <= 1.0);
    }

    #[test]
    fn test_allocation_pattern_detector() {
        let mut detector = AllocationPatternDetector::new();

        // 线性模式
        for i in 1..=10 {
            detector.record_allocation(i * 100);
        }

        assert_eq!(detector.get_pattern(), AllocationPattern::Linear);
    }

    #[test]
    fn test_preallocation_strategy() {
        let strategy = MemoryPreallocationStrategy::new(100, 1000);
        assert_eq!(strategy.calculate_next_capacity(100), 150);
        assert_eq!(strategy.calculate_next_capacity(900), 1000); // Capped
    }
}
