//! 组件面板
//!
//! 显示实体的组件详细信息。

use super::Panel;
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::*;

/// 组件面板
///
/// 显示选中实体的所有组件详细信息。
pub struct ComponentPanel {
    /// 是否显示面板
    visible: bool,
    /// 当前查看的实体
    current_entity: Option<Entity>,
    /// 组件信息缓存
    component_cache: Vec<ComponentDetails>,
}

/// 组件详情
#[derive(Debug, Clone)]
struct ComponentDetails {
    /// 组件名称
    name: String,
    /// 组件值（序列化后的字符串）
    value: String,
    /// 组件类型ID
    type_id: String,
}

impl ComponentPanel {
    /// 创建新的组件面板
    pub fn new() -> Self {
        Self {
            visible: true,
            current_entity: None,
            component_cache: Vec::new(),
        }
    }

    /// 设置要查看的实体
    pub fn set_entity(&mut self, entity: Entity) {
        self.current_entity = Some(entity);
        self.visible = true;
        self.component_cache.clear();
    }

    /// 清除当前实体
    pub fn clear_entity(&mut self) {
        self.current_entity = None;
        self.component_cache.clear();
    }

    /// 显示面板
    pub fn show(&mut self, ctx: &egui::Context, world: &World) {
        if !self.visible {
            return;
        }

        egui::Window::new("Components")
            .default_size([400.0, 500.0])
            .open(&mut self.visible)
            .show(ctx, |ui| {
                if let Some(entity) = self.current_entity {
                    // 显示实体信息
                    ui.horizontal(|ui| {
                        ui.label("Entity:");
                        ui.monospace(format!("{}", entity.to_bits()));
                    });

                    // 检查实体是否存活
                    if !world.is_alive(entity) {
                        ui.colored_label(egui::Color32::RED, "Entity is dead");
                        return;
                    }

                    ui.separator();

                    // 刷新组件信息
                    if self.component_cache.is_empty() {
                        self.refresh_components(world, entity);
                    }

                    // 显示组件列表
                    ui.label(format!("Total Components: {}", self.component_cache.len()));

                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        for component in &self.component_cache {
                            self.show_component(ui, component);
                        }
                    });

                    // 刷新按钮
                    if ui.button("Refresh").clicked() {
                        self.refresh_components(world, entity);
                    }
                } else {
                    ui.label("No entity selected");
                    ui.label("Double-click an entity in the Entities panel to view its components");
                }
            });
    }

    /// 显示单个组件
    fn show_component(&mut self, ui: &mut egui::Ui, component: &ComponentDetails) {
        egui::CollapsingHeader::new(&component.name).default_open(false).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.monospace(&component.type_id);
            });

            ui.separator();

            // 显示组件值
            ui.label("Value:");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.monospace(&component.value);
            });
        });
    }

    /// 刷新组件列表
    fn refresh_components(&mut self, world: &World, entity: Entity) {
        self.component_cache.clear();

        // 遍历所有archetype查找该实体
        for archetype in world.archetypes() {
            if !archetype.contains(entity) {
                continue;
            }

            // 遍历组件
            for component_id in archetype.components() {
                if let Some(details) = self.get_component_details(world, entity, component_id) {
                    self.component_cache.push(details);
                }
            }

            break;
        }
    }

    /// 获取组件详情
    fn get_component_details(
        &self,
        _world: &World,
        _entity: Entity,
        _component_id: ComponentId,
    ) -> Option<ComponentDetails> {
        // 注意：bevy_ecs的限制使得无法直接通过ComponentId获取组件数据
        // 这是一个简化的实现
        Some(ComponentDetails {
            name: "Component".to_string(),
            value: "<data not accessible>".to_string(),
            type_id: "unknown".to_string(),
        })
    }
}

impl Default for ComponentPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// 可视化组件数据的辅助trait
///
/// 实现此trait的组件可以在调试面板中以自定义格式显示。
pub trait DebugInspectable {
    /// 将组件数据转换为可读字符串
    fn inspect(&self) -> String;
}

/// 为常见类型实现DebugInspectable
impl DebugInspectable for String {
    fn inspect(&self) -> String {
        self.clone()
    }
}

impl<T: std::fmt::Debug> DebugInspectable for T {
    default fn inspect(&self) -> String {
        format!("{:?}", self)
    }
}
