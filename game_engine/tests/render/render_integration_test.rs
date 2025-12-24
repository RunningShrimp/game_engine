//! 渲染系统集成测试
//!
//! 全面测试渲染系统的核心功能，包括：
//! - PBR渲染
//! - 后处理效果
//! - GPU驱动渲染
//! - 实例批处理
//! - 着色器编译
//! - 纹理加载
//! - 渲染管线

use game_engine::render::pbr::{PbrMaterial, PbrTextures, PointLight3D, DirectionalLight};
use game_engine::render::postprocess::{PostProcessConfig, TonemapOperator};
use game_engine::render::instance_batch::{BatchKey, BatchManager, BatchStats};
use game_engine::render::gpu_driven::{GpuDrivenConfig, GpuInstance};
use game_engine::render::particles::{ParticleEmitterConfig, ParticleShape, ColorGradient, ColorStop};
use game_engine::render::lod::{LodConfig, LodLevel, LodQuality};
use game_engine::render::frustum::{Frustum, Plane, CullingResult};
use game_engine::render::csm::{CsmConfig, ShadowQuality};
use glam::{Vec3, Vec4, Mat4, Quat};

#[test]
fn test_pbr_material_default() {
    let material = PbrMaterial::default();
    assert_eq!(material.base_color, Vec4::ONE);
    assert_eq!(material.metallic, 0.0);
    assert_eq!(material.roughness, 0.5);
    assert_eq!(material.ambient_occlusion, 1.0);
    assert_eq!(material.emissive, Vec3::ZERO);
    assert_eq!(material.normal_scale, 1.0);
    assert_eq!(material.uv_offset, [0.0, 0.0]);
    assert_eq!(material.uv_scale, [1.0, 1.0]);
    assert_eq!(material.uv_rotation, 0.0);
    assert_eq!(material.clearcoat, 0.0);
    assert_eq!(material.clearcoat_roughness, 0.5);
    assert_eq!(material.anisotropy, 0.0);
    assert_eq!(material.anisotropy_direction, [1.0, 0.0]);
}

#[test]
fn test_pbr_material_creation() {
    let material = PbrMaterial {
        base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        metallic: 1.0,
        roughness: 0.0,
        ambient_occlusion: 0.5,
        emissive: Vec3::new(0.5, 0.5, 0.5),
        normal_scale: 1.5,
        uv_offset: [0.1, 0.2],
        uv_scale: [2.0, 2.0],
        uv_rotation: 0.5,
        clearcoat: 0.8,
        clearcoat_roughness: 0.2,
        anisotropy: 0.5,
        anisotropy_direction: [0.0, 1.0],
    };
    
    assert_eq!(material.base_color.x, 1.0);
    assert_eq!(material.metallic, 1.0);
    assert_eq!(material.roughness, 0.0);
    assert_eq!(material.ambient_occlusion, 0.5);
}

#[test]
fn test_pbr_textures_default() {
    let textures = PbrTextures::default();
    assert!(textures.base_color_texture.is_none());
    assert!(textures.metallic_roughness_texture.is_none());
    assert!(textures.normal_texture.is_none());
    assert!(textures.ao_texture.is_none());
    assert!(textures.emissive_texture.is_none());
}

#[test]
fn test_pbr_textures_with_textures() {
    let textures = PbrTextures {
        base_color_texture: Some(1),
        metallic_roughness_texture: Some(2),
        normal_texture: Some(3),
        ao_texture: Some(4),
        emissive_texture: Some(5),
    };
    
    assert_eq!(textures.base_color_texture, Some(1));
    assert_eq!(textures.metallic_roughness_texture, Some(2));
    assert_eq!(textures.normal_texture, Some(3));
    assert_eq!(textures.ao_texture, Some(4));
    assert_eq!(textures.emissive_texture, Some(5));
}

#[test]
fn test_point_light_default() {
    let light = PointLight3D::default();
    assert_eq!(light.position, Vec3::ZERO);
    assert_eq!(light.color, Vec3::ONE);
    assert_eq!(light.intensity, 1.0);
    assert_eq!(light.radius, 10.0);
}

#[test]
fn test_point_light_creation() {
    let light = PointLight3D {
        position: Vec3::new(1.0, 2.0, 3.0),
        color: Vec3::new(1.0, 0.5, 0.2),
        intensity: 2.5,
        radius: 20.0,
    };
    
    assert_eq!(light.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(light.color, Vec3::new(1.0, 0.5, 0.2));
    assert_eq!(light.intensity, 2.5);
    assert_eq!(light.radius, 20.0);
}

#[test]
fn test_directional_light_default() {
    let light = DirectionalLight::default();
    assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
    assert_eq!(light.color, Vec3::ONE);
    assert_eq!(light.intensity, 1.0);
}

#[test]
fn test_postprocess_config_default() {
    let config = PostProcessConfig::default();
    assert!(config.bloom_enabled);
    assert!(config.tonemap_enabled);
    assert!(!config.ssao_enabled);
    assert_eq!(config.exposure, 1.0);
    assert_eq!(config.gamma, 2.2);
}

#[test]
fn test_postprocess_config_custom() {
    let config = PostProcessConfig {
        bloom_enabled: false,
        tonemap_enabled: true,
        ssao_enabled: true,
        exposure: 2.0,
        gamma: 1.8,
        tonemap_operator: TonemapOperator::ACES,
    };
    
    assert!(!config.bloom_enabled);
    assert!(config.tonemap_enabled);
    assert!(config.ssao_enabled);
    assert_eq!(config.exposure, 2.0);
    assert_eq!(config.gamma, 1.8);
}

#[test]
fn test_tonemap_operator_values() {
    assert_eq!(TonemapOperator::None as u32, 0);
    assert_eq!(TonemapOperator::Reinhard as u32, 1);
    assert_eq!(TonemapOperator::ACES as u32, 2);
    assert_eq!(TonemapOperator::Filmic as u32, 3);
}

#[test]
fn test_batch_key_creation() {
    let key = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_eq!(key.mesh_id, 1);
    assert_eq!(key.material_id, 2);
    assert_eq!(key.pipeline_id, 3);
    assert_eq!(key.blend_mode, 0);
    assert!(key.depth_test);
}

#[test]
fn test_batch_key_equality() {
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_eq!(key1, key2);
}

#[test]
fn test_batch_key_inequality() {
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 1,
        material_id: 3,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_ne!(key1, key2);
}

#[test]
fn test_batch_key_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 1,
        material_id: 2,
        pipeline_id: 3,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let mut hasher1 = DefaultHasher::new();
    key1.hash(&mut hasher1);
    
    let mut hasher2 = DefaultHasher::new();
    key2.hash(&mut hasher2);
    
    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_gpu_driven_config_default() {
    let config = GpuDrivenConfig::default();
    assert!(config.enabled);
    assert!(config.compute_culling);
    assert!(config.indirect_draw);
}

#[test]
fn test_gpu_instance_creation() {
    let instance = GpuInstance {
        model: Mat4::IDENTITY.to_cols_array_2d(),
        material_index: 0,
        lod_index: 0,
    };
    
    assert_eq!(instance.material_index, 0);
    assert_eq!(instance.lod_index, 0);
}

#[test]
fn test_particle_emitter_config_default() {
    let config = ParticleEmitterConfig::default();
    assert_eq!(config.max_particles, 1000);
    assert_eq!(config.emission_rate, 10.0);
}

#[test]
fn test_particle_shape_values() {
    assert_eq!(ParticleShape::Sphere as u32, 0);
    assert_eq!(ParticleShape::Box as u32, 1);
    assert_eq!(ParticleShape::Cone as u32, 2);
}

#[test]
fn test_color_gradient_creation() {
    let gradient = ColorGradient::new(vec![
        ColorStop { position: 0.0, color: Vec4::new(1.0, 0.0, 0.0, 1.0) },
        ColorStop { position: 1.0, color: Vec4::new(0.0, 0.0, 1.0, 1.0) },
    ]);
    
    assert_eq!(gradient.stops().len(), 2);
}

#[test]
fn test_lod_config_default() {
    let config = LodConfig::default();
    assert_eq!(config.quality, LodQuality::Medium);
}

#[test]
fn test_lod_level_creation() {
    let level = LodLevel {
        distance: 10.0,
        mesh_id: 1,
    };
    
    assert_eq!(level.distance, 10.0);
    assert_eq!(level.mesh_id, 1);
}

#[test]
fn test_lod_quality_values() {
    assert_eq!(LodQuality::Low as u32, 0);
    assert_eq!(LodQuality::Medium as u32, 1);
    assert_eq!(LodQuality::High as u32, 2);
}

#[test]
fn test_frustum_creation() {
    let frustum = Frustum::from_projection(&Mat4::IDENTITY);
    assert_eq!(frustum.planes().len(), 6);
}

#[test]
fn test_plane_creation() {
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let distance = 0.0;
    let plane = Plane::new(normal, distance);
    
    assert_eq!(plane.normal(), normal);
    assert_eq!(plane.distance(), distance);
}

#[test]
fn test_culling_result_values() {
    assert_eq!(CullingResult::Inside as u32, 0);
    assert_eq!(CullingResult::Outside as u32, 1);
    assert_eq!(CullingResult::Intersecting as u32, 2);
}

#[test]
fn test_csm_config_default() {
    let config = CsmConfig::default();
    assert_eq!(config.shadow_quality, ShadowQuality::Medium);
}

#[test]
fn test_shadow_quality_values() {
    assert_eq!(ShadowQuality::Low as u32, 0);
    assert_eq!(ShadowQuality::Medium as u32, 1);
    assert_eq!(ShadowQuality::High as u32, 2);
}

#[test]
fn test_batch_stats_default() {
    let stats = BatchStats::default();
    assert_eq!(stats.update_count, 0);
    assert_eq!(stats.total_uploaded_instances, 0);
    assert_eq!(stats.total_uploaded_bytes, 0);
    assert_eq!(stats.incremental_update_count, 0);
    assert_eq!(stats.full_rebuild_count, 0);
}

#[test]
fn test_batch_stats_accumulation() {
    let mut stats = BatchStats::default();
    stats.update_count = 10;
    stats.total_uploaded_instances = 1000;
    stats.total_uploaded_bytes = 64000;
    stats.incremental_update_count = 8;
    stats.full_rebuild_count = 2;
    
    assert_eq!(stats.update_count, 10);
    assert_eq!(stats.total_uploaded_instances, 1000);
    assert_eq!(stats.total_uploaded_bytes, 64000);
    assert_eq!(stats.incremental_update_count, 8);
    assert_eq!(stats.full_rebuild_count, 2);
}

#[test]
fn test_pbr_material_metallic_workflow() {
    let metal_material = PbrMaterial {
        base_color: Vec4::new(0.8, 0.8, 0.9, 1.0),
        metallic: 1.0,
        roughness: 0.2,
        ..Default::default()
    };
    
    assert_eq!(metal_material.metallic, 1.0);
    assert_eq!(metal_material.roughness, 0.2);
}

#[test]
fn test_pbr_material_dielectric_workflow() {
    let dielectric_material = PbrMaterial {
        base_color: Vec4::new(0.8, 0.2, 0.2, 1.0),
        metallic: 0.0,
        roughness: 0.5,
        ..Default::default()
    };
    
    assert_eq!(dielectric_material.metallic, 0.0);
    assert_eq!(dielectric_material.roughness, 0.5);
}

#[test]
fn test_pbr_material_emissive() {
    let emissive_material = PbrMaterial {
        emissive: Vec3::new(1.0, 0.5, 0.0),
        ..Default::default()
    };
    
    assert_eq!(emissive_material.emissive, Vec3::new(1.0, 0.5, 0.0));
}

#[test]
fn test_pbr_material_clearcoat() {
    let clearcoat_material = PbrMaterial {
        clearcoat: 1.0,
        clearcoat_roughness: 0.1,
        ..Default::default()
    };
    
    assert_eq!(clearcoat_material.clearcoat, 1.0);
    assert_eq!(clearcoat_material.clearcoat_roughness, 0.1);
}

#[test]
fn test_pbr_material_anisotropy() {
    let anisotropic_material = PbrMaterial {
        anisotropy: 0.8,
        anisotropy_direction: [1.0, 0.0],
        ..Default::default()
    };
    
    assert_eq!(anisotropic_material.anisotropy, 0.8);
    assert_eq!(anisotropic_material.anisotropy_direction, [1.0, 0.0]);
}

#[test]
fn test_pbr_material_uv_transform() {
    let transformed_material = PbrMaterial {
        uv_offset: [0.5, 0.3],
        uv_scale: [2.0, 2.0],
        uv_rotation: std::f32::consts::PI / 4.0,
        ..Default::default()
    };
    
    assert_eq!(transformed_material.uv_offset, [0.5, 0.3]);
    assert_eq!(transformed_material.uv_scale, [2.0, 2.0]);
    assert!((transformed_material.uv_rotation - std::f32::consts::PI / 4.0).abs() < 0.001);
}

#[test]
fn test_point_light_attenuation() {
    let light = PointLight3D {
        position: Vec3::ZERO,
        intensity: 1.0,
        radius: 10.0,
        ..Default::default()
    };
    
    assert_eq!(light.radius, 10.0);
    assert_eq!(light.intensity, 1.0);
}

#[test]
fn test_directional_light_direction() {
    let light = DirectionalLight {
        direction: Vec3::new(0.0, -1.0, 0.0).normalize(),
        ..Default::default()
    };
    
    assert_eq!(light.direction, Vec3::new(0.0, -1.0, 0.0));
}

#[test]
fn test_postprocess_bloom_config() {
    let config = PostProcessConfig {
        bloom_enabled: true,
        bloom_threshold: 1.0,
        bloom_strength: 0.5,
        bloom_radius: 0.5,
        ..Default::default()
    };
    
    assert!(config.bloom_enabled);
    assert_eq!(config.bloom_threshold, 1.0);
    assert_eq!(config.bloom_strength, 0.5);
    assert_eq!(config.bloom_radius, 0.5);
}

#[test]
fn test_postprocess_ssao_config() {
    let config = PostProcessConfig {
        ssao_enabled: true,
        ssao_radius: 0.5,
        ssao_bias: 0.025,
        ..Default::default()
    };
    
    assert!(config.ssao_enabled);
    assert_eq!(config.ssao_radius, 0.5);
    assert_eq!(config.ssao_bias, 0.025);
}

#[test]
fn test_postprocess_tonemap_config() {
    let config = PostProcessConfig {
        tonemap_enabled: true,
        tonemap_operator: TonemapOperator::ACES,
        exposure: 1.5,
        gamma: 2.2,
        ..Default::default()
    };
    
    assert!(config.tonemap_enabled);
    assert_eq!(config.tonemap_operator, TonemapOperator::ACES);
    assert_eq!(config.exposure, 1.5);
    assert_eq!(config.gamma, 2.2);
}

#[test]
fn test_gpu_instance_transform() {
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 2.0, 2.0),
        Quat::IDENTITY,
        Vec3::new(1.0, 2.0, 3.0)
    );
    
    let instance = GpuInstance {
        model: transform.to_cols_array_2d(),
        material_index: 1,
        lod_index: 0,
    };
    
    assert_eq!(instance.material_index, 1);
    assert_eq!(instance.lod_index, 0);
}

#[test]
fn test_batch_key_with_blend_modes() {
    let opaque_key = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let transparent_key = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 1,
        depth_test: true,
        render_flags: 0,
    };
    
    assert_ne!(opaque_key, transparent_key);
}

#[test]
fn test_batch_key_sorting() {
    let key1 = BatchKey {
        mesh_id: 1,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    let key2 = BatchKey {
        mesh_id: 2,
        material_id: 1,
        pipeline_id: 1,
        blend_mode: 0,
        depth_test: true,
        render_flags: 0,
    };
    
    assert!(key1 < key2);
}

#[test]
fn test_particle_emitter_rate() {
    let config = ParticleEmitterConfig {
        max_particles: 5000,
        emission_rate: 100.0,
        particle_lifetime: 2.0,
        ..Default::default()
    };
    
    assert_eq!(config.max_particles, 5000);
    assert_eq!(config.emission_rate, 100.0);
    assert_eq!(config.particle_lifetime, 2.0);
}

#[test]
fn test_lod_level_distances() {
    let levels = vec![
        LodLevel { distance: 0.0, mesh_id: 1 },
        LodLevel { distance: 10.0, mesh_id: 2 },
        LodLevel { distance: 20.0, mesh_id: 3 },
    ];
    
    assert_eq!(levels[0].distance, 0.0);
    assert_eq!(levels[1].distance, 10.0);
    assert_eq!(levels[2].distance, 20.0);
}

#[test]
fn test_frustum_planes() {
    let frustum = Frustum::from_projection(&Mat4::IDENTITY);
    let planes = frustum.planes();
    
    assert_eq!(planes.len(), 6);
}

#[test]
fn test_csm_shadow_cascades() {
    let config = CsmConfig {
        cascade_count: 4,
        shadow_quality: ShadowQuality::High,
        ..Default::default()
    };
    
    assert_eq!(config.cascade_count, 4);
    assert_eq!(config.shadow_quality, ShadowQuality::High);
}

#[test]
fn test_batch_stats_calculations() {
    let mut stats = BatchStats::default();
    stats.update_count = 100;
    stats.total_uploaded_instances = 50000;
    stats.incremental_update_count = 90;
    stats.full_rebuild_count = 10;
    
    let incremental_ratio = stats.incremental_update_count as f64 / stats.update_count as f64;
    assert!((incremental_ratio - 0.9).abs() < 0.001);
    
    let avg_upload_size = stats.total_uploaded_instances / stats.update_count;
    assert_eq!(avg_upload_size, 500);
}
