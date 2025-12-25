# Tracy Profiler集成指南

## 概述

Tracy是一个高性能的实时性能分析工具，支持火焰图、GPU分析、内存追踪等功能。本指南介绍如何在游戏引擎中使用Tracy进行性能分析。

## 安装和配置

### 1. 添加依赖

Tracy已集成到游戏引擎中，通过特性标志启用：

```toml
[dependencies]
game_engine = { path = "../game_engine", features = ["tracy"] }
```

### 2. 安装Tracy Profiler应用程序

从 [Tracy GitHub](https://github.com/wolfpld/tracy) 下载并安装Tracy Profiler应用程序。

## 基本使用

### 作用域测量

使用`TracyScope`自动测量代码块的执行时间：

```rust
use game_engine::profiling::tracy::TracyScope;

{
    let _scope = TracyScope::new("my_function");
    // 你的代码
}
// 作用域结束时自动记录性能数据
```

### 便捷宏

使用便捷宏简化代码：

```rust
use game_engine::{tracy_scope, tracy_message, tracy_frame};

// 创建作用域
tracy_scope!("render_frame");

// 发送消息
tracy_message!("Important event");

// 帧标记
tracy_frame!();
```

### 带颜色的作用域

使用颜色区分不同类型的作用域：

```rust
use game_engine::profiling::tracy::TracyScope;

let _scope = TracyScope::with_color("render_pass", 0xFF0000); // 红色
```

## 高级功能

### GPU性能分析

```rust
use game_engine::profiling::tracy::{TracyGpuContext, TracyGpuSpan};

let gpu_context = TracyGpuContext::new("wgpu");
let gpu_span = gpu_context.begin_span("draw_call");
// GPU操作
drop(gpu_span); // 自动结束
gpu_context.collect(); // 收集GPU时间戳
```

### 内存分配追踪

```rust
use game_engine::profiling::tracy::TracyAllocation;

let ptr = Box::into_raw(Box::new([0u8; 1024]));
TracyAllocation::alloc(ptr, 1024);
// 使用内存
TracyAllocation::free(ptr);
```

### 帧标记

在游戏循环中标记帧边界：

```rust
use game_engine::profiling::tracy::TracyMessage;

loop {
    // 游戏逻辑
    TracyMessage::frame_mark();
    // 或使用命名帧
    TracyMessage::frame_mark_named("game_frame");
}
```

## 在游戏引擎中的集成

### 渲染系统

```rust
use game_engine::tracy_scope;

fn render_frame(renderer: &mut Renderer) {
    tracy_scope!("render_frame");
    
    {
        tracy_scope!("update_camera");
        renderer.update_camera();
    }
    
    {
        tracy_scope!("render_scene");
        renderer.render_scene();
    }
    
    tracy_frame!();
}
```

### 物理系统

```rust
use game_engine::tracy_scope;

fn update_physics(world: &mut PhysicsWorld) {
    tracy_scope!("physics_update");
    
    {
        tracy_scope!("collision_detection");
        world.detect_collisions();
    }
    
    {
        tracy_scope!("constraint_solving");
        world.solve_constraints();
    }
}
```

### 资源加载

```rust
use game_engine::tracy_scope;

async fn load_texture(path: &str) -> Result<Texture> {
    tracy_scope!("load_texture");
    tracy_message!(format!("Loading texture: {}", path));
    // 加载逻辑
}
```

## 查看分析结果

1. **启动Tracy Profiler应用程序**
2. **连接到应用程序**
   - 应用程序启动后会自动监听连接
   - 在Tracy中点击"Connect"连接到应用程序
3. **查看火焰图**
   - 在Tracy界面中查看实时火焰图
   - 分析函数调用栈和性能热点
4. **GPU分析**
   - 查看GPU时间线
   - 分析GPU利用率
5. **内存分析**
   - 查看内存分配模式
   - 检测内存泄漏

## 最佳实践

1. **合理使用作用域**
   - 不要过度使用，避免性能开销
   - 只在关键路径上使用

2. **命名规范**
   - 使用清晰、描述性的名称
   - 使用统一的命名约定

3. **帧标记**
   - 在游戏循环中标记每一帧
   - 使用命名帧标记区分不同类型的帧

4. **颜色编码**
   - 使用颜色区分不同类型的操作
   - 例如：红色=渲染，蓝色=物理，绿色=逻辑

5. **生产环境**
   - 在生产环境中禁用Tracy（不启用tracy特性）
   - Tracy只在开发和调试时使用

## 故障排除

### Tracy未连接

- 确保使用`--features tracy`编译
- 检查防火墙设置
- 确保Tracy Profiler应用程序正在运行

### 性能开销

- Tracy的开销很小，但在极高频的代码路径上可能影响性能
- 可以在关键路径上禁用Tracy

### 内存追踪不工作

- 确保正确调用`alloc`和`free`
- 检查指针有效性

## 更多资源

- [Tracy官方文档](https://github.com/wolfpld/tracy)
- [Tracy使用指南](https://github.com/wolfpld/tracy/wiki)
- [游戏引擎性能优化指南](../performance_tuning_guide.md)

