//  GPU管理器性能基准测试
//
//  测试GPU管理器的性能表现，包括：
//  - GPU剔除性能
//  - 间接绘制性能
//  - VRAM管理性能
//  - 综合渲染性能

use game_engine::render::gpu_driven::culling::GpuInstance;
use game_engine::render::gpu_unified_manager_v2::{
    EnhancedGpuRenderConfig, EnhancedGpuRenderManager,
};
use std::time::Instant;

/// 性能基准测试结果
#[derive(Debug)]
pub struct BenchmarkResults {
    /// 测试名称
    pub name: String,
    /// 迭代次数
    pub iterations: u32,
    /// 实例数量
    pub instance_count: u32,
    /// 总时间（毫秒）
    pub total_time_ms: f32,
    /// 平均时间（毫秒）
    pub avg_time_ms: f32,
    /// 最小时间（毫秒）
    pub min_time_ms: f32,
    /// 最大时间（毫秒）
    pub max_time_ms: f32,
    /// 每秒帧数
    pub fps: f32,
}

impl BenchmarkResults {
    /// 创建基准测试结果
    pub fn new(name: String, iterations: u32, instance_count: u32, times: Vec<f32>) -> Self {
        let total_time_ms: f32 = times.iter().sum();
        let avg_time_ms = total_time_ms / iterations as f32;
        let min_time_ms = times.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_time_ms = times.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let fps = if avg_time_ms > 0.0 {
            1000.0 / avg_time_ms
        } else {
            0.0
        };

        Self {
            name,
            iterations,
            instance_count,
            total_time_ms,
            avg_time_ms,
            min_time_ms,
            max_time_ms,
            fps,
        }
    }

    /// 打印结果
    pub fn print(&self) {
        println!("┌─ {} ", self.name);
        println!("│ Instances: {}", self.instance_count);
        println!("│ Iterations: {}", self.iterations);
        println!("│ Total time: {:.2}ms", self.total_time_ms);
        println!(
            "│ Average: {:.2}ms (min: {:.2}ms, max: {:.2}ms)",
            self.avg_time_ms, self.min_time_ms, self.max_time_ms
        );
        println!("│ FPS: {:.1}", self.fps);
        println!("└─────────────────────────────────────");
    }
}

/// GPU管理器基准测试套件
pub struct GpuManagerBenchmarks {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    test_instances: Vec<GpuInstance>,
}

impl GpuManagerBenchmarks {
    /// 创建基准测试套件
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            test_instances: Vec::new(),
        }
    }

    /// 初始化WGPU设备
    pub async fn init(&mut self) -> Result<(), String> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Benchmark Device"),
                    required_features: wgpu::Features::TIMESTAMP_QUERY
                        | wgpu::Features::INDIRECT_FIRST_INSTANCE,
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;

        self.device = Some(device);
        self.queue = Some(queue);

        Ok(())
    }

    /// 生成测试实例数据
    fn generate_test_instances(&mut self, count: u32) {
        self.test_instances.clear();
        self.test_instances.reserve(count as usize);

        for i in 0..count {
            let x = (i as f32 % 100.0 - 50.0) * 10.0;
            let y = ((i as f32 / 100.0).floor() % 100.0 - 50.0) * 10.0;
            let z = ((i as f32 / 10000.0).floor() % 100.0 - 50.0) * 10.0;

            let mut instance = GpuInstance::default();
            instance.model[3][0] = x;
            instance.model[3][1] = y;
            instance.model[3][2] = z;
            instance.aabb_min = [-0.5, -0.5, -0.5];
            instance.aabb_max = [0.5, 0.5, 0.5];
            instance.instance_id = i;

            self.test_instances.push(instance);
        }
    }

    /// 基准测试：基础GPU剔除性能
    pub fn benchmark_gpu_culling(
        &mut self,
        instance_count: u32,
        iterations: u32,
    ) -> BenchmarkResults {
        self.generate_test_instances(instance_count);

        let device = self.device.as_ref().expect("Device not initialized");
        let queue = self.queue.as_ref().expect("Queue not initialized");

        let config = EnhancedGpuRenderConfig {
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
            enable_distance_culling: false,
            ..Default::default()
        };

        let mut manager =
            EnhancedGpuRenderManager::new(device, config).expect("Failed to create manager");
        manager.update_instances(device, queue, &self.test_instances);

        let mut times = Vec::new();

        for _ in 0..iterations {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Benchmark Encoder"),
            });

            let start = Instant::now();
            manager
                .render(
                    &mut encoder,
                    device,
                    queue,
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    (0.0, 0.0, 0.0),
                    instance_count,
                )
                .expect("Render failed");
            let elapsed = start.elapsed().as_millis() as f32;

            times.push(elapsed);
            queue.submit(Some(encoder.finish()));
        }

        BenchmarkResults::new(
            "GPU Culling Performance".to_string(),
            iterations,
            instance_count,
            times,
        )
    }

    /// 基准测试：完整剔除系统性能
    pub fn benchmark_full_culling(
        &mut self,
        instance_count: u32,
        iterations: u32,
    ) -> BenchmarkResults {
        self.generate_test_instances(instance_count);

        let device = self.device.as_ref().expect("Device not initialized");
        let queue = self.queue.as_ref().expect("Queue not initialized");

        let config = EnhancedGpuRenderConfig {
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
            enable_distance_culling: true,
            max_view_distance: 1000.0,
            distance_culling_threshold: 500.0,
            ..Default::default()
        };

        let mut manager =
            EnhancedGpuRenderManager::new(device, config).expect("Failed to create manager");
        manager.update_instances(device, queue, &self.test_instances);

        let mut times = Vec::new();

        for _ in 0..iterations {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Benchmark Encoder"),
            });

            let start = Instant::now();
            manager
                .render(
                    &mut encoder,
                    device,
                    queue,
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    (0.0, 0.0, 0.0),
                    instance_count,
                )
                .expect("Render failed");
            let elapsed = start.elapsed().as_millis() as f32;

            times.push(elapsed);
            queue.submit(Some(encoder.finish()));
        }

        BenchmarkResults::new(
            "Full Culling Performance".to_string(),
            iterations,
            instance_count,
            times,
        )
    }

    /// 基准测试：VRAM管理性能
    pub fn benchmark_vram_management(
        &mut self,
        instance_count: u32,
        iterations: u32,
    ) -> BenchmarkResults {
        self.generate_test_instances(instance_count);

        let device = self.device.as_ref().expect("Device not initialized");
        let queue = self.queue.as_ref().expect("Queue not initialized");

        let config = EnhancedGpuRenderConfig {
            vram_budget: 50 * 1024 * 1024, // 50MB
            enable_auto_unload: true,
            resource_unload_delay: 10,
            ..Default::default()
        };

        let mut manager =
            EnhancedGpuRenderManager::new(device, config).expect("Failed to create manager");
        manager.update_instances(device, queue, &self.test_instances);

        let mut times = Vec::new();

        for _ in 0..iterations {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Benchmark Encoder"),
            });

            let start = Instant::now();
            manager
                .render(
                    &mut encoder,
                    device,
                    queue,
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    (0.0, 0.0, 0.0),
                    instance_count,
                )
                .expect("Render failed");
            let elapsed = start.elapsed().as_millis() as f32;

            times.push(elapsed);
            queue.submit(Some(encoder.finish()));
        }

        BenchmarkResults::new(
            "VRAM Management Performance".to_string(),
            iterations,
            instance_count,
            times,
        )
    }

    /// 运行所有基准测试
    pub async fn run_all_benchmarks(&mut self) -> Result<(), String> {
        self.init().await?;

        println!("╔════════════════════════════════════════════════════════╗");
        println!("║     GPU Manager Performance Benchmarks                 ║");
        println!("╚════════════════════════════════════════════════════════╝\n");

        let instance_counts = vec![1000, 5000, 10000, 50000];
        let iterations = 50;

        // 测试不同的实例数量
        for &count in &instance_counts {
            println!("\n▶ Testing with {} instances", count);
            println!("─────────────────────────────────────────────────────");

            let result1 = self.benchmark_gpu_culling(count, iterations);
            result1.print();

            let result2 = self.benchmark_full_culling(count, iterations);
            result2.print();

            let result3 = self.benchmark_vram_management(count, iterations);
            result3.print();
        }

        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║              Benchmarks Complete                       ║");
        println!("╚════════════════════════════════════════════════════════╝");

        Ok(())
    }
}

/// 运行基准测试的主函数
#[tokio::main]
async fn main() -> Result<(), String> {
    let mut benchmarks = GpuManagerBenchmarks::new();
    benchmarks.run_all_benchmarks().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_results_creation() {
        let times = vec![10.0, 20.0, 30.0];
        let results = BenchmarkResults::new("Test".to_string(), 3, 1000, times);

        assert_eq!(results.name, "Test");
        assert_eq!(results.iterations, 3);
        assert_eq!(results.instance_count, 1000);
        assert_eq!(results.avg_time_ms, 20.0);
        assert_eq!(results.min_time_ms, 10.0);
        assert_eq!(results.max_time_ms, 30.0);
    }
}
