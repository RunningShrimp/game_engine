use bevy_ecs::prelude::*;
use game_engine::ecs::{Transform, Velocity};
use glam::{Quat, Vec3};

fn main() {
    println!("=== 物理-变换批量同步示例 ===\n");

    let mut world = World::new();

    println!("1. 创建物理实体");
    println!("----------------");

    for i in 0..100 {
        let entity = world
            .spawn((
                Transform {
                    pos: Vec3::new(0.0, 10.0 + i as f32 * 0.5, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Velocity {
                    lin: Vec3::new(0.0, -1.0, 0.0),
                    ang: Vec3::ZERO,
                },
            ))
            .id();
        if i == 0 {
            println!("创建第一个实体 ID: {:?}", entity);
        }
    }

    println!("创建了 100 个物理实体\n");

    println!("2. 性能特性");
    println!("----------------");
    println!("批量同步优化:");
    println!("  - SoA (Structure of Arrays) 数据布局");
    println!("  - 缓存友好的内存访问");
    println!("  - SIMD 优化的距离计算");
    println!("  - 减少内存分配");

    println!("\n示例完成!");
    println!("在实际引擎中，批量同步系统将自动处理");
    println!("物理状态到Transform组件的同步。");
}
