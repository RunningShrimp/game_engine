#[cfg(test)]
mod tests {
    use crate::ecs::{PointLight, Sprite, Transform};
    use bevy_ecs::prelude::*;

    // Test resource type that implements the Resource trait
    #[derive(Resource, PartialEq, Eq, Debug)]
    struct TestResource(i32);

    // Additional resource type for testing primitive resources
    #[derive(Resource, PartialEq, Eq, Debug)]
    struct IntResource(i32);

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

    // ========================================
    // Entity Lifecycle Tests
    // ========================================

    #[test]
    fn test_entity_despawn() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        assert!(world.despawn(entity));
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_entity_with_multiple_components() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Transform::default(),
                Sprite::default(),
                PointLight::default(),
            ))
            .id();

        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
        assert!(world.get::<PointLight>(entity).is_some());
    }

    #[test]
    fn test_component_removal() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), Sprite::default())).id();

        world.entity_mut(entity).remove::<Sprite>();
        assert!(world.get::<Sprite>(entity).is_none());
        assert!(world.get::<Transform>(entity).is_some());
    }

    #[test]
    fn test_entity_mutability() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let mut transform =
            world.get_mut::<Transform>(entity).expect("Test: operation should succeed");
        transform.pos = glam::Vec3::new(1.0, 2.0, 3.0);

        let transform = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(transform.pos, glam::Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_multiple_entities() {
        let mut world = World::new();
        let e1 = world.spawn(Transform::default()).id();
        let e2 = world.spawn(Transform::default()).id();
        let e3 = world.spawn(Transform::default()).id();

        let mut query = world.query::<Entity>();
        assert_eq!(query.iter(&world).count(), 3);

        assert!(world.get::<Transform>(e1).is_some());
        assert!(world.get::<Transform>(e2).is_some());
        assert!(world.get::<Transform>(e3).is_some());
    }

    #[test]
    fn test_query_with_filter() {
        let mut world = World::new();
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn(Transform::default());

        let mut query = world.query_filtered::<&Transform, With<Sprite>>();
        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn test_query_without_component() {
        let mut world = World::new();
        world.spawn(Transform::default());
        world.spawn((Transform::default(), Sprite::default()));

        let mut query = world.query_filtered::<&Transform, Without<Sprite>>();
        assert_eq!(query.iter(&world).count(), 1);
    }

    #[test]
    fn test_component_mutability_in_query() {
        let mut world = World::new();
        world.spawn(Transform::default());

        let mut query = world.query::<&mut Transform>();
        for mut transform in query.iter_mut(&mut world) {
            transform.pos = glam::Vec3::new(5.0, 5.0, 5.0);
        }

        let mut query = world.query::<&Transform>();
        for transform in query.iter(&world) {
            assert_eq!(transform.pos, glam::Vec3::new(5.0, 5.0, 5.0));
        }
    }

    #[test]
    fn test_entity_clone_components() {
        let mut world = World::new();
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        let entity = world.spawn(transform.clone()).id();

        let retrieved = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(retrieved.pos, transform.pos);
    }

    #[test]
    fn test_sparse_component_distribution() {
        let mut world = World::new();
        for _ in 0..10 {
            world.spawn(Transform::default());
        }
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), PointLight::default()));

        let mut transform_query = world.query::<&Transform>();
        assert_eq!(transform_query.iter(&world).count(), 12);

        let mut sprite_query = world.query::<(&Transform, &Sprite)>();
        assert_eq!(sprite_query.iter(&world).count(), 1);
    }

    #[test]
    fn test_entity_resource_access() {
        let mut world = World::new();
        world.insert_resource(TestResource(42));

        assert_eq!(world.resource::<TestResource>().0, 42);
    }

    #[test]
    fn test_resource_mutability() {
        let mut world = World::new();
        world.insert_resource(TestResource(100));

        let mut value = world.resource_mut::<TestResource>();
        value.0 = 200;

        assert_eq!(world.resource::<TestResource>().0, 200);
    }

    #[test]
    fn test_multiple_resources() {
        let mut world = World::new();
        world.insert_resource(TestResource(10));

        #[derive(Resource)]
        struct FloatResource(f32);
        world.insert_resource(FloatResource(3.14));

        #[derive(Resource)]
        struct StringResource(String);
        world.insert_resource(StringResource(String::from("test")));

        assert_eq!(world.resource::<TestResource>().0, 10);
        assert_eq!(world.resource::<FloatResource>().0, 3.14);
        assert_eq!(&world.resource::<StringResource>().0, "test");
    }

    #[test]
    fn test_entity_batch_spawn() {
        let mut world = World::new();
        let entities: Vec<Entity> =
            (0..10).map(|_| world.spawn(Transform::default()).id()).collect();

        assert_eq!(entities.len(), 10);
        for entity in entities {
            assert!(world.get::<Transform>(entity).is_some());
        }
    }

    #[test]
    fn test_component_cloning() {
        let mut world = World::new();
        let sprite = Sprite {
            color: [0.5, 0.5, 0.5, 1.0],
            tex_index: 5,
            normal_tex_index: 2,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        };
        let entity = world.spawn(sprite.clone()).id();

        let retrieved = world.get::<Sprite>(entity).expect("Test: operation should succeed");
        assert_eq!(retrieved.color, sprite.color);
        assert_eq!(retrieved.tex_index, sprite.tex_index);
    }

    #[test]
    fn test_transform_position_update() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let mut transform =
            world.get_mut::<Transform>(entity).expect("Test: operation should succeed");
        transform.pos = glam::Vec3::new(10.0, 20.0, 30.0);

        let transform = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(transform.pos, glam::Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_transform_rotation_update() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let rotation = glam::Quat::from_rotation_z(std::f32::consts::PI / 2.0);
        let mut transform =
            world.get_mut::<Transform>(entity).expect("Test: operation should succeed");
        transform.rot = rotation;

        let transform = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(transform.rot, rotation);
    }

    #[test]
    fn test_transform_scale_update() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        let mut transform =
            world.get_mut::<Transform>(entity).expect("Test: operation should succeed");
        transform.scale = glam::Vec3::new(2.0, 2.0, 2.0);

        let transform = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(transform.scale, glam::Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_sprite_color_update() {
        let mut world = World::new();
        let entity = world.spawn(Sprite::default()).id();

        let mut sprite = world.get_mut::<Sprite>(entity).expect("Test: operation should succeed");
        sprite.color = [1.0, 0.0, 0.0, 1.0];

        let sprite = world.get::<Sprite>(entity).expect("Test: operation should succeed");
        assert_eq!(sprite.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_light_intensity_update() {
        let mut world = World::new();
        let entity = world.spawn(PointLight::default()).id();

        let mut light =
            world.get_mut::<PointLight>(entity).expect("Test: operation should succeed");
        light.intensity = 5.0;

        let light = world.get::<PointLight>(entity).expect("Test: operation should succeed");
        assert_eq!(light.intensity, 5.0);
    }

    #[test]
    fn test_light_radius_update() {
        let mut world = World::new();
        let entity = world.spawn(PointLight::default()).id();

        let mut light =
            world.get_mut::<PointLight>(entity).expect("Test: operation should succeed");
        light.radius = 200.0;

        let light = world.get::<PointLight>(entity).expect("Test: operation should succeed");
        assert_eq!(light.radius, 200.0);
    }

    #[test]
    fn test_entity_clear() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Transform::default(),
                Sprite::default(),
                PointLight::default(),
            ))
            .id();

        world.entity_mut(entity).clear();
        assert!(world.get::<Transform>(entity).is_none());
        assert!(world.get::<Sprite>(entity).is_none());
        assert!(world.get::<PointLight>(entity).is_none());
    }

    #[test]
    fn test_query_iter_mut() {
        let mut world = World::new();
        world.spawn(Transform::default());
        world.spawn(Transform::default());
        world.spawn(Transform::default());

        let mut query = world.query::<&mut Transform>();
        let count = query.iter_mut(&mut world).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_resource_remove() {
        let mut world = World::new();
        world.insert_resource(IntResource(42));
        assert!(world.contains_resource::<IntResource>());

        world.remove_resource::<IntResource>();
        assert!(!world.contains_resource::<IntResource>());
    }

    #[test]
    fn test_entity_archetype() {
        let mut world = World::new();
        let e1 = world.spawn((Transform::default(), Sprite::default())).id();
        let e2 = world.spawn((Transform::default(), Sprite::default())).id();

        // Both should be in the same archetype
        let mut query = world.query::<(&Transform, &Sprite)>();
        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn test_different_archetypes() {
        let mut world = World::new();
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((Transform::default(), PointLight::default()));

        let mut transform_sprite = world.query::<(&Transform, &Sprite)>();
        let mut transform_light = world.query::<(&Transform, &PointLight)>();

        assert_eq!(transform_sprite.iter(&world).count(), 1);
        assert_eq!(transform_light.iter(&world).count(), 1);
    }

    #[test]
    fn test_entity_exists() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        assert!(world.get_entity(entity).is_ok());
        assert!(world.despawn(entity));
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn test_component_equality() {
        let transform1 = Transform::default();
        let transform2 = Transform::default();

        assert_eq!(transform1.pos, transform2.pos);
        assert_eq!(transform1.rot, transform2.rot);
        assert_eq!(transform1.scale, transform2.scale);
    }

    #[test]
    fn test_velocity_component() {
        let mut world = World::new();
        let velocity = crate::ecs::Velocity {
            lin: glam::Vec3::new(1.0, 2.0, 3.0),
            ang: glam::Vec3::new(0.1, 0.2, 0.3),
        };
        let entity = world.spawn(velocity).id();

        let retrieved = world
            .get::<crate::ecs::Velocity>(entity)
            .expect("Test: operation should succeed");
        assert_eq!(retrieved.lin, velocity.lin);
        assert_eq!(retrieved.ang, velocity.ang);
    }

    #[test]
    fn test_nested_query() {
        let mut world = World::new();
        world.spawn((Transform::default(), Sprite::default()));
        world.spawn((
            Transform::default(),
            Sprite::default(),
            PointLight::default(),
        ));

        let mut outer_query = world.query::<(&Transform, &Sprite)>();
        let count = outer_query.iter(&world).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_empty_query() {
        let mut world = World::new();
        let mut query = world.query::<&Transform>();
        assert_eq!(query.iter(&world).count(), 0);
    }

    #[test]
    fn test_entity_with_many_components() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Transform::default(),
                Sprite::default(),
                PointLight::default(),
                crate::ecs::Velocity::new(),
            ))
            .id();

        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
        assert!(world.get::<PointLight>(entity).is_some());
        assert!(world.get::<crate::ecs::Velocity>(entity).is_some());
    }

    #[test]
    fn test_resource_default_insert() {
        let mut world = World::new();
        world.insert_resource(IntResource(100));
        assert_eq!(world.resource::<IntResource>().0, 100);
    }

    #[test]
    fn test_multiple_worlds() {
        let mut world1 = World::new();
        let mut world2 = World::new();

        let e1 = world1.spawn(Transform::default()).id();
        let e2 = world2.spawn(Transform::default()).id();

        // 验证各自world中的实体存在
        assert!(world1.get::<Transform>(e1).is_some());
        assert!(world2.get::<Transform>(e2).is_some());

        // 验证world之间是独立的（通过组件独立性验证）
        let transform1 = world1.get::<Transform>(e1).unwrap();
        let transform2 = world2.get::<Transform>(e2).unwrap();
        assert_eq!(transform1.pos, glam::Vec3::ZERO);
        assert_eq!(transform2.pos, glam::Vec3::ZERO);
    }

    #[test]
    fn test_sprite_texture_index() {
        let mut world = World::new();
        let sprite = Sprite {
            tex_index: 42,
            ..Default::default()
        };
        let entity = world.spawn(sprite).id();

        let retrieved = world.get::<Sprite>(entity).expect("Test: operation should succeed");
        assert_eq!(retrieved.tex_index, 42);
    }

    #[test]
    fn test_transform_complex_transform() {
        let mut world = World::new();
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            rot: glam::Quat::from_euler(glam::EulerRot::XYZ, 0.1, 0.2, 0.3),
            scale: glam::Vec3::new(2.0, 3.0, 4.0),
        };
        let entity = world.spawn(transform).id();

        let retrieved = world.get::<Transform>(entity).expect("Test: operation should succeed");
        assert_eq!(retrieved.pos, transform.pos);
        assert_eq!(retrieved.scale, transform.scale);
    }

    #[test]
    fn test_light_custom_parameters() {
        let mut world = World::new();
        let light = PointLight {
            intensity: 10.0,
            radius: 500.0,
            ..Default::default()
        };
        let entity = world.spawn(light).id();

        let retrieved = world.get::<PointLight>(entity).expect("Test: operation should succeed");
        assert_eq!(retrieved.intensity, 10.0);
        assert_eq!(retrieved.radius, 500.0);
    }

    #[test]
    fn test_entity_id_stability() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();

        assert_ne!(e1, e2);
    }

    #[test]
    fn test_component_removal_and_readd() {
        let mut world = World::new();
        let entity = world.spawn(Transform::default()).id();

        world.entity_mut(entity).remove::<Transform>();
        assert!(world.get::<Transform>(entity).is_none());

        world.entity_mut(entity).insert(Transform::default());
        assert!(world.get::<Transform>(entity).is_some());
    }

    #[test]
    fn test_sprite_color_variations() {
        let colors = [
            [1.0, 0.0, 0.0, 1.0], // Red
            [0.0, 1.0, 0.0, 1.0], // Green
            [0.0, 0.0, 1.0, 1.0], // Blue
            [1.0, 1.0, 0.0, 1.0], // Yellow
        ];

        for color in colors {
            let sprite = Sprite {
                color,
                ..Default::default()
            };
            assert_eq!(sprite.color, color);
        }
    }
}
