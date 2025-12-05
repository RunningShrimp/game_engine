use super::api::ScriptApi;
use super::system::{ScriptResult, ScriptValue};
use crate::ecs::{Camera, Projection, Sprite};
use bevy_ecs::prelude::*;
use std::sync::{Arc, Mutex};

/// 扩展的ECS脚本绑定 - 支持更多组件类型
pub struct ExtendedEcsBindings {
    world: Arc<Mutex<World>>,
}

impl ExtendedEcsBindings {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }

    /// 注册扩展的ECS相关的脚本API
    pub fn register_api(&self, api: &mut ScriptApi) {
        // Sprite组件相关API
        self.register_sprite_api(api);

        // Camera组件相关API
        self.register_camera_api(api);
    }

    /// 注册Sprite组件相关API
    fn register_sprite_api(&self, api: &mut ScriptApi) {
        let world = self.world.clone();

        // 添加Sprite组件
        api.register_function("add_sprite", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(Sprite::default());
                    ScriptResult::Success("Sprite component added".to_string())
                } else {
                    ScriptResult::Error("Entity not found".to_string())
                }
            } else {
                ScriptResult::Error("add_sprite() requires an entity ID".to_string())
            }
        });

        let world = self.world.clone();

        // 设置Sprite颜色
        api.register_function("set_sprite_color", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(r)),
                Some(ScriptValue::Float(g)),
                Some(ScriptValue::Float(b)),
            ) = (args.first(), args.get(1), args.get(2), args.get(3))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.color = [*r as f32, *g as f32, *b as f32, 1.0];
                    ScriptResult::Success("Sprite color updated".to_string())
                } else {
                    ScriptResult::Error("Sprite component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_sprite_color() requires entity_id, r, g, b".to_string())
            }
        });

        let world = self.world.clone();

        // 设置Sprite UV缩放
        api.register_function("set_sprite_uv_scale", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(u)),
                Some(ScriptValue::Float(v)),
            ) = (args.first(), args.get(1), args.get(2))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.uv_scale = [*u as f32, *v as f32];
                    ScriptResult::Success("Sprite UV scale updated".to_string())
                } else {
                    ScriptResult::Error("Sprite component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_sprite_uv_scale() requires entity_id, u, v".to_string())
            }
        });

        let world = self.world.clone();

        // 获取Sprite信息
        api.register_function("get_sprite", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(sprite) = world.get::<Sprite>(entity) {
                    let info = format!(
                        "Sprite {{ color: ({}, {}, {}, {}), tex_index: {}, layer: {} }}",
                        sprite.color[0],
                        sprite.color[1],
                        sprite.color[2],
                        sprite.color[3],
                        sprite.tex_index,
                        sprite.layer
                    );
                    ScriptResult::Success(info)
                } else {
                    ScriptResult::Error("Sprite component not found".to_string())
                }
            } else {
                ScriptResult::Error("get_sprite() requires an entity ID".to_string())
            }
        });
    }

    /// 注册Camera组件相关API
    fn register_camera_api(&self, api: &mut ScriptApi) {
        let world = self.world.clone();

        // 添加Camera组件
        api.register_function("add_camera", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(Camera {
                        is_active: true,
                        projection: Projection::Perspective {
                            fov: std::f32::consts::PI / 4.0,
                            aspect: 16.0 / 9.0,
                            near: 0.1,
                            far: 100.0,
                        },
                    });
                    ScriptResult::Success("Camera component added".to_string())
                } else {
                    ScriptResult::Error("Entity not found".to_string())
                }
            } else {
                ScriptResult::Error("add_camera() requires an entity ID".to_string())
            }
        });

        let world = self.world.clone();

        // 设置Camera FOV
        api.register_function("set_camera_fov", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), Some(ScriptValue::Float(fov))) =
                (args.first(), args.get(1))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut camera) = world.get_mut::<Camera>(entity) {
                    if let Projection::Perspective { fov, .. } = &mut camera.projection {
                        *fov = (*fov).to_radians();
                    }
                    ScriptResult::Success("Camera FOV updated".to_string())
                } else {
                    ScriptResult::Error("Camera component not found".to_string())
                }
            } else {
                ScriptResult::Error(
                    "set_camera_fov() requires entity_id, fov (degrees)".to_string(),
                )
            }
        });

        let world = self.world.clone();

        // 设置Camera近平面和远平面
        api.register_function("set_camera_planes", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(near)),
                Some(ScriptValue::Float(far)),
            ) = (args.first(), args.get(1), args.get(2))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut camera) = world.get_mut::<Camera>(entity) {
                    match &mut camera.projection {
                        Projection::Perspective {
                            near: n, far: f, ..
                        } => {
                            *n = *near as f32;
                            *f = *far as f32;
                        }
                        Projection::Orthographic {
                            near: n, far: f, ..
                        } => {
                            *n = *near as f32;
                            *f = *far as f32;
                        }
                    }
                    ScriptResult::Success("Camera planes updated".to_string())
                } else {
                    ScriptResult::Error("Camera component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_camera_planes() requires entity_id, near, far".to_string())
            }
        });

        let world = self.world.clone();

        // 获取Camera信息
        api.register_function("get_camera", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(camera) = world.get::<Camera>(entity) {
                    let info = match camera.projection {
                        Projection::Perspective { fov, aspect, near, far } => {
                            format!(
                                "Camera {{ type: Perspective, fov: {:.2}°, aspect: {:.2}, near: {}, far: {} }}",
                                fov.to_degrees(), aspect, near, far
                            )
                        }
                        Projection::Orthographic { scale, near, far } => {
                            format!(
                                "Camera {{ type: Orthographic, scale: {}, near: {}, far: {} }}",
                                scale, near, far
                            )
                        }
                    };
                    ScriptResult::Success(info)
                } else {
                    ScriptResult::Error("Camera component not found".to_string())
                }
            } else {
                ScriptResult::Error("get_camera() requires an entity ID".to_string())
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));

        let bindings = ExtendedEcsBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);

        // 创建实体
        let entity = {
            let mut world = world_arc.lock().unwrap();
            world.spawn_empty().id()
        };
        let entity_id = entity.to_bits() as i64;

        // 添加Sprite组件
        let result = api.call("add_sprite", &[ScriptValue::Int(entity_id)]);
        assert!(matches!(result, ScriptResult::Success(_)));

        // 设置Sprite颜色
        let result = api.call(
            "set_sprite_color",
            &[
                ScriptValue::Int(entity_id),
                ScriptValue::Float(1.0),
                ScriptValue::Float(0.0),
                ScriptValue::Float(0.0),
            ],
        );
        assert!(matches!(result, ScriptResult::Success(_)));
    }

    #[test]
    fn test_camera_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));

        let bindings = ExtendedEcsBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);

        // 创建实体
        let entity = {
            let mut world = world_arc.lock().unwrap();
            world.spawn_empty().id()
        };
        let entity_id = entity.to_bits() as i64;

        // 添加Camera组件
        let result = api.call("add_camera", &[ScriptValue::Int(entity_id)]);
        assert!(matches!(result, ScriptResult::Success(_)));

        // 设置Camera FOV
        let result = api.call(
            "set_camera_fov",
            &[ScriptValue::Int(entity_id), ScriptValue::Float(60.0)],
        );
        assert!(matches!(result, ScriptResult::Success(_)));
    }
}
