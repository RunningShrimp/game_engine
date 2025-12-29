#[cfg(test)]
mod tests {
    use crate::ecs::{PointLight, Sprite, Transform};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        assert!(world.get_entity(entity).is_ok());
    }

    #[test]
    fn test_component_insertion() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        world.entity_mut(entity).insert(Transform::default());
        assert!(world.get::<Transform>(entity).is_some());

        world.entity_mut(entity).insert(Sprite {
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
        assert!(world.get::<Sprite>(entity).is_some());

        world.entity_mut(entity).insert(PointLight::default());
        assert!(world.get::<PointLight>(entity).is_some());
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn(Transform::default());

        let mut query = world.query::<(&Transform, &Sprite)>();
        assert_eq!(query.iter(&world).count(), 2);

        let mut query_single = world.query::<&Transform>();
        assert_eq!(query_single.iter(&world).count(), 3);
    }

    // ========================================
    // Component Default Value Tests
    // ========================================

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.pos, glam::Vec3::ZERO);
        assert_eq!(transform.rot, glam::Quat::IDENTITY);
        assert_eq!(transform.scale, glam::Vec3::ONE);
    }

    #[test]
    fn test_transform_new() {
        let transform = Transform::new();
        assert_eq!(transform.pos, glam::Vec3::ZERO);
        assert_eq!(transform.rot, glam::Quat::IDENTITY);
        assert_eq!(transform.scale, glam::Vec3::ONE);
    }

    #[test]
    fn test_velocity_default() {
        let velocity = crate::ecs::Velocity::new();
        assert_eq!(velocity.lin, glam::Vec3::ZERO);
        assert_eq!(velocity.ang, glam::Vec3::ZERO);
    }

    #[test]
    fn test_sprite_default() {
        let sprite = Sprite::default();
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sprite.tex_index, 0);
        assert_eq!(sprite.normal_tex_index, 0);
    }

    #[test]
    fn test_sprite_new() {
        let sprite = Sprite::new();
        assert_eq!(sprite.color, [1.0; 4]);
        assert_eq!(sprite.tex_index, 0);
        assert_eq!(sprite.normal_tex_index, 0);
    }

    #[test]
    fn test_point_light_default() {
        let light = PointLight::default();
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.radius, 100.0);
    }

    #[test]
    fn test_point_light_new() {
        let light = PointLight::new();
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.radius, 100.0);
    }

    #[test]
    fn test_projection_default() {
        let projection = crate::ecs::Projection::default();
        assert!(matches!(projection, crate::ecs::Projection::Orthographic { .. }));
    }

    #[test]
    fn test_projection_new() {
        let projection = crate::ecs::Projection::new();
        assert!(matches!(projection, crate::ecs::Projection::Orthographic { .. }));
    }

    #[test]
    fn test_camera_default() {
        let camera = crate::ecs::Camera::default();
        assert_eq!(camera.is_active, true);
        assert!(matches!(camera.projection, crate::ecs::Projection::Orthographic { .. }));
    }

    #[test]
    fn test_camera_new() {
        let camera = crate::ecs::Camera::new();
        assert_eq!(camera.is_active, true);
    }

    #[test]
    fn test_material_default() {
        let material = crate::ecs::Material::default();
        assert_eq!(material.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
    }

    #[test]
    fn test_material_new() {
        let material = crate::ecs::Material::new();
        assert_eq!(material.color, [1.0; 4]);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
    }

    #[test]
    fn test_pbr_material_default() {
        let pbr_material = crate::ecs::PbrMaterialComp::default();
        assert_eq!(pbr_material.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(pbr_material.metallic, 0.0);
        assert_eq!(pbr_material.roughness, 0.5);
        assert_eq!(pbr_material.ambient_occlusion, 1.0);
    }

    #[test]
    fn test_pbr_material_new() {
        let pbr_material = crate::ecs::PbrMaterialComp::new();
        assert_eq!(pbr_material.base_color, [1.0; 4]);
        assert_eq!(pbr_material.metallic, 0.0);
        assert_eq!(pbr_material.roughness, 0.5);
    }

    #[test]
    fn test_point_light3d_default() {
        let light = crate::ecs::PointLight3D::default();
        assert_eq!(light.intensity, 1.0);
        assert_eq!(light.radius, 10.0);
    }

    #[test]
    fn test_point_light3d_new() {
        let light = crate::ecs::PointLight3D::new();
        assert_eq!(light.color, [1.0; 3]);
        assert_eq!(light.intensity, 1.0);
    }

    #[test]
    fn test_directional_light_default() {
        let light = crate::ecs::DirectionalLightComp::default();
        let default_dir: [f32; 3] = [0.0, -1.0, 0.0];
        assert_eq!(light.direction, default_dir);
    }

    #[test]
    fn test_directional_light_new() {
        let light = crate::ecs::DirectionalLightComp::new();
        assert_eq!(light.direction, [0.0, -1.0, 0.0]);
    }

    #[test]
    fn test_time_default() {
        let time = crate::ecs::Time::default();
        assert_eq!(time.elapsed_seconds, 0.0);
        assert_eq!(time.delta_seconds, 0.0);
    }

    #[test]
    fn test_time_fixed_time_step() {
        let time = crate::ecs::Time::default();
        assert_eq!(time.fixed_time_step, 1.0 / 60.0);
    }

    // ========================================
    // Custom Value Tests
    // ========================================

    #[test]
    fn test_transform_with_custom_values() {
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            rot: glam::Quat::from_rotation_x(0.5),
            scale: glam::Vec3::new(2.0, 2.0, 2.0),
        };
        assert_eq!(transform.pos.x, 1.0);
        assert_eq!(transform.pos.y, 2.0);
        assert_eq!(transform.pos.z, 3.0);
    }

    #[test]
    fn test_projection_perspective() {
        let fov = 45.0;
        let aspect = 16.0 / 9.0;
        let near = 0.5;
        let far = 500.0;
        let projection = crate::ecs::Projection::Perspective {
            fov,
            aspect,
            near,
            far,
        };

        assert!(matches!(projection, crate::ecs::Projection::Perspective { fov: f, aspect: a, near: n, far: fa } if f == fov && a == aspect && n == near && fa == far));
    }

    #[test]
    fn test_projection_orthographic() {
        let scale = 1.0;
        let near = 0.1;
        let far = 100.0;
        let projection = crate::ecs::Projection::Orthographic {
            scale,
            near,
            far,
        };

        assert!(matches!(projection, crate::ecs::Projection::Orthographic { scale: s, near: n, far: f } if s == scale && n == near && f == far));
    }

    #[test]
    fn test_material_with_custom_values() {
        let material = crate::ecs::Material {
            color: [0.5, 0.7, 0.9, 1.0],
            metallic: 0.8,
            roughness: 0.2,
        };
        assert_eq!(material.color, [0.5, 0.7, 0.9, 1.0]);
        assert_eq!(material.metallic, 0.8);
        assert_eq!(material.roughness, 0.2);
    }

    #[test]
    fn test_pbr_material_with_custom_values() {
        let pbr_material = crate::ecs::PbrMaterialComp {
            base_color: [0.8, 0.2, 0.1, 1.0],
            metallic: 1.0,
            roughness: 0.3,
            ambient_occlusion: 0.9,
            emissive: [0.0, 0.0, 0.0],
            emissive_strength: 0.0,
        };
        assert_eq!(pbr_material.base_color, [0.8, 0.2, 0.1, 1.0]);
        assert_eq!(pbr_material.metallic, 1.0);
        assert_eq!(pbr_material.roughness, 0.3);
        assert_eq!(pbr_material.ambient_occlusion, 0.9);
    }

    #[test]
    fn test_directional_light_with_custom_direction() {
        let custom_dir: [f32; 3] = [1.0, -0.5, 0.3];
        let light = crate::ecs::DirectionalLightComp {
            direction: custom_dir,
            ..Default::default()
        };
        assert_eq!(light.direction, custom_dir);
    }

    #[test]
    fn test_time_with_custom_values() {
        let elapsed = 5.5;
        let delta = 0.016;
        let time = crate::ecs::Time {
            elapsed_seconds: elapsed,
            delta_seconds: delta,
            fixed_time_step: 0.016,
            alpha: 1.0,
        };
        assert_eq!(time.elapsed_seconds, elapsed);
        assert_eq!(time.delta_seconds, delta);
    }

    #[test]
    fn test_sprite_with_custom_values() {
        let sprite = Sprite {
            color: [1.0, 0.5, 0.2, 0.8],
            tex_index: 5,
            normal_tex_index: 1,
            uv_off: [0.1, 0.2],
            uv_scale: [2.0, 2.0],
            layer: 5.0,
        };
        assert_eq!(sprite.color[0], 1.0);
        assert_eq!(sprite.tex_index, 5);
        assert_eq!(sprite.layer, 5.0);
    }

    // ========================================
    // Entity Component Tests
    // ========================================

    #[test]
    fn test_entity_with_transform() {
        let mut world = World::new();
        let entity = world.spawn(Transform {
            pos: glam::Vec3::new(10.0, 20.0, 30.0),
            ..Default::default()
        }).id();

        let transform = world.get::<Transform>(entity);
        assert!(transform.is_some());
        let transform = transform.unwrap();
        assert_eq!(transform.pos.x, 10.0);
    }

    #[test]
    fn test_entity_with_velocity() {
        let mut world = World::new();
        let entity = world.spawn(crate::ecs::Velocity {
            lin: glam::Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        }).id();

        let velocity = world.get::<crate::ecs::Velocity>(entity);
        assert!(velocity.is_some());
        let velocity = velocity.unwrap();
        assert_eq!(velocity.lin.x, 1.0);
    }

    #[test]
    fn test_entity_with_multiple_components() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            crate::ecs::Velocity::default(),
        )).id();

        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
        assert!(world.get::<crate::ecs::Velocity>(entity).is_some());
    }

    #[test]
    fn test_entity_despawn() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        assert!(world.get_entity(entity).is_ok());

        world.entity_mut(entity).despawn();
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_resource_insertion() {
        let mut world = World::new();
        world.insert_resource(crate::ecs::Time::default());
        assert!(world.get_resource::<crate::ecs::Time>().is_some());
    }

    #[test]
    fn test_viewport_default() {
        let viewport = crate::ecs::Viewport::default();
        assert_eq!(viewport.width, 0);
        assert_eq!(viewport.height, 0);
    }

    #[test]
    fn test_tileset_default() {
        let tileset = crate::ecs::TileSet::default();
        assert!(tileset.tiles.is_empty());
    }

    #[test]
    fn test_tile_chunk_config_default() {
        let config = crate::ecs::TileChunkConfig::default();
        assert_eq!(config.size[0], 0);
        assert_eq!(config.size[1], 0);
    }

    #[test]
    fn test_tile_chunks_default() {
        let chunks = crate::ecs::TileChunks::default();
        assert!(chunks.visible.is_empty());
    }

    #[test]
    fn test_tile_entity_pool_default() {
        let pool = crate::ecs::TileEntityPool::default();
        assert!(pool.unused.is_empty());
        assert_eq!(pool.capacity, 1000);
    }

    #[test]
    fn test_tile_entity_pool_new() {
        let pool = crate::ecs::TileEntityPool::new();
        assert!(pool.unused.is_empty());
        assert_eq!(pool.capacity, 1000);
    }

    #[test]
    fn test_previous_transform_default() {
        let prev = crate::ecs::PreviousTransform::default();
        assert_eq!(prev.pos, glam::Vec3::ZERO);
        assert_eq!(prev.rot, glam::Quat::IDENTITY);
        assert_eq!(prev.scale, glam::Vec3::ONE);
    }

    #[test]
    fn test_flipbook_default() {
        let flipbook = crate::ecs::Flipbook::default();
        assert!(flipbook.frames.is_empty());
        assert_eq!(flipbook.speed, 1.0);
        assert!(flipbook.looping);
    }

    #[test]
    fn test_flipbook_new() {
        let flipbook = crate::ecs::Flipbook::new();
        assert!(flipbook.frames.is_empty());
        assert_eq!(flipbook.current, 0);
    }

    #[test]
    fn test_query_with_velocity() {
        let mut world = World::new();
        world.spawn((
            Transform::default(),
            crate::ecs::Velocity::default(),
        ));
        world.spawn(Transform::default());

        let mut query = world.query::<(&Transform, &crate::ecs::Velocity)>();
        assert_eq!(query.iter(&world).count(), 1);
    }

    #[test]
    fn test_query_mut() {
        let mut world = World::new();
        world.spawn(Transform::default());

        let mut query = world.query::<&mut Transform>();
        for mut transform in query.iter_mut(&mut world) {
            transform.pos.x = 100.0;
        }

        let mut query = world.query::<&Transform>();
        for transform in query.iter(&world) {
            assert_eq!(transform.pos.x, 100.0);
        }
    }

    #[test]
    fn test_component_clone() {
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        };
        let cloned = transform;
        assert_eq!(transform.pos.x, cloned.pos.x);
    }

    #[test]
    fn test_transform_equality() {
        let t1 = Transform::default();
        let t2 = Transform::default();
        assert_eq!(t1.pos, t2.pos);
        assert_eq!(t1.rot, t2.rot);
        assert_eq!(t1.scale, t2.scale);
    }
}
