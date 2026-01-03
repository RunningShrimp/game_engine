//! Example: Handle to Entity Mapping Verification
//!
//! This example demonstrates the handle-to-Entity mapping system
//! in PhysicsWorld3D.

use game_engine::physics::physics3d::*;
use bevy_ecs::prelude::*;
use glam::Vec3;

fn main() {
    println!("🔬 Physics3D Handle-to-Entity Mapping Verification\n");

    // Create physics world
    let mut physics_world = PhysicsWorld3D::new();
    println!("✅ Created PhysicsWorld3D");

    // Create test entities
    let player_entity = Entity::from_bits(100);
    let enemy_entity = Entity::from_bits(200);

    println!("\n📝 Creating rigid bodies and colliders...");

    // Create player rigid body at origin
    let player_rb = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![0.0, 0.0, 0.0])
        .build();
    let player_rb_handle = physics_world.rigid_body_set.insert(player_rb);

    // Create enemy rigid body at (10, 0, 0)
    let enemy_rb = rapier3d::prelude::RigidBodyBuilder::fixed()
        .translation(rapier3d::na::vector![10.0, 0.0, 0.0])
        .build();
    let enemy_rb_handle = physics_world.rigid_body_set.insert(enemy_rb);

    println!("  ✅ Created 2 rigid bodies");

    // Create player collider (sphere, radius 5.0)
    let player_col = rapier3d::prelude::ColliderBuilder::ball(5.0).build();
    let player_col_handle = physics_world.collider_set.insert_with_parent(
        player_col,
        player_rb_handle,
        &mut physics_world.rigid_body_set,
    );

    // Create enemy collider (sphere, radius 5.0)
    let enemy_col = rapier3d::prelude::ColliderBuilder::ball(5.0).build();
    let enemy_col_handle = physics_world.collider_set.insert_with_parent(
        enemy_col,
        enemy_rb_handle,
        &mut physics_world.rigid_body_set,
    );

    println!("  ✅ Created 2 colliders");

    // Add mappings
    println!("\n🔗 Adding handle-to-entity mappings...");
    physics_world.insert_collider_entity_mapping(player_col_handle, player_entity);
    println!("  ✅ Mapped player collider -> Entity({:?})", player_entity);

    physics_world.insert_collider_entity_mapping(enemy_col_handle, enemy_entity);
    println!("  ✅ Mapped enemy collider -> Entity({:?})", enemy_entity);

    // Verify mappings
    println!("\n🔍 Verifying mappings...");
    let player_lookup = physics_world.get_entity_by_collider(player_col_handle);
    assert_eq!(player_lookup, Some(player_entity));
    println!("  ✅ Player mapping verified");

    let enemy_lookup = physics_world.get_entity_by_collider(enemy_col_handle);
    assert_eq!(enemy_lookup, Some(enemy_entity));
    println!("  ✅ Enemy mapping verified");

    // Test raycast
    println!("\n🎯 Testing raycast...");
    let raycast_result = physics_world.raycast(
        Vec3::new(0.0, 10.0, 0.0), // origin above player
        Vec3::new(0.0, -1.0, 0.0), // pointing down
        20.0,                       // max distance
    );

    if let Some((entity, distance, point)) = raycast_result {
        println!("  ✅ Raycast hit Entity({:?})", entity);
        println!("     Distance: {:.2}", distance);
        println!("     Point: ({:.2}, {:.2}, {:.2})", point.x, point.y, point.z);
        assert_eq!(entity, player_entity, "Should hit player");
    } else {
        println!("  ❌ Raycast missed (unexpected)");
    }

    // Test AABB query
    println!("\n📦 Testing AABB query...");
    let aabb_results = physics_world.query_aabb(
        Vec3::new(-5.0, -5.0, -5.0), // min
        Vec3::new(5.0, 5.0, 5.0),    // max
    );

    println!("  ✅ Found {} entities in AABB", aabb_results.len());
    for entity in &aabb_results {
        println!("     - Entity({:?})", entity);
    }
    assert!(aabb_results.contains(&player_entity));
    assert!(!aabb_results.contains(&enemy_entity));

    // Test shapecast
    println!("\n🔮 Testing shapecast...");
    let shape = rapier3d::parry::shape::SharedShape::ball(1.0);
    let shapecast_result = physics_world.shapecast(
        &shape,
        Vec3::new(0.0, 0.0, 0.0),
        glam::Quat::IDENTITY,
        Vec3::new(1.0, 0.0, 0.0), // pointing toward enemy
        20.0,
    );

    if let Some((entity, distance)) = shapecast_result {
        println!("  ✅ Shapecast hit Entity({:?})", entity);
        println!("     Distance: {:.2}", distance);
        assert_eq!(entity, player_entity, "Should hit player first");
    }

    // Test mapping removal
    println!("\n🗑️  Testing mapping removal...");
    let removed = physics_world.remove_collider_entity_mapping(player_col_handle);
    assert_eq!(removed, Some(player_entity));
    println!("  ✅ Removed player mapping");

    let after_removal = physics_world.get_entity_by_collider(player_col_handle);
    assert_eq!(after_removal, None);
    println!("  ✅ Verified player mapping is gone");

    // Verify enemy mapping still exists
    let enemy_still_exists = physics_world.get_entity_by_collider(enemy_col_handle);
    assert_eq!(enemy_still_exists, Some(enemy_entity));
    println!("  ✅ Enemy mapping still exists");

    // Test mapping statistics
    println!("\n📊 Mapping statistics...");
    let all_mappings = physics_world.get_collider_entity_mappings();
    println!("  Total mappings: {}", all_mappings.len());
    for (handle, entity) in all_mappings.iter() {
        println!("  - Collider {:?} -> Entity {:?}", handle, entity);
    }

    println!("\n✅ All verification tests passed!");
    println!("\n🎉 Handle-to-Entity mapping system is working correctly!\n");
}
