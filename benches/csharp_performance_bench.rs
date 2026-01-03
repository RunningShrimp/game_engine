// C# 脚本性能基准测试
//
// 测试进程池和热重载的性能提升。

#[cfg(feature = "csharp")]
use {
    criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId},
    game_engine::scripting::csharp::{CSharpContext, CSharpConfig},
    std::time::Duration,
};

#[cfg(feature = "csharp")]
fn bench_csharp_execution(c: &mut Criterion) {
    let mut ctx = CSharpContext::new();

    // 确保 .NET 运行时已初始化
    if let Err(e) = ctx.ensure_runtime_initialized() {
        tracing::error!("Failed to initialize .NET runtime: {}", e);
        return;
    }

    let script_code = r#"
using System;

public class BenchScript {
    public static int Main() {
        int sum = 0;
        for (int i = 0; i < 1000; i++) {
            sum += i;
        }
        return sum;
    }
}
"#;

    // 测试编译缓存性能
    let mut group = c.benchmark_group("csharp_execution");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    // 第一次执行（编译）
    group.bench_function("first_execution_compile", |b| {
        b.iter(|| {
            let _ = black_box(
                ctx.execute("bench_script", Some(script_code))
            );
        });
    });

    // 后续执行（使用缓存）
    group.bench_function("cached_execution", |b| {
        b.iter(|| {
            let _ = black_box(
                ctx.execute("bench_script", Some(script_code))
            );
        });
    });

    group.finish();
}

#[cfg(feature = "csharp")]
fn bench_process_pool(c: &mut Criterion) {
    let mut ctx = CSharpContext::new();

    if let Err(e) = ctx.ensure_runtime_initialized() {
        tracing::error!("Failed to initialize .NET runtime: {}", e);
        return;
    }

    let script_code = r#"
using System;

public class PoolBenchScript {
    public static int Main() {
        return 42;
    }
}
"#;

    let mut group = c.benchmark_group("process_pool");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));

    // 测试并发执行性能
    for num_executions in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_executions", num_executions),
            num_executions,
            |b, &n| {
                b.iter(|| {
                    for _ in 0..n {
                        let _ = black_box(
                            ctx.execute("pool_bench", Some(script_code))
                        );
                    }
                });
            }
        );
    }

    group.finish();
}

#[cfg(feature = "csharp")]
fn bench_hot_reload(c: &mut Criterion) {
    let script_dir = std::path::PathBuf::from("./benches/test_scripts");
    std::fs::create_dir_all(&script_dir).ok();

    let mut ctx = CSharpContext::new();

    if let Err(e) = ctx.ensure_runtime_initialized() {
        tracing::error!("Failed to initialize .NET runtime: {}", e);
        return;
    }

    // 启用热重载
    let _ = ctx.enable_hot_reload(vec![script_dir.clone()], 100);

    let mut group = c.benchmark_group("hot_reload");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("check_hot_reload", |b| {
        b.iter(|| {
            let _ = black_box(ctx.check_hot_reload());
        });
    });

    group.finish();
}

#[cfg(feature = "csharp")]
criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10))
        .sample_size(20);
    targets = bench_csharp_execution, bench_process_pool, bench_hot_reload
);

#[cfg(feature = "csharp")]
criterion_main!(benches);

#[cfg(not(feature = "csharp"))]
fn main() {
    eprintln!("This benchmark requires the 'csharp' feature.");
    eprintln!("Run with: cargo bench --features csharp");
}
