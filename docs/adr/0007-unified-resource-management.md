# ADR-0007: 统一资源管理系统设计

## 状态
已接受

## 日期
2025-01-27

## 背景

资源管理系统需要支持：
1. **统一接口**: 不同类型的资源（纹理、模型、音频）使用统一接口
2. **依赖管理**: 资源之间存在依赖关系（如模型依赖材质，材质依赖纹理）
3. **热重载**: 开发时支持资源文件修改后自动重载
4. **流式加载**: 大资源需要流式加载，避免阻塞
5. **压缩缓存**: 自动压缩和缓存资源，减少内存和带宽

现有的资源管理系统缺乏统一的接口和依赖管理能力。

## 决策

采用 **分层资源管理系统** 架构：

### 1. 统一资源接口层

```rust
pub trait Resource: Send + Sync {
    fn metadata(&self) -> &ResourceMetadata;
    fn size(&self) -> usize;
}

pub trait ResourceLoader: Send + Sync {
    type Resource: Resource;
    async fn load(&self, path: &Path) -> Result<Self::Resource, ResourceError>;
}
```

### 2. 统一资源管理器

**UnifiedResourceManager** 提供统一的资源管理接口：
- 统一的加载接口
- 自动缓存管理
- 资源生命周期管理

### 3. 依赖管理系统

**DependencyGraph** 管理资源依赖关系：
- 拓扑排序确定加载顺序
- 检测循环依赖
- 自动加载依赖资源

### 4. 热重载系统

**HotReloadManager** 支持资源热重载：
- 文件系统监控
- 依赖传播（修改的资源及其依赖都会重载）
- 事件通知

### 5. 流式加载系统

**StreamingLoader** 支持大资源流式加载：
- 分块加载
- 进度回调
- 优先级管理

### 6. 压缩缓存系统

**CompressedResourceCache** 自动压缩和缓存：
- 自动压缩资源
- LRU缓存管理
- 压缩格式选择

## 后果

### 正面影响

1. **统一性**: 所有资源类型使用统一接口，简化使用
2. **自动化**: 依赖管理和热重载自动化，减少手动工作
3. **性能**: 流式加载和压缩缓存提升性能
4. **开发效率**: 热重载加速开发迭代
5. **可扩展性**: 易于添加新的资源类型

### 负面影响

1. **复杂性**: 系统复杂度增加
2. **内存开销**: 依赖图和缓存需要额外内存
3. **文件系统开销**: 热重载需要监控文件系统

## 替代方案

### 方案 A：每种资源类型独立管理
- **优点**: 实现简单，类型安全
- **缺点**: 代码重复，缺乏统一接口
- **未被选择的原因**: 无法实现统一的依赖管理和热重载

### 方案 B：完全集中式管理，移除类型系统
- **优点**: 统一接口，简化实现
- **缺点**: 失去类型安全，难以扩展
- **未被选择的原因**: 类型安全对Rust项目很重要

## 实现细节

### 依赖图结构

```rust
pub struct DependencyGraph {
    resources: HashMap<PathBuf, ResourceNode>,
    dependencies: HashMap<PathBuf, Vec<ResourceDependency>>,
}

pub struct ResourceDependency {
    path: PathBuf,
    dependency_type: String,
    required: bool,
}
```

### 热重载事件

```rust
pub enum HotReloadEvent {
    ResourceModified(PathBuf),
    ResourceDeleted(PathBuf),
    ResourceCreated(PathBuf),
}
```

### 加载顺序算法

使用拓扑排序（Kahn算法）确定加载顺序：
1. 构建依赖图
2. 检测循环依赖
3. 拓扑排序确定加载顺序
4. 并行加载无依赖的资源

## 参考

- [API参考](../api_reference.md#资源管理)
- 实现：
  - `game_engine/src/resources/resource_trait.rs`
  - `game_engine/src/resources/unified_manager.rs`
  - `game_engine/src/resources/dependency_manager.rs`
  - `game_engine/src/resources/hot_reload.rs`
  - `game_engine/src/resources/streaming_loader.rs`
  - `game_engine/src/resources/compressed_cache.rs`

