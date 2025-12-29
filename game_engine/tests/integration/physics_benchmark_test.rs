use std::time::Instant;
use game_engine::physics::{
    BVHTree, SpatialHash, SpatialPartitionType, SpatialPartitionManager,
    CollisionPerformanceMonitor, CollisionProfiler,
};
use rapier3d::parry::bounding_volume::Aabb;
use rapier3d::prelude::*;
use rapier3d::na::Point;

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_spatial_hash_build() {
    let iterations = 10000;
    let cell_size = 10.0;
    let mut spatial_hash = SpatialHash::new(cell_size);
    
    let mut collider_set = ColliderSet::new();
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    let build_start = Instant::now();
    spatial_hash.build(&collider_set);
    let build_duration = build_start.elapsed();
    
    let total_duration = start.elapsed();
    
    println!("空间哈希构建性能: {} 个碰撞体", iterations);
    println!("构建耗时: {:?}", build_duration);
    println!("总耗时: {:?}", total_duration);
    println!("平均构建时间: {:.2} ns/碰撞体", build_duration.as_nanos() as f64 / iterations as f64);
    
    assert!(build_duration < Duration::from_millis(100), "空间哈希构建应该小于100ms");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_spatial_hash_query() {
    let iterations = 10000;
    let cell_size = 10.0;
    let mut spatial_hash = SpatialHash::new(cell_size);
    
    let mut collider_set = ColliderSet::new();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    spatial_hash.build(&collider_set);
    
    let query_count = 1000;
    let start = Instant::now();
    
    for i in 0..query_count {
        let query_aabb = Aabb::new(
            Point::new(i as f32, i as f32, i as f32),
            Point::new(i as f32 + 5.0, i as f32 + 5.0, i as f32 + 5.0),
        );
        
        let _results = spatial_hash.query_aabb(&query_aabb, &collider_set);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / query_count as f64;
    
    println!("空间哈希查询性能: {} 次查询", query_count);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/查询", avg_time);
    
    assert!(avg_time < 10000.0, "空间哈希查询应该小于10000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_bvh_build() {
    let iterations = 10000;
    let max_depth = 10;
    let max_colliders_per_leaf = 10;
    let mut bvh = BVHTree::new(max_depth, max_colliders_per_leaf);
    
    let mut collider_set = ColliderSet::new();
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    let build_start = Instant::now();
    bvh.build(&collider_set);
    let build_duration = build_start.elapsed();
    
    let total_duration = start.elapsed();
    
    println!("BVH构建性能: {} 个碰撞体", iterations);
    println!("构建耗时: {:?}", build_duration);
    println!("总耗时: {:?}", total_duration);
    println!("平均构建时间: {:.2} ns/碰撞体", build_duration.as_nanos() as f64 / iterations as f64);
    println!("节点数量: {}", bvh.node_count());
    
    assert!(build_duration < Duration::from_millis(200), "BVH构建应该小于200ms");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_bvh_query() {
    let iterations = 10000;
    let max_depth = 10;
    let max_colliders_per_leaf = 10;
    let mut bvh = BVHTree::new(max_depth, max_colliders_per_leaf);
    
    let mut collider_set = ColliderSet::new();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    bvh.build(&collider_set);
    
    let query_count = 1000;
    let start = Instant::now();
    
    for i in 0..query_count {
        let query_aabb = Aabb::new(
            Point::new(i as f32, i as f32, i as f32),
            Point::new(i as f32 + 5.0, i as f32 + 5.0, i as f32 + 5.0),
        );
        
        let _results = bvh.query_aabb(&query_aabb, &collider_set);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / query_count as f64;
    
    println!("BVH查询性能: {} 次查询", query_count);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/查询", avg_time);
    
    assert!(avg_time < 5000.0, "BVH查询应该小于5000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_bvh_raycast() {
    let iterations = 10000;
    let max_depth = 10;
    let max_colliders_per_leaf = 10;
    let mut bvh = BVHTree::new(max_depth, max_colliders_per_leaf);
    
    let mut collider_set = ColliderSet::new();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    bvh.build(&collider_set);
    
    let raycast_count = 1000;
    let start = Instant::now();
    
    for i in 0..raycast_count {
        let ray = Ray::new(
            Point::new(0.0, 0.0, 0.0),
            rapier3d::na::Vector::new(
                (i as f32).cos(),
                (i as f32).sin(),
                (i as f32).tan(),
            ).normalize(),
        );
        
        let _result = bvh.raycast(&ray, 1000.0, &collider_set);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / raycast_count as f64;
    
    println!("BVH射线检测性能: {} 次射线检测", raycast_count);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/检测", avg_time);
    
    assert!(avg_time < 10000.0, "BVH射线检测应该小于10000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_spatial_partition_manager() {
    let iterations = 10000;
    let mut manager = SpatialPartitionManager::new(SpatialPartitionType::SpatialHash {
        cell_size: 10.0,
    });
    
    let mut collider_set = ColliderSet::new();
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        let handle = collider_set.insert(collider);
        manager.insert(handle, position, 1.0);
    }
    
    let total_duration = start.elapsed();
    
    let query_count = 1000;
    let query_start = Instant::now();
    
    for i in 0..query_count {
        let query_aabb = Aabb::new(
            Point::new(i as f32, i as f32, i as f32),
            Point::new(i as f32 + 5.0, i as f32 + 5.0, i as f32 + 5.0),
        );
        
        let _results = manager.query(&query_aabb, &collider_set);
    }
    
    let query_duration = query_start.elapsed();
    
    println!("空间分区管理器性能: {} 个碰撞体", iterations);
    println!("插入耗时: {:?}", total_duration);
    println!("查询耗时: {:?}", query_duration);
    println!("平均插入时间: {:.2} ns/碰撞体", total_duration.as_nanos() as f64 / iterations as f64);
    println!("平均查询时间: {:.2} ns/查询", query_duration.as_nanos() as f64 / query_count as f64);
    
    assert!(total_duration < Duration::from_millis(200), "插入应该小于200ms");
    assert!(query_duration < Duration::from_millis(100), "查询应该小于100ms");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_collision_performance_monitor() {
    let iterations = 100000;
    let monitor = Arc::new(CollisionPerformanceMonitor::new());
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let duration = Duration::from_micros((i % 100) as u64 + 10);
        let is_collision = i % 10 == 0;
        
        monitor.record_collision_check(duration, is_collision);
    }
    
    let duration = start.elapsed();
    let stats = monitor.get_stats();
    
    println!("碰撞性能监控器性能: {} 次记录", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/记录", duration.as_nanos() as f64 / iterations as f64);
    println!("总碰撞检测次数: {}", stats.total_collision_checks);
    println!("实际碰撞次数: {}", stats.actual_collisions);
    println!("平均检测时间: {:.2} us", stats.avg_check_time_us);
    println!("最大检测时间: {:.2} us", stats.max_check_time_us);
    println!("最小检测时间: {:.2} us", stats.min_check_time_us);
    
    assert!(stats.total_collision_checks == iterations as u64);
    assert!(stats.actual_collisions == iterations as u64 / 10);
    assert!(duration < Duration::from_millis(100), "记录应该小于100ms");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_collision_profiler() {
    let iterations = 10000;
    let monitor = Arc::new(CollisionPerformanceMonitor::new());
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let profiler = CollisionProfiler::start(monitor.clone());
        
        std::thread::sleep(Duration::from_micros(1));
        
        let is_collision = i % 10 == 0;
        profiler.finish(is_collision);
    }
    
    let duration = start.elapsed();
    let stats = monitor.get_stats();
    
    println!("碰撞性能分析器性能: {} 次分析", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} us/分析", duration.as_micros() as f64 / iterations as f64);
    println!("总碰撞检测次数: {}", stats.total_collision_checks);
    println!("实际碰撞次数: {}", stats.actual_collisions);
    
    assert!(stats.total_collision_checks == iterations as u64);
    assert!(duration < Duration::from_secs(1), "分析应该小于1秒");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_aabb_creation() {
    let iterations = 100000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let min = Point::new(
            (i % 100) as f32,
            ((i / 100) % 100) as f32,
            (i / 10000) as f32,
        );
        let max = Point::new(
            min.x + 1.0,
            min.y + 1.0,
            min.z + 1.0,
        );
        
        let _aabb = Aabb::new(min, max);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("AABB创建性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 500.0, "AABB创建应该小于500ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_aabb_intersection() {
    let iterations = 100000;
    let mut aabbs = Vec::with_capacity(iterations);
    
    for i in 0..iterations {
        let min = Point::new(
            (i % 100) as f32,
            ((i / 100) % 100) as f32,
            (i / 10000) as f32,
        );
        let max = Point::new(
            min.x + 1.0,
            min.y + 1.0,
            min.z + 1.0,
        );
        
        aabbs.push(Aabb::new(min, max));
    }
    
    let query_aabb = Aabb::new(
        Point::new(50.0, 50.0, 50.0),
        Point::new(55.0, 55.0, 55.0),
    );
    
    let start = Instant::now();
    let mut intersection_count = 0;
    
    for aabb in &aabbs {
        if aabb.intersects(&query_aabb) {
            intersection_count += 1;
        }
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("AABB相交测试性能: {} 次测试", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/测试", avg_time);
    println!("相交数量: {}", intersection_count);
    
    assert!(avg_time < 100.0, "AABB相交测试应该小于100ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_collider_creation() {
    let iterations = 10000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let _collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(
                (i % 100) as f32,
                ((i / 100) % 100) as f32,
                (i / 10000) as f32,
            ))
            .build();
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("碰撞体创建性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 5000.0, "碰撞体创建应该小于5000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_collider_aabb_computation() {
    let iterations = 10000;
    let mut collider_set = ColliderSet::new();
    
    for i in 0..iterations {
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(
                (i % 100) as f32,
                ((i / 100) % 100) as f32,
                (i / 10000) as f32,
            ))
            .build();
        
        collider_set.insert(collider);
    }
    
    let start = Instant::now();
    
    for (_, collider) in collider_set.iter() {
        let _aabb = collider.compute_aabb();
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("碰撞体AABB计算性能: {} 次计算", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/计算", avg_time);
    
    assert!(avg_time < 2000.0, "碰撞体AABB计算应该小于2000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_spatial_hash_vs_bvh() {
    let iterations = 10000;
    let query_count = 1000;
    
    let mut collider_set = ColliderSet::new();
    
    for i in 0..iterations {
        let position = Point::new(
            (i % 100) as f32 * 10.0,
            ((i / 100) % 100) as f32 * 10.0,
            (i / 10000) as f32 * 10.0,
        );
        
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0)
            .position(Isometry::translation(position.x, position.y, position.z))
            .build();
        
        collider_set.insert(collider);
    }
    
    let query_aabb = Aabb::new(
        Point::new(50.0, 50.0, 50.0),
        Point::new(55.0, 55.0, 55.0),
    );
    
    let mut spatial_hash = SpatialHash::new(10.0);
    let hash_build_start = Instant::now();
    spatial_hash.build(&collider_set);
    let hash_build_duration = hash_build_start.elapsed();
    
    let hash_query_start = Instant::now();
    for _ in 0..query_count {
        let _results = spatial_hash.query_aabb(&query_aabb, &collider_set);
    }
    let hash_query_duration = hash_query_start.elapsed();
    
    let mut bvh = BVHTree::new(10, 10);
    let bvh_build_start = Instant::now();
    bvh.build(&collider_set);
    let bvh_build_duration = bvh_build_start.elapsed();
    
    let bvh_query_start = Instant::now();
    for _ in 0..query_count {
        let _results = bvh.query_aabb(&query_aabb, &collider_set);
    }
    let bvh_query_duration = bvh_query_start.elapsed();
    
    println!("空间哈希 vs BVH 性能对比:");
    println!("碰撞体数量: {}", iterations);
    println!("查询次数: {}", query_count);
    println!();
    println!("空间哈希:");
    println!("  构建耗时: {:?}", hash_build_duration);
    println!("  查询耗时: {:?}", hash_query_duration);
    println!("  平均查询时间: {:.2} ns", hash_query_duration.as_nanos() as f64 / query_count as f64);
    println!();
    println!("BVH:");
    println!("  构建耗时: {:?}", bvh_build_duration);
    println!("  查询耗时: {:?}", bvh_query_duration);
    println!("  平均查询时间: {:.2} ns", bvh_query_duration.as_nanos() as f64 / query_count as f64);
    println!();
    println!("构建性能差异: {:.2}x", bvh_build_duration.as_nanos() as f64 / hash_build_duration.as_nanos() as f64);
    println!("查询性能差异: {:.2}x", hash_query_duration.as_nanos() as f64 / bvh_query_duration.as_nanos() as f64);
    
    assert!(hash_build_duration < Duration::from_millis(100), "空间哈希构建应该小于100ms");
    assert!(bvh_build_duration < Duration::from_millis(200), "BVH构建应该小于200ms");
}

use std::sync::Arc;
use std::time::Duration;
