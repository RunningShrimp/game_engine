# 条件编译规范化文档

**版本**: v1.0
**日期**: 2025-12-31
**目标**: 统一条件编译使用规范，提高代码可维护性

---

## 目录

1. [基本概念](#基本概念)
2. [条件编译类型](#条件编译类型)
3. [使用规范](#使用规范)
4. [最佳实践](#最佳实践)
5. [反模式](#反模式)
6. [工具支持](#工具支持)

---

## 基本概念

### 什么是条件编译？

条件编译允许根据编译时的条件（如操作系统、feature、架构等）选择性地编译代码。

```rust
// 示例：只在启用特定feature时编译此代码
#[cfg(feature = "advanced_rendering")]
fn advanced_render_pass() { }
```

### 为什么要规范？

1. **可读性**: 清晰的条件编译逻辑更容易理解
2. **可维护性**: 统一的规范减少重构成本
3. **可测试性**: 明确的条件更容易测试
4. **性能**: 避免不必要的条件检查

---

## 条件编译类型

### 1. Feature条件

基于Cargo features的条件编译：

```rust
// 单个feature
#[cfg(feature = "python")]
fn python_integration() { }

// 多个feature（OR）
#[cfg(any(feature = "python", feature = "javascript"))]
fn scripting_integration() { }

// 多个feature（AND）
#[cfg(all(feature = "python", feature = "network"))]
fn python_network_integration() { }

// feature不存在
#[cfg(not(feature = "python"))]
fn no_python_fallback() { }
```

### 2. 操作系统条件

基于目标操作系统的条件编译：

```rust
// 特定操作系统
#[cfg(target_os = "windows")]
fn windows_specific() { }

#[cfg(target_os = "linux")]
fn linux_specific() { }

#[cfg(target_os = "macos")]
fn macos_specific() { }

// 操作系统系列
#[cfg(windows)]
fn any_windows() { }

#[cfg(unix)]
fn any_unix() { }

#[cfg(target_family = "windows")]
fn windows_family() { }
```

### 3. 架构条件

基于CPU架构的条件编译：

```rust
// 特定架构
#[cfg(target_arch = "x86_64")]
fn x86_64_optimized() { }

#[cfg(target_arch = "aarch64")]
fn arm64_optimized() { }

#[cfg(target_arch = "wasm32")]
fn wasm_compatible() { }

// 位数
#[cfg(target_pointer_width = "64")]
fn sixty_four_bit() { }
```

### 4. 编译器条件

基于Rust编译器版本的条件编译：

```rust
// 最小版本
#[cfg(rustc_version_ge = "1.70.0")]
fn modern_rust_feature() { }

// 特定版本
#[cfg(rustc_version = "1.70.0")]
fn exact_version() { }
```

### 5. 环境条件

基于编译环境的条件编译：

```rust
// 测试环境
#[cfg(test)]
mod tests { }

// 调试模式
#[cfg(debug_assertions)]
fn debug_only_checks() { }

// 发布模式
#[cfg(not(debug_assertions))]
fn release_optimized() { }

// 自定义定义
#[cfg(custom_build_flag)]
fn custom_build() { }
```

---

## 使用规范

### 规范1: 使用cfg_attr减少代码重复

**❌ 不好的做法**:
```rust
#[cfg(feature = "simd")]
fn process() { }

#[cfg(not(feature = "simd"))]
fn process() { }
```

**✅ 推荐做法**:
```rust
#[cfg_attr(feature = "simd", inline(always))]
fn process() { }
```

### 规范2: 使用有意义的条件名称

**❌ 不好的做法**:
```rust
#[cfg(feature = "feat1")]
fn feature_one() { }
```

**✅ 推荐做法**:
```rust
#[cfg(feature = "advanced_rendering")]
fn advanced_render_pass() { }
```

### 规范3: 复杂条件使用辅助宏

**❌ 不好的做法**:
```rust
#[cfg(all(
    feature = "render",
    any(
        target_os = "windows",
        target_os = "linux"
    ),
    not(feature = "basic")
))]
fn complex_condition() { }
```

**✅ 推荐做法**:
```rust
// 在lib.rs或mod.rs中定义辅助宏
#[cfg(all(
    feature = "render",
    any(windows, linux),
    not(feature = "basic")
))]
macro_rules! is_advanced_render_platform {
    () => { true };
}

#[cfg(is_advanced_render_platform)]
fn complex_condition() { }
```

### 规范4: 条件编译代码模块化

**❌ 不好的做法**:
```rust
// 一个大函数充满了条件编译
fn process_data() {
    #[cfg(feature = "optimization_a")]
    { /* 大量代码 */ }

    #[cfg(not(feature = "optimization_a"))]
    { /* 大量重复代码 */ }
}
```

**✅ 推荐做法**:
```rust
// 分模块实现
#[cfg(feature = "optimization_a")]
mod optimized {
    pub fn process_data() { }
}

#[cfg(not(feature = "optimization_a"))]
mod standard {
    pub fn process_data() { }
}

// 统一接口
fn process_data() {
    #[cfg(feature = "optimization_a")]
    optimized::process_data();

    #[cfg(not(feature = "optimization_a"))]
    standard::process_data();
}
```

### 规范5: 使用cfg!进行运行时检查

当需要运行时分支时使用`cfg!`宏：

```rust
fn process_data() {
    // 编译时选择的分支（无运行时开销）
    if cfg!(feature = "optimization") {
        // 优化版本
    } else {
        // 标准版本
    }
}
```

---

## 最佳实践

### 1. Feature组合设计

创建合理的feature层次结构：

```toml
[features]
# 基础功能
default = ["basic"]
basic = []

# 中级功能（依赖基础）
intermediate = ["basic", "extra_feature"]

# 高级功能（依赖中级）
advanced = ["intermediate", "more_features"]
```

### 2. 条件编译注释

为复杂的条件编译添加注释：

```rust
// ARM NEON优化仅在64位ARM架构上启用
#[cfg(all(
    feature = "simd",
    target_arch = "aarch64",
    target_pointer_width = "64"
))]
fn neon_optimized() { }
```

### 3. 条件编译测试

为不同的条件编译配置编写测试：

```rust
// 仅在特定feature下测试
#[cfg(test)]
#[cfg(feature = "python")]
mod python_tests {
    #[test]
    fn test_python_integration() { }
}

// 条件化测试实现
#[cfg(test)]
mod conditional_tests {
    #[test]
    #[cfg(feature = "optimization")]
    fn test_optimized_path() { }

    #[test]
    #[cfg(not(feature = "optimization"))]
    fn test_standard_path() { }
}
```

### 4. 文档中的条件编译

在文档中说明条件编译的要求：

```rust
/// 使用高级渲染功能
///
/// # Requirements
///
/// 此函数仅在以下条件下可用：
/// - 启用`advanced_rendering` feature
/// - 目标平台为64位系统
///
/// # Example
///
/// ```rust
/// #[cfg(feature = "advanced_rendering")]
/// use crate::render::advanced;
///
/// #[cfg(feature = "advanced_rendering")]
/// advanced::render_scene();
/// ```
#[cfg(feature = "advanced_rendering")]
pub fn advanced_render() { }
```

### 5. 性能敏感代码的条件编译

对于性能敏感的代码，使用条件编译避免运行时检查：

```rust
// 编译时选择的内联函数
#[cfg_attr(feature = "optimization", inline(always))]
#[cfg_attr(not(feature = "optimization"), inline(never))]
fn performance_critical() { }
```

---

## 反模式

### 反模式1: 过度条件编译

**❌ 避免**:
```rust
// 太多细粒度的条件
#[cfg(feature = "render_a")]
fn render_a() { }

#[cfg(feature = "render_b")]
fn render_b() { }

#[cfg(feature = "render_c")]
fn render_c() { }

// ... 更多类似代码
```

**✅ 推荐**:
```rust
// 使用更通用的feature
#[cfg(feature = "render")]
fn render() {
    // 运行时或配置文件选择具体实现
}
```

### 反模式2: 嵌套条件编译

**❌ 避免**:
```rust
#[cfg(feature = "a")]
fn example() {
    #[cfg(feature = "b")]
    { /* 代码 */ }

    #[cfg(not(feature = "b"))]
    { /* 其他代码 */ }
}
```

**✅ 推荐**:
```rust
#[cfg(all(feature = "a", feature = "b"))]
fn example_ab() { }

#[cfg(all(feature = "a", not(feature = "b")))]
fn example_a_only() { }
```

### 反模式3: 重复的条件

**❌ 避免**:
```rust
#[cfg(feature = "optimization")]
fn func1() { }

#[cfg(feature = "optimization")]
fn func2() { }

#[cfg(feature = "optimization")]
fn func3() { }
```

**✅ 推荐**:
```rust
// 创建条件编译模块
#[cfg(feature = "optimization")]
mod optimized {
    pub fn func1() { }
    pub fn func2() { }
    pub fn func3() { }
}

#[cfg(not(feature = "optimization"))]
mod standard {
    pub fn func1() { }
    pub fn func2() { }
    pub fn func3() { }
}

// 统一导出
#[cfg(feature = "optimization")]
pub use optimized::*;

#[cfg(not(feature = "optimization"))]
pub use standard::*;
```

### 反模式4: 不一致的条件

**❌ 避免**:
```rust
// 一个地方用feature名称
#[cfg(feature = "PYTHON")]
fn func1() { }

// 另一个地方用小写
#[cfg(feature = "python")]
fn func2() { }
```

**✅ 推荐**:
```rust
// 统一使用小写feature名称
#[cfg(feature = "python")]
fn func1() { }

#[cfg(feature = "python")]
fn func2() { }
```

---

## 工具支持

### 1. Cargo检查

使用cargo检查特定feature配置：

```bash
# 检查默认features
cargo check

# 检查所有features
cargo check --all-features

# 检查特定feature组合
cargo check --features "python,network"

# 检查无features
cargo check --no-default-features
```

### 2. Feature矩阵测试

创建测试矩阵：

```bash
# 测试所有feature组合
#!/bin/bash
for render in "basic" "advanced"; do
    for script in "" "python" "javascript"; do
        if [ -z "$script" ]; then
            cargo check --features "$render"
        else
            cargo check --features "$render,$script"
        fi
    done
done
```

### 3. CI/CD集成

在CI中测试不同配置：

```yaml
# .github/workflows/test.yml
jobs:
  test:
    strategy:
      matrix:
        features:
          - ""
          - "--all-features"
          - "--features python"
          - "--features network,encryption"
    steps:
      - uses: actions/checkout@v2
      - name: Test with ${{ matrix.features }}
        run: cargo test ${{ matrix.features }}
```

### 4. 文档生成

为不同feature配置生成文档：

```bash
# 生成默认配置文档
cargo doc --open

# 生成特定feature文档
cargo doc --features "python,network" --open

# 生成所有features文档
cargo doc --all-features --open
```

---

## 审核清单

在添加条件编译时，请确保：

- [ ] Feature名称清晰且有意义
- [ ] 使用正确的条件编译类型（feature/os/arch等）
- [ ] 复杂条件有注释说明
- [ ] 为不同配置编写测试
- [ ] 更新相关文档
- [ ] 检查是否有更好的替代方案（如trait对象）
- [ ] 确保向后兼容性（如果修改现有代码）

---

## 参考资源

### 官方文档
- [Rust Reference: Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Cargo Guide: Features](https://doc.rust-lang.org/cargo/reference/features.html)

### 相关RFC
- [RFC 2345: Intra-doc links](https://rust-lang.github.io/rfcs/2345-intra-doc-links.html)
- [RFC 2483: cfg(target_vendor)](https://rust-lang.github.io/rfcs/2483-target-vendor.html)

### 社区最佳实践
- [The Rust Book: Conditional Compilation](https://doc.rust-lang.org/book/conditional-compilation.html)
- [Rust by Example: cfg](https://doc.rust-lang.org/rust-by-example/attribute/cfg.html)

---

**文档维护**: 本文档应随项目发展定期更新
**反馈渠道**: 如有问题或建议，请在项目issue中讨论

**最后更新**: 2025-12-31
**文档版本**: v1.0
