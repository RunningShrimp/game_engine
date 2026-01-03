// VRAM Management Benchmarks
//
// Measures memory allocation, deallocation, and VRAM usage patterns

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

#[derive(Debug, Clone)]
struct VRAMAllocation {
    id: u32,
    size: usize,
    data: Vec<u8>,
}

struct VRAMManager {
    allocations: Vec<VRAMAllocation>,
    total_memory: usize,
    used_memory: usize,
}

impl VRAMManager {
    fn new(total_memory: usize) -> Self {
        Self {
            allocations: Vec::new(),
            total_memory,
            used_memory: 0,
        }
    }

    fn allocate(&mut self, size: usize) -> Option<u32> {
        if self.used_memory + size > self.total_memory {
            return None; // Out of memory
        }

        let id = self.allocations.len() as u32;
        let data = vec![0u8; size]; // Simulate actual allocation

        self.allocations.push(VRAMAllocation { id, size, data });
        self.used_memory += size;
        Some(id)
    }

    fn deallocate(&mut self, id: u32) -> bool {
        if let Some(pos) = self.allocations.iter().position(|a| a.id == id) {
            let allocation = self.allocations.remove(pos);
            self.used_memory -= allocation.size;
            true
        } else {
            false
        }
    }

    fn defragment(&mut self) {
        // Simulate defragmentation by compacting allocations
        self.allocations.sort_by_key(|a| a.id);
    }

    fn available_memory(&self) -> usize {
        self.total_memory - self.used_memory
    }

    fn utilization(&self) -> f64 {
        (self.used_memory as f64 / self.total_memory as f64) * 100.0
    }

    fn allocation_count(&self) -> usize {
        self.allocations.len()
    }
}

// Create test allocations simulating different asset types
fn create_texture_allocations(count: usize) -> Vec<usize> {
    (0..count)
        .map(|i| {
            // Simulate different texture sizes
            match i % 4 {
                0 => 1024 * 1024 * 4,      // 4K texture (16MB)
                1 => 2048 * 2048 * 4,      // 2K texture (16MB)
                2 => 1024 * 1024 * 4,      // 1K texture (4MB)
                _ => 512 * 512 * 4,        // 512x512 texture (1MB)
            }
        })
        .collect()
}

fn create_mesh_allocations(count: usize) -> Vec<usize> {
    (0..count)
        .map(|_| {
            // Simulate vertex + index buffers
            100_000 * 4 * 3 + 100_000 * 6 // ~1.5MB per mesh
        })
        .collect()
}

fn bench_vram_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_allocation");
    group.measurement_time(Duration::from_secs(10));

    for alloc_count in [10, 50, 100, 500].iter() {
        let allocations = create_texture_allocations(*alloc_count);
        let total_size: usize = allocations.iter().sum();

        group.throughput(Throughput::Bytes(total_size as u64));
        group.bench_with_input(
            BenchmarkId::new("textures", alloc_count),
            &allocations,
            |b, allocs| {
                let mut manager = VRAMManager::new(1024 * 1024 * 1024); // 1GB VRAM
                b.iter(|| {
                    for &size in allocs {
                        black_box(manager.allocate(black_box(size)));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_vram_deallocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_deallocation");
    group.measurement_time(Duration::from_secs(10));

    for alloc_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(alloc_count),
            alloc_count,
            |b, &count| {
                b.iter(|| {
                    let mut manager = VRAMManager::new(1024 * 1024 * 1024);
                    let mut ids = Vec::new();

                    // Allocate
                    let allocations = create_texture_allocations(count);
                    for size in allocations {
                        if let Some(id) = manager.allocate(size) {
                            ids.push(id);
                        }
                    }

                    // Deallocate
                    for id in ids {
                        black_box(manager.deallocate(black_box(id)));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_vram_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_fragmentation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("fragmentation_pattern", |b| {
        b.iter(|| {
            let mut manager = VRAMManager::new(1024 * 1024 * 512); // 512MB

            // Allocate and deallocate in a pattern that causes fragmentation
            for _ in 0..100 {
                let ids: Vec<_> = (0..10)
                    .filter_map(|_| manager.allocate(1024 * 1024))
                    .collect();

                // Deallocate every other allocation
                for (i, &id) in ids.iter().enumerate() {
                    if i % 2 == 0 {
                        manager.deallocate(id);
                    }
                }

                // Try to allocate larger blocks (will fail if fragmented)
                black_box(manager.allocate(1024 * 1024 * 10));
            }

            // Defragment
            black_box(manager.defragment());
        });
    });

    group.finish();
}

fn bench_vram_defragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_defragmentation");
    group.measurement_time(Duration::from_secs(10));

    let allocation_counts = [50, 100, 500];

    for count in allocation_counts.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut manager = VRAMManager::new(1024 * 1024 * 1024);
                    let mut ids = Vec::new();

                    // Create fragmented allocations
                    for _ in 0..count {
                        if let Some(id) = manager.allocate(1024 * 1024) {
                            ids.push(id);
                        }
                    }

                    // Deallocate randomly
                    for &id in ids.iter().step_by(3) {
                        manager.deallocate(id);
                    }

                    // Defragment
                    black_box(manager.defragment());
                });
            },
        );
    }

    group.finish();
}

fn bench_vram_memory_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_memory_pool");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("texture_pool_100", |b| {
        b.iter(|| {
            let mut manager = VRAMManager::new(1024 * 1024 * 512);
            let mut ids = Vec::new();

            // Allocate 100 textures
            for _ in 0..100 {
                if let Some(id) = manager.allocate(1024 * 1024 * 4) {
                    ids.push(id);
                }
            }

            // Randomly free and reallocate (simulating streaming)
            for i in 0..1000 {
                if let Some(&id) = ids.get(i % ids.len()) {
                    manager.deallocate(id);
                    ids[i % ids.len()] = manager.allocate(1024 * 1024 * 4).unwrap();
                }
            }

            black_box(manager.utilization());
        });
    });

    group.finish();
}

fn bench_vram_utilization(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_utilization");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("utilization_tracking", |b| {
        b.iter(|| {
            let mut manager = VRAMManager::new(1024 * 1024 * 1024);

            // Fill to different utilization levels
            for i in 0..10 {
                let count = 1024 * 1024 * 100 * i; // 0%, 10%, 20%, ... 90%
                for _ in 0..count {
                    manager.allocate(1024);
                }
                black_box(manager.utilization());
            }
        });
    });

    group.finish();
}

fn bench_vram_asset_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("vram_asset_streaming");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("streaming_1000_textures", |b| {
        b.iter(|| {
            let mut manager = VRAMManager::new(1024 * 1024 * 512);
            let mut ids = Vec::new();

            // Simulate streaming in textures
            for i in 0..1000 {
                if ids.len() > 500 {
                    // Remove oldest (LRU)
                    if let Some(id) = ids.remove(0) {
                        manager.deallocate(id);
                    }
                }

                // Add new
                if let Some(id) = manager.allocate(1024 * 1024 * 4) {
                    ids.push(id);
                }

                black_box(manager.utilization());
            }
        });
    });

    group.finish();
}

criterion_group!(
    name = vram_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_vram_allocation,
        bench_vram_deallocation,
        bench_vram_fragmentation,
        bench_vram_defragmentation,
        bench_vram_memory_pool,
        bench_vram_utilization,
        bench_vram_asset_streaming
);

criterion_main!(vram_benches);
