use crate::ecs::{Camera, PointLight, Projection, Sprite, Transform};
use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

/// 实体模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTemplate {
    Empty,
    Sprite,
    Camera,
    PointLight,
}

impl EntityTemplate {
    /// 获取模板的显示名称
    pub fn name(&self) -> &'static str {
        match self {
            EntityTemplate::Empty => "Empty Entity",
            EntityTemplate::Sprite => "Sprite",
            EntityTemplate::Camera => "Camera",
            EntityTemplate::PointLight => "Point Light",
        }
    }

    /// 获取模板的图标
    pub fn icon(&self) -> &'static str {
        match self {
            EntityTemplate::Empty => "📦",
            EntityTemplate::Sprite => "🖼",
            EntityTemplate::Camera => "📷",
            EntityTemplate::PointLight => "💡",
        }
    }

    /// 从模板创建实体
    pub fn spawn(&self, world: &mut World, position: Vec3) -> Entity {
        match self {
            EntityTemplate::Empty => world
                .spawn(Transform {
                    pos: position,
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })
                .id(),
            EntityTemplate::Sprite => world
                .spawn((
                    Transform {
                        pos: position,
                        rot: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    Sprite::default(),
                ))
                .id(),
            EntityTemplate::Camera => world
                .spawn((
                    Transform {
                        pos: position,
                        rot: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    Camera {
                        is_active: true,
                        projection: Projection::Perspective {
                            fov: std::f32::consts::PI / 4.0,
                            aspect: 16.0 / 9.0,
                            near: 0.1,
                            far: 100.0,
                        },
                    },
                ))
                .id(),
            EntityTemplate::PointLight => world
                .spawn((
                    Transform {
                        pos: position,
                        rot: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    PointLight::default(),
                ))
                .id(),
        }
    }
}

/// 实体创建器
#[derive(Default)]
pub struct EntityCreator {
    /// 可用的实体模板
    pub templates: Vec<EntityTemplate>,
    /// 当前拖拽的模板
    pub dragging_template: Option<EntityTemplate>,
}

impl EntityCreator {
    pub fn new() -> Self {
        Self {
            templates: vec![
                EntityTemplate::Empty,
                EntityTemplate::Sprite,
                EntityTemplate::Camera,
                EntityTemplate::PointLight,
            ],
            ..Default::default()
        }
    }

    /// 渲染实体创建器UI
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<(EntityTemplate, egui::Pos2)> {
        ui.heading("Entity Creator");
        ui.separator();

        ui.label("Drag a template to the scene to create an entity:");
        ui.separator();

        let mut created_entity = None;

        for template in &self.templates {
            let response = ui
                .horizontal(|ui| {
                    ui.label(format!("{} {}", template.icon(), template.name()));

                    // 拖拽按钮
                    let drag_button = ui.button("Drag");

                    if drag_button.clicked() {
                        self.dragging_template = Some(*template);
                    }

                    drag_button
                })
                .inner;

            // 检测拖拽
            if response.dragged() {
                self.dragging_template = Some(*template);
            }

            // 检测拖拽释放
            if response.drag_stopped() {
                if let Some(template) = self.dragging_template {
                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                        created_entity = Some((template, pos));
                    }
                    self.dragging_template = None;
                }
            }
        }

        // 显示拖拽状态
        if let Some(template) = self.dragging_template {
            ui.separator();
            ui.label(format!("Dragging: {} {}", template.icon(), template.name()));
            ui.label("Release to create entity");
        }

        created_entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_templates() {
        let mut world = World::new();

        // 测试创建空实体
        let entity = EntityTemplate::Empty.spawn(&mut world, Vec3::ZERO);
        assert!(world.get::<Transform>(entity).is_some());

        // 测试创建Sprite实体
        let entity = EntityTemplate::Sprite.spawn(&mut world, Vec3::ZERO);
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());

        // 测试创建Camera实体
        let entity = EntityTemplate::Camera.spawn(&mut world, Vec3::ZERO);
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Camera>(entity).is_some());

        // 测试创建PointLight实体
        let entity = EntityTemplate::PointLight.spawn(&mut world, Vec3::ZERO);
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<PointLight>(entity).is_some());
    }
}
