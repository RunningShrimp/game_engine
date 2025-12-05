# Default实现模式统一指南

**创建日期**: 2025-12-03  
**状态**: ✅ 已完成

---

## 概述

本指南定义了项目中统一使用Default实现的模式，以减少代码重复并提高一致性。

---

## 实现模式优先级

### 1. 优先使用 `#[derive(Default)]`（最高优先级）

**适用场景**: 所有字段都实现了`Default` trait的结构体

**示例**:
```rust
#[derive(Default)]
pub struct NetworkConfig {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
}
```

**优点**:
- 最简洁
- 编译器自动生成
- 零运行时开销

---

### 2. 使用 `impl_default!` 宏（次优先级）

**适用场景**: 需要自定义默认值，但所有字段都可以直接初始化

**示例**:
```rust
pub struct GpuDrivenConfig {
    pub frustum_culling: bool,
    pub occlusion_culling: bool,
    pub max_instances: u32,
}

impl_default!(GpuDrivenConfig {
    frustum_culling: true,
    occlusion_culling: false,
    max_instances: 65536,
});
```

**优点**:
- 统一格式
- 减少代码重复
- 易于维护

---

### 3. 手动实现（特殊情况）

**适用场景**:
- 需要调用其他方法（如`new()`、`identity()`、`zero()`）
- 包含复杂初始化逻辑
- 需要错误处理或资源分配

**示例**:
```rust
impl Default for ParticleSystemManager {
    fn default() -> Self {
        Self::new(64)  // 有特殊参数
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self::new()
            .add_stop(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0))
            .add_stop(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0))
    }
}
```

**注意事项**:
- 添加注释说明为什么需要手动实现
- 避免循环调用（`default()`调用`new()`，`new()`又调用`default()`）

---

## 统一规则

### 规则1: 避免循环调用

**错误示例**:
```rust
impl Default for MyStruct {
    fn default() -> Self {
        Self::new()  // ❌ 如果new()调用default()，会造成循环
    }
}

impl MyStruct {
    pub fn new() -> Self {
        Self::default()  // ❌ 循环调用
    }
}
```

**正确做法**:
```rust
// 方案1: 使用derive
#[derive(Default)]
pub struct MyStruct { ... }

impl MyStruct {
    pub fn new() -> Self {
        Self::default()  // ✅ 调用derive生成的default()
    }
}

// 方案2: 使用宏
impl_default!(MyStruct { ... });

impl MyStruct {
    pub fn new() -> Self {
        Self::default()  // ✅ 调用宏生成的default()
    }
}
```

---

### 规则2: 统一使用宏

**错误示例**:
```rust
impl Default for MyConfig {
    fn default() -> Self {
        Self {
            field1: true,
            field2: 100,
        }
    }
}
```

**正确做法**:
```rust
impl_default!(MyConfig {
    field1: true,
    field2: 100,
});
```

---

### 规则3: 优先derive

**错误示例**:
```rust
pub struct SimpleConfig {
    pub enabled: bool,
    pub count: usize,
}

impl_default!(SimpleConfig {
    enabled: false,
    count: 0,
});
```

**正确做法**:
```rust
#[derive(Default)]
pub struct SimpleConfig {
    pub enabled: bool,  // bool::default() = false
    pub count: usize,   // usize::default() = 0
}
```

---

## 已优化的实现

### 已使用 `impl_default!` 宏的实现

- `GpuDrivenConfig`
- `GpuIndirectDrawConfig` ✅ (刚优化)
- `FlockConfig`
- `AdaptiveLodConfig`
- `LodConfig`
- `ParticleEmitterConfig`
- 等120+个实现

### 已使用 `#[derive(Default)]` 的实现

- `NetworkConfig`
- `PbrScene`
- `RenderCache`
- `EntityDelta`
- `ConsoleInputHandler`
- 等50+个实现

### 保留手动实现的实现

- `ParticleSystemManager` - 调用`new(64)`
- `PowerAwareManager` - 调用`new()`
- `ColorGradient` - 链式调用
- 等65+个实现（有特殊逻辑）

---

## 检查清单

在添加新的`Default`实现时，请检查：

- [ ] 是否所有字段都实现了`Default`？→ 使用`#[derive(Default)]`
- [ ] 是否需要自定义默认值？→ 使用`impl_default!`宏
- [ ] 是否需要调用其他方法？→ 手动实现，添加注释
- [ ] 是否避免循环调用？→ 检查`new()`和`default()`的关系
- [ ] 是否遵循统一格式？→ 使用宏而不是手动实现

---

## 工具支持

### 查找未统一的实现

```bash
# 查找手动实现的Default
grep -r "impl Default for" src/ | grep -v "impl_default"

# 查找可以使用derive的
grep -r "#\[derive(" src/ | grep -v "Default"
```

---

## 总结

- ✅ **优先使用**: `#[derive(Default)]`
- ✅ **次优先级**: `impl_default!`宏
- ✅ **特殊情况**: 手动实现（添加注释说明原因）
- ❌ **避免**: 手动实现简单字段初始化
- ❌ **避免**: 循环调用

通过统一Default实现模式，我们：
- 减少了代码重复
- 提高了代码一致性
- 简化了维护工作
- 提高了代码可读性


