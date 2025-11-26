use bevy_ecs::prelude::*;
use glam::{Vec3, Quat};

#[derive(Component, Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Velocity {
    pub lin: Vec3,
    pub ang: Vec3,
}

#[derive(Component, Clone, Debug)]
pub struct Sprite {
    pub color: [f32; 4],
    pub tex_index: u32,
    pub normal_tex_index: u32, // 0 means no normal map
    pub uv_off: [f32; 2],
    pub uv_scale: [f32; 2],
    pub layer: f32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            color: [1.0; 4],
            tex_index: 0,
            normal_tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct PointLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            radius: 100.0,
            falloff: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Orthographic { scale: f32, near: f32, far: f32 },
    Perspective { fov: f32, aspect: f32, near: f32, far: f32 },
}

impl Default for Projection {
    fn default() -> Self {
        Self::Orthographic { scale: 1.0, near: 0.0, far: 1000.0 }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Camera {
    pub is_active: bool,
    pub projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            is_active: true,
            projection: Projection::default(),
        }
    }
}

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

impl Default for Material {
    fn default() -> Self {
        Self {
            color: [1.0; 4],
            metallic: 0.0,
            roughness: 0.5,
        }
    }
}

#[derive(Resource)]
pub struct Time {
    pub delta_seconds: f32,
    pub elapsed_seconds: f64,
    pub fixed_time_step: f64,
    pub alpha: f64,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
            fixed_time_step: 1.0 / 60.0,
            alpha: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PreviousTransform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TileSet {
    pub tiles: std::collections::HashMap<String, ( [f32;2], [f32;2] )>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Component, Clone, Debug)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: [f32;2],
    pub tiles: Vec<String>,
    pub layer: f32,
    pub atlas_tex_index: u32,
    pub dirty: bool,
    pub chunk_size: [u32;2],
}

pub fn tilemap_build_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut TileMap)>,
    tileset: Option<Res<TileSet>>,
    viewport: Option<Res<Viewport>>,
) {
    let ts = if let Some(ts) = tileset { ts } else { return; };
    let vp = viewport.map(|v| (v.width as f32, v.height as f32)).unwrap_or((800.0, 600.0));
    for (entity, t_base, mut tm) in query.iter_mut() {
        if !tm.dirty { continue; }
        // naive: spawn sprites for each non-zero tile
        let w = tm.width as i32;
        let h = tm.height as i32;
        for y in 0..h {
            for x in 0..w {
                let idx = (y as u32 * tm.width + x as u32) as usize;
                let id = tm.tiles.get(idx).cloned().unwrap_or_default();
                if id.is_empty() { continue; }
                if let Some((uv_off, uv_scale)) = ts.tiles.get(&id).cloned() {
                    let pos_x = t_base.pos.x + (x as f32 + 0.5) * tm.tile_size[0];
                    let pos_y = t_base.pos.y + (y as f32 + 0.5) * tm.tile_size[1];
                    if pos_x < -tm.tile_size[0] || pos_y < -tm.tile_size[1] || pos_x > vp.0 + tm.tile_size[0] || pos_y > vp.1 + tm.tile_size[1] {
                        continue;
                    }
                    let pos = Vec3::new(pos_x, pos_y, t_base.pos.z);
                    commands.spawn((
                        Transform { pos, rot: Quat::IDENTITY, scale: Vec3::new(tm.tile_size[0], tm.tile_size[1], 1.0) },
                        PreviousTransform::default(),
                        Sprite { color: [1.0,1.0,1.0,1.0], tex_index: tm.atlas_tex_index, normal_tex_index: 0, uv_off, uv_scale, layer: tm.layer },
                    ));
                }
            }
        }
        tm.dirty = false;
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct TileChunks { pub visible: std::collections::HashSet<(i32,i32)> }

#[derive(Component, Clone, Debug)]
pub struct ChunkTag { pub map: Entity, pub cx: i32, pub cy: i32 }

pub fn tilemap_chunk_system(
    mut commands: Commands,
    mut maps: Query<(Entity, &Transform, &mut TileMap, Option<&mut TileChunks>)>,
    tileset: Option<Res<TileSet>>,
    viewport: Option<Res<Viewport>>,
    cam_q: Query<(&Transform, &Camera)>,
    chunk_entities: Query<(Entity, &ChunkTag)>,
) {
    let ts = if let Some(ts) = tileset { ts } else { return; };
    let (vpw, vph) = viewport.map(|v| (v.width as f32, v.height as f32)).unwrap_or((800.0, 600.0));
    let mut cam_pos = glam::Vec3::new(vpw*0.5, vph*0.5, 0.0);
    for (t, c) in cam_q.iter() { if c.is_active { cam_pos = t.pos; break; } }
    for (map_e, t_base, mut tm, opt_chunks) in maps.iter_mut() {
        let current_visible = if let Some(ch) = opt_chunks.as_ref() { ch.visible.clone() } else { std::collections::HashSet::new() };
        let cx_sz = tm.chunk_size[0] as f32 * tm.tile_size[0];
        let cy_sz = tm.chunk_size[1] as f32 * tm.tile_size[1];
        let half_w = vpw * 0.5; let half_h = vph * 0.5;
        let view_min_x = cam_pos.x - half_w; let view_max_x = cam_pos.x + half_w;
        let view_min_y = cam_pos.y - half_h; let view_max_y = cam_pos.y + half_h;
        let start_cx = ((view_min_x - t_base.pos.x) / cx_sz).floor() as i32;
        let end_cx = ((view_max_x - t_base.pos.x) / cx_sz).ceil() as i32;
        let start_cy = ((view_min_y - t_base.pos.y) / cy_sz).floor() as i32;
        let end_cy = ((view_max_y - t_base.pos.y) / cy_sz).ceil() as i32;
        let mut new_vis: std::collections::HashSet<(i32,i32)> = std::collections::HashSet::new();
        for cy in start_cy..end_cy { for cx in start_cx..end_cx { new_vis.insert((cx, cy)); } }
        for &(cx, cy) in current_visible.iter() {
            if !new_vis.contains(&(cx, cy)) {
                for (ent, tag) in chunk_entities.iter() {
                    if tag.map == map_e && tag.cx == cx && tag.cy == cy { commands.entity(ent).despawn(); }
                }
            }
        }
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
                        if gx < 0 || gy < 0 || gx >= tiles_w || gy >= tiles_h { continue; }
                        let idx = (gy as u32 * tm.width + gx as u32) as usize;
                        if idx >= tm.tiles.len() { continue; }
                        let id = tm.tiles[idx].clone(); if id.is_empty() { continue; }
                        if let Some((uv_off, uv_scale)) = ts.tiles.get(&id).cloned() {
                            let pos = glam::Vec3::new(
                                base_x + (tx as f32 + 0.5) * tm.tile_size[0],
                                base_y + (ty as f32 + 0.5) * tm.tile_size[1],
                                t_base.pos.z,
                            );
                            commands.spawn((
                                Transform { pos, rot: Quat::IDENTITY, scale: glam::Vec3::new(tm.tile_size[0], tm.tile_size[1], 1.0) },
                                PreviousTransform::default(),
                                Sprite { color: [1.0,1.0,1.0,1.0], tex_index: tm.atlas_tex_index, normal_tex_index: 0, uv_off, uv_scale, layer: tm.layer },
                                ChunkTag { map: map_e, cx: *cx, cy: *cy },
                            ));
                        }
                    }
                }
            }
        }
        if let Some(mut ch) = opt_chunks { ch.visible = new_vis; } else { commands.entity(map_e).insert(TileChunks { visible: new_vis }); }
    }
}

impl Default for PreviousTransform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
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

impl Default for Flipbook {
    fn default() -> Self {
        Self { frames: Vec::new(), speed: 1.0, looping: true, elapsed: 0.0, current: 0 }
    }
}

pub fn flipbook_system(mut query: Query<(&mut Sprite, &mut Flipbook)>, time: Res<Time>) {
    for (mut sprite, mut fb) in query.iter_mut() {
        if fb.frames.is_empty() { continue; }
        fb.elapsed += time.delta_seconds * fb.speed;
        let mut t = fb.elapsed;
        let mut idx = fb.current;
        while t > fb.frames[idx].duration {
            t -= fb.frames[idx].duration;
            idx += 1;
            if idx >= fb.frames.len() {
                if fb.looping { idx = 0; } else { idx = fb.frames.len() - 1; break; }
            }
        }
        fb.current = idx;
        let fr = &fb.frames[idx];
        sprite.uv_off = fr.uv_off;
        sprite.uv_scale = fr.uv_scale;
    }
}
