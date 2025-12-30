//! 并发编程示例
//!
//! 本示例展示游戏引擎中的并发编程模式和最佳实践

use game_engine::prelude::*;

fn main() -> GameResult {
    println!("🔄 游戏引擎并发编程示例\n");

    // 示例1: 使用DashMap进行并发实体管理
    dashmap_entity_management();

    // 示例2: 使用parking_lot进行线程安全访问
    parking_lot_usage();

    // 示例3: 并发事件系统
    concurrent_event_system();

    // 示例4: 并行资源加载
    parallel_resource_loading();

    println!("✅ 所有并发示例执行完成！");

    Ok(())
}

/// 示例1: 使用DashMap进行并发实体管理
///
/// # 场景
///
/// 游戏中需要管理大量实体（玩家、敌人、道具等），多个系统需要并发访问
fn dashmap_entity_management() {
    println!("🎮 示例1: DashMap并发实体管理");

    use game_engine::resources::dashmap_optimizations::{ConcurrentEntityManager, EntityData};
    use std::thread;
    use std::time::Duration;

    let manager = ConcurrentEntityManager::new();

    // 场景1: 物理系统更新实体位置（并发写入）
    let manager_physics = manager.clone();
    let physics_thread = thread::spawn(move || {
        for i in 0..10 {
            manager_physics.update_entity(i, |entity| {
                entity.position.0 += 1.0;
            });
        }
        println!("  ✓ 物理系统更新了10个实体");
    });

    // 场景2: 渲染系统读取实体数据（并发读取）
    let manager_render = manager.clone();
    let render_thread = thread::spawn(move || {
        for i in 0..10 {
            if let Some(entity) = manager_render.get_entity(i) {
                // 渲染实体（不修改）
                let _ = entity.position;
            }
        }
        println!("  ✓ 渲染系统读取了10个实体");
    });

    // 场景3: AI系统更新实体状态（并发写入）
    let manager_ai = manager.clone();
    let ai_thread = thread::spawn(move || {
        for i in 0..10 {
            manager_ai.update_entity(i, |entity| {
                entity.active = true;
            });
        }
        println!("  ✓ AI系统更新了10个实体");
    });

    // 等待所有线程完成
    physics_thread.join().unwrap();
    render_thread.join().unwrap();
    ai_thread.join().unwrap();

    println!("  📊 总实体数: {}", manager.len());
    println!();
}

/// 示例2: 使用parking_lot进行线程安全访问
///
/// # 场景
///
/// 游戏状态需要被多个系统并发访问，读多写少
fn parking_lot_usage() {
    println!("🔒 示例2: parking_lot线程安全访问");

    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::thread;

    // 游戏状态
    let game_state = Arc::new(RwLock::new(GameState {
        score: 0,
        level: 1,
        paused: false,
    }));

    // 创建多个读取线程
    let mut handles = vec![];

    for i in 0..5 {
        let state = game_state.clone();
        let handle = thread::spawn(move || {
            // 读锁：多个线程可以同时持有
            let state = state.read();
            println!("  线程{}: 读取分数={}, 等级={}", i, state.score, state.level);
            // 读锁自动释放
        });
        handles.push(handle);
    }

    // 创建一个写入线程
    let state = game_state.clone();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        // 写锁：独占访问
        let mut state = state.write();
        state.score += 100;
        state.level += 1;
        println!("  写入线程: 更新分数={}, 等级={}", state.score, state.level);
    });
    handles.push(handle);

    // 等待所有线程
    for handle in handles {
        handle.join().unwrap();
    }

    println!("  ✓ parking_lot提供高性能的读写锁");
    println!();
}

/// 示例3: 并发事件系统
///
/// # 场景
///
/// 游戏事件需要被多个系统监听和处理
fn concurrent_event_system() {
    println!("📡 示例3: 并发事件系统");

    use game_engine::resources::dashmap_optimizations::EventBus;

    let bus = EventBus::<GameEvent>::new();

    // 订阅1: 物理系统
    bus.subscribe("collision".to_string(), |event| {
        if let GameEvent::Collision { entity_a, entity_b } = event {
            println!("  ⚡ 物理系统: 碰撞 {} <-> {}", entity_a, entity_b);
        }
    });

    // 订阅2: 音频系统
    bus.subscribe("collision".to_string(), |event| {
        if let GameEvent::Collision { .. } = event {
            println!("  🔊 音频系统: 播放碰撞音效");
        }
    });

    // 订阅3: UI系统
    bus.subscribe("score".to_string(), |event| {
        if let GameEvent::Score { points } = event {
            println!("  🎨 UI系统: 更新分数显示 +{}", points);
        }
    });

    // 发布事件
    bus.publish("collision", &GameEvent::Collision {
        entity_a: 1,
        entity_b: 2,
    });

    bus.publish("score", &GameEvent::Score { points: 100 });

    println!("  ✓ 事件总线支持多个订阅者");
    println!();
}

/// 示例4: 并行资源加载
///
/// # 场景
///
/// 游戏启动时需要加载大量资源，使用并行加载加速
fn parallel_resource_loading() {
    println!("⚡ 示例4: 并行资源加载");

    use game_engine::resources::optimized_manager::OptimizedAssetManager;
    use rayon::prelude::*;

    let manager = OptimizedAssetManager::new();

    // 要加载的资源列表
    let resources = vec![
        "player.png",
        "enemy.png",
        "background.png",
        "bullet.png",
        "explosion.png",
    ];

    // 使用rayon并行加载
    println!("  开始并行加载 {} 个资源...", resources.len());

    let results: Vec<_> = resources.par_iter()
        .map(|name| {
            // 模拟加载延迟
            std::thread::sleep(std::time::Duration::from_millis(10));
            (name.clone(), manager.load_texture(name))
        })
        .collect();

    // 统计结果
    let success = results.iter().filter(|(_, r)| r.is_ok()).count();
    let failed = results.iter().filter(|(_, r)| r.is_err()).count();

    println!("  ✓ 并行加载完成: 成功={}, 失败={}", success, failed);
    println!("  💡 并行加载比串行快3-5倍（取决于资源数量）");
    println!();
}

/// 游戏状态
#[derive(Debug)]
struct GameState {
    score: u32,
    level: u32,
    paused: bool,
}

/// 游戏事件
#[derive(Debug, Clone)]
enum GameEvent {
    Collision { entity_a: u32, entity_b: u32 },
    Score { points: u32 },
    LevelUp { new_level: u32 },
}
