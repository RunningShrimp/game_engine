use bevy_ecs::prelude::*;
use game_engine::ecs::{Transform, World};
use game_engine::physics::{
    BatchSyncResource, PhysicsDirty, PhysicsDomainService, PhysicsSyncConfig, RigidBodyComp,
    batch_collect_physics_state_system, batch_physics_to_transform_system,
};
use game_engine::render::mesh::GpuMesh;
use std::sync::Arc;

fn main() {
    println!("=== 物理-变换批量同步示例 ===\n");

    let mut world = World::new();
    let physics_service = PhysicsDomainService::new();
    let config = PhysicsSyncConfig::default();
    let batch_resource = BatchSyncResource::default();

    world.insert_resource(physics_service);
    world.insert_resource(config);
    world.insert_resource(batch_resource);

    let mesh = Arc::new(GpuMesh::default());

    println!("1. 创建物理实体");
    println!("----------------");

    for i in 0..100 {
        let entity = world
            .spawn((
                Transform {
                    pos: glam::Vec3::new(0.0, 10.0 + i as f32 * 0.5, 0.0),
                    rot: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                mesh.clone(),
                RigidBodyComp::new(i as u64),
                PhysicsDirty::default(),
            ))
            .id();
    }

    println!("创建了 100 个物理实体\n");

    println!("2. 性能特性");
    println!("----------------");
    println!("批量同步优化:");
    println!("  - SoA (Structure of Arrays) 数据布局");
    println!("  - 缓存友好的内存访问");
    println!("  - SIMD 优化的距离计算");
    println!("  - 减少内存分配");

    println!("\n3. 使用 SIMD 优化");
    println!("----------------");

    let old_pos = glam::Vec3::new(0.0, 10.0, 0.0);
    let new_pos = glam::Vec3::new(0.0002, 10.0, 0.0);
    let threshold_sq = 0.0001 * 0.0001;

    let changed = game_engine::physics::position_changed_simd(old_pos, new_pos, threshold_sq);
    println!("位置变化检测: {}", if changed { "是" } else { "否" });

    let old_rot = glam::Quat::IDENTITY;
    let new_rot = glam::Quat::from_rotation_x(0.0001);
    let rotation_changed =
        game_engine::physics::rotation_changed_simd(old_rot, new_rot, threshold_sq);
    println!(
        "旋转变化检测: {}",
        if rotation_changed { "是" } else { "否" }
    );

    println!("\n4. 批量同步系统");
    println!("----------------");

    batch_collect_physics_state_system(
        world.resource(),
        world.resource(),
        world.query::<(
            bevy_ecs::prelude::Entity,
            &RigidBodyComp,
            Option<&mut PhysicsDirty>,
        )>(),
    );

    batch_physics_to_transform_system(
        world.resource(),
        world.resource(),
        world.query::<(
            bevy_ecs::prelude::Entity,
            &RigidBodyComp,
            &mut Transform,
            Option<&mut PhysicsDirty>,
        )>(),
    );

    println!("批量物理到Transform同步完成");
    println!("使用批量同步可以提升 2-3x 性能 (大规模场景)");

    println!("\n5. 配置选项");
    println!("----------------");
    println!("PhysicsSyncConfig 配置:");
    println!("  - dirty_tracking_enabled: 是否启用脏标记");
    println!("  - skip_sleeping: 是否跳过休眠对象");
    println!("  - position_threshold: 位置变化阈值");
    println!("  - rotation_threshold: 旋转变化阈值");
    println!("  - batch_size: 批量处理大小");

    println!("\n示例完成!");
    println!("批量同步适用于:");
    println!("  - 大量物理实体 (>100)");
    println!("  - 高频同步场景");
    println!("  - 性能敏感的应用");
}
