//! ECS Extended Tests
//!
//! Comprehensive tests for ECS components and systems

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::*;
    use bevy_ecs::prelude::*;
    use glam::{Quat, Vec3};

    // ========================================
    // Transform Extended Tests
    // ========================================

    #[test]
    fn test_transform_identity() {
        let transform = Transform::default();

        assert_eq!(transform.pos, Vec3::ZERO);
        assert_eq!(transform.rot, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_scale_modification() {
        let mut transform = Transform::default();
        transform.scale = Vec3::new(2.0, 3.0, 4.0);

        assert_eq!(transform.scale.x, 2.0);
        assert_eq!(transform.scale.y, 3.0);
        assert_eq!(transform.scale.z, 4.0);
    }

    #[test]
    fn test_transform_complex_rotation() {
        let rot = Quat::from_euler(glam::EulerRot::XYZ, 0.1, 0.2, 0.3);
        let transform = Transform {
            pos: Vec3::ZERO,
            rot,
            scale: Vec3::ONE,
        };

        // 验证四元数被保持
        assert_eq!(transform.rot, rot);
    }

    #[test]
    fn test_transform_copy() {
        let t1 = Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::from_rotation_x(0.5),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };

        let t2 = t1;
        assert_eq!(t1.pos, t2.pos);
        assert_eq!(t1.rot, t2.rot);
        assert_eq!(t1.scale, t2.scale);
    }

    // ========================================
    // Velocity Extended Tests
    // ========================================

    #[test]
    fn test_velocity_with_values() {
        let velocity = Velocity {
            lin: Vec3::new(1.0, 2.0, 3.0),
            ang: Vec3::new(0.1, 0.2, 0.3),
        };

        assert_eq!(velocity.lin.x, 1.0);
        assert_eq!(velocity.ang.x, 0.1);
    }

    #[test]
    fn test_velocity_zero() {
        let velocity = Velocity::new();
        assert_eq!(velocity.lin, Vec3::ZERO);
        assert_eq!(velocity.ang, Vec3::ZERO);
    }

    #[test]
    fn test_velocity_high_values() {
        let velocity = Velocity {
            lin: Vec3::new(1000.0, 2000.0, 3000.0),
            ang: Vec3::new(10.0, 20.0, 30.0),
        };

        assert_eq!(velocity.lin.y, 2000.0);
        assert_eq!(velocity.ang.y, 20.0);
    }

    // ========================================
    // Sprite Extended Tests
    // ========================================

    #[test]
    fn test_sprite_color_variations() {
        let sprite1 = Sprite {
            color: [1.0, 0.0, 0.0, 1.0], // Red
            ..Default::default()
        };

        let sprite2 = Sprite {
            color: [0.0, 1.0, 0.0, 1.0], // Green
            ..Default::default()
        };

        assert_eq!(sprite1.color[0], 1.0); // R
        assert_eq!(sprite2.color[1], 1.0); // G
    }

    #[test]
    fn test_sprite_uv_transforms() {
        let sprite = Sprite {
            uv_off: [0.5, 0.25],
            uv_scale: [0.5, 0.5],
            ..Default::default()
        };

        assert_eq!(sprite.uv_off[0], 0.5);
        assert_eq!(sprite.uv_scale[1], 0.5);
    }

    #[test]
    fn test_sprite_layer_ordering() {
        let sprite1 = Sprite {
            layer: 0.0,
            ..Default::default()
        };

        let sprite2 = Sprite {
            layer: 10.0,
            ..Default::default()
        };

        assert!(sprite2.layer > sprite1.layer);
    }

    #[test]
    fn test_sprite_texture_indices() {
        let sprite = Sprite {
            tex_index: 5,
            normal_tex_index: 3,
            ..Default::default()
        };

        assert_eq!(sprite.tex_index, 5);
        assert_eq!(sprite.normal_tex_index, 3);
    }

    // ========================================
    // Material Extended Tests
    // ========================================

    #[test]
    fn test_material_metallic_spectrum() {
        let non_metal = Material {
            metallic: 0.0,
            ..Default::default()
        };

        let metal = Material {
            metallic: 1.0,
            ..Default::default()
        };

        assert_eq!(non_metal.metallic, 0.0);
        assert_eq!(metal.metallic, 1.0);
    }

    #[test]
    fn test_material_roughness_spectrum() {
        let smooth = Material {
            roughness: 0.0,
            ..Default::default()
        };

        let rough = Material {
            roughness: 1.0,
            ..Default::default()
        };

        assert_eq!(smooth.roughness, 0.0);
        assert_eq!(rough.roughness, 1.0);
    }

    #[test]
    fn test_material_transparency() {
        let opaque = Material {
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        };

        let transparent = Material {
            color: [1.0, 1.0, 1.0, 0.5],
            ..Default::default()
        };

        assert_eq!(opaque.color[3], 1.0);
        assert_eq!(transparent.color[3], 0.5);
    }

    // ========================================
    // PBR Material Extended Tests
    // ========================================

    #[test]
    fn test_pbr_material_all_properties() {
        let pbr = PbrMaterialComp {
            base_color: [0.8, 0.2, 0.1, 1.0],
            metallic: 0.9,
            roughness: 0.3,
            ambient_occlusion: 0.8,
            emissive: [1.0, 0.5, 0.0],
            emissive_strength: 2.0,
        };

        assert_eq!(pbr.base_color[0], 0.8);
        assert_eq!(pbr.metallic, 0.9);
        assert_eq!(pbr.roughness, 0.3);
        assert_eq!(pbr.ambient_occlusion, 0.8);
        assert_eq!(pbr.emissive[0], 1.0);
        assert_eq!(pbr.emissive_strength, 2.0);
    }

    #[test]
    fn test_pbr_material_emissive() {
        let glowing = PbrMaterialComp {
            emissive: [1.0, 1.0, 1.0],
            emissive_strength: 10.0,
            ..Default::default()
        };

        assert_eq!(glowing.emissive_strength, 10.0);
    }

    // ========================================
    // Camera Extended Tests
    // ========================================

    #[test]
    fn test_camera_active_state() {
        let active = Camera {
            is_active: true,
            ..Default::default()
        };

        let inactive = Camera {
            is_active: false,
            ..Default::default()
        };

        assert!(active.is_active);
        assert!(!inactive.is_active);
    }

    #[test]
    fn test_camera_orthographic_projection() {
        let camera = Camera {
            projection: Projection::Orthographic {
                scale: 1.0,
                near: 0.1,
                far: 100.0,
            },
            ..Default::default()
        };

        match camera.projection {
            Projection::Orthographic { scale, near, far } => {
                assert_eq!(scale, 1.0);
                assert_eq!(near, 0.1);
                assert_eq!(far, 100.0);
            }
            _ => panic!("Expected Orthographic projection"),
        }
    }

    #[test]
    fn test_camera_perspective_projection() {
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 45.0,
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 1000.0,
            },
            ..Default::default()
        };

        match camera.projection {
            Projection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => {
                assert_eq!(fov, 45.0);
                assert!((aspect - 1.777).abs() < 0.01);
                assert_eq!(near, 0.1);
                assert_eq!(far, 1000.0);
            }
            _ => panic!("Expected Perspective projection"),
        }
    }

    // ========================================
    // Light Extended Tests
    // ========================================

    #[test]
    fn test_point_light_intensity() {
        let dim = PointLight {
            intensity: 0.5,
            ..Default::default()
        };

        let bright = PointLight {
            intensity: 10.0,
            ..Default::default()
        };

        assert_eq!(dim.intensity, 0.5);
        assert_eq!(bright.intensity, 10.0);
    }

    #[test]
    fn test_point_light_colors() {
        let red_light = PointLight {
            color: [1.0, 0.0, 0.0],
            ..Default::default()
        };

        let blue_light = PointLight {
            color: [0.0, 0.0, 1.0],
            ..Default::default()
        };

        assert_eq!(red_light.color[0], 1.0);
        assert_eq!(blue_light.color[2], 1.0);
    }

    #[test]
    fn test_point_light_radius() {
        let small_light = PointLight {
            radius: 10.0,
            ..Default::default()
        };

        let large_light = PointLight {
            radius: 500.0,
            ..Default::default()
        };

        assert_eq!(small_light.radius, 10.0);
        assert_eq!(large_light.radius, 500.0);
    }

    #[test]
    fn test_point_light_3d_variations() {
        let light1 = PointLight3D {
            color: [1.0, 0.5, 0.2],
            intensity: 2.0,
            radius: 50.0,
        };

        assert_eq!(light1.color[1], 0.5);
        assert_eq!(light1.intensity, 2.0);
    }

    #[test]
    fn test_directional_light_directions() {
        let down = DirectionalLightComp {
            direction: [0.0, -1.0, 0.0],
            ..Default::default()
        };

        let diagonal = DirectionalLightComp {
            direction: [1.0, -1.0, 1.0],
            ..Default::default()
        };

        assert_eq!(down.direction[1], -1.0);
        assert_eq!(diagonal.direction[0], 1.0);
    }

    // ========================================
    // Time Extended Tests
    // ========================================

    #[test]
    fn test_time_accumulation() {
        let mut time = Time::default();

        time.delta_seconds = 0.016;
        time.elapsed_seconds += time.delta_seconds as f64;

        assert_eq!(time.elapsed_seconds, 0.016);
    }

    #[test]
    fn test_time_fixed_timestep_variations() {
        let time_30 = Time {
            fixed_time_step: 1.0 / 30.0,
            ..Default::default()
        };

        let time_60 = Time {
            fixed_time_step: 1.0 / 60.0,
            ..Default::default()
        };

        assert!((time_30.fixed_time_step - 0.0333).abs() < 0.001);
        assert!((time_60.fixed_time_step - 0.0167).abs() < 0.001);
    }

    #[test]
    fn test_time_alpha_interpolation() {
        let time = Time {
            alpha: 0.5,
            ..Default::default()
        };

        assert_eq!(time.alpha, 0.5);
    }

    // ========================================
    // Viewport Tests
    // ========================================

    #[test]
    fn test_viewport_resolutions() {
        let hd = Viewport {
            width: 1280,
            height: 720,
        };

        let full_hd = Viewport {
            width: 1920,
            height: 1080,
        };

        assert_eq!(hd.width, 1280);
        assert_eq!(full_hd.width, 1920);
    }

    #[test]
    fn test_viewport_aspect_ratio() {
        let viewport = Viewport {
            width: 1920,
            height: 1080,
        };

        let aspect = viewport.width as f32 / viewport.height as f32;
        assert!((aspect - 16.0 / 9.0).abs() < 0.01);
    }

    // ========================================
    // TileMap Tests
    // ========================================

    #[test]
    fn test_tilemap_basic() {
        let tilemap = TileMap {
            width: 10,
            height: 10,
            tile_size: [32.0, 32.0],
            tiles: vec!["tile1".to_string(); 100],
            dirty: false,
            layer: 0.0,
            atlas_tex_index: 0,
            chunk_size: [16, 16],
        };

        assert_eq!(tilemap.width, 10);
        assert_eq!(tilemap.height, 10);
        assert_eq!(tilemap.tiles.len(), 100);
    }

    #[test]
    fn test_tilemap_dirty_flag() {
        let mut tilemap = TileMap {
            width: 0,
            height: 0,
            tile_size: [0.0, 0.0],
            tiles: vec![],
            dirty: true,
            layer: 0.0,
            atlas_tex_index: 0,
            chunk_size: [16, 16],
        };

        assert!(tilemap.dirty);

        tilemap.dirty = false;
        assert!(!tilemap.dirty);
    }

    #[test]
    fn test_tilemap_chunk_config() {
        let config = TileChunkConfig { size: [16, 16] };

        assert_eq!(config.size[0], 16);
        assert_eq!(config.size[1], 16);
    }

    // ========================================
    // Flipbook Tests
    // ========================================

    #[test]
    fn test_flipbook_empty() {
        let flipbook = Flipbook::new();

        assert!(flipbook.frames.is_empty());
        assert_eq!(flipbook.current, 0);
    }

    #[test]
    fn test_flipbook_with_frames() {
        let mut flipbook = Flipbook::new();

        flipbook.frames.push(crate::ecs::FlipFrame {
            uv_off: [0.0, 0.0],
            uv_scale: [0.5, 0.5],
            duration: 0.1,
        });

        flipbook.frames.push(crate::ecs::FlipFrame {
            uv_off: [0.5, 0.0],
            uv_scale: [0.5, 0.5],
            duration: 0.1,
        });

        assert_eq!(flipbook.frames.len(), 2);
    }

    #[test]
    fn test_flipbook_looping() {
        let looping = Flipbook {
            looping: true,
            ..Default::default()
        };

        let oneshot = Flipbook {
            looping: false,
            ..Default::default()
        };

        assert!(looping.looping);
        assert!(!oneshot.looping);
    }

    // ========================================
    // PreviousTransform Tests
    // ========================================

    #[test]
    fn test_previous_transform_tracking() {
        let prev = PreviousTransform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        assert_eq!(prev.pos.x, 1.0);
    }

    // ========================================
    // Entity Pool Tests
    // ========================================

    #[test]
    fn test_tile_entity_pool_default() {
        let pool = TileEntityPool::new();

        assert_eq!(pool.capacity, 1000);
        assert!(pool.unused.is_empty());
    }

    // ========================================
    // System Tests
    // ========================================

    #[test]
    fn test_world_query_filtering() {
        let mut world = World::new();

        // Spawn entities with different components
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), Velocity::default()));
        world.spawn((Transform::default(), Sprite::default(), Velocity::default()));
        world.spawn_empty();

        // Query only entities with Transform and Sprite
        let mut query = world.query::<(&Transform, &Sprite)>();
        let count = query.iter(&world).count();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_resource_mutability() {
        let mut world = World::new();

        world.insert_resource(Time {
            delta_seconds: 0.033,
            elapsed_seconds: 1.5,
            fixed_time_step: 0.016,
            alpha: 0.0,
        });

        {
            let mut time = world.resource_mut::<Time>();
            time.elapsed_seconds += 0.033;
        }

        let time = world.resource::<Time>();
        assert_eq!(time.elapsed_seconds, 1.533);
    }

    // ========================================
    // Component Combination Tests
    // ========================================

    #[test]
    fn test_sprite_with_material() {
        let mut world = World::new();

        world.spawn((Sprite::default(), Material::default(), Transform::default()));

        let entity = world.query::<(Entity, &Sprite, &Material)>().iter(&world).next();

        assert!(entity.is_some());
    }

    #[test]
    fn test_camera_with_transform() {
        let mut world = World::new();

        world.spawn((Camera::default(), Transform::default()));

        let mut query = world.query::<(&Camera, &Transform)>();
        assert_eq!(query.iter(&world).count(), 1);
    }

    // ========================================
    // Performance Tests
    // ========================================

    #[test]
    fn test_spawn_many_entities() {
        let mut world = World::new();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            world.spawn((Transform::default(), Sprite::default()));
        }
        let duration = start.elapsed();

        // Should be fast (< 100ms)
        assert!(duration < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_query_performance() {
        let mut world = World::new();

        // Spawn 1000 entities
        for _ in 0..1000 {
            world.spawn((Transform::default(), Sprite::default()));
        }

        let start = std::time::Instant::now();
        let mut query = world.query::<(&Transform, &Sprite)>();
        let count = query.iter(&world).count();
        let duration = start.elapsed();

        assert_eq!(count, 1000);
        // Query should be fast (< 50ms)
        assert!(duration < std::time::Duration::from_millis(50));
    }
}
