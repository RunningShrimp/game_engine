# 错误类型定义宏使用指南

## 概述

`error_macros` 模块提供了一系列宏，用于减少错误类型定义中的重复代码。这些宏可以自动生成构造函数、严重级别方法和分类检查方法。

## 可用的宏

### 1. `define_error_variants!`

自动为错误变体生成构造函数和通用方法。

**语法**：
```rust
define_error_variants!(
    #[derive(Debug, Clone, thiserror::Error)]
    pub enum MyError {
        #[error("Error message: {field}")]
        VariantName {
            field1: Type1,
            field2: Type2,
        },
    }
);
```

**生成的功能**：
- 每个变体的构造函数（`variant_name()`）
- `severity()` 方法
- `is_recoverable()` 方法
- 自动添加 `severity: ErrorSeverity` 字段

**示例**：
```rust
use crate::error::{ErrorSeverity, error_macros::define_error_variants};

define_error_variants!(
    #[derive(Debug, Clone, thiserror::Error)]
    pub enum MyError {
        #[error("Item not found: {id}")]
        NotFound { id: String },

        #[error("Operation failed: {message}")]
        OperationFailed { message: String },
    }
);

// 使用构造函数
let err = MyError::not_found("item_123".to_string());
assert_eq!(err.severity(), ErrorSeverity::Error);
assert!(err.is_recoverable());
```

### 2. `impl_error_categories!`

为错误类型实现分类方法。

**语法**：
```rust
impl_error_categories!(MyError, ErrorCategory::MyCategory);
```

**生成的功能**：
- `category()` 方法，返回指定的错误分类

**示例**：
```rust
use crate::error::{ErrorCategory, error_macros::impl_error_categories};

impl_error_categories!(MyError, ErrorCategory::Resource);

let err = MyError::not_found("item_123");
assert_eq!(err.category(), ErrorCategory::Resource);
```

### 3. `impl_category_checks!`

为错误类型实现分类检查方法。

**语法**：
```rust
impl_category_checks!(MyError {
    is_file_related => [NotFound, LoadFailed, InvalidFormat],
    is_network_related => [Download, Upload, Streaming],
});
```

**生成的功能**：
- 每个指定的检查方法
- 方法返回 `bool`，表示错误是否属于该分类

**示例**：
```rust
use crate::error::error_macros::impl_category_checks;

impl_category_checks!(MyError {
    is_file_related => [NotFound, LoadFailed],
    is_network_related => [Download, Upload],
});

let err = MyError::not_found("file.txt");
assert!(err.is_file_related());
assert!(!err.is_network_related());
```

### 4. `define_error_with_custom_severity!`

定义带有自定义严重级别的错误变体。

**语法**：
```rust
define_error_with_custom_severity!(
    pub enum MyError {
        #[error("Critical error")]
        Critical { message: String } [Critical],

        #[error("Warning")]
        Warning { message: String } [Warning],
    }
);
```

**生成的功能**：
- 带有默认严重级别的构造函数
- 通用的 `with_severity()` 方法，可以为任何错误变体设置自定义严重级别
- `severity()` 和 `is_recoverable()` 方法

**示例**：
```rust
use crate::error::{ErrorSeverity, error_macros::define_error_with_custom_severity};

define_error_with_custom_severity!(
    pub enum MyError {
        #[error("Critical failure")]
        CriticalFailure { message: String } [Critical],

        #[error("Warning")]
        Warning { message: String } [Warning],
    }
);

// 使用默认严重级别
let crit = MyError::critical_failure("System crash".to_string());
assert_eq!(crit.severity(), ErrorSeverity::Critical);
assert!(!crit.is_recoverable());

// 使用自定义严重级别
let custom_crit = MyError::with_severity(
    MyError::warning("Deprecated API".to_string()),
    ErrorSeverity::Critical
);
assert_eq!(custom_crit.severity(), ErrorSeverity::Critical);
```

## 完整示例

### 简单错误类型

```rust
use crate::error::{ErrorCategory, ErrorSeverity, error_macros::*};

// 定义错误类型
define_error_variants!(
    #[derive(Debug, Clone, thiserror::Error)]
    pub enum StorageError {
        #[error("File not found: {path}")]
        NotFound { path: String },

        #[error("Failed to read: {path} - {message}")]
        ReadFailed { path: String, message: String },

        #[error("Permission denied: {path}")]
        PermissionDenied { path: String },
    }
);

// 实现分类
impl_error_categories!(StorageError, ErrorCategory::Resource);

// 实现分类检查
impl_category_checks!(StorageError {
    is_file_related => [NotFound, ReadFailed, PermissionDenied],
    is_permission_related => [PermissionDenied],
});

// 使用
let err = StorageError::not_found("/data/file.txt".to_string());
assert_eq!(err.category(), ErrorCategory::Resource);
assert!(err.is_file_related());
assert!(err.is_recoverable());
```

### 带自定义严重级别的错误类型

```rust
use crate::error::{ErrorCategory, ErrorSeverity, error_macros::*};

define_error_with_custom_severity!(
    #[derive(Debug, Clone, thiserror::Error)]
    pub enum SystemError {
        #[error("Out of memory")]
        OutOfMemory { resource: String } [Critical],

        #[error("Configuration error")]
        Configuration { message: String } [Error],

        #[error("Deprecated feature")]
        Deprecated { feature: String } [Warning],
    }
);

impl_error_categories!(SystemError, ErrorCategory::System);

impl_category_checks!(SystemError {
    is_memory_related => [OutOfMemory],
    is_config_related => [Configuration],
});

// 使用
let mem_err = SystemError::out_of_memory("Texture buffer".to_string());
assert_eq!(mem_err.severity(), ErrorSeverity::Critical);
assert!(!mem_err.is_recoverable());

// 覆盖严重级别
let custom_err = SystemError::with_severity(
    SystemError::configuration("Invalid port".to_string()),
    ErrorSeverity::Warning
);
assert_eq!(custom_err.severity(), ErrorSeverity::Warning);
```

## 优势

### 1. 减少重复代码

**不使用宏**：
```rust
pub enum MyError {
    NotFound { id: String, severity: ErrorSeverity },
    Failed { message: String, severity: ErrorSeverity },
}

impl MyError {
    pub fn not_found(id: String) -> Self {
        Self::NotFound {
            id,
            severity: ErrorSeverity::Error,
        }
    }

    pub fn failed(message: String) -> Self {
        Self::Failed {
            message,
            severity: ErrorSeverity::Error,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound { severity, .. } => *severity,
            Self::Failed { severity, .. } => *severity,
        }
    }

    pub fn is_recoverable(&self) -> bool {
        self.severity() < ErrorSeverity::Critical
    }
}
```

**使用宏**：
```rust
define_error_variants!(
    pub enum MyError {
        NotFound { id: String },
        Failed { message: String },
    }
);
// 自动生成所有构造函数和方法！
```

### 2. 一致性

所有错误类型都有一致的API：
- 构造函数命名：`variant_name()`
- 严重级别：`severity()`
- 可恢复性：`is_recoverable()`
- 分类：`category()`
- 分类检查：`is_*_related()`

### 3. 易于维护

添加新的错误变体时，只需在枚举定义中添加一行，宏会自动生成所有必要的方法。

### 4. 类型安全

宏在编译时展开，提供完整的类型检查和IDE支持。

## 最佳实践

### 1. 命名规范

- 错误类型：`<Module>Error`（例如 `AudioError`, `PhysicsError`）
- 构造函数：`snake_case`（例如 `not_found()`, `invalid_format()`）
- 分类检查：`is_<category>_related()`（例如 `is_file_related()`, `is_device_related()`）

### 2. 严重级别选择

- `Critical`: 不可恢复的错误（例如 `OutOfMemory`, `WorldNotInitialized`）
- `Error`: 普通错误（例如 `NotFound`, `LoadFailed`）
- `Warning`: 可恢复或次要问题（例如 `Deprecated`, `InvalidVolume`）

### 3. 错误分类

使用 `ErrorCategory` 枚举对错误进行分类：
- `Audio`: 音频系统错误
- `Physics`: 物理系统错误
- `Render`: 渲染系统错误
- `Resource`: 资源管理错误
- `Network`: 网络错误
- `System`: 系统级错误

### 4. 分层检查

提供多个级别的分类检查：
```rust
impl_category_checks!(MyError {
    // 通用检查
    is_file_related => [NotFound, LoadFailed, PermissionDenied],

    // 更具体的检查
    is_access_related => [PermissionDenied],
    is_io_related => [NotFound, LoadFailed],
});
```

## 迁移指南

### 现有错误类型迁移

**步骤1**：在错误文件顶部导入宏：
```rust
use crate::error::error_macros::*;
```

**步骤2**：使用 `define_error_variants!` 替换枚举定义

**步骤3**：添加 `impl_error_categories!`

**步骤4**：使用 `impl_category_checks!` 替换手动实现的检查方法

**步骤5**：删除重复的代码（构造函数、`severity()`、`is_recoverable()` 等）

**步骤6**：运行测试确保功能一致

### 示例迁移

**迁移前**：
```rust
pub enum AudioError {
    DeviceNotFound { device_name: String, severity: ErrorSeverity },
    // ... 更多变体
}

impl AudioError {
    pub fn device_not_found(device_name: String) -> Self { ... }
    pub fn severity(&self) -> ErrorSeverity { ... }
    pub fn is_recoverable(&self) -> bool { ... }
    pub fn is_device_related(&self) -> bool { ... }
}
```

**迁移后**：
```rust
define_error_variants!(
    pub enum AudioError {
        DeviceNotFound { device_name: String },
        // ... 更多变体
    }
);

impl_error_categories!(AudioError, ErrorCategory::Audio);

impl_category_checks!(AudioError {
    is_device_related => [DeviceNotFound, DeviceInitialization, DeviceConfiguration],
});
```

## 常见问题

### Q: 可以为同一个错误变体提供多个构造函数吗？

A: 目前宏只生成一个构造函数。如果需要多个构造函数，可以手动添加额外的构造函数作为 impl 块。

### Q: 如何处理带有非默认严重级别的错误？

A: 使用 `define_error_with_custom_severity!` 宏，或者在创建错误后使用 `with_severity()` 方法调整严重级别。

### Q: 分类检查方法可以返回更复杂的信息吗？

A: 宏生成的检查方法只返回 `bool`。如果需要更复杂的信息，可以手动实现额外的方法。

### Q: 可以与其他库集成吗？

A: 可以。宏生成的错误类型完全兼容 `thiserror`、`anyhow` 等错误处理库。

## 相关资源

- [Rust错误处理最佳实践](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror文档](https://docs.rs/thiserror/)
- [本项目错误架构](../src/error/README.md)
