// Indirect Drawing Benchmarks
//
// Measures the performance of indirect drawing compared to traditional drawing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

// Simulate traditional draw calls (one per instance)
fn traditional_draw_calls(instance_count: usize) -> Vec<u32> {
    (0..instance_count).map(|i| i as u32).collect()
}

// Simulate indirect draw calls (batched)
fn indirect_draw_calls(instance_count: usize, batch_size: usize) -> Vec<DrawCommand> {
    (0..(instance_count + batch_size - 1) / batch_size)
        .map(|batch| DrawCommand {
            vertex_count: (batch_size * 36) as u32, // 36 vertices per cube
            instance_count: if (batch + 1) * batch_size > instance_count {
                (instance_count - batch * batch_size) as u32
            } else {
                batch_size as u32
            },
            first_vertex: 0,
            first_instance: (batch * batch_size) as u32,
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct DrawCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

// Simulate command buffer overhead
fn execute_draw_calls_traditional(commands: &[u32]) -> usize {
    commands.iter().map(|_| {
        // Simulate driver overhead per draw call
        1_usize
    }).sum()
}

fn execute_draw_calls_indirect(commands: &[DrawCommand]) -> usize {
    commands.len()
}

fn bench_traditional_draw_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("traditional_draw_calls");
    group.measurement_time(Duration::from_secs(10));

    for instance_count in [1_000, 5_000, 10_000, 50_000].iter() {
        group.throughput(Throughput::Elements(*instance_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(instance_count),
            instance_count,
            |b, &count| {
                let commands = traditional_draw_calls(count);
                b.iter(|| execute_draw_calls_traditional(black_box(&commands)));
            },
        );
    }

    group.finish();
}

fn bench_indirect_draw_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("indirect_draw_calls");
    group.measurement_time(Duration::from_secs(10));

    for instance_count in [1_000, 5_000, 10_000, 50_000].iter() {
        for batch_size in [100, 500, 1_000].iter() {
            group.throughput(Throughput::Elements(*instance_count as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("n{}_batch{}", instance_count, batch_size), instance_count),
                &(*instance_count, *batch_size),
                |b, &(count, batch)| {
                    let commands = indirect_draw_calls(count, batch);
                    b.iter(|| execute_draw_calls_indirect(black_box(&commands)));
                },
            );
        }
    }

    group.finish();
}

fn bench_command_buffer_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_buffer_generation");
    group.measurement_time(Duration::from_secs(10));

    for instance_count in [1_000, 5_000, 10_000, 50_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("traditional", instance_count),
            instance_count,
            |b, &count| {
                b.iter(|| traditional_draw_calls(black_box(count)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("indirect_batch500", instance_count),
            instance_count,
            |b, &count| {
                b.iter(|| indirect_draw_calls(black_box(count), 500));
            },
        );
    }

    group.finish();
}

fn bench_draw_call_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_call_reduction");
    group.measurement_time(Duration::from_secs(10));

    let instance_count = 10_000;

    // Traditional
    group.bench_function("traditional", |b| {
        let commands = traditional_draw_calls(instance_count);
        b.iter(|| execute_draw_calls_traditional(black_box(&commands)));
    });

    // Indirect with different batch sizes
    for batch_size in [100, 500, 1_000, 2_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("indirect", batch_size),
            batch_size,
            |b, &batch| {
                let commands = indirect_draw_calls(instance_count, batch);
                b.iter(|| execute_draw_calls_indirect(black_box(&commands)));
            },
        );
    }

    group.finish();
}

// Calculate efficiency metrics
fn calculate_draw_call_reduction(traditional: usize, indirect: usize) -> f64 {
    ((traditional - indirect) as f64 / traditional as f64) * 100.0
}

fn bench_draw_call_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_call_comparison");
    group.measurement_time(Duration::from_secs(10));

    let test_cases = vec![
        (1_000, 100),
        (5_000, 500),
        (10_000, 500),
        (50_000, 1_000),
    ];

    for (instance_count, batch_size) in test_cases {
        let traditional = traditional_draw_calls(instance_count);
        let indirect = indirect_draw_calls(instance_count, batch_size);

        let reduction_pct = calculate_draw_call_reduction(traditional.len(), indirect.len());

        println!(
            "Instances: {}, Batch: {} -> {} draw calls ({}% reduction)",
            instance_count,
            batch_size,
            indirect.len(),
            reduction_pct
        );

        group.bench_with_input(
            BenchmarkId::new(format!("{}_{}", instance_count, batch_size), instance_count),
            &(traditional, indirect),
            |b, (trad, ind)| {
                b.iter(|| {
                    execute_draw_calls_traditional(black_box(trad));
                    execute_draw_calls_indirect(black_box(ind));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = indirect_draw_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_traditional_draw_calls,
        bench_indirect_draw_calls,
        bench_command_buffer_generation,
        bench_draw_call_reduction,
        bench_draw_call_comparison
);

criterion_main!(indirect_draw_benches);
