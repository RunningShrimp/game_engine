//! OBJ-specific asset loading functionality
//!
//! This module contains all OBJ-related code including:
//! - Asset loader implementation
//! - Scene import to world
//! - Material and texture processing

#[cfg(feature = "obj")]
use bevy_ecs::prelude::World;
#[cfg(feature = "obj")]
use std::sync::Arc;

// Re-export ObjScene when feature is enabled
#[cfg(feature = "obj")]
pub use super::obj_loader::{ObjScene, ObjDocument, ObjObject, ObjMesh, ObjMaterial};

// Import Handle, generate_tangents, and MaterialRegistry from manager module
#[cfg(feature = "obj")]
use super::manager::{Handle, MaterialRegistry, generate_tangents};

/// OBJ-specific asset loading operations
#[cfg(feature = "obj")]
pub struct ObjAssetLoader;

#[cfg(feature = "obj")]
impl ObjAssetLoader {
    /// Load OBJ file from string
    pub async fn load_from_str(content: &str, base_path: &str) -> Result<ObjScene, String> {
        let content = content.to_string();
        let base_path = base_path.to_string();

        let parse_res = tokio::task::spawn_blocking(move || {
            super::obj_loader::ObjLoader::from_str(&content, &base_path)
                .map_err(|e| e.to_string())
        }).await;

        match parse_res {
            Ok(Ok(scene)) => Ok(scene),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(format!("OBJ parsing task failed: {e}")),
        }
    }
}

/// Import OBJ scene into the world
#[cfg(feature = "obj")]
pub fn import_obj_to_world(
    world: &mut World,
    renderer: &mut crate::render::wgpu_utils::WgpuRenderer,
    handle: &Handle<ObjScene>,
) {
    if let Some(scene) = handle.get() {
        let doc = &scene.data;

        // Check if pbr renderer exists first to avoid borrowing conflicts
        if renderer.pbr_renderer.is_none() {
            tracing::warn!(target: "obj_assets", "PBR renderer not initialized, cannot import OBJ");
            return;
        }

        // Process all objects
        for (obj_idx, obj) in doc.objects.iter().enumerate() {
            tracing::info!(target: "obj_assets", "Processing object {}: {} ({} vertices)",
                obj_idx, obj.name, obj.mesh.positions.len());

            // Skip empty meshes
            if obj.mesh.positions.is_empty() {
                continue;
            }

            // Generate or use existing normals
            let normals = if obj.mesh.normals.is_empty() {
                generate_normals_from_positions(&obj.mesh.positions, &obj.mesh.indices)
            } else {
                obj.mesh.normals.clone()
            };

            // Generate or use existing UVs
            let uvs = if obj.mesh.uvs.is_empty() {
                vec![[0.0, 0.0]; obj.mesh.positions.len()]
            } else {
                obj.mesh.uvs.clone()
            };

            // Convert ObjIndex to plain u32 indices
            let indices: Vec<u32> = obj.mesh.indices.iter()
                .map(|idx| idx.vertex)
                .collect();

            // Generate or use existing tangents
            let tangents = generate_tangents(&obj.mesh.positions, &normals, &uvs, &indices);

            // Build vertex array
            let mut vertices = Vec::with_capacity(obj.mesh.positions.len());
            for i in 0..obj.mesh.positions.len() {
                vertices.push(crate::render::mesh::Vertex3D {
                    pos: obj.mesh.positions[i],
                    normal: normals[i],
                    uv: uvs[i],
                    tangent: tangents[i],
                });
            }

            // Create GPU mesh
            let gpu_mesh = renderer.create_gpu_mesh(&vertices, &indices);

            // Get or create material
            let default_mat = ObjMaterial::default();
            let material = if let Some(mat_idx) = obj.material_index {
                doc.materials.get(mat_idx).unwrap_or(&default_mat)
            } else {
                &default_mat
            };

            // Convert OBJ material to PBR material
            let pbr_material = convert_obj_material_to_pbr(material);

            // Get PBR renderer
            let pbr = renderer.pbr_renderer.as_ref()
                .expect("PBR renderer must be initialized");

            // Create texture set
            let device = renderer.device();
            let queue = renderer.queue();

            // Create default 1x1 white texture
            let default_img = image::RgbaImage::from_raw(
                1, 1,
                vec![255, 255, 255, 255]
            ).expect("Failed to create default texture");

            // Load textures from OBJ material
            let base_color_img = load_texture_from_path(&material.diffuse_map)
                .unwrap_or(default_img.clone());
            let normal_img = load_texture_from_path(&material.normal_map)
                .unwrap_or(default_img.clone());
            let specular_img = load_texture_from_path(&material.specular_map)
                .unwrap_or(default_img.clone());

            // OBJ uses specular map instead of metallic/roughness
            // Use specular for roughness approximation
            let metallic_roughness_img = specular_img;

            let tex_set = pbr.create_texture_set_from_images(
                device,
                queue,
                [base_color_img, metallic_roughness_img, normal_img, default_img.clone(), default_img],
                [true, false, false, false, true],
            );
            let tex_bg = Arc::new(tex_set.bind_group);

            // Create material bind group
            let (material_bg, material_buf): (Arc<wgpu::BindGroup>, Arc<wgpu::Buffer>) =
                pbr.create_material_bind_group(device, queue, &pbr_material);

            // Register material
            let mut registry = world.get_resource_or_insert_with::<MaterialRegistry>(Default::default);
            let mat_id = obj_idx as u64;
            registry.materials.insert(mat_id, (material_bg.clone(), material_buf.clone(), tex_bg.clone()));

            // Spawn entity with mesh renderer
            let comp = crate::render::instance_batch::Mesh3DRenderer {
                mesh: gpu_mesh,
                material_bind_group: material_bg,
                textures_bind_group: Some(tex_bg),
                material_uniform_buffer: Some(material_buf),
                mesh_id: mat_id,
                material_id: mat_id,
                pipeline_id: 0,
                blend_mode: 0,
                depth_test: true,
                render_flags: 0,
                visible: true,
            };
            let transform = crate::ecs::Transform::default();
            world.spawn((comp, transform));

            tracing::info!(target: "obj_assets", "Spawned mesh entity {} ({})",
                obj_idx, obj.name);
        }
    }
}

/// Convert OBJ material to PBR material
#[cfg(feature = "obj")]
fn convert_obj_material_to_pbr(obj_mat: &ObjMaterial) -> crate::render::pbr::PbrMaterial {
    let mut mat = crate::render::pbr::PbrMaterial::default();

    // OBJ uses diffuse, we map to base_color
    mat.base_color = glam::Vec4::from_array(obj_mat.diffuse);

    // OBJ uses specular and shininess
    // Approximate metallic from specular intensity
    let specular_intensity = (obj_mat.specular[0] + obj_mat.specular[1] + obj_mat.specular[2]) / 3.0;
    mat.metallic = specular_intensity;

    // Roughness from shininess (inverse relationship)
    // Higher shininess = smoother surface = lower roughness
    mat.roughness = 1.0 - (obj_mat.shininess / 128.0).min(1.0);

    mat.emissive = glam::Vec3::ZERO;
    mat.normal_scale = 1.0;
    mat.ambient_occlusion = 1.0;

    mat
}

/// Load texture from path
#[cfg(feature = "obj")]
fn load_texture_from_path(path: &Option<String>) -> Option<image::RgbaImage> {
    let path = path.as_ref()?;
    let path = std::path::Path::new(path);

    if path.exists() {
        match image::open(path) {
            Ok(img) => Some(img.to_rgba8()),
            Err(e) => {
                tracing::warn!(target: "obj_assets", "Failed to load texture {}: {}", path.display(), e);
                None
            }
        }
    } else {
        tracing::warn!(target: "obj_assets", "Texture file not found: {}", path.display());
        None
    }
}

/// Generate smooth normals from positions and indices
#[cfg(feature = "obj")]
fn generate_normals_from_positions(positions: &[[f32; 3]], obj_indices: &[super::obj_loader::ObjIndex]) -> Vec<[f32; 3]> {
    let indices: Vec<u32> = obj_indices.iter().map(|idx| idx.vertex).collect();
    let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];

    // Calculate face normals
    for chunk in indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }

        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = glam::Vec3::from_array(positions[i0]);
        let v1 = glam::Vec3::from_array(positions[i1]);
        let v2 = glam::Vec3::from_array(positions[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2);

        // Accumulate normals
        normals[i0][0] += normal.x;
        normals[i0][1] += normal.y;
        normals[i0][2] += normal.z;

        normals[i1][0] += normal.x;
        normals[i1][1] += normal.y;
        normals[i1][2] += normal.z;

        normals[i2][0] += normal.x;
        normals[i2][1] += normal.y;
        normals[i2][2] += normal.z;
    }

    // Normalize all normals
    for normal in &mut normals {
        let len = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if len > 0.0001 {
            normal[0] /= len;
            normal[1] /= len;
            normal[2] /= len;
        } else {
            normal[1] = 1.0; // Default up
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "obj")]
    fn test_generate_normals() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![
            super::obj_loader::ObjIndex { vertex: 0, tex_coord: None, normal: None },
            super::obj_loader::ObjIndex { vertex: 1, tex_coord: None, normal: None },
            super::obj_loader::ObjIndex { vertex: 2, tex_coord: None, normal: None },
        ];

        let normals = generate_normals_from_positions(&positions, &indices);

        // All normals should point up (Z+ for this triangle)
        assert_eq!(normals.len(), 3);
        for normal in &normals {
            assert!(normal[2] > 0.9); // Should be close to 1.0
        }
    }

    #[test]
    #[cfg(feature = "obj")]
    fn test_convert_obj_material() {
        let obj_mat = ObjMaterial {
            name: "TestMaterial".to_string(),
            diffuse: [1.0, 0.5, 0.0, 1.0],
            ambient: [0.1, 0.1, 0.1],
            specular: [0.8, 0.8, 0.8],
            shininess: 64.0,
            alpha: 1.0,
            diffuse_map: None,
            normal_map: None,
            specular_map: None,
        };

        let pbr_mat = convert_obj_material_to_pbr(&obj_mat);

        assert_eq!(pbr_mat.base_color.x, 1.0);
        assert_eq!(pbr_mat.base_color.y, 0.5);
        assert_eq!(pbr_mat.base_color.z, 0.0);
        assert_eq!(pbr_mat.metallic, 0.8);
        // shininess 64 should give roughness 1.0 - 64/128 = 0.5
        assert!((pbr_mat.roughness - 0.5).abs() < 0.01);
    }
}
