use super::api::ScriptApi;
use super::system::{ScriptValue, ScriptResult};
use bevy_ecs::prelude::*;
use crate::ecs::{Transform, Velocity};
use glam::Vec3;
use std::sync::{Arc, Mutex};

/// 物理和音频脚本绑定
pub struct PhysicsAudioBindings {
    world: Arc<Mutex<World>>,
}

impl PhysicsAudioBindings {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }
    
    /// 注册物理和音频相关的脚本API
    pub fn register_api(&self, api: &mut ScriptApi) {
        // 物理相关API
        self.register_physics_api(api);
        
        // 音频相关API
        self.register_audio_api(api);
    }
    
    /// 注册物理相关API
    fn register_physics_api(&self, api: &mut ScriptApi) {
        let world = self.world.clone();
        
        // 添加Velocity组件
        api.register_function("add_velocity", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.get(0) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(Velocity {
                        lin: Vec3::ZERO,
                        ang: Vec3::ZERO,
                    });
                    ScriptResult::Success("Velocity component added".to_string())
                } else {
                    ScriptResult::Error("Entity not found".to_string())
                }
            } else {
                ScriptResult::Error("add_velocity() requires an entity ID".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 设置线性速度
        api.register_function("set_linear_velocity", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), Some(ScriptValue::Float(x)), Some(ScriptValue::Float(y)), Some(ScriptValue::Float(z))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3)) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                    velocity.lin = Vec3::new(*x as f32, *y as f32, *z as f32);
                    ScriptResult::Success("Linear velocity updated".to_string())
                } else {
                    ScriptResult::Error("Velocity component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_linear_velocity() requires entity_id, x, y, z".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 设置角速度
        api.register_function("set_angular_velocity", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), Some(ScriptValue::Float(x)), Some(ScriptValue::Float(y)), Some(ScriptValue::Float(z))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3)) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                    velocity.ang = Vec3::new(*x as f32, *y as f32, *z as f32);
                    ScriptResult::Success("Angular velocity updated".to_string())
                } else {
                    ScriptResult::Error("Velocity component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_angular_velocity() requires entity_id, x, y, z".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 获取速度信息
        api.register_function("get_velocity", move |args| {
            if let Some(ScriptValue::Int(entity_id)) = args.get(0) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(velocity) = world.get::<Velocity>(entity) {
                    let info = format!(
                        "Velocity {{ linear: ({}, {}, {}), angular: ({}, {}, {}) }}",
                        velocity.lin.x, velocity.lin.y, velocity.lin.z,
                        velocity.ang.x, velocity.ang.y, velocity.ang.z
                    );
                    ScriptResult::Success(info)
                } else {
                    ScriptResult::Error("Velocity component not found".to_string())
                }
            } else {
                ScriptResult::Error("get_velocity() requires an entity ID".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 应用力 (简化版,直接修改速度)
        api.register_function("apply_force", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), Some(ScriptValue::Float(fx)), Some(ScriptValue::Float(fy)), Some(ScriptValue::Float(fz))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3)) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                    // 简化的力应用:直接加到速度上
                    velocity.lin += Vec3::new(*fx as f32, *fy as f32, *fz as f32) * 0.01;
                    ScriptResult::Success("Force applied".to_string())
                } else {
                    ScriptResult::Error("Velocity component not found".to_string())
                }
            } else {
                ScriptResult::Error("apply_force() requires entity_id, fx, fy, fz".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 射线检测 (简化版)
        api.register_function("raycast", move |args| {
            if let (Some(ScriptValue::Float(ox)), Some(ScriptValue::Float(oy)), Some(ScriptValue::Float(oz)),
                    Some(ScriptValue::Float(dx)), Some(ScriptValue::Float(dy)), Some(ScriptValue::Float(dz))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3), args.get(4), args.get(5)) {
                let mut world = world.lock().unwrap();
                
                let origin = Vec3::new(*ox as f32, *oy as f32, *oz as f32);
                let direction = Vec3::new(*dx as f32, *dy as f32, *dz as f32).normalize();
                
                // 简化的射线检测:检查所有实体的Transform
                let mut closest_entity: Option<Entity> = None;
                let mut closest_distance = f32::MAX;
                
                let mut query = world.query::<(Entity, &Transform)>();
                for (entity, transform) in query.iter(&world) {
                    let to_entity = transform.pos - origin;
                    let projection = to_entity.dot(direction);
                    
                    if projection > 0.0 {
                        let distance = to_entity.length();
                        if distance < closest_distance {
                            closest_entity = Some(entity);
                            closest_distance = distance;
                        }
                    }
                }
                
                if let Some(entity) = closest_entity {
                    ScriptResult::Success(format!("Hit entity {} at distance {}", entity.to_bits(), closest_distance))
                } else {
                    ScriptResult::Success("No hit".to_string())
                }
            } else {
                ScriptResult::Error("raycast() requires origin (ox, oy, oz) and direction (dx, dy, dz)".to_string())
            }
        });
    }
    
    /// 注册音频相关API
    fn register_audio_api(&self, api: &mut ScriptApi) {
        // 播放音效 (占位实现)
        api.register_function("play_sound", move |args| {
            if let Some(ScriptValue::String(sound_name)) = args.get(0) {
                // 实际实现需要集成音频库
                ScriptResult::Success(format!("Playing sound: {}", sound_name))
            } else {
                ScriptResult::Error("play_sound() requires a sound name".to_string())
            }
        });
        
        // 播放音乐 (占位实现)
        api.register_function("play_music", move |args| {
            if let Some(ScriptValue::String(music_name)) = args.get(0) {
                ScriptResult::Success(format!("Playing music: {}", music_name))
            } else {
                ScriptResult::Error("play_music() requires a music name".to_string())
            }
        });
        
        // 停止音乐 (占位实现)
        api.register_function("stop_music", move |_args| {
            ScriptResult::Success("Music stopped".to_string())
        });
        
        // 设置音量 (占位实现)
        api.register_function("set_volume", move |args| {
            if let Some(ScriptValue::Float(volume)) = args.get(0) {
                let volume = (*volume as f32).clamp(0.0, 1.0);
                ScriptResult::Success(format!("Volume set to {}", volume))
            } else {
                ScriptResult::Error("set_volume() requires a volume value (0.0-1.0)".to_string())
            }
        });
        
        // 3D音效 (占位实现)
        api.register_function("play_sound_3d", move |args| {
            if let (Some(ScriptValue::String(sound_name)), 
                    Some(ScriptValue::Float(x)), 
                    Some(ScriptValue::Float(y)), 
                    Some(ScriptValue::Float(z))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3)) {
                ScriptResult::Success(format!(
                    "Playing 3D sound '{}' at position ({}, {}, {})",
                    sound_name, x, y, z
                ))
            } else {
                ScriptResult::Error("play_sound_3d() requires sound_name, x, y, z".to_string())
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_physics_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));
        
        let bindings = PhysicsAudioBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);
        
        // 创建实体
        let entity = {
            let mut world = world_arc.lock().unwrap();
            world.spawn_empty().id()
        };
        let entity_id = entity.to_bits() as i64;
        
        // 添加Velocity组件
        let result = api.call("add_velocity", &[ScriptValue::Int(entity_id)]);
        assert!(matches!(result, ScriptResult::Success(_)));
        
        // 设置线性速度
        let result = api.call("set_linear_velocity", &[
            ScriptValue::Int(entity_id),
            ScriptValue::Float(1.0),
            ScriptValue::Float(2.0),
            ScriptValue::Float(3.0),
        ]);
        assert!(matches!(result, ScriptResult::Success(_)));
    }
    
    #[test]
    fn test_audio_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));
        
        let bindings = PhysicsAudioBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);
        
        // 播放音效
        let result = api.call("play_sound", &[ScriptValue::String("explosion.wav".to_string())]);
        assert!(matches!(result, ScriptResult::Success(_)));
        
        // 设置音量
        let result = api.call("set_volume", &[ScriptValue::Float(0.5)]);
        assert!(matches!(result, ScriptResult::Success(_)));
    }
}
