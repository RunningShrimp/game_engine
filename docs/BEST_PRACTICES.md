# 游戏引擎开发最佳实践指南

本文档提供游戏引擎开发的最佳实践和指导原则。

---

## 1. 架构设计原则

### 1.1 领域驱动设计（DDD）

#### 富领域对象模式

**原则**: 将业务逻辑封装到领域对象中，避免贫血模型。

**示例**:
```rust
// ✅ 好的设计：富领域对象
impl RenderObject {
    pub fn update_visibility(&mut self, frustum: &Frustum) {
        // 业务逻辑封装在领域对象中
        self.visible = frustum.intersects_sphere(
            self.bounding_center,
            self.bounding_radius
        );
    }
}

// ❌ 不好的设计：贫血模型
struct RenderObject {
    visible: bool,
    // 业务逻辑在外部
}
```

#### 聚合根设计

**原则**: 通过聚合根访问聚合内的实体，确保业务规则在边界内执行。

**示例**:
```rust
// ✅ 好的设计：通过聚合根访问
let mut scene = Scene::new(SceneId(1), "My Scene");
scene.add_entity(entity)?; // 业务规则在add_entity中执行

// ❌ 不好的设计：直接访问内部
scene.entities.insert(entity_id, entity); // 绕过业务规则
```

### 1.2 值对象模式

**原则**: 使用值对象封装领域概念，提高类型安全性和可维护性。

**示例**:
```rust
// ✅ 好的设计：值对象
let position = Position::new(1.0, 2.0, 3.0)?; // 包含验证
let distance = position.distance_to(other_position);

// ❌ 不好的设计：原始类型
let position: Vec3 = Vec3::new(1.0, 2.0, 3.0); // 无验证
```

---

## 2. 错误处理最佳实践

### 2.1 使用领域特定错误

**原则**: 为每个领域定义特定的错误类型，使用`thiserror`派生。

**示例**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid render state: {0}")]
    InvalidState(String),
    
    #[error("GPU operation failed: {0}")]
    GpuError(#[from] wgpu::Error),
}
```

### 2.2 错误恢复策略

**原则**: 为关键操作提供错误恢复机制。

**恢复策略类型**:
- **Retry**: 重试操作（带最大重试次数和延迟）
- **Fallback**: 使用备用方案
- **Compensation**: 执行补偿操作回滚状态
- **Ignore**: 忽略错误（仅用于非关键操作）

**示例**:
```rust
impl RenderObject {
    pub fn recover_from_error(&mut self) -> Result<(), RenderError> {
        // 恢复策略：重置到安全状态
        self.error_state = None;
        self.visible = true;
        Ok(())
    }

    pub fn create_compensation(&self) -> RenderObjectCompensation {
        // 创建补偿操作，用于回滚
        RenderObjectCompensation {
            id: self.id,
            previous_visible: self.visible,
            previous_lod: self.lod_selection.clone(),
        }
    }
}
```

### 2.3 错误聚合

**原则**: 使用错误聚合器收集和报告错误。

**示例**:
```rust
use game_engine::core::resources::ErrorAggregator;

fn system_with_error_handling(
    mut error_aggregator: ResMut<ErrorAggregator>
) {
    match risky_operation() {
        Ok(result) => { /* 处理成功 */ }
        Err(e) => {
            error_aggregator.record_error(e);
            // 继续执行，不中断系统
        }
    }
}
```

### 2.4 错误传播

**原则**: 在适当的层级处理错误，避免过度传播。

**策略**:
- **领域层**: 使用`DomainError`，包含恢复策略
- **服务层**: 转换错误，添加上下文
- **应用层**: 记录错误，提供用户友好的消息

**示例**:
```rust
// 领域层：返回领域错误
pub fn apply_force(&mut self, force: Vec3) -> Result<(), DomainError> {
    if force.length() > MAX_FORCE {
        return Err(DomainError::Physics(PhysicsError::InvalidForce(
            format!("Force {} exceeds maximum {}", force.length(), MAX_FORCE)
        )));
    }
    // ...
}

// 服务层：转换并添加上下文
pub fn apply_force_to_body(
    &mut self,
    body_id: RigidBodyId,
    force: Vec3
) -> Result<(), DomainError> {
    self.world.apply_force_to_body(body_id, force)
        .map_err(|e| DomainError::Physics(PhysicsError::BodyNotFound(
            format!("Failed to apply force to body {}: {}", body_id.as_u64(), e)
        )))
}
```

---

## 3. 性能优化指南

### 3.1 性能优化原则

**原则**: 
1. **先测量，再优化** - 使用性能分析器识别瓶颈
2. **优化关键路径** - 关注影响帧率的关键代码路径
3. **避免过早优化** - 在性能数据支持的情况下优化
4. **保持代码可读性** - 优化不应牺牲代码可维护性

### 3.2 SIMD优化

**原则**: 对关键路径的数学运算使用SIMD优化。

**适用场景**:
- 批量向量变换（如骨骼动画）
- 矩阵运算
- 物理计算（力、速度等）

**示例**:
```rust
use game_engine::performance::simd::{Vec3Simd, SimdBackend};

let backend = SimdBackend::best_available();
let result = Vec3Simd::batch_transform(vectors, matrix, backend);
```

**注意事项**:
- SIMD优化需要数据对齐
- 小批量数据可能不值得SIMD开销
- 使用硬件检测自动选择最佳后端

### 3.3 批次优化

**原则**: 使用实例化渲染减少绘制调用。

**策略**:
- **静态批次**: 不变的对象合并到静态批次
- **动态批次**: 每帧变化的对象使用动态批次
- **实例化渲染**: 相同网格的多个实例（>10）使用实例化

**示例**:
```rust
use game_engine::render::{BatchManager, DynamicBatchConfig};

let mut batch_manager = BatchManager::with_dynamic_config(
    DynamicBatchConfig::new(32, 2048, 512)
);
batch_manager.adjust_for_gpu(device);
```

**性能指标**:
- 目标：减少50%以上的draw call
- 批次大小：根据GPU能力调整（通常32-128个实例）

### 3.4 并行处理

**原则**: 对独立任务使用并行处理。

**适用场景**:
- A*寻路（多个AI实体）
- 物理岛屿计算
- 音频混合
- 批量数据转换

**示例**:
```rust
use game_engine::ai::ParallelPathfindingService;

let service = ParallelPathfindingService::new(nav_mesh, num_threads);
service.submit_path_requests(paths);
let results = service.collect_results();
```

**注意事项**:
- 确保任务之间无数据竞争
- 使用线程池避免频繁创建线程
- 考虑任务开销，小任务可能不适合并行

### 3.5 内存优化

**原则**: 减少内存分配，使用对象池。

**策略**:
- **对象池**: 重用频繁创建/销毁的对象
- **Arena分配器**: 批量分配，批量释放
- **预分配**: 在初始化时预分配缓冲区

**示例**:
```rust
use game_engine::ecs::TileEntityPool;

let mut pool = TileEntityPool::new(1000);
let entity = pool.allocate(); // 重用已分配的内存
```

### 3.6 GPU优化

**原则**: 充分利用GPU并行能力。

**策略**:
- **GPU驱动剔除**: 使用计算着色器进行视锥体剔除
- **间接绘制**: 减少CPU-GPU通信
- **异步着色器编译**: 避免阻塞主线程

**示例**:
```rust
// GPU驱动剔除（需要启用feature）
#[cfg(feature = "gpu_culling")]
renderer.enable_gpu_culling();
```

### 3.7 性能监控

**原则**: 持续监控性能指标，建立性能基准。

**关键指标**:
- 帧时间（目标：16.67ms @ 60fps）
- CPU使用率
- GPU使用率
- 内存使用
- Draw call数量

**工具**:
- 内置性能分析器（`game_engine::performance::Profiler`）
- 外部工具（perf, Instruments, RenderDoc）

---

## 4. 并发控制

### 4.1 Actor模式

**原则**: 使用Actor模式实现细粒度并发控制。

**示例**:
```rust
use game_engine::domain::actor::{ActorSystem, AudioActor};

let mut system = ActorSystem::new();
let audio_handle = system.register("audio", AudioActor::new())?;

// 在ECS系统中使用
fn audio_system(handle: Res<ActorHandle<AudioActorMessage>>) {
    handle.send(AudioActorMessage::Play { ... })?;
}
```

### 4.2 线程安全

**原则**: 使用Rust的类型系统保证线程安全。

**示例**:
```rust
// ✅ 好的设计：使用Arc和Mutex
let shared_data = Arc::new(Mutex::new(data));

// ✅ 好的设计：使用Send + Sync trait
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
}

// ❌ 不好的设计：共享可变状态
static mut COUNTER: u32 = 0; // 不安全
```

---

## 5. 测试编写指南

### 5.1 单元测试

**原则**: 为每个公共API编写单元测试。

**示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_object_creation() {
        let obj = RenderObject::new(id, mesh, transform);
        assert_eq!(obj.id, id);
        assert!(obj.visible);
    }

    #[test]
    fn test_render_object_visibility() {
        let mut obj = RenderObject::new(id, mesh, transform);
        let frustum = Frustum::from_view_projection(view_proj);
        obj.update_visibility(&frustum);
        assert!(obj.visible);
    }
}
```

### 5.2 集成测试

**原则**: 为系统间的交互编写集成测试。

**示例**:
```rust
#[test]
fn test_parallel_pathfinding_integration() {
    let mesh = create_test_nav_mesh();
    let service = ParallelPathfindingService::new(mesh, 4);
    
    let request_id = service.submit_request(start, end);
    let result = service.wait_for_result(request_id, 1000);
    
    assert!(result.is_some());
    assert!(result.unwrap().path.is_some());
}
```

### 5.3 属性测试（Property Testing）

**原则**: 使用proptest为值对象和领域逻辑添加属性测试。

**适用场景**:
- 值对象验证逻辑
- 数学运算（交换律、结合律等）
- 领域对象不变性

**示例**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_position_always_valid(
        x in -1000.0f32..1000.0,
        y in -1000.0f32..1000.0,
        z in -1000.0f32..1000.0
    ) {
        let pos = Position::new(x, y, z);
        prop_assert!(pos.is_some());
        let pos = pos.unwrap();
        prop_assert!(pos.x.is_finite());
        prop_assert!(pos.y.is_finite());
        prop_assert!(pos.z.is_finite());
    }
}
```

### 5.4 错误处理测试

**原则**: 为错误恢复策略和补偿操作编写专门测试。

**示例**:
```rust
#[test]
fn test_error_recovery() {
    let mut obj = RenderObject::new(id, mesh, transform);
    
    // 模拟错误状态
    obj.error_state = Some(RenderError::InvalidState("test".to_string()));
    
    // 测试恢复
    assert!(obj.recover_from_error().is_ok());
    assert!(obj.error_state.is_none());
    assert!(obj.visible);
}

#[test]
fn test_compensation() {
    let mut obj = RenderObject::new(id, mesh, transform);
    obj.visible = false;
    
    // 创建补偿操作
    let compensation = obj.create_compensation();
    
    // 修改状态
    obj.visible = true;
    
    // 应用补偿（回滚）
    compensation.apply(&mut obj);
    assert!(!obj.visible);
}
```

### 5.5 性能基准测试

**原则**: 为关键操作建立性能基准，防止性能回归。

**示例**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_physics_step(c: &mut Criterion) {
    let mut world = PhysicsWorld::new();
    // 设置测试场景
    
    c.bench_function("physics_step", |b| {
        b.iter(|| {
            world.step(black_box(0.016))?;
        });
    });
}

criterion_group!(benches, benchmark_physics_step);
criterion_main!(benches);
```

### 5.6 测试覆盖率目标

**原则**: 保持合理的测试覆盖率。

**目标**:
- **领域层**: 90%+ 覆盖率（业务逻辑关键）
- **服务层**: 80%+ 覆盖率
- **基础设施层**: 70%+ 覆盖率（硬件相关代码难以测试）

**工具**:
- `cargo tarpaulin` - Rust测试覆盖率工具
- `cargo test -- --nocapture` - 显示测试输出

### 5.7 测试编写最佳实践

**原则**: 编写清晰、可维护的测试。

**测试命名**:
- 使用描述性的测试名称：`test_render_object_visibility_update_with_frustum_culling`
- 测试名称应该说明测试的场景和预期结果

**测试组织**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 按功能分组测试
    mod creation {
        use super::*;

        #[test]
        fn test_new() { /* ... */ }
    }

    mod visibility {
        use super::*;

        #[test]
        fn test_update_visibility() { /* ... */ }
    }
}
```

**测试数据**:
- 使用测试辅助函数创建测试数据
- 使用`proptest`生成测试数据
- 避免硬编码的魔法数字

**测试隔离**:
- 每个测试应该是独立的
- 使用`#[test]`而不是共享状态
- 清理测试资源（如临时文件）

### 5.8 集成测试指南

**原则**: 为系统间的交互编写集成测试。

**集成测试位置**:
- `tests/integration_test.rs` - 主要集成测试文件
- `tests/`目录下的其他测试文件

**集成测试内容**:
- 系统间交互（渲染+物理、音频+网络等）
- 错误处理流程
- 性能关键路径
- 端到端场景

**示例**:
```rust
// tests/integration_test.rs
use game_engine::core::Engine;
use bevy_ecs::prelude::*;

#[test]
fn test_render_physics_integration() {
    // 测试渲染和物理系统的集成
    let mut world = World::new();
    // 设置测试场景
    // 运行系统
    // 验证结果
}
```

---

## 6. 代码组织

### 6.1 模块结构

**原则**: 按领域组织代码，保持模块职责单一。

```
src/
├── domain/          # 领域层（富领域对象）
├── services/        # 应用服务层
├── render/          # 基础设施层（渲染）
├── physics/         # 基础设施层（物理）
└── core/            # 核心引擎功能
```

### 6.2 命名约定

**原则**: 使用清晰的命名，遵循Rust命名约定。

- **类型**: `PascalCase` (如 `RenderObject`, `DynamicBatchConfig`)
- **函数**: `snake_case` (如 `update_visibility`, `submit_request`)
- **常量**: `SCREAMING_SNAKE_CASE` (如 `MAX_BATCH_SIZE`)
- **模块**: `snake_case` (如 `render`, `pathfinding`)

---

## 7. 文档编写

### 7.1 文档编写原则

**核心原则**:
1. **所有公共API必须有文档** - 启用`#![warn(missing_docs)]`确保文档完整性
2. **文档应该解释"为什么"而不仅仅是"是什么"** - 说明设计决策和使用场景
3. **包含使用示例** - 每个公共API都应该有可编译运行的示例
4. **说明错误情况** - 文档化可能的错误和恢复策略
5. **说明性能注意事项** - 对于性能敏感的操作，说明性能特征

### 7.2 公共API文档

**原则**: 为所有公共API添加文档注释。

**文档结构**:
```rust
/// 简短的一句话描述
///
/// 详细的描述，解释用途、设计决策和使用场景。
/// 可以包含多段内容。
///
/// # 参数
///
/// * `param1` - 参数1的描述和约束
/// * `param2` - 参数2的描述和约束
///
/// # 返回
///
/// 返回值的描述，包括成功和失败情况。
///
/// # 错误
///
/// 可能返回的错误类型和原因：
/// - `RenderError::InvalidState` - 当渲染状态无效时
/// - `RenderError::GpuError` - 当GPU操作失败时
///
/// # 性能
///
/// 性能注意事项（如适用）：
/// - 时间复杂度：O(n)
/// - 内存分配：可能分配临时缓冲区
///
/// # 示例
///
/// ```rust
/// use game_engine::render::RenderObject;
///
/// let obj = RenderObject::new(id, mesh, transform);
/// obj.update_visibility(&frustum);
/// ```
pub fn new(id: RenderObjectId, mesh: Arc<GpuMesh>, transform: Transform) -> Self {
    // ...
}
```

### 7.3 模块文档

**原则**: 为每个模块添加模块级文档注释，说明模块的职责和设计原则。

**示例**:
```rust
//! 渲染领域对象模块
//!
//! 实现富领域对象设计模式，将渲染业务逻辑封装到领域对象中。
//!
//! ## 设计原则
//!
//! - **RenderObject**: 封装渲染对象的业务逻辑（可见性计算、LOD选择等）
//! - **RenderStrategy**: 封装渲染策略决策（前向渲染、延迟渲染等）
//! - **RenderScene**: 聚合根，管理整个渲染场景，确保业务规则在边界内执行
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::domain::render::{RenderScene, RenderObject};
//!
//! let mut scene = RenderScene::new();
//! let obj = RenderObject::new(id, mesh, transform);
//! scene.add_object(obj)?;
//! ```
```

### 7.4 文档覆盖率目标

**原则**: 保持高文档覆盖率，特别是公共API。

**目标**:
- **公共API**: 100%文档覆盖率
- **模块级文档**: 所有模块都应该有模块级文档
- **使用示例**: 关键模块应该有使用示例

**工具**:
- `cargo doc --no-deps` - 生成文档并检查警告
- `cargo doc --open` - 生成并打开文档
- `#![warn(missing_docs)]` - 启用文档缺失警告

### 7.5 文档维护

**原则**: 保持文档与代码同步。

**检查清单**:
- [ ] 添加新公共API时，立即添加文档
- [ ] 修改API时，更新文档
- [ ] 定期运行`cargo doc`检查文档警告
- [ ] 确保示例代码可以编译运行

---

## 8. 代码重复消除

### 8.1 识别代码重复

**原则**: 定期审查代码，识别重复模式。

**常见重复模式**:
1. **Default实现重复** - 多个结构体有相似的`Default`实现
2. **构造函数模式不统一** - 部分使用`new()`，部分使用`default()`
3. **错误类型定义重复** - 多个模块定义相似的错误类型
4. **工具函数重复** - 相同功能的函数在多个模块中重复实现

**识别工具**:
- 代码审查
- 静态分析工具
- `grep`搜索重复模式

### 8.2 消除代码重复策略

#### 8.2.1 Default实现统一

**原则**: 优先使用`#[derive(Default)]`，需要自定义逻辑时才手动实现。

**示例**:
```rust
// ✅ 好的设计：使用derive
#[derive(Default)]
pub struct Config {
    pub enabled: bool,
    pub timeout: u64,
}

// ✅ 好的设计：需要自定义逻辑时手动实现
impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal // 自定义默认值
    }
}
```

#### 8.2.2 构造函数模式统一

**原则**: 统一使用`pub fn new() -> Self`模式，保留必要的builder模式。

**示例**:
```rust
// ✅ 好的设计：统一使用new()
impl RenderObject {
    pub fn new(id: RenderObjectId, mesh: Arc<GpuMesh>) -> Self {
        Self { id, mesh, /* ... */ }
    }
}

// ✅ 好的设计：复杂配置使用builder
impl RenderConfig {
    pub fn builder() -> RenderConfigBuilder {
        RenderConfigBuilder::default()
    }
}
```

#### 8.2.3 错误类型统一

**原则**: 使用`thiserror`派生宏，提取公共错误类型。

**示例**:
```rust
// ✅ 好的设计：使用thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid render state: {0}")]
    InvalidState(String),
    
    #[error("GPU operation failed: {0}")]
    GpuError(#[from] wgpu::Error),
}

// ✅ 好的设计：提取公共错误类型
pub mod common_errors {
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum CommonError {
        #[error("Resource not found: {0}")]
        NotFound(String),
    }
}
```

#### 8.2.4 工具函数提取

**原则**: 将重复的工具函数提取到公共模块。

**示例**:
```rust
// src/core/utils.rs
/// 获取当前Unix时间戳（秒）
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// 在其他模块中使用
use game_engine::core::utils::current_timestamp;
let ts = current_timestamp();
```

### 8.3 代码重复检查清单

**定期检查**:
- [ ] 是否有重复的Default实现？
- [ ] 构造函数模式是否统一？
- [ ] 错误类型定义是否统一？
- [ ] 是否有重复的工具函数？
- [ ] 是否有重复的业务逻辑？

**消除步骤**:
1. 识别重复模式
2. 提取公共实现
3. 更新所有引用
4. 添加测试验证
5. 更新文档

---

## 9. 性能监控

### 8.1 使用性能分析器

**原则**: 定期使用性能分析器识别瓶颈。

**示例**:
```rust
use game_engine::performance::Profiler;

let mut profiler = Profiler::new();
profiler.start_frame();

// 执行操作
render_scene();

profiler.end_frame();
let report = profiler.generate_report();
```

### 8.2 建立性能基准

**原则**: 为关键操作建立性能基准，防止回归。

**示例**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pathfinding(c: &mut Criterion) {
    c.bench_function("parallel_pathfinding", |b| {
        b.iter(|| {
            service.submit_path_requests(black_box(paths.clone()));
        });
    });
}

criterion_group!(benches, benchmark_pathfinding);
criterion_main!(benches);
```

---

## 10. 错误预防

### 9.1 使用类型系统

**原则**: 利用Rust的类型系统防止错误。

**示例**:
```rust
// ✅ 好的设计：使用新类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderObjectId(pub u64);

// ❌ 不好的设计：使用原始类型
type RenderObjectId = u64; // 容易混淆
```

### 9.2 验证输入

**原则**: 在边界处验证输入，使用值对象封装验证逻辑。

**示例**:
```rust
impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Option<Self> {
        // 验证：位置不能包含NaN或无穷大
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Self { x, y, z })
        } else {
            None
        }
    }
}
```

---

## 11. 代码审查清单

### 10.1 架构检查

- [ ] 是否遵循DDD原则？
- [ ] 业务逻辑是否封装在领域对象中？
- [ ] 是否通过聚合根访问聚合？
- [ ] 是否使用值对象封装领域概念？

### 10.2 性能检查

- [ ] 关键路径是否优化？
- [ ] 是否使用批次渲染？
- [ ] 是否使用并行处理？
- [ ] 是否有性能基准测试？

### 10.3 质量检查

- [ ] 是否有单元测试？
- [ ] 是否有文档注释？
- [ ] 错误处理是否完善？
- [ ] 代码是否遵循命名约定？

---

## 12. 常见陷阱

### 11.1 避免贫血模型

**问题**: 将业务逻辑放在服务层，领域对象只有数据。

**解决**: 将业务逻辑移到领域对象中。

### 11.2 避免过度抽象

**问题**: 为了"灵活性"创建过多抽象层。

**解决**: 遵循YAGNI原则，只在需要时抽象。

### 11.3 避免过早优化

**问题**: 在没有性能数据的情况下优化。

**解决**: 先测量，再优化。

---

## 13. 参考资源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Domain-Driven Design](https://www.domainlanguage.com/ddd/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

## 更新日志

- 2025-12-01: 基于系统审查报告完善文档编写指南、测试编写指南和代码重复消除指南
- 2025-01-XX: 初始版本

