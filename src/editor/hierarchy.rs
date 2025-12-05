use crate::impl_default;
use bevy_ecs::prelude::*;

/// 场景层级视图
pub struct HierarchyView {
    pub selected_entity: Option<Entity>,
}

impl HierarchyView {
    pub fn new() -> Self {
        Self::default()
    }

    /// 渲染层级视图 (使用egui)
    pub fn render(&mut self, ui: &mut egui::Ui, world: &World) {
        ui.heading("Scene Hierarchy");
        ui.separator();

        // 获取所有实体
        let mut entities: Vec<_> = world.iter_entities().collect();
        entities.sort_by_key(|e| e.id());

        for entity_ref in entities {
            let entity = entity_ref.id();

            // 获取实体名称 (简化版)
            let name = "Entity";

            // 显示实体
            let is_selected = self.selected_entity == Some(entity);
            if ui
                .selectable_label(is_selected, format!("{} (ID: {:?})", name, entity))
                .clicked()
            {
                self.selected_entity = Some(entity);
            }
        }
    }
}

impl_default!(HierarchyView {
    selected_entity: None,
});
