use crate::ecs::Transform;
use bevy_ecs::prelude::*;

/// 属性检查器
pub struct Inspector;

impl Inspector {
    /// 渲染检查器 (使用egui)
    pub fn render(ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>) {
        ui.heading("Inspector");
        ui.separator();

        if let Some(entity) = selected_entity {
            if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                // 显示实体ID
                ui.label(format!("Entity ID: {:?}", entity));
                ui.separator();

                // Name组件编辑 (占位)
                // ui.label("Name: Entity");

                // 编辑Transform组件
                if let Some(mut transform) = entity_mut.get_mut::<Transform>() {
                    ui.label("Transform:");

                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(
                            egui::DragValue::new(&mut transform.pos.x)
                                .prefix("X: ")
                                .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.pos.y)
                                .prefix("Y: ")
                                .speed(0.1),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.pos.z)
                                .prefix("Z: ")
                                .speed(0.1),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(
                            egui::DragValue::new(&mut transform.scale.x)
                                .prefix("X: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.scale.y)
                                .prefix("Y: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.scale.z)
                                .prefix("Z: ")
                                .speed(0.01),
                        );
                    });

                    // 旋转编辑 (简化版,直接编辑四元数)
                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(
                            egui::DragValue::new(&mut transform.rot.x)
                                .prefix("X: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.rot.y)
                                .prefix("Y: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.rot.z)
                                .prefix("Z: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut transform.rot.w)
                                .prefix("W: ")
                                .speed(0.01),
                        );
                    });

                    ui.separator();
                }

                // 显示其他组件 (占位)
                ui.label("Other Components:");
                ui.label("(Component list will be displayed here)");
            } else {
                ui.label("Entity not found");
            }
        } else {
            ui.label("No entity selected");
        }
    }
}
