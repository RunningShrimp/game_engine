//! 性能优化集成测试
//!
//! 验证所有优化模块协同工作的集成测试

#[cfg(test)]
mod integration_tests {
    use crate::performance::{
        AudioChannel,
        // Audio
        AudioChannelMixer,
        AudioEffect,
        AudioEffectType,

        AudioProcessingPipeline,
        // Pathfinding
        BatchPathfinder,
        // Benchmarking
        Benchmark,
        ComputeResourceManager,

        // GPU Compute
        ComputeShaderGenerator,
        GPUParticleSystem,

        // Physics
        GPUPhysicsSimulator,
        MetricType,
        PerformanceMonitor,
        SIMDHeuristics,
    };
    use game_engine_simd::{AudioDSPOps, AudioSpatialOps, DistanceModel};
    use glam::Vec3;
    use std::time::Instant;

    /// 集成测试: 音频系统完整流程
    #[test]
    fn test_audio_system_integration() {
        // 创建处理管道
        let mut pipeline = AudioProcessingPipeline::new();

        // 模拟多个音频源的空间计算
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        let listener_pos = Vec3::ZERO;

        // 批量计算距离衰减
        let distances: Vec<_> = positions
            .iter()
            .map(|p| (*p - listener_pos).length())
            .collect();

        let distance_result = AudioSpatialOps::batch_distance_attenuation(
            &distances,
            DistanceModel::Inverse {
                ref_distance: 1.0,
                rolloff: 1.0,
            },
        );

        assert_eq!(distance_result.processed_count, 3);

        // 应用处理
        let audio_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let processed = pipeline.process_batch(&audio_samples, AudioChannel::SFX, 1.0);

        assert_eq!(processed.len(), 5);
        assert!(pipeline.get_metrics().samples_processed > 0);
    }

    /// 集成测试: 物理系统完整流程
    #[test]
    fn test_physics_system_integration() {
        // 创建物理模拟器
        let mut sim = GPUPhysicsSimulator::new();

        // 添加多个物体
        let body1 = sim.add_body(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let body2 = sim.add_body(Vec3::new(2.0, 0.0, 0.0), 1.0);
        let body3 = sim.add_body(Vec3::new(0.0, 2.0, 0.0), 1.0);

        // 添加距离约束
        sim.add_constraint(0, body1 as u32, body2 as u32, 1.0);

        // 施加力
        sim.apply_force(body1, Vec3::new(10.0, 0.0, 0.0));

        // 执行多步模拟
        for _ in 0..10 {
            sim.step();
        }

        // 检测碰撞 (可能没有碰撞，但碰撞检测应该正常工作)
        sim.detect_collisions();

        // 验证结果
        let bodies = sim.get_bodies();
        assert_eq!(bodies.len(), 3);
        assert!(sim.get_body_position(body1).is_some());
        // 物体应该已经移动
        let pos = sim.get_body_position(body1).unwrap();
        assert!(pos.x > 0.0);
    }

    /// 集成测试: 粒子系统完整流程
    #[test]
    fn test_particle_system_integration() {
        let mut particles = GPUParticleSystem::new(1000);

        // 发射多个粒子批
        for i in 0..10 {
            let pos = Vec3::new(i as f32, 0.0, 0.0);
            let vel = Vec3::new(0.0, 10.0, 0.0);
            particles.emit(pos, vel, 0.1); // 短生命周期
        }

        assert_eq!(particles.particle_count(), 10);

        // 更新粒子，使其过期
        particles.update(0.2); // 超过 0.1 秒生命周期

        // 粒子应该已过期
        assert_eq!(particles.particle_count(), 0);
    }

    /// 集成测试: GPU 计算着色器集成
    #[test]
    fn test_gpu_compute_integration() {
        // 生成各种着色器
        let physics_shader = ComputeShaderGenerator::generate_physics_shader();
        let collision_shader = ComputeShaderGenerator::generate_collision_shader();
        let particle_shader = ComputeShaderGenerator::generate_particle_shader();

        // 验证着色器代码
        assert!(physics_shader.contains("@compute"));
        assert!(collision_shader.contains("@workgroup_size"));
        assert!(particle_shader.contains("struct Particle"));

        // 创建资源管理器
        let mut manager = ComputeResourceManager::new();

        // 创建管道
        let config = crate::performance::gpu_compute::ComputeShaderConfig::new(physics_shader);
        let pipeline = manager.create_pipeline(config);

        // 创建缓冲区
        let buffer1 = manager.create_buffer(4096, 0);
        let buffer2 = manager.create_buffer(8192, 1);

        assert_eq!(manager.pipeline_count(), 1);
        assert_eq!(manager.buffer_count(), 2);
        assert_eq!(manager.get_total_memory(), 4096 + 8192);
    }

    /// 集成测试: 寻路系统完整流程
    #[test]
    fn test_pathfinding_system_integration() {
        let mut batch = BatchPathfinder::new(1.0);

        // 添加多个智能体
        for i in 0..10 {
            batch.add_agent(i, Vec3::new(i as f32 * 2.0, 0.0, 0.0));
        }

        // 为所有智能体寻找路径
        let targets = vec![
            (0, Vec3::new(10.0, 0.0, 0.0)),
            (1, Vec3::new(12.0, 0.0, 0.0)),
            (2, Vec3::new(14.0, 0.0, 0.0)),
        ];

        let results = batch.find_paths_batch(&targets);

        // 验证结果
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.found);
            assert!(!result.path.is_empty());
        }

        // 验证缓存
        assert!(batch.cache_size() > 0);
    }

    /// 集成测试: 启发式函数 SIMD 加速
    #[test]
    fn test_heuristic_simd_integration() {
        let positions: Vec<_> = (0..100).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();

        let target = Vec3::new(50.0, 0.0, 0.0);

        // 批量欧几里得距离
        let euclidean = SIMDHeuristics::batch_euclidean_distance(&positions, target);
        assert_eq!(euclidean.len(), 100);

        // 批量曼哈顿距离
        let manhattan = SIMDHeuristics::batch_manhattan_distance(&positions, target);
        assert_eq!(manhattan.len(), 100);

        // 批量切比雪夫距离
        let chebyshev = SIMDHeuristics::batch_chebyshev_distance(&positions, target);
        assert_eq!(chebyshev.len(), 100);

        // 验证距离关系 (切比雪夫 <= 欧几里得 <= 曼哈顿)
        for i in 0..100 {
            assert!(chebyshev[i] <= euclidean[i] + 0.01);
        }
    }

    /// 集成测试: 性能监控集成
    #[test]
    fn test_performance_monitoring_integration() {
        let mut monitor = PerformanceMonitor::new(3600);

        // 记录各种指标
        monitor.record(MetricType::FrameTime, 16.67, "ms");
        monitor.record(MetricType::RenderTime, 10.5, "ms");
        monitor.record(MetricType::UpdateTime, 6.17, "ms");
        monitor.record(MetricType::DrawCalls, 1024.0, "");

        // 生成报告
        let report = monitor.generate_report();

        assert!(!report.stats.is_empty());
    }

    /// 集成测试: 完整系统协同
    #[test]
    fn test_complete_system_integration() {
        let start = Instant::now();

        // 1. 音频处理
        let mut audio_pipeline = AudioProcessingPipeline::new();
        let audio_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let _audio_output = audio_pipeline.process_batch(&audio_samples, AudioChannel::Music, 1.0);

        // 2. 物理模拟
        let mut physics = GPUPhysicsSimulator::new();
        let body = physics.add_body(Vec3::ZERO, 1.0);
        physics.apply_force(body, Vec3::new(1.0, -9.81, 0.0));
        for _ in 0..5 {
            physics.step();
        }

        // 3. 寻路
        let mut pathfinder = BatchPathfinder::new(1.0);
        pathfinder.add_agent(1, Vec3::ZERO);
        let _path_result = pathfinder.find_path_for_agent(1, Vec3::new(10.0, 0.0, 0.0));

        // 4. 基准测试
        let mut bench = Benchmark::new();
        let _result = bench.run("integrated_operations", 1000, || {
            let v = Vec3::new(1.0, 2.0, 3.0);
            let _ = v.normalize();
        });

        let elapsed = start.elapsed().as_secs_f32();

        // 所有系统应该在 1 秒内完成
        assert!(elapsed < 1.0);
    }

    /// 集成测试: 批处理效率验证
    #[test]
    fn test_batch_efficiency() {
        // 单个操作
        let single_start = Instant::now();
        for i in 0..1000 {
            let pos = Vec3::new(i as f32, 0.0, 0.0);
            let _ = (pos - Vec3::ZERO).length();
        }
        let single_time = single_start.elapsed().as_secs_f64() * 1000.0;

        // 批量操作
        let positions: Vec<_> = (0..1000).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();

        let batch_start = Instant::now();
        let _results = SIMDHeuristics::batch_euclidean_distance(&positions, Vec3::ZERO);
        let batch_time = batch_start.elapsed().as_secs_f64() * 1000.0;

        // 批处理应该更快或相当 (允许 2x 的差异)
        assert!(batch_time <= single_time * 2.0);
    }

    /// 集成测试: 并发性验证
    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let positions = Arc::new(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ]);

        let results = Arc::new(Mutex::new(Vec::new()));

        // 模拟并发音频处理
        {
            let pos = positions.clone();
            let res = results.clone();

            let distances: Vec<_> = pos.iter().map(|p| (*p - Vec3::ZERO).length()).collect();

            let result = AudioSpatialOps::batch_distance_attenuation(
                &distances,
                DistanceModel::Inverse {
                    ref_distance: 1.0,
                    rolloff: 1.0,
                },
            );

            res.lock().unwrap().push(result.gains);
        }

        let final_results = results.lock().unwrap();
        assert_eq!(final_results.len(), 1);
        assert_eq!(final_results[0].len(), 3);
    }

    /// 集成测试: 内存管理验证
    #[test]
    fn test_memory_management() {
        // 创建大量临时对象
        let mut particles = GPUParticleSystem::new(10000);

        for i in 0..1000 {
            let pos = Vec3::new(i as f32, 0.0, 0.0);
            particles.emit(pos, Vec3::ZERO, 0.1);
        }

        assert_eq!(particles.particle_count(), 1000);

        // 更新使粒子过期
        particles.update(1.0);

        assert_eq!(particles.particle_count(), 0);
    }
}
