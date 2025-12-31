# P1-3 交互式调试UI实现文档

## 任务概述

完成了基于egui的交互式调试面板，用于实时监控和调试游戏引擎状态。

## 实现日期

2025-12-31

## 目录结构

```
game_engine/src/debug/
├── mod.rs              # 模块定义和公共接口
├── ui.rs               # DebugUI核心结构
├── panels/
│   ├── mod.rs          # 面板模块定义
│   ├── entity_panel.rs    # 实体面板
│   ├── component_panel.rs # 组件面板
│   ├── performance_panel.rs # 性能面板
│   ├── console_panel.rs    # 控制台面板
│   └── resource_panel.rs   # 资源面板
└── visualizer.rs       # 可视化工具

game_engine/examples/
└── debug_ui_example.rs # 调试UI示例程序
```

## 核心功能

### 1. DebugUI核心结构

**文件**: `src/debug/ui.rs`

**功能**:
- 管理所有调试面板的显示和交互
- 提供菜单栏导航
- 统一的面板可见性控制
- 帧时间追踪和性能更新

**主要API**:
```rust
pub struct DebugUI {
    // 配置
    config: DebugConfig,

    // 各个面板
    entity_panel: EntityPanel,
    component_panel: ComponentPanel,
    performance_panel: PerformancePanel,
    console_panel: ConsolePanel,
    resource_panel: ResourcePanel,
}

impl DebugUI {
    pub fn new() -> Self;
    pub fn with_config(config: DebugConfig) -> Self;
    pub fn render(&mut self, ctx: &egui::Context, world: &World);
    pub fn log(&mut self, message: String);
    pub fn log_error(&mut self, error: String);
    pub fn toggle_panel(&mut self, panel: &str);
}
```

### 2. 实体面板 (EntityPanel)

**文件**: `src/debug/panels/entity_panel.rs`

**功能**:
- 显示所有ECS实体列表
- 显示实体的组件信息
- 支持搜索过滤
- 双击实体查看组件详情
- 显示实体存活状态

**特性**:
- 实时刷新实体列表
- 缓存机制减少查询开销
- 自动更新实体状态

### 3. 组件面板 (ComponentPanel)

**文件**: `src/debug/panels/component_panel.rs`

**功能**:
- 显示选中实体的所有组件
- 展示组件的详细信息
- 支持组件数据的序列化显示
- 提供刷新功能

**可扩展性**:
- 实现了`DebugInspectable` trait
- 支持自定义组件的可视化

### 4. 性能面板 (PerformancePanel)

**文件**: `src/debug/panels/performance_panel.rs`

**功能**:
- 实时FPS显示
- 帧时间监控（毫秒）
- Draw Calls统计
- 三角形数量统计
- 内存使用监控
- CPU/GPU使用率（预留接口）

**可视化图表**:
- FPS历史曲线
- 帧时间历史曲线
- 内存使用曲线
- 所有图表支持缩放和滚动

**统计信息**:
- 平均FPS
- 最小/最大FPS
- 平均帧时间

### 5. 控制台面板 (ConsolePanel)

**文件**: `src/debug/panels/console_panel.rs`

**功能**:
- 多级别日志显示（Info、Warning、Error、Debug）
- 日志搜索和过滤
- 彩色显示不同级别日志
- 自动滚动功能
- 日志导出功能

**日志级别**:
- Info - 灰色
- Warning - 黄色
- Error - 红色
- Debug - 浅蓝色

**特性**:
- 最大行数限制（可配置）
- 时间戳显示
- 来源标识
- 支持日志集成（可选）

### 6. 资源面板 (ResourcePanel)

**文件**: `src/debug/panels/resource_panel.rs`

**功能**:
- 显示资源加载状态
- 统计资源数量（总数、已加载、失败、加载中）
- 显示资源内存占用
- 资源类型分类
- 加载进度条

**资源类型**:
- Texture
- Mesh
- Shader
- Audio
- Font
- Model
- Animation
- Script
- Config

### 7. 可视化工具 (Visualizer)

**文件**: `src/debug/visualizer.rs`

**提供的可视化器**:

1. **PerformanceVisualizer** - 通用性能数据可视化
   - 自动缩放
   - 颜色配置
   - 统计信息（平均值、最小值、最大值）

2. **MemoryVisualizer** - 内存使用可视化
   - 总内存
   - 堆内存
   - GPU内存
   - 多曲线对比显示

3. **FPSVisualizer** - FPS专用可视化
   - 目标FPS参考线（60 FPS）
   - 颜色编码（绿色>60, 黄色>30, 红色<30）
   - 实时FPS显示

## 使用示例

### 基本使用

```rust
use game_engine::debug::DebugUI;

// 创建调试UI
let mut debug_ui = DebugUI::new();

// 在渲染循环中
loop {
    // 更新引擎
    engine.update().await?;

    // 渲染调试UI
    debug_ui.render(&egui_ctx, &world);

    // 添加日志
    debug_ui.log("Frame completed".to_string());
}
```

### 自定义配置

```rust
use game_engine::debug::{DebugUI, DebugConfig};

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

### 性能监控

```rust
// 更新性能指标
let frame_time = delta_time.as_secs_f32();
debug_ui
    .performance_panel()
    .update_metrics(frame_time, frame_count);

// 更新Draw Calls
debug_ui
    .performance_panel()
    .update_draw_calls(draw_calls, triangle_count);
```

### 日志输出

```rust
// 普通日志
debug_ui.log("Game started".to_string());

// 错误日志
debug_ui.log_error("Failed to load texture".to_string());

// 通过控制台面板直接添加
debug_ui
    .console_panel()
    .add_warning("Low memory".to_string());

debug_ui
    .console_panel()
    .add_debug("Debug info".to_string());
```

### 资源监控

```rust
use game_engine::debug::panels::{ResourcePanel, ResourceStats};

let stats = ResourceStats {
    resource_type: "Texture".to_string(),
    total_count: 100,
    loaded_count: 85,
    failed_count: 2,
    total_size: 1024 * 1024 * 50, // 50MB
    loading_count: 13,
};

debug_ui
    .resource_panel()
    .update_stats("Texture".to_string(), stats);
```

## 集成到渲染系统

### 与egui-wgpu集成

```rust
// 在渲染器初始化时
let egui_ctx = egui::Context::default();
let egui_wgpu_state = egui_wgpu::State::new(&device, &queue, &surface_config);

// 在渲染循环中
// 1. 输入处理
egui_wgpu::input::process_input(
    &egui_ctx,
    &window_event,
    &window,
);

// 2. UI更新
debug_ui.render(&egui_ctx, &world);

// 3. 渲染
let egui_output = egui_ctx.end_frame();
let paint_jobs = egue_ctx.tessellate(egui_output.shapes);

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

## 配置选项

### DebugConfig

```rust
pub struct DebugConfig {
    /// 是否启用调试UI
    pub enabled: bool,

    /// 默认面板可见性
    pub show_entities: bool,
    pub show_components: bool,
    pub show_performance: bool,
    pub show_console: bool,
    pub show_resources: bool,

    /// 性能历史记录长度
    pub performance_history_size: usize,

    /// 控制台日志最大行数
    pub console_max_lines: usize,
}
```

## 性能考虑

1. **实体缓存**: EntityPanel使用缓存机制，只在需要时刷新实体列表
2. **历史限制**: 所有历史记录都有最大长度限制，防止内存无限增长
3. **按需刷新**: 组件详情只在选中实体时加载
4. **过滤优化**: 日志过滤在显示时进行，不影响存储

## 限制和注意事项

1. **bevy_ecs限制**:
   - 无法直接通过ComponentId获取组件数据
   - 组件类型名称获取有限
   - 需要实际的World引用

2. **平台依赖**:
   - 内存使用获取当前仅在macOS上实现
   - CPU/GPU使用率需要平台特定实现

3. **性能开销**:
   - 实体列表遍历有O(n)复杂度
   - 大量实体时可能影响性能

## 扩展方向

1. **更多面板**:
   - 网络监控面板
   - 物理调试面板
   - 音频调试面板

2. **增强可视化**:
   - 3D场景树视图
   - 组件关系图
   - 内存热点分析

3. **交互功能**:
   - 实时修改组件值
   - 实体创建/删除
   - 资源重载

4. **集成改进**:
   - 与tracing集成
   - 与profiler集成
   - 远程调试支持

## 测试

示例程序位于 `examples/debug_ui_example.rs`，包含：

- DebugUI基本使用
- 实体面板测试
- 性能监控测试
- 控制台日志测试
- 资源监控测试

运行测试：
```bash
cargo test --example debug_ui_example
```

## 依赖

已添加到 `Cargo.toml`:
```toml
egui = "0.33.3"
egui-wgpu = "0.33.3"
egui-winit = "0.33.3"
```

这些依赖已经在项目中存在，无需额外添加。

## 完成状态

✅ 所有要求已完成：

1. ✅ 创建了完整的debug模块
2. ✅ 实现了DebugUI核心结构
3. ✅ 实现了5个调试面板（实体、组件、性能、控制台、资源）
4. ✅ 实现了性能指标可视化（图表）
5. ✅ 可以集成到渲染系统（与egui兼容）
6. ✅ 创建了示例程序
7. ✅ 在lib.rs中导出了debug模块

## 参考资料

- [egui文档](https://docs.rs/egui/)
- [egui-wgpu文档](https://docs.rs/egui-wgpu/)
- [bevy_ecs文档](https://docs.rs/bevy_ecs/)
