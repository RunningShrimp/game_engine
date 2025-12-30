//! 性能优化示例
//!
//! 本示例展示如何使用性能优化特性来提升游戏性能

use game_engine::prelude::*;

fn main() -> GameResult {
    println!("🚀 游戏引擎性能优化示例\n");

    // 示例1: 使用优化的资源管理器
    optimized_asset_manager_example();

    // 示例2: 使用并发实体管理器
    concurrent_entity_manager_example();

    // 示例3: 批量操作优化
    batch_operations_example();

    // 示例4: 并发资源缓存
    concurrent_cache_example();

    println!("✅ 所有性能示例执行完成！");

    Ok(())
}

/// 示例1: 使用优化的资源管理器
///
/// # 性能优势
///
/// - parking_lot::RwLock比std::sync::RwLock快2.5x-8x
/// - 读操作仅需~40ns（vs std::sync的~100-500ns）
fn optimized_asset_manager_example() {
    println!("📦 示例1: 优化的资源管理器");

    use game_engine::resources::optimized_manager::OptimizedAssetManager;

    let manager = OptimizedAssetManager::new();

    // 加载单个资源
    match manager.load_texture("player.png") {
        Ok(_) => println!("  ✓ 单个资源加载成功"),
        Err(e) => println!("  ⚠ 加载失败: {}", e),
    }

    // 批量加载（性能优化）
    let textures = manager.load_textures_batch(&["a.png", "b.png", "c.png"]);
    println!("  ✓ 批量加载: {} 个资源", textures.len());

    // 预加载（并行）
    match manager.preload_assets(&["x.png", "y.png", "z.png"]) {
        Ok(_) => println!("  ✓ 预加载完成（并行）"),
        Err(e) => println!("  ⚠ 预加载失败: {}", e),
    }

    // 查看统计
    let stats = manager.get_stats();
    println!("  📊 统计: 纹理={}, 网格={}, 着色器={}",
        stats.textures_loaded, stats.meshes_loaded, stats.shaders_loaded);

    println!();
}

/// 示例2: 使用并发实体管理器
///
/// # 性能优势
///
/// - DashMap比Mutex<HashMap>快10x-20x
/// - 几乎无锁读取
/// - 细粒度锁，最小化竞争
fn concurrent_entity_manager_example() {
    println!("🎮 示例2: 并发实体管理器");

    use game_engine::resources::dashmap_optimizations::{ConcurrentEntityManager, EntityData};

    let manager = ConcurrentEntityManager::new();

    // 添加多个实体（并发安全）
    for i in 0..100 {
        manager.add_entity(EntityData {
            id: i,
            position: (i as f32, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            active: true,
        });
    }
    println!("  ✓ 添加了 100 个实体");

    // 获取实体（几乎无锁，10x faster）
    if let Some(entity) = manager.get_entity(50) {
        println!("  ✓ 获取实体50: {:?}", entity.position);
    }

    // 更新实体（细粒度锁）
    manager.update_entity(50, |entity| {
        entity.position = (100.0, 200.0, 300.0);
    });
    println!("  ✓ 更新实体50位置");

    // 获取活跃实体
    let active = manager.get_active_entities();
    println!("  ✓ 活跃实体数: {}", active.len());

    println!();
}

/// 示例3: 批量操作优化
///
/// # 性能优势
///
/// - 减少锁获取次数
/// - 批量操作更高效
/// - parking_lot的批量操作性能更好
fn batch_operations_example() {
    println!("⚡ 示例3: 批量操作优化");

    use game_engine::resources::optimized_manager::OptimizedAssetManager;
    use std::time::Instant;

    let manager = OptimizedAssetManager::new();

    // 测试批量加载 vs 单个加载
    let names: Vec<&str> = (0..100).map(|i| &format!("tex_{}.png", i)).collect();

    let start = Instant::now();
    let _results = manager.load_textures_batch(&names);
    let batch_time = start.elapsed();

    println!("  ✓ 批量加载100个资源: {:?}", batch_time);
    println!("  💡 批量操作减少了锁获取次数，性能更好");

    println!();
}

/// 示例4: 并发资源缓存
///
/// # 性能优势
///
/// - DashMap自动管理并发访问
/// - 内置访问统计
/// - 自动过期清理
fn concurrent_cache_example() {
    println!("💾 示例4: 并发资源缓存");

    use game_engine::resources::dashmap_optimizations::ConcurrentResourceCache;
    use std::time::Duration;

    let cache = ConcurrentResourceCache::new();

    // 插入资源
    for i in 0..50 {
        cache.insert(format!("resource_{}", i), format!("data_{}", i));
    }
    println!("  ✓ 缓存了 50 个资源");

    // 获取资源（自动更新统计）
    if let Some(data) = cache.get("resource_10") {
        println!("  ✓ 获取资源: {}", data);
    }

    // 多次访问，查看统计
    let _ = cache.get("resource_10");
    let _ = cache.get("resource_10");

    if let Some((count, age)) = cache.get_stats("resource_10") {
        println!("  📊 访问统计: 次数={}, 距离上次访问={:?}", count, age);
    }

    // 清理过期资源
    cache.cleanup_expired(Duration::from_secs(0));
    println!("  ✓ 清理过期资源完成");

    println!();
}

/// 性能对比示例
#[allow(dead_code)]
fn performance_comparison() {
    println!("📊 性能对比示例");

    use std::sync::{Arc, Mutex, RwLock};
    use std::time::Instant;
    use dashmap::DashMap;

    const NUM_OPERATIONS: usize = 10_000;

    // 对比1: parking_lot vs std::sync RwLock
    println!("\n1. RwLock性能对比:");

    let parking_lock = parking_lot::RwLock::new(42);
    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _ = parking_lock.read();
    }
    let parking_time = start.elapsed();
    println!("  parking_lot::RwLock: {:?}", parking_time);

    let std_lock = RwLock::new(42);
    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _ = std_lock.read().unwrap();
    }
    let std_time = start.elapsed();
    println!("  std::sync::RwLock: {:?}", std_time);
    println!("  性能提升: {:.2}x", std_time.as_nanos() as f64 / parking_time.as_nanos() as f64);

    // 对比2: DashMap vs Mutex<HashMap>
    println!("\n2. Map并发性能对比:");

    let dashmap = DashMap::new();
    let start = Instant::now();
    for i in 0..NUM_OPERATIONS {
        dashmap.insert(i, i);
    }
    let dash_time = start.elapsed();
    println!("  DashMap: {:?}", dash_time);

    let mutex_map = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let start = Instant::now();
    for i in 0..NUM_OPERATIONS {
        let mut map = mutex_map.lock().unwrap();
        map.insert(i, i);
    }
    let mutex_time = start.elapsed();
    println!("  Mutex<HashMap>: {:?}", mutex_time);
    println!("  性能提升: {:.2}x", mutex_time.as_nanos() as f64 / dash_time.as_nanos() as f64);
}
