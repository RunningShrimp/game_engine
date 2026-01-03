// C# 脚本性能基准测试
//
// 测试DotNetCliHost的性能，特别是编译缓存的加速效果。

#[cfg(feature = "csharp")]
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
#[cfg(feature = "csharp")]
use game_engine::scripting::csharp_dotnet::DotNetCliHost;

/// 简单的Hello World C#脚本
const HELLO_WORLD_SCRIPT: &str = r#"
using System;

public class HelloWorld {
    public static string SayHello() {
        return "Hello from C#!";
    }
}
"#;

/// 更复杂的计算脚本
const CALCULATION_SCRIPT: &str = r#"
using System;
using System.Linq;

public class Calculator {
    public static int CalculateSum(int n) {
        return Enumerable.Range(1, n).Sum();
    }

    public static double CalculateAverage(int[] numbers) {
        return numbers.Average();
    }
}
"#;

/// 游戏逻辑脚本
const GAME_LOGIC_SCRIPT: &str = r#"
using System;
using GameEngine;

public class PlayerController {
    private static float speed = 5.0f;

    public static Vector3 Move(Vector3 position, Vector3 direction, float deltaTime) {
        return position + direction * speed * deltaTime;
    }

    public static bool CanJump(bool isGrounded) {
        return isGrounded;
    }
}
"#;

#[cfg(feature = "csharp")]
fn bench_hello_world(c: &mut Criterion) {
    let host = match DotNetCliHost::initialize() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("⚠️  Skipping benchmark: .NET SDK not available ({})", e);
            eprintln!("Install .NET SDK 8.0+ to run C# performance benchmarks");
            return;
        }
    };

    c.bench_function("hello_world_first_run", |b| {
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(HELLO_WORLD_SCRIPT, "hello_world"));
            // 清除缓存以确保每次都是首次编译
            let _ = host.clear_cache();
        });
    });

    c.bench_function("hello_world_cached", |b| {
        // 预热：编译一次以填充缓存
        let _ = host.compile_and_execute(HELLO_WORLD_SCRIPT, "hello_world_cached");

        b.iter(|| {
            let _ = black_box(host.compile_and_execute(HELLO_WORLD_SCRIPT, "hello_world_cached"));
        });
    });
}

#[cfg(feature = "csharp")]
fn bench_calculation(c: &mut Criterion) {
    let host = match DotNetCliHost::initialize() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("⚠️  Skipping benchmark: .NET SDK not available ({})", e);
            return;
        }
    };

    c.bench_function("calculation_first_run", |b| {
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(CALCULATION_SCRIPT, "calculation"));
            let _ = host.clear_cache();
        });
    });

    c.bench_function("calculation_cached", |b| {
        let _ = host.compile_and_execute(CALCULATION_SCRIPT, "calculation");
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(CALCULATION_SCRIPT, "calculation"));
        });
    });
}

#[cfg(feature = "csharp")]
fn bench_cache_hit_rate(c: &mut Criterion) {
    let host = match DotNetCliHost::initialize() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("⚠️  Skipping benchmark: .NET SDK not available ({})", e);
            return;
        }
    };

    let mut group = c.benchmark_group("cache_hit_rate");

    // 测试不同的缓存命中率场景
    for hit_rate in [0, 25, 50, 75, 100].iter() {
        let script_name = format!("hit_rate_{}", hit_rate);

        group.bench_with_input(
            BenchmarkId::new(format!("{}%_hits", hit_rate), hit_rate),
            |b, &_hit_rate| {
                b.iter(|| {
                    // 使用多个不同的脚本模拟不同的命中率
                    let scripts = vec![HELLO_WORLD_SCRIPT, CALCULATION_SCRIPT, GAME_LOGIC_SCRIPT];
                    let script = scripts.iter().cycle().take(100).collect::<Vec<_>>();

                    for (i, s) in script.iter().enumerate() {
                        let name = format!("{}_{}", script_name, i % scripts.len());
                        let _ = black_box(host.compile_and_execute(s, &name));
                    }
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "csharp")]
fn bench_script_sizes(c: &mut Criterion) {
    let host = match DotNetCliHost::initialize() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("⚠️  Skipping benchmark: .NET SDK not available ({})", e);
            return;
        }
    };

    let mut group = c.benchmark_group("script_size_impact");

    // 小型脚本 (~100 bytes)
    let small_script = r#"public class Small { public static int Run() => 42; }"#;

    // 中型脚本 (~500 bytes)
    let medium_script = r#"
using System;

public class Medium {
    private static int[] data = new int[100];

    public static int Process() {
        int sum = 0;
        for (int i = 0; i < data.Length; i++) {
            data[i] = i * 2;
            sum += data[i];
        }
        return sum;
    }
}
"#;

    // 大型脚本 (~2000 bytes)
    let large_script = r#"
using System;
using System.Collections.Generic;
using System.Linq;

public class Large {
    private static Dictionary<string, int> cache = new Dictionary<string, int>();

    public static void Initialize() {
        for (int i = 0; i < 1000; i++) {
            cache[$"key_{i}"] = i;
        }
    }

    public static int Process(string input) {
        if (cache.TryGetValue(input, out int value)) {
            return value;
        }
        return -1;
    }

    public static List<int> FilterEven(List<int> numbers) {
        return numbers.Where(n => n % 2 == 0).ToList();
    }

    public static double CalculateStatistics(List<int> numbers) {
        if (numbers.Count == 0) return 0.0;
        return numbers.Average();
    }
}
"#;

    group.bench_function("small_script", |b| {
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(small_script, "small"));
            let _ = host.clear_cache();
        });
    });

    group.bench_function("medium_script", |b| {
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(medium_script, "medium"));
            let _ = host.clear_cache();
        });
    });

    group.bench_function("large_script", |b| {
        b.iter(|| {
            let _ = black_box(host.compile_and_execute(large_script, "large"));
            let _ = host.clear_cache();
        });
    });

    group.finish();
}

#[cfg(feature = "csharp")]
fn bench_cache_statistics(c: &mut Criterion) {
    let host = match DotNetCliHost::initialize() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("⚠️  Skipping benchmark: .NET SDK not available ({})", e);
            return;
        }
    };

    c.bench_function("cache_warming", |b| {
        b.iter(|| {
            // 模拟缓存预热过程
            let scripts = vec![
                ("script1", HELLO_WORLD_SCRIPT),
                ("script2", CALCULATION_SCRIPT),
                ("script3", GAME_LOGIC_SCRIPT),
            ];

            for (name, script) in scripts.iter() {
                let _ = black_box(host.compile_and_execute(script, name));
            }

            // 输出缓存统计
            if let Some(stats) = host.get_cache_stats() {
                let hit_rate = host.get_cache_hit_rate();
                println!("Cache Stats - Hits: {}, Misses: {}, Hit Rate: {:.2}%",
                    stats.hits, stats.misses, hit_rate * 100.0);
            }
        });
    });
}

#[cfg(feature = "csharp")]
criterion_group!(
    name = csharp_benchmarks;
    config = Criterion::default().sample_size(10).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_hello_world, bench_calculation, bench_cache_hit_rate, bench_script_sizes, bench_cache_statistics
);

#[cfg(feature = "csharp")]
criterion_main!(csharp_benchmarks);

// 非 csharp feature 的空实现
#[cfg(not(feature = "csharp"))]
fn main() {
    println!("⚠️  C# benchmarks require 'csharp' feature to be enabled.");
    println!("Run with: cargo bench --features csharp --bench csharp_performance");
}
