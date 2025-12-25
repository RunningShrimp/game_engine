# 最佳实践指南

本文档提供游戏引擎开发的最佳实践，涵盖架构设计、ECS使用、资源管理、性能优化等方面。

## 目录

1. [架构设计](#架构设计)
2. [ECS使用](#ecs使用)
3. [资源管理](#资源管理)
4. [性能优化](#性能优化)
5. [错误处理](#错误处理)
6. [并发和异步](#并发和异步)
7. [测试策略](#测试策略)
8. [代码组织](#代码组织)

---

## 架构设计

### 分层架构原则

引擎采用清晰的分层架构，遵循依赖倒置原则：

```
应用层 (Application)
    ↓
领域层 (Domain)
    ↓
服务层 (Services)
    ↓
基础设施层 (Infrastructure)
```

**最佳实践**:

1. **依赖方向**: 上层依赖下层，下层不依赖上层
2. **接口抽象**: 使用trait定义接口，实现放在基础设施层
3. **领域逻辑**: 业务逻辑放在领域层，技术细节放在基础设施层

**示例**:

```rust
// ✅ 好的做法：领域层定义接口
pub trait RenderService {
    fn render_scene(&self, scene: &Scene) -> Result<(), RenderError>;
}

// ✅ 好的做法：基础设施层实现
pub struct WgpuRenderService {
    // wgpu实现细节
}

impl RenderService for WgpuRenderService {
    fn render_scene(&self, scene: &Scene) -> Result<(), RenderError> {
        // 实现细节
    }
}

// ❌ 避免：领域层依赖具体实现
// use game_engine::render::wgpu::WgpuRenderer; // 不要这样做
```

### 领域驱动设计（DDD）

**聚合根（Aggregate Roots）**:

- 控制实体的生命周期
- 维护聚合不变量
- 通过ID引用其他聚合

```rust
// ✅ 好的做法：聚合根控制访问
pub struct GameEntity {
    id: EntityId,
    components: Vec<Component>,
}

impl GameEntity {
    pub fn add_component(&mut self, component: Component) -> Result<(), DomainError> {
        // 验证业务规则
        self.validate_component(&component)?;
        self.components.push(component);
        Ok(())
    }
    
    fn validate_component(&self, component: &Component) -> Result<(), DomainError> {
        // 业务规则验证
        Ok(())
    }
}
```

**值对象（Value Objects）**:

- 不可变
- 通过值比较相等性
- 封装验证逻辑

```rust
// ✅ 好的做法：值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        // 可以添加验证逻辑
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
```

### 微内核架构

引擎支持微内核架构，核心功能最小化，其他功能作为服务运行：

```rust
use game_engine::core::microkernel::{ServiceRegistry, Service};

// 注册服务
let mut registry = ServiceRegistry::new();
registry.register_service(Box::new(RenderService::new()))?;
registry.register_service(Box::new(AudioService::new()))?;

// 通过消息总线通信
let message = Message::Render { scene };
registry.send_message("render_service", message).await?;
```

**优势**:
- 模块化：每个服务独立开发和测试
- 可扩展性：可以动态加载/卸载服务
- 隔离性：服务崩溃不会影响整个系统

---

## ECS使用

### 组件设计

**组件应该是纯数据结构**:

```rust
// ✅ 好的做法：纯数据组件
#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// ❌ 避免：组件包含逻辑
// #[derive(Component)]
// pub struct Transform {
//     pub position: Vec3,
//     pub fn update(&mut self) { // 不要这样做
//         // 逻辑应该在系统中
//     }
// }
```

**组件应该小而专注**:

```rust
// ✅ 好的做法：小而专注的组件
#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Component)]
pub struct Rotation(pub Quat);

#[derive(Component)]
pub struct Scale(pub Vec3);

// ❌ 避免：巨大的组件
// #[derive(Component)]
// pub struct Transform {
//     pub position: Vec3,
//     pub rotation: Quat,
//     pub scale: Vec3,
//     pub velocity: Vec3,
//     pub acceleration: Vec3,
//     // ... 太多字段
// }
```

### 系统设计

**系统应该是无状态的**:

```rust
// ✅ 好的做法：无状态系统
fn movement_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.position += velocity.0 * time.delta_seconds();
    }
}

// ❌ 避免：有状态系统
// fn movement_system(
//     mut query: Query<(&mut Transform, &Velocity)>,
//     mut state: Local<MovementState>, // 避免使用Local
// ) {
//     // ...
// }
```

**系统应该专注于单一职责**:

```rust
// ✅ 好的做法：单一职责
fn physics_system(query: Query<(&mut Transform, &RigidBody)>) {
    // 只处理物理
}

fn render_system(query: Query<(&Transform, &Sprite)>) {
    // 只处理渲染
}

// ❌ 避免：多职责系统
// fn update_system(query: Query<...>) {
//     // 处理物理、渲染、AI等所有内容
// }
```

**使用系统调度优化性能**:

```rust
use game_engine::core::system_scheduler::SystemScheduler;

let mut scheduler = SystemScheduler::new();

// 添加系统（自动分析依赖）
scheduler.add_system(physics_system);
scheduler.add_system(movement_system);
scheduler.add_system(render_system);

// 并行执行（自动检测可并行的系统）
scheduler.run_parallel(&mut world);
```

### 查询优化

**使用精确的查询**:

```rust
// ✅ 好的做法：精确查询
fn update_system(
    mut query: Query<(&mut Transform, &Velocity), (With<Player>, Without<Enemy>)>,
) {
    // 只查询玩家实体，排除敌人
}

// ❌ 避免：过于宽泛的查询
// fn update_system(mut query: Query<&mut Transform>) {
//     // 查询所有实体，可能包含不需要的
// }
```

**使用变更检测**:

```rust
// ✅ 好的做法：只处理变更的组件
fn sync_system(
    mut query: Query<&mut Transform, Changed<Position>>,
) {
    // 只处理位置发生变化的实体
}

// ❌ 避免：每帧处理所有实体
// fn sync_system(mut query: Query<&mut Transform>) {
//     // 即使没有变化也处理
// }
```

---

## 资源管理

### 资源加载

**使用异步加载**:

```rust
// ✅ 好的做法：异步加载
use game_engine::resources::CoroutineLoader;

let loader = CoroutineLoader::new();

// 高优先级加载关键资源
let texture = loader.load_critical("player_texture.png").await?;

// 后台预加载
loader.preload("level2_texture.png", Priority::Low).await?;
```

**使用统一资源管理器**:

```rust
// ✅ 好的做法：统一管理
use game_engine::resources::UnifiedResourceManager;

let mut manager = UnifiedResourceManager::new(device, queue);

// 加载各种资源类型
let texture = manager.load_texture("texture.png").await?;
let model = manager.load_model("model.gltf").await?;
let audio = manager.load_audio("sound.mp3").await?;

// 自动缓存管理
let stats = manager.cache_stats();
if stats.misses > stats.hits {
    // 考虑增加缓存大小
}
```

### 资源生命周期

**及时释放不需要的资源**:

```rust
// ✅ 好的做法：及时释放
{
    let texture = manager.load_texture("temp_texture.png").await?;
    // 使用纹理...
} // texture在这里自动释放

// ❌ 避免：长期持有不需要的资源
// let texture = manager.load_texture("temp_texture.png").await?;
// // ... 很久之后才释放
```

**使用对象池重用资源**:

```rust
// ✅ 好的做法：使用对象池
use game_engine::performance::memory::ObjectPool;

let mut pool = ObjectPool::new(100, || Particle::new());

// 从池中获取
let particle = pool.acquire();

// 使用粒子...

// 返回到池中
pool.release(particle);
```

### 热重载

**使用协程批量处理热重载事件**:

```rust
// ✅ 好的做法：批量处理
use game_engine::resources::hot_reload::HotReloadManager;

let mut manager = HotReloadManager::new("assets", dependency_graph)?;

// 批量处理事件
let events = manager.process_events_batch(100, Duration::from_millis(100)).await;

// 并发重载
let results = manager.reload_resources_concurrent(paths, reload_fn).await;
```

---

## 性能优化

### 协程使用

**使用协程处理I/O密集型任务**:

```rust
// ✅ 好的做法：协程处理I/O
use game_engine::audio::streaming::AudioStreamLoader;

let mut loader = AudioStreamLoader::new();

// 异步加载音频流
let stream_id = loader.start_streaming_async("music.ogg", config).await?;

// 并发更新所有流
loader.update_all_async().await?;
```

**使用`spawn_blocking`处理CPU密集型任务**:

```rust
// ✅ 好的做法：CPU密集型任务使用spawn_blocking
use tokio::task::spawn_blocking;

let result = spawn_blocking(move || {
    // CPU密集型计算
    heavy_computation()
}).await?;
```

**使用`Semaphore`限制并发数**:

```rust
// ✅ 好的做法：限制并发
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // 最多10个并发

for task in tasks {
    let permit = semaphore.clone().acquire_owned().await?;
    tokio::spawn(async move {
        // 执行任务
        drop(permit); // 释放许可
    });
}
```

### SIMD优化

**批量处理数据以充分利用SIMD**:

```rust
// ✅ 好的做法：批量处理
use game_engine::physics::batch_sync::Vec3Simd;

let positions: Vec<Vec3> = /* ... */;

// 批量处理（SIMD优化）
for chunk in positions.chunks(4) {
    let simd = Vec3Simd::from_slice(chunk);
    // SIMD操作
}

// ❌ 避免：逐个处理
// for pos in positions {
//     // 无法利用SIMD
// }
```

**确保数据对齐**:

```rust
// ✅ 好的做法：数据对齐
#[repr(align(16))]
struct AlignedData {
    positions: [Vec3; 4],
}
```

### GPU加速

**识别适合GPU加速的任务**:

```rust
// ✅ 好的做法：大规模并行任务使用GPU
use game_engine::performance::gpu::gpu_compute::GpuComputeContext;

// 粒子系统（10万+粒子）
let context = GpuComputeContext::new(device, queue)?;
let mut particle_system = GpuParticleSystem::new(context, config)?;
particle_system.update(delta_time)?; // GPU加速

// ❌ 避免：少量计算使用GPU
// let result = gpu_compute_small_task(data); // CPU更快
```

**减少CPU-GPU数据传输**:

```rust
// ✅ 好的做法：批量传输
buffer.write_all(&large_data);

// ❌ 避免：频繁小数据传输
// for item in items {
//     buffer.write(&item); // 每次传输都有开销
// }
```

### 渲染优化

**使用GPU驱动渲染**:

```rust
// ✅ 好的做法：启用GPU驱动渲染
use game_engine::render::gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig};

let config = GpuDrivenConfig {
    frustum_culling: true,
    occlusion_culling: true,
    lod_enabled: true,
    max_instances: 65536,
    ..Default::default()
};

let mut renderer = GpuDrivenRenderer::new(device, &config)?;
```

**使用LOD系统**:

```rust
// ✅ 好的做法：根据距离选择LOD
use game_engine::render::lod::{LodSelector, LodConfig};

let config = LodConfig {
    lod_levels: vec![
        LodLevel::new(0, 0.0, 50.0, "high"),
        LodLevel::new(1, 50.0, 100.0, "medium"),
        LodLevel::new(2, 100.0, 200.0, "low"),
    ],
    ..Default::default()
};

let selector = LodSelector::new(config);
let lod_level = selector.select_lod(camera_pos, object_pos);
```

**批处理绘制调用**:

```rust
// ✅ 好的做法：合并相同材质的绘制调用
use game_engine::render::instance_batch::BatchManager;

let mut batch_manager = BatchManager::new(device);

// 添加实例到批次
batch_manager.add_instance(batch_key, transform, color)?;

// 渲染批次（减少draw call）
batch_manager.render(encoder, device, queue)?;
```

---

## 错误处理

### 错误类型设计

**使用thiserror定义错误类型**:

```rust
// ✅ 好的做法：清晰的错误类型
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}
```

**使用Result传播错误**:

```rust
// ✅ 好的做法：使用?操作符
fn load_resource(path: &Path) -> Result<Resource, ResourceError> {
    let data = std::fs::read(path)?; // 自动传播IO错误
    let resource = parse_resource(&data)?; // 自动传播解析错误
    Ok(resource)
}
```

### 错误恢复

**实现错误恢复策略**:

```rust
// ✅ 好的做法：错误恢复
use game_engine::error::recovery::{RecoveryStrategy, ErrorSeverity};

match load_resource(path) {
    Ok(resource) => resource,
    Err(e) => {
        match e.severity() {
            ErrorSeverity::Critical => {
                // 关键错误，无法恢复
                panic!("Critical error: {}", e);
            }
            ErrorSeverity::Recoverable => {
                // 可恢复错误，使用默认值
                Resource::default()
            }
            ErrorSeverity::Warning => {
                // 警告，记录但继续
                tracing::warn!("Warning: {}", e);
                Resource::default()
            }
        }
    }
}
```

---

## 并发和异步

### 协程任务管理

**使用CoroutineTaskManager管理异步任务**:

```rust
// ✅ 好的做法：使用任务管理器
use game_engine::core::engine::game_loop_coroutine::{CoroutineTaskManager, TaskPriority};

let task_manager = world.get_resource::<CoroutineTaskManager>().unwrap();

// 提交任务
let task_id = task_manager.spawn_task(
    "ai_update".to_string(),
    TaskPriority::Normal,
    || async move {
        // 异步任务
        Ok(())
    }
).await;

// 监控任务
let stats = task_manager.stats().await;
if stats.failed_tasks > 0 {
    tracing::warn!("Some tasks failed");
}
```

### 网络消息处理

**使用批量处理提升性能**:

```rust
// ✅ 好的做法：批量处理网络消息
use game_engine::network::parallel::ParallelMessageProcessor;

let processor = ParallelMessageProcessor::new(32);

// 异步批量处理
let results = processor.process_messages_async(
    messages,
    state,
    Some(compressor)
).await;
```

### 避免阻塞

**避免在异步上下文中阻塞**:

```rust
// ✅ 好的做法：使用spawn_blocking
tokio::task::spawn_blocking(move || {
    // CPU密集型任务
    heavy_computation()
}).await?;

// ❌ 避免：直接阻塞
// heavy_computation(); // 会阻塞异步运行时
```

---

## 测试策略

### 单元测试

**测试业务逻辑**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let mut entity = GameEntity::new(EntityId::new(1));
        entity.add_component(Component::Transform(Transform::default())).unwrap();
        
        assert_eq!(entity.component_count(), 1);
    }
}
```

**测试边界条件**:

```rust
#[test]
fn test_physics_step_edge_cases() {
    let mut world = PhysicsWorld::new();
    
    // 测试零时间步长
    assert!(world.step(0.0).is_ok());
    
    // 测试负时间步长
    assert!(world.step(-0.016).is_err());
    
    // 测试极大时间步长
    assert!(world.step(1.0).is_ok());
}
```

### 集成测试

**测试模块集成**:

```rust
#[test]
fn test_render_physics_integration() {
    let mut world = World::new();
    
    // 创建实体
    let entity = world.spawn((
        Transform::default(),
        RigidBody::default(),
        Sprite::default(),
    ));
    
    // 运行物理系统
    physics_system(&mut world);
    
    // 验证渲染组件已更新
    let transform = world.get::<Transform>(entity).unwrap();
    assert_ne!(transform.position, Vec3::ZERO);
}
```

### 压力测试

**测试大规模场景**:

```rust
#[test]
#[ignore] // 标记为忽略，需要时手动运行
fn test_large_scale_physics() {
    let mut world = PhysicsWorld::new();
    
    // 创建10000个刚体
    for i in 0..10000 {
        let body = RigidBody::new(/* ... */);
        world.add_body(body).unwrap();
    }
    
    // 测试性能
    let start = Instant::now();
    world.step(0.016).unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 16, "Physics step took too long");
}
```

---

## 代码组织

### 模块结构

**按功能组织模块**:

```
game_engine/src/
├── core/           # 核心功能
├── domain/        # 领域层
├── render/        # 渲染
├── physics/       # 物理
├── network/       # 网络
├── resources/     # 资源管理
└── ...
```

**使用mod.rs统一导出**:

```rust
// ✅ 好的做法：统一导出
// game_engine/src/render/mod.rs
pub mod gpu_driven;
pub mod lod;
pub mod postprocess;

pub use gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig};
pub use lod::{LodSelector, LodConfig};
pub use postprocess::{PostProcessEffectManager, PostProcessEffect};
```

### 条件编译

**使用特性标志管理可选功能**:

```rust
// ✅ 好的做法：使用特性标志
#[cfg(feature = "gltf")]
pub mod gltf_loader;

#[cfg(not(feature = "gltf"))]
pub mod gltf_loader_stub;

// 统一导出
pub use gltf_loader::{GltfLoader, GltfScene};
```

**使用平台检测函数**:

```rust
// ✅ 好的做法：使用平台检测
use game_engine::platform::detection::is_wasm;

if is_wasm() {
    // WASM特定代码
} else {
    // 原生平台代码
}
```

### 文档注释

**为公共API添加文档**:

```rust
/// 创建新的物理世界
///
/// # 参数
/// - `gravity`: 重力向量
///
/// # 返回
/// 新的物理世界实例
///
/// # 示例
/// ```
/// use game_engine::domain::physics::PhysicsWorld;
///
/// let world = PhysicsWorld::new();
/// ```
pub fn new() -> Self {
    // ...
}
```

---

## 性能监控

### 使用性能监控工具

**监控帧率**:

```rust
use game_engine_performance::monitoring::SystemPerformanceMonitor;

let mut monitor = SystemPerformanceMonitor::new();
monitor.start()?;

let metrics = monitor.get_metrics();
if metrics.frame_time > 16.67 {
    tracing::warn!("Frame time exceeded target: {:.2}ms", metrics.frame_time);
}
```

**使用性能仪表盘**:

```rust
use game_engine::profiling::dashboard::PerformanceDashboard;

let mut dashboard = PerformanceDashboard::new("127.0.0.1:8080")?;
dashboard.start()?;

// 记录指标
dashboard.record_frame_time(16.67);
dashboard.record_cpu_time(11.67);
dashboard.record_gpu_time(5.0);
```

---

## 代码审查清单

### 架构审查

- [ ] 是否遵循分层架构原则？
- [ ] 依赖方向是否正确？
- [ ] 是否使用了适当的抽象？

### ECS审查

- [ ] 组件是否是纯数据结构？
- [ ] 系统是否无状态？
- [ ] 查询是否精确？
- [ ] 是否使用了变更检测？

### 性能审查

- [ ] 是否使用了协程处理I/O？
- [ ] 是否使用了SIMD优化？
- [ ] 是否使用了GPU加速（如适用）？
- [ ] 是否避免了不必要的分配？

### 错误处理审查

- [ ] 错误类型是否清晰？
- [ ] 是否实现了错误恢复？
- [ ] 是否使用了Result传播错误？

### 测试审查

- [ ] 是否添加了单元测试？
- [ ] 是否添加了集成测试？
- [ ] 是否测试了边界条件？

---

## 相关文档

- [API参考](api_reference.md)
- [性能调优指南](performance_tuning_guide.md)
- [架构文档](architecture.md)
- [条件编译指南](CONDITIONAL_COMPILATION_GUIDE.md)
- [协程游戏循环评估](coroutine_game_loop_evaluation.md)

---

**文档版本**: 1.0  
**创建日期**: 2025-12-23  
**维护者**: Game Engine Team

