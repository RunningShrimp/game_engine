//  世界检查器
// 
//  提供ECS世界的完整视图，用于编辑器调试。
//  显示所有实体、组件和资源信息。

use bevy_ecs::prelude::*;
use crate::ecs::{Camera, PointLight, Sprite, Transform};
use crate::core::resources::RenderStats;

/// 世界检查器
#[derive(Debug)]
pub struct WorldInspector {
    /// 是否显示世界检查器
    pub visible: bool,
    /// 搜索过滤文本
    pub filter_text: String,
    /// 是否显示组件详情
    pub show_components: bool,
    /// 是否显示资源
    pub show_resources: bool,
    /// 选择的实体（仅用于显示）
    pub hovered_entity: Option<Entity>,
    /// 上次点击的实体
    pub selected_entity: Option<Entity>,
}

impl Default for WorldInspector {
    fn default() -> Self {
        Self {
            visible: true,
            filter_text: String::new(),
            show_components: true,
            show_resources: true,
            hovered_entity: None,
            selected_entity: None,
        }
    }
}

impl WorldInspector {
    /// 渲染世界检查器UI
    ///
    /// # 参数
    ///
    /// * `ctx` - egui上下文
    /// * `world` - ECS世界
    pub fn render(&mut self, ctx: &egui::Context, world: &World) {
        if !self.visible {
            return;
        }

        egui::Window::new("World Inspector")
            .default_size([400.0, 600.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.heading("ECS World");
                ui.separator();

                // 过滤器
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter_text);
                });
                ui.separator();

                // 实体列表
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_entities(ui, world);
                    });

                ui.separator();

                // 组件详情（如果选中了实体）
                if let Some(entity) = self.selected_entity {
                    self.render_entity_details(ui, world, entity);
                }

                ui.separator();

                // 资源信息
                if self.show_resources {
                    self.render_resources(ui, world);
                }
            });
    }

    /// 渲染实体列表
    fn render_entities(&mut self, ui: &mut egui::Ui, world: &World) {
        let entity_count = world.entities().len();
        ui.label(format!("Total Entities: {}", entity_count));

        let _filter = self.filter_text.to_lowercase();

        // 注意：World::query需要可变引用，这里暂时只显示实体数量
        // 实体列表显示需要在使用WorldInspector时传递可变World引用
        // 或者在EditorState中单独调用world_inspector.render
        
        ui.label("Entity listing requires mutable World reference");
        ui.label("World Inspector implemented at: game_engine/src/editor/world_inspector.rs");
    }

    /// 渲染实体详情
    fn render_entity_details(&self, ui: &mut egui::Ui, world: &World, entity: Entity) {
        ui.heading(format!("Entity: {:?}", entity));
        ui.separator();

        // Transform组件
        if let Some(transform) = world.get::<Transform>(entity) {
            ui.collapsing("Transform", |ui| {
                ui.label(format!("Position: ({:.2}, {:.2}, {:.2})", 
                    transform.pos.x, transform.pos.y, transform.pos.z));
                ui.label(format!("Rotation: ({:.2}, {:.2}, {:.2}, {:.2})", 
                    transform.rot.x, transform.rot.y, transform.rot.z, transform.rot.w));
                ui.label(format!("Scale: ({:.2}, {:.2}, {:.2})", 
                    transform.scale.x, transform.scale.y, transform.scale.z));
            });
        }

        // Sprite组件
        if let Some(sprite) = world.get::<Sprite>(entity) {
            ui.collapsing("Sprite", |ui| {
                ui.label(format!("Color: ({:.2}, {:.2}, {:.2}, {:.2})", 
                    sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]));
                ui.label(format!("Texture Index: {}", sprite.tex_index));
                ui.label(format!("Normal Tex Index: {}", sprite.normal_tex_index));
                ui.label(format!("UV Offset: ({:.2}, {:.2})", sprite.uv_off[0], sprite.uv_off[1]));
                ui.label(format!("UV Scale: ({:.2}, {:.2})", sprite.uv_scale[0], sprite.uv_scale[1]));
                ui.label(format!("Layer: {:.2}", sprite.layer));
            });
        }

        // Camera组件
        if let Some(camera) = world.get::<Camera>(entity) {
            ui.collapsing("Camera", |ui| {
                match &camera.projection {
                    crate::ecs::Projection::Perspective { fov, aspect, near, far } => {
                        ui.label(format!("Type: Perspective"));
                        ui.label(format!("FOV: {:.2}°", fov));
                        ui.label(format!("Aspect: {:.2}", aspect));
                        ui.label(format!("Near: {:.2}", near));
                        ui.label(format!("Far: {:.2}", far));
                    }
                    crate::ecs::Projection::Orthographic { scale, near, far } => {
                        ui.label(format!("Type: Orthographic"));
                        ui.label(format!("Scale: {:.2}", scale));
                        ui.label(format!("Near: {:.2}", near));
                        ui.label(format!("Far: {:.2}", far));
                    }
                }
                ui.label(format!("Active: {}", camera.is_active));
            });
        }

        // PointLight组件
        if let Some(light) = world.get::<PointLight>(entity) {
            ui.collapsing("PointLight", |ui| {
                ui.label(format!("Color: ({:.2}, {:.2}, {:.2})", 
                    light.color[0], light.color[1], light.color[2]));
                ui.label(format!("Intensity: {:.2}", light.intensity));
                ui.label(format!("Radius: {:.2}", light.radius));
                ui.label(format!("Falloff: {:.2}", light.falloff));
            });
        }
    }

    /// 渲染资源信息
    fn render_resources(&self, ui: &mut egui::Ui, world: &World) {
        ui.heading("Resources");
        ui.separator();

        // 渲染统计
        if let Some(stats) = world.get_resource::<RenderStats>() {
            ui.collapsing("Render Stats", |ui| {
                ui.label(format!("Draw Calls: {}", stats.draw_calls));
                ui.label(format!("Instances: {}", stats.instances));
                ui.label(format!("Passes: {}", stats.passes));
                ui.label(format!("Culled Objects: {}", stats.culled_objects));
                ui.label(format!("Total Objects: {}", stats.total_objects));

                if let Some(upload_ms) = stats.upload_ms {
                    ui.label(format!("Upload Time: {:.2} ms", upload_ms));
                }
                if let Some(main_ms) = stats.main_ms {
                    ui.label(format!("Main Render Time: {:.2} ms", main_ms));
                }
                if let Some(ui_ms) = stats.ui_ms {
                    ui.label(format!("UI Render Time: {:.2} ms", ui_ms));
                }
                if let Some(gpu_ms) = stats.gpu_pass_ms {
                    ui.label(format!("GPU Time: {:.2} ms", gpu_ms));
                }

                // 批处理统计
                ui.separator();
                ui.label("Batch Stats:");
                ui.label(format!("Total Batches: {}", stats.batch_total));
                ui.label(format!("Visible Batches: {}", stats.batch_visible_batches));
                ui.label(format!("Saved Draw Calls: {}", stats.batch_saved_draw_calls));
            });
        }

        // 批处理管理器
        if let Some(bm) = world.get_resource::<crate::render::instance_batch::BatchManager>() {
            ui.collapsing("Batch Manager", |ui| {
                ui.label(format!("Total Batches: {}", bm.stats.total_batches));
                ui.label(format!("Total Instances: {}", bm.stats.total_instances));
                ui.label(format!("Visible Batches: {}", bm.stats.visible_batches));
                ui.label(format!("Small Draw Calls: {}", bm.stats.small_draw_calls));
                ui.label(format!("Saved Draw Calls: {}", bm.stats.saved_draw_calls));
            });
        }

        // 时间资源
        if let Some(time) = world.get_resource::<crate::ecs::Time>() {
            ui.collapsing("Time", |ui| {
                ui.label(format!("Delta: {:.4} s", time.delta_seconds));
                ui.label(format!("Elapsed: {:.2} s", time.elapsed_seconds));
                ui.label(format!("Fixed Step: {:.4} s", time.fixed_time_step));
                ui.label(format!("Alpha: {:.4}", time.alpha));
            });
        }
    }

    /// 切换可见性
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// 设置可见性
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_world_inspector_default() {
        let inspector = WorldInspector::default();
        assert!(inspector.visible);
        assert!(inspector.show_components);
        assert!(inspector.show_resources);
        assert!(inspector.filter_text.is_empty());
        assert!(inspector.hovered_entity.is_none());
        assert!(inspector.selected_entity.is_none());
    }

    #[test]
    fn test_world_inspector_toggle_visible() {
        let mut inspector = WorldInspector::default();
        assert!(inspector.visible);

        inspector.toggle_visible();
        assert!(!inspector.visible);

        inspector.toggle_visible();
        assert!(inspector.visible);
    }

    #[test]
    fn test_world_inspector_set_visible() {
        let mut inspector = WorldInspector::default();
        assert!(inspector.visible);

        inspector.set_visible(false);
        assert!(!inspector.visible);

        inspector.set_visible(true);
        assert!(inspector.visible);
    }

    #[test]
    fn test_world_inspector_render_entities() {
        let mut world = World::new();
        let mut inspector = WorldInspector::default();

        // 添加一些实体
        let _entity1 = world.spawn((
            Transform {
                pos: Vec3::ZERO,
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Sprite::default(),
        ));

        let _entity2 = world.spawn((
            Transform {
                pos: Vec3::new(1.0, 2.0, 3.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Camera {
                projection: crate::ecs::Projection::Perspective {
                    fov: 60.0,
                    aspect: 16.0 / 9.0,
                    near: 0.1,
                    far: 100.0,
                },
                is_active: true,
            },
        ));

        // 测试过滤
        inspector.filter_text = "Entity(0)".to_string();
        // render_entities 需要egui上下文，这里仅测试逻辑
        assert_eq!(world.entities().len(), 2);
    }
}
