# 故障排除指南

本文档提供游戏引擎常见问题的诊断和解决方案。

## 目录

1. [快速诊断](#快速诊断)
2. [编译问题](#编译问题)
3. [运行时错误](#运行时错误)
4. [性能问题](#性能问题)
5. [渲染问题](#渲染问题)
6. [物理问题](#物理问题)
7. [资源加载问题](#资源加载问题)
8. [网络问题](#网络问题)
9. [平台特定问题](#平台特定问题)
10. [调试技巧](#调试技巧)

---

## 快速诊断

### 问题分类

根据错误类型快速定位问题：

| 错误类型 | 可能原因 | 查看章节 |
|---------|---------|---------|
| 编译错误 | 依赖缺失、特性标志、平台兼容性 | [编译问题](#编译问题) |
| 运行时崩溃 | 空指针、内存溢出、未初始化 | [运行时错误](#运行时错误) |
| 性能下降 | 协程阻塞、SIMD未启用、GPU未利用 | [性能问题](#性能问题) |
| 渲染异常 | 着色器错误、纹理格式、GPU驱动 | [渲染问题](#渲染问题) |
| 物理异常 | 刚体配置、碰撞检测、时间步长 | [物理问题](#物理问题) |
| 资源加载失败 | 文件路径、格式不支持、依赖缺失 | [资源加载问题](#资源加载问题) |
| 网络连接失败 | 端口占用、防火墙、协议不匹配 | [网络问题](#网络问题) |

### 诊断工具

**使用性能监控工具**:

```rust
use game_engine_performance::monitoring::SystemPerformanceMonitor;

let mut monitor = SystemPerformanceMonitor::new();
monitor.start()?;

let metrics = monitor.get_metrics();
println!("Frame time: {:.2}ms", metrics.frame_time);
println!("CPU usage: {:.2}%", metrics.cpu_usage);
println!("Memory usage: {}MB", metrics.memory_usage_mb);
```

**使用性能仪表盘**:

```rust
use game_engine::profiling::dashboard::PerformanceDashboard;

let mut dashboard = PerformanceDashboard::new("127.0.0.1:8080")?;
dashboard.start()?;

// 访问 http://127.0.0.1:8080 查看实时性能指标
```

**启用详细日志**:

```rust
// 设置日志级别
std::env::set_var("RUST_LOG", "game_engine=debug");

// 或使用tracing
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

---

## 编译问题

### 问题：找不到模块

**错误信息**:
```
error[E0583]: file not found for module `xxx`
```

**可能原因**:
1. 文件路径不正确
2. `mod.rs`中未声明模块
3. 条件编译导致模块未包含

**解决方案**:

```rust
// 检查mod.rs
// game_engine/src/render/mod.rs
pub mod xxx; // 确保模块已声明

// 检查文件是否存在
// game_engine/src/render/xxx.rs

// 检查条件编译
#[cfg(feature = "xxx")]
pub mod xxx;
```

### 问题：特性标志未启用

**错误信息**:
```
error[E0432]: unresolved import `game_engine::xxx`
```

**可能原因**:
- 需要的特性标志未在`Cargo.toml`中启用

**解决方案**:

```toml
# Cargo.toml
[dependencies]
game_engine = { path = "../game_engine", features = ["gltf", "wasm"] }
```

### 问题：平台特定代码编译失败

**错误信息**:
```
error[E0432]: unresolved import `std::os::unix`
```

**可能原因**:
- 使用了平台特定的API但未添加条件编译

**解决方案**:

```rust
// 使用平台检测函数
use game_engine::platform::detection::{is_windows, is_macos, is_linux};

if is_windows() {
    // Windows特定代码
} else if is_macos() {
    // macOS特定代码
} else if is_linux() {
    // Linux特定代码
}
```

### 问题：依赖版本冲突

**错误信息**:
```
error: failed to select a version for `xxx`
```

**解决方案**:

```bash
# 更新依赖
cargo update

# 检查依赖树
cargo tree

# 清理并重新构建
cargo clean
cargo build
```

---

## 运行时错误

### 问题：空指针/空引用

**错误信息**:
```
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'
```

**可能原因**:
- 资源未初始化
- 实体已被销毁
- 组件未添加

**解决方案**:

```rust
// ❌ 避免：直接unwrap
let texture = manager.get_texture(&handle).unwrap();

// ✅ 好的做法：使用match或?
match manager.get_texture(&handle) {
    Some(texture) => {
        // 使用纹理
    }
    None => {
        tracing::warn!("Texture not found: {:?}", handle);
        // 使用默认纹理或返回错误
    }
}

// 或使用?
let texture = manager.get_texture(&handle)?;
```

### 问题：内存溢出

**错误信息**:
```
thread 'main' panicked at 'memory allocation of X bytes failed'
```

**可能原因**:
- 资源未释放
- 对象池未使用
- 大量实体未清理

**解决方案**:

```rust
// 使用对象池
use game_engine::performance::memory::ObjectPool;

let mut pool = ObjectPool::new(100, || Particle::new());

// 及时释放资源
{
    let resource = load_resource()?;
    // 使用资源...
} // 资源在这里自动释放

// 清理不需要的实体
world.clear(); // 或
for entity in entities_to_remove {
    world.despawn(entity);
}
```

### 问题：未初始化的资源

**错误信息**:
```
error: Resource not initialized
```

**解决方案**:

```rust
// 检查资源是否存在
if let Some(resource) = world.get_resource::<MyResource>() {
    // 使用资源
} else {
    // 初始化资源
    world.insert_resource(MyResource::new());
}
```

### 问题：线程安全问题

**错误信息**:
```
error[E0382]: borrow of moved value
```

**可能原因**:
- 在异步上下文中移动了非`Send`类型
- 多个线程同时访问可变数据

**解决方案**:

```rust
// 使用Arc<Mutex<T>>共享可变数据
use std::sync::{Arc, Mutex};

let shared_data = Arc::new(Mutex::new(MyData::new()));

let data_clone = shared_data.clone();
tokio::spawn(async move {
    let mut data = data_clone.lock().unwrap();
    // 修改数据
});

// 使用Arc<RwLock<T>>用于读多写少
use std::sync::RwLock;

let shared_data = Arc::new(RwLock::new(MyData::new()));
```

---

## 性能问题

### 问题：帧率下降

**症状**:
- 帧时间超过16.67ms（60 FPS）
- 游戏卡顿

**诊断步骤**:

1. **检查性能指标**:

```rust
use game_engine_performance::monitoring::SystemPerformanceMonitor;

let metrics = monitor.get_metrics();
if metrics.frame_time > 16.67 {
    println!("Frame time: {:.2}ms (target: 16.67ms)", metrics.frame_time);
    println!("CPU usage: {:.2}%", metrics.cpu_usage);
    println!("GPU usage: {:.2}%", metrics.gpu_usage);
}
```

2. **使用性能分析器**:

```rust
use game_engine::profiling::profiler::Profiler;

let mut profiler = Profiler::new();
profiler.start_frame();

// 游戏逻辑...

profiler.end_frame();
let report = profiler.get_report();

// 找出耗时最长的系统
for (scope, time) in report.scope_times {
    if time > 5.0 {
        println!("Slow system: {} ({:.2}ms)", scope, time);
    }
}
```

**常见原因和解决方案**:

1. **协程阻塞**:

```rust
// ❌ 避免：在异步上下文中阻塞
async fn bad_example() {
    std::thread::sleep(Duration::from_millis(100)); // 阻塞！
}

// ✅ 好的做法：使用tokio::time::sleep
async fn good_example() {
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// ✅ CPU密集型任务使用spawn_blocking
tokio::task::spawn_blocking(move || {
    heavy_computation()
}).await?;
```

2. **SIMD未启用**:

```rust
// 检查SIMD支持
use game_engine::platform::detection::WasmSimdSupport;

let simd = WasmSimdSupport::detect();
if !simd.is_available() {
    println!("SIMD not available, performance may be reduced");
}

// 使用批量处理以利用SIMD
for chunk in data.chunks(4) {
    let simd = Vec3Simd::from_slice(chunk);
    // SIMD操作
}
```

3. **GPU未充分利用**:

```rust
// 启用GPU驱动渲染
let config = GpuDrivenConfig {
    frustum_culling: true,
    occlusion_culling: true,
    lod_enabled: true,
    max_instances: 65536,
    ..Default::default()
};
```

### 问题：内存使用过高

**症状**:
- 内存持续增长
- 系统变慢

**诊断**:

```rust
use game_engine::resources::memory_monitor::MemoryMonitor;

let monitor = MemoryMonitor::new();
let stats = monitor.get_stats();

println!("Total memory: {}MB", stats.total_mb);
println!("Texture memory: {}MB", stats.texture_mb);
println!("Buffer memory: {}MB", stats.buffer_mb);
```

**解决方案**:

1. **使用对象池**:

```rust
let mut pool = ObjectPool::new(100, || Particle::new());
```

2. **及时释放资源**:

```rust
// 使用作用域确保资源释放
{
    let texture = load_texture("temp.png")?;
    // 使用纹理...
} // 纹理在这里释放
```

3. **使用资源缓存限制**:

```rust
let mut manager = UnifiedResourceManager::new(device, queue);
manager.set_cache_limit(1024 * 1024 * 500); // 500MB限制
```

---

## 渲染问题

### 问题：黑屏/无渲染

**可能原因**:
1. GPU未初始化
2. 着色器编译失败
3. 渲染目标未设置

**诊断**:

```rust
// 检查GPU初始化
if let Err(e) = render_backend.initialize() {
    println!("GPU initialization failed: {}", e);
    return Err(e);
}

// 检查着色器编译
match shader.compile() {
    Ok(_) => println!("Shader compiled successfully"),
    Err(e) => println!("Shader compilation failed: {}", e),
}

// 检查渲染目标
if render_target.is_none() {
    println!("Render target not set");
}
```

**解决方案**:

```rust
// 确保正确初始化
let mut backend = RenderBackend::new(&instance, window)?;
backend.initialize()?;

// 检查着色器错误
let shader = Shader::new(device, shader_code)?;

// 设置渲染目标
backend.set_render_target(Some(render_target));
```

### 问题：纹理显示错误

**可能原因**:
1. 纹理格式不支持
2. 纹理尺寸过大
3. 纹理未正确上传到GPU

**诊断**:

```rust
// 检查纹理格式
let format = texture.format();
println!("Texture format: {:?}", format);

// 检查纹理尺寸
let size = texture.size();
println!("Texture size: {}x{}", size.width, size.height);

// 检查GPU限制
let limits = device.limits();
println!("Max texture size: {}", limits.max_texture_dimension_2d);
```

**解决方案**:

```rust
// 使用支持的格式
let format = TextureFormat::Rgba8UnormSrgb;

// 检查尺寸限制
if size.width > limits.max_texture_dimension_2d {
    // 缩放纹理
    let scaled = scale_texture(texture, max_size)?;
}

// 确保纹理已上传
texture.upload_to_gpu(device, queue)?;
```

### 问题：着色器编译错误

**错误信息**:
```
Shader compilation failed: ...
```

**诊断**:

```rust
// 启用详细着色器日志
std::env::set_var("WGPU_SHADER_DEBUG", "1");

// 检查WGSL语法
let shader = Shader::compile(device, shader_code)?;
```

**常见错误**:

1. **语法错误**:
```wgsl
// ❌ 错误
var position: vec3<f32> = vec3(1.0, 2.0); // 参数数量错误

// ✅ 正确
var position: vec3<f32> = vec3<f32>(1.0, 2.0, 3.0);
```

2. **类型不匹配**:
```wgsl
// ❌ 错误
var value: f32 = vec3(1.0, 2.0, 3.0);

// ✅ 正确
var value: vec3<f32> = vec3<f32>(1.0, 2.0, 3.0);
```

### 问题：后处理效果不工作

**可能原因**:
1. 效果未启用
2. 输入纹理未设置
3. 效果链顺序错误

**解决方案**:

```rust
// 确保效果已启用
manager.add_effect(PostProcessEffect::Bloom {
    intensity: 0.8,
    threshold: 1.0,
    radius: 5.0,
});

// 设置输入纹理
manager.render(
    &mut encoder,
    device,
    queue,
    &scene_view,        // 场景纹理
    Some(&depth_view),  // 深度纹理
    Some(&motion_view), // 运动向量纹理
    &output_view,       // 输出纹理
);
```

---

## 物理问题

### 问题：刚体穿透

**可能原因**:
1. 时间步长过大
2. 碰撞体配置错误
3. CCD（连续碰撞检测）未启用

**解决方案**:

```rust
// 使用较小的时间步长
physics_world.step(0.016)?; // 60 FPS

// 启用CCD
let config = PhysicsConfig {
    ccd_enabled: true,
    ccd_max_penetration: 0.01,
    ..Default::default()
};

// 检查碰撞体配置
let collider = Collider::ball(radius)
    .with_active_events(ActiveEvents::COLLISION_EVENTS)
    .with_ccd_enabled(true);
```

### 问题：物理模拟不稳定

**可能原因**:
1. 时间步长不一致
2. 刚体质量配置错误
3. 约束配置错误

**解决方案**:

```rust
// 使用固定时间步长
let fixed_dt = 0.016; // 60 FPS
physics_world.step(fixed_dt)?;

// 检查质量配置
let body = RigidBody::new(
    RigidBodyType::Dynamic,
    Vec3::ZERO,
    Quat::IDENTITY,
)
.with_mass(1.0) // 设置合理的质量
.with_linear_damping(0.1) // 添加阻尼
.with_angular_damping(0.1);

// 检查约束
let joint = RevoluteJoint::new()
    .with_local_axis1(Vec3::Z)
    .with_limits([-PI, PI]);
```

### 问题：物理性能下降

**症状**:
- 大量刚体时性能下降
- 物理步进耗时过长

**解决方案**:

```rust
// 使用并行物理世界
use game_engine::physics::parallel::ParallelPhysicsWorld;

let mut parallel_physics = ParallelPhysicsWorld::new();

// 异步物理步进
physics_world.step_async(0.016).await?;

// 使用空间分区优化
let config = PhysicsConfig {
    spatial_partition: SpatialPartition::Grid {
        cell_size: 10.0,
    },
    ..Default::default()
};
```

---

## 资源加载问题

### 问题：资源加载失败

**错误信息**:
```
ResourceError::NotFound { path: "..." }
```

**诊断**:

```rust
// 检查文件是否存在
use std::path::Path;

let path = Path::new("assets/texture.png");
if !path.exists() {
    println!("File not found: {:?}", path);
    // 检查相对路径和绝对路径
    println!("Current dir: {:?}", std::env::current_dir()?);
}
```

**解决方案**:

```rust
// 使用绝对路径
let absolute_path = std::env::current_dir()?
    .join("assets")
    .join("texture.png");

// 或使用资源管理器
let handle = manager.load_texture("texture.png").await?;
```

### 问题：GLTF加载失败

**可能原因**:
1. GLTF特性未启用
2. 文件格式不支持
3. 依赖资源缺失

**解决方案**:

```toml
# Cargo.toml
[dependencies]
game_engine = { path = "../game_engine", features = ["gltf"] }
```

```rust
// 检查文件格式
let extension = path.extension()
    .and_then(|e| e.to_str())
    .unwrap_or("");

if extension != "gltf" && extension != "glb" {
    return Err(GltfLoadError::InvalidFormat);
}

// 检查依赖资源
let scene = loader.load_scene("model.gltf").await?;
for texture_path in scene.texture_paths {
    if !Path::new(&texture_path).exists() {
        println!("Missing texture: {}", texture_path);
    }
}
```

### 问题：热重载不工作

**可能原因**:
1. 文件监听未启动
2. 事件处理未调用
3. 依赖图未更新

**解决方案**:

```rust
// 确保热重载管理器已启动
let mut manager = HotReloadManager::new("assets", dependency_graph)?;

// 监视资源
manager.watch_resource("assets/texture.png".into())?;

// 定期处理事件
let events = manager.process_events_batch(100, Duration::from_millis(100)).await;

// 更新依赖图
for event in events {
    match event {
        HotReloadEvent::ResourceModified(path) => {
            let targets = manager.get_reload_targets(&path);
            // 重新加载依赖资源
        }
        _ => {}
    }
}
```

---

## 网络问题

### 问题：连接失败

**错误信息**:
```
NetworkError::ConnectionFailed
```

**诊断**:

```rust
// 检查端口是否被占用
use std::net::TcpListener;

let port = 8080;
match TcpListener::bind(format!("127.0.0.1:{}", port)) {
    Ok(_) => println!("Port {} is available", port),
    Err(e) => println!("Port {} is in use: {}", port, e),
}
```

**解决方案**:

```rust
// 使用不同的端口
let config = NetworkConfig {
    port: 8081,
    ..Default::default()
};

// 检查防火墙设置
// Windows: 检查Windows防火墙
// Linux: 检查iptables
// macOS: 检查系统偏好设置
```

### 问题：消息丢失

**可能原因**:
1. 缓冲区溢出
2. 网络延迟
3. 消息处理速度慢

**解决方案**:

```rust
// 增加缓冲区大小
let (tx, rx) = tokio::sync::mpsc::channel(1000); // 增加缓冲区

// 使用批量处理
let processor = ParallelMessageProcessor::new(32);
let results = processor.process_messages_async(messages, state, None).await;

// 添加消息确认机制
connection.send_with_ack(message).await?;
```

### 问题：同步延迟

**症状**:
- 网络实体位置不同步
- 延迟较高

**解决方案**:

```rust
// 启用插值
let config = SyncConfig {
    interpolation_enabled: true,
    interpolation_delay: Duration::from_millis(100),
    ..Default::default()
};

// 启用预测
let config = SyncConfig {
    prediction_enabled: true,
    ..Default::default()
};

// 使用延迟补偿
let compensator = DelayCompensator::new();
let compensated_state = compensator.compensate(state, latency)?;
```

---

## 平台特定问题

### WASM平台

**问题：SIMD未启用**

**解决方案**:

```bash
# 启用SIMD
cargo build --target wasm32-unknown-unknown --features simd

# 或使用wasm-pack
wasm-pack build --target web --features simd
```

**问题：内存限制**

**解决方案**:

```rust
// 使用内存池
use game_engine::platform::wasm_performance::WasmMemoryPool;

let config = WasmMemoryPoolConfig {
    initial_size: 1024 * 1024, // 1MB
    max_size: 10 * 1024 * 1024, // 10MB
    ..Default::default()
};

let mut pool = WasmMemoryPool::new(config);
```

### 移动平台

**问题：性能下降**

**解决方案**:

```rust
// 使用移动平台优化配置
use game_engine::platform::mobile::MobileConfig;

let config = MobileConfig {
    target_fps: 30, // 降低目标帧率
    reduce_quality: true,
    ..Default::default()
};
```

**问题：触摸输入不响应**

**解决方案**:

```rust
// 检查触摸事件处理
use game_engine::platform::mobile::TouchInput;

let touch_input = TouchInput::new();
touch_input.handle_touch_event(event)?;
```

---

## 调试技巧

### 启用详细日志

```rust
// 设置日志级别
std::env::set_var("RUST_LOG", "game_engine=debug");

// 或使用tracing
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::TRACE)
    .with_target(false)
    .init();
```

### 使用性能分析器

```rust
use game_engine::profiling::profiler::Profiler;

let mut profiler = Profiler::new();

profiler.start_frame();
profiler.begin_scope("update");
// 更新逻辑
profiler.end_scope("update");

profiler.end_frame();
let report = profiler.get_report();
```

### 使用断点调试

```rust
// 使用dbg!宏
let value = dbg!(compute_value());

// 使用println!调试
println!("Entity position: {:?}", entity.position);

// 使用tracing
tracing::debug!("Entity position: {:?}", entity.position);
```

### 检查内存泄漏

```rust
use game_engine::resources::memory_monitor::MemoryMonitor;

let monitor = MemoryMonitor::new();

// 定期检查内存使用
let stats = monitor.get_stats();
if stats.total_mb > 1000 {
    println!("Memory usage high: {}MB", stats.total_mb);
}
```

---

## 获取帮助

### 文档资源

- [API参考](api_reference.md)
- [最佳实践](best_practices.md)
- [性能调优指南](performance_tuning_guide.md)
- [架构文档](architecture.md)

### 报告问题

如果问题仍未解决，请提供以下信息：

1. **错误信息**: 完整的错误消息
2. **代码示例**: 最小可复现的代码
3. **环境信息**: 
   - 操作系统和版本
   - Rust版本 (`rustc --version`)
   - 引擎版本
4. **日志**: 相关的日志输出
5. **性能数据**: 如果涉及性能问题

---

**文档版本**: 1.0  
**创建日期**: 2025-12-23  
**维护者**: Game Engine Team

