use super::api::{ExtendedScriptValue, ScriptApi};
use super::system::{ScriptResult, ScriptValue};
use crate::ecs::Transform;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use std::sync::{Arc, Mutex};

/// ECS脚本绑定 - 提供实体和组件操作的脚本接口
pub struct EcsScriptBindings {
    world: Arc<Mutex<World>>,
}

impl EcsScriptBindings {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }

    /// 注册ECS相关的脚本API
    pub fn register_api(&self, api: &mut ScriptApi) {
        let world = self.world.clone();

        // 创建实体
        api.register_function("create_entity", move |_args| {
            let mut world = world.lock().unwrap();
            let entity = world.spawn_empty().id();
            ScriptResult::Success(entity.to_bits().to_string())
        });

        let world = self.world.clone();

        // 销毁实体
        api.register_function("destroy_entity", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                if world.despawn(entity) {
                    ScriptResult::Success("Entity destroyed".to_string())
                } else {
                    ScriptResult::Error("Entity not found".to_string())
                }
            } else {
                ScriptResult::Error("destroy_entity() requires an entity ID".to_string())
            }
        });

        let world = self.world.clone();

        // 获取Transform组件
        api.register_function("get_transform", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(transform) = world.get::<Transform>(entity) {
                    let value = ExtendedScriptValue::Object(
                        vec![
                            (
                                "position".to_string(),
                                ExtendedScriptValue::Vec3(transform.pos),
                            ),
                            (
                                "rotation".to_string(),
                                ExtendedScriptValue::Quat(transform.rot),
                            ),
                            (
                                "scale".to_string(),
                                ExtendedScriptValue::Vec3(transform.scale),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    );
                    ScriptResult::Success(format!("{:?}", value))
                } else {
                    ScriptResult::Error("Transform component not found".to_string())
                }
            } else {
                ScriptResult::Error("get_transform() requires an entity ID".to_string())
            }
        });

        let world = self.world.clone();

        // 设置Transform位置
        api.register_function("set_position", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(x)),
                Some(ScriptValue::Float(y)),
                Some(ScriptValue::Float(z)),
            ) = (args.first(), args.get(1), args.get(2), args.get(3))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                    transform.pos = Vec3::new(*x as f32, *y as f32, *z as f32);
                    ScriptResult::Success("Position updated".to_string())
                } else {
                    ScriptResult::Error("Transform component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_position() requires entity_id, x, y, z".to_string())
            }
        });

        let world = self.world.clone();

        // 设置Transform旋转 (欧拉角,度)
        api.register_function("set_rotation", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(x)),
                Some(ScriptValue::Float(y)),
                Some(ScriptValue::Float(z)),
            ) = (args.first(), args.get(1), args.get(2), args.get(3))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                    transform.rot = Quat::from_euler(
                        glam::EulerRot::XYZ,
                        (*x as f32).to_radians(),
                        (*y as f32).to_radians(),
                        (*z as f32).to_radians(),
                    );
                    ScriptResult::Success("Rotation updated".to_string())
                } else {
                    ScriptResult::Error("Transform component not found".to_string())
                }
            } else {
                ScriptResult::Error(
                    "set_rotation() requires entity_id, x, y, z (degrees)".to_string(),
                )
            }
        });

        let world = self.world.clone();

        // 设置Transform缩放
        api.register_function("set_scale", move |args| {
            if let (
                Some(ScriptValue::Int(entity_id)),
                Some(ScriptValue::Float(x)),
                Some(ScriptValue::Float(y)),
                Some(ScriptValue::Float(z)),
            ) = (args.first(), args.get(1), args.get(2), args.get(3))
            {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                    transform.scale = Vec3::new(*x as f32, *y as f32, *z as f32);
                    ScriptResult::Success("Scale updated".to_string())
                } else {
                    ScriptResult::Error("Transform component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_scale() requires entity_id, x, y, z".to_string())
            }
        });

        let world = self.world.clone();

        // 添加Transform组件
        api.register_function("add_transform", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.first() {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);

                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(Transform::default());
                    ScriptResult::Success("Transform component added".to_string())
                } else {
                    ScriptResult::Error("Entity not found".to_string())
                }
            } else {
                ScriptResult::Error("add_transform() requires an entity ID".to_string())
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));

        let bindings = EcsScriptBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);

        // 测试创建实体
        let result = api.call("create_entity", &[]);
        assert!(matches!(result, ScriptResult::Success(_)));

        // 测试添加Transform组件
        if let ScriptResult::Success(entity_id_str) = result {
            let entity_id: i64 = entity_id_str.parse().unwrap();
            let result = api.call("add_transform", &[ScriptValue::Int(entity_id)]);
            assert!(matches!(result, ScriptResult::Success(_)));

            // 测试设置位置
            let result = api.call(
                "set_position",
                &[
                    ScriptValue::Int(entity_id),
                    ScriptValue::Float(1.0),
                    ScriptValue::Float(2.0),
                    ScriptValue::Float(3.0),
                ],
            );
            assert!(matches!(result, ScriptResult::Success(_)));
        }
    }
}
