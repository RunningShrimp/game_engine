# 条件编译最佳实践指南

## 概述

本文档描述游戏引擎项目中条件编译的使用规范和最佳实践，旨在减少条件编译的使用，提高代码可维护性。

---

## 核心原则

### 1. 优先使用Trait抽象

**问题代码**（大量条件编译）:
```rust
#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(not(feature = "dashmap"))]
use std::collections::HashMap;
use std::sync::RwLock;

pub struct OptimizedAssetManager {
    #[cfg(feature = "dashmap")]
    textures: DashMap<String, OptimizedHandle<String>>,

    #[cfg(not(feature = "dashmap"))]
    textures: RwLock<HashMap<String, OptimizedHandle<String>>>,

    // ... 更多字段重复相同模式
}
```

**推荐代码**（Trait抽象）:
```rust
// src/resources/concurrent/mod.rs
pub trait ConcurrentMap<K, V>: Send + Sync
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V) -> Option<V>;
    fn remove(&self, key: &K) -> Option<V>;
    fn len(&self) -> usize;
}

// src/resources/optimized_manager.rs
pub struct OptimizedAssetManager {
    textures: DefaultConcurrentMap<String, OptimizedHandle<String>>,
}
```

**收益**:
- ✅ 条件编译: 30 → 5处（-83%）
- ✅ 代码重复: 消除~600行
- ✅ 可维护性: 单一实现路径

---

## 2. 条件编译的适用场景

### ✅ 适合使用条件编译的情况

#### 2.1 依赖库的feature gate
```rust
#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn process_batch(&mut self, items: Vec<Item>) {
    #[cfg(feature = "parallel")]
    {
        items.par_iter().for_each(|item| self.process(item));
    }

    #[cfg(not(feature = "parallel"))]
    {
        items.iter().for_each(|item| self.process(item));
    }
}
```

#### 2.2 平台特定代码
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn scalar_version(x: f32) -> f32 {
    x * x
}
```

#### 2.3 测试专用代码
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // 测试代码
    }
}
```

### ❌ 避免使用条件编译的情况

#### 2.4 替代条件编译的Trait抽象

**反模式**（每个方法都有条件编译）:
```rust
impl OptimizedAssetManager {
    #[cfg(feature = "dashmap")]
    pub fn get_texture(&self, name: &str) -> Option<OptimizedHandle<String>> {
        self.textures.get(name).map(|h| h.clone())
    }

    #[cfg(not(feature = "dashmap"))]
    pub fn get_texture(&self, name: &str) -> Option<OptimizedHandle<String>> {
        let guard = self.textures.read().unwrap();
        guard.get(name).cloned()
    }

    // 对每个方法重复相同模式...
}
```

**推荐模式**（统一接口）:
```rust
impl OptimizedAssetManager {
    pub fn get_texture(&self, name: &str) -> Option<OptimizedHandle<String>> {
        let key = name.to_string();
        self.textures.get(&key)  // trait方法，无条件编译
    }
}
```

---

## 3. Trait抽象设计模式

### 3.1 Adapter模式

**结构**:
```rust
// trait定义
pub trait ConcurrentMap<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    // ...
}

// DashMap adapter
#[cfg(feature = "dashmap")]
pub struct DashMapAdapter<K, V> {
    inner: DashMap<K, V>,
}

#[cfg(feature = "dashmap")]
impl<K, V> ConcurrentMap<K, V> for DashMapAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|r| r.clone())
    }
    // ...
}

// RwLock adapter
#[cfg(not(feature = "dashmap"))]
pub struct RwLockAdapter<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> ConcurrentMap<K, V> for RwLockAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.read().ok()?.get(key).cloned()
    }
    // ...
}
```

### 3.2 类型别名（Feature-gated）

```rust
// 默认实现：根据feature自动选择
#[cfg(feature = "dashmap")]
pub type DefaultConcurrentMap<K, V> = DashMapAdapter<K, V>;

#[cfg(not(feature = "dashmap"))]
pub type DefaultConcurrentMap<K, V> = RwLockAdapter<K, V>;
```

**使用**:
```rust
pub struct MyService {
    // 单一代码路径，编译时选择实现
    cache: DefaultConcurrentMap<String, Value>,
}
```

---

## 4. 条件编译统计与监控

### 4.1 统计脚本

```bash
#!/bin/bash
# count_conditional_compilation.sh

echo "条件编译统计:"
echo "=============="

# 统计#[cfg(...)]出现次数
find src -name "*.rs" -exec grep -h "#\[cfg" {} \; | sort | uniq -c | sort -rn

# 统计feature相关的条件编译
echo ""
echo "Feature条件编译:"
find src -name "*.rs" -exec grep -h '#\[cfg(feature' {} \; | sort | uniq -c | sort -rn
```

### 4.2 目标指标

| 模块 | 优化前 | 优化后 | 目标 | 状态 |
|------|--------|--------|------|------|
| optimized_manager.rs | 30 | <5 | <10 | ✅ |
| server.rs | 54 | 54 | <20 | 🔄 基础设施完成 |
| 全项目总计 | 217 | <150 | <150 | 🔄 进行中 |

---

## 5. 重构指南

### 5.1 识别条件编译热点

**步骤**:
1. 运行统计脚本，找出条件编译最多的文件
2. 分析重复模式（如DashMap vs RwLock）
3. 评估是否适合trait抽象

### 5.2 重构流程

**Step 1**: 创建trait定义
```rust
// 新文件: src/xxx/concurrent/trait.rs
pub trait ConcurrentCollection<K, V>: Send + Sync {
    // 定义统一接口
}
```

**Step 2**: 实现adapters
```rust
// DashMap adapter
#[cfg(feature = "dashmap")]
impl<K, V> ConcurrentCollection<K, V> for DashMap<K, V> { ... }

// Mutex adapter
#[cfg(not(feature = "dashmap"))]
impl<K, V> ConcurrentCollection<K, V> for Mutex<HashMap<K, V>> { ... }
```

**Step 3**: 重构使用代码
```rust
// 优化前
#[cfg(feature = "dashmap")]
pub fn get(&self, key: &K) -> Option<V> { ... }

#[cfg(not(feature = "dashmap"))]
pub fn get(&self, key: &K) -> Option<V> { ... }

// 优化后
pub fn get(&self, key: &K) -> Option<V> {
    self.collection.get(key)  // trait方法
}
```

**Step 4**: 测试所有feature组合
```bash
cargo test --no-default-features
cargo test --features dashmap
cargo test --all-features
```

---

## 6. 常见陷阱

### 6.1 性能陷阱

**担忧**: Trait抽象会有性能损失？

**事实**: Rust的trait在编译期进行**单态化**（monomorphization），零成本抽象。

```rust
// 编译后等价于直接使用DashMap，无性能损失
let map: DefaultConcurrentMap<K, V> = ...;
```

### 6.2 类型推断陷阱

**问题**: Trait bounds必须完整
```rust
// ❌ 错误
pub trait ConcurrentMap<K, V> { }

// ✅ 正确
pub trait ConcurrentMap<K, V>: Send + Sync
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{ }
```

### 6.3 导出陷阱

**问题**: Feature-gated类型导出

```rust
// ❌ 错误：无条件导出feature-gated类型
pub use client_registry::{
    ClientRegistry, DashMapClientRegistry, MutexClientRegistry, DefaultClientRegistry,
};

// ✅ 正确：条件导出
pub use client_registry::{ClientRegistry, DefaultClientRegistry};

#[cfg(feature = "dashmap")]
pub use client_registry::DashMapClientRegistry;

#[cfg(not(feature = "dashmap"))]
pub use client_registry::MutexClientRegistry;
```

---

## 7. 检查清单

在引入新的条件编译前，评估是否可以通过以下方式避免：

- [ ] Trait抽象
- [ ] 策略模式
- [ ] 依赖注入
- [ ] 配置驱动（运行时）

如果必须使用条件编译：
- [ ] 文档化为什么需要条件编译
- [ ] 测试所有feature组合
- [ ] 评估是否可以后续重构为trait抽象

---

## 8. 相关资源

- **项目文档**: `CONDITIONAL_COMPILATION_GUIDE.md`
- **性能指南**: `PERFORMANCE_BEST_PRACTICES.md`
- **Async指南**: `ASYNC_USAGE_GUIDE.md`

---
**版本**: v0.1.0
**更新**: 2025-12-31
