# Debug UI 快速使用指南

## 1. 创建DebugUI实例

```rust
use game_engine::debug::DebugUI;

// 使用默认配置
let mut debug_ui = DebugUI::new();

// 或使用自定义配置
use game_engine::debug::DebugConfig;

let config = DebugConfig {
    enabled: true,
    show_entities: true,
    show_performance: true,
    show_console: true,
    performance_history_size: 600,
    console_max_lines: 2000,
    ..Default::default()
};

let mut debug_ui = DebugUI::with_config(config);
```

## 2. 在游戏循环中使用

```rust
// 游戏主循环
loop {
    // 1. 更新引擎
    engine.update().await?;

    // 2. 获取帧时间
    let frame_time = delta_time.as_secs_f32();

    // 3. 更新性能面板
    debug_ui
        .performance_panel()
        .update_metrics(frame_time, frame_count);

    // 4. 添加日志（可选）
    if frame_count % 60 == 0 {
        debug_ui.log(format!("Running frame {}", frame_count));
    }

    // 5. 渲染DebugUI
    debug_ui.render(&egui_ctx, &world);

    frame_count += 1;
}
```

## 3. 与egui-wgpu集成

### 初始化阶段

```rust
// 创建egui上下文
let egui_ctx = egui::Context::default();

// 创建egui-wgpu状态
let egui_wgpu_state = egui_wgpu::State::new(&device, &queue, &surface_config);
```

### 渲染循环

```rust
// 1. 处理输入
for event in &window_events {
    egui_wgpu::input::process_input(
        &egui_ctx,
        event,
        &window,
    );
}

// 2. 更新UI
debug_ui.render(&egui_ctx, &world);

// 3. 渲染UI
let egui_output = egui_ctx.end_frame();
let screen_descriptor = egui_wgpu::ScreenDescriptor {
    size_in_pixels: [surface_config.width, surface_config.height],
    pixels_per_point: window.scale_factor() as f32,
};

let paint_jobs = egui_ctx.tessellate(egui_output.shapes);

for job in paint_jobs {
    egui_wgpu_state.paint(
        &device,
        &queue,
        &mut encoder,
        &job,
        &screen_descriptor,
    );
}
```

## 4. 常用功能

### 添加日志

```rust
// 普通日志
debug_ui.log("Game started".to_string());

// 错误日志
debug_ui.log_error("Failed to load asset".to_string());

// 通过控制台面板（更详细）
debug_ui
    .console_panel()
    .add_debug("Debug info".to_string());

debug_ui
    .console_panel()
    .add_warning("Low memory".to_string());
```

### 更新性能指标

```rust
// 更新FPS和帧时间
debug_ui
    .performance_panel()
    .update_metrics(frame_time, frame_count);

// 更新Draw Calls
debug_ui
    .performance_panel()
    .update_draw_calls(draw_calls, triangle_count);
```

### 更新资源统计

```rust
use game_engine::debug::panels::ResourceStats;

let stats = ResourceStats {
    resource_type: "Texture".to_string(),
    total_count: 100,
    loaded_count: 85,
    failed_count: 2,
    total_size: 50 * 1024 * 1024, // 50MB
    loading_count: 13,
};

debug_ui
    .resource_panel()
    .update_stats("Texture".to_string(), stats);
```

### 切换面板显示

```rust
// 切换特定面板
debug_ui.toggle_panel("entities");
debug_ui.toggle_panel("performance");
debug_ui.toggle_panel("console");

// 或者直接修改配置
let config = DebugConfig {
    show_entities: true,
    show_components: false,
    show_performance: true,
    show_console: true,
    show_resources: false,
    ..Default::default()
};

let debug_ui = DebugUI::with_config(config);
```

## 5. 面板说明

### 实体面板
- 显示所有ECS实体
- 双击实体查看组件详情
- 支持搜索过滤

### 组件面板
- 显示选中实体的所有组件
- 展示组件详细信息
- 支持刷新

### 性能面板
- 实时FPS显示
- 帧时间、Draw Calls统计
- 内存使用监控
- 历史曲线图表
- 统计信息（平均、最小、最大）

### 控制台面板
- 多级别日志（Info、Warning、Error、Debug）
- 彩色显示
- 搜索过滤
- 日志导出

### 资源面板
- 资源加载状态
- 内存占用统计
- 加载进度显示

## 6. 键盘快捷键

目前DebugUI使用鼠标交互，可以自行添加键盘快捷键：

```rust
// 在输入处理中添加
if let Some(key_event) = event.as_keyboard_input() {
    if key_event.state == winit::event::ElementState::Pressed {
        match key_event.key_code {
            winit::event::VirtualKeyCode::F1 => {
                debug_ui.toggle_panel("entities");
            }
            winit::event::VirtualKeyCode::F2 => {
                debug_ui.toggle_panel("performance");
            }
            winit::event::VirtualKeyCode::F3 => {
                debug_ui.toggle_panel("console");
            }
            _ => {}
        }
    }
}
```

## 7. 最佳实践

### 性能优化
- 限制历史记录大小
- 定期清理旧日志
- 使用搜索过滤减少显示内容

### 日志管理
- 合理使用日志级别
- 定期导出重要日志
- 避免在每帧都输出日志

### 配置建议
```rust
// 开发配置
DebugConfig {
    enabled: true,
    show_entities: true,
    show_performance: true,
    show_console: true,
    performance_history_size: 600,
    console_max_lines: 2000,
    ..Default::default()
}

// 发布配置（禁用调试UI）
DebugConfig {
    enabled: false,
    ..Default::default()
}
```

## 8. 故障排除

### UI不显示
- 检查`enabled`是否为true
- 确认egui上下文正确传递
- 检查渲染顺序

### 性能下降
- 减少`performance_history_size`
- 减少`console_max_lines`
- 禁用不需要的面板

### 实体列表为空
- 确认World参数正确传递
- 检查实体是否已创建
- 尝试刷新面板

## 9. 示例代码

完整示例请参考：
- `examples/debug_ui_example.rs` - 基础示例
- `examples/debug_ui_integrated.rs` - 完整集成示例

运行示例：
```bash
cargo run --example debug_ui_example
cargo run --example debug_ui_integrated
```

## 10. 进一步阅读

- 详细实现文档：`docs/P1-3-debug-ui-implementation.md`
- API文档：运行`cargo doc --open`查看
- egui文档：https://docs.rs/egui/
