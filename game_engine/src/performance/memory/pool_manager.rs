//! 对象池管理器
//!
//! 提供预定义的常用对象池，支持高频分配的对象类型。
//!
//! ## 设计目标
//!
//! 1. **预定义池**：为常见对象类型提供预配置的对象池
//! 2. **自动管理**：自动调整池大小，优化内存使用
//! 3. **性能监控**：跟踪池的使用情况和性能指标
//! 4. **线程安全**：所有池都是线程安全的

use game_engine_performance::memory::object_pool::{PoolStats, SyncObjectPool};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use tracing;

/// 对象池管理器
///
/// 管理多个预定义的对象池，提供统一的访问接口
pub struct PoolManager {
    /// Vec<u8> 对象池（用于临时缓冲区）
    vec_u8_pool: Arc<SyncObjectPool<Vec<u8>>>,
    /// Vec<f32> 对象池（用于浮点数组）
    vec_f32_pool: Arc<SyncObjectPool<Vec<f32>>>,
    /// Vec<Vec3> 对象池（用于3D向量数组）
    vec_vec3_pool: Arc<SyncObjectPool<Vec<Vec3>>>,
    /// Vec<Mat4> 对象池（用于矩阵数组）
    vec_mat4_pool: Arc<SyncObjectPool<Vec<Mat4>>>,
    /// String 对象池（用于临时字符串）
    string_pool: Arc<SyncObjectPool<String>>,
    /// HashMap<String, String> 对象池（用于临时映射）
    hashmap_string_pool: Arc<SyncObjectPool<HashMap<String, String>>>,
    /// Vec<u32> 对象池（用于索引数组）
    vec_u32_pool: Arc<SyncObjectPool<Vec<u32>>>,
    /// Vec<Vec3> 对象池（扩展，用于位置数组等）
    vec_vec3_pool_extended: Arc<SyncObjectPool<Vec<Vec3>>>,
}

impl PoolManager {
    /// 创建新的对象池管理器
    pub fn new() -> Self {
        Self {
            vec_u8_pool: Arc::new(SyncObjectPool::new(
                || Vec::<u8>::new(),
                32,  // 初始大小
                256, // 最大大小
            )),
            vec_f32_pool: Arc::new(SyncObjectPool::new(|| Vec::<f32>::new(), 32, 256)),
            vec_vec3_pool: Arc::new(SyncObjectPool::new(|| Vec::<Vec3>::new(), 16, 128)),
            vec_mat4_pool: Arc::new(SyncObjectPool::new(|| Vec::<Mat4>::new(), 16, 128)),
            string_pool: Arc::new(SyncObjectPool::new(|| String::new(), 32, 256)),
            hashmap_string_pool: Arc::new(SyncObjectPool::new(
                || HashMap::<String, String>::new(),
                16,
                128,
            )),
            vec_u32_pool: Arc::new(SyncObjectPool::new(|| Vec::<u32>::new(), 32, 256)),
            vec_vec3_pool_extended: Arc::new(SyncObjectPool::new(|| Vec::<Vec3>::new(), 32, 256)),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            vec_u8_pool: Arc::new(SyncObjectPool::new(
                || Vec::<u8>::new(),
                config.vec_u8_initial,
                config.vec_u8_max,
            )),
            vec_f32_pool: Arc::new(SyncObjectPool::new(
                || Vec::<f32>::new(),
                config.vec_f32_initial,
                config.vec_f32_max,
            )),
            vec_vec3_pool: Arc::new(SyncObjectPool::new(
                || Vec::<Vec3>::new(),
                config.vec_vec3_initial,
                config.vec_vec3_max,
            )),
            vec_mat4_pool: Arc::new(SyncObjectPool::new(
                || Vec::<Mat4>::new(),
                config.vec_mat4_initial,
                config.vec_mat4_max,
            )),
            string_pool: Arc::new(SyncObjectPool::new(
                || String::new(),
                config.string_initial,
                config.string_max,
            )),
            hashmap_string_pool: Arc::new(SyncObjectPool::new(
                || HashMap::<String, String>::new(),
                config.hashmap_initial,
                config.hashmap_max,
            )),
            vec_u32_pool: Arc::new(SyncObjectPool::new(
                || Vec::<u32>::new(),
                config.vec_u32_initial,
                config.vec_u32_max,
            )),
            vec_vec3_pool_extended: Arc::new(SyncObjectPool::new(
                || Vec::<Vec3>::new(),
                config.vec_vec3_extended_initial,
                config.vec_vec3_extended_max,
            )),
        }
    }

    /// 获取 Vec<u8> 对象池
    pub fn vec_u8_pool(&self) -> Arc<SyncObjectPool<Vec<u8>>> {
        self.vec_u8_pool.clone()
    }

    /// 获取 Vec<f32> 对象池
    pub fn vec_f32_pool(&self) -> Arc<SyncObjectPool<Vec<f32>>> {
        self.vec_f32_pool.clone()
    }

    /// 获取 Vec<Vec3> 对象池
    pub fn vec_vec3_pool(&self) -> Arc<SyncObjectPool<Vec<Vec3>>> {
        self.vec_vec3_pool.clone()
    }

    /// 获取 Vec<Mat4> 对象池
    pub fn vec_mat4_pool(&self) -> Arc<SyncObjectPool<Vec<Mat4>>> {
        self.vec_mat4_pool.clone()
    }

    /// 获取 String 对象池
    pub fn string_pool(&self) -> Arc<SyncObjectPool<String>> {
        self.string_pool.clone()
    }

    /// 获取 HashMap<String, String> 对象池
    pub fn hashmap_string_pool(&self) -> Arc<SyncObjectPool<HashMap<String, String>>> {
        self.hashmap_string_pool.clone()
    }

    /// 获取 Vec<u32> 对象池
    pub fn vec_u32_pool(&self) -> Arc<SyncObjectPool<Vec<u32>>> {
        self.vec_u32_pool.clone()
    }

    /// 获取扩展的 Vec<Vec3> 对象池（用于位置数组等）
    pub fn vec_vec3_pool_extended(&self) -> Arc<SyncObjectPool<Vec<Vec3>>> {
        self.vec_vec3_pool_extended.clone()
    }

    /// 获取所有池的统计信息
    pub fn stats(&self) -> PoolManagerStats {
        PoolManagerStats {
            vec_u8: self.vec_u8_pool.stats(),
            vec_f32: self.vec_f32_pool.stats(),
            vec_vec3: self.vec_vec3_pool.stats(),
            vec_mat4: self.vec_mat4_pool.stats(),
            string: self.string_pool.stats(),
            hashmap_string: self.hashmap_string_pool.stats(),
            vec_u32: self.vec_u32_pool.stats(),
            vec_vec3_extended: self.vec_vec3_pool_extended.stats(),
        }
    }

    /// 预热所有池
    pub fn warm_up_all(&self) {
        self.vec_u8_pool.warm_up(32);
        self.vec_f32_pool.warm_up(32);
        self.vec_vec3_pool.warm_up(16);
        self.vec_mat4_pool.warm_up(16);
        self.string_pool.warm_up(32);
        self.hashmap_string_pool.warm_up(16);
        self.vec_u32_pool.warm_up(32);
        self.vec_vec3_pool_extended.warm_up(32);
    }

    /// 清空所有池
    pub fn clear_all(&self) {
        self.vec_u8_pool.clear();
        self.vec_f32_pool.clear();
        self.vec_vec3_pool.clear();
        self.vec_mat4_pool.clear();
        self.string_pool.clear();
        self.hashmap_string_pool.clear();
        self.vec_u32_pool.clear();
        self.vec_vec3_pool_extended.clear();
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 对象池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub vec_u8_initial: usize,
    pub vec_u8_max: usize,
    pub vec_f32_initial: usize,
    pub vec_f32_max: usize,
    pub vec_vec3_initial: usize,
    pub vec_vec3_max: usize,
    pub vec_mat4_initial: usize,
    pub vec_mat4_max: usize,
    pub string_initial: usize,
    pub string_max: usize,
    pub hashmap_initial: usize,
    pub hashmap_max: usize,
    pub vec_u32_initial: usize,
    pub vec_u32_max: usize,
    pub vec_vec3_extended_initial: usize,
    pub vec_vec3_extended_max: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            vec_u8_initial: 32,
            vec_u8_max: 256,
            vec_f32_initial: 32,
            vec_f32_max: 256,
            vec_vec3_initial: 16,
            vec_vec3_max: 128,
            vec_mat4_initial: 16,
            vec_mat4_max: 128,
            string_initial: 32,
            string_max: 256,
            hashmap_initial: 16,
            hashmap_max: 128,
            vec_u32_initial: 32,
            vec_u32_max: 256,
            vec_vec3_extended_initial: 32,
            vec_vec3_extended_max: 256,
        }
    }
}

/// 对象池管理器统计信息
#[derive(Debug, Clone)]
pub struct PoolManagerStats {
    pub vec_u8: PoolStats,
    pub vec_f32: PoolStats,
    pub vec_vec3: PoolStats,
    pub vec_mat4: PoolStats,
    pub string: PoolStats,
    pub hashmap_string: PoolStats,
    pub vec_u32: PoolStats,
    pub vec_vec3_extended: PoolStats,
}

impl PoolManagerStats {
    /// 计算总体缓存命中率
    pub fn overall_hit_rate(&self) -> f32 {
        let total_allocations = self.vec_u8.allocations
            + self.vec_f32.allocations
            + self.vec_vec3.allocations
            + self.vec_mat4.allocations
            + self.string.allocations
            + self.hashmap_string.allocations
            + self.vec_u32.allocations
            + self.vec_vec3_extended.allocations;

        let total_hits = self.vec_u8.cache_hits
            + self.vec_f32.cache_hits
            + self.vec_vec3.cache_hits
            + self.vec_mat4.cache_hits
            + self.string.cache_hits
            + self.hashmap_string.cache_hits
            + self.vec_u32.cache_hits
            + self.vec_vec3_extended.cache_hits;

        if total_allocations == 0 {
            0.0
        } else {
            total_hits as f32 / total_allocations as f32
        }
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        tracing::info!(
            target: "memory",
            "Pool Manager Stats - Overall Hit Rate: {:.2}%",
            self.overall_hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<u8>: {} allocations, {:.2}% hit rate",
            self.vec_u8.allocations,
            self.vec_u8.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<f32>: {} allocations, {:.2}% hit rate",
            self.vec_f32.allocations,
            self.vec_f32.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<Vec3>: {} allocations, {:.2}% hit rate",
            self.vec_vec3.allocations,
            self.vec_vec3.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<Mat4>: {} allocations, {:.2}% hit rate",
            self.vec_mat4.allocations,
            self.vec_mat4.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  String: {} allocations, {:.2}% hit rate",
            self.string.allocations,
            self.string.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  HashMap<String, String>: {} allocations, {:.2}% hit rate",
            self.hashmap_string.allocations,
            self.hashmap_string.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<u32>: {} allocations, {:.2}% hit rate",
            self.vec_u32.allocations,
            self.vec_u32.hit_rate() * 100.0
        );
        tracing::info!(
            target: "memory",
            "  Vec<Vec3> (extended): {} allocations, {:.2}% hit rate",
            self.vec_vec3_extended.allocations,
            self.vec_vec3_extended.hit_rate() * 100.0
        );
    }
}

// ============================================================================
// 可重置对象实现
// ============================================================================
// Resettable trait实现已移至game_engine_performance::memory::object_pool
// ============================================================================

// ============================================================================
// 便捷函数
// ============================================================================

/// 全局对象池管理器（单例）
static GLOBAL_POOL_MANAGER: std::sync::OnceLock<Arc<PoolManager>> = std::sync::OnceLock::new();

/// 获取全局对象池管理器
pub fn global_pool_manager() -> Arc<PoolManager> {
    GLOBAL_POOL_MANAGER.get_or_init(|| Arc::new(PoolManager::new())).clone()
}

/// 便捷函数：获取 Vec<u8> 对象
pub fn acquire_vec_u8() -> Vec<u8> {
    global_pool_manager().vec_u8_pool().acquire()
}

/// 便捷函数：归还 Vec<u8> 对象
pub fn release_vec_u8(vec: Vec<u8>) {
    global_pool_manager().vec_u8_pool().release(vec);
}

/// 便捷函数：获取 Vec<f32> 对象
pub fn acquire_vec_f32() -> Vec<f32> {
    global_pool_manager().vec_f32_pool().acquire()
}

/// 便捷函数：归还 Vec<f32> 对象
pub fn release_vec_f32(vec: Vec<f32>) {
    global_pool_manager().vec_f32_pool().release(vec);
}

/// 便捷函数：获取 Vec<Vec3> 对象
pub fn acquire_vec_vec3() -> Vec<Vec3> {
    global_pool_manager().vec_vec3_pool().acquire()
}

/// 便捷函数：归还 Vec<Vec3> 对象
pub fn release_vec_vec3(vec: Vec<Vec3>) {
    global_pool_manager().vec_vec3_pool().release(vec);
}

/// 便捷函数：获取 Vec<Mat4> 对象
pub fn acquire_vec_mat4() -> Vec<Mat4> {
    global_pool_manager().vec_mat4_pool().acquire()
}

/// 便捷函数：归还 Vec<Mat4> 对象
pub fn release_vec_mat4(vec: Vec<Mat4>) {
    global_pool_manager().vec_mat4_pool().release(vec);
}

/// 便捷函数：获取 String 对象
pub fn acquire_string() -> String {
    global_pool_manager().string_pool().acquire()
}

/// 便捷函数：归还 String 对象
pub fn release_string(s: String) {
    global_pool_manager().string_pool().release(s);
}

/// 便捷函数：获取 Vec<u32> 对象
pub fn acquire_vec_u32() -> Vec<u32> {
    global_pool_manager().vec_u32_pool().acquire()
}

/// 便捷函数：归还 Vec<u32> 对象
pub fn release_vec_u32(vec: Vec<u32>) {
    global_pool_manager().vec_u32_pool().release(vec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager() {
        let manager = PoolManager::new();

        // 测试 Vec<u8> 池
        let mut vec = manager.vec_u8_pool().acquire();
        vec.push(1);
        vec.push(2);
        manager.vec_u8_pool().release(vec);

        // 测试 String 池
        let mut s = manager.string_pool().acquire();
        s.push_str("test");
        manager.string_pool().release(s);

        // 获取统计信息
        let stats = manager.stats();
        assert!(stats.vec_u8.allocations > 0);
    }

    #[test]
    fn test_global_pool_manager() {
        // 测试全局池管理器
        let vec = acquire_vec_u8();
        release_vec_u8(vec);

        let s = acquire_string();
        release_string(s);
    }

    #[test]
    fn test_resettable() {
        let mut vec: Vec<u8> = vec![1, 2, 3];
        vec.clear();
        assert_eq!(vec.len(), 0);

        let mut s = String::from("test");
        s.clear();
        assert_eq!(s.len(), 0);
    }
}
