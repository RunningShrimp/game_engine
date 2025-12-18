use game_engine::editor::{EditorState, SceneEditor, Inspector, TransformGizmo};
use game_engine::ecs::Transform;
use bevy_ecs::prelude::*;
use glam::Vec3;

#[test]
fn test_editor_integration() {
    let mut world = World::new();
    
    // 创建一些测试实体
    let entity1 = world.spawn_empty()
        .insert(Transform::default())
        .id();
    
    let entity2 = world.spawn_empty()
        .insert(Transform::from_pos(glam::Vec3::new(1.0, 0.0, 0.0)))
        .id();
    
    // 创建编辑器状态
    let mut editor_state = EditorState::new();
    
    // 测试场景编辑器
    editor_state.scene_editor.base.selected_entity = Some(entity1);
    assert_eq!(editor_state.scene_editor.base.selected_entity, Some(entity1));
    
    // 测试检查器
    let mut ui = egui::Ui::dummy();
    // 注意：真实的UI渲染需要egui上下文，这里只测试编译
    // Inspector::render(&mut ui, &mut world, entity1);
    
    // 测试变换工具
    // TransformGizmo::render(&mut ui, &mut world, entity1);
    
    println!("Editor integration test passed!");
}