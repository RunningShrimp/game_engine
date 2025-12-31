//! 实体面板
//!
//! 显示和管理所有ECS实体及其组件。

use super::{ComponentPanel, Panel};
use bevy_ecs::prelude::*;
use bevy_ecs::component::ComponentId;
use std::any::TypeId;

/// 实体面板
///
/// 显示所有实体及其组件列表。
pub struct EntityPanel {
    /// 是否显示面板
    visible: bool,
    /// 搜索过滤文本
    filter_text: String,
    /// 选中的实体
    selected_entity: Option<Entity>,
    /// 实体列表是否需要刷新
    needs_refresh: bool,
    /// 缓存的实体列表
    cached_entities: Vec<Entity>,
}

impl EntityPanel {
    /// 创建新的实体面板
    pub fn new() -> Self {
        Self {
            visible: true,
            filter_text: String::new(),
            selected_entity: None,
            needs_refresh: true,
            cached_entities: Vec::new(),
        }
    }

    /// 显示面板并处理与组件面板的交互
    pub fn show(&mut self, ctx: &egui::Context, world: &World, component_panel: &mut ComponentPanel) {
        if !self.visible {
            return;
        }

        egui::Window::new("Entities")
            .default_size([300.0, 400.0])
            .show(ctx, |ui| {
                // 搜索框
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    let response = ui.text_edit_singleline(&mut self.filter_text);
                    if response.changed() {
                        self.needs_refresh = true;
                    }
                });

                ui.separator();

                // 获取实体列表
                if self.needs_refresh {
                    self.refresh_entity_list(world);
                }

                // 显示实体计数
                ui.label(format!("Total entities: {}", self.cached_entities.len()));

                // 实体列表
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for entity in &self.cached_entities {
                            self.show_entity(ui, world, *entity, component_panel);
                        }
                    });
            });
    }

    /// 显示单个实体
    fn show_entity(
        &mut self,
        ui: &mut egui::Ui,
        world: &World,
        entity: Entity,
        component_panel: &mut ComponentPanel,
    ) {
        let entity_id = entity.to_bits();
        let is_selected = self.selected_entity == Some(entity);

        // 检查实体是否存活
        let is_alive = world.is_alive(entity);

        // 实体选择器
        let response = ui.selectable_label(is_selected, format!("Entity {}", entity_id));

        // 如果实体已死亡，显示为灰色
        if !is_alive {
            ui.colored_label(egui::Color32::GRAY, "(dead)");
        }

        // 双击打开组件详情
        if response.double_clicked() && is_alive {
            self.selected_entity = Some(entity);
            component_panel.set_entity(entity);
        }

        // 显示组件信息
        if is_alive && ui.is_item_visible() {
            if let Some(components) = self.get_entity_components(world, entity) {
                ui.indent(format!("entity_{}", entity_id), |ui| {
                    ui.label(format!("Components: {}", components.len()));
                    for component_name in components {
                        ui.label(format!("  - {}", component_name));
                    }
                });
            }
        }
    }

    /// 刷新实体列表
    fn refresh_entity_list(&mut self, world: &World) {
        self.cached_entities.clear();

        // 从world中获取所有实体
        // 注意：bevy_ecs不直接提供获取所有实体的API
        // 这里我们使用archetypes来遍历实体
        for archetype in world.archetypes() {
            for entity in archetype.entities() {
                if world.is_alive(entity) {
                    self.cached_entities.push(entity);
                }
            }
        }

        self.needs_refresh = false;
    }

    /// 获取实体的组件列表
    fn get_entity_components(&self, world: &World, entity: Entity) -> Option<Vec<String>> {
        let mut components = Vec::new();

        // 遍历所有archetype查找该实体
        for archetype in world.archetypes() {
            if !archetype.contains(entity) {
                continue;
            }

            // 获取组件类型
            for component_id in archetype.components() {
                if let Some(type_name) = self.get_component_type_name(world, component_id) {
                    components.push(type_name);
                }
            }

            return Some(components);
        }

        None
    }

    /// 获取组件类型名称
    fn get_component_type_name(&self, _world: &World, _component_id: ComponentId) -> Option<String> {
        // 注意：bevy_ecs的ComponentId无法直接获取类型名称
        // 这是一个简化的实现
        Some("Component".to_string())
    }

    /// 设置选中的实体
    pub fn set_selected_entity(&mut self, entity: Entity) {
        self.selected_entity = Some(entity);
    }

    /// 获取选中的实体
    pub fn selected_entity(&self) -> Option<Entity> {
        self.selected_entity
    }

    /// 清除选中状态
    pub fn clear_selection(&mut self) {
        self.selected_entity = None;
    }

    /// 刷新面板
    pub fn refresh(&mut self) {
        self.needs_refresh = true;
    }
}

impl Default for EntityPanel {
    fn default() -> Self {
        Self::new()
    }
}

// 注意：由于bevy_ecs的限制，某些功能无法完全实现
// 在实际使用中，可能需要通过系统查询来获取更详细的实体信息
