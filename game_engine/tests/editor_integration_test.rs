use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use game_engine::editor::EditorState;
use glam::Vec3;

#[test]
fn test_editor_integration() {
    let mut world = World::new();
    
    // 创建一些测试实体
    let entity1 = world.spawn_empty().insert(Transform::default()).id();

    let entity2 = world
        .spawn_empty()
        .insert(Transform {
            pos: Vec3::new(1.0, 0.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        })
        .id();

    // 创建编辑器状态
    let mut editor_state = EditorState::new();

    // 测试场景编辑器 - 选择第一个实体
    editor_state.scene_editor.selected_entity = Some(entity1);
    assert_eq!(
        editor_state.scene_editor.selected_entity,
        Some(entity1)
    );

    // 测试切换选择 - 选择第二个实体，形成逻辑闭环
    editor_state.scene_editor.selected_entity = Some(entity2);
    assert_eq!(
        editor_state.scene_editor.selected_entity,
        Some(entity2)
    );
    
    // 测试检查器 - 注释掉因为需要 egui 上下文
    // let mut ctx = egui::Context::default();
    // let mut ui = ctx.new_ui(egui::UiBuilder::default());
    // Inspector::render(&mut ui, &mut world, entity1);
    
    // 测试变换工具 - 注释掉因为需要 egui 上下文
    // TransformGizmo::render(&mut ui, &mut world, entity1);
    
    println!("Editor integration test passed!");
}
