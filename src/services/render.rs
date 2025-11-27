use bevy_ecs::prelude::*;
use crate::render::wgpu::{WgpuRenderer, Instance};
use crate::ecs::{Transform, Mesh, PbrMaterialComp, PointLight3D as EcsPointLight3D, DirectionalLightComp};
use crate::render::mesh::GpuMesh;
use crate::render::pbr::{PbrMaterial, PointLight3D, DirectionalLight};
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;



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
#[derive(Default)]
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
        self.offscreen_cache.get(&id).is_none_or(|c| c.dirty)
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

impl Default for RenderService {
    fn default() -> Self {
        Self::new()
    }
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
    
    // ========================================================================
    // PBR Scene Building
    // ========================================================================
    
    /// 构建PBR场景 - 提取网格、材质和光源
    pub fn build_pbr_scene(&mut self, world: &mut World) -> PbrScene {
        let mut meshes = Vec::new();
        let mut point_lights = Vec::new();
        let mut dir_lights = Vec::new();
        
        // 提取带有PBR材质的网格
        let mut mesh_query = world.query::<(&Mesh, &Transform, Option<&PbrMaterialComp>)>();
        for (mesh, transform, pbr_mat) in mesh_query.iter(world) {
            if let Some(gpu_mesh) = mesh.handle.get() {
                let material = if let Some(mat) = pbr_mat {
                    PbrMaterial {
                        base_color: Vec4::from_array(mat.base_color),
                        metallic: mat.metallic,
                        roughness: mat.roughness,
                        ambient_occlusion: mat.ambient_occlusion,
                        emissive: Vec3::from_array(mat.emissive),
                        normal_scale: 1.0,
                    }
                } else {
                    PbrMaterial::default()
                };
                meshes.push((gpu_mesh.clone(), *transform, material));
            }
        }
        
        // 提取3D点光源
        let mut point_light_query = world.query::<(&Transform, &EcsPointLight3D)>();
        for (transform, light) in point_light_query.iter(world) {
            point_lights.push(PointLight3D {
                position: transform.pos,
                color: Vec3::from_array(light.color),
                intensity: light.intensity,
                radius: light.radius,
            });
        }
        
        // 提取方向光
        let mut dir_light_query = world.query::<&DirectionalLightComp>();
        for light in dir_light_query.iter(world) {
            dir_lights.push(DirectionalLight {
                direction: Vec3::from_array(light.direction),
                color: Vec3::from_array(light.color),
                intensity: light.intensity,
            });
        }
        
        PbrScene {
            meshes,
            point_lights,
            dir_lights,
        }
    }
    
    /// 执行PBR渲染
    pub fn paint_pbr(
        &mut self,
        renderer: &mut WgpuRenderer,
        scene: &PbrScene,
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
        egui_shapes: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
    ) {
        self.layer_cache.new_frame();
        
        renderer.render_pbr(
            &scene.meshes,
            &scene.point_lights,
            &scene.dir_lights,
            view_proj,
            camera_pos,
            egui_renderer,
            egui_shapes,
            pixels_per_point,
        );
    }
}

/// PBR场景数据
pub struct PbrScene {
    pub meshes: Vec<(GpuMesh, Transform, PbrMaterial)>,
    pub point_lights: Vec<PointLight3D>,
    pub dir_lights: Vec<DirectionalLight>,
}
