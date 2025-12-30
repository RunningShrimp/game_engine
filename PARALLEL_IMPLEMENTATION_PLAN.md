# 游戏引擎并行开发实施计划 v1.0

**项目**: Rust 高性能通用游戏引擎
**计划日期**: 2025-12-30
**当前版本**: v0.4.0
**目标版本**: v0.5.0 (生产就绪)
**计划周期**: 12个月（2025-12-30 至 2026-12-30）

---

## 目录

1. [执行摘要](#执行摘要)
2. [技术栈升级](#技术栈升级)
3. [并行开发策略](#并行开发策略)
4. [阶段性实施计划](#阶段性实施计划)
5. [详细任务分解](#详细任务分解)
6. [依赖关系图](#依赖关系图)
7. [风险管理](#风险管理)
8. [质量保证](#质量保证)

---

## 执行摘要

### 项目背景

根据《系统全面审查报告》，游戏引擎在功能完整性（8.5/10）、性能优化（9.0/10）方面表现优秀，但在可维护性（6.5/10）和架构实践（7.5/10）方面存在改进空间。

### 核心目标

| 维度 | 当前评分 | 目标评分 | 改进幅度 |
|------|---------|---------|---------|
| 功能完整性 | 8.5/10 | 9.5/10 | +12% |
| 性能优化 | 9.0/10 | 9.5/10 | +6% |
| 可维护性 | 6.5/10 | 9.0/10 | +38% |
| 架构实践 | 7.5/10 | 9.0/10 | +20% |

### 关键指标

- **条件编译**: 从 170+ 处减少至 <50 处（-70%）
- **代码重复**: 减少 30% 重复代码
- **异步优化**: 重构 144 个异步函数中的 40%（~57个）
- **测试覆盖**: 从未知提升至 >80%
- **依赖版本**: 升级至 Rust 2024 edition 和最新稳定版本

---

## 技术栈升级

### Rust Edition 升级

| 项目 | 当前版本 | 目标版本 | 状态 |
|------|---------|---------|------|
| **Rust Edition** | 2024 | 2024 | ✅ 已使用 |
| **Rust Toolchain** | 未知 | 1.85.0+ | ⚠️ 需确认 |

**Rust 2024 Edition 新特性**:
- ✅ 异步闭包 (`async || {}`)
- ✅ `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` traits
- ✅ 元组的 `FromIterator` 和 `Extend` (支持 1-12 元素)
- ✅ `#[diagnostic::do_not_recommend]` 属性
- ✅ 改进的 RPIT 生命周期捕获
- ✅ 调整的临时变量作用域
- ✅ Future 和 IntoFuture 加入 prelude

### 依赖版本升级计划

#### Workspace 核心依赖升级

| 依赖 | 当前版本 | 最新版本 | 升级优先级 | 破坏性变更 |
|------|---------|---------|-----------|-----------|
| `serde` | 1.0.228 | 1.0.228 | ✅ 最新 | 无 |
| `serde_json` | 1.0.145 | 1.0.145 | ✅ 最新 | 无 |
| `thiserror` | 2.0 | 2.0 | ✅ 最新 | 无 |
| `log` | 0.4.29 | 0.4.29 | ✅ 最新 | 无 |
| `tracing` | 0.1.44 | 0.1.44 | ✅ 最新 | 无 |
| `bevy_ecs` | 0.17.3 | 0.17.3 | ✅ 最新 | 无 |
| `glam` | 0.30 | 0.30 | ✅ 最新 | 无 |
| `tokio` | 1.48 | 1.48 | ✅ 最新 | 无 |
| `parking_lot` | 0.12.5 | 0.12.5 | ✅ 最新 | 无 |
| `wgpu` | 27.0.1 | 27.0.1 | ✅ 最新 | 无 |
| `futures` | 0.3.31 | 0.3.31 | ✅ 最新 | 无 |
| `flate2` | 1.1.5 | 1.1.5 | ✅ 最新 | 无 |
| `uuid` | 1.19 | 1.19 | ✅ 最新 | 无 |
| `dashmap` | 6.1 | 6.1 | ✅ 最新 | 无 |
| `base64` | 0.22 | 0.22 | ✅ 最新 | 无 |
| `getrandom` | 0.3 | 0.3 | ✅ 最新 | 无 |
| `hashbrown` | 0.16 | 0.16 | ✅ 最新 | 无 |
| `digest` | 0.10 | 0.10 | ✅ 最新 | 无 |
| `tower-http` | 0.6 | 0.6 | ✅ 最新 | 无 |
| `rand` | 0.9 | 0.9 | ✅ 最新 | 无 |

**评估**: 所有 workspace 依赖已使用最新版本 ✅

#### 游戏引擎依赖升级

| 依赖 | 当前版本 | 最新版本 | 升级优先级 | 破坏性变更 |
|------|---------|---------|-----------|-----------|
| `winit` | 0.30 | 0.30 | ✅ 最新 | 无 |
| `pollster` | 0.4 | 0.4 | ✅ 最新 | 无 |
| `bytemuck` | 1.24 | 1.24 | ✅ 最新 | 无 |
| `bincode` | 1.3 | 2.0 | ⚠️ 推荐 | 有 |
| `image` | 0.25 | 0.25 | ✅ 最新 | 无 |
| `notify` | 8.2 | 8.2 | ✅ 最新 | 无 |
| `sha2` | 0.10 | 0.10 | ✅ 最新 | 无 |
| `hex` | 0.4 | 0.4 | ⚠️ 旧 | 有 |
| `hmac` | 0.12 | 0.12 | ✅ 最新 | 无 |
| `aes-gcm` | 0.10 | 0.10 | ✅ 最新 | 无 |
| `tokio-util` | 0.7.17 | 0.7.17 | ✅ 最新 | 无 |
| `async-trait` | 0.1.89 | 0.1.89 | ✅ 最新 | 无 |
| `crossbeam-channel` | 0.5.15 | 0.5.15 | ✅ 最新 | 无 |
| `libloading` | 0.9.0 | 0.9.0 | ✅ 最新 | 无 |
| `rayon` | 1.11.0 | 1.11.0 | ✅ 最新 | 无 |
| `rapier2d` | 0.31 | 0.31 | ✅ 最新 | 无 |
| `rapier3d` | 0.31 | 0.31 | ✅ 最新 | 无 |
| `gltf` | 1.4.1 | 1.4.1 | ✅ 最新 | 无 |
| `openxr` | 0.20.0 | 0.20.0 | ✅ 最新 | 无 |
| `egui` | 0.33.3 | 0.33.3 | ✅ 最新 | 无 |
| `egui-wgpu` | 0.33.3 | 0.33.3 | ✅ 最新 | 无 |
| `egui-winit` | 0.33.3 | 0.33.3 | ✅ 最新 | 无 |
| `rquickjs` | 0.11.0 | 0.11.0 | ✅ 最新 | 无 |
| `tracing-subscriber` | 0.3.22 | 0.3.22 | ✅ 最新 | 无 |
| `rodio` | 0.21.1 | 0.21.1 | ✅ 最新 | 无 |
| `raw-window-handle` | 0.6.2 | 0.6.2 | ✅ 最新 | 无 |

**需要升级的依赖**:
1. **bincode**: 1.3 → 2.0 (破坏性变更，需要API调整)
2. **hex**: 0.4 → 0.5 (小版本更新)

---

## 并行开发策略

### 并行开发原则

1. **独立性优先**: 将任务分解为相互独立的模块
2. **接口契约**: 通过明确的接口隔离开发团队
3. **特性分支**: 每个任务使用独立的特性分支
4. **持续集成**: 每日合并到主分支的集成分支

### 并行任务分组

#### A组：基础设施重构（可并行）
- A1: 条件编译清理（P0-1）
- A2: 代码去重（P0-2）
- A3: 依赖升级（P0-0）

#### B组：性能优化（可并行）
- B1: 异步优化（P0-3）
- B2: 资源管理优化（P0-4）

#### C组：架构重构（依赖A组）
- C1: 消除贫血模型（P1-1）
- C2: 建立分层架构（P1-2）
- C3: CQRS模式（P2-1）

#### D组：质量保证（可并行）
- D1: 测试增强（P1-3）
- D2: CI/CD建立（P1-4）

#### E组：功能增强（依赖C组）
- E1: 事件溯源完善（P2-2）
- E2: 网络复制系统（P2-3）

### 并行开发时间线

```
月份  1  2  3  4  5  6  7  8  9  10 11 12
A组  ████████
B组     ████████
C组        ████████
D        ████████
E组           ████████████████
```

---

## 阶段性实施计划

### 第一阶段：基础设施重构（第1-2个月）

#### 目标
- 清理技术债务
- 统一代码风格
- 提升可维护性

#### 任务分解

**A1: 条件编译清理** (2周)
- 统一资源管理器实现（DashMap）
- 移除无意义的条件分支
- 更新文档说明特性选择

**A2: 代码去重** (2周)
- 合并 manager.rs 和 optimized_manager.rs
- 统一物理组件接口
- 清理中间产物文件

**A3: 依赖升级** (2周)
- 升级 bincode 到 2.0
- 升级 hex 到 0.5
- 测试所有依赖兼容性

**A4: 中间产物文件清理** (1周)
- 移动示例代码到 examples/
- 移动指南文档到 docs/guides/
- 清理 archive 文件

**A5: 文档更新** (1周)
- 更新 ADR 文档
- 更新 API 文档
- 更新迁移指南

#### 交付物
- ✅ 减少 70% 条件编译
- ✅ 减少 30% 代码重复
- ✅ 完整的依赖版本升级
- ✅ 更新的文档系统

### 第二阶段：性能优化（第2-4个月）

#### 目标
- 消除过度异步化
- 提升并发性能
- 优化资源加载

#### 任务分解

**B1: 异步优化** (4周)
- 识别过度异步化的代码
- 重构纯计算为同步函数
- 使用 Rayon 并行化批量操作
- 性能基准测试对比

**B2: 资源管理优化** (3周)
- 实现资源预加载
- 优化流式加载缓冲区
- 实现智能缓存策略

**B3: 渲染优化** (3周)
- GPU粒子系统
- 批次渲染优化
- 着色器优化

#### 交付物
- ✅ 重构 57 个异步函数（40%）
- ✅ 5x-10x 性能提升（特定场景）
- ✅ 减少 30% 资源加载时间

### 第三阶段：架构重构（第3-6个月）

#### 目标
- 建立清晰的分层架构
- 消除贫血模型
- 实现CQRS模式

#### 任务分解

**C1: 消除贫血模型** (6周)
- 为 Transform 添加行为方法
- 为 RigidBody 添加行为方法
- 为其他领域对象添加行为
- 编写单元测试

**C2: 建立分层架构** (6周)
- 引入领域层（domain/）
- 引入应用层（application/）
- 重构基础设施层（infrastructure/）
- 更新模块导入

**C3: CQRS模式实现** (4周)
- 实现 CommandHandler trait
- 实现 QueryHandler trait
- 实现命令总线
- 实现查询总线
- 编写集成测试

#### 交付物
- ✅ 清晰的分层架构
- ✅ 富领域对象
- ✅ CQRS模式实现
- ✅ 完整的测试覆盖

### 第四阶段：质量保证（第4-7个月）

#### 目标
- 建立完整的测试体系
- 实现 CI/CD 流程
- 提升代码质量

#### 任务分解

**D1: 测试增强** (8周)
- 为每个特性组合添加测试
- 添加集成测试
- 添加性能测试
- 添加模糊测试
- 达到 >80% 测试覆盖率

**D2: CI/CD建立** (4周)
- 配置 GitHub Actions
- 自动化测试
- 自动化部署
- 代码质量检查

**D3: 文档完善** (4周)
- API 文档
- 架构文档
- 用户手册
- 开发者指南

#### 交付物
- ✅ >80% 测试覆盖率
- ✅ 完整的 CI/CD 流程
- ✅ 完善的文档系统

### 第五阶段：高级功能（第6-12个月）

#### 目标
- 完善事件溯源
- 实现网络复制
- 增强功能完整性

#### 任务分解

**E1: 事件溯源完善** (8周)
- 实现事件存储
- 实现事件重放
- 实现时间旅行
- 实现快照机制

**E2: 网络复制系统** (12周)
- 实现服务器权威复制
- 实现客户端预测
- 实现服务器调解
- 实现插值和外推

**E3: 其他高级功能** (16周)
- 保存/加载系统
- 本地化系统
- 粒子系统增强
- 后处理效果扩展

#### 交付物
- ✅ 完整的事件溯源系统
- ✅ 网络复制系统
- ✅ 生产就绪的游戏引擎

---

## 详细任务分解

### P0-0: 依赖升级（立即执行）

#### 任务描述
升级所有依赖到最新稳定版本，确保与 Rust 2024 edition 兼容。

#### 子任务

**P0-0-1: 升级 bincode 到 2.0**
```toml
# Cargo.toml
bincode = { version = "2.0", features = ["serde"] }
```
- [ ] 更新 Cargo.toml
- [ ] 修改 API 调用（bincode 2.0 API 变更）
- [ ] 运行测试验证
- [ ] 更新文档

**P0-0-2: 升级 hex 到 0.5**
```toml
# Cargo.toml
hex = "0.5"
```
- [ ] 更新 Cargo.toml
- [ ] 修改 API 调用（如有破坏性变更）
- [ ] 运行测试验证

**P0-0-3: 验证 Rust 2024 兼容性**
- [ ] 确认使用 edition = "2024"
- [ ] 测试异步闭包特性
- [ ] 测试新的 AsyncFn traits
- [ ] 更新代码以使用新特性

#### 预期时间
1-2天

#### 依赖关系
无

---

### P0-1: 条件编译清理（第1-2周）

#### 任务描述
减少 70% 的条件编译使用，统一实现路径。

#### 子任务

**P0-1-1: 统一资源管理器实现**
```rust
// 在 src/resources/mod.rs 中统一
pub type AssetManager<T> = DashMapAssetManager<T>;

// 移除所有 #[cfg(feature = "dashmap")] 条件编译
```
- [ ] 分析当前条件编译使用情况
- [ ] 选择默认实现（推荐 DashMap）
- [ ] 更新所有使用点
- [ ] 移除条件编译
- [ ] 更新文档

**P0-1-2: 简化密钥交换实现**
- [ ] 统一为安全密钥交换（ECDH + HKDF）
- [ ] 移除不安全实现
- [ ] 更新文档说明

**P0-1-3: 简化 GLTF 加载器**
- [ ] 移除 GLTF 相关的条件编译
- [ ] 使用可选依赖机制
- [ ] 更新文档

**P0-1-4: 简化 Tracy 集成**
- [ ] 统一 Tracy 接口
- [ ] 使用条件编译仅在 feature 启用时编译
- [ ] 更新文档

#### 预期时间
2周

#### 依赖关系
无

#### 影响范围
- 31个文件
- 170+ 处条件编译

---

### P0-2: 代码去重（第3-4周）

#### 任务描述
合并重复的代码实现，减少维护负担。

#### 子任务

**P0-2-1: 合并资源管理器**
```rust
// 合并 manager.rs 和 optimized_manager.rs
pub struct AssetManager<T> {
    // 使用 DashMap 作为默认实现
    resources: DashMap<AssetId, Arc<AssetContainer<T>>>,
}
```
- [ ] 分析两个管理器的差异
- [ ] 选择最佳实现
- [ ] 合并代码
- [ ] 更新所有使用点
- [ ] 删除旧文件
- [ ] 运行测试

**P0-2-2: 统一物理组件接口**
- [ ] 分析物理组件版本差异
- [ ] 设计统一接口
- [ ] 实现统一接口
- [ ] 迁移现有代码
- [ ] 删除旧实现

**P0-2-3: 清理中间产物文件**
- [ ] 移动 async_optimization.rs 到 examples/
- [ ] 移动 lock_optimization_guide.rs 到 docs/guides/
- [ ] 移动 dashmap_optimizations.rs（合并到主管理器）
- [ ] 清理 docs/archive/ 目录
- [ ] 更新所有引用

#### 预期时间
2周

#### 依赖关系
依赖 P0-1

---

### P0-3: 异步优化（第5-8周）

#### 任务描述
识别并重构过度异步化的代码，提升性能。

#### 子任务

**P0-3-1: 识别过度异步化**
- [ ] 扫描所有 async fn
- [ ] 分析每个异步函数的必要性
- [ ] 分类：纯计算、简单查询、I/O操作
- [ ] 生成优化清单

**P0-3-2: 重构纯计算为同步**
```rust
// 示例
// ❌ 之前
pub async fn calculate_physics(...) -> (f32, f32, f32)

// ✅ 之后
pub fn calculate_physics(...) -> (f32, f32, f32)
```
- [ ] 重构物理计算函数
- [ ] 重构向量运算函数
- [ ] 重构状态查询函数
- [ ] 更新所有调用点
- [ ] 性能测试

**P0-3-3: 使用 Rayon 并行化批量操作**
```rust
// 示例
use rayon::prelude::*;
entities.par_iter_mut().for_each(|entity| {
    // 处理实体
});
```
- [ ] 识别可并行化的批量操作
- [ ] 实现 Rayon 并行版本
- [ ] 性能测试
- [ ] 文档更新

**P0-3-4: 性能基准测试**
- [ ] 建立基准测试套件
- [ ] 测试优化前后性能
- [ ] 生成性能报告
- [ ] 验证 5x-10x 性能提升

#### 预期时间
4周

#### 依赖关系
依赖 P0-2

#### 预期成果
- 重构 57 个异步函数（40%）
- 5x-10x 性能提升（特定场景）

---

### P1-1: 消除贫血模型（第9-14周）

#### 任务描述
为领域对象添加行为方法，实现富领域对象。

#### 子任务

**P1-1-1: 重构 Transform 组件**
```rust
impl Transform {
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    pub fn rotate(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis, angle);
        self.rotation = rotation * self.rotation;
    }

    pub fn look_at(&mut self, target: Vec3) {
        let forward = (target - self.position).normalize();
        // 实现look_at逻辑
    }

    pub fn matrix(&self) -> Mat4 {
        // 返回变换矩阵
    }
}
```
- [ ] 分析当前 Transform 实现
- [ ] 设计行为方法
- [ ] 实现行为方法
- [ ] 更新所有使用点
- [ ] 添加单元测试

**P1-1-2: 重构 RigidBody 组件**
```rust
impl RigidBodyComponent {
    pub fn apply_force(&mut self, force: Vec3) {
        // F = ma -> a = F/m
        let acceleration = force / self.mass;
        self.velocity += acceleration;
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        // 瞬时改变速度
        self.velocity += impulse / self.mass;
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }
}
```
- [ ] 分析当前 RigidBody 实现
- [ ] 设计行为方法
- [ ] 实现行为方法
- [ ] 更新所有使用点
- [ ] 添加单元测试

**P1-1-3: 重构其他领域对象**
- [ ] 识别其他贫血对象
- [ ] 为每个对象添加行为方法
- [ ] 更新使用点
- [ ] 添加测试

#### 预期时间
6周

#### 依赖关系
依赖 P0-3

---

### P1-2: 建立分层架构（第15-20周）

#### 任务描述
建立清晰的分层架构：领域层、应用层、基础设施层。

#### 子任务

**P1-2-1: 创建领域层**
```
src/domain/
├── physics/
│   ├── mod.rs
│   ├── rigid_body.rs      # 富领域对象
│   ├── collider.rs
│   └── world.rs
├── audio/
│   ├── mod.rs
│   ├── source.rs          # 富领域对象
│   └── listener.rs
├── rendering/
│   ├── mod.rs
│   ├── mesh.rs            # 富领域对象
│   └── material.rs
└── mod.rs
```
- [ ] 设计领域层结构
- [ ] 创建领域对象
- [ ] 实现领域服务
- [ ] 添加领域事件

**P1-2-2: 创建应用层**
```
src/application/
├── systems/
│   ├── mod.rs
│   ├── physics_system.rs
│   ├── rendering_system.rs
│   └── audio_system.rs
├── services/
│   ├── mod.rs
│   ├── asset_service.rs
│   └── scene_service.rs
└── mod.rs
```
- [ ] 设计应用层结构
- [ ] 创建 ECS 系统
- [ ] 创建应用服务
- [ ] 实现命令/查询处理

**P1-2-3: 重构基础设施层**
```
src/infrastructure/
├── render/
│   ├── wgpu_backend.rs
│   └── pipelines/
├── resources/
│   ├── manager.rs
│   └── cache.rs
└── network/
    ├── client.rs
    └── server.rs
```
- [ ] 重构现有代码到基础设施层
- [ ] 实现基础设施服务
- [ ] 更新模块导入

**P1-2-4: 更新模块导入**
- [ ] 更新所有使用点的导入
- [ ] 更新文档
- [ ] 运行测试

#### 预期时间
6周

#### 依赖关系
依赖 P1-1

---

### P1-3: 增强测试覆盖率（第17-24周）

#### 任务描述
建立完整的测试体系，达到 >80% 测试覆盖率。

#### 子任务

**P1-3-1: 单元测试**
- [ ] 为所有领域对象添加单元测试
- [ ] 为所有服务添加单元测试
- [ ] 为所有工具函数添加单元测试

**P1-3-2: 集成测试**
```
tests/
├── integration/
│   ├── physics_tests.rs
│   ├── rendering_tests.rs
│   └── network_tests.rs
└── mod.rs
```
- [ ] 设计集成测试结构
- [ ] 实现系统集成测试
- [ ] 实现端到端测试

**P1-3-3: 性能测试**
```
benches/
├── physics_bench.rs
├── rendering_bench.rs
└── resource_bench.rs
```
- [ ] 设计性能测试结构
- [ ] 实现性能基准测试
- [ ] 建立性能回归测试

**P1-3-4: 特性组合测试**
- [ ] 测试所有特性组合
- [ ] 测试条件编译路径
- [ ] 测试可选依赖

**P1-3-5: 测试覆盖率报告**
- [ ] 配置 tarpaulin 或 cargo-llvm-cov
- [ ] 生成覆盖率报告
- [ ] 验证 >80% 覆盖率

#### 预期时间
8周（并行进行）

#### 依赖关系
依赖 P1-2

---

### P2-1: 实现CQRS模式（第21-24周）

#### 任务描述
实现命令查询职责分离（CQRS）模式。

#### 子任务

**P2-1-1: 定义核心 Trait**
```rust
/// 命令处理器
pub trait CommandHandler<C> {
    fn handle(&mut self, command: C) -> Result<(), Error>;
}

/// 查询处理器
pub trait QueryHandler<Q, R> {
    fn handle(&self, query: Q) -> Result<R, Error>;
}
```
- [ ] 定义 CommandHandler trait
- [ ] 定义 QueryHandler trait
- [ ] 定义错误类型
- [ ] 添加文档

**P2-1-2: 实现命令总线**
```rust
pub struct CommandBus {
    handlers: HashMap<TypeId, Box<dyn CommandHandler<dyn Any>>>,
}

impl CommandBus {
    pub fn register<C>(&mut self, handler: impl CommandHandler<C> + 'static)
    where
        C: Any + 'static,
    {
        // 注册处理器
    }

    pub fn dispatch<C>(&mut self, command: C) -> Result<(), Error>
    where
        C: Any + 'static,
    {
        // 分发命令
    }
}
```
- [ ] 设计命令总线结构
- [ ] 实现命令注册
- [ ] 实现命令分发
- [ ] 添加错误处理

**P2-1-3: 实现查询总线**
```rust
pub struct QueryBus {
    handlers: HashMap<TypeId, Box<dyn QueryHandler<dyn Any, dyn Any>>>,
}

impl QueryBus {
    pub fn register<Q, R>(&mut self, handler: impl QueryHandler<Q, R> + 'static)
    where
        Q: Any + 'static,
        R: Any + 'static,
    {
        // 注册处理器
    }

    pub fn execute<Q, R>(&self, query: Q) -> Result<R, Error>
    where
        Q: Any + 'static,
        R: Any + 'static,
    {
        // 执行查询
    }
}
```
- [ ] 设计查询总线结构
- [ ] 实现查询注册
- [ ] 实现查询执行
- [ ] 添加错误处理

**P2-1-4: 实现示例命令和查询**
```rust
// 示例命令
pub struct CreateEntity {
    pub id: Uuid,
    pub transform: Transform,
}

// 示例查询
pub struct GetEntityPosition {
    pub id: Uuid,
}

pub struct EntityPosition {
    pub position: Vec3,
}
```
- [ ] 实现示例命令处理器
- [ ] 实现示例查询处理器
- [ ] 添加集成测试

#### 预期时间
4周

#### 依赖关系
依赖 P1-2

---

### P2-2: 完善事件溯源系统（第25-32周）

#### 任务描述
完善事件溯源系统，支持事件存储、重放和时间旅行。

#### 子任务

**P2-2-1: 实现事件存储**
```rust
pub trait EventStore {
    fn append(&mut self, aggregate_id: Uuid, events: Vec<DomainEvent>)
        -> Result<(), Error>;

    fn replay(&self, aggregate_id: Uuid, from_version: Option<usize>)
        -> Result<Vec<DomainEvent>, Error>;

    fn snapshot(&self, aggregate_id: Uuid)
        -> Result<Option<AggregateSnapshot>, Error>;

    fn save_snapshot(&mut self, snapshot: AggregateSnapshot)
        -> Result<(), Error>;
}
```
- [ ] 定义 EventStore trait
- [ ] 实现内存事件存储
- [ ] 实现文件事件存储
- [ ] 实现 SQLite 事件存储（可选）

**P2-2-2: 实现事件重放**
```rust
pub trait EventSourcedAggregate {
    type Event;

    fn apply_event(&mut self, event: Self::Event);

    fn replay(events: Vec<Self::Event>) -> Self
    where
        Self: Default,
    {
        let mut aggregate = Self::default();
        for event in events {
            aggregate.apply_event(event);
        }
        aggregate
    }
}
```
- [ ] 定义 EventSourcedAggregate trait
- [ ] 为聚合根实现事件重放
- [ ] 添加重放性能优化（快照）

**P2-2-3: 实现时间旅行**
```rust
pub struct TimeTravelService<E: EventStore> {
    event_store: E,
}

impl<E: EventStore> TimeTravelService<E> {
    pub fn travel_to(&self, aggregate_id: Uuid, timestamp: SystemTime)
        -> Result<Box<dyn Any>, Error> {
        // 获取指定时间点的事件并重放
    }

    pub fn get_history(&self, aggregate_id: Uuid)
        -> Result<Vec<DomainEvent>, Error> {
        // 获取完整事件历史
    }
}
```
- [ ] 设计时间旅行服务
- [ ] 实现时间点状态恢复
- [ ] 实现事件历史查询
- [ ] 添加 UI 工具（可选）

**P2-2-4: 实现快照机制**
```rust
pub struct Snapshot {
    pub aggregate_id: Uuid,
    pub version: usize,
    pub timestamp: SystemTime,
    pub data: Vec<u8>,
}

pub struct SnapshotService {
    snapshots: DashMap<Uuid, Snapshot>,
}
```
- [ ] 设计快照结构
- [ ] 实现快照生成策略
- [ ] 实现快照存储
- [ ] 实现快照恢复

#### 预期时间
8周

#### 依赖关系
依赖 P2-1

---

### P2-3: 实现网络复制系统（第29-48周）

#### 任务描述
实现服务器权威的网络复制系统，支持客户端预测和服务器调解。

#### 子任务

**P2-3-1: 定义复制接口**
```rust
pub trait Replicable: Serialize + DeserializeOwned {
    fn replicate(&self) -> ReplicationData;

    fn apply_replication(&mut self, data: ReplicationData) -> Result<(), Error>;

    fn priority(&self) -> ReplicationPriority {
        ReplicationPriority::Normal
    }
}

pub enum ReplicationPriority {
    High,      // 玩家、重要NPC
    Normal,    // 普通实体
    Low,       // 装饰性对象
}
```
- [ ] 定义 Replicable trait
- [ ] 定义 ReplicationData 结构
- [ ] 实现序列化/反序列化
- [ ] 实现优先级系统

**P2-3-2: 实现服务器权威复制**
```rust
pub struct ReplicationServer {
    world: World,
    clients: DashMap<ClientId, ClientConnection>,
    replication_config: ReplicationConfig,
}

impl ReplicationServer {
    pub fn add_replicable<C: Component + Replicable>(&mut self, entity: Entity, component: C) {
        // 添加可复制组件
    }

    pub fn replicate_to_clients(&mut self) {
        // 向所有客户端复制状态
    }

    pub fn replicate_to_client(&mut self, client_id: ClientId, entity: Entity) {
        // 向特定客户端复制实体
    }
}
```
- [ ] 设计服务器架构
- [ ] 实现实体状态跟踪
- [ ] 实现增量更新
- [ ] 实现优先级调度

**P2-3-3: 实现客户端预测**
```rust
pub struct PredictionClient {
    predicted_state: LocalState,
    server_state: ServerState,
    pending_moves: VecDeque<PlayerMove>,
}

impl PredictionClient {
    pub fn predict_move(&mut self, player_move: PlayerMove) {
        // 预测移动
    }

    pub fn reconcile_with_server(&mut self, server_state: ServerState) {
        // 与服务器状态调解
    }
}
```
- [ ] 设计客户端预测架构
- [ ] 实现移动预测
- [ ] 实现服务器调解
- [ ] 实现回滚机制

**P2-3-4: 实现插值和外推**
```rust
pub struct Interpolation {
    buffer: Vec<TimedState>,
    interpolation_delay: Duration,
}

impl Interpolation {
    pub fn add_state(&mut self, state: TimedState) {
        // 添加状态到缓冲区
    }

    pub fn interpolate(&self, time: SystemTime) -> Option<State> {
        // 插值计算状态
    }
}

pub struct Extrapolation {
    last_state: TimedState,
    velocity: Vec3,
}

impl Extrapolation {
    pub fn extrapolate(&self, time: SystemTime) -> State {
        // 外推计算状态
    }
}
```
- [ ] 实现插值系统
- [ ] 实现外推系统
- [ ] 实现延迟补偿
- [ ] 性能优化

**P2-3-5: 实现带宽优化**
```rust
pub struct DeltaCompression {
    last_state: HashMap<EntityId, EntityState>,
}

impl DeltaCompression {
    pub fn compress(&self, current_state: EntityState) -> Vec<u8> {
        // 计算差异并压缩
    }
}
```
- [ ] 实现增量压缩
- [ ] 实现优先级调度
- [ ] 实现相关更新
- [ ] 带宽测试

#### 预期时间
20周

#### 依赖关系
依赖 P2-2

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

## 风险管理

### 风险识别

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 依赖升级破坏性变更 | 中 | 高 | 充分测试、分步升级 |
| 重构引入新 Bug | 高 | 高 | 完整测试、代码审查 |
| 性能回退 | 中 | 中 | 基准测试、性能监控 |
| 开发延期 | 中 | 中 | 缓冲时间、优先级管理 |
| 人员变动 | 低 | 中 | 知识共享、文档完善 |

### 风险应对策略

**依赖升级风险**
- 在单独分支进行升级测试
- 逐步升级，每次升级后运行测试
- 保留降级方案

**重构风险**
- 使用特性分支隔离更改
- 实施强制性代码审查
- 建立完整的测试体系
- 使用静态分析工具

**性能风险**
- 建立性能基准测试
- 每次重构后运行性能测试
- 使用性能分析工具

**延期风险**
- 为每个任务添加 20% 缓冲时间
- 优先级管理，确保核心功能按时完成
- 定期进度审查

---

## 质量保证

### 代码质量标准

**代码审查清单**
- [ ] 遵循 Rust 命名规范
- [ ] 没有编译警告（#![deny(warnings)]）
- [ ] 通过 clippy 检查（cargo clippy）
- [ ] 通过 fmt 检查（cargo fmt --check）
- [ ] 测试覆盖率 >80%
- [ ] 所有公共 API 有文档注释
- [ ] 没有安全隐患

**静态分析**
```bash
# 运行 clippy
cargo clippy --all-features -- -D warnings

# 检查格式
cargo fmt --check

# 检查未使用的依赖
cargo +nightly udeps

# 安全审计
cargo audit
```

### 测试策略

**单元测试**
- 每个模块有对应的测试模块
- 测试覆盖率 >80%
- 使用 property-based testing (proptest)

**集成测试**
- 端到端场景测试
- 跨模块集成测试
- 性能回归测试

**CI/CD 流程**
```yaml
# GitHub Actions 示例
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo fmt --check
```

### 性能基准

**基准测试框架**
```rust
// 使用 Criterion.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_physics(c: &mut Criterion) {
    c.bench_function("physics_update", |b| {
        b.iter(|| {
            physics_update(black_box(/* ... */));
        });
    });
}

criterion_group!(benches, benchmark_physics);
criterion_main!(benches);
```

**性能目标**
- 物理更新: <1ms (1000 entities)
- 渲染帧率: >60 FPS @ 1080p
- 资源加载: <100ms (10MB texture)
- 网络延迟: <50ms (局域网)

---

## 总结

### 关键里程碑

| 里程碑 | 日期 | 交付物 |
|--------|------|--------|
| M1: 基础设施重构完成 | 第2月 | P0-0, P0-1, P0-2 完成 |
| M2: 性能优化完成 | 第4月 | P0-3 完成，5x-10x 性能提升 |
| M3: 架构重构完成 | 第6月 | P1-1, P1-2, P2-1 完成 |
| M4: 质量保证完成 | 第7月 | P1-3 完成，>80% 测试覆盖 |
| M5: 事件溯源完成 | 第8月 | P2-2 完成 |
| M6: 生产就绪 | 第12月 | P2-3 完成，v0.5.0 发布 |

### 成功标准

**技术指标**
- ✅ 减少 70% 条件编译
- ✅ 减少 30% 代码重复
- ✅ 重构 57 个异步函数
- ✅ >80% 测试覆盖率
- ✅ 5x-10x 性能提升（特定场景）

**架构指标**
- ✅ 清晰的分层架构
- ✅ 富领域对象
- ✅ CQRS 模式实现
- ✅ 完整的事件溯源
- ✅ 网络复制系统

**质量指标**
- ✅ 零编译警告
- ✅ 零 clippy 警告
- ✅ 零已知安全漏洞
- ✅ 完整的文档
- ✅ CI/CD 流程

---

**文档版本**: v1.0
**最后更新**: 2025-12-30
**维护者**: 开发团队
**审查周期**: 每月审查进度，每季度调整计划
