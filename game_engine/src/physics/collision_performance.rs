//! 碰撞检测性能监控
//!
//! 提供碰撞检测性能统计和监控功能。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 碰撞检测性能统计
#[derive(Debug, Clone)]
pub struct CollisionPerformanceStats {
    /// 总碰撞检测次数
    pub total_collision_checks: u64,
    /// 实际碰撞次数
    pub actual_collisions: u64,
    /// 平均检测时间（微秒）
    pub avg_check_time_us: f64,
    /// 最大检测时间（微秒）
    pub max_check_time_us: f64,
    /// 最小检测时间（微秒）
    pub min_check_time_us: f64,
    /// 空间分区查询次数
    pub spatial_query_count: u64,
    /// 空间分区查询命中率（0.0-1.0）
    pub spatial_query_hit_rate: f64,
}

impl Default for CollisionPerformanceStats {
    fn default() -> Self {
        Self {
            total_collision_checks: 0,
            actual_collisions: 0,
            avg_check_time_us: 0.0,
            max_check_time_us: 0.0,
            min_check_time_us: f64::INFINITY,
            spatial_query_count: 0,
            spatial_query_hit_rate: 0.0,
        }
    }
}

/// 碰撞检测性能监控器
#[derive(Debug)]
pub struct CollisionPerformanceMonitor {
    /// 总碰撞检测次数
    total_checks: Arc<AtomicU64>,
    /// 实际碰撞次数
    actual_collisions: Arc<AtomicU64>,
    /// 检测时间总和（微秒）
    total_time_us: Arc<AtomicU64>,
    /// 最大检测时间（微秒）
    max_time_us: Arc<AtomicU64>,
    /// 最小检测时间（微秒）
    min_time_us: Arc<AtomicU64>,
    /// 空间分区查询次数
    spatial_queries: Arc<AtomicU64>,
    /// 空间分区查询命中次数
    spatial_hits: Arc<AtomicU64>,
}

impl CollisionPerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            total_checks: Arc::new(AtomicU64::new(0)),
            actual_collisions: Arc::new(AtomicU64::new(0)),
            total_time_us: Arc::new(AtomicU64::new(0)),
            max_time_us: Arc::new(AtomicU64::new(0)),
            min_time_us: Arc::new(AtomicU64::new(u64::MAX)),
            spatial_queries: Arc::new(AtomicU64::new(0)),
            spatial_hits: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 记录碰撞检测
    pub fn record_collision_check(&self, duration: std::time::Duration, is_collision: bool) {
        let time_us = duration.as_micros() as u64;
        
        self.total_checks.fetch_add(1, Ordering::Relaxed);
        if is_collision {
            self.actual_collisions.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_time_us.fetch_add(time_us, Ordering::Relaxed);
        
        // 更新最大时间
        loop {
            let current_max = self.max_time_us.load(Ordering::Relaxed);
            if time_us <= current_max {
                break;
            }
            if self.max_time_us.compare_exchange(
                current_max,
                time_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
        
        // 更新最小时间
        loop {
            let current_min = self.min_time_us.load(Ordering::Relaxed);
            if time_us >= current_min {
                break;
            }
            if self.min_time_us.compare_exchange(
                current_min,
                time_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }

    /// 记录空间分区查询
    pub fn record_spatial_query(&self, hit: bool) {
        self.spatial_queries.fetch_add(1, Ordering::Relaxed);
        if hit {
            self.spatial_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> CollisionPerformanceStats {
        let total_checks = self.total_checks.load(Ordering::Relaxed);
        let actual_collisions = self.actual_collisions.load(Ordering::Relaxed);
        let total_time_us = self.total_time_us.load(Ordering::Relaxed);
        let max_time_us = self.max_time_us.load(Ordering::Relaxed);
        let min_time_us = self.min_time_us.load(Ordering::Relaxed);
        let spatial_queries = self.spatial_queries.load(Ordering::Relaxed);
        let spatial_hits = self.spatial_hits.load(Ordering::Relaxed);

        let avg_time_us = if total_checks > 0 {
            total_time_us as f64 / total_checks as f64
        } else {
            0.0
        };

        let hit_rate = if spatial_queries > 0 {
            spatial_hits as f64 / spatial_queries as f64
        } else {
            0.0
        };

        CollisionPerformanceStats {
            total_collision_checks: total_checks,
            actual_collisions,
            avg_check_time_us: avg_time_us,
            max_check_time_us: max_time_us as f64,
            min_check_time_us: if min_time_us == u64::MAX { 0.0 } else { min_time_us as f64 },
            spatial_query_count: spatial_queries,
            spatial_query_hit_rate: hit_rate,
        }
    }

    /// 重置统计
    pub fn reset(&self) {
        self.total_checks.store(0, Ordering::Relaxed);
        self.actual_collisions.store(0, Ordering::Relaxed);
        self.total_time_us.store(0, Ordering::Relaxed);
        self.max_time_us.store(0, Ordering::Relaxed);
        self.min_time_us.store(u64::MAX, Ordering::Relaxed);
        self.spatial_queries.store(0, Ordering::Relaxed);
        self.spatial_hits.store(0, Ordering::Relaxed);
    }
}

impl Default for CollisionPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 碰撞检测性能分析器（用于单次检测）
pub struct CollisionProfiler {
    start: Instant,
    monitor: Arc<CollisionPerformanceMonitor>,
}

impl CollisionProfiler {
    /// 开始性能分析
    pub fn start(monitor: Arc<CollisionPerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            monitor,
        }
    }

    /// 结束性能分析并记录
    pub fn finish(self, is_collision: bool) {
        let duration = self.start.elapsed();
        self.monitor.record_collision_check(duration, is_collision);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor() {
        let monitor = Arc::new(CollisionPerformanceMonitor::new());
        
        // 记录几次检测
        monitor.record_collision_check(Duration::from_micros(100), false);
        monitor.record_collision_check(Duration::from_micros(200), true);
        monitor.record_collision_check(Duration::from_micros(150), false);

        let stats = monitor.get_stats();
        assert_eq!(stats.total_collision_checks, 3);
        assert_eq!(stats.actual_collisions, 1);
        assert!(stats.avg_check_time_us > 0.0);
    }

    #[test]
    fn test_spatial_query_tracking() {
        let monitor = Arc::new(CollisionPerformanceMonitor::new());
        
        monitor.record_spatial_query(true);
        monitor.record_spatial_query(false);
        monitor.record_spatial_query(true);

        let stats = monitor.get_stats();
        assert_eq!(stats.spatial_query_count, 3);
        assert!((stats.spatial_query_hit_rate - 2.0 / 3.0).abs() < 0.001);
    }
}

