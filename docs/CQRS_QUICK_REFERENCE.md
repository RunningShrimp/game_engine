# CQRS Quick Reference

## Physics Module

### Setup
```rust
use game_engine::domain::cqrs::CqrsManager;
use game_engine::physics::cqrs::PhysicsApplicationService;
use std::sync::Arc;

let cqrs = Arc::new(CqrsManager::new());
let physics = PhysicsApplicationService::new(cqrs);
```

### Queries (Read)

#### Get Position
```rust
let pos = physics.get_position(body_id, &world)?;
```

#### Get Bodies in Radius
```rust
let nearby = physics.query_in_radius(center, 100.0, &world);
```

#### Get Dynamic Bodies
```rust
let dynamic = physics.get_dynamic_bodies(&world);
```

#### Batch Query
```rust
let model = physics.query_model().read().unwrap();
let positions = model.batch_get_positions(&ids);
```

### Commands (Write)

#### Create Body
```rust
let command = CreateRigidBodyCommand { body };
cqrs.execute_command(command, &mut world)?;
```

#### Update Position
```rust
physics.update_position(id, new_position, &mut world)?;
```

#### Apply Impulse
```rust
physics.apply_impulse(id, impulse, &mut world)?;
```

#### Set Velocity
```rust
physics.set_velocity(id, velocity, &mut world)?;
```

#### Remove Body
```rust
physics.remove_body(id, &mut world)?;
```

## Render Module

### Setup
```rust
use game_engine::render::cqrs::RenderApplicationService;

let render = RenderApplicationService::new(cqrs);
```

### Queries (Read)

#### Get Visibility
```rust
let visible = render.get_visibility(object_id, &world)?;
```

#### Get Transform
```rust
let transform = render.get_world_transform(object_id, &world)?;
```

#### Get Visible Objects
```rust
let visible_objects = render.get_visible_objects(&world);
```

#### Get Static Objects
```rust
let static_objects = render.get_static_objects(&world);
```

#### Query in Radius
```rust
let nearby = render.query_in_radius(center, 100.0, &world);
```

#### Batch Get Transforms
```rust
let transforms = render.batch_get_transforms(&ids, &world);
```

### Zero-Allocation Iterators

#### Iterate Visible Objects
```rust
let model = render.query_model().read().unwrap();
let visible: Vec<_> = model.iter_visible_objects()
    .filter(|id| should_render(id))
    .collect();
```

#### Iterate in Radius
```rust
let nearby: Vec<_> = model.iter_in_radius(center, radius)
    .collect();
```

#### Batch with Buffer Reuse
```rust
let mut buffer = Vec::new();
model.batch_get_transforms_to(&ids, &mut buffer);
```

### Commands (Write)

#### Update Transform
```rust
render.update_transform(id, new_transform, &mut world)?;
```

#### Set Visibility
```rust
render.set_visibility(id, true, &mut world)?;
```

#### Update Material
```rust
render.update_material(id, "material_id".to_string(), &mut world)?;
```

#### Remove Object
```rust
render.remove_object(id, &mut world)?;
```

## Performance Tips

### Do's
- ✓ Use query models for read operations
- ✓ Batch multiple queries
- ✓ Use iterators instead of collecting
- ✓ Reuse buffers for batch operations

### Don'ts
- ✗ Don't access physics world directly for queries
- ✗ Don't loop individual queries
- ✗ Don't collect intermediate results unnecessarily
- ✗ Don't use commands for read operations

## Common Patterns

### Batch Position Update
```rust
// Get current positions
let ids = vec![id1, id2, id3];
let positions = physics.query_model()
    .read()
    .unwrap()
    .batch_get_positions(&ids);

// Update positions
for (id, pos) in ids.iter().zip(positions.iter()) {
    if let Some(new_pos) = pos {
        physics.update_position(*id, *new_pos, &mut world)?;
    }
}
```

### Frustum Culling with CQRS
```rust
let model = render.query_model().read().unwrap();
let visible: Vec<_> = model.iter_in_frustum(camera_center, camera_radius)
    .filter(|id| is_in_frustum(*id))
    .collect();

for obj_id in visible {
    render_object(obj_id);
}
```

### Spatial Query with Physics
```rust
let nearby = physics.query_in_radius(player_pos, 100.0, &world);
for body_id in nearby {
    if is_collision(body_id) {
        handle_collision(body_id);
    }
}
```

## Testing

### Unit Test
```rust
#[test]
fn test_query() {
    let model = PhysicsQueryModel::from_world(&bodies);
    assert_eq!(model.body_count(), bodies.len());
}
```

### Performance Test
```bash
cargo test --lib physics::cqrs_performance_tests -- --ignored
```

## Error Handling

All commands return `Result<(), String>`:
```rust
if let Err(e) = physics.update_position(id, pos, &mut world) {
    eprintln!("Update failed: {}", e);
}
```

All queries return `Option<T>`:
```rust
if let Some(pos) = physics.get_position(id, &world) {
    // Use position
}
```

## Files

- Core: `game_engine/src/domain/cqrs.rs`
- Physics: `game_engine/src/physics/cqrs.rs`
- Render: `game_engine/src/render/cqrs.rs`
- Tests: `game_engine/src/physics/cqrs_performance_tests.rs`
- Tests: `game_engine/src/render/cqrs_performance_tests.rs`

## Performance Targets

- Single position lookup: **20-30% faster**
- Batch queries: **40-50% faster**
- Spatial queries: **25-35% faster**

Run `cargo test --lib test_full_benchmark_suite -- --ignored` to verify.
