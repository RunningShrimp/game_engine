// 内存优化模块
//
// 提供对象池、缓存策略和内存分配优化

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// 对象池系统
// ============================================================================

/// 对象池trait
pub trait Pool<T> {
    /// 从池中获取对象
    fn acquire(&self) -> Pooled<T>;
    /// 将对象返回池中
    fn release(&self, item: T);
    /// 获取池中可用对象数量
    fn available(&self) -> usize;
    /// 获取池中总对象数量
    fn total(&self) -> usize;
}

/// 池化对象包装器
pub struct Pooled<T> {
    item: Option<T>,
    pool: Arc<dyn Pool<T>>,
}

impl<T> Pooled<T> {
    /// 创建新的池化对象
    pub fn new(item: T, pool: Arc<dyn Pool<T>>) -> Self {
        Self {
            item: Some(item),
            pool,
        }
    }

    /// 获取内部对象的引用
    pub fn get(&self) -> &T {
        self.item.as_ref().expect("Pooled item not available")
    }

    /// 获取内部对象的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        self.item.as_mut().expect("Pooled item not available")
    }

    /// 解包获取内部对象（不会返回池中）
    pub fn into_inner(mut self) -> T {
        self.item.take().expect("Pooled item not available")
    }
}

impl<T> Drop for Pooled<T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            self.pool.release(item);
        }
    }
}

/// 通用对象池
pub struct ObjectPool<T, F>
where
    F: Fn() -> T,
{
    /// 对象工厂函数
    factory: F,
    /// 可用对象队列
    available: Vec<T>,
    /// 最大池大小
    max_size: usize,
    /// 当前创建的对象总数
    total_count: usize,
    /// 重置函数
    reset_fn: Option<Box<dyn Fn(&mut T)>>,
}

impl<T, F> ObjectPool<T, F>
where
    T: Send + 'static,
    F: Fn() -> T + Send + 'static,
{
    /// 创建新的对象池
    pub fn new(factory: F, initial_size: usize, max_size: usize) -> Self {
        let mut pool = Self {
            factory,
            available: Vec::with_capacity(initial_size),
            max_size,
            total_count: 0,
            reset_fn: None,
        };

        // 预创建对象
        for _ in 0..initial_size {
            pool.available.push((pool.factory)());
            pool.total_count += 1;
        }

        pool
    }

    /// 设置重置函数
    pub fn with_reset(mut self, reset_fn: Box<dyn Fn(&mut T)>) -> Self {
        self.reset_fn = Some(reset_fn);
        self
    }

    /// 从池中获取对象
    pub fn acquire(&self) -> Pooled<T> {
        // 这个方法需要内部可变性，实际实现需要使用Mutex或RwLock
        // 这里简化实现
        panic!("Use ObjectPool::acquire_sync instead in single-threaded context");
    }

    /// 将对象返回池中
    pub fn release(&self, _item: T) {
        // 需要内部可变性
        panic!("Use ObjectPool::release_sync instead in single-threaded context");
    }

    /// 获取可用对象数量
    pub fn available(&self) -> usize {
        self.available.len()
    }

    /// 获取总对象数量
    pub fn total(&self) -> usize {
        self.total_count
    }
}

/// 同步对象池（单线程版本）
pub struct SyncObjectPool<T, F>
where
    F: Fn() -> T,
{
    /// 对象工厂函数
    factory: F,
    /// 可用对象队列
    available: Vec<T>,
    /// 最大池大小
    max_size: usize,
    /// 当前创建的对象总数
    total_count: usize,
    /// 重置函数
    reset_fn: Option<Box<dyn Fn(&mut T)>>,
}

impl<T, F> SyncObjectPool<T, F>
where
    F: Fn() -> T,
{
    /// 创建新的同步对象池
    pub fn new(factory: F, initial_size: usize, max_size: usize) -> Self {
        let mut pool = Self {
            factory,
            available: Vec::with_capacity(initial_size),
            max_size,
            total_count: 0,
            reset_fn: None,
        };

        // 预创建对象
        for _ in 0..initial_size {
            pool.available.push((pool.factory)());
            pool.total_count += 1;
        }

        pool
    }

    /// 设置重置函数
    pub fn with_reset(mut self, reset_fn: Box<dyn Fn(&mut T)>) -> Self {
        self.reset_fn = Some(reset_fn);
        self
    }

    /// 从池中获取对象
    pub fn acquire_sync(&mut self) -> T {
        if let Some(mut item) = self.available.pop() {
            // 重置对象状态
            if let Some(ref reset_fn) = self.reset_fn {
                reset_fn(&mut item);
            }
            item
        } else if self.total_count < self.max_size {
            let item = (self.factory)();
            self.total_count += 1;
            item
        } else {
            panic!("Object pool exhausted");
        }
    }

    /// 将对象返回池中
    pub fn release_sync(&mut self, item: T) {
        if self.available.len() < self.max_size {
            self.available.push(item);
        }
        // 如果池已满，丢弃对象
    }

    /// 获取可用对象数量
    pub fn available(&self) -> usize {
        self.available.len()
    }

    /// 获取总对象数量
    pub fn total(&self) -> usize {
        self.total_count
    }

    /// 清空池
    pub fn clear(&mut self) {
        self.available.clear();
        self.total_count = 0;
    }
}

// ============================================================================
// 实体对象池
// ============================================================================

/// 简化的实体ID（用于对象池）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u32);

impl EntityId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

/// 实体对象池
#[derive(Debug)]
pub struct EntityPool {
    /// 可用实体ID队列
    available_entities: Vec<EntityId>,
    /// 下一个实体ID
    next_id: u32,
    /// 最大实体数量
    max_entities: usize,
    /// 活跃实体计数
    active_count: usize,
}

impl EntityPool {
    /// 创建新的实体池
    pub fn new(max_entities: usize) -> Self {
        Self {
            available_entities: Vec::new(),
            next_id: 0,
            max_entities,
            active_count: 0,
        }
    }

    /// 分配新实体
    pub fn allocate(&mut self) -> Option<EntityId> {
        if self.active_count >= self.max_entities {
            return None;
        }

        if let Some(entity) = self.available_entities.pop() {
            self.active_count += 1;
            Some(entity)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.active_count += 1;
            Some(EntityId::new(id))
        }
    }

    /// 释放实体
    pub fn deallocate(&mut self, entity: EntityId) {
        self.available_entities.push(entity);
        self.active_count -= 1;
    }

    /// 获取活跃实体数量
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// 获取可用实体数量
    pub fn available_count(&self) -> usize {
        self.available_entities.len()
    }

    /// 清空池
    pub fn clear(&mut self) {
        self.available_entities.clear();
        self.next_id = 0;
        self.active_count = 0;
    }
}

// ============================================================================
// 组件对象池
// ============================================================================

/// 组件对象池
pub struct ComponentPool<T> {
    /// 组件数据
    components: Vec<Option<T>>,
    /// 可用索引队列
    free_indices: Vec<usize>,
    /// 最大组件数量
    max_components: usize,
    /// 活跃组件计数
    active_count: usize,
}

impl<T> ComponentPool<T> {
    /// 创建新的组件池
    pub fn new(max_components: usize) -> Self {
        Self {
            components: Vec::with_capacity(max_components),
            free_indices: Vec::new(),
            max_components,
            active_count: 0,
        }
    }

    /// 分配组件槽位
    pub fn allocate(&mut self) -> Option<usize> {
        if self.active_count >= self.max_components {
            return None;
        }

        if let Some(index) = self.free_indices.pop() {
            self.active_count += 1;
            Some(index)
        } else {
            let index = self.components.len();
            if index < self.max_components {
                self.components.push(None);
                self.active_count += 1;
                Some(index)
            } else {
                None
            }
        }
    }

    /// 释放组件槽位
    pub fn deallocate(&mut self, index: usize) -> Option<T> {
        if index < self.components.len() {
            if let Some(component) = self.components[index].take() {
                self.free_indices.push(index);
                self.active_count -= 1;
                Some(component)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 设置组件
    pub fn set(&mut self, index: usize, component: T) -> bool {
        if index < self.components.len() {
            self.components[index] = Some(component);
            true
        } else {
            false
        }
    }

    /// 获取组件
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.components.len() {
            self.components[index].as_ref()
        } else {
            None
        }
    }

    /// 获取组件的可变引用
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.components.len() {
            self.components[index].as_mut()
        } else {
            None
        }
    }

    /// 获取活跃组件数量
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// 清空池
    pub fn clear(&mut self) {
        self.components.clear();
        self.free_indices.clear();
        self.active_count = 0;
    }
}

// ============================================================================
// 缓存策略
// ============================================================================

/// LRU缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<K, V> {
    key: K,
    value: V,
    last_access: Instant,
    access_count: u64,
}

/// LRU缓存
#[derive(Debug)]
pub struct LruCache<K, V>
where
    K: PartialEq + Eq + std::hash::Hash + Clone,
{
    entries: HashMap<K, CacheEntry<K, V>>,
    max_capacity: usize,
    max_age: Duration,
}

impl<K, V> LruCache<K, V>
where
    K: PartialEq + Eq + std::hash::Hash + Clone,
{
    /// 创建新的LRU缓存
    pub fn new(max_capacity: usize, max_age: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            max_capacity,
            max_age,
        }
    }

    /// 插入或更新条目
    pub fn insert(&mut self, key: K, value: V) {
        // 如果缓存已满，移除最少使用的条目
        if self.entries.len() >= self.max_capacity && !self.entries.contains_key(&key) {
            self.evict_lru();
        }

        let entry = CacheEntry {
            key: key.clone(),
            value,
            last_access: Instant::now(),
            access_count: 0,
        };
        self.entries.insert(key, entry);
    }

    /// 获取条目
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_access = Instant::now();
            entry.access_count += 1;
            Some(&entry.value)
        } else {
            None
        }
    }

    /// 移除条目
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// 移除最少使用的条目
    fn evict_lru(&mut self) {
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| (entry.last_access, entry.access_count))
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            self.entries.remove(&key);
        }
    }

    /// 移除过期条目
    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_access) > self.max_age)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.entries.remove(&key);
        }
    }
}

// ============================================================================
// 着色器缓存
// ============================================================================

/// 着色器缓存
#[derive(Debug)]
pub struct ShaderCache {
    /// 已编译的着色器
    shaders: HashMap<String, Vec<u32>>,
    /// 着色器源代码
    sources: HashMap<String, String>,
    /// LRU缓存
    lru: LruCache<String, Vec<u32>>,
    /// 最大缓存大小
    max_shaders: usize,
}

impl ShaderCache {
    /// 创建新的着色器缓存
    pub fn new(max_shaders: usize) -> Self {
        Self {
            shaders: HashMap::new(),
            sources: HashMap::new(),
            lru: LruCache::new(max_shaders, Duration::from_secs(3600)),
            max_shaders,
        }
    }

    /// 预热缓存（加载常用着色器）
    pub fn warmup(&mut self, common_shaders: Vec<(String, String)>) {
        for (name, source) in common_shaders {
            self.sources.insert(name.clone(), source);
            // 这里应该编译着色器，简化为占位符
            self.lru.insert(name, vec![]);
        }
    }

    /// 获取已编译的着色器
    pub fn get(&mut self, name: &str) -> Option<&[u32]> {
        if let Some(spirv) = self.lru.get(&name.to_string()) {
            Some(spirv)
        } else if let Some(spirv) = self.shaders.get(name) {
            Some(spirv)
        } else {
            None
        }
    }

    /// 插入编译好的着色器
    pub fn insert(&mut self, name: String, spirv: Vec<u32>) {
        self.shaders.insert(name.clone(), spirv.clone());
        self.lru.insert(name, spirv);
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.shaders.clear();
        self.sources.clear();
        self.lru.clear();
    }
}

// ============================================================================
// 资源预加载优化
// ============================================================================

/// 资源预加载器
#[derive(Debug)]
pub struct ResourcePreloader {
    /// 待加载资源队列
    pending: Vec<String>,
    /// 已加载资源
    loaded: HashMap<String, Vec<u8>>,
    /// 最大并发加载数
    max_concurrent: usize,
}

impl ResourcePreloader {
    /// 创建新的预加载器
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: Vec::new(),
            loaded: HashMap::new(),
            max_concurrent,
        }
    }

    /// 添加待加载资源
    pub fn add(&mut self, resource_path: String) {
        self.pending.push(resource_path);
    }

    /// 执行预加载
    pub fn load(&mut self) {
        // 简化实现：加载所有待加载资源
        for path in self.pending.drain(..) {
            // 实际应该从文件系统加载
            self.loaded.insert(path, vec![]);
        }
    }

    /// 获取已加载资源
    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.loaded.get(path).map(|data| data.as_slice())
    }

    /// 清空已加载资源
    pub fn clear(&mut self) {
        self.loaded.clear();
    }
}

// ============================================================================
// 减少分配优化
// ============================================================================

/// Vec缓冲区重用池
#[derive(Debug)]
pub struct VecBufferPool<T> {
    buffers: Vec<Vec<T>>,
    max_buffers: usize,
}

impl<T> VecBufferPool<T> {
    /// 创建新的缓冲区池
    pub fn new(initial_size: usize, max_buffers: usize) -> Self {
        let mut buffers = Vec::with_capacity(initial_size);
        for _ in 0..initial_size {
            buffers.push(Vec::new());
        }

        Self {
            buffers,
            max_buffers,
        }
    }

    /// 获取缓冲区
    pub fn acquire(&mut self) -> Vec<T> {
        self.buffers
            .pop()
            .unwrap_or_else(|| Vec::new())
    }

    /// 返回缓冲区
    pub fn release(&mut self, mut buffer: Vec<T>) {
        buffer.clear();
        if self.buffers.len() < self.max_buffers {
            self.buffers.push(buffer);
        }
    }

    /// 获取可用缓冲区数量
    pub fn available(&self) -> usize {
        self.buffers.len()
    }
}

/// String interning（字符串驻留）
#[derive(Debug)]
pub struct StringInterner {
    strings: HashMap<String, u32>,
    rev_strings: Vec<String>,
    next_id: u32,
}

impl StringInterner {
    /// 创建新的字符串驻留器
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            rev_strings: Vec::new(),
            next_id: 0,
        }
    }

    /// 驻留字符串
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.strings.insert(s.to_string(), id);
            self.rev_strings.push(s.to_string());
            id
        }
    }

    /// 根据ID获取字符串
    pub fn get(&self, id: u32) -> Option<&str> {
        self.rev_strings.get(id as usize).map(|s| s.as_str())
    }

    /// 清空驻留数据
    pub fn clear(&mut self) {
        self.strings.clear();
        self.rev_strings.clear();
        self.next_id = 0;
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_pool() {
        let mut pool = EntityPool::new(10);

        let entity1 = pool.allocate().unwrap();
        let entity2 = pool.allocate().unwrap();

        assert_eq!(pool.active_count(), 2);

        pool.deallocate(entity1);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn test_component_pool() {
        let mut pool = ComponentPool::<i32>::new(10);

        let idx1 = pool.allocate().unwrap();
        pool.set(idx1, 42);
        assert_eq!(*pool.get(idx1).unwrap(), 42);

        let _ = pool.deallocate(idx1);
        assert!(pool.get(idx1).is_none());
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2, Duration::from_secs(60));

        cache.insert("key1".to_string(), "value1");
        cache.insert("key2".to_string(), "value2");

        assert_eq!(cache.get(&"key1".to_string()).unwrap(), &"value1");

        // 插入第三个条目应该驱逐最少使用的条目
        cache.insert("key3".to_string(), "value3");
        assert!(cache.get(&"key2".to_string()).is_none());
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello"); // 应该返回相同的ID

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(interner.get(id1), Some("hello"));
    }

    #[test]
    fn test_vec_buffer_pool() {
        let mut pool = VecBufferPool::<i32>::new(2, 5);

        let mut buf1 = pool.acquire();
        buf1.push(1);
        buf1.push(2);

        pool.release(buf1);

        assert_eq!(pool.available(), 1);

        let buf2 = pool.acquire();
        assert!(buf2.is_empty()); // 缓冲区应该被清空
    }
}
