//! # ECS基础示例
//!
//! 此示例展示如何使用ECS系统创建基本的游戏场景。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example ecs_basics
//! ```

use bevy_ecs::prelude::*;
use game_engine::prelude::*;
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建ECS World
    let mut world = World::new();

    // 创建Resources
    let mut resources = Resources::default();
    resources.insert(Time {
        delta_seconds: 0.016,
        elapsed_seconds: 0.0,
        fixed_time_step: 1.0 / 60.0,
        alpha: 0.0,
    });
    resources.insert(Viewport {
        width: 800,
        height: 600,
    });

    // 创建Schedule
    let mut schedule = Schedule::default();
    schedule.add_stage(
        "update",
        SystemStage::parallel()
            .with_system(spawn_entities.system())
            .with_system(print_positions.system())
            .with_system(rotate_system.system()),
    );

    // 运行几个帧
    println!("=== ECS Basics Example ===\n");

    for frame in 0..5 {
        println!("--- Frame {} ---", frame);

        // 更新时间
        let mut time = resources.get_mut::<Time>().unwrap();
        time.elapsed_seconds += time.delta_seconds;
        drop(time);

        // 运行系统
        schedule.run(&mut world, &mut resources);

        println!();
    }

    println!("=== Total entities: {} ===", world.iter::<Entity>().count());

    Ok(())
}

/// 创建初始实体
fn spawn_entities(mut commands: Commands) {
    println!("Spawning entities...");

    // 创建玩家实体
    commands.spawn((
        Transform {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Velocity {
            lin: Vec3::new(1.0, 0.0, 0.0),
            ang: Vec3::ZERO,
        },
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0], // 红色
            tex_index: 0,
            normal_tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        },
    ));

    // 创建敌人实体
    for i in 0..3 {
        commands.spawn((
            Transform {
                pos: Vec3::new(i as f32 * 2.0, 1.0, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Velocity {
                lin: Vec3::new(-0.5, 0.0, 0.0),
                ang: Vec3::ZERO,
            },
            Sprite {
                color: [0.0, 1.0, 0.0, 1.0], // 绿色
                tex_index: 0,
                normal_tex_index: 0,
                uv_off: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
                layer: 0.0,
            },
        ));
    }

    println!("Spawned 4 entities");
}

/// 打印所有实体位置
fn print_positions(query: Query<(Entity, &Transform, &Velocity)>) {
    for (entity, transform, velocity) in query.iter() {
        println!(
            "Entity {:?}: pos=({:.2}, {:.2}, {:.2}) vel=({:.2}, {:.2}, {:.2})",
            entity,
            transform.pos.x,
            transform.pos.y,
            transform.pos.z,
            velocity.lin.x,
            velocity.lin.y,
            velocity.lin.z
        );
    }
}

/// 旋转系统 - 更新实体位置
fn rotate_system(mut query: Query<&mut Transform>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.pos.x += transform.pos.x * time.delta_seconds;
    }
}
