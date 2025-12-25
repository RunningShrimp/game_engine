//! 编辑器模块单元测试

use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use game_engine::editor::{
    EditorState, Inspector, CommandManager, Command, CommandError,
    SceneEditorEnhanced, TransformGizmo, HierarchyView, GizmoMode,
};
use glam::{Vec3, Quat};

// 测试命令实现
#[derive(Debug)]
struct TestCommand {
    value: i32,
    executed: bool,
}

impl Command for TestCommand {
    fn execute(&mut self, _context: &mut dyn std::any::Any) -> Result<(), CommandError> {
        self.executed = true;
        Ok(())
    }

    fn undo(&mut self, _context: &mut dyn std::any::Any) -> Result<(), CommandError> {
        self.executed = false;
        Ok(())
    }

    fn description(&self) -> &str {
        "Test Command"
    }
}

#[test]
fn test_editor_state_new() {
    let state = EditorState::new();
    assert_eq!(state.scene_editor.selected_entities.len(), 0);
    assert_eq!(state.command_manager.max_history(), 100);
}

#[test]
fn test_editor_state_default() {
    let state = EditorState::default();
    assert_eq!(state.scene_editor.selected_entities.len(), 0);
}

#[test]
fn test_command_manager_new() {
    let manager = CommandManager::new(50);
    assert_eq!(manager.max_history(), 50);
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_command_manager_execute() {
    let mut manager = CommandManager::new(100);
    let mut context = ();
    
    let command = Box::new(TestCommand {
        value: 42,
        executed: false,
    });
    
    manager.execute(command, &mut context).unwrap();
    assert!(manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_command_manager_undo_redo() {
    let mut manager = CommandManager::new(100);
    let mut context = ();
    
    let command1 = Box::new(TestCommand {
        value: 1,
        executed: false,
    });
    let command2 = Box::new(TestCommand {
        value: 2,
        executed: false,
    });
    
    manager.execute(command1, &mut context).unwrap();
    manager.execute(command2, &mut context).unwrap();
    
    assert!(manager.can_undo());
    assert!(!manager.can_redo());
    
    // 撤销
    manager.undo(&mut context).unwrap();
    assert!(manager.can_undo());
    assert!(manager.can_redo());
    
    // 重做
    manager.redo(&mut context).unwrap();
    assert!(manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_command_manager_history_limit() {
    let mut manager = CommandManager::new(3);
    let mut context = ();
    
    // 执行超过限制的命令
    for i in 0..5 {
        let command = Box::new(TestCommand {
            value: i,
            executed: false,
        });
        manager.execute(command, &mut context).unwrap();
    }
    
    // 应该只保留最后3个命令
    let mut count = 0;
    while manager.can_undo() {
        manager.undo(&mut context).unwrap();
        count += 1;
    }
    
    assert_eq!(count, 3);
}

#[test]
fn test_scene_editor_enhanced_new() {
    let editor = SceneEditorEnhanced::new();
    assert_eq!(editor.selected_entities.len(), 0);
}

#[test]
fn test_scene_editor_enhanced_selection() {
    let mut editor = SceneEditorEnhanced::new();
    let mut world = World::new();
    
    let entity1 = world.spawn_empty().id();
    let entity2 = world.spawn_empty().id();
    
    // 添加选择
    editor.add_selection(entity1);
    assert_eq!(editor.selected_entities.len(), 1);
    assert!(editor.selected_entities.contains(&entity1));
    
    // 添加另一个选择
    editor.add_selection(entity2);
    assert_eq!(editor.selected_entities.len(), 2);
    
    // 移除选择
    editor.remove_selection(entity1);
    assert_eq!(editor.selected_entities.len(), 1);
    assert!(!editor.selected_entities.contains(&entity1));
    assert!(editor.selected_entities.contains(&entity2));
    
    // 清空选择
    editor.clear_selection();
    assert_eq!(editor.selected_entities.len(), 0);
}

#[test]
fn test_scene_editor_enhanced_toggle_selection() {
    let mut editor = SceneEditorEnhanced::new();
    let entity = Entity::from_raw(1);
    
    // 第一次切换：添加
    editor.toggle_selection(entity);
    assert!(editor.selected_entities.contains(&entity));
    
    // 第二次切换：移除
    editor.toggle_selection(entity);
    assert!(!editor.selected_entities.contains(&entity));
}

#[test]
fn test_hierarchy_view_new() {
    let view = HierarchyView::new();
    assert!(view.selected_entity.is_none());
}

#[test]
fn test_hierarchy_view_selection() {
    let mut view = HierarchyView::new();
    let entity = Entity::from_raw(1);
    
    // 选择实体
    view.selected_entity = Some(entity);
    assert_eq!(view.selected_entity, Some(entity));
    
    // 清除选择
    view.selected_entity = None;
    assert!(view.selected_entity.is_none());
}

#[test]
fn test_transform_gizmo_new() {
    let gizmo = TransformGizmo::new();
    // 验证默认状态
    assert_eq!(gizmo.mode, GizmoMode::Translate);
    assert!(gizmo.selected_axis.is_none());
}

#[test]
fn test_inspector_new() {
    let inspector = Inspector::default();
    // Inspector主要是渲染功能，这里只测试创建
    assert!(true); // 占位测试
}

#[test]
fn test_command_error_display() {
    let error = CommandError::ExecutionFailed("Test error".to_string());
    assert_eq!(error.to_string(), "Execution failed: Test error");
    
    let error = CommandError::UndoFailed("Undo error".to_string());
    assert_eq!(error.to_string(), "Undo failed: Undo error");
    
    let error = CommandError::CannotMerge;
    assert_eq!(error.to_string(), "Commands cannot be merged");
    
    let error = CommandError::InvalidState("Invalid".to_string());
    assert_eq!(error.to_string(), "Invalid state: Invalid");
}

#[test]
fn test_command_manager_clear() {
    let mut manager = CommandManager::new(100);
    let mut context = ();
    
    // 执行一些命令
    for i in 0..5 {
        let command = Box::new(TestCommand {
            value: i,
            executed: false,
        });
        manager.execute(command, &mut context).unwrap();
    }
    
    assert!(manager.can_undo());
    
    // 清空
    manager.clear();
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_scene_editor_enhanced_copy_paste() {
    let mut editor = SceneEditorEnhanced::new();
    let mut world = World::new();
    
    // 创建实体
    let entity = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(1.0, 2.0, 3.0)))
        .id();
    
    // 选择实体
    editor.add_selection(entity);
    
    // 复制
    editor.copy_selected(&world);
    assert!(!editor.clipboard.is_empty());
    
    // 粘贴
    let pasted = editor.paste(&mut world);
    assert_eq!(pasted.len(), 1);
    
    // 验证粘贴的实体位置
    let pasted_entity = pasted[0];
    let transform = world.get::<Transform>(pasted_entity).unwrap();
    assert_eq!(transform.pos, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_scene_editor_enhanced_duplicate() {
    let mut editor = SceneEditorEnhanced::new();
    let mut world = World::new();
    
    // 创建实体
    let entity = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(0.0, 0.0, 0.0)))
        .id();
    
    // 选择实体
    editor.add_selection(entity);
    
    // 复制
    let duplicated = editor.duplicate_selected(&mut world);
    assert_eq!(duplicated.len(), 1);
    
    // 验证复制的实体位置（应该偏移了）
    let duplicated_entity = duplicated[0];
    let transform = world.get::<Transform>(duplicated_entity).unwrap();
    assert_eq!(transform.pos, Vec3::new(1.0, 0.0, 0.0));
}

