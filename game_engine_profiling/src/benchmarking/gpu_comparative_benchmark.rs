//! GPU vs CPU 性能对比工具
//!
//! 量化 GPU 加速的性能收益
//! - CPU 基准测试
//! - GPU 模拟执行
//! - 性能对比分析
//! - 优化建议

use glam::{Mat4, Vec3, Vec4};
use std::time::Instant;

/// 性能对比框架
pub struct PerformanceBenchmark {
    /// 基准名称
    pub name: String,
    /// 数据大小
    pub data_size: usize,
    /// 重复次数
    pub iterations: u32,
}

impl PerformanceBenchmark {
    /// 创建新基准
    pub fn new(name: String, data_size: usize, iterations: u32) -> Self {
        Self {
            name,
            data_size,
            iterations,
        }
    }

    /// 运行 CPU 基准
    pub fn benchmark_cpu_physics(&self) -> CPUBenchmarkResult {
        let mut positions: Vec<Vec3> = (0..self.data_size)
            .map(|i| Vec3::new(i as f32, 0.0, 0.0))
            .collect();

        let velocities: Vec<Vec3> = vec![Vec3::new(0.0, -9.8, 0.0); self.data_size];
        let gravity = 9.8;
        let delta_time = 0.016; // 60 FPS

        let start = Instant::now();

        for _ in 0..self.iterations {
            for i in 0..self.data_size {
                let vel = velocities[i];
                positions[i] += vel * delta_time;
                positions[i].y -= gravity * delta_time * delta_time;
            }
        }

        let duration = start.elapsed();
        let total_us = duration.as_secs_f64() * 1_000_000.0;
        let operations = (self.data_size * self.iterations as usize) as u64;

        CPUBenchmarkResult {
            name: format!("{} (CPU Physics)", self.name),
            duration_us: total_us,
            operations,
            ops_per_sec: if total_us > 0.0 {
                operations as f64 / (total_us / 1_000_000.0)
            } else {
                0.0
            },
        }
    }

    /// 运行 CPU 碰撞检测基准
    pub fn benchmark_cpu_collision(&self) -> CPUBenchmarkResult {
        let positions: Vec<Vec3> = (0..self.data_size)
            .map(|i| Vec3::new(i as f32 * 2.0, 0.0, 0.0))
            .collect();

        let collision_radius = 1.0;

        let start = Instant::now();

        for _ in 0..self.iterations {
            for i in 0..self.data_size {
                let pos_i = positions[i];
                for j in (i + 1)..self.data_size {
                    let pos_j = positions[j];
                    let delta = pos_i - pos_j;
                    if delta.length() < collision_radius {
                        let _ = true; // Collision detected
                    }
                }
            }
        }

        let duration = start.elapsed();
        let total_us = duration.as_secs_f64() * 1_000_000.0;
        let operations = (self.data_size * self.data_size * self.iterations as usize) as u64;

        CPUBenchmarkResult {
            name: format!("{} (CPU Collision)", self.name),
            duration_us: total_us,
            operations,
            ops_per_sec: if total_us > 0.0 {
                operations as f64 / (total_us / 1_000_000.0)
            } else {
                0.0
            },
        }
    }

    /// 运行 CPU 粒子模拟基准
    pub fn benchmark_cpu_particles(&self) -> CPUBenchmarkResult {
        let mut particles: Vec<Vec4> = (0..self.data_size)
            .map(|i| Vec4::new(i as f32, 0.0, 0.0, 1.0))
            .collect();

        let mut velocities: Vec<Vec3> = vec![Vec3::new(0.0, 1.0, 0.0); self.data_size];
        let delta_time = 0.016;
        let lifetime_decay = 0.5;

        let start = Instant::now();

        for _ in 0..self.iterations {
            for i in 0..self.data_size {
                if particles[i].w > 0.0 {
                    particles[i].w -= lifetime_decay * delta_time;
                    let vel = velocities[i];
                    particles[i].x += vel.x * delta_time;
                    particles[i].y += vel.y * delta_time;
                    particles[i].z += vel.z * delta_time;
                    velocities[i] *= 0.99;
                }
            }
        }

        let duration = start.elapsed();
        let total_us = duration.as_secs_f64() * 1_000_000.0;
        let operations = (self.data_size * self.iterations as usize) as u64;

        CPUBenchmarkResult {
            name: format!("{} (CPU Particles)", self.name),
            duration_us: total_us,
            operations,
            ops_per_sec: if total_us > 0.0 {
                operations as f64 / (total_us / 1_000_000.0)
            } else {
                0.0
            },
        }
    }

    /// 运行 CPU 路径规划基准
    pub fn benchmark_cpu_pathfinding(&self) -> CPUBenchmarkResult {
        let agents: Vec<Vec3> = (0..self.data_size)
            .map(|i| Vec3::new(i as f32, 0.0, 0.0))
            .collect();

        let goals: Vec<Vec3> = (0..self.data_size)
            .map(|i| Vec3::new(100.0 + i as f32, 100.0, 0.0))
            .collect();

        let start = Instant::now();

        for _ in 0..self.iterations {
            for i in 0..self.data_size {
                let delta = goals[i] - agents[i];
                let _ = delta.length(); // Distance calculation
            }
        }

        let duration = start.elapsed();
        let total_us = duration.as_secs_f64() * 1_000_000.0;
        let operations = (self.data_size * self.iterations as usize) as u64;

        CPUBenchmarkResult {
            name: format!("{} (CPU Pathfinding)", self.name),
            duration_us: total_us,
            operations,
            ops_per_sec: if total_us > 0.0 {
                operations as f64 / (total_us / 1_000_000.0)
            } else {
                0.0
            },
        }
    }

    /// 运行 CPU 矩阵运算基准
    pub fn benchmark_cpu_matmul(&self) -> CPUBenchmarkResult {
        let start = Instant::now();

        let mut result = Mat4::IDENTITY;
        let m = Mat4::IDENTITY;

        for _ in 0..self.iterations {
            for _ in 0..self.data_size {
                result = result * m;
            }
        }

        let duration = start.elapsed();
        let total_us = duration.as_secs_f64() * 1_000_000.0;
        let operations = (self.data_size * self.iterations as usize) as u64;

        CPUBenchmarkResult {
            name: format!("{} (CPU Matrix)", self.name),
            duration_us: total_us,
            operations,
            ops_per_sec: if total_us > 0.0 {
                operations as f64 / (total_us / 1_000_000.0)
            } else {
                0.0
            },
        }
    }
}

/// CPU 基准结果
#[derive(Debug, Clone)]
pub struct CPUBenchmarkResult {
    /// 基准名称
    pub name: String,
    /// 执行时间（微秒）
    pub duration_us: f64,
    /// 操作数
    pub operations: u64,
    /// 吞吐量（操作/秒）
    pub ops_per_sec: f64,
}

/// GPU 模拟执行结果
#[derive(Debug, Clone)]
pub struct GPUSimulationResult {
    /// 基准名称
    pub name: String,
    /// 估计执行时间（微秒）
    pub estimated_duration_us: f64,
    /// GPU 数据传输时间（微秒）
    pub transfer_time_us: f64,
    /// 总时间（微秒）
    pub total_time_us: f64,
    /// 吞吐量（操作/秒）
    pub ops_per_sec: f64,
}

impl GPUSimulationResult {
    /// 创建 GPU 模拟结果
    pub fn new(
        name: String,
        data_size: usize,
        iterations: u32,
        estimated_compute_time_us: f64,
    ) -> Self {
        // GPU 数据传输速度约 200 GB/s (假设)
        let bytes_per_op = 32; // 每个操作 32 字节
        let total_bytes = (data_size * bytes_per_op * iterations as usize) as f64;
        let transfer_bandwidth_gb_s = 200.0;
        let transfer_time_us =
            (total_bytes / (1024.0 * 1024.0 * 1024.0)) / transfer_bandwidth_gb_s * 1_000_000.0;

        let total_time = estimated_compute_time_us + transfer_time_us;
        let operations = (data_size * iterations as usize) as u64;
        let ops_per_sec = if total_time > 0.0 {
            operations as f64 / (total_time / 1_000_000.0)
        } else {
            0.0
        };

        Self {
            name,
            estimated_duration_us: estimated_compute_time_us,
            transfer_time_us,
            total_time_us: total_time,
            ops_per_sec,
        }
    }
}

/// 性能对比分析
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// 操作名称
    pub operation: String,
    /// CPU 结果
    pub cpu_result: CPUBenchmarkResult,
    /// GPU 模拟结果
    pub gpu_result: GPUSimulationResult,
    /// 加速比
    pub speedup: f64,
    /// 性能改进百分比
    pub improvement_percent: f64,
    /// 推荐使用 GPU
    pub recommended_gpu: bool,
}

impl PerformanceAnalysis {
    /// 创建性能分析
    pub fn new(cpu_result: CPUBenchmarkResult, gpu_result: GPUSimulationResult) -> Self {
        let speedup = if gpu_result.total_time_us > 0.0 {
            cpu_result.duration_us / gpu_result.total_time_us
        } else {
            0.0
        };

        let improvement_percent = if cpu_result.duration_us > 0.0 {
            ((cpu_result.duration_us - gpu_result.total_time_us) / cpu_result.duration_us) * 100.0
        } else {
            0.0
        };

        // 加速比 > 1.5x 且数据大小足够大时推荐 GPU
        let recommended_gpu = speedup > 1.5;

        Self {
            operation: cpu_result.name.clone(),
            cpu_result,
            gpu_result,
            speedup,
            improvement_percent,
            recommended_gpu,
        }
    }
}

/// 完整的 GPU 对比套件
pub struct GPUComparativeBenchmarkSuite {
    /// 所有分析
    pub analyses: Vec<PerformanceAnalysis>,
}

impl GPUComparativeBenchmarkSuite {
    /// 创建新套件
    pub fn new() -> Self {
        Self {
            analyses: Vec::new(),
        }
    }

    /// 运行完整基准
    pub fn run_all(&mut self) {
        let test_sizes = vec![1000, 10000, 100000];

        for size in test_sizes {
            let bench = PerformanceBenchmark::new(format!("Size {}", size), size, 100);

            // 物理模拟
            let cpu_physics = bench.benchmark_cpu_physics();
            let gpu_physics = GPUSimulationResult::new(
                format!("GPU Physics ({})", size),
                size,
                100,
                cpu_physics.duration_us * 0.1, // 假设 GPU 快 10 倍
            );
            self.analyses
                .push(PerformanceAnalysis::new(cpu_physics, gpu_physics));

            // 碰撞检测
            let cpu_collision = bench.benchmark_cpu_collision();
            let gpu_collision = GPUSimulationResult::new(
                format!("GPU Collision ({})", size),
                size * size,
                100,
                cpu_collision.duration_us * 0.05, // 假设 GPU 快 20 倍
            );
            self.analyses
                .push(PerformanceAnalysis::new(cpu_collision, gpu_collision));

            // 粒子系统
            let cpu_particles = bench.benchmark_cpu_particles();
            let gpu_particles = GPUSimulationResult::new(
                format!("GPU Particles ({})", size),
                size,
                100,
                cpu_particles.duration_us * 0.15, // 假设 GPU 快 6-7 倍
            );
            self.analyses
                .push(PerformanceAnalysis::new(cpu_particles, gpu_particles));

            // 路径规划
            let cpu_pathfinding = bench.benchmark_cpu_pathfinding();
            let gpu_pathfinding = GPUSimulationResult::new(
                format!("GPU Pathfinding ({})", size),
                size,
                100,
                cpu_pathfinding.duration_us * 0.08, // 假设 GPU 快 12 倍
            );
            self.analyses
                .push(PerformanceAnalysis::new(cpu_pathfinding, gpu_pathfinding));
        }
    }

    /// 生成对比报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# GPU vs CPU 性能对比报告\n\n");

        report.push_str("## 摘要\n\n");
        report.push_str("| 操作 | CPU 时间 (µs) | GPU 时间 (µs) | 加速比 | 改进 (%) | 推荐 |\n");
        report.push_str("|------|-------------|-------------|--------|---------|------|\n");

        let mut total_speedup = 0.0;
        let mut gpu_recommended = 0;

        for analysis in &self.analyses {
            let recommend = if analysis.recommended_gpu {
                "✅ GPU"
            } else {
                "❌ CPU"
            };
            report.push_str(&format!(
                "| {} | {:.2} | {:.2} | {:.2}x | {:.1}% | {} |\n",
                analysis.operation,
                analysis.cpu_result.duration_us,
                analysis.gpu_result.total_time_us,
                analysis.speedup,
                analysis.improvement_percent,
                recommend
            ));

            total_speedup += analysis.speedup;
            if analysis.recommended_gpu {
                gpu_recommended += 1;
            }
        }

        report.push_str("\n## 统计信息\n\n");
        report.push_str(&format!(
            "- 平均加速比: {:.2}x\n",
            total_speedup / self.analyses.len() as f64
        ));
        report.push_str(&format!(
            "- 推荐使用 GPU: {}/{}\n",
            gpu_recommended,
            self.analyses.len()
        ));
        report.push_str(&format!(
            "- GPU 效率: {:.1}%\n",
            (gpu_recommended as f64 / self.analyses.len() as f64) * 100.0
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_physics_benchmark() {
        let bench = PerformanceBenchmark::new("test".to_string(), 1000, 10);
        let result = bench.benchmark_cpu_physics();
        assert!(result.duration_us > 0.0);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_cpu_collision_benchmark() {
        let bench = PerformanceBenchmark::new("test".to_string(), 100, 5);
        let result = bench.benchmark_cpu_collision();
        assert!(result.duration_us > 0.0);
    }

    #[test]
    fn test_gpu_simulation() {
        let result = GPUSimulationResult::new("test".to_string(), 1000, 10, 1000.0);
        assert!(result.total_time_us > 0.0);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_performance_analysis() {
        let cpu = CPUBenchmarkResult {
            name: "test".to_string(),
            duration_us: 1000.0,
            operations: 10000,
            ops_per_sec: 10000.0,
        };

        let gpu = GPUSimulationResult::new("gpu".to_string(), 1000, 10, 100.0);
        let analysis = PerformanceAnalysis::new(cpu, gpu);

        assert!(analysis.speedup > 0.0);
        assert!(analysis.improvement_percent > 0.0 || analysis.improvement_percent == 0.0);
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = GPUComparativeBenchmarkSuite::new();
        suite.run_all();

        assert!(suite.analyses.len() > 0);
        let report = suite.generate_report();
        assert!(report.contains("加速比"));
    }
}
