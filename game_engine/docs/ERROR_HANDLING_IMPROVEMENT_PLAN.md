# 错误处理体系改进计划

## 执行日期
2025年12月27日

## 当前状态

### 优点 ✅
1. **完善的错误类型体系**
   - 8个模块级错误类型（Audio, Physics, Render, Resource, Input, System, Script, Platform）
   - 统一的 `EngineError` 作为错误入口
   - `ErrorSeverity` 和 `ErrorCategory` 分类

2. **良好的错误结构**
   - 使用 `thiserror` 自动派生
   - 包含详细上下文信息（message, severity, location）
   - 支持错误链（Chain variant）
   - 支持多错误聚合（Multiple variant）

3. **错误恢复机制**
   - `ErrorRecovery` trait
   - `RecoveryManager` 恢复管理器
   - `RetryExecutor` 重试机制

### 问题 ❌
1. **命名冲突**
   - `AudioError` 在 `src/error/audio_error.rs` 和 `src/domain/errors.rs` 重复定义
   - `PhysicsError` 在 `src/error/physics_error.rs` 和 `src/domain/errors.rs` 重复定义

2. **错误重复**
   - Domain层不应重新定义错误类型
   - 应该统一使用 `src/error/` 中的定义

3. **缺少辅助方法**
   - 添加错误上下文不够方便
   - 错误转换和传播可以更简洁

4. **缺少用户友好的错误消息**
   - 技术错误对终端用户不友好
   - 需要本地化和简化显示

## 改进方案

### 方案1: 统一错误类型定义 🔴 P0

**目标**：消除命名冲突，统一错误定义

**步骤**：
1. 修改 `src/domain/errors.rs`，移除重复的 `AudioError` 和 `PhysicsError`
2. 从 `src/error/` 重新导出这些类型
3. 更新所有使用方

**变更**：
```rust
// src/domain/errors.rs

// ❌ 删除
// pub enum AudioError { ... }
// pub enum PhysicsError { ... }

// ✅ 重新导出
pub use crate::error::{AudioError, PhysicsError};

// ✅ 保留Domain特有错误
pub enum DomainError {
    #[error("Audio domain error: {0}")]
    Audio(#[from] AudioError),  // 现在引用 crate::error::AudioError
    #[error("Physics domain error: {0}")]
    Physics(#[from] PhysicsError),  // 现在引用 crate::error::PhysicsError
    ...
}
```

**收益**：
- ✅ 消除命名冲突
- ✅ 统一错误定义
- ✅ 减少代码重复

### 方案2: 添加错误上下文辅助方法 🟠 P1

**目标**：方便地添加错误上下文

**实现**：在 `EngineError` 添加辅助方法

```rust
// src/error/engine_error.rs

impl EngineError {
    /// 添加上下文信息
    pub fn context(self, context: impl Into<String>) -> Self {
        Self::Chain {
            context: context.into(),
            source: Box::new(self),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Self::Chain { metadata, .. } = &mut self {
            metadata.insert(key.into(), value.into());
        }
        self
    }

    /// 设置错误位置
    pub fn at(self, location: impl Into<String>) -> Self {
        match self {
            Self::General { message, source, severity, backtrace, .. } => {
                Self::General {
                    message,
                    source,
                    severity,
                    location: Some(location.into()),
                    backtrace,
                }
            }
            _ => self.context(location),
        }
    }
}
```

**使用示例**：
```rust
// 之前
Err(EngineError::general("Failed to load texture"))

// 之后
Err(EngineError::general("Failed to load texture")
    .context("In scene initialization")
    .with_metadata("scene", "main_menu")
    .with_metadata("texture", "background.png")
    .at("src/scenes/menu.rs:42"))
```

**收益**：
- ✅ 更详细的错误上下文
- ✅ 更好的调试体验
- ✅ 流式API，易用

### 方案3: 添加用户友好的错误显示 🟡 P2

**目标**：提供用户友好的错误消息

**实现**：添加 `DisplayUser` trait

```rust
// src/error/user_friendly.rs

/// 用户友好的错误显示
pub trait DisplayUser {
    /// 获取用户友好的错误消息（简化技术细节）
    fn user_message(&self) -> String;

    /// 获取建议的恢复操作
    fn suggested_action(&self) -> Option<String>;
}

impl DisplayUser for EngineError {
    fn user_message(&self) -> String {
        match self {
            EngineError::Audio(AudioError::DeviceNotFound { .. }) => {
                "未找到音频设备。请检查您的音频设备是否已连接。".to_string()
            }
            EngineError::Resource(ResourceError::NotFound { path, .. }) => {
                format!("无法加载资源：{}", path)
            }
            _ => self.to_string(),  // 其他错误使用默认显示
        }
    }

    fn suggested_action(&self) -> Option<String> {
        match self {
            EngineError::Render(_) => {
                Some("请检查您的显卡驱动是否已更新。".to_string())
            }
            EngineError::Resource(ResourceError::NotFound { .. }) => {
                Some("请重新安装游戏或验证游戏文件完整性。".to_string())
            }
            _ => None,
        }
    }
}
```

**收益**：
- ✅ 用户友好的错误消息
- ✅ 多语言支持的基础
- ✅ 更好的用户体验

### 方案4: 错误转换宏 🟡 P2

**目标**：简化错误转换代码

**实现**：创建错误上下文宏

```rust
// src/error/macros.rs

#[macro_export]
macro_rules! error_context {
    ($err:expr, $ctx:expr) => {
        $err.context($ctx)
    };
}

#[macro_export]
macro_rules! error_at {
    ($err:expr, $file:expr, $line:expr) => {
        $err.at(concat!($file, ":", $line))
    };
}

#[macro_export]
macro_rules! bail {
    ($msg:expr) => {
        return Err(EngineError::general($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(EngineError::general(format!($fmt, $($arg)*)))
    };
}
```

**使用示例**：
```rust
// 之前
return Err(EngineError::general("Failed to connect to server"));

// 之后
bail!("Failed to connect to server");

// 带格式化
bail!("Failed to connect to {} after {} attempts", server, attempts);

// 添加上下文
let result = load_texture(path)
    .map_err(|e| error_context!(e, "Failed to load main menu texture"))?;
```

**收益**：
- ✅ 减少样板代码
- ✅ 提高代码可读性
- ✅ 统一错误处理模式

### 方案5: 错误追踪改进 🟢 P3

**目标**：增强错误追踪和诊断

**实现**：
1. 添加错误ID（便于日志搜索）
2. 添加错误时间戳
3. 添加错误统计

```rust
// src/error/engine_error.rs

use std::time::SystemTime;
use uuid::Uuid;

impl EngineError {
    /// 生成错误ID
    pub fn with_id(mut self) -> (Self, String) {
        let id = Uuid::new_v4().to_string();
        if let Self::General { ref mut message, .. } = &mut self {
            *message = format!("[{}] {}", id, message);
        }
        (self, id)
    }

    /// 记录错误发生时间
    pub fn with_timestamp(mut self) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if let Self::Chain { ref mut metadata, .. } = &mut self {
            metadata.insert("timestamp".to_string(), timestamp.to_string());
        }
        self
    }
}
```

**收益**：
- ✅ 便于日志追踪
- ✅ 便于错误统计
- ✅ 便于用户报告

## 实施优先级

| 方案 | 优先级 | 预计工时 | 收益 |
|------|--------|----------|------|
| 1. 统一错误类型 | 🔴 P0 | 2天 | 高 |
| 2. 错误上下文辅助 | 🟠 P1 | 1天 | 中 |
| 3. 用户友好显示 | 🟡 P2 | 3天 | 中 |
| 4. 错误转换宏 | 🟡 P2 | 1天 | 低 |
| 5. 错误追踪改进 | 🟢 P3 | 2天 | 低 |

## 实施步骤

### 第一步：统一错误类型（P0）

1. 修改 `src/domain/errors.rs`
2. 更新导入和使用
3. 运行测试验证

### 第二步：添加错误上下文辅助方法（P1）

1. 在 `EngineError` 中添加辅助方法
2. 更新文档和示例
3. 添加单元测试

### 第三步：实现用户友好显示（P2）

1. 创建 `DisplayUser` trait
2. 为主要错误类型实现该trait
3. 添加本地化支持框架

### 第四步：创建错误转换宏（P2）

1. 创建 `src/error/macros.rs`
2. 定义便捷宏
3. 更新使用示例

### 第五步：增强错误追踪（P3）

1. 添加错误ID支持
2. 添加时间戳支持
3. 集成到日志系统

## 验收标准

### P0 验收
- ✅ 无命名冲突
- ✅ `cargo check` 通过
- ✅ 所有测试通过
- ✅ 无错误重复

### P1 验收
- ✅ 错误上下文辅助方法可用
- ✅ 文档和示例完整
- ✅ 单元测试覆盖

### P2 验收
- ✅ `DisplayUser` trait 实现
- ✅ 错误宏定义完整
- ✅ 用户友好消息覆盖常见错误

### P3 验收
- ✅ 错误ID生成工作
- ✅ 时间戳记录工作
- ✅ 日志集成完成

## 风险和缓解

### 潜在风险

1. **破坏现有代码**
   - **缓解**：保持向后兼容，逐步迁移
   - **策略**：添加deprecated警告，保留旧API一段时间

2. **性能开销**
   - **缓解**：仅在Debug模式启用详细追踪
   - **策略**：使用cfg(debug_assertions)条件编译

3. **API变更**
   - **缓解**：新功能作为可选辅助方法
   - **策略**：不改变现有错误类型定义

## 长期目标

1. **多语言支持**
   - 基于用户友好显示的本地化框架
   - 翻译文件和热更新

2. **错误遥测**
   - 收集匿名错误统计
   - 帮助识别常见问题

3. **AI辅助错误恢复**
   - 基于错误历史的智能恢复建议
   - 自动问题诊断

## 总结

本改进计划旨在提升错误处理的五个方面：

1. **一致性**：统一错误类型定义
2. **易用性**：添加辅助方法和宏
3. **用户友好**：提供简化的错误消息
4. **可追踪性**：增强错误诊断能力
5. **可维护性**：减少重复，提高代码质量

通过这些改进，游戏引擎的错误处理将达到生产级标准。

---

**创建时间**: 2025年12月27日
**状态**: 📋 计划中
**下一步**: 实施P0优先级改进
