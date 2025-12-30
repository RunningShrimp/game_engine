# 游戏引擎最佳实践指南

## 概述

本指南提供了使用游戏引擎的最佳实践，帮助您编写高性能、可维护的代码。

---

## 架构原则

### 1. 关注点分离

**原则**: 不同职责的代码应该分离

**示例**:
```rust
// ❌ 错误：混合关注点
fn update_and_render(&mut self) {
    self.update_physics();
    self.render();
    self.play_audio();
}

// ✅ 正确：分离关注点
fn update(&mut self) {
    self.update_physics();
    self.play_audio();
}

fn render(&self) {
    self.render_scene();
}
```

---

### 2. 依赖注入

**原则**: 使用依赖注入而非硬编码依赖

**示例**:
```rust
// ❌ 错误：硬编码依赖
struct Game {
    physics: PhysicsEngine,
    renderer: Renderer,
}

impl Game {
    fn new() -> Self {
        Self {
            physics: PhysicsEngine::new(),
            renderer: Renderer::new(),
        }
    }
}

// ✅ 正确：依赖注入
struct Game<P, R> {
    physics: P,
    renderer: R,
}

impl<P: Physics, R: Render> Game<P, R> {
    fn new(physics: P, renderer: R) -> Self {
        Self { physics, renderer }
    }
}
```

---

### 3. 组合优于继承

**原则**: 使用trait对象和组合

**示例**:
```rust
// ❌ 错误：深层继承
trait GameObject { }
trait RenderableObject: GameObject { }
trait PhysicalObject: GameObject { }
trait Player: RenderableObject + PhysicalObject { }

// ✅ 正确：组合
struct Entity {
    render: Option<Box<dyn Render>>,
    physics: Option<Box<dyn Physics>>,
    behavior: Option<Box<dyn Behavior>>,
}
```

---

## 性能优化

### 1. 同步优于异步

**原则**: 纯计算使用同步函数

**示例**:
```rust
// ❌ 错误：纯计算使用async
pub async fn calculate_physics(pos: Vec3, vel: Vec3, dt: f32) -> Vec3 {
    pos + vel * dt
}

// ✅ 正确：使用同步函数
pub fn calculate_physics(pos: Vec3, vel: Vec3, dt: f32) -> Vec3 {
    pos + vel * dt
}
```

**何时使用async**:
- 网络I/O
- 大文件I/O (>100KB)
- 需要并行的I/O操作

**何时使用sync**:
- 纯计算
- 简单查询
- 内存操作

---

### 2. 批量操作

**原则**: 批量处理减少开销

**示例**:
```rust
// ❌ 错误：逐个处理
for i in 0..1000 {
    scheduler.schedule(Task::new(
        format!("task_{}", i),
        Box::new(|| /* ... */),
        TaskPriority::Medium,
    ));
}

// ✅ 正确：批量处理
let tasks: Vec<_> = (0..1000)
    .map(|i| {
        Task::new(
            format!("task_{}", i),
            Box::new(|| /* ... */),
            TaskPriority::Medium,
        )
    })
    .collect();
scheduler.schedule_batch(tasks);
```

---

### 3. 使用高效的数据结构

**原则**: 根据使用场景选择合适的数据结构

**示例**:
```rust
// ❌ 错误：使用Vec查找
fn find_entity(entities: &Vec<Entity>, id: u32) -> Option<&Entity> {
    entities.iter().find(|e| e.id == id)
}

// ✅ 正确：使用HashMap
fn find_entity(entities: &HashMap<u32, Entity>, id: u32) -> Option<&Entity> {
    entities.get(&id)
}

// ✅ 更好：使用DashMap（并发场景）
use dashmap::DashMap;

fn find_entity(entities: &DashMap<u32, Entity>, id: u32) -> Option<Entity> {
    entities.get(&id).map(|e| e.clone())
}
```

---

### 4. 避免不必要的分配

**原则**: 重用缓冲区，减少分配

**示例**:
```rust
// ❌ 错误：每次循环分配
for i in 0..1000 {
    let buffer = vec![0u8; 1024];
    process(&buffer);
}

// ✅ 正确：重用缓冲区
let mut buffer = vec![0u8; 1024];
for i in 0..1000 {
    buffer.clear();
    process(&mut buffer);
}
```

---

### 5. 使用对象池

**原则**: 重用对象而非频繁创建销毁

**示例**:
```rust
use std::sync::Mutex;

struct ObjectPool<T> {
    objects: Vec<Mutex<T>>,
}

impl<T: Default> ObjectPool<T> {
    fn new(capacity: usize) -> Self {
        let objects = (0..capacity)
            .map(|_| Mutex::new(T::default()))
            .collect();
        Self { objects }
    }

    fn get(&self) -> MutexGuard<T> {
        // 简化实现：轮询获取
        let index = 0; // 实际应该使用更智能的策略
        self.objects[index].lock().unwrap()
    }
}
```

---

## 错误处理

### 1. 使用Result而非panic

**原则**: 可恢复的错误使用Result

**示例**:
```rust
// ❌ 错误：panic!不可恢复
fn divide(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        panic!("除数不能为零");
    }
    a / b
}

// ✅ 正确：返回Result
fn divide(a: f32, b: f32) -> Result<f32, String> {
    if b == 0.0 {
        return Err("除数不能为零".to_string());
    }
    Ok(a / b)
}
```

---

### 2. 提供有意义的错误信息

**原则**: 错误信息应该清晰有用

**示例**:
```rust
// ❌ 错误：模糊的错误信息
fn load_asset(path: &str) -> Result<Vec<u8>, String> {
    Err("加载失败".to_string())
}

// ✅ 正确：详细的错误信息
fn load_asset(path: &str) -> Result<Vec<u8>, String> {
    use std::io;
    std::fs::read(path)
        .map_err(|e| format!("无法加载资产 '{}': {}", path, e))
}
```

---

### 3. 使用thiserror

**原则**: 使用thiserror简化错误处理

**示例**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameEngineError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("资源未找到: {0}")]
    ResourceNotFound(String),
    
    #[error("渲染错误: {0}")]
    Render(String),
}
```

---

## 并发编程

### 1. 最小化锁范围

**原则**: 锁的持有时间尽可能短

**示例**:
```rust
// ❌ 错误：长时间持有锁
let data = mutex.lock().unwrap();
// ... 大量计算 ...
drop(data);

// ✅ 正确：尽快释放锁
let result = {
    let data = mutex.lock().unwrap();
    data.calculate()
};
// ... 计算不持有锁 ...
```

---

### 2. 使用读写锁

**原则**: 读多写少场景使用RwLock

**示例**:
```rust
use parking_lot::RwLock;

struct GameState {
    entities: RwLock<Vec<Entity>>,
}

impl GameState {
    // 读操作：并发
    fn get_entity(&self, id: u32) -> Option<Entity> {
        let entities = self.entities.read();
        entities.iter().find(|e| e.id == id).cloned()
    }

    // 写操作：独占
    fn add_entity(&self, entity: Entity) {
        let mut entities = self.entities.write();
        entities.push(entity);
    }
}
```

---

### 3. 使用原子操作

**原则**: 简单计数器使用原子类型

**示例**:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct Metrics {
    frame_count: AtomicUsize,
    entity_count: AtomicUsize,
}

impl Metrics {
    fn increment_frame(&self) {
        self.frame_count.fetch_add(1, Ordering::Relaxed);
    }

    fn get_frame_count(&self) -> usize {
        self.frame_count.load(Ordering::Relaxed)
    }
}
```

---

### 4. 使用channels传递消息

**原则**: 线程间通信使用channels

**示例**:
```rust
use std::sync::mpsc;

enum Message {
    Update(f32),
    Render,
    Shutdown,
}

fn worker_thread(receiver: mpsc::Receiver<Message>) {
    for msg in receiver {
        match msg {
            Message::Update(dt) => {
                // 更新逻辑
            }
            Message::Render => {
                // 渲染逻辑
            }
            Message::Shutdown => {
                break;
            }
        }
    }
}
```

---

## 内存管理

### 1. 使用RAII

**原则**: 资源获取即初始化

**示例**:
```rust
struct Texture {
    id: u32,
}

impl Texture {
    fn new(path: &str) -> Result<Self, String> {
        let id = load_texture(path)?;
        Ok(Self { id })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unload_texture(self.id);
    }
}
```

---

### 2. 避免循环引用

**原则**: 使用Weak打破循环

**示例**:
```rust
use std::rc::{Rc, Weak};

struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}
```

---

### 3. 使用Cow避免克隆

**原则**: 可能修改时使用Cow

**示例**:
```rust
use std::borrow::Cow;

fn process_string(s: Cow<str>) -> Cow<str> {
    if s.contains("old") {
        // 需要修改
        Cow::Owned(s.replace("old", "new"))
    } else {
        // 不需要修改
        s
    }
}
```

---

## 测试策略

### 1. 单元测试

**原则**: 测试单个函数

**示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_physics() {
        let pos = Vec3::ZERO;
        let vel = Vec3::new(1.0, 2.0, 3.0);
        let dt = 0.016;

        let result = calculate_physics(pos, vel, dt);

        assert!((result.x - 0.016).abs() < 0.0001);
        assert!((result.y - 0.032).abs() < 0.0001);
        assert!((result.z - 0.048).abs() < 0.0001);
    }
}
```

---

### 2. 集成测试

**原则**: 测试模块交互

**示例**:
```rust
// tests/integration_test.rs

use game_engine::prelude::*;

#[test]
fn test_engine_initialization() {
    let engine = Engine::new(EngineConfig::default()).unwrap();
    assert_eq!(engine.get_frame_count(), 0);
}

#[test]
fn test_entity_spawn() {
    let mut world = World::new();
    let entity = world.spawn((Transform::default(),));
    assert!(world.is_alive(entity));
}
```

---

### 3. 性能测试

**原则**: 使用Criterion进行基准测试

**示例**:
```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn benchmark_physics(c: &mut Criterion) {
    c.bench_function("calculate_physics", |b| {
        b.iter(|| {
            black_box(calculate_physics(
                black_box(Vec3::ZERO),
                black_box(Vec3::new(1.0, 2.0, 3.0)),
                black_box(0.016),
            ))
        })
    });
}

criterion_group!(benches, benchmark_physics);
criterion_main!(benches);
```

---

## 文档编写

### 1. API文档

**原则**: 提供清晰的API文档

**示例**:
```rust
/// 计算物理位置更新
///
/// # 参数
///
/// - `position`: 当前位置
/// - `velocity`: 速度向量
/// - `delta_time`: 时间步长（秒）
///
/// # 返回
///
/// 新的位置
///
/// # 示例
///
/// ```
/// use game_engine::prelude::*;
///
/// let new_pos = calculate_physics(
///     Vec3::ZERO,
///     Vec3::new(1.0, 2.0, 3.0),
///     0.016,
/// );
/// ```
///
/// # 性能
///
/// 此函数是同步的，性能比异步版本快约10倍。
pub fn calculate_physics(position: Vec3, velocity: Vec3, delta_time: f32) -> Vec3 {
    position + velocity * delta_time
}
```

---

### 2. 示例代码

**原则**: 提供可运行的示例

**示例**:
```rust
/// # 示例
///
/// ```
/// use game_engine::prelude::*;
///
/// fn main() {
///     let mut world = World::new();
///     let entity = world.spawn((Transform::default(),));
///     
///     // 查询实体
///     let mut query = world.query::<&mut Transform>();
///     for mut transform in query.iter_mut(&mut world) {
///         transform.pos.x += 1.0;
///     }
/// }
/// ```
```

---

## 安全性

### 1. 输入验证

**原则**: 验证所有外部输入

**示例**:
```rust
fn spawn_entity(id: u32, position: Vec3) -> Result<Entity, String> {
    if id == 0 {
        return Err("实体ID不能为零".to_string());
    }
    
    if !position.is_finite() {
        return Err("位置必须有限".to_string());
    }
    
    Ok(Entity::new(id, position))
}
```

---

### 2. 防止整数溢出

**原则**: 使用checked或saturating操作

**示例**:
```rust
// ❌ 错误：可能溢出
fn add(a: u32, b: u32) -> u32 {
    a + b
}

// ✅ 正确：检查溢出
fn add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

// 或使用saturating
fn add_saturating(a: u32, b: u32) -> u32 {
    a.saturating_add(b)
}
```

---

## 代码风格

### 1. 命名约定

**原则**: 遵循Rust命名约定

```rust
// 结构体：PascalCase
struct GameState { }

// 函数和变量：snake_case
fn update_game_state() { }
let current_state = GameState::new();

// 常量：SCREAMING_SNAKE_CASE
const MAX_ENTITIES: usize = 10000;

// Trait: PascalCase
trait Updatable { }
```

---

### 2. 代码组织

**原则**: 逻辑分组，清晰结构

```rust
// 1. 导入
use std::collections::HashMap;
use crate::module::Type;

// 2. 类型定义
pub struct MyStruct { }

// 3. Trait实现
impl MyStruct {
    pub fn new() -> Self { }
}

// 4. 私有函数
fn helper_function() { }

// 5. 测试
#[cfg(test)]
mod tests { }
```

---

## 工具使用

### 1. Clippy

**原则**: 使用Clippy捕获常见错误

```bash
cargo clippy --workspace
```

### 2. Rustfmt

**原则**: 保持代码格式一致

```bash
cargo fmt --all
```

### 3. 文档测试

**原则**: 文档中的示例应该是可测试的

```bash
cargo test --workspace --doc
```

---

## 总结

### 核心原则

1. **简单性**: 保持代码简单易懂
2. **性能**: 避免不必要的开销
3. **安全**: 类型安全和内存安全
4. **可维护性**: 清晰的结构和文档
5. **测试**: 充分的测试覆盖

### 记住

> "过早优化是万恶之源，但不优化的代码也是灾难。"

遵循最佳实践，在合适的时机进行优化。

---

**祝您编码愉快！** 🚀
