//! GI缓存系统
//!
//! 提供高效的缓存管理：
//! - 纹理缓存
//! - 光照数据缓存
//! - LRU策略
//! - 内存管理

use crate::render::{RenderDevice, TextureFormat};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// GI缓存
pub struct GICache {
    device: Arc<RenderDevice>,

    // 缓存配置
    config: CacheConfig,

    // 纹理缓存
    texture_cache: Arc<Mutex<TextureCache>>,

    // 数据缓存
    data_cache: Arc<Mutex<DataCache>>,

    // 统计
    stats: Arc<Mutex<CacheStats>>,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大内存 (字节)
    pub max_memory: usize,

    /// 纹理缓存比例 (0.0 - 1.0)
    pub texture_ratio: f32,

    /// 数据缓存比例 (0.0 - 1.0)
    pub data_ratio: f32,

    /// LRU设置
    pub lru_enabled: bool,

    /// 自动清理
    pub auto_cleanup: bool,

    /// 清理间隔
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 512 * 1024 * 1024, // 512MB
            texture_ratio: 0.7,
            data_ratio: 0.3,
            lru_enabled: true,
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(30),
        }
    }
}

/// 纹理缓存
struct TextureCache {
    entries: HashMap<String, TextureCacheEntry>,
    total_memory: usize,
    max_memory: usize,
    access_order: Vec<String>,
}

/// 纹理缓存条目
struct TextureCacheEntry {
    texture: wgpu::Texture,
    size: usize,
    last_access: Instant,
    access_count: u64,
}

/// 数据缓存
struct DataCache {
    entries: HashMap<String, DataCacheEntry>,
    total_memory: usize,
    max_memory: usize,
    access_order: Vec<String>,
}

/// 数据缓存条目
struct DataCacheEntry {
    data: Vec<u8>,
    size: usize,
    last_access: Instant,
    access_count: u64,
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 纹理缓存命中数
    pub texture_hits: u64,
    /// 纹理缓存未命中数
    pub texture_misses: u64,
    /// 数据缓存命中数
    pub data_hits: u64,
    /// 数据缓存未命中数
    pub data_misses: u64,
    /// 总内存使用
    pub total_memory: usize,
    /// 清理次数
    pub cleanup_count: u64,
}

impl GICache {
    /// 创建新的缓存
    pub fn new(device: Arc<RenderDevice>, max_memory: usize) -> Result<Self, String> {
        let config = CacheConfig {
            max_memory,
            ..Default::default()
        };

        let texture_max = (max_memory as f32 * config.texture_ratio) as usize;
        let data_max = (max_memory as f32 * config.data_ratio) as usize;

        let texture_cache = Arc::new(Mutex::new(TextureCache {
            entries: HashMap::new(),
            total_memory: 0,
            max_memory: texture_max,
            access_order: Vec::new(),
        }));

        let data_cache = Arc::new(Mutex::new(DataCache {
            entries: HashMap::new(),
            total_memory: 0,
            max_memory: data_max,
            access_order: Vec::new(),
        }));

        Ok(Self {
            device,
            config,
            texture_cache,
            data_cache,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        })
    }

    /// 更新缓存
    pub fn update(&self) {
        if self.config.auto_cleanup {
            // 检查是否需要清理
            let texture_cache = self.texture_cache.lock().unwrap();
            let data_cache = self.data_cache.lock().unwrap();

            let needs_cleanup = texture_cache.total_memory > texture_cache.max_memory * 9 / 10
                || data_cache.total_memory > data_cache.max_memory * 9 / 10;

            drop(texture_cache);
            drop(data_cache);

            if needs_cleanup {
                self.cleanup();
            }
        }
    }

    /// 获取纹理
    pub fn get_texture(&self, key: &str) -> Option<wgpu::Texture> {
        let mut cache = self.texture_cache.lock().unwrap();

        if let Some(entry) = cache.entries.get_mut(key) {
            entry.last_access = Instant::now();
            entry.access_count += 1;

            // 更新访问顺序
            if self.config.lru_enabled {
                cache.access_order.retain(|k| k != key);
                cache.access_order.push(key.to_string());
            }

            // 更新统计
            let mut stats = self.stats.lock().unwrap();
            stats.texture_hits += 1;

            Some(entry.texture.clone())
        } else {
            // 更新统计
            let mut stats = self.stats.lock().unwrap();
            stats.texture_misses += 1;

            None
        }
    }

    /// 插入纹理
    pub fn insert_texture(&self, key: String, texture: wgpu::Texture, size: usize) {
        let mut cache = self.texture_cache.lock().unwrap();

        // 检查是否需要腾出空间
        if cache.total_memory + size > cache.max_memory {
            // 释放最少使用的纹理
            Self::evict_textures(&mut cache, size);
        }

        let entry = TextureCacheEntry {
            texture,
            size,
            last_access: Instant::now(),
            access_count: 1,
        };

        cache.entries.insert(key.clone(), entry);
        cache.total_memory += size;

        if self.config.lru_enabled {
            cache.access_order.push(key);
        }
    }

    /// 获取数据
    pub fn get_data(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.data_cache.lock().unwrap();

        if let Some(entry) = cache.entries.get_mut(key) {
            entry.last_access = Instant::now();
            entry.access_count += 1;

            // 更新访问顺序
            if self.config.lru_enabled {
                cache.access_order.retain(|k| k != key);
                cache.access_order.push(key.to_string());
            }

            // 更新统计
            let mut stats = self.stats.lock().unwrap();
            stats.data_hits += 1;

            Some(entry.data.clone())
        } else {
            // 更新统计
            let mut stats = self.stats.lock().unwrap();
            stats.data_misses += 1;

            None
        }
    }

    /// 插入数据
    pub fn insert_data(&self, key: String, data: Vec<u8>) {
        let size = data.len();
        let mut cache = self.data_cache.lock().unwrap();

        // 检查是否需要腾出空间
        if cache.total_memory + size > cache.max_memory {
            // 释放最少使用的数据
            Self::evict_data(&mut cache, size);
        }

        let entry = DataCacheEntry {
            data,
            size,
            last_access: Instant::now(),
            access_count: 1,
        };

        cache.entries.insert(key.clone(), entry);
        cache.total_memory += size;

        if self.config.lru_enabled {
            cache.access_order.push(key);
        }
    }

    /// 清理缓存
    pub fn cleanup(&self) {
        let mut texture_cache = self.texture_cache.lock().unwrap();
        let mut data_cache = self.data_cache.lock().unwrap();

        // 清理纹理缓存
        while texture_cache.total_memory > texture_cache.max_memory * 8 / 10 {
            if let Some(key) = texture_cache.access_order.first() {
                let key = key.clone();
                if let Some(entry) = texture_cache.entries.remove(&key) {
                    texture_cache.total_memory -= entry.size;
                }
                texture_cache.access_order.remove(0);
            } else {
                break;
            }
        }

        // 清理数据缓存
        while data_cache.total_memory > data_cache.max_memory * 8 / 10 {
            if let Some(key) = data_cache.access_order.first() {
                let key = key.clone();
                if let Some(entry) = data_cache.entries.remove(&key) {
                    data_cache.total_memory -= entry.size;
                }
                data_cache.access_order.remove(0);
            } else {
                break;
            }
        }

        // 更新统计
        let mut stats = self.stats.lock().unwrap();
        stats.cleanup_count += 1;
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut texture_cache = self.texture_cache.lock().unwrap();
        let mut data_cache = self.data_cache.lock().unwrap();

        texture_cache.entries.clear();
        texture_cache.access_order.clear();
        texture_cache.total_memory = 0;

        data_cache.entries.clear();
        data_cache.access_order.clear();
        data_cache.total_memory = 0;
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f32 {
        let stats = self.stats.lock().unwrap();
        let total_requests = stats.texture_hits + stats.texture_misses
            + stats.data_hits + stats.data_misses;

        if total_requests == 0 {
            return 0.0;
        }

        let total_hits = stats.texture_hits + stats.data_hits;
        total_hits as f32 / total_requests as f32
    }

    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.lock().unwrap();
        CacheStats {
            total_memory: {
                let texture_cache = self.texture_cache.lock().unwrap();
                let data_cache = self.data_cache.lock().unwrap();
                texture_cache.total_memory + data_cache.total_memory
            },
            ..stats.clone()
        }
    }

    /// 释放纹理
    fn evict_textures(cache: &mut TextureCache, required_space: usize) {
        let mut freed_space = 0;

        while freed_space < required_space {
            if let Some(key) = cache.access_order.first() {
                let key = key.clone();
                if let Some(entry) = cache.entries.remove(&key) {
                    freed_space += entry.size;
                    cache.total_memory -= entry.size;
                }
                cache.access_order.remove(0);
            } else {
                break;
            }
        }
    }

    /// 释放数据
    fn evict_data(cache: &mut DataCache, required_space: usize) {
        let mut freed_space = 0;

        while freed_space < required_space {
            if let Some(key) = cache.access_order.first() {
                let key = key.clone();
                if let Some(entry) = cache.entries.remove(&key) {
                    freed_space += entry.size;
                    cache.total_memory -= entry.size;
                }
                cache.access_order.remove(0);
            } else {
                break;
            }
        }
    }
}
