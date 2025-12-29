//! # Render CQRS Performance Benchmarking Tests
//!
//! This module contains performance tests for the render CQRS implementation.

use super::cqrs::*;
use crate::render::domain_objects::RenderObjectId;
use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Performance metrics for render operations
#[derive(Debug, Clone)]
pub struct RenderPerformanceMetrics {
    pub operation_name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time_per_operation: Duration,
    pub operations_per_second: f64,
}

impl RenderPerformanceMetrics {
    pub fn new(operation_name: String, iterations: usize, total_time: Duration) -> Self {
        let avg_time = total_time / iterations as u32;
        let ops_per_sec = if total_time.as_secs_f64() > 0.0 {
            iterations as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };

        Self {
            operation_name,
            iterations,
            total_time,
            avg_time_per_operation: avg_time,
            operations_per_second: ops_per_sec,
        }
    }

    pub fn print(&self) {
        println!(
            "=== {} ===",
            self.operation_name
        );
        println!(
            "  Iterations: {}",
            self.iterations
        );
        println!(
            "  Total time: {:?}",
            self.total_time
        );
        println!(
            "  Avg time/op: {:?}",
            self.avg_time_per_operation
        );
        println!(
            "  Ops/sec: {:.2}",
            self.operations_per_second
        );
    }
}

/// Render benchmark suite
pub struct RenderCqrsBenchmarkSuite {
    world: World,
    cqrs_manager: Arc<crate::domain::cqrs::CqrsManager>,
    query_model: Arc<RwLock<RenderQueryModel>>,
}

impl RenderCqrsBenchmarkSuite {
    pub fn new() -> Self {
        let world = World::new();
        let cqrs_manager = Arc::new(crate::domain::cqrs::CqrsManager::new());
        let query_model = Arc::new(RwLock::new(RenderQueryModel {
            object_ids: Vec::new(),
            world_transforms: Vec::new(),
            positions: Vec::new(),
            visible: Vec::new(),
            is_static: Vec::new(),
            lod_levels: Vec::new(),
            bounding_centers: Vec::new(),
            bounding_radii: Vec::new(),
        }));

        Self {
            world,
            cqrs_manager,
            query_model,
        }
    }

    /// Setup test scenario with mock render objects
    pub fn setup_scenario(&mut self, object_count: usize) {
        let mut model = self.query_model.write().expect("Test: operation should succeed");

        model.object_ids = (0..object_count)
            .map(|i| RenderObjectId::new(i as u64))
            .collect();

        model.world_transforms = (0..object_count)
            .map(|i| {
                let x = (i as f32 % 100.0) * 2.0;
                let y = (i as f32 / 100.0).floor() * 2.0;
                Mat4::from_translation(Vec3::new(x, y, 0.0))
            })
            .collect();

        model.positions = (0..object_count)
            .map(|i| {
                let x = (i as f32 % 100.0) * 2.0;
                let y = (i as f32 / 100.0).floor() * 2.0;
                Vec3::new(x, y, 0.0)
            })
            .collect();

        model.visible = (0..object_count).map(|_| true).collect();
        model.is_static = (0..object_count).map(|i| i % 2 == 0).collect();
        model.lod_levels = vec![0; object_count];
        model.bounding_centers = model.positions.clone();
        model.bounding_radii = vec![1.0; object_count];
    }

    /// Benchmark: Get visible objects
    pub fn benchmark_get_visible_objects(&self, iterations: usize) -> RenderPerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = query_model.query_visible_objects();
        }

        let total_time = start.elapsed();
        RenderPerformanceMetrics::new("GetVisibleObjects".to_string(), iterations, total_time)
    }

    /// Benchmark: Get static objects
    pub fn benchmark_get_static_objects(&self, iterations: usize) -> RenderPerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = query_model.query_static_objects();
        }

        let total_time = start.elapsed();
        RenderPerformanceMetrics::new("GetStaticObjects".to_string(), iterations, total_time)
    }

    /// Benchmark: Query in radius
    pub fn benchmark_query_in_radius(&self, iterations: usize) -> RenderPerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for i in 0..iterations {
            let center = Vec3::new(i as f32 % 50.0, 0.0, 0.0);
            let _ = query_model.query_in_radius(center, 10.0);
        }

        let total_time = start.elapsed();
        RenderPerformanceMetrics::new("QueryInRadius".to_string(), iterations, total_time)
    }

    /// Benchmark: Batch get transforms
    pub fn benchmark_batch_get_transforms(
        &self,
        batch_size: usize,
        iterations: usize,
    ) -> RenderPerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");

        let ids: Vec<RenderObjectId> =
            (0..batch_size).map(|i| RenderObjectId::new(i as u64)).collect();

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = query_model.batch_get_transforms(&ids);
        }

        let total_time = start.elapsed();
        RenderPerformanceMetrics::new(
            format!("BatchGetTransforms (batch_size={})", batch_size),
            iterations * batch_size,
            total_time,
        )
    }

    /// Benchmark: Get world transform (single)
    pub fn benchmark_get_world_transform(&self, iterations: usize) -> RenderPerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for i in 0..iterations {
            let id = RenderObjectId::new((i % 1000) as u64);
            let _ = query_model.get_world_transform(id);
        }

        let total_time = start.elapsed();
        RenderPerformanceMetrics::new("GetWorldTransform".to_string(), iterations, total_time)
    }

    /// Run full benchmark suite
    pub fn run_full_benchmark_suite(&mut self, object_count: usize) -> RenderBenchmarkReport {
        println!("Setting up render benchmark with {} objects...", object_count);
        self.setup_scenario(object_count);
        println!("Setup complete.\n");

        let mut report = RenderBenchmarkReport::new(object_count);

        // Benchmark 1: Get visible objects
        println!("Benchmarking get visible objects...");
        let visible = self.benchmark_get_visible_objects(1000);
        visible.print();
        println!();

        report.add_metric("GetVisibleObjects".to_string(), visible);

        // Benchmark 2: Get static objects
        println!("Benchmarking get static objects...");
        let static_objs = self.benchmark_get_static_objects(1000);
        static_objs.print();
        println!();

        report.add_metric("GetStaticObjects".to_string(), static_objs);

        // Benchmark 3: Query in radius
        println!("Benchmarking query in radius...");
        let radius = self.benchmark_query_in_radius(1000);
        radius.print();
        println!();

        report.add_metric("QueryInRadius".to_string(), radius);

        // Benchmark 4: Batch get transforms
        println!("Benchmarking batch get transforms...");
        let batch = self.benchmark_batch_get_transforms(100, 100);
        batch.print();
        println!();

        report.add_metric("BatchGetTransforms".to_string(), batch);

        // Benchmark 5: Single transform lookup
        println!("Benchmarking single transform lookup...");
        let single = self.benchmark_get_world_transform(10000);
        single.print();
        println!();

        report.add_metric("GetWorldTransform".to_string(), single);

        report
    }
}

/// Render benchmark report
#[derive(Debug, Clone)]
pub struct RenderBenchmarkReport {
    pub object_count: usize,
    pub metrics: Vec<RenderBenchmarkMetric>,
}

#[derive(Debug, Clone)]
pub struct RenderBenchmarkMetric {
    pub operation_name: String,
    pub metrics: RenderPerformanceMetrics,
}

impl RenderBenchmarkReport {
    pub fn new(object_count: usize) -> Self {
        Self {
            object_count,
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, operation_name: String, metrics: RenderPerformanceMetrics) {
        self.metrics.push(RenderBenchmarkMetric {
            operation_name,
            metrics,
        });
    }

    pub fn print_summary(&self) {
        println!("\n");
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║        Render CQRS Performance Benchmark Report                ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("Test Configuration:");
        println!("  Object count: {}", self.object_count);
        println!();

        println!("Performance Metrics:");
        println!("┌────────────────────────────────────────────────────────────────┐");
        println!("│ {:<50} │", "Operation");
        println!("├────────────────────────────────────────────────────────────────┤");
        println!("│ {:<50} │", "Name");
        println!("│ {:<20} │ {:>15} │ {:>15} │", "", "Time (ns)", "Ops/sec");
        println!("├────────────────────────────────────────────────────────────────┤");

        for metric in &self.metrics {
            println!("│ {:<50} │", metric.operation_name);
            println!("│ {:<20} │ {:>15.0} │ {:>15.0} │",
                "",
                metric.metrics.avg_time_per_operation.as_nanos(),
                metric.metrics.operations_per_second
            );
            println!("├────────────────────────────────────────────────────────────────┤");
        }

        println!("└────────────────────────────────────────────────────────────────┘");
        println!();

        // Calculate average ops/sec
        let avg_ops_per_sec: f64 = self.metrics.iter()
            .map(|m| m.metrics.operations_per_second)
            .sum::<f64>() / self.metrics.len() as f64;

        println!("Average Operations Per Second: {:.2}", avg_ops_per_sec);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Expensive benchmark test - run manually
    fn test_render_full_benchmark_suite() {
        let mut suite = RenderCqrsBenchmarkSuite::new();
        let report = suite.run_full_benchmark_suite(1000);
        report.print_summary();

        // Verify all operations completed successfully
        assert!(!report.metrics.is_empty());
    }

    #[test]
    fn test_render_query_model_basic() {
        let suite = RenderCqrsBenchmarkSuite::new();

        // Setup small test scenario
        let model = RenderQueryModel {
            object_ids: vec![RenderObjectId::new(1), RenderObjectId::new(2)],
            world_transforms: vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::X)],
            positions: vec![Vec3::ZERO, Vec3::X],
            visible: vec![true, false],
            is_static: vec![true, false],
            lod_levels: vec![0, 0],
            bounding_centers: vec![Vec3::ZERO, Vec3::X],
            bounding_radii: vec![1.0, 1.0],
        };

        // Test queries
        assert_eq!(model.object_count(), 2);
        assert_eq!(model.visible_count(), 1);
        assert_eq!(model.static_count(), 1);

        let visible = model.query_visible_objects();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0], RenderObjectId::new(1));

        let static_objs = model.query_static_objects();
        assert_eq!(static_objs.len(), 1);
        assert_eq!(static_objs[0], RenderObjectId::new(1));
    }

    #[test]
    fn test_render_query_in_radius() {
        let model = RenderQueryModel {
            object_ids: vec![
                RenderObjectId::new(1),
                RenderObjectId::new(2),
                RenderObjectId::new(3),
            ],
            world_transforms: vec![Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY],
            positions: vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0)],
            visible: vec![true, true, true],
            is_static: vec![true, false, true],
            lod_levels: vec![0, 0, 0],
            bounding_centers: vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0)],
            bounding_radii: vec![1.0, 1.0, 1.0],
        };

        let in_radius = model.query_in_radius(Vec3::ZERO, 10.0);
        assert_eq!(in_radius.len(), 2);
        assert!(in_radius.contains(&RenderObjectId::new(1)));
        assert!(in_radius.contains(&RenderObjectId::new(2)));
    }

    #[test]
    fn test_render_batch_operations() {
        let model = RenderQueryModel {
            object_ids: vec![RenderObjectId::new(1), RenderObjectId::new(2)],
            world_transforms: vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::X)],
            positions: vec![Vec3::ZERO, Vec3::X],
            visible: vec![true, true],
            is_static: vec![true, false],
            lod_levels: vec![0, 0],
            bounding_centers: vec![Vec3::ZERO, Vec3::X],
            bounding_radii: vec![1.0, 1.0],
        };

        let ids = vec![RenderObjectId::new(1), RenderObjectId::new(2)];
        let transforms = model.batch_get_transforms(&ids);

        assert_eq!(transforms.len(), 2);
        assert_eq!(transforms[0], Some(Mat4::IDENTITY));
        assert_eq!(transforms[1], Some(Mat4::from_translation(Vec3::X)));
    }
}
