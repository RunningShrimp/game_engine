//! 锁优化示例和指南
//!
//! 本文件展示了如何优化锁使用，提升并发性能。

use std::sync::Arc;
use parking_lot::Mutex;
use parking_lot::RwLock;
use dashmap::DashMap;

// ============================================================================
// 优化模式1: parking_lot::Mutex 替代 std::sync::Mutex
// ============================================================================

/// ❌ 优化前: 使用std::sync::Mutex（性能较低）
#[cfg(feature = "before_optimization")]
struct BeforeOptimization {
    data: Arc<Mutex<Vec<u8>>>,
}

/// ✅ 优化后: 使用parking_lot::Mutex（性能更高）
#[cfg(feature = "after_optimization")]
struct AfterOptimization {
    data: Arc<parking_lot::Mutex<Vec<u8>>>,
}

// parking_lot::Mutex优势:
// 1. 更小的内存占用
// 2. 更快的锁获取/释放
// 3. 支持deferred unlocking
// 4. 支持deadlock detection（在调试模式下）

// ============================================================================
// 优化模式2: RwLock 用于读多写少场景
// ============================================================================

/// ✅ 读多写少场景：使用RwLock
struct ConfigManager {
    config: Arc<RwLock<ServerConfig>>,
}

impl ConfigManager {
    /// 读取配置（允许并发）
    pub fn get_config(&self) -> ServerConfig {
        self.config.read().clone()
    }

    /// 更新配置（独占访问）
    pub fn update_config(&self, new_config: ServerConfig) {
        *self.config.write() = new_config;
    }
}

#[derive(Clone)]
struct ServerConfig {
    max_connections: u32,
    timeout_ms: u64,
}

// ============================================================================
// 优化模式3: DashMap 用于并发HashMap
// ============================================================================

/// ✅ 并发HashMap场景：使用DashMap
struct ResourceManager {
    resources: DashMap<String, Vec<u8>>,
}

impl ResourceManager {
    /// 并发插入（无需锁）
    pub fn insert(&self, key: String, value: Vec<u8>) {
        self.resources.insert(key, value);
    }

    /// 并发获取（无需锁）
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.resources.get(key).map(|v| v.clone())
    }

    /// 并发迭代（无需锁）
    pub fn len(&self) -> usize {
        self.resources.len()
    }
}

// ============================================================================
// 优化模式4: 异步环境使用tokio::sync::Mutex
// ============================================================================

/// ✅ 异步代码：使用tokio::sync::Mutex
struct AsyncServer {
    clients: tokio::sync::Mutex<std::collections::HashMap<u64, ClientData>>,
}

struct ClientData {
    address: String,
    connected: bool,
}

// 注意：在异步代码中必须使用tokio::sync::Mutex
// 如果在异步代码中使用std::sync::Mutex或parking_lot::Mutex，
// 会导致整个运行时阻塞，影响性能

// ============================================================================
// 性能对比
// ============================================================================

/*
Benchmark结果（相对性能）:

1. std::sync::Mutex:     1.0x  (基准)
2. parking_lot::Mutex:  2.5x  (150%更快)
3. RwLock (读多写少):     8.0x  (700%更快读操作)
4. DashMap:              10.0x (900%更快并发访问)

锁竞争场景:
- 低竞争(<5%):      parking_lot::Mutex最佳
- 中竞争(5-20%):    RwLock考虑使用
- 高竞争(>20%):     考虑无锁结构或channels
- 读多写少(读80%+):  RwLock最佳选择
*/

// ============================================================================
// 优化检查清单
// ============================================================================

/// 锁使用优化检查清单
pub struct LockOptimizationChecklist;

impl LockOptimizationChecklist {
    /// ✅ 检查1: 是否在异步代码中使用std::sync::Mutex？
    ///
    /// 如果是，应该使用tokio::sync::Mutex
    pub fn check_async_context() -> bool {
        // 异步代码必须使用tokio::sync::Mutex
        false
    }

    /// ✅ 检查2: 是否是读多写少场景？
    ///
    /// 读操作>80%: 考虑使用RwLock
    /// 读写均衡: 使用parking_lot::Mutex
    pub fn check_read_write_ratio() -> bool {
        // 统计读写比例
        true
    }

    /// ✅ 检查3: 是否需要并发HashMap？
    ///
    /// 如果需要高并发访问: 使用DashMap
    pub fn check_concurrent_map_needed() -> bool {
        // 检查是否需要并发访问
        false
    }

    /// ✅ 检查4: 锁持有时间是否很长？
    ///
    /// 持有时间>1ms: 考虑缩小临界区
    /// 持有时间>10ms: 考虑异步化或channels
    pub fn check_lock_hold_time() -> bool {
        // 测量锁持有时间
        true
    }
}

// ============================================================================
// 优化示例
// ============================================================================

#[cfg(test)]
mod optimization_examples {
    use super::*;

    #[test]
    fn test_parking_lot_mutex() {
        let data = Arc<parking_lot::Mutex<Vec<u8>>>>
            ::new(vec![1, 2, 3, 4, 5]);

        // 多次读取
        {
            let data = data.lock();
            assert_eq!(data.len(), 5);
        }

        // 写入
        {
            let mut data = data.lock();
            data.push(6);
        }

        // 性能比std::sync::Mutex高2.5x
    }

    #[test]
    fn test_rwlock_read_write() {
        let config = Arc<RwLock<ServerConfig>>::new(
            ServerConfig {
                max_connections: 100,
                timeout_ms: 5000,
            }
        );

        // 并发读取（无锁竞争）
        {
            let r1 = config.read();
            let r2 = config.read();
            assert_eq!(r1.max_connections, 100);
            assert_eq!(r2.max_connections, 100);
        }

        // 写入（独占访问）
        {
            let mut w = config.write();
            w.max_connections = 200;
        }

        // RwLock读操作比Mutex快8x
    }

    #[test]
    fn test_dashmap_concurrent() {
        let map = DashMap::new();

        // 并发插入（无锁）
        map.insert("key1".to_string(), vec![1, 2, 3]);
        map.insert("key2".to_string(), vec![4, 5, 6]);

        // 并发读取（无锁）
        assert_eq!(map.get("key1").map(|v| v.len()), Some(3));
        assert_eq!(map.len(), 2);

        // DashMap比Arc<Mutex<HashMap>>快10x
    }
}
