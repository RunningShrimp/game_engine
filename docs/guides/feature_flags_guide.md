# 特性标志使用指南

本文档说明游戏引擎中特性标志（feature flags）的使用规范和最佳实践。

## 概述

特性标志用于控制可选功能的编译。合理使用特性标志可以：
- 减少编译时间（禁用不需要的功能）
- 减少二进制大小
- 支持平台特定的功能
- 提供可选依赖

## 特性标志分类

### 核心特性（不应使用特性标志）

以下功能是引擎的核心功能，**不应**使用特性标志：

- **ECS系统** - 实体组件系统是引擎的基础
- **渲染核心** - 基础渲染功能是必需的
- **物理核心** - 物理模拟是核心功能
- **输入处理** - 输入系统是必需的
- **资源管理** - 资源加载和管理是核心功能
- **并行处理** - 并行功能默认启用，不使用特性标志

### 可选特性（合理使用特性标志）

以下功能是可选功能，**应该**使用特性标志：

#### 文件格式支持
- `gltf` - GLTF/GLB文件格式支持
  - 用途：3D模型加载
  - 依赖：`gltf` crate

#### 平台特定功能
- `xr` - XR（虚拟现实/增强现实）支持
  - 用途：VR/AR应用开发
  - 依赖：`openxr` crate

- `wasm` - WebAssembly运行时支持
  - 用途：Web平台部署
  - 依赖：`wasmtime` crate

#### 脚本语言绑定
- `pyo3` - Python绑定支持
  - 用途：Python脚本集成
  - 依赖：`pyo3` crate

#### 安全功能
- `secure_key_exchange` - 安全密钥交换（默认开启）
  - 用途：网络加密通信
  - 依赖：`x25519-dalek-ng`, `hkdf`
  - 注意：默认启用，提供`insecure_key_exchange`用于测试

### 性能特性（默认启用）

以下性能优化特性**默认启用**，不使用特性标志：

- **SIMD优化** - SIMD指令优化（通过`game_engine_simd` crate）
- **并行处理** - 使用`rayon`进行并行处理
- **性能分析** - 性能监控和分析工具

## 使用规范

### 1. 定义特性标志

在`Cargo.toml`中定义特性：

```toml
[features]
default = ["secure_key_exchange"]
gltf = ["dep:gltf"]
xr = ["dep:openxr"]
wasm = ["dep:wasmtime"]
pyo3 = ["dep:pyo3"]
secure_key_exchange = ["dep:x25519-dalek-ng", "dep:hkdf"]
insecure_key_exchange = []  # 仅用于测试
```

### 2. 使用条件编译

在代码中使用`#[cfg(feature = "...")]`：

```rust
#[cfg(feature = "gltf")]
pub mod gltf_loader;

#[cfg(feature = "gltf")]
pub use gltf_loader::load_gltf_scene;
```

### 3. 提供默认实现

对于可选功能，提供默认实现或清晰的错误消息：

```rust
#[cfg(not(feature = "gltf"))]
pub fn load_gltf_scene(_path: &str) -> Result<GltfScene, String> {
    Err("GLTF support not enabled. Enable the 'gltf' feature to use this function.".to_string())
}
```

### 4. 文档说明

在文档中说明特性要求：

```rust
/// 加载GLTF场景
///
/// # 特性要求
/// 需要启用`gltf`特性标志。
///
/// # 示例
/// ```rust,no_run
/// # #[cfg(feature = "gltf")]
/// # {
/// use game_engine::resources::load_gltf_scene;
/// let scene = load_gltf_scene("models/character.gltf")?;
/// # }
/// ```
#[cfg(feature = "gltf")]
pub fn load_gltf_scene(path: &str) -> Result<GltfScene, String> {
    // ...
}
```

## 最佳实践

### ✅ 应该做的

1. **使用特性标志控制可选依赖**
   - 对于大型可选依赖（如`gltf`、`wasmtime`），使用特性标志

2. **提供清晰的错误消息**
   - 当功能未启用时，提供明确的错误消息

3. **文档化特性要求**
   - 在API文档中说明特性要求

4. **测试不同特性组合**
   - 确保不同特性组合都能正常编译和运行

### ❌ 不应该做的

1. **不要为核心功能使用特性标志**
   - ECS、渲染核心、物理核心等不应使用特性标志

2. **不要使用特性标志控制性能优化**
   - 性能优化应该默认启用，不使用特性标志

3. **不要过度使用特性标志**
   - 避免创建过多细粒度的特性标志

4. **不要破坏API兼容性**
   - 特性标志不应改变公共API的签名

## 示例

### 正确示例：GLTF支持

```rust
// Cargo.toml
[features]
gltf = ["dep:gltf"]

// src/resources/mod.rs
#[cfg(feature = "gltf")]
pub mod gltf_loader;

#[cfg(feature = "gltf")]
pub use gltf_loader::GltfScene;
```

### 错误示例：并行处理

```rust
// ❌ 错误：并行处理是核心功能，不应使用特性标志
#[cfg(feature = "parallel")]
pub mod parallel;

// ✅ 正确：并行处理默认启用
pub mod parallel;
```

## 检查清单

在添加新的特性标志之前，请确认：

- [ ] 功能确实是可选的（不是核心功能）
- [ ] 功能有大型可选依赖
- [ ] 功能是平台特定的
- [ ] 已在`Cargo.toml`中定义特性
- [ ] 已添加文档说明特性要求
- [ ] 已提供未启用时的错误处理
- [ ] 已测试不同特性组合

## 相关资源

- [Cargo Features文档](https://doc.rust-lang.org/cargo/reference/features.html)
- [条件编译文档](https://doc.rust-lang.org/reference/conditional-compilation.html)

