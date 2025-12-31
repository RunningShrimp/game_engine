//  热重载系统性能基准测试
//
//  测试脚本热重载系统的性能，对比Mutex vs DashMap实现

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine::services::script_hot_reload::{
    FunctionChange, FunctionChangeType, HotReloadConfig, ReloadRecovery, ScriptHotReloadManager,
    ScriptType,
};
use std::path::PathBuf;
use std::time::Duration;

/// 创建测试脚本内容
fn create_test_script_content(function_count: usize) -> String {
    let mut content = String::new();
    for i in 0..function_count {
        content.push_str(&format!(
            r#"
function test_function_{}() {{
    console.log("Test function {}");
    return {};
}}
"#,
            i, i, i
        ));
    }
    content
}

/// 创建临时脚本文件
fn create_temp_script(content: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("bench_script_{}.js", fastrand::u64(..)));
    std::fs::write(&file_path, content).unwrap();
    file_path
}

/// 基准测试：创建热重载管理器
fn bench_hot_reload_manager_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_reload_manager_creation");

    group.bench_function("create_manager_default", |b| {
        b.iter(|| black_box(ScriptHotReloadManager::new(Default::default())));
    });

    group.bench_function("create_manager_with_config", |b| {
        let config = HotReloadConfig {
            enabled: true,
            check_interval_ms: 100,
            preserve_state: true,
            show_notifications: false,
            watched_extensions: vec!["js".to_string()],
            enable_incremental_reload: true,
            max_backups: 10,
        };
        b.iter(|| black_box(ScriptHotReloadManager::new(config.clone())));
    });

    group.finish();
}

/// 基准测试：添加脚本监控
fn bench_watch_script(c: &mut Criterion) {
    let mut group = c.benchmark_group("watch_script");

    for script_size in [10, 50, 100, 500].iter() {
        let content = create_test_script_content(*script_size);
        let file_path = create_temp_script(&content);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("functions_{}", script_size)),
            script_size,
            |b, _| {
                let manager = ScriptHotReloadManager::new(Default::default());

                b.iter(|| {
                    black_box(
                        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap(),
                    )
                });
            },
        );

        // 清理
        std::fs::remove_file(&file_path).ok();
    }

    group.finish();
}

/// 基准测试：检查并重载脚本
fn bench_check_and_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_and_reload");

    for script_count in [1, 10, 50, 100].iter() {
        let content = create_test_script_content(10);
        let mut file_paths = Vec::new();

        // 创建多个脚本文件
        for i in 0..*script_count {
            let file_path = create_temp_script(&content);
            file_paths.push(file_path);
        }

        let manager = ScriptHotReloadManager::new(Default::default());

        // 添加所有脚本到监控
        for file_path in &file_paths {
            manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("scripts_{}", script_count)),
            script_count,
            |b, _| {
                b.iter(|| black_box(manager.check_and_reload()));
            },
        );

        // 清理
        for file_path in &file_paths {
            std::fs::remove_file(file_path).ok();
        }
    }

    group.finish();
}

/// 基准测试：并发访问 - DashMap优势场景
fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");

    // 测试多线程读取性能
    for thread_count in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("read_threads_{}", thread_count)),
            thread_count,
            |b, &threads| {
                let content = create_test_script_content(50);
                let file_path = create_temp_script(&content);

                let manager = std::sync::Arc::new(ScriptHotReloadManager::new(Default::default()));
                manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();

                b.iter(|| {
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let manager = manager.clone();
                            std::thread::spawn(move || {
                                black_box(manager.get_watched_scripts());
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });

                std::fs::remove_file(&file_path).ok();
            },
        );
    }

    group.finish();
}

/// 基准测试：增量重载
fn bench_incremental_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_reload");

    // 小规模修改
    group.bench_function("small_change", |b| {
        let old_content = create_test_script_content(10);
        let mut new_content = old_content.clone();
        new_content.push_str("\nfunction new_function() { return 42; }");

        let file_path = create_temp_script(&old_content);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = std::sync::Arc::new(ScriptHotReloadManager::new(HotReloadConfig {
            enable_incremental_reload: true,
            ..Default::default()
        }));

        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();

        // 更新文件内容
        std::fs::write(&file_path, new_content).unwrap();

        b.to_async(&rt).iter(|| async {
            let manager = manager.clone();
            let path = file_path.clone();
            black_box(manager.reload_incremental(&path).await)
        });

        std::fs::remove_file(&file_path).ok();
    });

    // 大规模修改
    group.bench_function("large_change", |b| {
        let old_content = create_test_script_content(100);
        let new_content = create_test_script_content(100); // 全部修改

        let file_path = create_temp_script(&old_content);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = std::sync::Arc::new(ScriptHotReloadManager::new(HotReloadConfig {
            enable_incremental_reload: true,
            ..Default::default()
        }));

        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();

        // 更新文件内容
        std::fs::write(&file_path, new_content).unwrap();

        b.to_async(&rt).iter(|| async {
            let manager = manager.clone();
            let path = file_path.clone();
            black_box(manager.reload_incremental(&path).await)
        });

        std::fs::remove_file(&file_path).ok();
    });

    group.finish();
}

/// 基准测试：备份和恢复
fn bench_backup_and_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("backup_and_recovery");

    let content = create_test_script_content(100);
    let file_path = create_temp_script(&content);

    group.bench_function("backup_script", |b| {
        let recovery = ReloadRecovery::new(10);
        b.iter(|| black_box(recovery.backup_script(&file_path, &content)));
    });

    group.bench_function("rollback", |b| {
        let recovery = ReloadRecovery::new(10);
        recovery.backup_script(&file_path, &content);

        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt)
            .iter(|| async { black_box(recovery.rollback_on_failure(&file_path).await) });
    });

    std::fs::remove_file(&file_path).ok();
    group.finish();
}

/// 基准测试：函数提取和分析
fn bench_function_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_analysis");

    for function_count in [10, 50, 100, 500].iter() {
        let content = create_test_script_content(*function_count);
        let file_path = create_temp_script(&content);

        let manager = ScriptHotReloadManager::new(Default::default());

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("functions_{}", function_count)),
            &content,
            |b, content| {
                b.iter(|| black_box(manager.extract_functions(content, &file_path).unwrap()));
            },
        );

        std::fs::remove_file(&file_path).ok();
    }

    group.finish();
}

/// 基准测试：获取监控脚本列表
fn bench_get_watched_scripts(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_watched_scripts");

    for script_count in [10, 50, 100, 500].iter() {
        let manager = ScriptHotReloadManager::new(Default::default());

        // 添加多个脚本
        for i in 0..*script_count {
            let content = create_test_script_content(10);
            let file_path = create_temp_script(&content);
            manager.watch_script(file_path, ScriptType::JavaScript).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("scripts_{}", script_count)),
            script_count,
            |b, _| {
                b.iter(|| black_box(manager.get_watched_scripts()));
            },
        );
    }

    group.finish();
}

/// 基准测试：哈希计算
fn bench_hash_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_calculation");

    for content_size in [100, 1000, 10000, 100000].iter() {
        let content = "x".repeat(*content_size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("bytes_{}", content_size)),
            &content,
            |b, content| {
                b.iter(|| {
                    black_box(game_engine::services::script_hot_reload::ScriptHotReloadManager::calculate_hash(content))
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：完整热重载流程
fn bench_full_reload_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_reload_workflow");

    group.bench_function("small_script_reload", |b| {
        let content = create_test_script_content(10);
        let file_path = create_temp_script(&content);

        let manager = ScriptHotReloadManager::new(Default::default());
        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();

        // 注册回调
        manager.register_reload_callback(|_path, _content| Ok(()));

        b.iter(|| {
            // 模拟文件修改
            std::thread::sleep(Duration::from_millis(1));
            let modified_content = create_test_script_content(10);
            std::fs::write(&file_path, modified_content).unwrap();

            black_box(manager.check_and_reload())
        });

        std::fs::remove_file(&file_path).ok();
    });

    group.bench_function("large_script_reload", |b| {
        let content = create_test_script_content(100);
        let file_path = create_temp_script(&content);

        let manager = ScriptHotReloadManager::new(Default::default());
        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();

        // 注册回调
        manager.register_reload_callback(|_path, _content| Ok(()));

        b.iter(|| {
            // 模拟文件修改
            std::thread::sleep(Duration::from_millis(1));
            let modified_content = create_test_script_content(100);
            std::fs::write(&file_path, modified_content).unwrap();

            black_box(manager.check_and_reload())
        });

        std::fs::remove_file(&file_path).ok();
    });

    group.finish();
}

/// 基准测试：性能对比 - Mutex vs DashMap
#[cfg(feature = "hot-reload-optim")]
fn bench_mutex_vs_dashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutex_vs_dashmap_comparison");

    let content = create_test_script_content(50);
    let file_paths: Vec<_> = (0..10).map(|_| create_temp_script(&content)).collect();

    let manager = ScriptHotReloadManager::new(Default::default());

    // 添加所有脚本到监控
    for file_path in &file_paths {
        manager.watch_script(file_path.clone(), ScriptType::JavaScript).unwrap();
    }

    group.bench_function("dashmap_version", |b| {
        b.iter(|| black_box(manager.get_watched_scripts()));
    });

    // 清理
    for file_path in &file_paths {
        std::fs::remove_file(file_path).ok();
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_hot_reload_manager_creation,
    bench_watch_script,
    bench_check_and_reload,
    bench_concurrent_access,
    bench_incremental_reload,
    bench_backup_and_recovery,
    bench_function_analysis,
    bench_get_watched_scripts,
    bench_hash_calculation,
    bench_full_reload_workflow,
    #[cfg(feature = "hot-reload-optim")]
    bench_mutex_vs_dashmap,
);

criterion_main!(benches);
