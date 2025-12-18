use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Component, Clone)]
pub struct TileMap {
    pub tiles: Vec<u32>, // Texture indices, 0 means empty
    pub width: u32,
    pub height: u32,
    pub tile_size: Vec2,
    pub atlas_handle: Option<crate::resources::manager::Handle<u32>>, // Or Atlas handle
}

impl_default!(TileMap {
    tiles: vec![],
    width: 0,
    height: 0,
    tile_size: Vec2::new(32.0, 32.0),
    atlas_handle: None,
});

impl TileMap {
    pub fn new(width: u32, height: u32, tile_size: Vec2) -> Self {
        Self {
            tiles: vec![0; (width * height) as usize],
            width,
            height,
            tile_size,
            atlas_handle: None,
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile_id: u32) {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile_id;
        }
    }
}
