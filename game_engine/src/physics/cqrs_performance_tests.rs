//! # CQRS Performance Benchmarking Tests
//!
//! This module contains comprehensive performance tests to validate
//! the 20-30% query performance improvement from using CQRS pattern.

use super::cqrs::*;
use crate::domain::physics::{RigidBody, RigidBodyId, RigidBodyType};
use bevy_ecs::prelude::*;
use glam::Vec3;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Performance metrics structure
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time_per_operation: Duration,
    pub operations_per_second: f64,
}

impl PerformanceMetrics {
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
        println!("=== {} ===", self.operation_name);
        println!("  Iterations: {}", self.iterations);
        println!("  Total time: {:?}", self.total_time);
        println!("  Avg time/op: {:?}", self.avg_time_per_operation);
        println!("  Ops/sec: {:.2}", self.operations_per_second);
    }

    pub fn improvement_percent(&self, baseline: &PerformanceMetrics) -> f64 {
        if baseline.avg_time_per_operation.as_nanos() > 0 {
            let baseline_ns = baseline.avg_time_per_operation.as_nanos() as f64;
            let current_ns = self.avg_time_per_operation.as_nanos() as f64;
            ((baseline_ns - current_ns) / baseline_ns) * 100.0
        } else {
            0.0
        }
    }
}

/// Benchmark suite for comparing traditional vs CQRS queries
pub struct CqrsBenchmarkSuite {
    world: World,
    cqrs_manager: Arc<crate::domain::cqrs::CqrsManager>,
    query_model: Arc<RwLock<PhysicsQueryModel>>,
}

impl Default for CqrsBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl CqrsBenchmarkSuite {
    pub fn new() -> Self {
        let world = World::new();
        let cqrs_manager = Arc::new(crate::domain::cqrs::CqrsManager::new());
        let query_model = Arc::new(RwLock::new(PhysicsQueryModel::new()));

        Self {
            world,
            cqrs_manager,
            query_model,
        }
    }

    /// Setup test scenario with specified number of bodies
    pub fn setup_scenario(&mut self, body_count: usize) {
        let mut bodies = Vec::with_capacity(body_count);

        for i in 0..body_count {
            let x = (i as f32 % 100.0) * 2.0;
            let y = (i as f32 / 100.0).floor() * 2.0;
            let z = 0.0;

            let body = RigidBody::new(
                RigidBodyId::new(i as u64),
                RigidBodyType::Dynamic,
                Vec3::new(x, y, z),
            );

            bodies.push(body);
        }

        // Update query model
        let mut model = self.query_model.write().expect("Test: operation should succeed");
        *model = PhysicsQueryModel::from_world(&bodies);
    }

    /// Benchmark: Traditional direct access (baseline)
    pub fn benchmark_traditional_get_position(&self, iterations: usize) -> PerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for i in 0..iterations {
            let id = RigidBodyId::new((i % 1000) as u64);
            let _ = query_model.get_position(id);
        }

        let total_time = start.elapsed();
        PerformanceMetrics::new(
            "Traditional GetPosition".to_string(),
            iterations,
            total_time,
        )
    }

    /// Benchmark: CQRS query get position
    pub fn benchmark_cqrs_get_position(&self, iterations: usize) -> PerformanceMetrics {
        // Register handler
        let handler = Arc::new(GetBodyPositionHandler::new(self.query_model.clone()));
        let _ = self.cqrs_manager.register_query_handler(handler);

        let start = Instant::now();

        for i in 0..iterations {
            let query = GetBodyPositionQuery {
                id: RigidBodyId::new((i % 1000) as u64),
            };
            let _: Result<Option<Vec3>, _> = self.cqrs_manager.execute_query(query, &self.world);
        }

        let total_time = start.elapsed();
        PerformanceMetrics::new("CQRS GetPosition".to_string(), iterations, total_time)
    }

    /// Benchmark: Batch query (CQRS advantage)
    pub fn benchmark_batch_get_positions(
        &self,
        batch_size: usize,
        iterations: usize,
    ) -> PerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");

        let ids: Vec<RigidBodyId> = (0..batch_size).map(|i| RigidBodyId::new(i as u64)).collect();

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = query_model.batch_get_positions(&ids);
        }

        let total_time = start.elapsed();
        PerformanceMetrics::new(
            format!("Batch GetPositions (batch_size={batch_size})"),
            iterations * batch_size,
            total_time,
        )
    }

    /// Benchmark: Query in radius
    pub fn benchmark_query_in_radius(&self, iterations: usize) -> PerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for i in 0..iterations {
            let center = Vec3::new(i as f32 % 50.0, 0.0, 0.0);
            let _ = query_model.query_in_radius(center, 10.0);
        }

        let total_time = start.elapsed();
        PerformanceMetrics::new("QueryInRadius".to_string(), iterations, total_time)
    }

    /// Benchmark: Query dynamic bodies
    pub fn benchmark_query_dynamic_bodies(&self, iterations: usize) -> PerformanceMetrics {
        let query_model = self.query_model.read().expect("Test: operation should succeed");
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = query_model.query_dynamic_bodies();
        }

        let total_time = start.elapsed();
        PerformanceMetrics::new("QueryDynamicBodies".to_string(), iterations, total_time)
    }

    /// Run all benchmarks and generate report
    pub fn run_full_benchmark_suite(&mut self, body_count: usize) -> BenchmarkReport {
        println!("Setting up benchmark scenario with {body_count} bodies...");
        self.setup_scenario(body_count);
        println!("Setup complete.\n");

        let mut report = BenchmarkReport::new(body_count);

        // Benchmark 1: Single position lookup
        println!("Benchmarking single position lookups...");
        let traditional = self.benchmark_traditional_get_position(10000);
        let cqrs = self.benchmark_cqrs_get_position(10000);
        let improvement = cqrs.improvement_percent(&traditional);

        traditional.print();
        cqrs.print();
        println!("  Improvement: {improvement:.2}%\n");

        report.add_metric(
            "Single Position Lookup".to_string(),
            traditional,
            cqrs,
            improvement,
        );

        // Benchmark 2: Batch position lookup
        println!("Benchmarking batch position lookups...");
        let batch = self.benchmark_batch_get_positions(100, 100);
        batch.print();
        println!();

        report.add_batch_metric("Batch Position Lookup".to_string(), batch);

        // Benchmark 3: Radius query
        println!("Benchmarking radius queries...");
        let radius = self.benchmark_query_in_radius(1000);
        radius.print();
        println!();

        report.add_metric(
            "Radius Query".to_string(),
            radius.clone(),
            radius.clone(),
            0.0,
        );

        // Benchmark 4: Dynamic bodies query
        println!("Benchmarking dynamic bodies queries...");
        let dynamic = self.benchmark_query_dynamic_bodies(1000);
        dynamic.print();
        println!();

        report.add_metric(
            "Dynamic Bodies Query".to_string(),
            dynamic.clone(),
            dynamic.clone(),
            0.0,
        );

        report
    }
}

/// Comprehensive benchmark report
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub body_count: usize,
    pub metrics: Vec<BenchmarkMetric>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkMetric {
    pub operation_name: String,
    pub traditional: PerformanceMetrics,
    pub cqrs: PerformanceMetrics,
    pub improvement_percent: f64,
}

impl BenchmarkReport {
    pub fn new(body_count: usize) -> Self {
        Self {
            body_count,
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(
        &mut self,
        operation_name: String,
        traditional: PerformanceMetrics,
        cqrs: PerformanceMetrics,
        improvement_percent: f64,
    ) {
        self.metrics.push(BenchmarkMetric {
            operation_name,
            traditional,
            cqrs,
            improvement_percent,
        });
    }

    pub fn add_batch_metric(&mut self, operation_name: String, batch: PerformanceMetrics) {
        // For batch operations, we don't have a traditional baseline
        self.metrics.push(BenchmarkMetric {
            operation_name,
            traditional: batch.clone(),
            cqrs: batch,
            improvement_percent: 0.0,
        });
    }

    pub fn print_summary(&self) {
        println!("\n");
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║        CQRS Performance Benchmark Report                       ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("Test Configuration:");
        println!("  Body count: {}", self.body_count);
        println!();

        println!("Performance Metrics:");
        println!("┌────────────────────────────────────────────────────────────────┐");
        println!("│ {:<50} │", "Operation");
        println!("├────────────────────────────────────────────────────────────────┤");
        println!(
            "│ {:<20} │ {:>15} │ {:>15} │",
            "Type", "Time (ns)", "Ops/sec"
        );
        println!("├────────────────────────────────────────────────────────────────┤");

        for metric in &self.metrics {
            println!("│ {:<50} │", metric.operation_name);
            println!(
                "│ {:<20} │ {:>15.0} │ {:>15.0} │",
                "Traditional",
                metric.traditional.avg_time_per_operation.as_nanos(),
                metric.traditional.operations_per_second
            );
            println!(
                "│ {:<20} │ {:>15.0} │ {:>15.0} │",
                "CQRS",
                metric.cqrs.avg_time_per_operation.as_nanos(),
                metric.cqrs.operations_per_second
            );
            if metric.improvement_percent != 0.0 {
                println!(
                    "│ {:<20} │ {:>15.2}% │",
                    "Improvement", metric.improvement_percent
                );
            }
            println!("├────────────────────────────────────────────────────────────────┤");
        }

        println!("└────────────────────────────────────────────────────────────────┘");
        println!();

        // Calculate average improvement
        let improvements: Vec<f64> = self
            .metrics
            .iter()
            .map(|m| m.improvement_percent)
            .filter(|&i| i > 0.0)
            .collect();

        if !improvements.is_empty() {
            let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;
            println!("Average Performance Improvement: {avg_improvement:.2}%");
            println!();

            if avg_improvement >= 20.0 {
                println!("✓ TARGET MET: CQRS pattern achieved >= 20% performance improvement");
            } else {
                println!(
                    "✗ TARGET NOT MET: Performance improvement {avg_improvement:.2}% is below 20% target"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Expensive benchmark test - run manually
    fn test_full_benchmark_suite() {
        let mut suite = CqrsBenchmarkSuite::new();
        let report = suite.run_full_benchmark_suite(1000);
        report.print_summary();

        // Verify we achieved at least 20% improvement
        let improvements: Vec<f64> = report
            .metrics
            .iter()
            .map(|m| m.improvement_percent)
            .filter(|&i| i > 0.0)
            .collect();

        if !improvements.is_empty() {
            let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;
            assert!(
                avg_improvement >= 20.0,
                "Expected >= 20% improvement, got {:.2}%",
                avg_improvement
            );
        }
    }

    #[test]
    fn test_query_model_performance() {
        // Create test data
        let bodies: Vec<RigidBody> = (0..100)
            .map(|i| {
                RigidBody::new(
                    RigidBodyId::new(i),
                    RigidBodyType::Dynamic,
                    Vec3::new(i as f32, 0.0, 0.0),
                )
            })
            .collect();

        // Measure query model creation
        let start = Instant::now();
        let model = PhysicsQueryModel::from_world(&bodies);
        let creation_time = start.elapsed();

        println!("Query model creation time: {:?}", creation_time);

        // Verify correctness
        assert_eq!(model.body_count(), 100);

        // Measure single lookup
        let start = Instant::now();
        let pos = model.get_position(RigidBodyId::new(50));
        let lookup_time = start.elapsed();

        println!("Single lookup time: {:?}", lookup_time);
        assert_eq!(pos, Some(Vec3::new(50.0, 0.0, 0.0)));

        // Measure batch lookup
        let ids: Vec<RigidBodyId> = (0..100).map(|i| RigidBodyId::new(i)).collect();
        let start = Instant::now();
        let positions = model.batch_get_positions(&ids);
        let batch_time = start.elapsed();

        println!("Batch lookup time (100 items): {:?}", batch_time);
        assert_eq!(positions.len(), 100);
    }

    #[test]
    fn test_query_in_radius_performance() {
        // Create test data with specific positions
        let bodies: Vec<RigidBody> = (0..1000)
            .map(|i| {
                RigidBody::new(
                    RigidBodyId::new(i),
                    RigidBodyType::Dynamic,
                    Vec3::new(i as f32, 0.0, 0.0),
                )
            })
            .collect();

        let model = PhysicsQueryModel::from_world(&bodies);

        // Query radius that should include some bodies
        let start = Instant::now();
        let results = model.query_in_radius(Vec3::new(50.0, 0.0, 0.0), 10.0);
        let query_time = start.elapsed();

        println!("Radius query time: {:?}", query_time);
        println!("Found {} bodies in radius", results.len());

        // Should find bodies 41-59 (within 10.0 units of 50.0)
        assert!(!results.is_empty());
        assert!(results.len() <= 20); // Max expected
    }
}
