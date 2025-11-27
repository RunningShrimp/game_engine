use bevy_ecs::prelude::*;
use crate::ecs::{Transform, Sprite, PointLight, Camera, Projection, PbrMaterialComp, PointLight3D, DirectionalLightComp};
use crate::physics::{RigidBodyDesc, ColliderDesc, ShapeType};
use serde::{Serialize, Deserialize};
use glam::{Vec3, Quat};
use std::collections::HashMap;
use rapier2d::prelude::RigidBodyType;

/// 序列化的场景数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    /// 场景名称
    pub name: String,
    /// 场景版本
    #[serde(default)]
    pub version: u32,
    /// 实体列表
    pub entities: Vec<SerializedEntity>,
    /// 场景元数据
    #[serde(default)]
    pub metadata: SceneMetadata,
}

/// 场景元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// 作者
    #[serde(default)]
    pub author: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 创建时间 (Unix timestamp)
    #[serde(default)]
    pub created_at: u64,
    /// 修改时间 (Unix timestamp)
    #[serde(default)]
    pub modified_at: u64,
}

/// 序列化的实体数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// 实体ID
    pub id: u64,
    /// 实体名称 (可选)
    #[serde(default)]
    pub name: Option<String>,
    /// 组件列表
    pub components: HashMap<String, SerializedComponent>,
}

/// 序列化的组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializedComponent {
    /// 变换组件
    Transform {
        position: [f32; 3],
        rotation: [f32; 4], // Quaternion (x, y, z, w)
        scale: [f32; 3],
    },
    /// 精灵渲染组件
    Sprite {
        color: [f32; 4],
        tex_index: u32,
        normal_tex_index: u32,
        uv_off: [f32; 2],
        uv_scale: [f32; 2],
        layer: f32,
    },
    /// 2D点光源组件
    PointLight {
        color: [f32; 3],
        radius: f32,
        intensity: f32,
        falloff: f32,
    },
    /// 相机组件
    Camera {
        is_active: bool,
        projection: SerializedProjection,
    },
    /// 刚体描述组件
    RigidBody {
        body_type: SerializedRigidBodyType,
        position: [f32; 2],
    },
    /// 碰撞体描述组件
    Collider {
        shape_type: SerializedShapeType,
        half_extents: [f32; 2],
        radius: f32,
    },
    /// PBR材质组件
    PbrMaterial {
        base_color: [f32; 4],
        metallic: f32,
        roughness: f32,
        ambient_occlusion: f32,
        emissive: [f32; 3],
        emissive_strength: f32,
    },
    /// 3D点光源组件
    PointLight3D {
        color: [f32; 3],
        intensity: f32,
        radius: f32,
    },
    /// 方向光组件
    DirectionalLight {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
}

/// 序列化的投影类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedProjection {
    Orthographic {
        scale: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

/// 序列化的刚体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedRigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}

/// 序列化的形状类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedShapeType {
    Cuboid,
    Ball,
}

impl SerializedScene {
    /// 当前序列化版本
    pub const CURRENT_VERSION: u32 = 1;
    
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: Self::CURRENT_VERSION,
            entities: Vec::new(),
            metadata: SceneMetadata::default(),
        }
    }
    
    /// 从World序列化场景
    pub fn from_world(world: &World, name: impl Into<String>) -> Self {
        let mut scene = Self::new(name);
        scene.metadata.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        scene.metadata.modified_at = scene.metadata.created_at;
        
        // 遍历所有实体
        for entity in world.iter_entities() {
            let mut serialized_entity = SerializedEntity {
                id: entity.id().to_bits(),
                name: None,
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
            
            // 序列化Sprite组件
            if let Some(sprite) = world.get::<Sprite>(entity.id()) {
                serialized_entity.components.insert(
                    "Sprite".to_string(),
                    SerializedComponent::Sprite {
                        color: sprite.color,
                        tex_index: sprite.tex_index,
                        normal_tex_index: sprite.normal_tex_index,
                        uv_off: sprite.uv_off,
                        uv_scale: sprite.uv_scale,
                        layer: sprite.layer,
                    },
                );
            }
            
            // 序列化PointLight组件
            if let Some(light) = world.get::<PointLight>(entity.id()) {
                serialized_entity.components.insert(
                    "PointLight".to_string(),
                    SerializedComponent::PointLight {
                        color: light.color,
                        radius: light.radius,
                        intensity: light.intensity,
                        falloff: light.falloff,
                    },
                );
            }
            
            // 序列化Camera组件
            if let Some(camera) = world.get::<Camera>(entity.id()) {
                let projection = match camera.projection {
                    Projection::Orthographic { scale, near, far } => {
                        SerializedProjection::Orthographic { scale, near, far }
                    }
                    Projection::Perspective { fov, aspect, near, far } => {
                        SerializedProjection::Perspective { fov, aspect, near, far }
                    }
                };
                serialized_entity.components.insert(
                    "Camera".to_string(),
                    SerializedComponent::Camera {
                        is_active: camera.is_active,
                        projection,
                    },
                );
            }
            
            // 序列化RigidBodyDesc组件
            if let Some(rb) = world.get::<RigidBodyDesc>(entity.id()) {
                let body_type = match rb.body_type {
                    RigidBodyType::Dynamic => SerializedRigidBodyType::Dynamic,
                    RigidBodyType::Fixed => SerializedRigidBodyType::Fixed,
                    RigidBodyType::KinematicPositionBased => SerializedRigidBodyType::KinematicPositionBased,
                    RigidBodyType::KinematicVelocityBased => SerializedRigidBodyType::KinematicVelocityBased,
                };
                serialized_entity.components.insert(
                    "RigidBody".to_string(),
                    SerializedComponent::RigidBody {
                        body_type,
                        position: rb.position,
                    },
                );
            }
            
            // 序列化ColliderDesc组件
            if let Some(col) = world.get::<ColliderDesc>(entity.id()) {
                let shape_type = match col.shape_type {
                    ShapeType::Cuboid => SerializedShapeType::Cuboid,
                    ShapeType::Ball => SerializedShapeType::Ball,
                };
                serialized_entity.components.insert(
                    "Collider".to_string(),
                    SerializedComponent::Collider {
                        shape_type,
                        half_extents: col.half_extents,
                        radius: col.radius,
                    },
                );
            }
            
            // 序列化PbrMaterialComp组件
            if let Some(mat) = world.get::<PbrMaterialComp>(entity.id()) {
                serialized_entity.components.insert(
                    "PbrMaterial".to_string(),
                    SerializedComponent::PbrMaterial {
                        base_color: mat.base_color,
                        metallic: mat.metallic,
                        roughness: mat.roughness,
                        ambient_occlusion: mat.ambient_occlusion,
                        emissive: mat.emissive,
                        emissive_strength: mat.emissive_strength,
                    },
                );
            }
            
            // 序列化PointLight3D组件
            if let Some(light) = world.get::<PointLight3D>(entity.id()) {
                serialized_entity.components.insert(
                    "PointLight3D".to_string(),
                    SerializedComponent::PointLight3D {
                        color: light.color,
                        intensity: light.intensity,
                        radius: light.radius,
                    },
                );
            }
            
            // 序列化DirectionalLightComp组件
            if let Some(light) = world.get::<DirectionalLightComp>(entity.id()) {
                serialized_entity.components.insert(
                    "DirectionalLight".to_string(),
                    SerializedComponent::DirectionalLight {
                        direction: light.direction,
                        color: light.color,
                        intensity: light.intensity,
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
            for (_component_name, component_data) in &serialized_entity.components {
                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    match component_data {
                        SerializedComponent::Transform { position, rotation, scale } => {
                            entity_mut.insert(Transform {
                                pos: Vec3::from_array(*position),
                                rot: Quat::from_array(*rotation),
                                scale: Vec3::from_array(*scale),
                            });
                        }
                        SerializedComponent::Sprite { color, tex_index, normal_tex_index, uv_off, uv_scale, layer } => {
                            entity_mut.insert(Sprite {
                                color: *color,
                                tex_index: *tex_index,
                                normal_tex_index: *normal_tex_index,
                                uv_off: *uv_off,
                                uv_scale: *uv_scale,
                                layer: *layer,
                            });
                        }
                        SerializedComponent::PointLight { color, radius, intensity, falloff } => {
                            entity_mut.insert(PointLight {
                                color: *color,
                                radius: *radius,
                                intensity: *intensity,
                                falloff: *falloff,
                            });
                        }
                        SerializedComponent::Camera { is_active, projection } => {
                            let proj = match projection {
                                SerializedProjection::Orthographic { scale, near, far } => {
                                    Projection::Orthographic { scale: *scale, near: *near, far: *far }
                                }
                                SerializedProjection::Perspective { fov, aspect, near, far } => {
                                    Projection::Perspective { fov: *fov, aspect: *aspect, near: *near, far: *far }
                                }
                            };
                            entity_mut.insert(Camera {
                                is_active: *is_active,
                                projection: proj,
                            });
                        }
                        SerializedComponent::RigidBody { body_type, position } => {
                            let bt = match body_type {
                                SerializedRigidBodyType::Dynamic => RigidBodyType::Dynamic,
                                SerializedRigidBodyType::Fixed => RigidBodyType::Fixed,
                                SerializedRigidBodyType::KinematicPositionBased => RigidBodyType::KinematicPositionBased,
                                SerializedRigidBodyType::KinematicVelocityBased => RigidBodyType::KinematicVelocityBased,
                            };
                            entity_mut.insert(RigidBodyDesc {
                                body_type: bt,
                                position: *position,
                            });
                        }
                        SerializedComponent::Collider { shape_type, half_extents, radius } => {
                            let st = match shape_type {
                                SerializedShapeType::Cuboid => ShapeType::Cuboid,
                                SerializedShapeType::Ball => ShapeType::Ball,
                            };
                            entity_mut.insert(ColliderDesc {
                                shape_type: st,
                                half_extents: *half_extents,
                                radius: *radius,
                            });
                        }
                        SerializedComponent::PbrMaterial { base_color, metallic, roughness, ambient_occlusion, emissive, emissive_strength } => {
                            entity_mut.insert(PbrMaterialComp {
                                base_color: *base_color,
                                metallic: *metallic,
                                roughness: *roughness,
                                ambient_occlusion: *ambient_occlusion,
                                emissive: *emissive,
                                emissive_strength: *emissive_strength,
                            });
                        }
                        SerializedComponent::PointLight3D { color, intensity, radius } => {
                            entity_mut.insert(PointLight3D {
                                color: *color,
                                intensity: *intensity,
                                radius: *radius,
                            });
                        }
                        SerializedComponent::DirectionalLight { direction, color, intensity } => {
                            entity_mut.insert(DirectionalLightComp {
                                direction: *direction,
                                color: *color,
                                intensity: *intensity,
                            });
                        }
                    }
                }
            }
        }
        
        entity_map
    }
    
    /// 清空场景中的所有实体
    pub fn clear_world(world: &mut World) {
        // 收集所有实体ID
        let entities: Vec<Entity> = world.iter_entities().map(|e| e.id()).collect();
        // 删除所有实体
        for entity in entities {
            world.despawn(entity);
        }
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
