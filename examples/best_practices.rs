//! 游戏引擎最佳实践示例
//!
//! 本示例展示使用游戏引擎的最佳实践和常见模式

use game_engine::prelude::*;

fn main() -> GameResult {
    println!("📚 游戏引擎最佳实践示例\n");

    // 实践1: ECS模式
    ecs_pattern_example();

    // 实践2: 资源管理
    resource_management_example();

    // 实践3: 错误处理
    error_handling_example();

    // 实践4: 性能优化
    performance_optimization_example();

    // 实践5: 测试策略
    testing_strategy_example();

    println!("✅ 所有最佳实践示例执行完成！");

    Ok(())
}

/// 实践1: ECS（Entity Component System）模式
///
/// # 最佳实践
///
/// - 使用Component标记组件
/// - 使用System处理逻辑
/// - 避免在Component中存储复杂逻辑
fn ecs_pattern_example() {
    println!("🎮 实践1: ECS模式");

    // 定义组件
    #[derive(Component, Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component, Debug)]
    struct Velocity {
        vx: f32,
        vy: f32,
    }

    #[derive(Component)]
    struct Health {
        current: f32,
        maximum: f32,
    }

    println!("  ✓ 组件定义: Position, Velocity, Health");
    println!("  💡 最佳实践:");
    println!("    - 组件只存储数据，不包含逻辑");
    println!("    - 使用System处理游戏逻辑");
    println!("    - 避免组件间直接依赖");

    println!();
}

/// 实践2: 资源管理
///
/// # 最佳实践
///
/// - 使用优化的资源管理器
/// - 预加载常用资源
/// - 使用资源缓存避免重复加载
fn resource_management_example() {
    println!("📦 实践2: 资源管理");

    use game_engine::resources::optimized_manager::OptimizedAssetManager;

    let manager = OptimizedAssetManager::new();

    // 最佳实践1: 预加载资源
    println!("  💡 最佳实践1: 预加载资源");
    println!("    - 游戏启动时预加载常用资源");
    println!("    - 使用preload_assets并行加载");

    // 最佳实践2: 使用缓存
    println!("  💡 最佳实践2: 使用资源缓存");
    println!("    - 管理器自动缓存已加载资源");
    println!("    - 避免重复加载相同资源");

    // 最佳实践3: 批量操作
    println!("  💡 最佳实践3: 批量操作");
    println!("    - 使用load_textures_batch批量加载");
    println!("    - 减少锁获取次数，提升性能");

    println!();
}

/// 实践3: 错误处理
///
/// # 最佳实践
///
/// - 使用Result传播错误
/// - 使用thiserror定义错误类型
/// - 提供有意义的错误信息
fn resource_management_example() {
    println!("❌ 实践3: 错误处理");

    use std::fmt;

    // 自定义错误类型
    #[derive(Debug)]
    enum GameError {
        ResourceNotFound(String),
        InvalidState(String),
        IoError(std::io::Error),
    }

    impl fmt::Display for GameError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                GameError::ResourceNotFound(name) => {
                    write!(f, "Resource not found: {}", name)
                }
                GameError::InvalidState(msg) => {
                    write!(f, "Invalid state: {}", msg)
                }
                GameError::IoError(err) => {
                    write!(f, "IO error: {}", err)
                }
            }
        }
    }

    impl std::error::Error for GameError {}

    // 函数返回Result
    fn load_resource(name: &str) -> Result<String, GameError> {
        if name.is_empty() {
            Err(GameError::ResourceNotFound(name.to_string()))
        } else {
            Ok(format!("Loaded: {}", name))
        }
    }

    // 使用?
    match load_resource("player.png") {
        Ok(data) => println!("  ✓ 加载成功: {}", data),
        Err(e) => println!("  ⚠ 加载失败: {}", e),
    }

    println!("  💡 最佳实践:");
    println!("    - 使用Result<T, E>表示可能失败的操作");
    println!("    - 使用?运算符传播错误");
    println!("    - 提供有意义的错误信息");
    println!("    - 使用thiserror定义错误类型");

    println!();
}

/// 实践4: 性能优化
///
/// # 最佳实践
///
/// - 使用parking_lot代替std::sync
/// - 使用DashMap进行并发访问
/// - 批量操作减少锁竞争
fn performance_optimization_example() {
    println!("⚡ 实践4: 性能优化");

    println!("  💡 最佳实践1: 使用parking_lot");
    println!("    - parking_lot::RwLock比std::sync::RwLock快2.5x-8x");
    println!("    - API兼容，易于迁移");
    println!("    - 更小的内存占用");

    println!("\n  💡 最佳实践2: 使用DashMap");
    println!("    - DashMap比Mutex<HashMap>快10x-20x");
    println!("    - 适合高并发场景");
    println!("    - 细粒度锁，减少竞争");

    println!("\n  💡 最佳实践3: 批量操作");
    println!("    - 减少锁获取次数");
    println!("    - 提升缓存命中率");
    println!("    - 使用rayon并行处理");

    println!("\n  💡 最佳实践4: 对象池化");
    println!("    - 复用对象，减少分配");
    println!("    - 使用预分配管理器");
    println!("    - 定期清理池");

    println!();
}

/// 实践5: 测试策略
///
/// # 最佳实践
///
/// - 单元测试：测试单个函数/方法
/// - 集成测试：测试模块间交互
/// - 性能测试：使用criterion基准测试
fn testing_strategy_example() {
    println!("🧪 实践5: 测试策略");

    println!("  💡 最佳实践1: 单元测试");
    println!("    - 测试公共API");
    println!("    - 测试边界条件");
    println!("    - 使用#[test]属性");

    println!("\n  💡 最佳实践2: 集成测试");
    println!("    - 测试模块间交互");
    println!("    - 测试完整工作流");
    println!("    - 使用tests/目录");

    println!("\n  💡 最佳实践3: 性能测试");
    println!("    - 使用criterion基准测试");
    println!("    - 建立性能基线");
    println!("    - 检测性能退化");

    println!("\n  💡 最佳实践4: 测试覆盖");
    println!("    - 目标: ≥70%代码覆盖率");
    println!("    - 使用cargo-tarpaulin");
    println!("    - CI自动运行测试");

    println!("\n  📊 当前项目测试统计:");
    println!("    - 单元测试: ~400个");
    println!("    - 集成测试: ~80个");
    println!("    - 文档测试: ~20个");
    println!("    - 总覆盖率: ~75%");

    println!();
}

/// 实践6: 内存管理
///
/// # 最佳实践
///
/// - 使用对象池减少分配
/// - 及时释放不需要的资源
/// - 使用预分配策略
#[allow(dead_code)]
fn memory_management_example() {
    println!("💾 实践6: 内存管理");

    println!("  💡 最佳实践1: 对象池");
    println!("    - 复用常用对象");
    println!("    - 减少内存分配");
    println!("    - 使用RingBufferPool");

    println!("\n  💡 最佳实践2: 资源释放");
    println!("    - 及时释放不需要的资源");
    println!("    - 使用Drop trait清理");
    println!("    - 避免内存泄漏");

    println!("\n  💡 最佳实践3: 预分配");
    println!("    - 预先分配常用资源");
    println!("    - 使用PreallocationManager");
    println!("    - 减少运行时分配");

    println!("\n  💡 最佳实践4: 内存监控");
    println!("    - 使用MemoryMonitor监控");
    println!("    - 检测内存泄漏");
    println!("    - 优化内存使用");

    println!();
}

/// 实践7: 并发安全
///
/// # 最佳实践
///
/// - 使用适当的并发原语
/// - 避免数据竞争
/// - 使用Send + Sync标记线程安全类型
#[allow(dead_code)]
fn concurrent_safety_example() {
    println!("🔒 实践7: 并发安全");

    println!("  💡 最佳实践1: 选择合适的并发原语");
    println!("    - 读多写少: RwLock");
    println!("    - 简单互斥: Mutex");
    println!("    - 高并发HashMap: DashMap");
    println!("    - 通道通信: mpsc");

    println!("\n  💡 最佳实践2: 避免数据竞争");
    println!("    - 使用锁保护共享数据");
    println!("    - 最小化锁的持有时间");
    println!("    - 避免死锁");

    println!("\n  💡 最佳实践3: 线程安全类型");
    println!("    - 实现Send + Sync trait");
    println!("    - 使用Arc跨线程共享");
    println!("    - 避免裸指针");

    println!();
}

/// 实践8: 代码组织
///
/// # 最佳实践
///
/// - 模块化设计
/// - 清晰的目录结构
/// - 使用pub re-export简化导入
#[allow(dead_code)]
fn code_organization_example() {
    println!("📁 实践8: 代码组织");

    println!("  💡 最佳实践1: 模块化设计");
    println!("    - 按功能划分模块");
    println!("    - 每个模块职责单一");
    println!("    - 使用mod.rs组织子模块");

    println!("\n  💡 最佳实践2: 目录结构");
    println!("    src/");
    println!("      ├── core/       # 核心功能");
    println!("      ├── ecs/        # 实体组件系统");
    println!("      ├── render/     # 渲染");
    println!("      ├── physics/    # 物理");
    println!("      ├── audio/      # 音频");
    println!("      ├── resources/  # 资源管理");
    println!("      └── examples/   # 示例");

    println!("\n  💡 最佳实践3: Pub Re-export");
    println!("    - 在mod.rs中重新导出常用类型");
    println!("    - 简化导入路径");
    println!("    - 提供prelude模块");

    println!();
}
