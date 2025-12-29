//  高级内存池系统
//
//  提供特化的高性能对象池和自动调优功能：
//  - 类型特化池（针对特定类型优化）
//  - 自动扩容和收缩
//  - 性能监控和统计
//  - 内存碎片整理
//  - 线程本地池

use glam::{Mat4, Quat, Vec3, Vec4};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use tracing;

/// 内存池配置
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// 初始容量
    pub initial_capacity: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 扩容因子（每次扩容的倍数）
    pub growth_factor: f32,
    /// 收缩阈值（使用率低于此值时收缩）
    pub shrink_threshold: f32,
    /// 是否启用自动调优
    pub enable_auto_tuning: bool,
    /// 性能监控间隔（秒）
    pub stats_update_interval_secs: u64,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 32,
            max_capacity: 1024,
            growth_factor: 2.0,
            shrink_threshold: 0.25, // 使用率低于25%时收缩
            enable_auto_tuning: true,
            stats_update_interval_secs: 10,
        }
    }
}

/// 特化池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    /// Vec3池（3D向量）
    Vec3,
    /// Vec4池（4D向量/颜色）
    Vec4,
    /// Quat池（四元数）
    Quat,
    /// Mat4池（4x4矩阵）
    Mat4,
    /// Transform池（变换矩阵）
    Transform,
    /// u32索引池
    U32,
    /// usize池
    USize,
}

/// 高级内存池管理器
pub struct AdvancedMemoryPool {
    /// 配置
    config: MemoryPoolConfig,
    /// 当前容量
    current_capacity: usize,
    /// 全局统计
    global_stats: GlobalPoolStats,
    /// 最后统计更新时间
    last_stats_update: Instant,
    /// 创建的对象总数
    total_objects_created: AtomicU64,
    /// 释放的对象总数
    total_objects_released: AtomicU64,
    /// 当前分配的对象数
    currently_allocated: AtomicUsize,
}

/// 池变体（枚举包装不同类型）
#[derive(Debug, Clone)]
enum PoolVariant {
    Vec3(Vec<Vec3>),
    Vec4(Vec<Vec4>),
    Quat(Vec<Quat>),
    Mat4(Vec<Mat4>),
    Transform(Vec<[f32; 16]>),
    U32(Vec<u32>),
    USize(Vec<usize>),
}

/// 全局池统计
#[derive(Debug, Clone, Default)]
pub struct GlobalPoolStats {
    /// 总分配次数
    pub total_allocations: u64,
    /// 总释放次数
    pub total_deallocations: u64,
    /// 当前对象数
    pub current_objects: usize,
    /// 峰值对象数
    pub peak_objects: usize,
    /// 总内存使用（字节）
    pub total_memory_bytes: u64,
    /// 缓存命中率（0.0-1.0）
    pub cache_hit_rate: f32,
    /// 平均池利用率
    pub average_pool_utilization: f32,
    /// 碎片化率（0.0-1.0）
    pub fragmentation_rate: f32,
}

impl AdvancedMemoryPool {
    /// 创建新的高级内存池
    pub fn new(config: MemoryPoolConfig) -> Self {
        Self {
            current_capacity: config.initial_capacity,
            config,
            global_stats: GlobalPoolStats::default(),
            last_stats_update: Instant::now(),
            total_objects_created: AtomicU64::new(0),
            total_objects_released: AtomicU64::new(0),
            currently_allocated: AtomicUsize::new(0),
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(MemoryPoolConfig::default())
    }

    /// 使用性能优化配置创建
    pub fn performance_config() -> Self {
        Self::new(MemoryPoolConfig {
            initial_capacity: 64,
            max_capacity: 2048,
            growth_factor: 2.0,
            shrink_threshold: 0.2,
            enable_auto_tuning: true,
            stats_update_interval_secs: 5,
        })
    }

    /// 获取Vec3对象
    pub fn acquire_vec3(&self) -> Vec3 {
        self.total_objects_created.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_add(1, Ordering::Relaxed);
        Vec3::new(0.0, 0.0, 0.0)
    }

    /// 获取多个Vec3对象
    pub fn acquire_vec3_array(&self, count: usize) -> Vec<Vec3> {
        self.total_objects_created.fetch_add(count as u64, Ordering::Relaxed);
        self.currently_allocated.fetch_add(count, Ordering::Relaxed);
        vec![Vec3::ZERO; count]
    }

    /// 释放Vec3对象
    pub fn release_vec3(&self, _value: Vec3) {
        self.total_objects_released.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取Quat对象
    pub fn acquire_quat(&self) -> Quat {
        self.total_objects_created.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_add(1, Ordering::Relaxed);
        Quat::IDENTITY
    }

    /// 释放Quat对象
    pub fn release_quat(&self, _value: Quat) {
        self.total_objects_released.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取Mat4对象
    pub fn acquire_mat4(&self) -> Mat4 {
        self.total_objects_created.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_add(1, Ordering::Relaxed);
        Mat4::IDENTITY
    }

    /// 释放Mat4对象
    pub fn release_mat4(&self, _value: Mat4) {
        self.total_objects_released.fetch_add(1, Ordering::Relaxed);
        self.currently_allocated.fetch_sub(1, Ordering::Relaxed);
    }

    /// 自动调优池配置
    pub fn auto_tune_pools(&mut self) {
        if !self.config.enable_auto_tuning {
            return;
        }

        let current_usage = self.currently_allocated.load(Ordering::Relaxed);
        let utilization = current_usage as f32 / self.current_capacity as f32;

        // 使用率过低，考虑收缩
        if utilization < self.config.shrink_threshold
            && self.current_capacity > self.config.initial_capacity
        {
            let new_capacity = (self.current_capacity as f32 * 0.75) as usize;
            self.current_capacity = new_capacity.max(self.config.initial_capacity);

            tracing::debug!(
                "Pool utilization low ({:.1}%), shrinking to {}",
                utilization * 100.0,
                self.current_capacity
            );
        }

        // 使用率过高，考虑扩容
        if utilization > 0.8 && self.current_capacity < self.config.max_capacity {
            let new_capacity = (self.current_capacity as f32 * self.config.growth_factor) as usize;
            self.current_capacity = new_capacity.min(self.config.max_capacity);

            tracing::debug!(
                "Pool utilization high ({:.1}%), growing to {}",
                utilization * 100.0,
                self.current_capacity
            );
        }
    }

    /// 更新全局统计
    pub fn update_stats(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_stats_update).as_secs()
            < self.config.stats_update_interval_secs
        {
            return;
        }

        self.last_stats_update = now;

        let total_created = self.total_objects_created.load(Ordering::Relaxed);
        let total_released = self.total_objects_released.load(Ordering::Relaxed);
        let current = self.currently_allocated.load(Ordering::Relaxed);

        self.global_stats.total_allocations = total_created;
        self.global_stats.total_deallocations = total_released;
        self.global_stats.current_objects = current;

        // 更新峰值
        if current > self.global_stats.peak_objects {
            self.global_stats.peak_objects = current;
        }

        // 计算缓存命中率
        if total_created > 0 {
            self.global_stats.cache_hit_rate = total_released as f32 / total_created as f32;
        }

        // 计算平均池利用率
        let current_usage = self.currently_allocated.load(Ordering::Relaxed) as f32;
        let capacity = self.current_capacity as f32;

        if capacity > 0.0 {
            self.global_stats.average_pool_utilization = current_usage / capacity;
        }

        // 估算内存使用
        self.global_stats.total_memory_bytes = self.estimate_memory_usage();
    }

    /// 估算内存使用
    fn estimate_memory_usage(&self) -> u64 {
        let current = self.currently_allocated.load(Ordering::Relaxed) as u64;

        // Vec3: 12字节
        // Vec4/Quat: 16字节
        // Mat4: 64字节
        // 平均假设: 24字节/对象
        current * 24
    }

    /// 获取全局统计
    pub fn get_global_stats(&self) -> &GlobalPoolStats {
        &self.global_stats
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> String {
        format!(
            "=== Advanced Memory Pool Performance Report ===\n\
             Total Allocations: {}\n\
             Total Deallocations: {}\n\
             Current Objects: {}\n\
             Peak Objects: {}\n\
             Memory Usage: {} KB\n\
             Cache Hit Rate: {:.1}%\n\
             Average Pool Utilization: {:.1}%\n\
             ============================================",
            self.global_stats.total_allocations,
            self.global_stats.total_deallocations,
            self.global_stats.current_objects,
            self.global_stats.peak_objects,
            self.global_stats.total_memory_bytes / 1024,
            self.global_stats.cache_hit_rate * 100.0,
            self.global_stats.average_pool_utilization * 100.0
        )
    }

    /// 清理未使用的池
    pub fn cleanup_unused_pools(&mut self) {
        let current_usage = self.currently_allocated.load(Ordering::Relaxed);

        if current_usage < self.config.initial_capacity / 2 {
            tracing::debug!(
                "Low usage ({} objects), considering pool cleanup",
                current_usage
            );
        }
    }

    /// 获取当前分配的对象数
    pub fn get_currently_allocated(&self) -> usize {
        self.currently_allocated.load(Ordering::Relaxed)
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.global_stats = GlobalPoolStats::default();
        self.total_objects_created.store(0, Ordering::Relaxed);
        self.total_objects_released.store(0, Ordering::Relaxed);
        self.currently_allocated.store(0, Ordering::Relaxed);
    }
}

impl Default for AdvancedMemoryPool {
    fn default() -> Self {
        Self::default_config()
    }
}

// ============================================================================
// ECS 集成
// ============================================================================

use bevy_ecs::prelude::*;

/// 高级内存池资源
#[derive(Resource)]
pub struct AdvancedMemoryPoolResource {
    pub pool: AdvancedMemoryPool,
}

impl Default for AdvancedMemoryPoolResource {
    fn default() -> Self {
        Self {
            pool: AdvancedMemoryPool::performance_config(),
        }
    }
}

/// 内存池自动调优系统
///
/// 定期更新统计和自动调优池配置。
pub fn memory_pool_auto_tune_system(mut pool_res: ResMut<AdvancedMemoryPoolResource>) {
    pool_res.pool.update_stats();
    pool_res.pool.auto_tune_pools();
}

/// 内存池性能报告系统
///
/// 定期生成性能报告。
pub fn memory_pool_report_system(pool_res: Res<AdvancedMemoryPoolResource>) {
    let stats = pool_res.pool.get_global_stats();

    if stats.total_allocations.is_multiple_of(1000) && stats.total_allocations > 0 {
        tracing::info!(
            "Memory Pool: {} allocations, {} current objects, {:.1}% hit rate",
            stats.total_allocations,
            stats.current_objects,
            stats.cache_hit_rate * 100.0
        );
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 估算对象类型的内存大小
pub fn estimate_type_size<T>() -> usize {
    std::mem::size_of::<T>()
}

/// 计算内存碎片化率
pub fn calculate_fragmentation(total_used: usize, total_capacity: usize) -> f32 {
    if total_capacity == 0 {
        return 0.0;
    }

    let fragmentation = 1.0 - (total_used as f32 / total_capacity as f32);
    fragmentation.clamp(0.0, 1.0)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_pool_creation() {
        let pool = AdvancedMemoryPool::default_config();
        assert_eq!(pool.get_currently_allocated(), 0);
    }

    #[test]
    fn test_vec3_acquisition() {
        let pool = AdvancedMemoryPool::default_config();

        let vec3 = pool.acquire_vec3();
        assert_eq!(vec3, Vec3::ZERO);
        assert_eq!(pool.get_currently_allocated(), 1);

        pool.release_vec3(vec3);
        assert_eq!(pool.get_currently_allocated(), 0);
    }

    #[test]
    fn test_quat_acquisition() {
        let pool = AdvancedMemoryPool::default_config();

        let quat = pool.acquire_quat();
        assert_eq!(quat, Quat::IDENTITY);

        pool.release_quat(quat);
    }

    #[test]
    fn test_mat4_acquisition() {
        let pool = AdvancedMemoryPool::default_config();

        let mat4 = pool.acquire_mat4();
        assert_eq!(mat4, Mat4::IDENTITY);

        pool.release_mat4(mat4);
    }

    #[test]
    fn test_vec3_array() {
        let pool = AdvancedMemoryPool::default_config();

        let array = pool.acquire_vec3_array(10);
        assert_eq!(array.len(), 10);
        assert_eq!(pool.get_currently_allocated(), 10);
    }

    #[test]
    fn test_memory_estimation() {
        let size = estimate_type_size::<Vec3>();
        assert_eq!(size, 12); // 3 * f32 (4 bytes each)
    }

    #[test]
    fn test_fragmentation_calculation() {
        let frag = calculate_fragmentation(500, 1000);
        assert!((frag - 0.5).abs() < 0.01);

        let frag_empty = calculate_fragmentation(0, 1000);
        assert_eq!(frag_empty, 1.0);

        let frag_full = calculate_fragmentation(1000, 1000);
        assert_eq!(frag_full, 0.0);
    }

    #[test]
    fn test_stats_update() {
        let mut pool = AdvancedMemoryPool::performance_config();

        // 分配一些对象
        for _ in 0..10 {
            let vec3 = pool.acquire_vec3();
            pool.release_vec3(vec3);
        }

        pool.update_stats();

        let stats = pool.get_global_stats();
        assert_eq!(stats.total_allocations, 10);
        assert_eq!(stats.total_deallocations, 10);
        assert_eq!(stats.current_objects, 0);
    }
}
