# 核心模块 API 参考

## Engine

引擎主类，负责初始化和运行游戏循环。

### 示例

```rust
use game_engine::core::Engine;

let mut engine = Engine::new();
engine.initialize()?;

// 游戏主循环
loop {
    engine.update()?;
}
```

### 主要方法

- `new()` - 创建新引擎实例
- `initialize()` - 初始化引擎
- `update()` - 更新一帧
- `shutdown()` - 关闭引擎

## Scheduler

系统调度器，管理 ECS 系统的执行顺序。

### 示例

```rust
use game_engine::core::Scheduler;

let mut scheduler = Scheduler::new();
scheduler.add_system(update_transform_system);
scheduler.add_system(render_system);
scheduler.run(&mut world);
```

## Resources

引擎资源管理。

### 主要资源

- `Time` - 时间资源
- `RenderStats` - 渲染统计
- `AssetMetrics` - 资源指标

