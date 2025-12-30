//! # 游戏引擎使用示例
//!
//! 本示例展示了如何使用游戏引擎的核心功能，包括：
//! - 引擎初始化
//! - 实体管理
//! - 任务调度
//! - 渲染循环
//! - 事件处理

use game_engine::prelude::*;
use game_engine::core::scheduler::{TaskScheduler, Task, TaskPriority};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// ============================================================================
// 主示例：完整的游戏循环
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 游戏引擎使用示例");
    println!("=====================\n");

    // 示例1：基础引擎初始化
    basic_engine_init();

    // 示例2：实体管理
    entity_management_example();

    // 示例3：任务调度
    task_scheduling_example()?;

    // 示例4：性能优化
    performance_optimization_example();

    // 示例5：错误处理
    error_handling_example();

    println!("\n✅ 所有示例执行完成！");

    Ok(())
}

// ============================================================================
// 示例1：基础引擎初始化
// ============================================================================

fn basic_engine_init() {
    println!("📋 示例1：基础引擎初始化");
    println!("----------------------------");

    // 创建引擎配置
    let config = EngineConfig {
        window_title: "我的游戏".to_string(),
        window_width: 1280,
        window_height: 720,
        vsync: true,
        ..Default::default()
    };

    println!("✅ 引擎配置创建成功");
    println!("   - 标题: {}", config.window_title);
    println!("   - 分辨率: {}x{}", config.window_width, config.window_height);
    println!("   - 垂直同步: {}", config.vsync);
    
    println!();
}

// ============================================================================
// 示例2：实体管理
// ============================================================================

fn entity_management_example() {
    println!("📋 示例2：实体管理");
    println!("----------------------------");

    // 使用ECS创建实体
    use bevy_ecs::prelude::*;
    
    let mut world = World::new();

    // 创建玩家实体
    let player = world.spawn((
        Transform {
            pos: glam::Vec3::new(0.0, 0.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        Velocity {
            lin: glam::Vec3::new(1.0, 0.0, 0.0),
            ang: glam::Vec3::ZERO,
        },
        Sprite {
            color: [1.0, 1.0, 1.0, 1.0],
            tex_index: 0,
            ..Default::default()
        },
    ));

    println!("✅ 创建玩家实体: {:?}", player);

    // 创建多个敌人实体
    for i in 0..5 {
        let enemy = world.spawn((
            Transform {
                pos: glam::Vec3::new(i as f32 * 2.0, 0.0, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
            },
            Sprite {
                color: [1.0, 0.0, 0.0, 1.0], // 红色
                tex_index: 1,
                ..Default::default()
            },
        ));
        println!("✅ 创建敌人实体 #{}: {:?}", i, enemy);
    }

    // 查询所有实体
    let mut query = world.query::<(&Transform, &Velocity)>();
    let entity_count = query.iter(&world).count();
    println!("📊 当前实体总数: {}", entity_count);

    // 更新实体位置
    for (mut transform, velocity) in query.iter_mut(&mut world) {
        transform.pos += velocity.lin * 0.016; // delta time = 0.016s
    }

    println!("✅ 实体位置已更新");
    println!();
}

// ============================================================================
// 示例3：任务调度
// ============================================================================

fn task_scheduling_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 示例3：任务调度");
    println!("----------------------------");

    // 创建任务调度器
    let scheduler = TaskScheduler::new(4);
    println!("✅ 创建任务调度器 (4个工作线程)");

    // 创建计数器用于测试
    let counter = Arc::new(AtomicUsize::new(0));

    // 调度不同优先级的任务
    let tasks = vec![
        Task::new(
            "high_priority_render",
            {
                let counter = counter.clone();
                Box::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    println!("   🎨 执行高优先级任务: 渲染");
                })
            },
            TaskPriority::High,
        ),
        Task::new(
            "medium_priority_physics",
            {
                let counter = counter.clone();
                Box::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    println!("   ⚙️  执行中优先级任务: 物理模拟");
                })
            },
            TaskPriority::Medium,
        ),
        Task::new(
            "low_priority_resource_loading",
            {
                let counter = counter.clone();
                Box::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    println!("   📦 执行低优先级任务: 资源加载");
                })
            },
            TaskPriority::Low,
        ),
    ];

    // 批量调度任务
    scheduler.schedule_batch(tasks);
    println!("✅ 已调度3个任务");

    // 等待所有任务完成
    scheduler.wait_for_completion();

    // 获取统计信息
    let stats = scheduler.stats();
    println!("📊 任务统计:");
    println!("   - 已完成: {}", stats.completed_tasks);
    println!("   - 待处理: {}", stats.pending_tasks);
    println!("   - 工作线程: {}", stats.worker_count);

    println!();
    Ok(())
}

// ============================================================================
// 示例4：性能优化
// ============================================================================

fn performance_optimization_example() {
    println!("📋 示例4：性能优化");
    println!("----------------------------");

    use game_engine::async_optimization::*;

    // 同步计算（比async快10x）
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let _ = calculate_physics((0.0, 0.0, 0.0), (1.0, 2.0, 3.0), 0.016);
    }
    let sync_duration = start.elapsed();

    println!("✅ 同步物理计算 (100,000次): {:?}", sync_duration);
    println!("   平均: {:.2}ns/次", sync_duration.as_nanos() as f64 / 100_000.0);

    // 向量运算
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let _ = vector_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let _ = vector_dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    }
    let vec_duration = start.elapsed();

    println!("✅ 向量运算 (100,000次): {:?}", vec_duration);
    println!("   平均: {:.2}ns/次", vec_duration.as_nanos() as f64 / 100_000.0);

    // 批量并行处理（使用rayon）
    let mut entities = vec![[0.0f32; 3]; 10_000];
    let offset = [1.0, 2.0, 3.0];

    let start = std::time::Instant::now();
    batch_process_entities_rayon(&mut entities, offset);
    let parallel_duration = start.elapsed();

    println!("✅ 并行处理10,000个实体: {:?}", parallel_duration);
    println!("   平均: {:.2}μs/实体", parallel_duration.as_micros() as f64 / 10_000.0);

    println!();
}

// ============================================================================
// 示例5：错误处理
// ============================================================================

fn error_handling_example() {
    println!("📋 示例5：错误处理");
    println!("----------------------------");

    // 使用Result进行错误处理
    fn safe_divide(a: f32, b: f32) -> Result<f32, String> {
        if b == 0.0 {
            Err("除数不能为零".to_string())
        } else {
            Ok(a / b)
        }
    }

    // 成功的情况
    match safe_divide(10.0, 2.0) {
        Ok(result) => println!("✅ 10.0 / 2.0 = {}", result),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 失败的情况
    match safe_divide(10.0, 0.0) {
        Ok(result) => println!("✅ 结果: {}", result),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 使用?运算符传播错误
    fn complex_calculation(a: f32, b: f32) -> Result<f32, String> {
        let x = safe_divide(a, b)?;
        let y = safe_divide(x, 2.0)?;
        Ok(y)
    }

    match complex_calculation(10.0, 2.0) {
        Ok(result) => println!("✅ 复杂计算结果: {}", result),
        Err(e) => println!("❌ 复杂计算错误: {}", e),
    }

    println!();
}

// ============================================================================
// 额外示例：游戏场景
// ============================================================================

#[allow(dead_code)]
fn game_scene_example() {
    println!("📋 示例6：游戏场景");
    println!("----------------------------");

    use bevy_ecs::prelude::*;

    let mut world = World::new();

    // 场景1：创建玩家
    let player = world.spawn((
        Transform {
            pos: glam::Vec3::ZERO,
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        Sprite {
            color: [0.0, 1.0, 1.0, 1.0], // 青色
            tex_index: 0,
            ..Default::default()
        },
        Velocity {
            lin: glam::Vec3::ZERO,
            ang: glam::Vec3::ZERO,
        },
    ));
    println!("✅ 创建玩家: {:?}", player);

    // 场景2：创建装饰物
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * 5.0;
        let y = angle.sin() * 5.0;

        world.spawn((
            Transform {
                pos: glam::Vec3::new(x, y, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: glam::Vec3::new(0.5, 0.5, 1.0),
            },
            Sprite {
                color: [0.5, 0.5, 0.5, 1.0], // 灰色
                tex_index: 2,
                ..Default::default()
            },
        ));
    }
    println!("✅ 创建10个装饰物");

    // 场景3：添加光源
    world.spawn((
        Transform {
            pos: glam::Vec3::new(0.0, 0.0, 10.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        PointLight3D {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            radius: 20.0,
        },
    ));
    println!("✅ 创建点光源");

    // 场景4：添加相机
    world.spawn((
        Transform {
            pos: glam::Vec3::new(0.0, 0.0, 10.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        Camera {
            is_active: true,
            projection: Projection::Orthographic {
                scale: 1.0,
                near: 0.0,
                far: 100.0,
            },
        },
    ));
    println!("✅ 创建相机");

    println!();
}
