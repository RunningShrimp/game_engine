//! # Entity Component System (ECS)
//!
//! This module provides a high-performance Entity Component System architecture
//! based on Bevy ECS.
//!
//! ## Features
//!
//! - **Efficient Component Storage** - High-performance component access via Bevy ECS
//! - **SoA Layout** - Structure of Arrays layout for cache optimization
//! - **Dirty Flag Tracking** - Sync only changed components
//! - **Query System** - Flexible entity query API
//! - **System Scheduling** - Parallel system execution and dependency management
//!
//! ## Core Components
//!
//! - [`Transform`] - Position, rotation, and scale component
//! - [`Velocity`] - Linear and angular velocity
//! - [`Sprite`] - 2D sprite rendering component
//! - [`Camera`] - Camera component
//! - [`Material`] - Material component
//! - [`PbrMaterialComp`] - PBR material component
//! - [`PointLight3D`] - 3D point light
//! - [`DirectionalLightComp`] - Directional light
//! - [`Time`] - Time resource
//!
//! ## Optimization Features
//!
//! - [`SoALayoutManager`] - SoA layout manager
//! - [`DirtyTrackingResource`] - Dirty flag tracking
//! - [`TileEntityPool`] - Tile entity object pool
//!
//! ## Examples
//!
//! ### Basic Entity Creation
//!
//! ```rust,no_run
//! use bevy_ecs::prelude::*;
//! use game_engine::ecs::{Transform, Sprite};
//!
//! fn spawn_sprite(mut commands: Commands) {
//!     commands.spawn((
//!         Transform::default(),
//!         Sprite {
//!             color: [1.0, 1.0, 1.0, 1.0],
//!             tex_index: 0,
//!             ..Default::default()
//!         },
//!     ));
//! }
//! ```
//!
//! ### System Query
//!
//! ```rust,no_run
//! use bevy_ecs::prelude::*;
//! use game_engine::ecs::Transform;
//!
//! fn update_positions(mut query: Query<&mut Transform>) {
//!     for mut transform in query.iter_mut() {
//!         transform.pos.x += 1.0;
//!     }
//! }
//! ```

// 模块私有实现说明：
// - 基于Bevy ECS实现实体组件系统
// - 提供瓦片地图系统的实体池优化
// - 支持精灵动画（flipbook）系统
// - 集成SoA布局和脏标记追踪用于性能优化

use crate::impl_default;
use crate::impl_default_and_new;
pub use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

pub mod soa_layout;
pub use soa_layout::{SoALayoutManager, SoAStats, SoATransformStorage, SoAVelocityStorage};

pub mod dirty_tracking;
pub use dirty_tracking::{ComponentDirty, DirtyFlags, DirtyTrackingConfig, DirtyTrackingResource};

pub mod query_cache;
pub use query_cache::{QueryCache, QueryCacheConfig, QueryCacheStats};

/// Transform component representing position, rotation, and scale
///
/// This component is used to define the spatial transformation of entities in 3D space.
#[derive(Component, Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    /// Position in 3D space
    pub pos: Vec3,
    /// Rotation as a quaternion
    pub rot: Quat,
    /// Scale factors along each axis
    pub scale: Vec3,
}

impl_default_and_new!(Transform {
    pos: Vec3::ZERO,
    rot: Quat::IDENTITY,
    scale: Vec3::ONE,
});

/// Velocity component for linear and angular velocity
///
/// This component represents the velocity of an entity, including both linear
/// and angular components.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Velocity {
    /// Linear velocity
    pub lin: Vec3,
    /// Angular velocity
    pub ang: Vec3,
}

impl Velocity {
    /// Creates a new velocity component with default values (zero velocity)
    pub fn new() -> Self {
        Self::default()
    }
}

// 注意：Velocity已经使用#[derive(Default)]，new()方法调用default()是正确的模式

/// 2D sprite rendering component
///
/// This component defines how a 2D sprite is rendered, including its color,
/// texture coordinates, and rendering layer.
#[derive(Component, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    /// RGBA color
    pub color: [f32; 4],
    /// Texture index
    pub tex_index: u32,
    /// Normal texture index (0 means no normal map)
    pub normal_tex_index: u32,
    /// UV offset
    pub uv_off: [f32; 2],
    /// UV scale
    pub uv_scale: [f32; 2],
    /// Rendering layer (z-order for 2D)
    pub layer: f32,
}

impl_default_and_new!(Sprite {
    color: [1.0; 4],
    tex_index: 0,
    normal_tex_index: 0,
    uv_off: [0.0, 0.0],
    uv_scale: [1.0, 1.0],
    layer: 0.0,
});

#[derive(Component, Clone, Debug)]
pub struct TextureHandle {
    pub handle: crate::resources::manager::Handle<u32>,
}

#[derive(Component, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PointLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

impl_default_and_new!(PointLight {
    color: [1.0, 1.0, 1.0],
    intensity: 1.0,
    radius: 100.0,
    falloff: 1.0,
});

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub enum Projection {
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

impl Default for Projection {
    fn default() -> Self {
        Projection::Orthographic {
            scale: 1.0,
            near: 0.0,
            far: 1000.0,
        }
    }
}

impl Projection {
    /// 创建默认投影
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Component, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    pub is_active: bool,
    pub projection: Projection,
}

impl_default_and_new!(Camera {
    is_active: true,
    projection: Projection::default(),
});

#[derive(Component, Clone, Debug)]
pub struct Mesh {
    pub handle: crate::resources::manager::Handle<crate::render::mesh::GpuMesh>,
}

#[derive(Component, Clone, Debug)]
pub struct Material {
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

impl_default_and_new!(Material {
    color: [1.0; 4],
    metallic: 0.0,
    roughness: 0.5,
});

/// Physically Based Rendering (PBR) material component
///
/// This component defines a PBR material with full physical rendering parameters.
#[derive(Component, Clone, Debug)]
pub struct PbrMaterialComp {
    /// Base color (RGBA)
    pub base_color: [f32; 4],
    /// Metallic factor (0.0 = non-metallic, 1.0 = metallic)
    pub metallic: f32,
    /// Roughness factor (0.0 = smooth mirror, 1.0 = rough diffuse)
    pub roughness: f32,
    /// Ambient occlusion
    pub ambient_occlusion: f32,
    /// Emissive color (RGB)
    pub emissive: [f32; 3],
    /// Emissive intensity
    pub emissive_strength: f32,
}

impl_default_and_new!(PbrMaterialComp {
    base_color: [1.0, 1.0, 1.0, 1.0],
    metallic: 0.0,
    roughness: 0.5,
    ambient_occlusion: 1.0,
    emissive: [0.0, 0.0, 0.0],
    emissive_strength: 0.0,
});

/// 3D point light component
///
/// This component defines a point light source that emits light in all directions
/// from a specific position in 3D space.
#[derive(Component, Clone, Debug)]
pub struct PointLight3D {
    /// RGB color
    pub color: [f32; 3],
    /// Light intensity
    pub intensity: f32,
    /// Light radius
    pub radius: f32,
}

impl_default_and_new!(PointLight3D {
    color: [1.0, 1.0, 1.0],
    intensity: 1.0,
    radius: 10.0,
});

/// Directional light component (e.g., sunlight)
///
/// This component defines a directional light source that emits parallel light rays
/// in a specific direction, typically used to simulate sunlight.
#[derive(Component, Clone, Debug)]
pub struct DirectionalLightComp {
    /// Light direction vector
    pub direction: [f32; 3],
    /// RGB color
    pub color: [f32; 3],
    /// Light intensity
    pub intensity: f32,
}

impl_default_and_new!(DirectionalLightComp {
    direction: [0.0, -1.0, 0.0],
    color: [1.0, 1.0, 1.0],
    intensity: 1.0,
});

#[derive(Resource)]
pub struct Time {
    pub delta_seconds: f32,
    pub elapsed_seconds: f64,
    pub fixed_time_step: f64,
    pub alpha: f64,
}

impl_default!(Time {
    delta_seconds: 0.0,
    elapsed_seconds: 0.0,
    fixed_time_step: 1.0 / 60.0,
    alpha: 0.0,
});

#[derive(Component, Clone, Copy, Debug)]
pub struct PreviousTransform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TileSet {
    pub tiles: std::collections::HashMap<String, ([f32; 2], [f32; 2])>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TileChunkConfig {
    pub size: [u32; 2],
}

#[derive(Component, Clone, Debug)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: [f32; 2],
    pub tiles: Vec<String>,
    pub layer: f32,
    pub atlas_tex_index: u32,
    pub dirty: bool,
    pub chunk_size: [u32; 2],
}

pub fn tilemap_build_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut TileMap)>,
    tileset: Option<Res<TileSet>>,
    viewport: Option<Res<Viewport>>,
) {
    let ts = if let Some(ts) = tileset {
        ts
    } else {
        return;
    };
    let vp = viewport.map(|v| (v.width as f32, v.height as f32)).unwrap_or((800.0, 600.0));
    for (entity, t_base, mut tm) in query.iter_mut() {
        if !tm.dirty {
            continue;
        }
        // 记录正在更新的实体，用于调试和性能监控
        let _entity_index = entity.index();
        // naive: spawn sprites for each non-zero tile
        let w = tm.width as i32;
        let h = tm.height as i32;
        for y in 0..h {
            for x in 0..w {
                let idx = (y as u32 * tm.width + x as u32) as usize;
                let id = tm.tiles.get(idx).cloned().unwrap_or_default();
                if id.is_empty() {
                    continue;
                }
                if let Some((uv_off, uv_scale)) = ts.tiles.get(&id).cloned() {
                    let pos_x = t_base.pos.x + (x as f32 + 0.5) * tm.tile_size[0];
                    let pos_y = t_base.pos.y + (y as f32 + 0.5) * tm.tile_size[1];
                    if pos_x < -tm.tile_size[0]
                        || pos_y < -tm.tile_size[1]
                        || pos_x > vp.0 + tm.tile_size[0]
                        || pos_y > vp.1 + tm.tile_size[1]
                    {
                        continue;
                    }
                    let pos = Vec3::new(pos_x, pos_y, t_base.pos.z);
                    commands.spawn((
                        Transform {
                            pos,
                            rot: Quat::IDENTITY,
                            scale: Vec3::new(tm.tile_size[0], tm.tile_size[1], 1.0),
                        },
                        PreviousTransform::default(),
                        Sprite {
                            color: [1.0, 1.0, 1.0, 1.0],
                            tex_index: tm.atlas_tex_index,
                            normal_tex_index: 0,
                            uv_off,
                            uv_scale,
                            layer: tm.layer,
                        },
                    ));
                }
            }
        }
        tm.dirty = false;
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct TileChunks {
    pub visible: std::collections::HashSet<(i32, i32)>,
}

#[derive(Resource)]
pub struct TileEntityPool {
    unused: Vec<Entity>,
    capacity: usize,
}

impl Default for TileEntityPool {
    fn default() -> Self {
        Self {
            unused: Vec::with_capacity(1000),
            capacity: 1000,
        }
    }
}

impl TileEntityPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_spawn(&mut self, commands: &mut Commands) -> Entity {
        if let Some(entity) = self.unused.pop() {
            // 复用现有实体
            entity
        } else {
            // 创建新实体
            commands.spawn_empty().id()
        }
    }

    pub fn recycle(&mut self, entity: Entity, commands: &mut Commands) {
        // 先移除所有组件，准备复用
        commands
            .entity(entity)
            .remove::<(Transform, PreviousTransform, Sprite, ChunkTag)>();
        if self.unused.len() < self.capacity {
            self.unused.push(entity);
        } else {
            // 池已满，销毁实体
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ChunkTag {
    pub map: Entity,
    pub cx: i32,
    pub cy: i32,
}

pub fn tilemap_chunk_system(
    mut commands: Commands,
    mut pool: ResMut<TileEntityPool>,
    mut maps: Query<(Entity, &Transform, &mut TileMap, Option<&mut TileChunks>)>,
    tileset: Option<Res<TileSet>>,
    viewport: Option<Res<Viewport>>,
    cam_q: Query<(&Transform, &Camera)>,
    chunk_entities: Query<(Entity, &ChunkTag)>,
) {
    let ts = if let Some(ts) = tileset {
        ts
    } else {
        return;
    };
    let (vpw, vph) = viewport.map(|v| (v.width as f32, v.height as f32)).unwrap_or((800.0, 600.0));
    let mut cam_pos = glam::Vec3::new(vpw * 0.5, vph * 0.5, 0.0);
    for (t, c) in cam_q.iter() {
        if c.is_active {
            cam_pos = t.pos;
            break;
        }
    }
    for (map_e, t_base, tm, opt_chunks) in maps.iter_mut() {
        let current_visible = if let Some(ch) = opt_chunks.as_ref() {
            ch.visible.clone()
        } else {
            std::collections::HashSet::new()
        };
        let cx_sz = tm.chunk_size[0] as f32 * tm.tile_size[0];
        let cy_sz = tm.chunk_size[1] as f32 * tm.tile_size[1];
        let half_w = vpw * 0.5;
        let half_h = vph * 0.5;
        let view_min_x = cam_pos.x - half_w;
        let view_max_x = cam_pos.x + half_w;
        let view_min_y = cam_pos.y - half_h;
        let view_max_y = cam_pos.y + half_h;
        let start_cx = ((view_min_x - t_base.pos.x) / cx_sz).floor() as i32;
        let end_cx = ((view_max_x - t_base.pos.x) / cx_sz).ceil() as i32;
        let start_cy = ((view_min_y - t_base.pos.y) / cy_sz).floor() as i32;
        let end_cy = ((view_max_y - t_base.pos.y) / cy_sz).ceil() as i32;
        let mut new_vis: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for cy in start_cy..end_cy {
            for cx in start_cx..end_cx {
                new_vis.insert((cx, cy));
            }
        }

        // 回收不再可见的chunk中的tile实体
        for &(cx, cy) in current_visible.iter() {
            if !new_vis.contains(&(cx, cy)) {
                for (ent, tag) in chunk_entities.iter() {
                    if tag.map == map_e && tag.cx == cx && tag.cy == cy {
                        pool.recycle(ent, &mut commands);
                    }
                }
            }
        }

        // 为新可见的chunk创建tile实体，使用实体池
        for (cx, cy) in new_vis.iter() {
            if !current_visible.contains(&(*cx, *cy)) {
                let base_x = t_base.pos.x + *cx as f32 * cx_sz;
                let base_y = t_base.pos.y + *cy as f32 * cy_sz;
                let tiles_w = tm.width as i32;
                let tiles_h = tm.height as i32;
                for ty in 0..tm.chunk_size[1] as i32 {
                    for tx in 0..tm.chunk_size[0] as i32 {
                        let gx = *cx * tm.chunk_size[0] as i32 + tx;
                        let gy = *cy * tm.chunk_size[1] as i32 + ty;
                        if gx < 0 || gy < 0 || gx >= tiles_w || gy >= tiles_h {
                            continue;
                        }
                        let idx = (gy as u32 * tm.width + gx as u32) as usize;
                        if idx >= tm.tiles.len() {
                            continue;
                        }
                        let id = tm.tiles[idx].clone();
                        if id.is_empty() {
                            continue;
                        }
                        if let Some((uv_off, uv_scale)) = ts.tiles.get(&id).cloned() {
                            let pos = glam::Vec3::new(
                                base_x + (tx as f32 + 0.5) * tm.tile_size[0],
                                base_y + (ty as f32 + 0.5) * tm.tile_size[1],
                                t_base.pos.z,
                            );
                            let entity = pool.get_or_spawn(&mut commands);
                            commands.entity(entity).insert((
                                Transform {
                                    pos,
                                    rot: Quat::IDENTITY,
                                    scale: glam::Vec3::new(tm.tile_size[0], tm.tile_size[1], 1.0),
                                },
                                PreviousTransform::default(),
                                Sprite {
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    tex_index: tm.atlas_tex_index,
                                    normal_tex_index: 0,
                                    uv_off,
                                    uv_scale,
                                    layer: tm.layer,
                                },
                                ChunkTag {
                                    map: map_e,
                                    cx: *cx,
                                    cy: *cy,
                                },
                            ));
                        }
                    }
                }
            }
        }
        if let Some(mut ch) = opt_chunks {
            ch.visible = new_vis;
        } else {
            commands.entity(map_e).insert(TileChunks { visible: new_vis });
        }
    }
}

impl_default_and_new!(PreviousTransform {
    pos: Vec3::ZERO,
    rot: Quat::IDENTITY,
    scale: Vec3::ONE,
});
#[derive(Clone, Debug)]
pub struct FlipFrame {
    pub uv_off: [f32; 2],
    pub uv_scale: [f32; 2],
    pub duration: f32,
}

#[derive(Component, Clone, Debug)]
pub struct Flipbook {
    pub frames: Vec<FlipFrame>,
    pub speed: f32,
    pub looping: bool,
    pub elapsed: f32,
    pub current: usize,
}

impl_default_and_new!(Flipbook {
    frames: Vec::new(),
    speed: 1.0,
    looping: true,
    elapsed: 0.0,
    current: 0,
});

pub fn flipbook_system(mut query: Query<(&mut Sprite, &mut Flipbook)>, time: Res<Time>) {
    for (mut sprite, mut fb) in query.iter_mut() {
        if fb.frames.is_empty() {
            continue;
        }
        fb.elapsed += time.delta_seconds * fb.speed;
        let mut t = fb.elapsed;
        let mut idx = fb.current;
        while t > fb.frames[idx].duration {
            t -= fb.frames[idx].duration;
            idx += 1;
            if idx >= fb.frames.len() {
                if fb.looping {
                    idx = 0;
                } else {
                    idx = fb.frames.len() - 1;
                    break;
                }
            }
        }
        fb.current = idx;
        let fr = &fb.frames[idx];
        sprite.uv_off = fr.uv_off;
        sprite.uv_scale = fr.uv_scale;
    }
}

#[derive(Component)]
pub struct AiComponent {
    #[cfg(feature = "ai-integration")]
    pub behavior_tree:
        Option<std::sync::Arc<std::sync::Mutex<crate::ai::behavior_tree::BehaviorTree>>>,
    #[cfg(feature = "ai-integration")]
    pub state_machine:
        Option<std::sync::Arc<std::sync::Mutex<crate::ai::state_machine::StateMachine>>>,
}

#[cfg(test)]
mod tests;

// ========================================
// 综合测试模块
// ========================================

// Temporarily disabled due to missing modules
// #[cfg(test)]
// mod entity_manager_tests;

// #[cfg(test)]
// mod component_validator_tests;

#[cfg(test)]
mod soa_layout_tests;

#[cfg(test)]
mod dirty_tracking_tests;

#[cfg(test)]
mod system_integration_tests;

#[cfg(test)]
mod extended_tests;
