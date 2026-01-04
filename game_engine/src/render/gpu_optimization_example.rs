//  GPU优化示例和性能测试
//
//  演示如何使用增强的GPU管理器进行优化渲染。

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_unified_manager_v2::{
    EnhancedGpuRenderConfig, EnhancedGpuRenderManager, EnhancedGpuRenderStats,
};
use std::time::Instant;

/// GPU优化示例
pub struct GpuOptimizationExample {
    /// GPU渲染管理器
    manager: Option<EnhancedGpuRenderManager>,
    /// 实例数据
    instances: Vec<GpuInstance>,
    /// 视图投影矩阵
    view_proj: [[f32; 4]; 4],
    /// 相机位置
    camera_position: (f32, f32, f32),
}

impl Default for GpuOptimizationExample {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuOptimizationExample {
    /// 创建新的示例
    pub fn new() -> Self {
        Self {
            manager: None,
            instances: Vec::new(),
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            camera_position: (0.0, 0.0, 0.0),
        }
    }

    /// 初始化GPU管理器
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `config`: 渲染配置
    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        config: EnhancedGpuRenderConfig,
    ) -> Result<(), String> {
        let manager = EnhancedGpuRenderManager::new(device, config)
            .map_err(|e| format!("Failed to create GPU manager: {e}"))?;
        self.manager = Some(manager);
        Ok(())
    }

    /// 生成测试实例数据
    ///
    /// 创建大量实例用于性能测试。
    ///
    /// # 参数
    ///
    /// - `count`: 实例数量
    /// - `spread`: 扩散范围
    pub fn generate_test_instances(&mut self, count: u32, spread: f32) {
        self.instances.clear();
        self.instances.reserve(count as usize);

        for i in 0..count {
            // 在3D空间中随机分布实例
            let x = (i as f32 % 10.0 - 5.0) * spread;
            let y = ((i as f32 / 10.0).floor() % 10.0 - 5.0) * spread;
            let z = ((i as f32 / 100.0).floor() % 10.0 - 5.0) * spread;

            let mut instance = GpuInstance::default();

            // 设置模型矩阵（平移）
            instance.model[3][0] = x;
            instance.model[3][1] = y;
            instance.model[3][2] = z;

            // 设置AABB（单位立方体）
            instance.aabb_min = [-0.5, -0.5, -0.5];
            instance.aabb_max = [0.5, 0.5, 0.5];

            // 设置实例ID
            instance.instance_id = i;

            self.instances.push(instance);
        }
    }

    /// 运行性能测试
    ///
    /// 测试不同配置下的渲染性能。
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    /// - `iterations`: 测试迭代次数
    ///
    /// # 返回
    ///
    /// 返回性能测试结果。
    pub fn run_performance_test(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        iterations: u32,
    ) -> Result<PerformanceTestResult, String> {
        if self.manager.is_none() {
            return Err("GPU manager not initialized".to_string());
        }

        let manager = self.manager.as_mut().unwrap();
        let mut results = Vec::new();
        let instance_count = self.instances.len() as u32;

        println!("Starting performance test...");
        println!("  Instances: {instance_count}");
        println!("  Iterations: {iterations}");

        // 上传实例数据
        manager.update_instances(device, queue, &self.instances);

        // 运行测试迭代
        for iter in 0..iterations {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Performance Test Iteration {iter}")),
            });

            // 执行渲染
            let start = Instant::now();
            let stats = manager.render(
                &mut encoder,
                device,
                queue,
                self.view_proj,
                self.camera_position,
                instance_count,
            )?;
            let duration = start.elapsed();

            // 提交命令
            queue.submit(Some(encoder.finish()));

            // 记录结果
            results.push(stats.clone());

            // 每10次迭代打印一次进度
            if (iter + 1) % 10 == 0 {
                println!(
                    "  Iteration {}/{}: {:.2}ms, visible: {}/{} ({:.1}%)",
                    iter + 1,
                    iterations,
                    duration.as_millis(),
                    stats.visible_instances,
                    stats.total_instances,
                    (1.0 - stats.cull_rate) * 100.0
                );
            }
        }

        // 计算统计数据
        let avg_gpu_time = results.iter().map(|s| s.gpu_time_ms).sum::<f32>() / iterations as f32;
        let avg_cull_rate = results.iter().map(|s| s.cull_rate).sum::<f32>() / iterations as f32;
        let avg_visible = results.iter().map(|s| s.visible_instances).sum::<u32>() / iterations;

        Ok(PerformanceTestResult {
            iterations,
            instance_count,
            avg_gpu_time_ms: avg_gpu_time,
            avg_cull_rate,
            avg_visible_instances: avg_visible,
            vram_usage: manager.get_stats().vram_used,
            vram_budget: manager.get_stats().vram_budget,
        })
    }

    /// 比较不同剔除策略的性能
    ///
    /// 测试启用/禁用不同剔除策略的性能差异。
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    ///
    /// # 返回
    ///
    /// 返回比较结果。
    pub fn compare_culling_strategies(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<CullingComparisonResult, String> {
        println!("\n=== Comparing Culling Strategies ===\n");

        let instance_count = self.instances.len() as u32;

        // 测试1: 无剔除
        println!("Test 1: No culling");
        let config_no_cull = EnhancedGpuRenderConfig {
            enable_frustum_culling: false,
            enable_occlusion_culling: false,
            enable_distance_culling: false,
            ..Default::default()
        };

        let mut manager1 =
            EnhancedGpuRenderManager::new(device, config_no_cull).map_err(|e| e.to_string())?;
        manager1.update_instances(device, queue, &self.instances);

        let mut encoder1 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("No Culling Test"),
        });
        let start1 = Instant::now();
        let stats1 = manager1.render(
            &mut encoder1,
            device,
            queue,
            self.view_proj,
            self.camera_position,
            instance_count,
        )?;
        let duration1 = start1.elapsed();
        queue.submit(Some(encoder1.finish()));

        println!(
            "  Time: {:.2}ms, Visible: {}/{}",
            duration1.as_millis(),
            stats1.visible_instances,
            stats1.total_instances
        );

        // 测试2: 仅视锥剔除
        println!("\nTest 2: Frustum culling only");
        let config_frustum = EnhancedGpuRenderConfig {
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
            enable_distance_culling: false,
            ..Default::default()
        };

        let mut manager2 =
            EnhancedGpuRenderManager::new(device, config_frustum).map_err(|e| e.to_string())?;
        manager2.update_instances(device, queue, &self.instances);

        let mut encoder2 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frustum Culling Test"),
        });
        let start2 = Instant::now();
        let stats2 = manager2.render(
            &mut encoder2,
            device,
            queue,
            self.view_proj,
            self.camera_position,
            instance_count,
        )?;
        let duration2 = start2.elapsed();
        queue.submit(Some(encoder2.finish()));

        println!(
            "  Time: {:.2}ms, Visible: {}/{} ({:.1}% culled)",
            duration2.as_millis(),
            stats2.visible_instances,
            stats2.total_instances,
            stats2.cull_rate * 100.0
        );

        // 测试3: 完整剔除（视锥+距离）
        println!("\nTest 3: Full culling (frustum + distance)");
        let config_full = EnhancedGpuRenderConfig {
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
            enable_distance_culling: true,
            max_view_distance: 1000.0,
            distance_culling_threshold: 500.0,
            ..Default::default()
        };

        let mut manager3 =
            EnhancedGpuRenderManager::new(device, config_full).map_err(|e| e.to_string())?;
        manager3.update_instances(device, queue, &self.instances);

        let mut encoder3 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Full Culling Test"),
        });
        let start3 = Instant::now();
        let stats3 = manager3.render(
            &mut encoder3,
            device,
            queue,
            self.view_proj,
            self.camera_position,
            instance_count,
        )?;
        let duration3 = start3.elapsed();
        queue.submit(Some(encoder3.finish()));

        println!(
            "  Time: {:.2}ms, Visible: {}/{} ({:.1}% culled)",
            duration3.as_millis(),
            stats3.visible_instances,
            stats3.total_instances,
            stats3.cull_rate * 100.0
        );

        // 计算性能提升
        let speedup_no_cull = duration1.as_secs_f32() / duration2.as_secs_f32();
        let speedup_full = duration1.as_secs_f32() / duration3.as_secs_f32();

        println!("\n=== Performance Summary ===");
        println!("Frustum culling speedup: {speedup_no_cull:.2}x");
        println!("Full culling speedup: {speedup_full:.2}x");

        Ok(CullingComparisonResult {
            no_culling_time_ms: duration1.as_millis() as f32,
            frustum_culling_time_ms: duration2.as_millis() as f32,
            full_culling_time_ms: duration3.as_millis() as f32,
            frustum_cull_rate: stats2.cull_rate,
            full_cull_rate: stats3.cull_rate,
            frustum_speedup: speedup_no_cull,
            full_speedup: speedup_full,
        })
    }

    /// VRAM压力测试
    ///
    /// 测试VRAM管理和资源卸载功能。
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    ///
    /// # 返回
    ///
    /// 返回VRAM测试结果。
    pub fn vram_stress_test(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<VramStressTestResult, String> {
        println!("\n=== VRAM Stress Test ===\n");

        // 设置较小的VRAM预算以测试卸载
        let vram_budget = 100 * 1024 * 1024; // 100MB
        let config = EnhancedGpuRenderConfig {
            vram_budget,
            vram_warning_threshold: 0.8,
            enable_auto_unload: true,
            resource_unload_delay: 10,
            ..Default::default()
        };

        let mut manager =
            EnhancedGpuRenderManager::new(device, config).map_err(|e| e.to_string())?;

        let instance_count = self.instances.len() as u32;
        manager.update_instances(device, queue, &self.instances);

        // 模拟多帧渲染
        println!("Simulating 100 frames...");
        for frame in 0..100 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("VRAM Test Frame {frame}")),
            });

            manager.render(
                &mut encoder,
                device,
                queue,
                self.view_proj,
                self.camera_position,
                instance_count,
            )?;
            queue.submit(Some(encoder.finish()));

            // 每20帧打印一次状态
            if (frame + 1) % 20 == 0 {
                let stats = manager.get_stats();
                println!(
                    "  Frame {}: VRAM {:.1}MB / {:.1}MB ({:.1}%)",
                    frame + 1,
                    stats.vram_used as f32 / (1024.0 * 1024.0),
                    stats.vram_budget as f32 / (1024.0 * 1024.0),
                    stats.vram_usage_ratio * 100.0
                );
            }
        }

        let final_stats = manager.get_stats();

        println!("\n=== VRAM Test Complete ===");
        println!(
            "Final VRAM usage: {:.1}MB / {:.1}MB ({:.1}%)",
            final_stats.vram_used as f32 / (1024.0 * 1024.0),
            final_stats.vram_budget as f32 / (1024.0 * 1024.0),
            final_stats.vram_usage_ratio * 100.0
        );
        println!("Resources unloaded: {}", final_stats.unloaded_resources);

        Ok(VramStressTestResult {
            initial_vram_mb: (vram_budget as f32 / (1024.0 * 1024.0)),
            final_vram_used_mb: (final_stats.vram_used as f32 / (1024.0 * 1024.0)),
            vram_usage_ratio: final_stats.vram_usage_ratio,
            resources_unloaded: final_stats.unloaded_resources,
        })
    }
}

/// 性能测试结果
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    /// 迭代次数
    pub iterations: u32,
    /// 实例数量
    pub instance_count: u32,
    /// 平均GPU时间（毫秒）
    pub avg_gpu_time_ms: f32,
    /// 平均剔除率
    pub avg_cull_rate: f32,
    /// 平均可见实例数
    pub avg_visible_instances: u32,
    /// VRAM使用量
    pub vram_usage: usize,
    /// VRAM预算
    pub vram_budget: usize,
}

/// 剔除策略比较结果
#[derive(Debug, Clone)]
pub struct CullingComparisonResult {
    /// 无剔除时间（毫秒）
    pub no_culling_time_ms: f32,
    /// 视锥剔除时间（毫秒）
    pub frustum_culling_time_ms: f32,
    /// 完整剔除时间（毫秒）
    pub full_culling_time_ms: f32,
    /// 视锥剔除率
    pub frustum_cull_rate: f32,
    /// 完整剔除率
    pub full_cull_rate: f32,
    /// 视锥剔除加速比
    pub frustum_speedup: f32,
    /// 完整剔除加速比
    pub full_speedup: f32,
}

/// VRAM压力测试结果
#[derive(Debug, Clone)]
pub struct VramStressTestResult {
    /// 初始VRAM预算（MB）
    pub initial_vram_mb: f32,
    /// 最终VRAM使用量（MB）
    pub final_vram_used_mb: f32,
    /// VRAM使用率
    pub vram_usage_ratio: f32,
    /// 卸载的资源数
    pub resources_unloaded: u32,
}

/// 运行完整的GPU优化演示
///
/// 这是示例的入口函数，展示所有功能。
pub fn run_gpu_optimization_demo(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), String> {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║     GPU Optimization Demo - Enhanced Renderer         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let mut example = GpuOptimizationExample::new();

    // 初始化
    println!("Initializing GPU manager...");
    let config = EnhancedGpuRenderConfig::default();
    example.initialize(device, config)?;

    // 生成测试数据
    println!("Generating test instances (10,000)...\n");
    example.generate_test_instances(10000, 10.0);

    // 运行性能测试
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║              Performance Test                          ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    let perf_result = example.run_performance_test(device, queue, 50)?;
    println!("\nPerformance Test Results:");
    println!("  Average GPU time: {:.2}ms", perf_result.avg_gpu_time_ms);
    println!(
        "  Average cull rate: {:.1}%",
        perf_result.avg_cull_rate * 100.0
    );
    println!("  Average visible: {}", perf_result.avg_visible_instances);
    println!(
        "  VRAM usage: {:.1}MB / {:.1}MB",
        perf_result.vram_usage as f32 / (1024.0 * 1024.0),
        perf_result.vram_budget as f32 / (1024.0 * 1024.0)
    );

    // 比较剔除策略
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║           Culling Strategy Comparison                  ║");
    println!("╚════════════════════════════════════════════════════════╝");
    let cull_result = example.compare_culling_strategies(device, queue)?;
    println!("\nCulling Comparison Summary:");
    println!("  No culling: {:.2}ms", cull_result.no_culling_time_ms);
    println!(
        "  Frustum culling: {:.2}ms ({:.2}x speedup)",
        cull_result.frustum_culling_time_ms, cull_result.frustum_speedup
    );
    println!(
        "  Full culling: {:.2}ms ({:.2}x speedup)",
        cull_result.full_culling_time_ms, cull_result.full_speedup
    );

    // VRAM压力测试
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║               VRAM Stress Test                          ║");
    println!("╚════════════════════════════════════════════════════════╝");
    let vram_result = example.vram_stress_test(device, queue)?;
    println!("\nVRAM Test Summary:");
    println!("  Budget: {:.1}MB", vram_result.initial_vram_mb);
    println!(
        "  Final usage: {:.1}MB ({:.1}%)",
        vram_result.final_vram_used_mb,
        vram_result.vram_usage_ratio * 100.0
    );
    println!("  Resources unloaded: {}", vram_result.resources_unloaded);

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║                    Demo Complete                       ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_creation() {
        let example = GpuOptimizationExample::new();
        assert!(example.manager.is_none());
        assert_eq!(example.instances.len(), 0);
    }

    #[test]
    fn test_instance_generation() {
        let mut example = GpuOptimizationExample::new();
        example.generate_test_instances(100, 5.0);
        assert_eq!(example.instances.len(), 100);
    }
}
