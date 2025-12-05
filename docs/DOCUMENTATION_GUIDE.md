# 文档添加指南

**创建日期**: 2025-12-03  
**目的**: 指导如何为公共API添加文档注释

---

## 文档规范

### 1. 模块级文档

使用 `//!` 添加模块级文档：

```rust
//! 模块名称
//!
//! 模块的简要描述。
//!
//! ## 功能
//!
//! - 功能1
//! - 功能2
//!
//! ## 示例
//!
//! ```rust
//! use game_engine::module::Type;
//!
//! let instance = Type::new();
//! ```
```

### 2. 类型文档

使用 `///` 添加类型文档：

```rust
/// 类型名称
///
/// 类型的详细描述。
///
/// # 示例
///
/// ```rust
/// use game_engine::module::Type;
///
/// let instance = Type::new();
/// ```
pub struct Type {
    /// 字段描述
    pub field: u32,
}
```

### 3. 函数文档

使用 `///` 添加函数文档：

```rust
/// 函数名称
///
/// 函数的详细描述。
///
/// # 参数
///
/// * `param1` - 参数1的描述
/// * `param2` - 参数2的描述
///
/// # 返回
///
/// 返回值的描述
///
/// # 错误
///
/// 可能返回的错误
///
/// # 示例
///
/// ```rust
/// use game_engine::module::function;
///
/// let result = function(param1, param2)?;
/// ```
pub fn function(param1: u32, param2: String) -> Result<(), Error> {
    // ...
}
```

---

## 优先级

### 高优先级模块（优先添加文档）

1. **core/** - 核心功能
2. **domain/** - 领域对象
3. **render/** - 渲染系统
4. **physics/** - 物理系统
5. **audio/** - 音频系统

### 中优先级模块

- **ai/** - AI系统
- **network/** - 网络系统
- **xr/** - XR系统
- **services/** - 服务层

---

## 检查清单

添加文档时，确保：

- [ ] 所有公共类型都有文档
- [ ] 所有公共函数都有文档
- [ ] 包含使用示例
- [ ] 说明错误情况
- [ ] 说明性能注意事项（如适用）
- [ ] 运行 `cargo doc` 验证文档生成

---

## 工具

- `cargo doc --no-deps`: 生成文档
- `cargo doc --open`: 生成并打开文档
- `cargo doc --document-private-items`: 包含私有项

---

## 更新记录

- 2025-12-03: 创建文档指南

