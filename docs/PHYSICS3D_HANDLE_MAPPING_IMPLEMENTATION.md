# Physics3D Handle to Entity Mapping Implementation

## Overview

This document describes the implementation of the handle to Entity mapping system for the PhysicsWorld3D, which resolves the TODO markers in `physics/physics3d.rs:12` and `:199`.

## Problem Statement

Previously, the physics system used a placeholder entity constant (`PLACEHOLDER_ENTITY`) instead of real ECS Entity IDs. This meant that raycast, shapecast, and query_aabb operations returned meaningless entity IDs, making it impossible to identify which game entities were involved in physics queries.

## Solution

Implemented a proper handle-to-Entity mapping system using a HashMap that tracks the relationship between Rapier physics collider handles and Bevy ECS entities.

## Implementation Details

### 1. Data Structure

Added a HashMap to `PhysicsWorld3D`:

```rust
#[derive(Resource)]
pub struct PhysicsWorld3D {
    // ... existing fields ...
    /// Collider handle 到 Entity 的映射表
    collider_entity_map: HashMap<ColliderHandle, Entity>,
}
```

### 2. Mapping Management Methods

Implemented five public methods for managing the mappings:

#### `insert_collider_entity_mapping`
```rust
pub fn insert_collider_entity_mapping(&mut self, handle: ColliderHandle, entity: Entity)
```
Adds a new mapping between a collider handle and an entity.

#### `remove_collider_entity_mapping`
```rust
pub fn remove_collider_entity_mapping(&mut self, handle: ColliderHandle) -> Option<Entity>
```
Removes a mapping and returns the associated entity if it existed.

#### `get_entity_by_collider`
```rust
pub fn get_entity_by_collider(&self, handle: ColliderHandle) -> Option<Entity>
```
Retrieves the entity associated with a collider handle.

#### `get_collider_entity_mappings`
```rust
pub fn get_collider_entity_mappings(&self) -> &HashMap<ColliderHandle, Entity>
```
Returns a read-only reference to the complete mapping table.

#### `clear_collider_entity_mappings`
```rust
pub fn clear_collider_entity_mappings(&mut self)
```
Clears all mappings (useful for world reset).

### 3. System Integration

Updated `init_physics_bodies_3d` to automatically create mappings when colliders are created:

```rust
// Create collider
let col_handle = collider_set.insert_with_parent(collider, rb_handle, rigid_body_set);

// Add mapping
physics.insert_collider_entity_mapping(col_handle, entity);
```

### 4. Query Methods Updated

#### `raycast`
Now returns the actual Entity that was hit:
```rust
let entity = self.get_entity_by_collider(collider_handle);
if let Some(entity) = entity {
    closest_hit = Some((entity, distance, hit_point));
}
```

#### `shapecast`
Now returns the actual Entity:
```rust
let entity = self.get_entity_by_collider(collider_handle);
if let Some(entity) = entity {
    closest_hit = Some((entity, final_distance));
}
```

#### `query_aabb`
Now returns a Vec of actual Entities:
```rust
if let Some(entity) = self.get_entity_by_collider(collider_handle) {
    hit_entities.push(entity);
}
```

## Testing

### Unit Tests

Added comprehensive unit tests in `physics3d.rs`:

1. **`test_raycast`** - Verifies raycast returns real entity
2. **`test_collider_entity_mapping`** - Tests basic CRUD operations
3. **`test_query_aabb_with_mapping`** - Tests AABB queries with real entities
4. **`test_shapecast_with_mapping`** - Tests shapecast with real entities

### Integration Tests

Created `tests/physics_handle_mapping_test.rs` with:

1. **`test_handle_to_entity_mapping_integration`** - End-to-end test
2. **`test_mapping_management`** - Tests all mapping management methods
3. **`test_multiple_colliders_same_entity`** - Tests multiple colliders per entity

## Usage Example

```rust
use game_engine::physics::physics3d::*;
use bevy_ecs::prelude::*;

fn setup_physics(mut physics: ResMut<PhysicsWorld3D>, mut commands: Commands) {
    // Spawn entity with physics
    let entity = commands.spawn_empty().id();

    // Create rigid body
    let rb = RigidBodyBuilder::dynamic().build();
    let rb_handle = physics.rigid_body_set.insert(rb);

    // Create collider
    let collider = ColliderBuilder::ball(1.0).build();
    let col_handle = physics.collider_set.insert_with_parent(
        collider,
        rb_handle,
        &mut physics.rigid_body_set
    );

    // Map collider to entity (automatic in init_physics_bodies_3d)
    physics.insert_collider_entity_mapping(col_handle, entity);

    commands.entity(entity).insert(RigidBody3D { handle: rb_handle });
}

fn check_collisions(physics: Res<PhysicsWorld3D>) {
    // Raycast and get real entity
    if let Some((entity, distance, point)) = physics.raycast(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        100.0,
    ) {
        println!("Hit entity {:?} at distance {}", entity, distance);
        // Can now use 'entity' with ECS queries
    }
}
```

## Memory Safety

- **Ownership**: The HashMap owns the mappings, ensuring proper lifecycle management
- **Borrow Checker**: All methods respect Rust's borrowing rules
- **Cleanup**: When entities are despawned, mappings should be removed via `remove_collider_entity_mapping`

## Performance Considerations

- **Lookup**: O(1) average case for HashMap lookups
- **Memory**: One entry per collider, typically minimal overhead
- **Updates**: Only updated on collider creation/removal, not per-frame

## Future Enhancements

1. **Automatic Cleanup**: Add a system that removes mappings when entities are despawned
2. **Bi-directional Mapping**: Add Entity → ColliderHandle mapping for reverse lookups
3. **Query Pipeline**: Integrate with Rapier's QueryPipeline for better performance
4. **Debug Visualization**: Add debug rendering to visualize mappings

## Migration Notes

### Breaking Changes
- None - this is an internal implementation detail

### API Changes
- Query methods now return real Entity IDs instead of PLACEHOLDER_ENTITY
- New public methods for mapping management

### Compatibility
- Fully backward compatible with existing code
- Existing tests updated to use new system

## Files Modified

1. `/game_engine/src/physics/physics3d.rs`
   - Added HashMap field
   - Added mapping management methods
   - Updated query methods
   - Added unit tests
   - Removed PLACEHOLDER_ENTITY constant

2. `/tests/physics_handle_mapping_test.rs` (new)
   - Integration tests for mapping system

## TODO Items Resolved

✅ **Line 12**: "实现proper handle -> Entity映射，使用实际的Entity关联"
   - Implemented via HashMap<ColliderHandle, Entity>

✅ **Line 199**: "使用 collider_handle 创建更真实的实体ID映射"
   - Implemented in query_aabb and all other query methods

## Verification

Run tests with:
```bash
cargo test physics3d::tests
cargo test --test physics_handle_mapping_test
```

## Conclusion

The handle-to-Entity mapping system is now fully implemented, tested, and ready for use. All physics queries now return meaningful Entity IDs that can be used with the ECS system for game logic.
