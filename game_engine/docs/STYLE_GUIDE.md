# 文档语言规范 (Documentation Language Standard)

## 概述 (Overview)

本规范定义了游戏引擎项目的文档语言标准，旨在提高代码的可读性和可维护性。

This specification defines the documentation language standard for the game engine project, aiming to improve code readability and maintainability.

---

## Rust Edition

**要求**: 使用 Rust 2024 edition（稳定版本）

**Requirement**: Use Rust 2024 edition (stable version)

```toml
# Cargo.toml
[package]
edition = "2024"
```

**参考 (Reference)**: https://blog.rust-lang.org.cn/2025/02/20/Rust-1.85.0.html

---

## 公开API文档（English Public API Documentation）

### 适用范围 (Scope)

所有公开的API必须使用英文文档注释：

All public APIs MUST use English documentation comments:

- `pub struct` - 公开结构体
- `pub enum` - 公开枚举
- `pub fn` - 公开函数
- `pub trait` - 公开特质
- `pub mod` - 公开模块
- `pub const` / `pub static` - 公开常量
- `pub type` - 公开类型别名

### 文档注释格式 (Documentation Comment Format)

```rust
//! # Module Title
//!
//! This module provides...
//!
//! ## Examples
//!
//! ```

/// Brief description of the struct.
///
/// # Description (Optional)
///
/// More detailed explanation...
///
/// # Examples
///
/// ```
/// use game_engine::MyStruct;
///
/// let instance = MyStruct::new();
/// ```
///
/// # Panics
///
/// Describe when this function panics...
///
/// # Errors
///
/// Describe potential errors...
///
/// # Safety
///
/// If this function is `unsafe`, describe safety requirements...
pub struct MyStruct {
    /// Public field documentation
    pub field: Type,
    // Private fields use Chinese comments
    private_field: Type,  // 私有字段：中文注释
}

impl MyStruct {
    /// Creates a new instance of MyStruct.
    ///
    /// # Examples
    ///
    /// ```
    /// let instance = MyStruct::new();
    /// ```
    pub fn new() -> Self {
        // 初始化实现：中文注释说明实现细节
        Self {
            field: Type::default(),
            private_field: Type::default(),
        }
    }

    /// Public method documentation in English.
    ///
    /// # Arguments
    ///
    /// * `arg1` - Description of argument 1
    /// * `arg2` - Description of argument 2
    ///
    /// # Returns
    ///
    /// Description of return value
    pub fn public_method(&self, arg1: Type1, arg2: Type2) -> ReturnType {
        // 实现细节：使用中文注释
        // 算法步骤1：...
        // 算法步骤2：...
        unimplemented!()
    }

    // 私有方法：使用中文注释
    // 功能：执行内部计算
    fn private_method(&self) -> Type {
        // 内部逻辑说明
        unimplemented!()
    }
}
```

---

## 私有实现注释（Chinese Private Implementation Comments）

### 适用范围 (Scope)

所有私有实现细节使用中文注释：

All private implementation details use Chinese comments:

- 私有方法 (`fn private_method()`)
- 内部字段 (struct private fields)
- 行注释 (`// Comment`)
- 块注释 (`/* Comment */` 或 `//!/` inside `impl`)
- 实现细节说明
- 性能关键点标记
- 算法步骤说明

### 注释格式 (Comment Format)

```rust
impl MyStruct {
    // === 私有辅助方法 ===

    // 执行内部数据验证
    // 参数：
    //   - data: 待验证的数据
    // 返回：
    //   - Result<(), Error>: 验证结果
    fn validate_internal(&self, data: &Data) -> Result<(), Error> {
        // 步骤1：检查数据完整性
        if !data.is_complete() {
            return Err(Error::IncompleteData);
        }

        // 步骤2：验证数据一致性
        self.check_consistency(data)?;

        // 步骤3：性能优化：提前退出
        Ok(())
    }

    // 性能关键点：此方法在热路径上，避免内存分配
    #[inline]
    fn fast_path_calculation(&self, input: Input) -> Output {
        // 使用SIMD优化：批量处理4个元素
        // 参考：game_engine_simd::batch
        unsafe {
            // SIMD指令优化实现
            self.simd_implementation(input)
        }
    }
}
```

---

## 特殊注释规范 (Special Comment Standards)

### 性能标记 (Performance Tags)

```rust
// ⚠️ 性能关键点：此函数每帧调用，避免堆分配
// 🚀 热路径：已使用SIMD优化
// 🔒 线程安全：使用内部可变性
```

### 安全标记 (Safety Tags)

```rust
// ⚠️ unsafe: 调用者必须确保指针有效
// 📋 要求：
//   1. lifetime必须超过返回值
//   2. 指针必须对齐
pub unsafe fn dangerous_operation(ptr: *const u8) -> &'static [u8] {
    // 实现细节...
}
```

### TODO/FIXME 标记

```rust
// TODO: 未来版本需要优化此算法的时间复杂度
// FIXME: 临时解决方案，等待上游库修复
// HACK: 由于库限制，使用变通方案
```

---

## 模块级文档示例 (Module-Level Documentation Example)

```rust
//! # Core Engine Module
//!
//! This module provides the core engine functionality including:
//!
//! - Engine initialization and lifecycle management
//! - Main loop control
//! - System coordination
//! - Resource management
//!
//! ## Architecture
//!
//! The engine follows a microkernel architecture pattern:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Engine (Microkernel)        │
//! │  ┌─────────┐  ┌─────────────────┐  │
//! │  │Scheduler│  │  System Registry│  │
//! │  └─────────┘  └─────────────────┘  │
//! └─────────────────────────────────────┘
//!          │              │
//!          ▼              ▼
//!     ┌────────┐    ┌────────────┐
//!     │Systems │    │  Plugins   │
//!     └────────┘    └────────────┘
//! ```
//!
//! ## Examples
//!
//! ```rust
//! use game_engine::core::Engine;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut engine = Engine::new();
//! engine.start().await?;
//! # Ok(())
//! # }
//! ```

// 模块私有实现说明：
// - 使用微内核架构模式
// - 通过trait object实现插件系统
// - 使用tokio运行时管理异步任务
use crate::internal::scheduler::Scheduler;

// 内部模块：私有实现使用中文注释
mod internal {
    // 调度器实现：管理任务执行顺序
    pub struct Scheduler {
        // 任务队列
        queue: Vec<Task>,
    }
}
```

---

## 检查清单 (Checklist)

### 编写代码时 (When Writing Code)

- [ ] 公开API (`pub`) 是否有英文文档注释？
- [ ] 私有实现是否有中文注释？
- [ ] 文档注释是否包含示例代码？
- [ ] `unsafe` 代码是否有详细的安全说明？
- [ ] 性能关键代码是否有性能标记？

### 代码审查时 (During Code Review)

- [ ] 公开API文档是否清晰易懂？
- [ ] 私有实现注释是否解释了"为什么"？
- [ ] 是否使用了Rust 2024 edition特性？
- [ ] 复杂算法是否有步骤说明？
- [ ] 是否有未解释的magic numbers？

---

## 工具验证 (Tool Verification)

### 生成文档

```bash
# 生成并打开文档
cargo doc --no-deps --open

# 生成所有feature的文档
cargo doc --all-features

# 检查文档完整性（警告会被视为错误）
cargo doc --no-deps --document-private-items
```

### 检查文档覆盖率

```bash
# 使用tarpaulin检查文档覆盖率
cargo tarpaulin --doc --timeout 120 --out Html
```

---

## 最佳实践 (Best Practices)

### 1. 文档即测试 (Documentation as Tests)

文档中的示例代码会被 `cargo test` 自动编译和运行，确保示例代码的正确性。

Example code in documentation is automatically compiled and run by `cargo test`, ensuring correctness of examples.

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use game_engine::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 2. 清晰的抽象层级 (Clear Abstraction Layers)

- **公开API**: 描述"是什么"和"怎么用"（英文）
- **实现注释**: 解释"为什么这样做"和"如何实现"（中文）

### 3. 保持文档更新 (Keep Documentation Updated)

- 修改API时同步更新文档注释
- 使用 `cargo doc` 验证文档正确性
- 在PR中检查文档变更

### 4. 性能敏感代码注释 (Performance-Sensitive Code Comments)

对于性能关键的代码，提供详细的中文注释说明：

For performance-critical code, provide detailed Chinese comments:

```rust
// 性能优化：使用预分配避免动态扩容
// 预期容量：基于历史数据，约1000个元素
let mut vec = Vec::with_capacity(1000);

// ⚠️ 热路径：避免锁竞争
// 使用lock-free算法优化
self.lock_free_queue.push(item);
```

---

## 参考资源 (Resources)

- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust Documentation](https://rust-lang.github.io/rust-clippy/master/index.html)

---

## 版本历史 (Version History)

- **v1.0** (2025-12-29): 初始版本，定义基础规范
  - 确定Rust 2024 edition
  - 区分公开API（英文）和私有实现（中文）
  - 添加文档生成和验证流程
