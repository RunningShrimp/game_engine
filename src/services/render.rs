use bevy_ecs::prelude::*;
use crate::render::wgpu::{WgpuRenderer, Instance};
use crate::ecs::{Transform, Mesh};
use crate::render::mesh::GpuMesh;
use glam::Mat4;
use std::collections::HashMap;

/// Flutter-style Layer Types for compositing
#[derive(Clone, Debug)]
pub enum Layer {
    /// Leaf layer: directly paints content
    Picture {
        commands: Vec<DrawCommand>,
        transform: Mat4,
    },
    /// Container layer: composites children with transform
    Transform {
        matrix: Mat4,
        children: Vec<Layer>,
    },
    /// Opacity layer: applies alpha to subtree
    Opacity {
        alpha: f32,
        child: Box<Layer>,
    },
    /// Clip layer: clips content to rect
    ClipRect {
        rect: [f32; 4], // x, y, w, h
        child: Box<Layer>,
    },
    /// Offscreen layer: renders to texture for caching
    Offscreen {
        id: u32,
        size: [u32; 2],
        child: Box<Layer>,
    },
}

/// Low-level draw commands
#[derive(Clone, Debug)]
pub enum DrawCommand {
    DrawMesh {
        mesh: GpuMesh,
        transform: Mat4,
    },
    DrawSprite {
        texture_id: u32,
        rect: [f32; 4],
        uv_rect: [f32; 4],
        color: [f32; 4],
    },
    DrawRect {
        rect: [f32; 4],
        color: [f32; 4],
        radius: f32,
    },
}

/// Represents a node in the visual tree (Flutter-style RenderObject)
#[derive(Clone)]
pub enum RenderObject {
    Mesh {
        mesh: GpuMesh,
        transform: Transform,
    },
    Sprite {
        texture_id: u32,
        rect: [f32; 4],
        transform: Transform,
    },
    Container {
        transform: Transform,
        children: Vec<RenderObject>,
    },
    Opacity {
        opacity: f32,
        child: Box<RenderObject>,
    },
}

/// Cache for differential rendering
pub struct LayerCache {
    /// Cached offscreen textures by ID
    offscreen_cache: HashMap<u32, CachedTexture>,
    /// Frame counter for LRU eviction
    frame_count: u64,
}

struct CachedTexture {
    last_used_frame: u64,
    dirty: bool,
}

impl Default for LayerCache {
    fn default() -> Self {
        Self {
            offscreen_cache: HashMap::new(),
            frame_count: 0,
        }
    }
}

impl LayerCache {
    pub fn new_frame(&mut self) {
        self.frame_count += 1;
        // Evict textures not used for 60 frames
        self.offscreen_cache.retain(|_, v| {
            self.frame_count - v.last_used_frame < 60
        });
    }
    
    pub fn mark_used(&mut self, id: u32) {
        if let Some(cached) = self.offscreen_cache.get_mut(&id) {
            cached.last_used_frame = self.frame_count;
        }
    }
    
    pub fn is_dirty(&self, id: u32) -> bool {
        self.offscreen_cache.get(&id).map_or(true, |c| c.dirty)
    }
    
    pub fn mark_clean(&mut self, id: u32) {
        if let Some(cached) = self.offscreen_cache.get_mut(&id) {
            cached.dirty = false;
        } else {
            self.offscreen_cache.insert(id, CachedTexture {
                last_used_frame: self.frame_count,
                dirty: false,
            });
        }
    }
}

pub struct RenderService {
    /// Layer cache for differential updates
    pub layer_cache: LayerCache,
    /// Scene tree from last frame
    pub last_frame_objects: Vec<RenderObject>,
}

impl RenderService {
    pub fn new() -> Self {
        Self {
            layer_cache: LayerCache::default(),
            last_frame_objects: Vec::new(),
        }
    }

    /// The "Build" phase: Construct the Render Tree from ECS data
    pub fn build_scene(&mut self, world: &mut World) -> Vec<RenderObject> {
        let mut scene = Vec::new();

        // Query Meshes
        let mut query_mesh = world.query::<(&Mesh, &Transform)>();
        for (mesh, t) in query_mesh.iter(world) {
            if let Some(gpu_mesh) = mesh.handle.get() {
                scene.push(RenderObject::Mesh {
                    mesh: gpu_mesh.clone(),
                    transform: *t,
                });
            }
        }

        scene
    }
    
    /// Convert RenderObject tree to Layer tree for compositing
    pub fn build_layer_tree(&self, scene: &[RenderObject]) -> Layer {
        let mut commands = Vec::new();
        
        for obj in scene {
            self.collect_draw_commands(obj, Mat4::IDENTITY, &mut commands);
        }
        
        Layer::Picture {
            commands,
            transform: Mat4::IDENTITY,
        }
    }
    
    fn collect_draw_commands(&self, obj: &RenderObject, parent: Mat4, out: &mut Vec<DrawCommand>) {
        match obj {
            RenderObject::Mesh { mesh, transform } => {
                let local = Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
                let global = parent * local;
                out.push(DrawCommand::DrawMesh {
                    mesh: mesh.clone(),
                    transform: global,
                });
            }
            RenderObject::Container { transform, children } => {
                let local = Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
                let global = parent * local;
                for child in children {
                    self.collect_draw_commands(child, global, out);
                }
            }
            RenderObject::Opacity { opacity: _, child } => {
                // TODO: Handle opacity in compositing pass
                self.collect_draw_commands(child, parent, out);
            }
            RenderObject::Sprite { .. } => {
                // TODO: Handle 2D sprites
            }
        }
    }

    /// The "Paint" phase: Flatten the tree into draw calls
    pub fn paint(
        &mut self, 
        renderer: &mut WgpuRenderer, 
        scene: &[RenderObject], 
        instances: &[Instance],
        view_proj: [[f32; 4]; 4],
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
        egui_shapes: &[egui::ClippedPrimitive],
        pixels_per_point: f32
    ) {
        // Update cache
        self.layer_cache.new_frame();
        
        let mut mesh_draws: Vec<(GpuMesh, Transform)> = Vec::new();
        let mut _instance_draws: Vec<Instance> = Vec::new(); 

        for obj in scene {
            self.traverse_recursive(obj, Mat4::IDENTITY, &mut mesh_draws, &mut _instance_draws);
        }

        renderer.render(instances, &mesh_draws, view_proj, egui_renderer, egui_shapes, pixels_per_point);
    }

    fn traverse_recursive(
        &self, 
        obj: &RenderObject, 
        parent_transform: Mat4, 
        mesh_draws: &mut Vec<(GpuMesh, Transform)>,
        _instance_draws: &mut Vec<Instance>
    ) {
        match obj {
            RenderObject::Mesh { mesh, transform } => {
                let local = Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
                let global = parent_transform * local;
                let (scale, rot, pos) = global.to_scale_rotation_translation();
                mesh_draws.push((mesh.clone(), Transform { pos, rot, scale }));
            },
            RenderObject::Container { transform, children } => {
                let local = Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
                let global = parent_transform * local;
                for child in children {
                    self.traverse_recursive(child, global, mesh_draws, _instance_draws);
                }
            },
            _ => {}
        }
    }
}
