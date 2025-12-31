# Debug UI 模块 - 添加到主README

## 在主README中添加以下内容

### 在"Core Modules"部分添加：

```markdown
- **Debugging**: Interactive debug UI with real-time performance monitoring
```

### 在"Advanced Features"部分扩展：

```markdown
- **Debugging**: Scene editor, property inspector, performance monitoring, and interactive debug UI
```

### 在"Examples"部分添加：

```markdown
## Debug UI

### Basic Usage

```rust
use game_engine::debug::DebugUI;

let mut debug_ui = DebugUI::new();

// In game loop
debug_ui.render(&egui_ctx, &world);
debug_ui.log("Frame completed".to_string());
```

### Performance Monitoring

```rust
debug_ui
    .performance_panel()
    .update_metrics(frame_time, frame_count);
```

For more examples, see:
- [Debug UI Example](../examples/debug_ui_example.rs)
- [Debug UI Integration](../examples/debug_ui_integrated.rs)
- [Debug UI Documentation](./P1-3-debug-ui-implementation.md)
```

### 在"Features"部分添加（如果有）：

```markdown
### Debug UI (P1-3) ✅

Interactive debug panels for engine monitoring and debugging:

- **Entity Panel** - View all ECS entities and their components
- **Component Panel** - Inspect component details
- **Performance Panel** - Real-time FPS, frame time, and memory monitoring
- **Console Panel** - Multi-level logging with filtering
- **Resource Panel** - Track resource loading status

**Key Features:**
- Real-time performance charts
- Colored log levels (Info, Warning, Error, Debug)
- Entity search and filtering
- Export console logs
- Configurable panel visibility

**Status:** ✅ Completed (2025-12-31)
**Documentation:** [P1-3 Debug UI Implementation](./P1-3-debug-ui-implementation.md)
```

## 快速开始

### 1. 创建DebugUI

```rust
let mut debug_ui = DebugUI::new();
```

### 2. 在游戏循环中使用

```rust
loop {
    // Update engine
    engine.update().await?;

    // Update performance
    debug_ui
        .performance_panel()
        .update_metrics(delta_time.as_secs_f32(), frame_count);

    // Render debug UI
    debug_ui.render(&egui_ctx, &world);
}
```

### 3. 添加日志

```rust
debug_ui.log("Game started".to_string());
debug_ui.log_error("Failed to load texture".to_string());
```

## 面板说明

### 实体面板
- 显示所有ECS实体
- 双击查看组件
- 搜索和过滤

### 组件面板
- 显示组件详情
- 支持数据可视化
- 实时刷新

### 性能面板
- FPS监控
- 帧时间统计
- 内存使用曲线
- Draw Calls统计

### 控制台面板
- 多级别日志
- 彩色显示
- 搜索过滤
- 日志导出

### 资源面板
- 资源加载状态
- 内存占用
- 加载进度

## 集成到egui-wgpu

```rust
// Initialize
let egui_ctx = egui::Context::default();
let egui_wgpu_state = egui_wgpu::State::new(&device, &queue, &config);
let mut debug_ui = DebugUI::new();

// Game loop
egui_wgpu::input::process_input(&egui_ctx, &event, &window);
debug_ui.render(&egui_ctx, &world);

// Render
let egui_output = egui_ctx.end_frame();
let paint_jobs = egui_ctx.tessellate(egui_output.shapes);
// ... render code
```

## 示例

运行示例程序：

```bash
cargo run --example debug_ui_example
cargo run --example debug_ui_integrated
```

## 文档

- [实现文档](./P1-3-debug-ui-implementation.md)
- [快速开始](./P1-3-quick-start.md)
- [完成报告](./P1-3-completion-report.md)

## 依赖

所有依赖已存在于项目中：

```toml
egui = "0.33.3"
egui-wgpu = "0.33.3"
egui-winit = "0.33.3"
```

---

**注意**: 这是添加到主README的内容建议。实际使用时请根据主README的格式和风格进行调整。
