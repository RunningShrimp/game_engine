# 资源管理优化计划

## 概述

本文档概述游戏引擎资源管理系统的优化建议和实施计划。

## 当前资源管理架构

### 现有资源加载器

| 模块 | 文件 | 功能 |
|------|------|------|
| 协程加载器 | `resources/coroutine_loader.rs` | 基于 tokio 的异步资源加载 |
| 预加载管理器 | `resources/preload_manager.rs` | 资源预加载和依赖管理 |
| 预分配管理器 | `resources/preallocation_manager.rs` | 资源预分配以减少运行时分配 |
| 内存分配器 | `resources/memory_allocator.rs` | GPU 内存池管理 |
| 内存监控 | `resources/memory_monitor.rs` | 内存使用监控和预警 |
| 纹理压缩 | `resources/texture_compression.rs` | 纹理压缩和解压 |
| GLTF 加载器 | `resources/gltf_loader.rs` | GLTF/GLB 模型加载 |

### 资源管理器

| 模块 | 文件 | 功能 |
|------|------|------|
| 场景管理器 | `scene/manager.rs` | 场景资源和实体管理 |
| 实例批处理 | `render/instance_batch.rs` | 渲染批处理优化 |
| 间接绘制 | `render/gpu_driven/indirect_manager.rs` | GPU 驱动渲染 |
| 剔除管理器 | `render/gpu_driven/culling_manager.rs` | 视锥剔除优化 |

## 优化建议

### 1. 统一资源接口 (高优先级)

**目标**: 创建统一的资源加载和管理接口

```rust
/// 统一资源 trait
pub trait Resource: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Metadata: Send + Sync;

    fn metadata(&self) -> &Self::Metadata;
    fn size_bytes(&self) -> usize;
    fn is_loaded(&self) -> bool;
}

/// 统一资源加载器 trait
pub trait ResourceLoader: Send + Sync {
    type Resource: Resource;
    type Context;

    async fn load(&self, path: &Path, ctx: &Self::Context) -> Result<Self::Resource, ResourceError>;
    async fn preload(&self, paths: &[PathBuf], ctx: &Self::Context) -> Vec<Result<Self::Resource, ResourceError>>;
}

/// 统一资源管理器
pub struct ResourceManager<L: ResourceLoader> {
    loader: L,
    cache: HashMap<PathBuf, L::Resource>,
    pending: HashMap<PathBuf, tokio::task::JoinHandle<L::Resource>>,
}
```

**优点**:
- 统一的资源生命周期管理
- 更好的缓存策略
- 简化单元测试

**预计工作量**: 3-4 天

### 2. 实现资源热重载 (中优先级)

**目标**: 在运行时动态重新加载修改的资源

```rust
pub struct HotReloadManager {
    watcher: RecommendedWatcher,
    reload_handlers: HashMap<PathBuf, Box<dyn ReloadHandler>>,
}

pub trait ReloadHandler: Send + Sync {
    fn on_reload(&self, path: &Path, resource: &dyn Resource);
}
```

**优点**:
- 提高开发效率
- 无需重启游戏即可看到资源更改

**预计工作量**: 2-3 天

### 3. 实现资源依赖管理 (中优先级)

**目标**: 跟踪和管理资源之间的依赖关系

```rust
pub struct DependencyGraph {
    nodes: HashMap<PathBuf, DependencyNode>,
    edges: Vec<(PathBuf, PathBuf)>,
}

pub struct DependencyNode {
    path: PathBuf,
    dependents: Vec<PathBuf>,
    dependencies: Vec<PathBuf>,
    last_modified: SystemTime,
}
```

**优点**:
- 自动预加载依赖资源
- 确保正确的加载顺序
- 支持循环依赖检测

**预计工作量**: 2-3 天

### 4. 实现资源流式加载 (低优先级)

**目标**: 大型资源（如高分辨率纹理）的流式加载

```rust
pub struct StreamingLoader {
    chunk_size: usize,
    max_concurrent: usize,
}

pub struct StreamingHandle<T> {
    receiver: tokio::sync::mpsc::Receiver<Chunk<T>>,
    progress: Arc<AtomicUsize>,
}
```

**优点**:
- 减少初始加载时间
- 更好的内存利用
- 支持渐进式质量加载

**预计工作量**: 3-4 天

### 5. 实现资源压缩和缓存 (低优先级)

**目标**: 压缩资源并实现磁盘缓存

```rust
pub struct CompressedResourceCache {
    cache_dir: PathBuf,
    compression_algorithm: CompressionAlgorithm,
}

pub enum CompressionAlgorithm {
    Zstd { level: i32 },
    Lz4,
    Brotli { quality: u32 },
}
```

**优点**:
- 减少磁盘占用
- 更快的加载速度（如果 CPU 足够快）

**预计工作量**: 2 天

## 实施计划

### 阶段 1: 统一资源接口 (Week 1-2)
- [ ] 定义 `Resource` trait
- [ ] 定义 `ResourceLoader` trait
- [ ] 实现 `ResourceManager`
- [ ] 迁移现有加载器到新接口
- [ ] 单元测试

### 阶段 2: 资源依赖管理 (Week 3)
- [ ] 实现 `DependencyGraph`
- [ ] 实现依赖解析器
- [ ] 集成到 `ResourceManager`
- [ ] 单元测试

### 阶段 3: 资源热重载 (Week 4)
- [ ] 实现 `HotReloadManager`
- [ ] 实现 `ReloadHandler` trait
- [ ] 添加文件系统监视器
- [ ] 集成到 `ResourceManager`
- [ ] E2E 测试

### 阶段 4: 流式加载和压缩 (Week 5-6)
- [ ] 实现 `StreamingLoader`
- [ ] 实现流式纹理加载
- [ ] 实现 `CompressedResourceCache`
- [ ] 性能基准测试
- [ ] 文档更新

## 性能目标

| 指标 | 当前 | 目标 | 测量方法 |
|------|------|------|----------|
| 资源加载时间 | 未知 | 减少 30% | 基准测试 |
| 内存占用 | 未知 | 减少 20% | 内存分析器 |
| 缓存命中率 | 未知 | > 80% | 监控指标 |
| 热重载延迟 | N/A | < 500ms | 手动测试 |

## 风险评估

### 高风险
- **向后兼容性**: 统一接口可能破坏现有代码
- **性能回归**: 新增抽象层可能影响性能

### 中风险
- **热重载稳定性**: 文件系统监视可能不可靠
- **依赖循环**: 复杂的资源依赖可能导致死锁

### 低风险
- **流式加载**: 仅用于特定场景，影响有限
- **压缩算法**: 可选功能，不影响核心流程

## 建议优先级

| 任务 | 优先级 | 理由 |
|------|--------|------|
| 统一资源接口 | P1 | 基础设施，影响其他优化 |
| 资源依赖管理 | P1 | 改善加载流程 |
| 资源热重载 | P2 | 提高开发效率 |
| 资源流式加载 | P3 | 优化特定场景 |
| 资源压缩和缓存 | P3 | 存储优化 |

## 参考资料

- 现有实现:
  - `resources/coroutine_loader.rs`
  - `resources/preload_manager.rs`
  - `resources/memory_allocator.rs`
- 类似项目:
  - Bevy Engine Asset System
  - Amethyst Asset Loader
  - Unity Asset Bundle
