// Entity CRUD Benchmarks
//
// Measures the performance of entity Create, Read, Update, Delete operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Entity {
    id: Uuid,
    name: String,
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
    visible: bool,
    tags: Vec<String>,
}

struct EntityManager {
    entities: HashMap<Uuid, Entity>,
    name_index: HashMap<String, Uuid>,
}

impl EntityManager {
    fn new() -> Self {
        Self {
            entities: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    fn create(&mut self, entity: Entity) -> Result<(), String> {
        if self.name_index.contains_key(&entity.name) {
            return Err("Entity name already exists".to_string());
        }

        self.name_index.insert(entity.name.clone(), entity.id);
        self.entities.insert(entity.id, entity);
        Ok(())
    }

    fn read(&self, id: Uuid) -> Option<&Entity> {
        self.entities.get(&id)
    }

    fn read_by_name(&self, name: &str) -> Option<&Entity> {
        self.name_index
            .get(name)
            .and_then(|id| self.entities.get(id))
    }

    fn update(&mut self, id: Uuid, mut entity: Entity) -> Result<(), String> {
        if !self.entities.contains_key(&id) {
            return Err("Entity not found".to_string());
        }

        // Update name index if name changed
        if let Some(old_entity) = self.entities.get(&id) {
            if old_entity.name != entity.name {
                self.name_index.remove(&old_entity.name);
                self.name_index.insert(entity.name.clone(), id);
            }
        }

        entity.id = id;
        self.entities.insert(id, entity);
        Ok(())
    }

    fn delete(&mut self, id: Uuid) -> Result<Entity, String> {
        let entity = self.entities.remove(&id).ok_or("Entity not found")?;
        self.name_index.remove(&entity.name);
        Ok(entity)
    }

    fn list_all(&self) -> Vec<&Entity> {
        self.entities.values().collect()
    }

    fn count(&self) -> usize {
        self.entities.len()
    }

    fn find_by_tag(&self, tag: &str) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    }
}

// Test fixtures
fn create_test_entity(id: usize) -> Entity {
    Entity {
        id: Uuid::new_v4(),
        name: format!("Entity_{}", id),
        position: (id as f32 * 10.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        scale: (1.0, 1.0, 1.0),
        visible: true,
        tags: vec!["test".to_string(), format!("tag_{}", id % 10)],
    }
}

fn bench_entity_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_create");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let mut manager = EntityManager::new();
            b.iter(|| {
                for i in 0..n {
                    let entity = create_test_entity(i);
                    black_box(manager.create(black_box(entity))).ok();
                }
            });
        });
    }

    group.finish();
}

fn bench_entity_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_read");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        let mut manager = EntityManager::new();
        let mut ids = Vec::new();

        for i in 0..*count {
            let entity = create_test_entity(i);
            let id = entity.id;
            manager.create(entity).unwrap();
            ids.push(id);
        }

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                for i in 0..n {
                    black_box(manager.read(black_box(ids[i % ids.len()])));
                }
            });
        });
    }

    group.finish();
}

fn bench_entity_read_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_read_by_name");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        let mut manager = EntityManager::new();
        let mut names = Vec::new();

        for i in 0..*count {
            let entity = create_test_entity(i);
            let name = entity.name.clone();
            manager.create(entity).unwrap();
            names.push(name);
        }

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                for i in 0..n.min(100) {
                    black_box(manager.read_by_name(black_box(&names[i % names.len()])));
                }
            });
        });
    }

    group.finish();
}

fn bench_entity_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_update");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        let mut manager = EntityManager::new();
        let mut ids = Vec::new();

        for i in 0..*count {
            let entity = create_test_entity(i);
            let id = entity.id;
            manager.create(entity).unwrap();
            ids.push(id);
        }

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                for i in 0..n.min(100) {
                    let id = ids[i % ids.len()];
                    if let Some(entity) = manager.read(id) {
                        let mut updated = entity.clone();
                        updated.position.0 += 1.0;
                        black_box(manager.update(black_box(id), black_box(updated))).ok();
                    }
                }
            });
        });
    }

    group.finish();
}

fn bench_entity_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_delete");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            b.iter(|| {
                let mut manager = EntityManager::new();
                let mut ids = Vec::new();

                // Setup
                for i in 0..n {
                    let entity = create_test_entity(i);
                    let id = entity.id;
                    manager.create(entity).unwrap();
                    ids.push(id);
                }

                // Delete
                for id in ids {
                    black_box(manager.delete(black_box(id))).ok();
                }
            });
        });
    }

    group.finish();
}

fn bench_entity_find_by_tag(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_find_by_tag");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1_000, 10_000].iter() {
        let mut manager = EntityManager::new();

        for i in 0..*count {
            let entity = create_test_entity(i);
            manager.create(entity).unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                black_box(manager.find_by_tag(black_box("test")));
            });
        });
    }

    group.finish();
}

fn bench_entity_crud_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_crud_mixed");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("mixed_operations_1000", |b| {
        b.iter(|| {
            let mut manager = EntityManager::new();
            let mut ids = Vec::new();

            // Create 1000 entities
            for i in 0..1000 {
                let entity = create_test_entity(i);
                let id = entity.id;
                manager.create(entity).unwrap();
                ids.push(id);
            }

            // Read 100
            for i in 0..100 {
                manager.read(ids[i]);
            }

            // Update 50
            for i in 0..50 {
                if let Some(entity) = manager.read(ids[i]) {
                    let mut updated = entity.clone();
                    updated.position.0 += 1.0;
                    manager.update(ids[i], updated).unwrap();
                }
            }

            // Delete 25
            for i in 0..25 {
                manager.delete(ids[i]).unwrap();
            }

            black_box(manager.count());
        });
    });

    group.finish();
}

criterion_group!(
    name = entity_crud_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_entity_create,
        bench_entity_read,
        bench_entity_read_by_name,
        bench_entity_update,
        bench_entity_delete,
        bench_entity_find_by_tag,
        bench_entity_crud_mixed
);

criterion_main!(entity_crud_benches);
