//! Physics Handle to Entity Mapping Integration Test
//!
//! This test verifies that the PhysicsWorld3D properly maintains
//! handle to Entity mappings for raycast, shapecast, and AABB queries.

use game_engine::physics::physics3d::*;
use bevy_ecs::prelude::*;
use glam::Vec3;

#[test]
fn test_handle_to_entity_mapping_integration() {
    let mut physics_world = PhysicsWorld3D::new();

    // Create test entities
    let entity1 = Entity::from_bits(1000);
    let entity2 = Entity::from_bits(2000);

    // Create rigid bodies and colliders
    let rb1 = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![0.0, 0.0, 0.0])
        .build();
    let rb_handle1 = physics_world.rigid_body_set.insert(rb1);

    let rb2 = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![10.0, 0.0, 0.0])
        .build();
    let rb_handle2 = physics_world.rigid_body_set.insert(rb2);

    // Create colliders
    let col1 = rapier3d::prelude::ColliderBuilder::ball(5.0).build();
    let col_handle1 = physics_world
        .collider_set
        .insert_with_parent(col1, rb_handle1, &mut physics_world.rigid_body_set);

    let col2 = rapier3d::prelude::ColliderBuilder::ball(5.0).build();
    let col_handle2 = physics_world
        .collider_set
        .insert_with_parent(col2, rb_handle2, &mut physics_world.rigid_body_set);

    // Add mappings
    physics_world.insert_collider_entity_mapping(col_handle1, entity1);
    physics_world.insert_collider_entity_mapping(col_handle2, entity2);

    // Test raycast
    let raycast_result = physics_world.raycast(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        20.0,
    );

    assert!(raycast_result.is_some(), "Raycast should hit entity1");
    let (hit_entity, distance, _point) = raycast_result.unwrap();
    assert_eq!(hit_entity, entity1, "Raycast should return entity1");
    assert!(distance > 0.0, "Distance should be positive");

    // Test query_aabb
    let aabb_results = physics_world.query_aabb(
        Vec3::new(-2.0, -2.0, -2.0),
        Vec3::new(2.0, 2.0, 2.0),
    );

    assert_eq!(aabb_results.len(), 1, "AABB query should find 1 entity");
    assert!(aabb_results.contains(&entity1), "AABB query should contain entity1");
    assert!(!aabb_results.contains(&entity2), "AABB query should not contain entity2");

    // Test shapecast
    let shape = rapier3d::parry::shape::SharedShape::ball(1.0);
    let shapecast_result = physics_world.shapecast(
        &shape,
        Vec3::new(0.0, 0.0, 0.0),
        glam::Quat::IDENTITY,
        Vec3::new(1.0, 0.0, 0.0),
        20.0,
    );

    assert!(shapecast_result.is_some(), "Shapecast should hit");
    let (hit_entity, _distance) = shapecast_result.unwrap();
    assert_eq!(hit_entity, entity1, "Shapecast should return entity1");

    // Test mapping removal
    let removed = physics_world.remove_collider_entity_mapping(col_handle1);
    assert_eq!(removed, Some(entity1), "Should remove entity1 mapping");

    let after_removal = physics_world.get_entity_by_collider(col_handle1);
    assert_eq!(after_removal, None, "Entity1 mapping should be removed");

    // Test that entity2 mapping still exists
    let entity2_lookup = physics_world.get_entity_by_collider(col_handle2);
    assert_eq!(entity2_lookup, Some(entity2), "Entity2 mapping should still exist");

    println!("✅ All handle to entity mapping tests passed!");
}

#[test]
fn test_mapping_management() {
    let mut physics_world = PhysicsWorld3D::new();

    // Create test collider
    let rb = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![0.0, 0.0, 0.0])
        .build();
    let rb_handle = physics_world.rigid_body_set.insert(rb);

    let collider = rapier3d::prelude::ColliderBuilder::ball(1.0).build();
    let col_handle = physics_world
        .collider_set
        .insert_with_parent(collider, rb_handle, &mut physics_world.rigid_body_set);

    // Test insert and get
    let test_entity = Entity::from_bits(999);
    physics_world.insert_collider_entity_mapping(col_handle, test_entity);

    let retrieved = physics_world.get_entity_by_collider(col_handle);
    assert_eq!(retrieved, Some(test_entity), "Should retrieve inserted entity");

    // Test get all mappings
    let mappings = physics_world.get_collider_entity_mappings();
    assert!(mappings.contains_key(&col_handle), "Mappings should contain the collider handle");
    assert_eq!(mappings.get(&col_handle), Some(&test_entity), "Mapping should match");

    // Test clear
    physics_world.clear_collider_entity_mappings();
    let after_clear = physics_world.get_entity_by_collider(col_handle);
    assert_eq!(after_clear, None, "Should be empty after clear");

    println!("✅ All mapping management tests passed!");
}

#[test]
fn test_multiple_colliders_same_entity() {
    let mut physics_world = PhysicsWorld3D::new();

    // Test that we can map multiple colliders to the same entity
    let entity = Entity::from_bits(5000);

    let rb1 = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![0.0, 0.0, 0.0])
        .build();
    let rb_handle1 = physics_world.rigid_body_set.insert(rb1);

    let rb2 = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![10.0, 0.0, 0.0])
        .build();
    let rb_handle2 = physics_world.rigid_body_set.insert(rb2);

    let col1 = rapier3d::prelude::ColliderBuilder::ball(1.0).build();
    let col_handle1 = physics_world
        .collider_set
        .insert_with_parent(col1, rb_handle1, &mut physics_world.rigid_body_set);

    let col2 = rapier3d::prelude::ColliderBuilder::ball(1.0).build();
    let col_handle2 = physics_world
        .collider_set
        .insert_with_parent(col2, rb_handle2, &mut physics_world.rigid_body_set);

    // Map both colliders to the same entity
    physics_world.insert_collider_entity_mapping(col_handle1, entity);
    physics_world.insert_collider_entity_mapping(col_handle2, entity);

    // Both should return the same entity
    let entity1 = physics_world.get_entity_by_collider(col_handle1);
    let entity2 = physics_world.get_entity_by_collider(col_handle2);

    assert_eq!(entity1, Some(entity), "Collider1 should map to entity");
    assert_eq!(entity2, Some(entity), "Collider2 should map to entity");

    println!("✅ Multiple colliders to same entity test passed!");
}
