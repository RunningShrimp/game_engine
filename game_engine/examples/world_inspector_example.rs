use game_engine::ecs::{Transform, World};
use game_engine::editor::WorldInspector;
use game_engine::render::mesh::GpuMesh;
use std::sync::Arc;

fn main() {
    println!("=== World Inspector 示例 ===\n");

    let mut world = World::new();
    let mut inspector = WorldInspector::default();

    let mesh = Arc::new(GpuMesh::default());

    for i in 0..10 {
        world.spawn((
            Transform {
                pos: glam::Vec3::new(i as f32, 0.0, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
            },
            mesh.clone(),
        ));
    }

    println!("创建了 {} 个实体\n", world.iter_entities().count());

    println!("World Inspector 功能:");
    println!("  - 实时查看所有实体和组件");
    println!("  - 筛选和搜索实体");
    println!("  - 查看和编辑组件属性");
    println!("  - 选择和查看实体详情");
    println!("\n在编辑器中打开 World Inspector 查看更多功能");
}
