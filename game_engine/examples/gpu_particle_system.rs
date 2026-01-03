//! GPU粒子系统示例
//!
//! 演示如何使用GPU粒子系统创建各种粒子效果。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example gpu_particle_system
//! ```

use game_engine::render::gpu_particles::{
    EmitterShape, EmitterType, ForceField, ForceFieldType, GpuParticleSystem, ParticleEmitter,
    ParticleId,
};
use glam::Vec3;

fn main() {
    // 初始化日志
    env_logger::init();

    println!("=== GPU粒子系统示例 ===\n");

    // 创建粒子系统
    let mut particle_system = GpuParticleSystem::new();

    println!("✓ 创建GPU粒子系统");

    // ========================================
    // 示例1: 创建火焰发射器
    // ========================================
    println!("\n--- 示例1: 火焰效果 ---");

    let fire_emitter_id = ParticleId::new(1);
    let mut fire_emitter = ParticleEmitter::new(fire_emitter_id, "Fire".to_string());

    // 配置火焰发射器
    fire_emitter.shape = EmitterShape {
        emitter_type: EmitterType::Point,
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: glam::Quat::IDENTITY,
        size: Vec3::ONE,
    };
    fire_emitter.emission_rate = 500.0; // 每秒500个粒子
    fire_emitter.lifetime = 2.0; // 2秒生命周期
    fire_emitter.velocity_range = (2.0, 5.0); // 向上速度
    fire_emitter.size_range = (0.2, 0.5);
    fire_emitter.color = Vec3::new(1.0, 0.5, 0.0).extend(1.0); // 橙色

    particle_system.add_emitter(fire_emitter);

    // 发射粒子
    for _ in 0..10 {
        particle_system.emit_particles(
            fire_emitter_id,
            50,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0), // 向上发射
            2.0,
        );
    }

    println!(
        "✓ 火焰发射器创建成功，当前粒子数: {}",
        particle_system.total_particles()
    );

    // ========================================
    // 示例2: 创建喷泉效果
    // ========================================
    println!("\n--- 示例2: 喷泉效果 ---");

    let fountain_emitter_id = ParticleId::new(2);
    let mut fountain_emitter = ParticleEmitter::new(fountain_emitter_id, "Fountain".to_string());

    fountain_emitter.shape = EmitterShape {
        emitter_type: EmitterType::Circle,
        position: Vec3::new(5.0, 0.0, 0.0),
        rotation: glam::Quat::IDENTITY,
        size: Vec3::new(1.0, 1.0, 1.0),
    };
    fountain_emitter.emission_rate = 300.0;
    fountain_emitter.lifetime = 3.0;
    fountain_emitter.velocity_range = (5.0, 8.0);
    fountain_emitter.color = Vec3::new(0.5, 0.8, 1.0).extend(0.7); // 淡蓝色

    particle_system.add_emitter(fountain_emitter);

    // 发射喷泉粒子
    for i in 0..20 {
        let angle = (i as f32 / 20.0) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * 0.5;
        let z = angle.sin() * 0.5;

        particle_system.emit_particles(
            fountain_emitter_id,
            10,
            Vec3::new(5.0 + x, 0.0, z),
            Vec3::new(x * 2.0, 8.0, z * 2.0),
            3.0,
        );
    }

    println!(
        "✓ 喷泉发射器创建成功，当前粒子数: {}",
        particle_system.total_particles()
    );

    // ========================================
    // 示例3: 添加力场
    // ========================================
    println!("\n--- 示例3: 力场系统 ---");

    // 添加重力
    let gravity = ForceField::gravity(9.81);
    particle_system.add_force_field(gravity);
    println!("✓ 添加重力场 (9.81 m/s²)");

    // 添加风力
    let wind = ForceField::wind(Vec3::new(1.0, 0.0, 0.0), 2.0);
    particle_system.add_force_field(wind);
    println!("✓ 添加风力场 (向右, 2.0 m/s)");

    // 添加吸引力
    let attraction = ForceField::attraction(Vec3::new(10.0, 5.0, 0.0), 5.0, 5.0);
    particle_system.add_force_field(attraction);
    println!("✓ 添加吸引力场 (中心: (10, 5, 0), 强度: 5.0, 半径: 5.0)");

    // ========================================
    // 示例4: 模拟更新
    // ========================================
    println!("\n--- 示例4: 粒子模拟 ---");

    let mut frame_count = 0;
    let total_frames = 60;

    while frame_count < total_frames {
        let delta_time = 0.016; // 60 FPS

        // 每帧发射新粒子
        if frame_count % 5 == 0 {
            particle_system.emit_particles(
                fire_emitter_id,
                10,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 3.0, 0.0),
                2.0,
            );
        }

        // 更新粒子系统
        particle_system.update(delta_time);

        // 每10帧打印一次状态
        if frame_count % 10 == 0 {
            println!(
                "帧 {}: {} 个活跃粒子",
                frame_count,
                particle_system.total_particles()
            );
        }

        frame_count += 1;
    }

    println!("\n✓ 模拟完成 (60帧)");

    // ========================================
    // 示例5: 粒子数据访问
    // ========================================
    println!("\n--- 示例5: 粒子数据访问 ---");

    let particles = particle_system.get_particle_data();
    println!("✓ 获取粒子数据: {} 个粒子", particles.len());

    if !particles.is_empty() {
        println!("  第一个粒子:");
        println!(
            "    位置: ({:.2}, {:.2}, {:.2})",
            particles[0].position[0], particles[0].position[1], particles[0].position[2]
        );
        println!(
            "    速度: ({:.2}, {:.2}, {:.2})",
            particles[0].velocity[0], particles[0].velocity[1], particles[0].velocity[2]
        );
        println!("    生命周期: {:.2}", particles[0].lifetime);
    }

    // ========================================
    // 示例6: 清除粒子
    // ========================================
    println!("\n--- 示例6: 清除粒子 ---");

    println!("清除前: {} 个粒子", particle_system.total_particles());
    particle_system.clear_particles();
    println!("清除后: {} 个粒子", particle_system.total_particles());

    // ========================================
    // 性能统计
    // ========================================
    println!("\n--- 性能统计 ---");
    println!("发射器数量: {}", particle_system.emitter_count());
    println!("GPU可用: {}", particle_system.is_gpu_available());

    println!("\n=== 示例完成 ===");
}
