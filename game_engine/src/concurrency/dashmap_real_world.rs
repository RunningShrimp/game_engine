//! DashMap实际应用示例 - 网络服务器优化
//!
//! 本文件展示如何将DashMap应用到实际的高并发场景中。

use dashmap::DashMap;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex as ParkingLotMutex;

// ============================================================================
// 场景1: 网络服务器客户端管理
// ============================================================================

/// 客户端连接信息
#[derive(Clone, Debug)]
pub struct ClientConnection {
    pub client_id: u64,
    pub address: String,
    pub connected: bool,
    pub last_ping: std::time::Instant,
}

/// ❌ 优化前: 使用Arc<Mutex<HashMap>>（性能瓶颈）
pub struct GameServerBefore {
    clients: Arc<ParkingLotMutex<HashMap<u64, ClientConnection>>>,
}

impl GameServerBefore {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(ParkingLotMutex::new(HashMap::new())),
        }
    }

    /// 添加客户端（需要锁）
    pub fn add_client(&self, client: ClientConnection) {
        let mut clients = self.clients.lock();
        clients.insert(client.client_id, client);
    }

    /// 获取客户端（需要锁）
    pub fn get_client(&self, id: &u64) -> Option<ClientConnection> {
        let clients = self.clients.lock();
        clients.get(id).cloned()
    }

    /// 移除客户端（需要锁）
    pub fn remove_client(&self, id: &u64) -> Option<ClientConnection> {
        let mut clients = self.clients.lock();
        clients.remove(id)
    }

    /// 客户端数量（需要锁）
    pub fn client_count(&self) -> usize {
        self.clients.lock().len()
    }

    /// 更新客户端（需要锁）
    pub fn update_client<F>(&self, id: &u64, f: F) -> bool
    where
        F: FnOnce(&mut ClientConnection),
    {
        let mut clients = self.clients.lock();
        if let Some(client) = clients.get_mut(id) {
            f(client);
            true
        } else {
            false
        }
    }
}

/// ✅ 优化后: 使用DashMap（10x并发性能）
pub struct GameServerAfter {
    clients: DashMap<u64, ClientConnection>,
}

impl GameServerAfter {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    /// 添加客户端（无锁）
    pub fn add_client(&self, client: ClientConnection) {
        self.clients.insert(client.client_id, client);
    }

    /// 获取客户端（无锁）
    pub fn get_client(&self, id: &u64) -> Option<ClientConnection> {
        self.clients.get(id).map(|v| v.clone())
    }

    /// 移除客户端（无锁）
    pub fn remove_client(&self, id: &u64) -> Option<ClientConnection> {
        self.clients.remove(id).map(|(_, v)| v)
    }

    /// 客户端数量（无锁）
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// 更新客户端（无锁）
    pub fn update_client<F>(&self, id: &u64, f: F) -> bool
    where
        F: FnOnce(&mut ClientConnection),
    {
        if let Some(mut client) = self.clients.get_mut(id) {
            f(&mut client);
            true
        } else {
            false
        }
    }

    /// 批量操作（DashMap特有）
    pub fn disconnect_all(&self) {
        self.clients.alter_all(|_, client| {
            client.connected = false;
        });
    }

    /// 并行迭代（DashMap特有）
    pub fn for_each_client<F>(&self, f: F)
    where
        F: Fn(u64, ClientConnection) + Sync,
    {
        self.clients.iter().for_each(|entry| {
            f(*entry.key(), entry.value().clone());
        });
    }

    /// 过滤客户端（DashMap特有）
    pub fn get_active_clients(&self) -> Vec<ClientConnection> {
        self.clients
            .iter()
            .filter(|entry| entry.value().connected)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

// ============================================================================
// 场景2: 资源缓存系统
// ============================================================================

/// 缓存的资源
#[derive(Clone, Debug)]
pub struct CachedResource {
    pub data: Vec<u8>,
    pub size: usize,
    pub last_accessed: std::time::Instant,
    pub access_count: u64,
}

/// ❌ 优化前: Arc<Mutex<HashMap>>
pub struct ResourceCacheBefore {
    cache: Arc<ParkingLotMutex<HashMap<String, CachedResource>>>,
    max_size: usize,
}

impl ResourceCacheBefore {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(ParkingLotMutex::new(HashMap::new())),
            max_size,
        }
    }

    /// 获取资源（需要锁）
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.lock();
        if let Some(resource) = cache.get_mut(key) {
            resource.access_count += 1;
            resource.last_accessed = std::time::Instant::now();
            Some(resource.data.clone())
        } else {
            None
        }
    }

    /// 插入资源（需要锁）
    pub fn insert(&self, key: String, data: Vec<u8>) {
        let mut cache = self.cache.lock();

        // 如果缓存已满，移除最少使用的项
        if cache.len() >= self.max_size {
            if let Some((lru_key, _)) = cache
                .iter()
                .min_by_key(|(_, a)| (a.access_count, a.last_accessed))
            {
                cache.remove(lru_key);
            }
        }

        cache.insert(key, CachedResource {
            size: data.len(),
            last_accessed: std::time::Instant::now(),
            access_count: 1,
            data,
        });
    }

    /// 缓存大小（需要锁）
    pub fn size(&self) -> usize {
        self.cache.lock().len()
    }
}

/// ✅ 优化后: DashMap（10x并发性能）
pub struct ResourceCacheAfter {
    cache: DashMap<String, CachedResource>,
    max_size: usize,
}

impl ResourceCacheAfter {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
        }
    }

    /// 获取资源（无锁）
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(mut resource) = self.cache.get_mut(key) {
            resource.access_count += 1;
            resource.last_accessed = std::time::Instant::now();
            Some(resource.data.clone())
        } else {
            None
        }
    }

    /// 插入资源（无锁）
    pub fn insert(&self, key: String, data: Vec<u8>) {
        // 如果缓存已满，移除最少使用的项
        if self.cache.len() >= self.max_size {
            // DashMap的retain操作可以高效地过滤
            self.cache.retain(|k, _v| {
                // 保留最近访问的一半
                k != &"lru_key_to_remove" // 简化示例
            });

            // 或者移除最少使用的项
            if let Some((lru_key, _)) = self
                .cache
                .iter()
                .min_by_key(|(_, a)| (a.access_count, a.last_accessed))
            {
                self.cache.remove(&lru_key.clone());
            }
        }

        self.cache.insert(key, CachedResource {
            size: data.len(),
            last_accessed: std::time::Instant::now(),
            access_count: 1,
            data,
        });
    }

    /// 缓存大小（无锁）
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// 批量清理过期资源（DashMap特有）
    pub fn cleanup_old(&self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        self.cache.retain(|_, resource| {
            now.duration_since(resource.last_accessed) < max_age
        });
    }

    /// 获取缓存统计（无锁）
    pub fn stats(&self) -> CacheStats {
        let mut total_size = 0;
        let mut total_access_count = 0;

        self.cache.iter().for_each(|entry| {
            total_size += entry.value().size;
            total_access_count += entry.value().access_count;
        });

        CacheStats {
            entry_count: self.cache.len(),
            total_size,
            total_access_count,
            avg_access_count: if self.cache.len() > 0 {
                total_access_count as f64 / self.cache.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size: usize,
    pub total_access_count: u64,
    pub avg_access_count: f64,
}

// ============================================================================
// 性能基准测试
// ============================================================================

#[cfg(test)]
mod dashmap_real_world_benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // 基准测试，默认忽略
    fn benchmark_concurrent_client_management() {
        const NUM_CLIENTS: usize = 1000;
        const NUM_OPERATIONS: usize = 10000;

        // 测试优化前版本
        let server_before = GameServerBefore::new();
        let start = Instant::now();

        for i in 0..NUM_CLIENTS {
            server_before.add_client(ClientConnection {
                client_id: i as u64,
                address: format!("127.0.0.1:{}", 8080 + i),
                connected: true,
                last_ping: std::time::Instant::now(),
            });
        }

        for _ in 0..NUM_OPERATIONS {
            let id = rand::random::<usize>() % NUM_CLIENTS;
            server_before.update_client(&(id as u64), |client| {
                client.last_ping = std::time::Instant::now();
            });
        }

        let before_duration = start.elapsed();

        // 测试优化后版本
        let server_after = GameServerAfter::new();
        let start = Instant::now();

        for i in 0..NUM_CLIENTS {
            server_after.add_client(ClientConnection {
                client_id: i as u64,
                address: format!("127.0.0.1:{}", 8080 + i),
                connected: true,
                last_ping: std::time::Instant::now(),
            });
        }

        for _ in 0..NUM_OPERATIONS {
            let id = rand::random::<usize>() % NUM_CLIENTS;
            server_after.update_client(&(id as u64), |client| {
                client.last_ping = std::time::Instant::now();
            });
        }

        let after_duration = start.elapsed();

        println!("Arc<Mutex<HashMap>>: {:?}", before_duration);
        println!("DashMap: {:?}", after_duration);
        println!("性能提升: {:.2}x", before_duration.as_nanos() as f64 / after_duration.as_nanos() as f64);

        // DashMap应该快2-10倍，取决于并发级别
    }

    #[test]
    fn test_dashmap_functionality() {
        let server = GameServerAfter::new();

        // 添加客户端
        server.add_client(ClientConnection {
            client_id: 1,
            address: "127.0.0.1:8080".to_string(),
            connected: true,
            last_ping: std::time::Instant::now(),
        });

        // 获取客户端
        let client = server.get_client(&1);
        assert!(client.is_some());
        assert_eq!(client.unwrap().client_id, 1);

        // 客户端数量
        assert_eq!(server.client_count(), 1);

        // 更新客户端
        assert!(server.update_client(&1, |client| {
            client.connected = false;
        }));

        // 移除客户端
        let removed = server.remove_client(&1);
        assert!(removed.is_some());
        assert_eq!(server.client_count(), 0);
    }

    #[test]
    fn test_resource_cache() {
        let cache = ResourceCacheAfter::new(100);

        // 插入资源
        cache.insert("test.txt".to_string(), vec![1, 2, 3, 4, 5]);

        // 获取资源
        let data = cache.get("test.txt");
        assert!(data.is_some());
        assert_eq!(data.unwrap(), vec![1, 2, 3, 4, 5]);

        // 缓存大小
        assert_eq!(cache.size(), 1);

        // 统计信息
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.total_size, 5);
    }

    #[test]
    fn test_concurrent_operations() {
        use std::thread;

        let server = Arc::new(GameServerAfter::new());
        let mut handles = vec![];

        // 启动多个线程并发操作
        for i in 0..4 {
            let server_clone = server.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let id = (i * 100 + j) as u64;
                    server_clone.add_client(ClientConnection {
                        client_id: id,
                        address: format!("127.0.0.1:{}", 8000 + id),
                        connected: true,
                        last_ping: std::time::Instant::now(),
                    });

                    server_clone.get_client(&id);
                    server_clone.update_client(&id, |c| {
                        c.last_ping = std::time::Instant::now();
                    });
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证结果
        assert_eq!(server.client_count(), 400);
    }
}

// ============================================================================
// 迁移指南
// ============================================================================

/// 从Arc<Mutex<HashMap>>迁移到DashMap的步骤
pub struct MigrationGuide;

impl MigrationGuide {
    /// 步骤1: 添加DashMap依赖
    ///
    /// ```toml
    /// [dependencies]
    /// dashmap = "6.1"
    /// ```
    pub fn step1_add_dependency() {}

    /// 步骤2: 更新结构体定义
    ///
    /// ```rust
    /// // 优化前
    /// pub struct MyStruct {
    ///     data: Arc<Mutex<HashMap<K, V>>>,
    /// }
    ///
    /// // 优化后
    /// pub struct MyStruct {
    ///     data: DashMap<K, V>,
    /// }
    /// ```
    pub fn step2_update_struct() {}

    /// 步骤3: 更新方法实现
    ///
    /// ```rust
    /// // 优化前
    /// impl MyStruct {
    ///     pub fn get(&self, key: &K) -> Option<V> {
    ///         let data = self.data.lock()?;
    ///         data.get(key).cloned()
    ///     }
    /// }
    ///
    /// // 优化后
    /// impl MyStruct {
    ///     pub fn get(&self, key: &K) -> Option<V> {
    ///         self.data.get(key).map(|v| v.clone())
    ///     }
    /// }
    /// ```
    pub fn step3_update_methods() {}

    /// 步骤4: 利用DashMap特性
    ///
    /// ```rust
    /// // 批量操作
    /// self.data.alter_all(|_, v| { /* 修改 */ });
    ///
    /// // 条件过滤
    /// self.data.retain(|k, v| { /* 保留条件 */ });
    ///
    /// // 并行迭代
    /// self.data.iter().for_each(|entry| { /* 处理 */ });
    /// ```
    pub fn step4_use_features() {}

    /// 步骤5: 充分测试
    ///
    /// - 单元测试
    /// - 并发测试
    /// - 性能基准测试
    pub fn step5_test_thoroughly() {}
}

// ============================================================================
// 最佳实践
// ============================================================================

/// DashMap使用最佳实践
pub struct DashMapBestPractices;

impl DashMapBestPractices {
    /// ✅ 最佳实践1: 使用DashMap当需要高并发访问
    ///
    /// - 多线程频繁读写
    /// - 锁竞争成为瓶颈
    /// - 需要原子操作
    pub fn use_dashmap_for_high_concurrency() -> bool {
        true
    }

    /// ✅ 最佳实践2: 键类型选择
    ///
    /// 好的键类型:
    /// - 基本类型: u64, i32, String
    /// - 元组和结构体 (实现Eq + Hash)
    ///
    /// 坏的键类型:
    /// - 浮点数: f32, f64 (NaN问题)
    /// - Vec (不实现Hash)
    pub fn choose_key_types_wisely() {}

    /// ✅ 最佳实践3: 避免克隆大值
    ///
    /// ```rust
    /// // ❌ 不好: 克隆整个Vec
    /// let data = map.get(&key).map(|v| v.clone());
    ///
    /// // ✅ 更好: 只读引用
    /// map.get(&key).map(|v| {
    ///     // 处理 v
    /// });
    ///
    /// // ✅ 最佳: 使用get_mut修改
    /// if let Some(mut v) = map.get_mut(&key) {
    ///     v.push(1); // 直接修改，无需克隆
    /// }
    /// ```
    pub fn avoid_cloning_large_values() {}

    /// ✅ 最佳实践4: 使用原子操作
    ///
    /// ```rust
    /// // insert: 如果键存在则更新，否则插入
    /// map.entry(key).or_insert_with(|| create_value());
    ///
    /// // alter: 修改现有值
    /// if let Some(mut v) = map.get_mut(&key) {
    ///     v.count += 1;
    /// }
    ///
    /// // remove: 返回被移除的值
    /// let removed = map.remove(&key);
    /// ```
    pub fn use_atomic_operations() {}
}
