//! FBX-specific asset loading functionality
//!
//! This module contains all FBX-related code including:
//! - Asset loader implementation
//! - Scene import to world
//! - Material and texture processing

#[cfg(feature = "fbx")]
use bevy_ecs::prelude::World;
#[cfg(feature = "fbx")]
use std::sync::Arc;

// Re-export FbxScene when feature is enabled
#[cfg(feature = "fbx")]
pub use super::fbx_loader::{
    FbxDocument, FbxGlobalSettings, FbxMaterial, FbxMesh, FbxScene, FbxTexture,
};

// Import Handle, generate_tangents, and MaterialRegistry from manager module
#[cfg(feature = "fbx")]
use super::manager::{Handle, MaterialRegistry, generate_tangents};

/// FBX-specific asset loading operations
#[cfg(feature = "fbx")]
pub struct FbxAssetLoader;

#[cfg(feature = "fbx")]
impl FbxAssetLoader {
    /// Load FBX file from bytes
    pub async fn load_from_bytes(bytes: Vec<u8>) -> Result<FbxScene, String> {
        let bytes_for_parse = bytes.clone();
        let parse_res = tokio::task::spawn_blocking(move || {
            super::fbx_loader::FbxLoader::from_bytes(&bytes_for_parse).map_err(|e| e.to_string())
        })
        .await;

        match parse_res {
            Ok(Ok(scene)) => Ok(scene),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(format!("FBX parsing task failed: {e}")),
        }
    }
}

/// Import FBX scene into the world
#[cfg(feature = "fbx")]
pub fn import_fbx_to_world(
    world: &mut World,
    renderer: &mut crate::render::wgpu_utils::WgpuRenderer,
    handle: &Handle<FbxScene>,
) {
    if let Some(scene) = handle.get() {
        let doc = &scene.data;

        // Check if pbr renderer exists first to avoid borrowing conflicts
        if renderer.pbr_renderer.is_none() {
            tracing::warn!(target: "fbx_assets", "PBR renderer not initialized, cannot import FBX");
            return;
        }

        // Process all meshes
        for (mesh_idx, fbx_mesh) in doc.meshes.iter().enumerate() {
            tracing::info!(target: "fbx_assets", "Processing mesh {}: {} ({} vertices)",
                mesh_idx, fbx_mesh.name, fbx_mesh.positions.len());

            // Skip empty meshes
            if fbx_mesh.positions.is_empty() {
                continue;
            }

            // Generate or use existing normals
            let normals = match fbx_mesh.normals.is_empty() {
                true => generate_normals(&fbx_mesh.positions, &fbx_mesh.indices),
                false => fbx_mesh.normals.clone(),
            };

            // Generate or use existing UVs
            let uvs = match fbx_mesh.uvs.is_empty() {
                true => vec![[0.0, 0.0]; fbx_mesh.positions.len()],
                false => fbx_mesh.uvs.clone(),
            };

            // Generate or use existing tangents
            let tangents = match fbx_mesh.tangents.is_empty() {
                true => generate_tangents(&fbx_mesh.positions, &normals, &uvs, &fbx_mesh.indices),
                false => fbx_mesh.tangents.clone(),
            };

            // Build vertex array
            let mut vertices = Vec::with_capacity(fbx_mesh.positions.len());
            for i in 0..fbx_mesh.positions.len() {
                vertices.push(crate::render::mesh::Vertex3D {
                    pos: fbx_mesh.positions[i],
                    normal: normals[i],
                    uv: uvs[i],
                    tangent: tangents[i],
                });
            }

            // Create GPU mesh
            let gpu_mesh = renderer.create_gpu_mesh(&vertices, &fbx_mesh.indices);

            // Get or create material
            // TODO: Get material index from mesh attribute
            let default_mat = FbxMaterial::default();
            let material = doc.materials.get(0).unwrap_or(&default_mat);

            // Convert FBX material to PBR material
            let pbr_material = convert_fbx_material_to_pbr(material);

            // Get PBR renderer
            let pbr = renderer.pbr_renderer.as_ref().expect("PBR renderer must be initialized");

            // Create texture set
            let device = renderer.device();
            let queue = renderer.queue();

            // Create default 1x1 white texture
            let default_img = image::RgbaImage::from_raw(1, 1, vec![255, 255, 255, 255])
                .expect("Failed to create default texture");

            // Load textures from FBX material
            let base_color_img = load_fbx_texture(&doc.textures, &material.textures.base_color)
                .unwrap_or(default_img.clone());
            let metallic_roughness_img =
                load_fbx_texture(&doc.textures, &material.textures.metallic_roughness)
                    .unwrap_or(default_img.clone());
            let normal_img = load_fbx_texture(&doc.textures, &material.textures.normal)
                .unwrap_or(default_img.clone());
            let occlusion_img = load_fbx_texture(&doc.textures, &material.textures.occlusion)
                .unwrap_or(default_img.clone());
            let emissive_img = load_fbx_texture(&doc.textures, &material.textures.emissive)
                .unwrap_or(default_img.clone());

            let tex_set = pbr.create_texture_set_from_images(
                device,
                queue,
                [
                    base_color_img,
                    metallic_roughness_img,
                    normal_img,
                    occlusion_img,
                    emissive_img,
                ],
                [true, false, false, false, true],
            );
            let tex_bg = Arc::new(tex_set.bind_group);

            // Create material bind group
            let (material_bg, material_buf): (Arc<wgpu::BindGroup>, Arc<wgpu::Buffer>) =
                pbr.create_material_bind_group(device, queue, &pbr_material);

            // Register material
            let mut registry =
                world.get_resource_or_insert_with::<MaterialRegistry>(Default::default);
            let mat_id = mesh_idx as u64;
            registry.materials.insert(
                mat_id,
                (material_bg.clone(), material_buf.clone(), tex_bg.clone()),
            );

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

            tracing::info!(target: "fbx_assets", "Spawned mesh entity {} ({})",
                mesh_idx, fbx_mesh.name);
        }

        // Process skeletons and animation data
        if !doc.skeletons.is_empty() {
            tracing::info!(target: "fbx_assets", "Processing {} skeletons", doc.skeletons.len());
            // TODO: Import skeletons and animation data
        }

        if !doc.animations.is_empty() {
            tracing::info!(target: "fbx_assets", "Processing {} animations", doc.animations.len());
            // TODO: Import animation clips
        }
    }
}

/// Convert FBX material to PBR material
#[cfg(feature = "fbx")]
fn convert_fbx_material_to_pbr(fbx_mat: &FbxMaterial) -> crate::render::pbr::PbrMaterial {
    let mut mat = crate::render::pbr::PbrMaterial::default();

    mat.base_color = glam::Vec4::from_array(fbx_mat.base_color);
    mat.metallic = fbx_mat.metallic;
    mat.roughness = fbx_mat.roughness;
    mat.emissive = glam::Vec3::from_array(fbx_mat.emissive);
    mat.normal_scale = fbx_mat.normal_scale;
    mat.ambient_occlusion = 1.0; // FBX may not have this

    // UV transform from base color texture
    if let Some(ref tex_name) = fbx_mat.textures.base_color {
        if let Some(tex) = find_texture_by_name(tex_name) {
            mat.uv_offset = tex.transform.offset;
            mat.uv_scale = tex.transform.scale;
            mat.uv_rotation = tex.transform.rotation;
        }
    }

    mat
}

/// Load texture from FBX texture reference
#[cfg(feature = "fbx")]
fn load_fbx_texture(
    textures: &[FbxTexture],
    texture_ref: &Option<String>,
) -> Option<image::RgbaImage> {
    let tex_name = texture_ref.as_ref()?;
    let tex = find_texture_by_name(tex_name)?;

    // Try to load texture from path
    let path = std::path::Path::new(&tex.path);
    if path.exists() {
        match image::open(path) {
            Ok(img) => Some(img.to_rgba8()),
            Err(e) => {
                tracing::warn!(target: "fbx_assets", "Failed to load texture {}: {}", tex.path, e);
                None
            }
        }
    } else {
        tracing::warn!(target: "fbx_assets", "Texture file not found: {}", tex.path);
        None
    }
}

/// Find texture by name in texture list
#[cfg(feature = "fbx")]
fn find_texture_by_name(_name: &str) -> Option<FbxTexture> {
    // TODO: Implement texture lookup
    None
}

/// Generate smooth normals from positions and indices
#[cfg(feature = "fbx")]
fn generate_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
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
    #[cfg(feature = "fbx")]
    fn test_generate_normals() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2];

        let normals = generate_normals(&positions, &indices);

        // All normals should point up (Z+ for this triangle)
        assert_eq!(normals.len(), 3);
        for normal in &normals {
            assert!(normal[2] > 0.9); // Should be close to 1.0
        }
    }

    #[test]
    #[cfg(feature = "fbx")]
    fn test_convert_fbx_material() {
        let fbx_mat = FbxMaterial {
            name: "TestMaterial".to_string(),
            material_type: "PBR".to_string(),
            base_color: [1.0, 0.5, 0.0, 1.0],
            metallic: 0.8,
            roughness: 0.3,
            emissive: [0.1, 0.1, 0.1],
            normal_scale: 1.5,
            textures: Default::default(),
        };

        let pbr_mat = convert_fbx_material_to_pbr(&fbx_mat);

        assert_eq!(pbr_mat.base_color.x, 1.0);
        assert_eq!(pbr_mat.base_color.y, 0.5);
        assert_eq!(pbr_mat.base_color.z, 0.0);
        assert_eq!(pbr_mat.metallic, 0.8);
        assert_eq!(pbr_mat.roughness, 0.3);
        assert_eq!(pbr_mat.normal_scale, 1.5);
    }
}
