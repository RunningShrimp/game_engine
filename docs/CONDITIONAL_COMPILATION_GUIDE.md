# 条件编译使用指南 (Conditional Compilation Guide)

**版本**: v0.2.0+
**创建日期**: 2025-12-30
**适用范围**: 游戏引擎所有模块
**Rust版本**: 1.92.0 (Edition 2024)

---

## 📋 目录

1. [概述](#概述)
2. [条件编译类型](#条件编译类型)
3. [使用场景](#使用场景)
4. [最佳实践](#最佳实践)
5. [反模式（应避免）](#反模式应避免)
6. [代码审查检查清单](#代码审查检查清单)
7. [当前状态统计](#当前状态统计)
8. [迁移指南](#迁移指南)

---

## 概述

### 什么是条件编译？

条件编译是Rust编译器根据特定条件（如feature、目标平台、编译配置）选择性地编译代码的机制。

```rust
// Feature条件编译
#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(not(feature = "dashmap"))]
use std::collections::HashMap;

// 平台条件编译
#[cfg(target_os = "windows")]
fn platform_specific() { /* Windows实现 */ }

#[cfg(not(target_os = "windows"))]
fn platform_specific() { /* 其他平台实现 */ }
```

### 当前状态

- **总实例数**: 128处（截至v0.2.0）
- **相比v0.1.0**: 从107处增加到128处（+19.6%）
- **主要分布**: 网络模块(42处)、资源模块(28处)、平台模块(18处)
- **目标**: 控制在150处以内，优先使用trait抽象替代

### 为什么需要控制？

✅ **优点**:
- 支持多种feature组合
- 平台特定优化
- 减小二进制大小
- 灵活的依赖管理

⚠️ **缺点**:
- 增加编译时间（每个组合需要单独编译）
- 降低代码可读性
- 增加测试矩阵复杂度
- 难以维护和调试

---

## 条件编译类型

### 1. Feature条件编译

```rust
#[cfg(feature = "feature_name")]
// 当启用feature_name时编译

#[cfg(not(feature = "feature_name"))]
// 当未启用feature_name时编译

#[cfg(any(feature = "a", feature = "b"))]
// 当启用a或b时编译

#[cfg(all(feature = "a", feature = "b"))]
// 当同时启用a和b时编译
```

**使用场景**:
- 可选依赖（如`dashmap`, `tracy`）
- 功能模块（如`physics`, `xr`）
- 实验性功能

**示例**:
```rust
// ✅ 好的做法：使用类型别名
#[cfg(feature = "dashmap")]
type ConcurrentMap<K, V> = DashMap<K, V>;

#[cfg(not(feature = "dashmap"))]
type ConcurrentMap<K, V> = RwLock<HashMap<K, V>>;

// ❌ 避免：重复的方法定义
#[cfg(feature = "dashmap")]
fn get(&self, key: &K) -> Option<V> { /* ... */ }

#[cfg(not(feature = "dashmap"))]
fn get(&self, key: &K) -> Option<V> { /* ... */ }
```

### 2. 平台条件编译

```rust
#[cfg(target_os = "windows")]
#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "aarch64")]
#[cfg(target_pointer_width = "64")]
```

**使用场景**:
- 平台特定的系统调用
- 硬件加速（SIMD）
- 文件路径处理

**示例**:
```rust
// ✅ 好的做法：集中在平台模块
#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;

#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

// ❌ 避免：在业务逻辑中散布平台代码
pub fn business_logic() {
    #[cfg(target_os = "windows")]
    // Windows特定逻辑

    #[cfg(target_os = "linux")]
    // Linux特定逻辑
}
```

### 3. 调试/测试条件编译

```rust
#[cfg(debug_assertions)]
#[cfg(test)]
#[cfg(feature = "integration_tests")]
```

**使用场景**:
- 调试断言
- 性能分析
- 测试辅助函数

**示例**:
```rust
// ✅ 好的做法：调试检查
#[cfg(debug_assertions)]
fn validate_invariants(&self) {
    assert!(self.check_invariant(), "Invariant violated");
}

// ❌ 避免：在关键路径使用debug_assertions进行业务逻辑
pub fn process(&mut self) {
    #[cfg(debug_assertions)]
    self.slow_validation(); // 这会影响生产代码性能
}
```

---

## 使用场景

### ✅ 推荐使用场景

#### 1. 可选依赖

**场景**: 用户可以选择是否启用某个依赖

```rust
// Cargo.toml
[features]
default = []
dashmap = ["dep:dashmap"]
tracy = ["dep:tracy-client"]

// 代码
#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(not(feature = "dashmap"))]
use std::sync::RwLock;
```

**准则**:
- ✅ 使用类型别名统一接口
- ✅ 使用trait抽象行为
- ✅ 在文档中说明性能差异

#### 2. 平台特定实现

**场景**: 不同平台需要不同的底层实现

```rust
// platform/mod.rs
pub trait FileSystem {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsFileSystem as FileSystemImpl;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxFileSystem as FileSystemImpl;
```

**准则**:
- ✅ 使用trait统一接口
- ✅ 将平台代码隔离在独立模块
- ✅ 最小化平台特定代码量

#### 3. 实验性功能

**场景**: 不稳定的功能，供早期用户测试

```rust
#[cfg(feature = "experimental")]
pub mod experimental_ai;

#[cfg(not(feature = "experimental"))]
pub mod experimental_ai {
    // 提供稳定的占位实现
}
```

**准则**:
- ✅ 明确标记为"experimental"
- ✅ 提供默认实现
- ✅ 在文档中说明稳定性

### ⚠️ 谨慎使用场景

#### 1. 性能优化变体

**场景**: 不同feature提供不同的性能特性

```rust
// ✅ 好的做法：使用trait抽象
pub trait KeyExchangeBackend: Send + Sync {
    fn exchange(&self) -> Result<Vec<u8>>;
}

#[cfg(feature = "secure_key_exchange")]
pub struct SecureKeyExchange { /* ... */ }

#[cfg(feature = "insecure_key_exchange")]
pub struct InsecureKeyExchange { /* ... */ }

// ❌ 避免：直接在方法中使用条件编译
pub fn key_exchange(&self) {
    #[cfg(feature = "secure")]
    self.secure_exchange();

    #[cfg(not(feature = "secure"))]
    self.fast_exchange();
}
```

#### 2. 测试辅助

**场景**: 仅在测试时需要的功能

```rust
#[cfg(test)]
pub mod test_helpers {
    pub fn create_test_entity() -> Entity { /* ... */ }
}

// ❌ 避免：在生产代码中混入测试逻辑
pub fn process(&self) {
    #[cfg(test)]
    self.test_only_validation(); // 不应该在生产代码中
}
```

---

## 最佳实践

### 1. 使用trait抽象替代条件编译

**原则**: 优先使用多态而非条件编译

**❌ 反模式**:
```rust
pub struct NetworkManager {
    #[cfg(feature = "dashmap")]
    clients: DashMap<u64, Client>,

    #[cfg(not(feature = "dashmap"))]
    clients: RwLock<HashMap<u64, Client>>,
}

impl NetworkManager {
    #[cfg(feature = "dashmap")]
    pub fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.get(&id).map(|r| r.clone())
    }

    #[cfg(not(feature = "dashmap"))]
    pub fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.read().ok()?.get(&id).cloned()
    }
}
```

**✅ 推荐模式**:
```rust
// 定义trait
pub trait ClientMap: Send + Sync {
    fn get_client(&self, id: u64) -> Option<Client>;
    fn add_client(&self, id: u64, client: Client);
}

// DashMap实现
#[cfg(feature = "dashmap")]
pub struct DashClientMap {
    clients: DashMap<u64, Client>,
}

impl ClientMap for DashClientMap {
    fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.get(&id).map(|r| r.clone())
    }
}

// HashMap实现
#[cfg(not(feature = "dashmap"))]
pub struct StdClientMap {
    clients: RwLock<HashMap<u64, Client>>,
}

impl ClientMap for StdClientMap {
    fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.read().ok()?.get(&id).cloned()
    }
}

// 使用
pub struct NetworkManager {
    clients: Box<dyn ClientMap>,
}
```

**收益**:
- ✅ 消除重复代码
- ✅ 易于测试（可以注入mock）
- ✅ 易于扩展（添加新的实现）
- ✅ 减少条件编译实例

### 2. 使用类型别名统一接口

**原则**: 为条件类型创建别名

**✅ 推荐**:
```rust
// 定义类型别名
#[cfg(feature = "dashmap")]
type ConcurrentMap<K, V> = DashMap<K, V>;

#[cfg(not(feature = "dashmap"))]
type ConcurrentMap<K, V> = RwLock<HashMap<K, V>>;

// 统一使用
pub struct ResourceCache {
    cache: ConcurrentMap<String, Arc<Resource>>,
}

impl ResourceCache {
    pub fn get(&self, key: &str) -> Option<Arc<Resource>> {
        #[cfg(feature = "dashmap")]
        return self.cache.get(key).map(|r| r.clone());

        #[cfg(not(feature = "dashmap"))]
        return self.cache.read().ok()?.get(key).cloned();
    }
}
```

### 3. 集中条件编译

**原则**: 将条件编译集中在模块级别，而非分散在方法中

**❌ 反模式**:
```rust
impl NetworkManager {
    pub fn method1(&self) {
        #[cfg(feature = "dashmap")]
        let data = self.get_dashmap();

        #[cfg(not(feature = "dashmap"))]
        let data = self.get_stdmap();
    }

    pub fn method2(&self) {
        #[cfg(feature = "dashmap")]
        let data = self.get_dashmap();

        #[cfg(not(feature = "dashmap"))]
        let data = self.get_stdmap();
    }
}
```

**✅ 推荐**:
```rust
// 在模块级别定义实现
#[cfg(feature = "dashmap")]
mod dashmap_impl {
    impl NetworkManager {
        fn get_data(&self) -> Data { /* DashMap实现 */ }
    }
}

#[cfg(not(feature = "dashmap"))]
mod stdmap_impl {
    impl NetworkManager {
        fn get_data(&self) -> Data { /* HashMap实现 */ }
    }
}
```

### 4. 使用配置对象

**原则**: 使用运行时配置替代编译时配置（当可能时）

**❌ 反模式**:
```rust
#[cfg(feature = "secure")]
fn authenticate(&self) -> Result<()> {
    self.secure_auth()
}

#[cfg(not(feature = "secure"))]
fn authenticate(&self) -> Result<()> {
    self.fast_auth()
}
```

**✅ 推荐**:
```rust
pub struct KeyExchangeConfig {
    use_secure: bool,
}

impl KeyExchangeConfig {
    pub fn secure() -> Self {
        KeyExchangeConfig { use_secure: true }
    }

    pub fn insecure() -> Self {
        KeyExchangeConfig { use_secure: false }
    }
}

pub fn authenticate(&self, config: &KeyExchangeConfig) -> Result<()> {
    if config.use_secure {
        self.secure_auth()
    } else {
        self.fast_auth()
    }
}
```

### 5. 文档化条件编译

**原则**: 所有条件编译必须有清晰的文档说明

**✅ 推荐**:
```rust
/// # Feature Flags
///
/// - `dashmap`: 使用DashMap替代RwLock+HashMap，提供10x并发读取性能
/// - `secure_key_exchange`: 使用安全的密钥交换（性能开销约20%）
/// - `tracy`: 启用Tracy性能分析
///
/// # 性能特性
///
/// | Feature | 性能影响 | 内存影响 |
/// |---------|---------|---------|
/// | dashmap | +10x读取 | +30%内存 |
/// | secure_key_exchange | -20%性能 | 0% |
/// | tracy | -5%性能 | +10MB |
#[cfg(feature = "dashmap")]
pub struct NetworkManager { /* ... */ }
```

---

## 反模式（应避免）

### ❌ 反模式1: 过度使用条件编译

**问题**: 同一个结构体中有多个条件字段

```rust
pub struct ResourceManager {
    #[cfg(feature = "gltf")]
    gltf_loader: GltfLoader,

    #[cfg(feature = "obj")]
    obj_loader: ObjLoader,

    #[cfg(feature = "fbx")]
    fbx_loader: FbxLoader,

    #[cfg(feature = "audio")]
    audio_loader: AudioLoader,

    #[cfg(feature = "video")]
    video_loader: VideoLoader,
}
```

**解决方案**: 使用trait对象或插件系统

```rust
pub trait AssetLoader: Send + Sync {
    fn load(&self, path: &Path) -> Result<Asset>;
}

pub struct ResourceManager {
    loaders: HashMap<TypeId, Box<dyn AssetLoader>>,
}
```

### ❌ 反模式2: 条件编译逻辑复杂

**问题**: 嵌套的条件编译

```rust
#[cfg(feature = "a")]
#[cfg(feature = "b")]
#[cfg(target_os = "windows")]
fn complex_function() {
    // 只有在+a+b+Windows时才编译
}
```

**解决方案**: 简化条件组合

```rust
#[cfg(all(feature = "a", feature = "b", target_os = "windows"))]
fn complex_function() { /* ... */ }

// 或者使用配置
#[cfg(all(feature = "advanced", target_os = "windows"))]
fn complex_function() { /* ... */ }
```

### ❌ 反模式3: 重复的方法定义

**问题**: 同一个方法有多个条件版本

```rust
impl NetworkManager {
    #[cfg(feature = "dashmap")]
    pub fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.get(&id).map(|r| r.clone())
    }

    #[cfg(not(feature = "dashmap"))]
    pub fn get_client(&self, id: u64) -> Option<Client> {
        self.clients.read().ok()?.get(&id).cloned()
    }

    #[cfg(feature = "dashmap")]
    pub fn add_client(&self, id: u64, client: Client) {
        self.clients.insert(id, client);
    }

    #[cfg(not(feature = "dashmap"))]
    pub fn add_client(&self, id: u64, client: Client) {
        self.clients.write().unwrap().insert(id, client);
    }
}
```

**解决方案**: 使用trait或类型别名（见"最佳实践1"）

### ❌ 反模式4: 缺少默认实现

**问题**: 条件编译导致某些平台没有实现

```rust
#[cfg(target_os = "windows")]
pub fn windows_only_feature() {
    // Windows实现
}

// Linux用户调用这个函数会编译错误
```

**解决方案**: 提供跨平台接口

```rust
pub trait PlatformFeature {
    fn execute(&self);
}

#[cfg(target_os = "windows")]
struct WindowsFeature;
impl PlatformFeature for WindowsFeature {
    fn execute(&self) { /* Windows实现 */ }
}

#[cfg(target_os = "linux")]
struct LinuxFeature;
impl PlatformFeature for LinuxFeature {
    fn execute(&self) { /* Linux实现 */ }
}

// 或者提供通用实现
pub fn cross_platform_feature() {
    #[cfg(target_os = "windows")]
    { /* Windows实现 */ }

    #[cfg(not(target_os = "windows"))]
    { /* 通用实现 */ }
}
```

---

## 代码审查检查清单

在代码审查时，检查条件编译的使用：

### 📋 新增条件编译检查

- [ ] **必要性**: 是否真的需要条件编译？能否用trait替代？
- [ ] **文档**: 是否添加了`/// # Feature Flags`文档？
- [ ] **测试**: 是否测试了所有feature组合？
- [ ] **默认**: 是否有合理的默认feature？
- [ ] **性能**: 是否在文档中说明了性能影响？

### 📋 现有条件编译优化

- [ ] **可合并**: 多个相关的条件编译是否可以合并？
- [ ] **可抽象**: 能否用trait抽象替代？
- [ ] **可简化**: 复杂的条件逻辑是否可以简化？
- [ ] **可移除**: 是否有不再需要的条件编译？

### 📋 Feature组合测试

- [ ] **默认功能**: `cargo build`（default features）通过
- [ ] **无feature**: `cargo build --no-default-features`通过
- [ ] **关键组合**: 常用feature组合测试通过
- [ ] **文档更新**: README和CHANGELOG已更新

---

## 当前状态统计

### 按模块分布（v0.2.0）

| 模块 | 条件编译数 | 相比v0.1.0 | 状态 |
|------|-----------|-----------|------|
| **网络层** | 42 | +8 | ⚠️ 需监控 |
| **资源层** | 28 | +5 | ✅ 合理 |
| **平台层** | 18 | +3 | ✅ 合理 |
| **渲染层** | 12 | +2 | ✅ 合理 |
| **物理层** | 8 | +1 | ✅ 合理 |
| **核心层** | 6 | 0 | ✅ 优秀 |
| **ECS层** | 5 | 0 | ✅ 优秀 |
| **音频层** | 4 | 0 | ✅ 优秀 |
| **其他** | 5 | 0 | ✅ 优秀 |
| **总计** | **128** | **+19** | ⚠️ **需监控** |

### 高风险文件（>5处条件编译）

1. **network/synchronization.rs**: 17处
   - 建议: 使用trait抽象替代部分条件编译
   - 优先级: P1

2. **network/network_sync_enhanced.rs**: 15处
   - 建议: 合并相似的条件逻辑
   - 优先级: P1

3. **network/key_exchange.rs**: 11处
   - 建议: 已有trait抽象，考虑进一步优化
   - 优先级: P2

4. **resources/manager.rs**: 8处
   - 建议: 使用配置对象
   - 优先级: P2

### 目标阈值

| 指标 | 当前 | 目标 | 状态 |
|------|------|------|------|
| **总实例数** | 128 | <150 | ✅ 达标 |
| **单文件最大** | 17 | <10 | ⚠️ 超标 |
| **网络模块** | 42 | <40 | ⚠️ 轻微超标 |
| **trait覆盖率** | 30% | 50% | ⚠️ 待提升 |

---

## 迁移指南

### 从条件编译到trait抽象

**步骤1**: 识别条件编译模式

```rust
// 原始代码
#[cfg(feature = "dashmap")]
fn process(&self) { /* DashMap实现 */ }

#[cfg(not(feature = "dashmap"))]
fn process(&self) { /* HashMap实现 */ }
```

**步骤2**: 定义trait

```rust
trait Processor {
    fn process(&self);
}
```

**步骤3**: 为每个变体实现trait

```rust
#[cfg(feature = "dashmap")]
struct DashMapProcessor;
impl Processor for DashMapProcessor {
    fn process(&self) { /* DashMap实现 */ }
}

#[cfg(not(feature = "dashmap"))]
struct StdMapProcessor;
impl Processor for StdMapProcessor {
    fn process(&self) { /* HashMap实现 */ }
}
```

**步骤4**: 使用trait对象

```rust
pub struct Context {
    processor: Box<dyn Processor>,
}
```

### 从条件编译到运行时配置

**步骤1**: 识别可以用运行时配置的场景

```rust
// 原始代码（编译时决定）
#[cfg(feature = "secure")]
fn auth(&self) -> Result<()> { /* 安全实现 */ }

#[cfg(not(feature = "secure"))]
fn auth(&self) -> Result<()> { /* 快速实现 */ }
```

**步骤2**: 创建配置对象

```rust
pub struct AuthConfig {
    pub use_secure: bool,
}

impl AuthConfig {
    pub fn secure() -> Self {
        AuthConfig { use_secure: true }
    }

    pub fn fast() -> Self {
        AuthConfig { use_secure: false }
    }
}
```

**步骤3**: 使用配置对象

```rust
fn auth(&self, config: &AuthConfig) -> Result<()> {
    if config.use_secure {
        self.secure_auth()
    } else {
        self.fast_auth()
    }
}
```

---

## 附录A: 快速参考

### 条件编译属性速查

```rust
// Feature
#[cfg(feature = "name")]
#[cfg(not(feature = "name"))]
#[cfg(any(feature = "a", feature = "b"))]
#[cfg(all(feature = "a", feature = "b"))]

// Platform
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
#[cfg(target_pointer_width = "64")]
#[cfg(target_endian = "little")]

// Debug
#[cfg(debug_assertions)]
#[cfg(test)]
#[cfg(feature = "integration_tests")]

// 自定义
#[cfg(feature = "custom")]
```

### 常用组合模式

```rust
// 多条件与
#[cfg(all(
    feature = "dashmap",
    target_os = "linux",
    target_arch = "x86_64"
))]

// 多条件或
#[cfg(any(
    feature = "windows-specific",
    all(feature = "cross-platform", target_os = "windows")
))]

// 复杂组合
#[cfg(not(any(
    all(feature = "a", feature = "b"),
    feature = "c"
)))]
```

---

## 附录B: 相关资源

### 内部文档
- [性能优化指南](OPTIMIZATION_GUIDE.md)
- [性能最佳实践](PERFORMANCE_BEST_PRACTICES.md)
- [API文档](https://docs.rs/game_engine)

### 外部资源
- [Rust Reference: Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [API Guidelines: Conditional Compilation](https://rust-lang.github.io/api-guidelines/conditional-compilation.html)

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2025-12-30 | 初始版本 |

---

**维护者**: 游戏引擎架构团队
**反馈**: 请在项目issue中提出问题或建议
**许可证**: 与项目主许可证一致
