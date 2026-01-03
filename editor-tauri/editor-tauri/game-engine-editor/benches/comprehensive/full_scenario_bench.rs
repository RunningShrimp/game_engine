// Full Scenario Benchmarks
//
// Comprehensive end-to-end benchmarks simulating real editor usage scenarios

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use std::collections::HashMap;
use uuid::Uuid;

// Simplified mock structures for comprehensive testing
#[derive(Clone)]
struct Entity {
    id: Uuid,
    name: String,
    transform: Transform,
    material_id: Option<String>,
}

#[derive(Clone, Copy)]
struct Transform {
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
}

struct EditorScene {
    entities: HashMap<Uuid, Entity>,
    materials: HashMap<String, Material>,
    history: Vec<SceneSnapshot>,
    current_snapshot: usize,
}

#[derive(Clone)]
struct Material {
    id: String,
    name: String,
    properties: HashMap<String, f32>,
}

#[derive(Clone)]
struct SceneSnapshot {
    entities: HashMap<Uuid, Entity>,
}

impl EditorScene {
    fn new() -> Self {
        Self {
            entities: HashMap::new(),
            materials: HashMap::new(),
            history: Vec::new(),
            current_snapshot: 0,
        }
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
    }

    fn remove_entity(&mut self, id: Uuid) -> Option<Entity> {
        self.entities.remove(&id)
    }

    fn update_entity_transform(&mut self, id: Uuid, transform: Transform) -> bool {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.transform = transform;
            true
        } else {
            false
        }
    }

    fn create_snapshot(&mut self) {
        let snapshot = SceneSnapshot {
            entities: self.entities.clone(),
        };

        // Keep only last 100 snapshots
        if self.history.len() >= 100 {
            self.history.remove(0);
        }

        self.history.push(snapshot);
        self.current_snapshot = self.history.len() - 1;
    }

    fn restore_snapshot(&mut self) -> bool {
        if self.current_snapshot > 0 && self.current_snapshot < self.history.len() {
            if let Some(snapshot) = self.history.get(self.current_snapshot) {
                self.entities = snapshot.entities.clone();
                return true;
            }
        }
        false
    }
}

// Test scenarios
fn scenario_create_large_scene(entity_count: usize) -> EditorScene {
    let mut scene = EditorScene::new();

    for i in 0..entity_count {
        let entity = Entity {
            id: Uuid::new_v4(),
            name: format!("Entity_{}", i),
            transform: Transform {
                position: ((i as f32 * 10.0) % 1000.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            },
            material_id: None,
        };
        scene.add_entity(entity);
    }

    scene
}

fn scenario_edit_session(entity_count: usize, operations: usize) -> EditorScene {
    let mut scene = scenario_create_large_scene(entity_count);
    let entity_ids: Vec<_> = scene.entities.keys().copied().collect();

    for i in 0..operations {
        // Simulate various edit operations
        match i % 5 {
            0 => {
                // Create snapshot
                scene.create_snapshot();
            }
            1 => {
                // Transform entity
                if let Some(&id) = entity_ids.get(i % entity_ids.len()) {
                    scene.update_entity_transform(
                        id,
                        Transform {
                            position: (i as f32, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0, 1.0),
                            scale: (1.0, 1.0, 1.0),
                        },
                    );
                }
            }
            2 => {
                // Add new entity
                let entity = Entity {
                    id: Uuid::new_v4(),
                    name: format!("NewEntity_{}", i),
                    transform: Transform {
                        position: (0.0, 0.0, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    },
                    material_id: None,
                };
                scene.add_entity(entity);
            }
            3 => {
                // Delete entity
                if let Some(&id) = entity_ids.get(i % entity_ids.len()) {
                    scene.remove_entity(id);
                }
            }
            _ => {
                // Restore snapshot
                scene.restore_snapshot();
            }
        }
    }

    scene
}

fn bench_scene_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_creation");
    group.measurement_time(Duration::from_secs(15));

    for entity_count in [1_000, 5_000, 10_000, 50_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| scenario_create_large_scene(black_box(count)));
            },
        );
    }

    group.finish();
}

fn bench_editing_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing_session");
    group.measurement_time(Duration::from_secs(15));

    for (entity_count, operations) in [
        (1_000, 100),
        (5_000, 500),
        (10_000, 1_000),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new(format!("n{}_ops{}", entity_count, operations), entity_count),
            &(entity_count, operations),
            |b, &(entities, ops)| {
                b.iter(|| scenario_edit_session(black_box(entities), black_box(ops)));
            },
        );
    }

    group.finish();
}

fn bench_undo_redo_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("undo_redo_session");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("1000_entities_100_snapshots", |b| {
        b.iter(|| {
            let mut scene = scenario_create_large_scene(1_000);

            // Create snapshots
            for i in 0..100 {
                for (_, entity) in scene.entities.iter_mut().take(10) {
                    entity.transform.position.0 += 1.0;
                }
                scene.create_snapshot();
            }

            // Undo/redo cycle
            for _ in 0..50 {
                scene.current_snapshot = scene.current_snapshot.saturating_sub(1);
                scene.restore_snapshot();

                if scene.current_snapshot < scene.history.len() {
                    scene.current_snapshot += 1;
                    scene.restore_snapshot();
                }
            }

            black_box(scene);
        });
    });

    group.finish();
}

fn bench_material_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_updates");
    group.measurement_time(Duration::from_secs(15));

    for (entity_count, material_count) in [(1_000, 10), (10_000, 50), (50_000, 100)].iter() {
        group.bench_with_input(
            BenchmarkId::new(format!("n{}_m{}", entity_count, material_count), entity_count),
            &(entity_count, material_count),
            |b, &(entities, materials)| {
                b.iter(|| {
                    let mut scene = scenario_create_large_scene(entities);

                    // Add materials
                    for i in 0..*materials {
                        let mat = Material {
                            id: format!("mat_{}", i),
                            name: format!("Material_{}", i),
                            properties: HashMap::new(),
                        };
                        scene.materials.insert(mat.id.clone(), mat);
                    }

                    // Assign materials to entities
                    for (i, entity) in scene.entities.iter_mut().take(entities).enumerate() {
                        entity.material_id = Some(format!("mat_{}", i % materials));
                    }

                    black_box(scene);
                });
            },
        );
    }

    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("batch_transform_1000", |b| {
        b.iter(|| {
            let mut scene = scenario_create_large_scene(10_000);
            let entity_ids: Vec<_> = scene.entities.keys().copied().take(1000).collect();

            for &id in &entity_ids {
                if let Some(entity) = scene.entities.get_mut(&id) {
                    entity.transform.position.0 += 10.0;
                    entity.transform.position.1 += 5.0;
                }
            }

            black_box(scene);
        });
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(15));

    for entity_count in [1_000, 10_000, 50_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| {
                    let scene = scenario_create_large_scene(count);
                    let size = std::mem::size_of_val(&scene);
                    black_box(size);
                });
            },
        );
    }

    group.finish();
}

fn bench_full_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_workflow");
    group.measurement_time(Duration::from_secs(20));

    group.bench_function("complete_editing_workflow", |b| {
        b.iter(|| {
            // Step 1: Create scene
            let mut scene = scenario_create_large_scene(5_000);

            // Step 2: Create initial snapshot
            scene.create_snapshot();

            // Step 3: Perform edits
            let entity_ids: Vec<_> = scene.entities.keys().copied().take(100).collect();
            for (i, &id) in entity_ids.iter().enumerate() {
                scene.update_entity_transform(
                    id,
                    Transform {
                        position: (i as f32, 0.0, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (2.0, 2.0, 2.0),
                    },
                );
            }

            // Step 4: Create another snapshot
            scene.create_snapshot();

            // Step 5: Undo
            scene.current_snapshot = scene.current_snapshot.saturating_sub(1);
            scene.restore_snapshot();

            // Step 6: Redo
            scene.current_snapshot += 1;
            scene.restore_snapshot();

            // Step 7: Add more entities
            for i in 0..100 {
                let entity = Entity {
                    id: Uuid::new_v4(),
                    name: format!("NewEntity_{}", i),
                    transform: Transform {
                        position: (0.0, i as f32, 0.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    },
                    material_id: None,
                };
                scene.add_entity(entity);
            }

            black_box(scene);
        });
    });

    group.finish();
}

criterion_group!(
    name = comprehensive_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(20))
        .sample_size(50);
    targets =
        bench_scene_creation,
        bench_editing_session,
        bench_undo_redo_session,
        bench_material_updates,
        bench_batch_operations,
        bench_memory_usage,
        bench_full_workflow
);

criterion_main!(comprehensive_benches);
