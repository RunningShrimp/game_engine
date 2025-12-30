# 游戏引擎任务分解执行清单

**项目**: Rust 高性能通用游戏引擎
**文档版本**: v1.0
**创建日期**: 2025-12-30
**参考文档**: PARALLEL_IMPLEMENTATION_PLAN.md

---

## 快速导航

- [P0: 立即执行任务](#p0-立即执行任务) - 第1-2个月
- [P1: 短期执行任务](#p1-短期执行任务) - 第3-6个月
- [P2: 中长期执行任务](#p2-中长期执行任务) - 第6-12个月
- [依赖关系图](#依赖关系图)
- [执行检查表](#执行检查表)

---

## P0: 立即执行任务

### P0-0: 依赖升级 (1-2天)

#### P0-0-1: 升级 bincode 1.3 → 2.0

**优先级**: 🔴 P0
**预估时间**: 4-6小时
**依赖**: 无
**破坏性**: 是

**任务步骤**:
```bash
# 1. 创建特性分支
git checkout -b upgrade/upgrade-bincode-2.0

# 2. 更新 Cargo.toml
# 在 game_engine/Cargo.toml 中:
bincode = { version = "2.0", features = ["serde"] }

# 3. 更新代码（API 变更）
# bincode 2.0 主要变化:
# - 序列化 API 变更
# - 错误处理变更
```

**代码迁移指南**:
```rust
// ❌ bincode 1.x
use bincode::{serialize, deserialize};

let data = serialize(&obj)?;
let obj: MyType = deserialize(&data)?;

// ✅ bincode 2.0
use bincode::{encode_from_slice, decode_from_slice};

let mut buffer = Vec::new();
let size = encode_from_slice(&mut buffer, &obj, bincode::config::standard())?;
let (obj, _): (MyType, usize) = decode_from_slice(&data, bincode::config::standard())?;
```

**需要更新的文件**:
- [ ] `game_engine/src/network/synchronization.rs`
- [ ] `game_engine/src/resources/compressed_cache.rs`
- [ ] `game_engine/src/scene/serialization.rs`
- [ ] 其他使用 bincode 的文件

**检查清单**:
- [ ] 更新 Cargo.toml
- [ ] 更新所有 API 调用
- [ ] 运行 `cargo build --all-features`
- [ ] 运行 `cargo test --all-features`
- [ ] 更新文档
- [ ] 提交 PR

**预期结果**:
- ✅ 所有编译错误解决
- ✅ 所有测试通过
- ✅ 性能无回退

---

#### P0-0-2: 升级 hex 0.4 → 0.5

**优先级**: 🟡 P1
**预估时间**: 2-3小时
**依赖**: 无
**破坏性**: 可能

**任务步骤**:
```bash
# 1. 创建特性分支
git checkout -b upgrade/upgrade-hex-0.5

# 2. 更新 Cargo.toml
# 在 workspace.dependencies 或 game_engine/Cargo.toml 中:
hex = "0.5"
```

**需要检查的文件**:
- [ ] `game_engine/src/network/key_exchange.rs`
- [ ] 其他使用 hex 编码的文件

**检查清单**:
- [ ] 更新 Cargo.toml
- [ ] 运行 `cargo build --all-features`
- [ ] 运行 `cargo test --all-features`
- [ ] 提交 PR

---

#### P0-0-3: 验证 Rust 2024 兼容性

**优先级**: 🟢 P2
**预估时间**: 1-2天
**依赖**: 无

**任务步骤**:

1. **确认 Rust 工具链版本**
```bash
rustc --version  # 应该 >= 1.85.0
cargo --version
```

2. **验证 edition 配置**
```toml
# game_engine/Cargo.toml
[package]
edition = "2024"
```

3. **测试新特性**

**异步闭包测试**:
```rust
// 添加测试文件: game_engine/tests/async_closure.rs
#[test]
fn test_async_closure() {
    let async_add = async |a: i32, b: i32| a + b;
    // 测试异步闭包功能
}
```

**AsyncFn traits 测试**:
```rust
// 测试新的 AsyncFn traits
use std::future::Future;
use std::async_fn::AsyncFn; // 如果稳定
```

**元组 FromIterator 测试**:
```rust
#[test]
fn test_tuple_from_iterator() {
    let data = vec![1, 2, 3];
    let (a, b, c): (i32, i32, i32) = data.into_iter().collect();
    assert_eq!((a, b, c), (1, 2, 3));
}
```

**检查清单**:
- [ ] Rust 工具链 >= 1.85.0
- [ ] edition = "2024" 配置正确
- [ ] 异步闭包测试通过
- [ ] 新 traits 测试通过
- [ ] 元组 collect 测试通过
- [ ] 所有现有测试通过

---

### P0-1: 条件编译清理 (2周)

#### P0-1-1: 统一资源管理器实现（DashMap）

**优先级**: 🔴 P0
**预估时间**: 3-5天
**依赖**: P0-0
**影响范围**: 60+ 处条件编译

**问题分析**:
当前存在大量 `#[cfg(feature = "dashmap")]` 条件编译，两个分支使用相同类型。

**解决方案**:
```rust
// 在 game_engine/src/resources/mod.rs 中统一

// 1. 移除条件编译，直接使用 DashMap
use dashmap::DashMap;
use parking_lot::RwLock;

// 2. 定义统一的资源管理器
pub type AssetManager<T> = DashMapAssetManager<T>;

// 3. 实现统一的资源管理器
pub struct DashMapAssetManager<T> {
    resources: DashMap<AssetId, Arc<AssetContainer<T>>>,
}
```

**任务步骤**:

1. **分析当前使用情况**
```bash
# 找出所有使用条件编译的地方
grep -r "#\[cfg(feature = \"dashmap\")\]" game_engine/src/
```

2. **修改类型定义**
```rust
// game_engine/src/resources/mod.rs

// ❌ 移除
#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(not(feature = "dashmap"))]
use parking_lot::RwLock;

// ✅ 统一为
use dashmap::DashMap;
use parking_lot::RwLock;
```

3. **更新 Cargo.toml**
```toml
# 将 dashmap 设为默认特性
[features]
default = ["dashmap"]
dashmap = ["dep:dashmap"]
```

4. **更新所有使用点**
- [ ] `game_engine/src/resources/manager.rs` (6处)
- [ ] `game_engine/src/resources/optimized_manager.rs` (28处)
- [ ] `game_engine/src/resources/unified_manager.rs` (9处)
- [ ] `game_engine/src/network/synchronization.rs` (9处)
- [ ] `game_engine/src/network/server.rs` (27处)
- 其他文件...

**检查清单**:
- [ ] 分析所有条件编译使用
- [ ] 更新类型定义
- [ ] 更新 Cargo.toml 特性配置
- [ ] 更新所有使用点（约60处）
- [ ] 移除无意义的条件编译
- [ ] 运行 `cargo build --all-features`
- [ ] 运行 `cargo test --all-features`
- [ ] 更新文档

**预期结果**:
- ✅ 减少 60+ 处条件编译
- ✅ DashMap 成为默认实现
- ✅ 代码更简洁易维护

---

#### P0-1-2: 简化密钥交换实现

**优先级**: 🟡 P1
**预估时间**: 2-3天
**依赖**: P0-1-1

**问题分析**:
存在 `secure_key_exchange` 和 `insecure_key_exchange` 两个特性，不安全实现仅用于测试。

**解决方案**:
```rust
// 移除不安全实现，统一使用 ECDH + HKDF

// game_engine/src/network/key_exchange.rs

// ❌ 移除
#[cfg(feature = "insecure_key_exchange")]
pub struct InsecureKeyExchange { /* ... */ }

// ✅ 保留并改进
#[cfg(feature = "secure_key_exchange")]
pub struct SecureKeyExchange {
    // ECDH + HKDF 实现
}

// 设为默认特性
[features]
default = ["secure_key_exchange"]
secure_key_exchange = ["dep:x25519-dalek-ng", "dep:hkdf"]
```

**检查清单**:
- [ ] 移除不安全密钥交换实现
- [ ] 更新 Cargo.toml 特性配置
- [ ] 更新所有使用点
- [ ] 更新文档说明安全性
- [ ] 运行测试

---

#### P0-1-3: 简化 GLTF 加载器条件编译

**优先级**: 🟡 P1
**预估时间**: 1-2天
**依赖**: P0-1-1

**解决方案**:
```rust
// 使用可选依赖的标准模式

// Cargo.toml
[dependencies]
gltf = { version = "1.4.1", optional = true }

[features]
default = []  # 不默认启用
gltf = ["dep:gltf"]

// 代码中使用条件编译
#[cfg(feature = "gltf")]
pub mod gltf_loader;

#[cfg(not(feature = "gltf"))]
pub mod gltf_loader {
    // 提供存根实现或编译时错误
}
```

**检查清单**:
- [ ] 检查 GLTF 相关条件编译（15处）
- [ ] 统一条件编译模式
- [ ] 更新文档
- [ ] 运行测试

---

#### P0-1-4: 简化 Tracy 集成条件编译

**优先级**: 🟡 P1
**预估时间**: 1-2天
**依赖**: P0-1-1

**解决方案**:
```rust
// 统一 Tracy 接口

// game_engine/src/profiling/mod.rs

#[cfg(feature = "tracy")]
pub use tracy::*;

#[cfg(not(feature = "tracy"))]
pub use mock::*;  // 提供存根实现

// 存根实现
#[cfg(not(feature = "tracy"))]
mod mock {
    #[inline(always)]
    pub fn span(name: &str) -> Span {
        Span::new()
    }

    pub struct Span;
    impl Span {
        pub fn new() -> Self { Self }
        pub fn end(self) {}
    }
}
```

**检查清单**:
- [ ] 检查 Tracy 相关条件编译（10处）
- [ ] 创建统一的接口
- [ ] 更新文档
- [ ] 运行测试

---

### P0-2: 代码去重 (2周)

#### P0-2-1: 合并 manager.rs 和 optimized_manager.rs

**优先级**: 🔴 P0
**预估时间**: 3-5天
**依赖**: P0-1

**问题分析**:
存在两个资源管理器实现，功能重叠，维护困难。

**任务步骤**:

1. **对比分析两个实现**
```bash
# 查看差异
diff game_engine/src/resources/manager.rs game_engine/src/resources/optimized_manager.rs
```

2. **选择最佳实现**
- 保留 `DashMapAssetManager`（性能更好）
- 合并两者的最佳实践

3. **创建统一的资源管理器**
```rust
// game_engine/src/resources/manager.rs

pub struct AssetManager<T> {
    // 使用 DashMap
    resources: DashMap<AssetId, Arc<AssetContainer<T>>>,
    // 使用 parking_lot::RwLock 用于其他状态
    metadata: RwLock<HashMap<AssetId, AssetMetadata>>,
}

impl<T: Send + Sync + 'static> AssetManager<T> {
    pub fn new() -> Self {
        Self {
            resources: DashMap::new(),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    // 合并两个实现的最佳方法
    pub fn load(&mut self, id: AssetId, loader: impl Loader<T>) -> Handle<T> {
        // 合并后的实现
    }

    pub async fn load_async(&self, id: AssetId) -> Result<Handle<T>, Error> {
        // 异步加载
    }
}
```

4. **迁移所有使用点**
- [ ] 更新 `game_engine/src/core/engine/asset_processor.rs`
- [ ] 更新其他使用管理器的模块

5. **删除旧文件**
```bash
git rm game_engine/src/resources/optimized_manager.rs
git rm game_engine/src/resources/dashmap_optimizations.rs
```

**检查清单**:
- [ ] 对比分析两个实现
- [ ] 设计统一接口
- [ ] 实现新的资源管理器
- [ ] 迁移所有使用点
- [ ] 删除旧文件
- [ ] 运行测试
- [ ] 更新文档

**预期结果**:
- ✅ 单一资源管理器实现
- ✅ 减少 30% 重复代码
- ✅ 更易维护

---

#### P0-2-2: 统一物理组件接口

**优先级**: 🟡 P1
**预估时间**: 2-3天
**依赖**: P0-2-1

**问题分析**:
物理组件存在多版本并存，接口不统一。

**任务步骤**:

1. **识别所有物理组件**
```bash
find game_engine/src/physics -name "*.rs" | xargs grep -l "Component"
```

2. **设计统一接口**
```rust
// game_engine/src/physics/components.rs

/// 统一的物理组件接口
pub trait PhysicsComponent: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// 刚体组件
#[derive(Component)]
pub struct RigidBody {
    pub handle: RapierHandle,
    pub velocity: Vec3,
    pub mass: f32,
    // ... 其他字段
}

impl RigidBody {
    // 行为方法（见 P1-1-2）
    pub fn apply_force(&mut self, force: Vec3) { /* ... */ }
    pub fn apply_impulse(&mut self, impulse: Vec3) { /* ... */ }
}
```

3. **迁移现有代码**
- [ ] 更新 `RigidBodyComponent` → `RigidBody`
- [ ] 更新 `ColliderComponent` → `Collider`
- [ ] 更新其他物理组件

**检查清单**:
- [ ] 识别所有物理组件
- [ ] 设计统一接口
- [ ] 迁移现有代码
- [ ] 删除旧组件
- [ ] 运行测试
- [ ] 更新文档

---

#### P0-2-3: 清理中间产物文件

**优先级**: 🟢 P2
**预估时间**: 1天
**依赖**: P0-2-1

**任务步骤**:

1. **移动示例代码**
```bash
# 创建 examples 目录
mkdir -p game_engine/examples/async

# 移动文件
mv game_engine/src/async_optimization.rs game_engine/examples/async/
```

2. **移动指南文档**
```bash
# 创建指南目录
mkdir -p game_engine/docs/guides

# 移动文件
mv game_engine/src/concurrency/lock_optimization_guide.rs game_engine/docs/guides/
# 转换为 .md 文档
```

3. **清理归档文档**
```bash
# 归档到独立位置或删除
# 建议: 保留重要文档，删除重复的

# 清理 archive 目录
rm -rf game_engine/docs/archive/legacy_reports/
rm -rf game_engine/docs/archive/optimization/
rm -rf game_engine/docs/archive/progress_reports/
```

4. **更新引用**
- [ ] 更新 `game_engine/src/lib.rs`
- [ ] 更新文档中的链接
- [ ] 更新 README

**检查清单**:
- [ ] 移动 async_optimization.rs
- [ ] 移动 lock_optimization_guide.rs
- [ ] 清理 docs/archive/ 目录
- [ ] 更新所有引用
- [ ] 运行 `cargo build` 验证

---

### P0-3: 异步优化 (4周)

#### P0-3-1: 识别过度异步化

**优先级**: 🔴 P0
**预估时间**: 3-5天
**依赖**: P0-2

**任务步骤**:

1. **扫描所有异步函数**
```bash
# 查找所有 pub async fn
grep -r "pub async fn" game_engine/src/ | wc -l  # 144个

# 输出到文件
grep -r "pub async fn" game_engine/src/ > async_functions.txt
```

2. **分析每个异步函数**
创建分析脚本：
```python
# analyze_async.py
import re
from pathlib import Path

async_patterns = {
    'pure_calculation': r'(calculate|compute|process).*\{(?!.*await)',
    'simple_query': r'(get|query|find).*\{(?!.*await)',
    'io_operation': r'.*(load|save|read|write|network).*',
}

for file in Path('game_engine/src').rglob('*.rs'):
    content = file.read_text()
    # 分析并分类异步函数
```

3. **生成优化清单**
```
# 需要优化的异步函数分类

纯计算（应改为同步）:
- calculate_physics
- vector_add
- calculate_distance
- ...

简单查询（应改为同步）:
- get_entity_count
- query_entity_state
- ...

批量操作（应使用 Rayon）:
- batch_process_entities
- parallel_physics_update
- ...

网络 I/O（保持异步）:
- network_send
- receive_packet
- ...
```

4. **创建优化报告**
```markdown
# 异步优化分析报告

## 统计
- 总异步函数: 144
- 需要优化: ~57 (40%)
  - 纯计算: 25
  - 简单查询: 20
  - 批量操作: 12

## 优化列表
（详细列表）
```

**检查清单**:
- [ ] 扫描所有异步函数
- [ ] 分析每个函数的必要性
- [ ] 分类：纯计算、简单查询、I/O
- [ ] 生成优化清单
- [ ] 创建优化报告

---

#### P0-3-2: 重构纯计算为同步函数

**优先级**: 🔴 P0
**预估时间**: 5-7天
**依赖**: P0-3-1

**任务步骤**:

1. **物理计算重构**
```rust
// ❌ 之前
pub async fn calculate_physics(
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    delta_time: f32,
) -> (f32, f32, f32) {
    let new_x = position.0 + velocity.0 * delta_time;
    let new_y = position.1 + velocity.1 * delta_time;
    let new_z = position.2 + velocity.2 * delta_time;
    (new_x, new_y, new_z)
}

// ✅ 之后
pub fn calculate_physics(
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    delta_time: f32,
) -> (f32, f32, f32) {
    let new_x = position.0 + velocity.0 * delta_time;
    let new_y = position.1 + velocity.1 * delta_time;
    let new_z = position.2 + velocity.2 * delta_time;
    (new_x, new_y, new_z)
}
```

2. **向量运算重构**
```rust
// ❌ 之前
pub async fn vector_add(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]]
}

// ✅ 之后
pub fn vector_add(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]]
}
```

3. **状态查询重构**
```rust
// ❌ 之前
pub async fn get_entity_count(entities: &[u32]) -> usize {
    entities.len()
}

// ✅ 之后
pub fn get_entity_count(entities: &[u32]) -> usize {
    entities.len()
}
```

4. **更新所有调用点**
```bash
# 找出所有调用点
grep -r "calculate_physics(" game_engine/src/
# 更新调用点，移除 .await
```

**需要重构的函数**:
- [ ] 物理计算函数 (~25个)
- [ ] 向量运算函数 (~10个)
- [ ] 状态查询函数 (~20个)

**检查清单**:
- [ ] 重构物理计算函数
- [ ] 重构向量运算函数
- [ ] 重构状态查询函数
- [ ] 更新所有调用点
- [ ] 运行测试

**预期结果**:
- ✅ 重构 50+ 个异步函数
- ✅ 消除不必要的 async 开销
- ✅ 5x-10x 性能提升

---

#### P0-3-3: 使用 Rayon 并行化批量操作

**优先级**: 🟡 P1
**预估时间**: 3-5天
**依赖**: P0-3-2

**任务步骤**:

1. **识别可并行化的批量操作**
```rust
// 候选函数
async fn batch_process_entities(entities: &mut [Entity])
async fn parallel_physics_update(world: &mut World)
async fn batch_transform_update(transforms: &mut [Transform])
```

2. **使用 Rayon 重构**
```rust
// ❌ 之前
pub async fn batch_process_entities(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
) {
    for entity in entities.iter_mut() {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    }
}

// ✅ 之后
pub fn batch_process_entities(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
) {
    use rayon::prelude::*;
    entities.par_iter_mut().for_each(|entity| {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    });
}
```

3. **启用 Rayon 特性**
```toml
# Cargo.toml
[features]
default = ["rayon"]
rayon = ["dep:rayon"]  # rayon 已在依赖中
```

**检查清单**:
- [ ] 识别批量操作函数
- [ ] 使用 Rayon 重构
- [ ] 性能测试
- [ ] 更新文档

**预期结果**:
- ✅ 2-4x 性能提升（取决于核心数）
- ✅ 更好的 CPU 利用率

---

#### P0-3-4: 性能基准测试

**优先级**: 🔴 P0
**预估时间**: 3-5天
**依赖**: P0-3-2, P0-3-3

**任务步骤**:

1. **创建基准测试套件**
```rust
// game_engine/benches/performance.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_physics_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_calculation");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let positions: Vec<(f32, f32, f32)> = (0..size).map(|_| (0.0, 0.0, 0.0)).collect();
            let velocities: Vec<(f32, f32, f32)> = (0..size).map(|_| (1.0, 2.0, 3.0)).collect();

            b.iter(|| {
                for (pos, vel) in positions.iter().zip(velocities.iter()) {
                    let _ = calculate_physics(*pos, *vel, 0.016);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_vector_operations(c: &mut Criterion) {
    c.bench_function("vector_add", |b| {
        let v1 = [1.0, 2.0, 3.0];
        let v2 = [4.0, 5.0, 6.0];
        b.iter(|| vector_add(black_box(v1), black_box(v2)));
    });
}

fn benchmark_rayon_vs_sequential(c: &mut Criterion) {
    // 对比 Rayon 和顺序执行的性能
}

criterion_group!(
    benches,
    benchmark_physics_calculation,
    benchmark_vector_operations,
    benchmark_rayon_vs_sequential
);
criterion_main!(benches);
```

2. **配置 Criterion**
```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "performance"
harness = false
```

3. **运行基准测试**
```bash
# 运行基准测试
cargo bench --bench performance

# 生成报告
cargo bench --bench performance -- --save-baseline main
```

4. **对比优化前后性能**
```bash
# 优化前
cargo bench --bench performance -- --save-baseline before

# 优化后
cargo bench --bench performance -- --save-baseline after

# 对比
cargo bench --bench performance -- --baseline main
```

**检查清单**:
- [ ] 创建基准测试套件
- [ ] 配置 Criterion
- [ ] 运行优化前基准
- [ ] 运行优化后基准
- [ ] 生成性能对比报告
- [ ] 验证性能提升

**预期结果**:
- ✅ 物理计算: 10x 提升
- ✅ 向量运算: 10x 提升
- ✅ 批量操作: 2-4x 提升

---

## P1: 短期执行任务

### P1-1: 消除贫血模型

#### P1-1-1: 重构 Transform 组件

**优先级**: 🟡 P1
**预估时间**: 3-4天
**依赖**: P0-3

**任务步骤**:

```rust
// game_engine/src/physics/transform.rs

use glam::{Vec3, Quat, Mat4};

#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// 创建新变换
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }

    /// 创建单位变换
    pub fn identity() -> Self {
        Self::default()
    }

    /// 平移
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    /// 旋转（欧拉角）
    pub fn rotate_euler(&mut self, euler: Vec3) {
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);
        self.rotation = rotation * self.rotation;
    }

    /// 绕轴旋转
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis, angle);
        self.rotation = rotation * self.rotation;
    }

    /// 缩放
    pub fn scale_by(&mut self, scale: Vec3) {
        self.scale *= scale;
    }

    /// 设置缩放
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// Look at
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);

        // 构建旋转矩阵
        let rotation = Mat3::from_cols(right, up, -forward);
        self.rotation = Quat::from_mat3(&rotation);
    }

    /// 获取前方向量
    pub fn forward(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::NEG_Z)
    }

    /// 获取右方向量
    pub fn right(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::X)
    }

    /// 获取上方向量
    pub fn up(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::Y)
    }

    /// 获取变换矩阵
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.position,
        )
    }

    /// 获取逆矩阵
    pub fn inverse_matrix(&self) -> Mat4 {
        self.matrix().inverse()
    }

    /// 组合变换
    pub fn combine(&self, other: &Transform) -> Transform {
        Transform {
            position: self.position + self.rotation.mul_vec3(other.position * self.scale),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }
}
```

**测试**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let mut t = Transform::default();
        t.translate(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_look_at() {
        let mut t = Transform::default();
        t.look_at(Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        assert!((t.forward() - Vec3::NEG_Z).length() < 0.001);
    }

    #[test]
    fn test_combine() {
        let t1 = Transform::new(Vec3::X, Quat::IDENTITY, Vec3::ONE);
        let t2 = Transform::new(Vec3::Y, Quat::IDENTITY, Vec3::ONE);
        let t3 = t1.combine(&t2);
        assert_eq!(t3.position, Vec3::new(2.0, 1.0, 0.0));
    }
}
```

**检查清单**:
- [ ] 实现 Transform 方法
- [ ] 编写单元测试
- [ ] 更新所有使用点
- [ ] 运行测试
- [ ] 更新文档

---

#### P1-1-2: 重构 RigidBody 组件

**优先级**: 🟡 P1
**预估时间**: 4-5天
**依赖**: P1-1-1

```rust
// game_engine/src/physics/rigid_body.rs

use glam::Vec3;

#[derive(Component, Debug)]
pub struct RigidBody {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub inverse_mass: f32,
    pub inertia: f32,
    pub inverse_inertia: f32,
    pub is_static: bool,
    pub is_kinematic: bool,
}

impl RigidBody {
    /// 创建动态刚体
    pub fn dynamic(mass: f32) -> Self {
        assert!(mass > 0.0, "Mass must be positive");
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass,
            inverse_mass: 1.0 / mass,
            inertia: 1.0,
            inverse_inertia: 1.0,
            is_static: false,
            is_kinematic: false,
        }
    }

    /// 创建静态刚体
    pub fn static_body() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: f32::INFINITY,
            inverse_mass: 0.0,
            inertia: f32::INFINITY,
            inverse_inertia: 0.0,
            is_static: true,
            is_kinematic: false,
        }
    }

    /// 创建运动学刚体
    pub fn kinematic() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: f32::INFINITY,
            inverse_mass: 0.0,
            inertia: f32::INFINITY,
            inverse_inertia: 0.0,
            is_static: false,
            is_kinematic: true,
        }
    }

    /// 施加力（F = ma -> a = F/m）
    pub fn apply_force(&mut self, force: Vec3) {
        if self.is_static || self.is_kinematic {
            return;
        }
        let acceleration = force * self.inverse_mass;
        self.velocity += acceleration;
    }

    /// 施加冲量（瞬时改变速度）
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.is_static || self.is_kinematic {
            return;
        }
        self.velocity += impulse * self.inverse_mass;
    }

    /// 施加扭矩
    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.is_static || self.is_kinematic {
            return;
        }
        let angular_acceleration = torque * self.inverse_inertia;
        self.angular_velocity += angular_acceleration;
    }

    /// 设置速度
    pub fn set_velocity(&mut self, velocity: Vec3) {
        if !self.is_static {
            self.velocity = velocity;
        }
    }

    /// 设置角速度
    pub fn set_angular_velocity(&mut self, angular_velocity: Vec3) {
        if !self.is_static {
            self.angular_velocity = angular_velocity;
        }
    }

    /// 获取动量（p = mv）
    pub fn momentum(&self) -> Vec3 {
        self.velocity * self.mass
    }

    /// 获取动能（E = 0.5 * m * v²）
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    /// 阻尼
    pub fn apply_damping(&mut self, linear_damping: f32, angular_damping: f32) {
        if !self.is_static && !self.is_kinematic {
            self.velocity *= (1.0 - linear_damping);
            self.angular_velocity *= (1.0 - angular_damping);
        }
    }
}
```

**检查清单**:
- [ ] 实现 RigidBody 方法
- [ ] 编写单元测试
- [ ] 更新所有使用点
- [ ] 运行测试
- [ ] 更新文档

---

### P1-2: 建立分层架构

详细任务步骤请参见 PARALLEL_IMPLEMENTATION_PLAN.md。

---

### P1-3: 增强测试覆盖率

详细任务步骤请参见 PARALLEL_IMPLEMENTATION_PLAN.md。

---

## P2: 中长期执行任务

### P2-1: 实现CQRS模式

详细任务步骤请参见 PARALLEL_IMPLEMENTATION_PLAN.md。

---

### P2-2: 完善事件溯源系统

详细任务步骤请参见 PARALLEL_IMPLEMENTATION_PLAN.md。

---

### P2-3: 实现网络复制系统

详细任务步骤请参见 PARALLEL_IMPLEMENTATION_PLAN.md。

---

## 依赖关系图

```
P0-0 依赖升级
    ↓
P0-1 条件编译清理 ─────────┐
    ↓                      │
P0-2 代码去重              │
    ↓                      │
P0-3 异步优化 ─────────────┤
    ↓                      │
    ├──→ P1-3 测试增强 ←───┤
    ↓                      │
P1-1 消除贫血模型          │ (并行)
    ↓                      │
P1-2 分层架构 ←────────────┘
    ↓
P2-1 CQRS模式
    ↓
P2-2 事件溯源
    ↓
P2-3 网络复制系统
```

---

## 执行检查表

### 开始前检查

- [ ] 已阅读 PARALLEL_IMPLEMENTATION_PLAN.md
- [ ] 已阅读 SYSTEM_COMPREHENSIVE_REVIEW_REPORT.md
- [ ] 已安装 Rust 1.85.0+
- [ ] 已配置开发环境
- [ ] 已设置 Git 分支策略

### P0 阶段完成检查

- [ ] P0-0: 依赖升级完成
- [ ] P0-1: 条件编译清理完成（减少 70%）
- [ ] P0-2: 代码去重完成（减少 30%）
- [ ] P0-3: 异步优化完成（5x-10x 性能提升）
- [ ] 所有测试通过
- [ ] 性能基准测试通过

### P1 阶段完成检查

- [ ] P1-1: 贫血模型消除完成
- [ ] P1-2: 分层架构建立完成
- [ ] P1-3: 测试覆盖率 >80%
- [ ] 代码审查通过
- [ ] 文档更新完成

### P2 阶段完成检查

- [ ] P2-1: CQRS 模式实现完成
- [ ] P2-2: 事件溯源完善完成
- [ ] P2-3: 网络复制系统实现完成
- [ ] 生产就绪版本发布

---

**文档版本**: v1.0
**最后更新**: 2025-12-30
**相关文档**:
- PARALLEL_IMPLEMENTATION_PLAN.md
- SYSTEM_COMPREHENSIVE_REVIEW_REPORT.md
