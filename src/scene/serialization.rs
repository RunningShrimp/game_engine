use bevy_ecs::prelude::*;
use crate::ecs::Transform;
use serde::{Serialize, Deserialize};
use glam::{Vec3, Quat};
use std::collections::HashMap;

/// 序列化的场景数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    /// 场景名称
    pub name: String,
    /// 实体列表
    pub entities: Vec<SerializedEntity>,
}

/// 序列化的实体数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// 实体ID
    pub id: u64,
    /// 组件列表
    pub components: HashMap<String, SerializedComponent>,
}

/// 序列化的组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializedComponent {
    Transform {
        position: [f32; 3],
        rotation: [f32; 4], // Quaternion (x, y, z, w)
        scale: [f32; 3],
    },
    // 可以添加更多组件类型
}

impl SerializedScene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
        }
    }
    
    /// 从World序列化场景
    pub fn from_world(world: &World, name: impl Into<String>) -> Self {
        let mut scene = Self::new(name);
        
        // 遍历所有实体
        for entity in world.iter_entities() {
            let mut serialized_entity = SerializedEntity {
                id: entity.id().to_bits(),
                components: HashMap::new(),
            };
            
            // 序列化Transform组件
            if let Some(transform) = world.get::<Transform>(entity.id()) {
                serialized_entity.components.insert(
                    "Transform".to_string(),
                    SerializedComponent::Transform {
                        position: transform.pos.to_array(),
                        rotation: [transform.rot.x, transform.rot.y, transform.rot.z, transform.rot.w],
                        scale: transform.scale.to_array(),
                    },
                );
            }
            
            // 只添加有组件的实体
            if !serialized_entity.components.is_empty() {
                scene.entities.push(serialized_entity);
            }
        }
        
        scene
    }
    
    /// 反序列化场景到World
    pub fn to_world(&self, world: &mut World) -> HashMap<u64, Entity> {
        let mut entity_map = HashMap::new();
        
        for serialized_entity in &self.entities {
            let entity = world.spawn_empty().id();
            entity_map.insert(serialized_entity.id, entity);
            
            // 反序列化组件
            for (component_name, component_data) in &serialized_entity.components {
                match component_data {
                    SerializedComponent::Transform { position, rotation, scale } => {
                        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                            entity_mut.insert(Transform {
                                pos: Vec3::from_array(*position),
                                rot: Quat::from_array(*rotation),
                                scale: Vec3::from_array(*scale),
                            });
                        }
                    }
                }
            }
        }
        
        entity_map
    }
    
    /// 保存场景到JSON文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// 从JSON文件加载场景
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let scene = serde_json::from_str(&json)?;
        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scene_serialization() {
        let mut world = World::new();
        
        // 创建一些测试实体
        world.spawn(Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
        
        world.spawn(Transform {
            pos: Vec3::new(4.0, 5.0, 6.0),
            rot: Quat::IDENTITY,
            scale: Vec3::new(2.0, 2.0, 2.0),
        });
        
        // 序列化场景
        let scene = SerializedScene::from_world(&world, "test_scene");
        assert_eq!(scene.entities.len(), 2);
        
        // 反序列化场景
        let mut new_world = World::new();
        let entity_map = scene.to_world(&mut new_world);
        assert_eq!(entity_map.len(), 2);
        
        // 验证反序列化的数据
        let mut query = new_world.query::<&Transform>();
        let transforms: Vec<_> = query.iter(&new_world).collect();
        assert_eq!(transforms.len(), 2);
    }
    
    #[test]
    fn test_scene_file_io() {
        let mut world = World::new();
        
        world.spawn(Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
        
        let scene = SerializedScene::from_world(&world, "test_scene");
        
        // 保存到文件
        let path = "/tmp/test_scene.json";
        scene.save_to_file(path).unwrap();
        
        // 从文件加载
        let loaded_scene = SerializedScene::load_from_file(path).unwrap();
        assert_eq!(loaded_scene.name, "test_scene");
        assert_eq!(loaded_scene.entities.len(), 1);
        
        // 清理测试文件
        std::fs::remove_file(path).ok();
    }
}
