use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use glam::{Quat, Vec3};

fn main() {
    println!("=== World Inspector 示例 ===\n");

    let mut world = World::new();

    for i in 0..10 {
        world.spawn((Transform {
            pos: Vec3::new(i as f32, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        },));
    }

    println!(
        "创建了 {} 个实体\n",
        world.query::<EntityRef>().iter(&world).count()
    );

    println!("World Inspector 功能:");
    println!("  - 实时查看所有实体和组件");
    println!("  - 筛选和搜索实体");
    println!("  - 查看和编辑组件属性");
    println!("  - 选择和查看实体详情");

    println!("\n示例完成!");
    println!("World Inspector 在引擎编辑器中提供完整功能");
}
