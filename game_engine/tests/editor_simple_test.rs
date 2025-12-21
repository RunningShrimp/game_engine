use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use game_engine::editor::{EditorState, SceneEditor, SceneEditorEnhanced, ViewMode};
use glam::Vec3;

#[test]
fn test_editor_basics() {
    let mut world = World::new();

    // 测试场景编辑器基础功能
    let mut scene_editor = SceneEditor::new();
    assert_eq!(scene_editor.view_mode, ViewMode::Perspective);
    assert_eq!(scene_editor.camera_position, Vec3::new(0.0, 5.0, 10.0));

    // 创建一个实体
    let entity = world
        .spawn_empty()
        .insert(Transform::from_pos(Vec3::new(1.0, 2.0, 3.0)))
        .id();

    // 测试增强场景编辑器
    let mut enhanced_editor = SceneEditorEnhanced::new();
    enhanced_editor.add_selection(entity);
    assert!(enhanced_editor.selected_entities.contains(&entity));

    println!("Editor basic functionality test passed!");
}
