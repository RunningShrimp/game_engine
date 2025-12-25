//! 编辑器模块集成测试

use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use game_engine::editor::{
    EditorState, Inspector, SceneEditorEnhanced, TransformGizmo,
    CommandManager, Command, CommandError, HierarchyView,
};
use glam::{Vec3, Quat};

// 测试命令：移动实体
#[derive(Debug)]
struct MoveEntityCommand {
    entity: Entity,
    old_position: Vec3,
    new_position: Vec3,
}

impl Command for MoveEntityCommand {
    fn execute(&mut self, context: &mut dyn std::any::Any) -> Result<(), CommandError> {
        if let Some(world) = context.downcast_mut::<World>() {
            if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
                transform.pos = self.new_position;
                Ok(())
            } else {
                Err(CommandError::ExecutionFailed("Entity not found".to_string()))
            }
        } else {
            Err(CommandError::ExecutionFailed("Invalid context".to_string()))
        }
    }

    fn undo(&mut self, context: &mut dyn std::any::Any) -> Result<(), CommandError> {
        if let Some(world) = context.downcast_mut::<World>() {
            if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
                transform.pos = self.old_position;
                Ok(())
            } else {
                Err(CommandError::UndoFailed("Entity not found".to_string()))
            }
        } else {
            Err(CommandError::UndoFailed("Invalid context".to_string()))
        }
    }

    fn description(&self) -> &str {
        "Move Entity"
    }
}

#[test]
fn test_editor_state_integration() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();

    // 创建实体
    let entity1 = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(1.0, 2.0, 3.0)))
        .id();
    
    let entity2 = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(4.0, 5.0, 6.0)))
        .id();

    // 测试场景编辑器选择
    editor_state.scene_editor.add_selection(entity1);
    assert!(editor_state.scene_editor.selected_entities.contains(&entity1));

    // 测试层级视图
    editor_state.hierarchy_view.selected_entity = Some(entity1);
    assert_eq!(editor_state.hierarchy_view.selected_entity, Some(entity1));

    // 测试命令管理器
    let old_pos = world.get::<Transform>(entity1).unwrap().pos;
    let new_pos = Vec3::new(10.0, 20.0, 30.0);
    
    let command = Box::new(MoveEntityCommand {
        entity: entity1,
        old_position: old_pos,
        new_position: new_pos,
    });
    
    editor_state.command_manager.execute(command, &mut world).unwrap();
    
    // 验证位置已更新
    let transform = world.get::<Transform>(entity1).unwrap();
    assert_eq!(transform.pos, new_pos);
    
    // 撤销
    editor_state.command_manager.undo(&mut world).unwrap();
    let transform = world.get::<Transform>(entity1).unwrap();
    assert_eq!(transform.pos, old_pos);
}

#[test]
fn test_editor_workflow() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();

    // 1. 创建实体
    let entity = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::ZERO))
        .id();

    // 2. 选择实体
    editor_state.scene_editor.add_selection(entity);
    assert_eq!(editor_state.scene_editor.selected_entities.len(), 1);

    // 3. 移动实体（通过命令）
    let old_pos = Vec3::ZERO;
    let new_pos = Vec3::new(5.0, 5.0, 5.0);
    
    let command = Box::new(MoveEntityCommand {
        entity,
        old_position: old_pos,
        new_position: new_pos,
    });
    
    editor_state.command_manager.execute(command, &mut world).unwrap();
    
    // 4. 验证位置
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.pos, new_pos);

    // 5. 撤销操作
    editor_state.command_manager.undo(&mut world).unwrap();
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.pos, old_pos);

    // 6. 重做操作
    editor_state.command_manager.redo(&mut world).unwrap();
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.pos, new_pos);
}

#[test]
fn test_editor_multi_entity_selection() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();

    // 启用多选模式
    editor_state.scene_editor.multi_select_enabled = true;

    // 创建多个实体
    let entities: Vec<Entity> = (0..5)
        .map(|i| {
            world.spawn_empty()
                .insert(Transform::from_pos(Vec3::new(i as f32, 0.0, 0.0)))
                .id()
        })
        .collect();

    // 选择所有实体
    for entity in &entities {
        editor_state.scene_editor.add_selection(*entity);
    }

    assert_eq!(editor_state.scene_editor.selected_entities.len(), 5);

    // 批量移动所有选中的实体
    for entity in &entities {
        let old_pos = world.get::<Transform>(*entity).unwrap().pos;
        let new_pos = old_pos + Vec3::new(10.0, 0.0, 0.0);
        
        let command = Box::new(MoveEntityCommand {
            entity: *entity,
            old_position: old_pos,
            new_position: new_pos,
        });
        
        editor_state.command_manager.execute(command, &mut world).unwrap();
    }

    // 验证所有实体都已移动
    for (i, entity) in entities.iter().enumerate() {
        let transform = world.get::<Transform>(*entity).unwrap();
        assert_eq!(transform.pos.x, (i as f32) + 10.0);
    }

    // 撤销所有操作
    for _ in 0..5 {
        editor_state.command_manager.undo(&mut world).unwrap();
    }

    // 验证所有实体已恢复
    for (i, entity) in entities.iter().enumerate() {
        let transform = world.get::<Transform>(*entity).unwrap();
        assert_eq!(transform.pos.x, i as f32);
    }
}

#[test]
fn test_editor_hierarchy_expansion() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();

    // 创建父子实体
    let parent = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(0.0, 0.0, 0.0)))
        .id();
    
    let child1 = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(1.0, 0.0, 0.0)))
        .id();
    
    let child2 = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(2.0, 0.0, 0.0)))
        .id();

    // 选择父实体
    editor_state.hierarchy_view.selected_entity = Some(parent);
    assert_eq!(editor_state.hierarchy_view.selected_entity, Some(parent));

    // 选择父实体和子实体
    editor_state.scene_editor.add_selection(parent);
    editor_state.scene_editor.add_selection(child1);
    editor_state.scene_editor.add_selection(child2);

    assert_eq!(editor_state.scene_editor.selected_entities.len(), 3);
}

#[test]
fn test_editor_command_history_limit() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();
    
    // 设置较小的历史限制
    editor_state.command_manager.set_max_history(3);

    let entity = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::ZERO))
        .id();

    // 执行超过限制的命令
    for i in 0..5 {
        let old_pos = if i == 0 {
            Vec3::ZERO
        } else {
            Vec3::new((i - 1) as f32, 0.0, 0.0)
        };
        let new_pos = Vec3::new(i as f32, 0.0, 0.0);
        
        let command = Box::new(MoveEntityCommand {
            entity,
            old_position: old_pos,
            new_position: new_pos,
        });
        
        editor_state.command_manager.execute(command, &mut world).unwrap();
    }

    // 应该只能撤销最后3个命令
    let mut undo_count = 0;
    while editor_state.command_manager.can_undo() {
        editor_state.command_manager.undo(&mut world).unwrap();
        undo_count += 1;
    }

    assert_eq!(undo_count, 3);
}

#[test]
fn test_editor_selection_clearing() {
    let mut world = World::new();
    let mut editor_state = EditorState::new();

    // 创建并选择多个实体
    let entities: Vec<Entity> = (0..3)
        .map(|_| world.spawn_empty().id())
        .collect();

    for entity in &entities {
        editor_state.scene_editor.add_selection(*entity);
    }

    assert_eq!(editor_state.scene_editor.selected_entities.len(), 3);

    // 清空选择
    editor_state.scene_editor.clear_selection();
    assert_eq!(editor_state.scene_editor.selected_entities.len(), 0);
}

#[test]
fn test_editor_transform_gizmo_integration() {
    let mut world = World::new();
    let gizmo = TransformGizmo::new();

    let entity = world.spawn_empty()
        .insert(Transform::from_pos(Vec3::new(1.0, 2.0, 3.0)))
        .id();

    // 测试gizmo默认状态
    assert_eq!(gizmo.mode, GizmoMode::Translate);
    assert!(gizmo.selected_axis.is_none());
}

