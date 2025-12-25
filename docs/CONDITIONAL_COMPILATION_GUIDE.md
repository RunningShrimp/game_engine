# 条件编译使用规范

本文档定义了游戏引擎中条件编译（`#[cfg(...)]`）的使用规范和最佳实践。

## 目录

1. [基本原则](#基本原则)
2. [平台检测规范](#平台检测规范)
3. [特性门控最佳实践](#特性门控最佳实践)
4. [代码示例](#代码示例)
5. [反模式](#反模式)
6. [工具和检查](#工具和检查)

## 基本原则

### 1. 集中管理原则

**原则**: 平台检测和特性检测逻辑应集中管理，避免在代码中散落。

**实现**:
- 使用 `platform::detection` 模块进行平台检测
- 使用 `compat::features::FeatureSet` 进行特性检测

**示例**:
```rust
// ❌ 不推荐 - 散落的平台检测
#[cfg(target_os = "windows")]
fn windows_specific() { }

#[cfg(target_os = "macos")]
fn macos_specific() { }

// ✅ 推荐 - 使用集中管理的检测函数
use game_engine::platform::detection;

if detection::is_windows() {
    windows_specific();
} else if detection::is_macos() {
    macos_specific();
}
```

### 2. 模块级别封装原则

**原则**: 复杂条件编译逻辑应封装在独立模块中，而不是在函数级别。

**实现**:
- 将特性相关的实现拆分为独立模块（如 `gltf_loader_impl.rs` 和 `gltf_loader_stub.rs`）
- 在主模块中统一导出

**示例**:
```rust
// ❌ 不推荐 - 函数级别的条件编译
#[cfg(feature = "gltf")]
pub fn load_gltf() { /* 实现 */ }

#[cfg(not(feature = "gltf"))]
pub fn load_gltf() { /* 存根 */ }

// ✅ 推荐 - 模块级别封装
#[cfg(feature = "gltf")]
#[path = "gltf_loader_impl.rs"]
mod gltf_loader_impl;

#[cfg(not(feature = "gltf"))]
#[path = "gltf_loader_stub.rs"]
mod gltf_loader_stub;

#[cfg(feature = "gltf")]
pub use gltf_loader_impl::*;

#[cfg(not(feature = "gltf"))]
pub use gltf_loader_stub::*;
```

### 3. 统一导出模式

**原则**: 所有特性门控的模块应使用统一的导出模式。

**标准模式**:
```rust
// 在各个模块的 mod.rs 中统一管理
#[cfg(feature = "xr")]
pub mod xr;

#[cfg(not(feature = "xr"))]
pub mod xr {
    // 提供编译时错误提示
    compile_error!("XR feature is not enabled. Add 'xr' feature to Cargo.toml");
}
```

## 平台检测规范

### 使用 platform::detection 模块

所有平台检测应使用 `platform::detection` 模块提供的函数：

```rust
use game_engine::platform::detection;

// 平台类型检测
if detection::is_mobile() { }
if detection::is_desktop() { }
if detection::is_console() { }
if detection::is_web() { }

// 特定平台检测
if detection::is_windows() { }
if detection::is_macos() { }
if detection::is_linux() { }
if detection::is_android() { }
if detection::is_ios() { }

// 架构检测
if detection::is_x86_64() { }
if detection::is_aarch64() { }
if detection::is_wasm32() { }

// 能力检测
if detection::supports_simd() { }
```

### 平台信息结构

使用 `PlatformInfo` 获取完整的平台信息：

```rust
use game_engine::platform::detection::PlatformInfo;

let info = PlatformInfo::current();
println!("OS: {}, Arch: {}", info.os, info.arch);
```

## 特性门控最佳实践

### 使用 compat::features::FeatureSet

特性检测应使用 `FeatureSet`：

```rust
use game_engine::compat::features::FeatureSet;

let features = FeatureSet::current();
if features.xr_enabled {
    // XR功能代码
}

if features.is_feature_enabled("gltf") {
    // GLTF加载代码
}
```

### 特性导出模式

#### 模式1: 模块级别条件编译

```rust
// mod.rs
#[cfg(feature = "xr")]
pub mod xr_impl;

#[cfg(not(feature = "xr"))]
pub mod xr_impl {
    compile_error!("XR feature is not enabled. Add 'xr' feature to Cargo.toml");
}

#[cfg(feature = "xr")]
pub use xr_impl::*;
```

#### 模式2: 文件级别条件编译（使用 #[path]）

```rust
// mod.rs
#[cfg(feature = "gltf")]
#[path = "gltf_loader_impl.rs"]
mod gltf_loader_impl;

#[cfg(not(feature = "gltf"))]
#[path = "gltf_loader_stub.rs"]
mod gltf_loader_stub;

#[cfg(feature = "gltf")]
pub use gltf_loader_impl::*;

#[cfg(not(feature = "gltf"))]
pub use gltf_loader_stub::*;
```

#### 模式3: 类型级别条件编译（简单情况）

```rust
// 仅当类型定义简单时使用
#[cfg(feature = "xr")]
pub struct XrSession { /* ... */ }

#[cfg(not(feature = "xr"))]
pub struct XrSession {
    // 存根实现或编译错误
}
```

### 避免嵌套条件编译

**原则**: 避免深层嵌套的条件编译，使用模块封装。

```rust
// ❌ 不推荐 - 嵌套过深
#[cfg(feature = "wasm")]
#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    #[cfg(feature = "xr")]
    mod wasm_xr { }
}

// ✅ 推荐 - 使用模块封装
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm_impl;

#[cfg(all(feature = "wasm", target_arch = "wasm32", feature = "xr"))]
mod wasm_xr;
```

## 代码示例

### 示例1: 特性门控模块

```rust
// resources/mod.rs
#[cfg(feature = "gltf")]
#[path = "gltf_loader_impl.rs"]
mod gltf_loader_impl;

#[cfg(not(feature = "gltf"))]
#[path = "gltf_loader_stub.rs"]
mod gltf_loader_stub;

// 统一导出
#[cfg(feature = "gltf")]
pub use gltf_loader_impl::{GltfLoadError, GltfLoader, GltfScene};

#[cfg(not(feature = "gltf"))]
pub use gltf_loader_stub::{GltfLoadError, GltfLoader, GltfScene};
```

### 示例2: 平台特定实现

```rust
// platform/mod.rs
use crate::platform::detection;

#[cfg(target_arch = "wasm32")]
pub mod web_fs;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_fs;

// 统一导出
#[cfg(target_arch = "wasm32")]
pub use web_fs::WebFilesystem as Filesystem;

#[cfg(not(target_arch = "wasm32"))]
pub use native_fs::NativeFilesystem as Filesystem;
```

### 示例3: 运行时特性检测

```rust
use game_engine::compat::features::FeatureSet;

fn initialize_features() {
    let features = FeatureSet::current();
    
    if features.xr_enabled {
        initialize_xr();
    }
    
    if features.gltf_enabled {
        initialize_gltf_loader();
    }
    
    println!("Enabled features: {:?}", features.enabled_features());
}
```

## 反模式

### 反模式1: 散落的平台检测

```rust
// ❌ 不推荐
#[cfg(target_os = "windows")]
fn windows_func() { }

#[cfg(target_os = "macos")]
fn macos_func() { }

// ✅ 推荐
use game_engine::platform::detection;

fn platform_specific_func() {
    if detection::is_windows() {
        windows_impl();
    } else if detection::is_macos() {
        macos_impl();
    }
}
```

### 反模式2: 深层嵌套条件编译

```rust
// ❌ 不推荐
#[cfg(feature = "wasm")]
#[cfg(target_arch = "wasm32")]
#[cfg(feature = "xr")]
mod complex_nested { }

// ✅ 推荐
#[cfg(all(feature = "wasm", target_arch = "wasm32", feature = "xr"))]
mod wasm_xr;
```

### 反模式3: 重复的条件编译检查

```rust
// ❌ 不推荐 - 重复检查
#[cfg(feature = "gltf")]
fn func1() {
    #[cfg(feature = "gltf")]
    let x = load_gltf();
}

// ✅ 推荐 - 使用特性集
use game_engine::compat::features::FeatureSet;

fn func1() {
    let features = FeatureSet::current();
    if features.gltf_enabled {
        let x = load_gltf();
    }
}
```

### 反模式4: 函数级别的条件编译（复杂实现）

```rust
// ❌ 不推荐 - 复杂实现的条件编译
#[cfg(feature = "gltf")]
pub fn load_gltf() -> Result<GltfScene, GltfLoadError> {
    // 大量实现代码
}

#[cfg(not(feature = "gltf"))]
pub fn load_gltf() -> Result<GltfScene, GltfLoadError> {
    Err(GltfLoadError::FeatureNotEnabled)
}

// ✅ 推荐 - 模块级别封装
// 见示例1
```

## 工具和检查

### 检查条件编译使用

使用以下命令检查条件编译的使用情况：

```bash
# 统计条件编译使用次数
grep -r "#\[cfg(" game_engine/src --include="*.rs" | wc -l

# 查找嵌套条件编译
grep -r "#\[cfg(" game_engine/src --include="*.rs" | grep -E "cfg\(.*cfg\("
```

### 代码审查检查清单

在代码审查时，检查以下项目：

- [ ] 平台检测是否使用了 `platform::detection` 模块？
- [ ] 特性检测是否使用了 `compat::features::FeatureSet`？
- [ ] 是否存在深层嵌套的条件编译？
- [ ] 特性门控模块是否使用统一的导出模式？
- [ ] 是否存在重复的条件编译检查？
- [ ] 复杂实现是否已拆分为独立模块？

## 迁移指南

### 从散落的条件编译迁移到集中管理

1. **识别所有平台检测**:
   ```bash
   grep -r "target_os\|target_arch" game_engine/src --include="*.rs"
   ```

2. **替换为 detection 模块**:
   ```rust
   // 替换前
   #[cfg(target_os = "windows")]
   
   // 替换后
   use game_engine::platform::detection;
   if detection::is_windows() { }
   ```

3. **识别所有特性检测**:
   ```bash
   grep -r 'feature = "' game_engine/src --include="*.rs"
   ```

4. **替换为 FeatureSet**:
   ```rust
   // 替换前
   #[cfg(feature = "gltf")]
   
   // 替换后
   use game_engine::compat::features::FeatureSet;
   let features = FeatureSet::current();
   if features.gltf_enabled { }
   ```

## 常见问题

### Q: 什么时候使用编译时条件编译 vs 运行时检测？

**A**: 
- **编译时条件编译** (`#[cfg]`): 用于平台特定代码、特性门控、测试代码
- **运行时检测**: 用于硬件能力检测、可选功能启用

### Q: 如何处理可选依赖？

**A**: 使用 Cargo 特性门控，并在模块级别提供存根实现：

```rust
#[cfg(feature = "optional_dep")]
pub mod optional_impl;

#[cfg(not(feature = "optional_dep"))]
pub mod optional_stub;
```

### Q: 如何测试条件编译的代码？

**A**: 使用不同的特性组合运行测试：

```bash
# 测试默认特性
cargo test

# 测试所有特性
cargo test --all-features

# 测试特定特性
cargo test --features gltf,xr
```

## 参考

- [Rust 条件编译文档](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Cargo 特性文档](https://doc.rust-lang.org/cargo/reference/features.html)
- [平台检测模块](../game_engine/src/platform/detection.rs)
- [特性管理模块](../game_engine/src/compat/features.rs)

