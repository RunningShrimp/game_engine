//! GLTF-specific asset loading functionality
//!
//! This module contains all GLTF-related code to reduce conditional compilation
//! complexity in the main manager.rs file.

use bevy_ecs::world::World;
use std::sync::Arc;

#[cfg(feature = "gltf")]
use gltf;

// Re-export GltfScene when feature is enabled
#[cfg(feature = "gltf")]
pub use super::gltf_loader::GltfScene;

// Import Handle, generate_tangents, and MaterialRegistry from manager module
use super::manager::{Handle, MaterialRegistry, generate_tangents};

/// GLTF-specific asset loading operations
#[cfg(feature = "gltf")]
pub struct GltfAssetLoader;

#[cfg(feature = "gltf")]
impl GltfAssetLoader {
    /// Load GLTF file from bytes
    pub async fn load_from_bytes(bytes: Vec<u8>) -> Result<GltfScene, String> {
        let bytes_for_import = bytes.clone();
        let load_res =
            tokio::task::spawn_blocking(move || gltf::import_slice(&bytes_for_import)).await;

        match load_res {
            Ok(Ok(data)) => {
                // Try to parse JSON (.gltf), GLB will fail and return None
                let json = String::from_utf8(bytes.clone())
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
                Ok(GltfScene {
                    data: Arc::new(data),
                    json,
                })
            }
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Import GLTF scene into the world
#[cfg(feature = "gltf")]
pub fn import_gltf_to_world(
    world: &mut World,
    renderer: &mut crate::render::wgpu_utils::WgpuRenderer,
    handle: &Handle<GltfScene>,
) {
    if let Some(scene) = handle.get() {
        let (doc, buffers, images) = &*scene.data;

        // Check if pbr renderer exists first to avoid borrowing conflicts
        if renderer.pbr_renderer.is_none() {
            return;
        }

        // Process all primitives to collect mesh data and material info separately
        let mut primitive_data = Vec::new();

        for mesh in doc.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));
                let positions: Vec<[f32; 3]> =
                    reader.read_positions().map(|it| it.collect()).unwrap_or_default();
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|it| it.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

                // Choose UV set index: prioritize baseColor, then MR/normal/AO/emissive
                let mut texcoord_index = 0u32;
                let mt = primitive.material();
                if let Some(info) = mt.pbr_metallic_roughness().base_color_texture() {
                    texcoord_index = info.tex_coord();
                } else if let Some(info) = mt.pbr_metallic_roughness().metallic_roughness_texture()
                {
                    texcoord_index = info.tex_coord();
                } else if let Some(info) = mt.normal_texture() {
                    texcoord_index = info.tex_coord();
                } else if let Some(info) = mt.occlusion_texture() {
                    texcoord_index = info.tex_coord();
                } else if let Some(info) = mt.emissive_texture() {
                    texcoord_index = info.tex_coord();
                }

                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(texcoord_index)
                    .map(|tc| tc.into_f32())
                    .map(|it| it.collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                let mut tangents: Vec<[f32; 4]> =
                    reader.read_tangents().map(|it| it.collect()).unwrap_or_default();

                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|r| r.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());

                if tangents.is_empty() {
                    tangents = generate_tangents(&positions, &normals, &uvs, &indices);
                }

                let mut vertices = Vec::with_capacity(positions.len());
                for i in 0..positions.len() {
                    vertices.push(crate::render::mesh::Vertex3D {
                        pos: positions[i],
                        normal: normals[i],
                        uv: uvs[i],
                        tangent: tangents[i],
                    });
                }

                // GLTF material parameter mapping
                let mr = primitive.material().pbr_metallic_roughness();
                let mut mat = crate::render::pbr::PbrMaterial::default();
                let base = mr.base_color_factor();
                mat.base_color = glam::Vec4::from_array(base);
                mat.metallic = mr.metallic_factor();
                mat.roughness = mr.roughness_factor();
                mat.emissive = glam::Vec3::from_array(primitive.material().emissive_factor());
                mat.normal_scale =
                    primitive.material().normal_texture().map(|n| n.scale()).unwrap_or(1.0);
                mat.ambient_occlusion =
                    primitive.material().occlusion_texture().map(|o| o.strength()).unwrap_or(1.0);

                // KHR_texture_transform (UV transform) parsing (only for .gltf JSON)
                let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));
                let mut final_vertices = vertices;
                if let Some(ref json) = scene.json
                    && let Some(materials) = json.get("materials").and_then(|v| v.as_array())
                    && let Some(mi) = primitive.material().index()
                    && let Some(mj) = materials.get(mi)
                    && let Some(pbr_json) = mj.get("pbrMetallicRoughness")
                    && let Some(bct) = pbr_json.get("baseColorTexture")
                    && let Some(ext) = bct.get("extensions")
                    && let Some(tt) = ext.get("KHR_texture_transform")
                {
                    if let Some(off) = tt.get("offset").and_then(|x| x.as_array())
                        && off.len() >= 2
                    {
                        mat.uv_offset = [
                            off[0].as_f64().unwrap_or(0.0) as f32,
                            off[1].as_f64().unwrap_or(0.0) as f32,
                        ];
                    }
                    if let Some(scl) = tt.get("scale").and_then(|x| x.as_array())
                        && scl.len() >= 2
                    {
                        mat.uv_scale = [
                            scl[0].as_f64().unwrap_or(1.0) as f32,
                            scl[1].as_f64().unwrap_or(1.0) as f32,
                        ];
                    }
                    if let Some(rot) = tt.get("rotation").and_then(|x| x.as_f64()) {
                        mat.uv_rotation = rot as f32;
                    }
                    if let Some(tc) = tt.get("texCoord").and_then(|x| x.as_u64()) {
                        let tc_i = tc as u32;
                        let uvs2: Vec<[f32; 2]> = reader
                            .read_tex_coords(tc_i)
                            .map(|tc| tc.into_f32())
                            .map(|it| it.collect())
                            .unwrap_or_else(|| {
                                // Get original UVs if the alternate set doesn't exist
                                let mut texcoord_index = 0u32;
                                let mt = primitive.material();
                                if let Some(info) = mt.pbr_metallic_roughness().base_color_texture()
                                {
                                    texcoord_index = info.tex_coord();
                                } else if let Some(info) =
                                    mt.pbr_metallic_roughness().metallic_roughness_texture()
                                {
                                    texcoord_index = info.tex_coord();
                                } else if let Some(info) = mt.normal_texture() {
                                    texcoord_index = info.tex_coord();
                                } else if let Some(info) = mt.occlusion_texture() {
                                    texcoord_index = info.tex_coord();
                                } else if let Some(info) = mt.emissive_texture() {
                                    texcoord_index = info.tex_coord();
                                }
                                reader
                                    .read_tex_coords(texcoord_index)
                                    .map(|tc| tc.into_f32())
                                    .map(|it| it.collect())
                                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()])
                            });
                        // Replace with new UVs
                        for i in 0..final_vertices.len() {
                            if i < uvs2.len() {
                                final_vertices[i].uv = uvs2[i];
                            }
                        }
                    }
                }

                // Store the vertex and index data for later processing
                let mesh_id = mesh.index() as u64;
                let mat_id = primitive.material().index().unwrap_or(0) as u64;

                // Store primitive data for later processing to avoid borrowing conflicts
                primitive_data.push((
                    final_vertices,
                    indices,
                    mat,
                    mat_id,
                    mesh_id,
                    primitive.material().index().unwrap_or(0),
                ));
            }
        }

        // Now process the collected data using the renderer
        for (vertices, indices, mat, mat_id, mesh_id, material_index) in primitive_data {
            // Create GPU mesh (this requires mutable access to renderer)
            let gpu_mesh = renderer.create_gpu_mesh(&vertices, &indices);

            // Now we can safely use the pbr renderer (immutable access) with device and queue
            let pbr = renderer
                .pbr_renderer
                .as_ref()
                .expect("PBR renderer must be initialized to import GLTF assets");
            let device = renderer.device();
            let queue = renderer.queue();

            // Build texture binding groups (five textures) and persist textures
            // Create 1x1 white default texture (fixed parameters, won't fail)
            let default_img = image::RgbaImage::from_raw(1, 1, vec![255, 255, 255, 255])
                .expect("Failed to create default 1x1 white image - this should never happen");

            let materials: Vec<gltf::Material> = scene.data.0.materials().collect();
            let material = &materials[material_index];
            let mr = material.pbr_metallic_roughness();

            let bc_img = mr
                .base_color_texture()
                .map(|info| &images[info.texture().source().index()])
                .map(to_rgba)
                .unwrap_or(default_img.clone());

            let mr_img = mr
                .metallic_roughness_texture()
                .map(|info| &images[info.texture().source().index()])
                .map(to_rgba)
                .unwrap_or(default_img.clone());

            let n_img = material
                .normal_texture()
                .map(|info| &images[info.texture().source().index()])
                .map(to_rgba)
                .unwrap_or(default_img.clone());

            let ao_img = material
                .occlusion_texture()
                .map(|info| &images[info.texture().source().index()])
                .map(to_rgba)
                .unwrap_or(default_img.clone());

            let em_img = material
                .emissive_texture()
                .map(|info| &images[info.texture().source().index()])
                .map(to_rgba)
                .unwrap_or(default_img.clone());

            let tex_set = pbr.create_texture_set_from_images(
                device,
                queue,
                [bc_img, mr_img, n_img, ao_img, em_img],
                [true, false, false, false, true],
            );
            let tex_bg = std::sync::Arc::new(tex_set.bind_group);

            // Material registration and reuse
            let mut registry =
                world.get_resource_or_insert_with::<MaterialRegistry>(Default::default);

            let (material_bg, material_buf): (
                std::sync::Arc<wgpu::BindGroup>,
                std::sync::Arc<wgpu::Buffer>,
            ) = if let Some(triple) = registry.materials.get(&mat_id) {
                // Use already registered material
                (triple.0.clone(), triple.1.clone())
            } else {
                let (bg, buf): (
                    std::sync::Arc<wgpu::BindGroup>,
                    std::sync::Arc<wgpu::Buffer>,
                ) = pbr.create_material_bind_group(device, queue, &mat);
                // Register material, including texture bind group
                registry.materials.insert(mat_id, (bg.clone(), buf.clone(), tex_bg.clone()));
                (bg, buf)
            };

            let comp = crate::render::instance_batch::Mesh3DRenderer {
                mesh: gpu_mesh,
                material_bind_group: material_bg,
                textures_bind_group: Some(tex_bg),
                material_uniform_buffer: Some(material_buf),
                mesh_id,
                material_id: mat_id,
                pipeline_id: 0,   // Default pipeline ID
                blend_mode: 0,    // Default blend mode (opaque)
                depth_test: true, // Enable depth test by default
                render_flags: 0,  // No special render flags
                visible: true,
            };
            let transform = crate::ecs::Transform::default();
            world.spawn((comp, transform));
        }
    }
}

/// Convert gltf image data to RGBA image format
#[cfg(feature = "gltf")]
pub fn to_rgba(data: &gltf::image::Data) -> image::RgbaImage {
    match data.format {
        gltf::image::Format::R8G8B8A8 => {
            image::RgbaImage::from_raw(data.width, data.height, data.pixels.clone())
                .unwrap_or_else(|| image::RgbaImage::new(data.width, data.height))
        }
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity((data.width * data.height * 4) as usize);
            for i in (0..data.pixels.len()).step_by(3) {
                rgba.extend_from_slice(&[
                    data.pixels[i],
                    data.pixels[i + 1],
                    data.pixels[i + 2],
                    255,
                ]);
            }
            image::RgbaImage::from_raw(data.width, data.height, rgba).unwrap_or_else(|| {
                // If image creation fails (usually due to size mismatch), create a default image
                log::warn!(
                    "Failed to create image from raw data ({}x{}), using default",
                    data.width,
                    data.height
                );
                image::RgbaImage::new(data.width.max(1), data.height.max(1))
            })
        }
        _ => image::RgbaImage::new(data.width, data.height),
    }
}
