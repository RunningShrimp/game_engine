//! 增强的场景编辑器功能
//!
//! 提供多选、复制粘贴等高级编辑功能

use crate::ecs::Transform;
use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 实体数据（用于复制粘贴）
#[derive(Debug, Clone, Serialize, Deserialize)]
/// 实体数据（用于复制粘贴）
///
/// 当前实现仅序列化Transform组件，其他组件数据序列化功能待实现。
/// 未来计划：使用serde完整序列化所有组件数据。
pub struct EntityData {
    /// 实体的变换组件
    pub transform: Transform,
    /// 组件类型名称列表（当前未包含实际数据）
    pub components: Vec<String>, // 组件类型名称
}

/// 增强的场景编辑器
pub struct SceneEditorEnhanced {
    /// 基础场景编辑器数据
    pub base: crate::editor::scene_editor::SceneEditor,
    /// 选中的实体集合（多选）
    pub selected_entities: HashSet<Entity>,
    /// 剪贴板（存储复制的实体数据）
    pub clipboard: Vec<EntityData>,
    /// 是否启用多选模式
    pub multi_select_enabled: bool,
    /// 框选起始位置
    pub selection_box_start: Option<egui::Pos2>,
    /// 框选结束位置
    pub selection_box_end: Option<egui::Pos2>,
}

impl SceneEditorEnhanced {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加选中实体
    pub fn add_selection(&mut self, entity: Entity) {
        if self.multi_select_enabled {
            self.selected_entities.insert(entity);
        } else {
            self.selected_entities.clear();
            self.selected_entities.insert(entity);
        }
        self.base.selected_entity = Some(entity);
    }

    /// 移除选中实体
    pub fn remove_selection(&mut self, entity: Entity) {
        self.selected_entities.remove(&entity);
        if self.base.selected_entity == Some(entity) {
            self.base.selected_entity = self.selected_entities.iter().next().copied();
        }
    }

    /// 清除所有选中
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
        self.base.selected_entity = None;
    }

    /// 切换实体选中状态
    pub fn toggle_selection(&mut self, entity: Entity) {
        if self.selected_entities.contains(&entity) {
            self.remove_selection(entity);
        } else {
            self.add_selection(entity);
        }
    }

    /// 复制选中的实体
    pub fn copy_selected(&mut self, world: &World) {
        self.clipboard.clear();

        for entity in &self.selected_entities {
            if let Some(entity_data) = self.serialize_entity(*entity, world) {
                self.clipboard.push(entity_data);
            }
        }
    }

    /// 粘贴实体
    pub fn paste(&mut self, world: &mut World) -> Vec<Entity> {
        let mut pasted_entities = Vec::new();

        for entity_data in &self.clipboard {
            if let Some(new_entity) = self.deserialize_entity(entity_data, world) {
                pasted_entities.push(new_entity);
            }
        }

        // 选中粘贴的实体
        self.selected_entities.clear();
        for entity in &pasted_entities {
            self.selected_entities.insert(*entity);
        }

        if let Some(first) = pasted_entities.first() {
            self.base.selected_entity = Some(*first);
        }

        pasted_entities
    }

    /// 删除选中的实体
    pub fn delete_selected(&mut self, world: &mut World) {
        for entity in &self.selected_entities {
            if let Some(mut entity_mut) = world.get_entity_mut(*entity) {
                entity_mut.despawn();
            }
        }
        self.clear_selection();
    }

    /// 复制选中的实体（创建副本）
    pub fn duplicate_selected(&mut self, world: &mut World) -> Vec<Entity> {
        self.copy_selected(world);

        // 稍微偏移位置
        for entity_data in &mut self.clipboard {
            entity_data.transform.pos += Vec3::new(1.0, 0.0, 0.0);
        }

        self.paste(world)
    }

    /// 序列化实体数据
    fn serialize_entity(&self, entity: Entity, world: &World) -> Option<EntityData> {
        // 获取 Transform 组件
        let transform = world.get::<Transform>(entity).cloned()?;

        // 获取所有组件类型（简化版）
        // 注意：当前实现仅记录组件类型名称，不序列化实际数据
        // 未来计划：使用serde完整序列化所有组件数据
        let components = Vec::new();

        Some(EntityData {
            transform,
            components,
        })
    }

    /// 反序列化实体数据
    fn deserialize_entity(&self, data: &EntityData, world: &mut World) -> Option<Entity> {
        let entity = world.spawn_empty().id();

        // 添加 Transform 组件
        world.entity_mut(entity).insert(data.transform.clone());

        // 注意：当前实现仅恢复Transform组件
        // 未来计划：根据components列表恢复所有组件数据

        Some(entity)
    }

    /// 开始框选
    pub fn start_box_selection(&mut self, pos: egui::Pos2) {
        self.selection_box_start = Some(pos);
        self.selection_box_end = Some(pos);
    }

    /// 更新框选
    pub fn update_box_selection(&mut self, pos: egui::Pos2) {
        self.selection_box_end = Some(pos);
    }

    /// 完成框选
    pub fn finish_box_selection(&mut self, world: &mut World) {
        if let (Some(start), Some(end)) = (self.selection_box_start, self.selection_box_end) {
            let rect = egui::Rect::from_two_pos(start, end);

            // 清除当前选中（如果按住 Shift 则保留）
            // 这里简化处理，总是清除
            if !self.multi_select_enabled {
                self.clear_selection();
            }

            // 查找框选范围内的实体
            let mut query = world.query::<(Entity, &Transform)>();
            for (entity, transform) in query.iter(world) {
                let screen_pos = self.base.world_to_screen(transform.pos, rect);
                if rect.contains(screen_pos) {
                    self.add_selection(entity);
                }
            }
        }

        self.selection_box_start = None;
        self.selection_box_end = None;
    }

    /// 绘制框选矩形
    pub fn draw_selection_box(&self, painter: &egui::Painter, rect: egui::Rect) {
        if let (Some(start), Some(end)) = (self.selection_box_start, self.selection_box_end) {
            let selection_rect = egui::Rect::from_two_pos(start, end);
            painter.rect_stroke(
                selection_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            );
            painter.rect_filled(
                selection_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(100, 150, 255, 30),
            );
        }
    }

    /// 获取选中实体的中心点
    pub fn get_selection_center(&self, world: &mut World) -> Option<Vec3> {
        if self.selected_entities.is_empty() {
            return None;
        }

        let mut query = world.query::<&Transform>();
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for entity in &self.selected_entities {
            if let Ok(transform) = query.get(world, *entity) {
                sum += transform.pos;
                count += 1;
            }
        }

        if count > 0 {
            Some(sum / count as f32)
        } else {
            None
        }
    }

    /// 移动选中的实体
    pub fn move_selected(&mut self, world: &mut World, delta: Vec3) {
        let mut query = world.query::<&mut Transform>();
        for entity in &self.selected_entities {
            if let Ok(mut transform) = query.get_mut(world, *entity) {
                transform.pos += delta;
            }
        }
    }
}

impl Default for SceneEditorEnhanced {
    fn default() -> Self {
        Self {
            base: crate::editor::scene_editor::SceneEditor::new(),
            selected_entities: HashSet::new(),
            clipboard: Vec::new(),
            multi_select_enabled: true,
            selection_box_start: None,
            selection_box_end: None,
        }
    }
}
