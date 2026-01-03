# 代码去重重构迁移指南

**版本**: 1.0
**日期**: 2026-01-02
**面向**: 开发者

---

## 目录

1. [快速开始](#快速开始)
2. [错误类型重构](#错误类型重构)
3. [构造函数重构](#构造函数重构)
4. [平台代码重构](#平台代码重构)
5. [最佳实践](#最佳实践)
6. [常见问题](#常见问题)
7. [检查清单](#检查清单)

---

## 快速开始

### 1.1 迁移前准备

```bash
# 1. 创建feature分支
git checkout -b feature/code-deduplication-refactoring

# 2. 建立测试基线
cargo test --all 2>&1 | tee test_baseline.log
cargo bench --all 2>&1 | tee bench_baseline.log

# 3. 记录代码行数
find game_engine/src -name "*.rs" -exec wc -l {} + | tail -1 > lines_baseline.txt
```

### 1.2 工具导入

```rust
// 错误处理宏
use game_engine::error::simple_error;
use game_engine::error::standard_error;
use game_engine::error::field_error;
use game_engine::error::combined_error;

// 构造函数宏
use game_engine::core::constructor::simple_new;
use game_engine::core::constructor::new_with_defaults;
use game_engine::core::constructor::builder;

// Platform trait
use game_engine::platform::traits::{Platform, current_platform};
```

---

## 错误类型重构

### 2.1 识别可重构的错误类型

**检查清单**:

- [ ] 错误类型包含3-10个变体
- [ ] 大多数变体是简单的包装器（如`Io(std::io::Error)`）
- [ ] 错误变体使用`#[error]`属性
- [ ] 没有复杂的自定义逻辑

**不适合重构的情况**:
- ❌ 错误类型包含复杂的自定义方法
- ❌ 错误变体有特殊的数据转换逻辑
- ❌ 错误类型是公共API的一部分（需要保持稳定）

### 2.2 simple_error! 使用指南

**适用场景**: 简单的错误包装器

**之前**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
```

**之后**:
```rust
simple_error! {
    #[derive(Debug)]
    pub enum MyError {
        #[error("IO error: {0}")]
        Io: std::io::Error,

        #[error("Parse error: {0}")]
        Parse: String,

        #[error("Not found: {0}")]
        NotFound: String,
    }
}
```

**优点**:
- ✅ 自动生成`From<std::io::Error>`实现
- ✅ 减少样板代码
- ✅ 保持API兼容

### 2.3 standard_error! 使用指南

**适用场景**: 标准错误变体（Io, Parse, NotFound, Invalid, Other）

**之前**:
```rust
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    Invalid(String),

    #[error("Other error: {0}")]
    Other(String),
}
```

**之后**:
```rust
standard_error! {
    #[derive(Debug)]
    pub enum ResourceError
}
```

**优点**:
- ✅ 一行代码定义完整错误类型
- ✅ 包含所有常见错误变体
- ✅ 自动生成From实现

### 2.4 field_error! 使用指南

**适用场景**: 带自定义字段的错误

**之前**:
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing config: {key}")]
    Missing { key: String },

    #[error("Invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },

    #[error("Type mismatch for {key}: expected {expected}, got {actual}")]
    TypeMismatch { key: String, expected: String, actual: String },
}
```

**之后**:
```rust
field_error! {
    #[derive(Debug)]
    pub enum ConfigError {
        #[error("Missing config: {key}")]
        Missing { key: String },

        #[error("Invalid value for {key}: {value}")]
        InvalidValue { key: String, value: String },

        #[error("Type mismatch for {key}: expected {expected}, got {actual}")]
        TypeMismatch { key: String, expected: String, actual: String },
    }
}
```

**优点**:
- ✅ 支持命名字段
- ✅ 保持错误消息格式
- ✅ 减少重复代码

### 2.5 combined_error! 使用指南

**适用场景**: 组合多个错误类型

**之前**:
```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource error: {0}")]
    Resource(ResourceError),

    #[error("Network error: {0}")]
    Network(NetworkError),

    #[error("Config error: {0}")]
    Config(ConfigError),
}
```

**之后**:
```rust
combined_error! {
    #[derive(Debug)]
    pub enum AppError {
        #[error("Resource error: {0}")]
        Resource: ResourceError,

        #[error("Network error: {0}")]
        Network: NetworkError,

        #[error("Config error: {0}")]
        Config: ConfigError,
    }
}
```

**优点**:
- ✅ 统一错误处理
- ✅ 自动生成Display实现
- ✅ 减少样板代码

---

## 构造函数重构

### 3.1 simple_new! 使用指南

**适用场景**: 带参数的简单构造函数

**之前**:
```rust
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
```

**之后**:
```rust
simple_new! {
    pub struct Point {
        pub x: f32,
        pub y: f32,
    }
}
```

**优点**:
- ✅ 自动生成构造函数
- ✅ 减少样板代码
- ✅ 保持API兼容

### 3.2 new_with_defaults! 使用指南

**适用场景**: 带默认值的配置结构

**之前**:
```rust
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub vsync: bool,
}

impl WindowConfig {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            title: "My Window".to_string(),
            vsync: true,
        }
    }

    pub fn with_values(width: u32, height: u32, title: String, vsync: bool) -> Self {
        Self { width, height, title, vsync }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::new()
    }
}
```

**之后**:
```rust
new_with_defaults! {
    pub struct WindowConfig {
        pub width: u32 = 1920,
        pub height: u32 = 1080,
        pub title: String = String::from("My Window"),
        pub vsync: bool = true,
    }
}
```

**优点**:
- ✅ 自动生成`new()`和`with_values()`
- ✅ 自动实现`Default`
- ✅ 减少大量样板代码

### 3.3 builder! 使用指南

**适用场景**: 复杂配置结构，需要可选参数

**之前**:
```rust
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub max_connections: usize,
}

pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
    max_connections: Option<usize>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            timeout: None,
            max_connections: None,
        }
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("Missing host")?,
            port: self.port.ok_or("Missing port")?,
            timeout: self.timeout.ok_or("Missing timeout")?,
            max_connections: self.max_connections.ok_or("Missing max_connections")?,
        })
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}
```

**之后**:
```rust
builder! {
    pub struct Config {
        pub host: String,
        pub port: u16,
        pub timeout: u64,
        pub max_connections: usize,
    }
}
```

**使用**:
```rust
let config = Config::builder()
    .host("localhost".to_string())
    .port(8080)
    .timeout(30)
    .max_connections(100)
    .build()?;
```

**优点**:
- ✅ 自动生成完整Builder模式
- ✅ 减少约80行样板代码
- ✅ 类型安全的构建过程

---

## 平台代码重构

### 4.1 识别条件编译模式

**需要重构的模式**:
- ❌ `#[cfg(target_os = "windows")]`
- ❌ `#[cfg(target_os = "macos")]`
- ❌ `#[cfg(target_os = "linux")]`
- ❌ `#[cfg(target_arch = "wasm32")]`

**应该使用**:
- ✅ `Platform` trait
- ✅ `current_platform()`
- ✅ 运行时平台检测

### 4.2 文件系统路径重构

**之前**:
```rust
#[cfg(target_os = "windows")]
fn get_app_data_dir() -> PathBuf {
    PathBuf::from("C:\\ProgramData")
}

#[cfg(target_os = "macos")]
fn get_app_data_dir() -> PathBuf {
    PathBuf::from("/Library/Application Support")
}

#[cfg(target_os = "linux")]
fn get_app_data_dir() -> PathBuf {
    PathBuf::from("/usr/share")
}
```

**之后**:
```rust
use game_engine::platform::traits::{current_platform, Platform};

fn get_app_data_dir() -> Result<PathBuf, String> {
    current_platform().app_data_dir()
}
```

**优点**:
- ✅ 减少条件编译
- ✅ 提高可测试性
- ✅ 运行时灵活性

### 4.3 平台能力检测重构

**之前**:
```rust
#[cfg(target_os = "ios")]
fn supports_touch() -> bool {
    true
}

#[cfg(not(target_os = "ios"))]
fn supports_touch() -> bool {
    false
}
```

**之后**:
```rust
use game_engine::platform::traits::{current_platform, Platform};

fn supports_touch() -> bool {
    current_platform().supports_touch()
}
```

**优点**:
- ✅ 消除重复代码
- ✅ 统一平台接口
- ✅ 易于扩展新平台

### 4.4 路径分隔符处理

**之前**:
```rust
#[cfg(target_os = "windows")]
const PATH_SEP: char = '\\';

#[cfg(not(target_os = "windows"))]
const PATH_SEP: char = '/';
```

**之后**:
```rust
use game_engine::platform::traits::{current_platform, Platform};

fn join_path(parts: &[&str]) -> String {
    let platform = current_platform();
    let sep = platform.path_separator();
    parts.join(&sep.to_string())
}
```

---

## 最佳实践

### 5.1 重构顺序

**推荐顺序**:
1. **错误类型**（低风险）
   - 从内部模块开始
   - 逐步扩展到公共API
   - 保持错误消息不变

2. **构造函数**（中风险）
   - 从私有类型开始
   - 逐步处理公共类型
   - 保持构造函数签名不变

3. **平台代码**（高风险）
   - 最后处理
   - 充分测试
   - 分步骤重构

### 5.2 测试策略

**每批重构后**:
```bash
# 1. 编译检查
cargo check --all-targets --all-features

# 2. 单元测试
cargo test --package game_engine --lib

# 3. 集成测试
cargo test --all

# 4. Clippy检查
cargo clippy --all-targets --all-features

# 5. 性能基准
cargo bench --package game_engine
```

### 5.3 代码审查清单

**错误类型重构**:
- [ ] 错误变体名称保持不变
- [ ] 错误消息格式一致
- [ ] `From`实现正确生成
- [ ] 所有测试通过
- [ ] 无Clippy警告

**构造函数重构**:
- [ ] 构造函数签名保持不变
- [ ] `Default`实现正确
- [ ] Builder模式完整
- [ ] 所有测试通过
- [ ] 性能无回归

**平台代码重构**:
- [ ] 条件编译正确移除
- [ ] 运行时行为一致
- [ ] 所有平台测试通过
- [ ] 性能无明显下降
- [ ] 文档更新完成

### 5.4 版本控制

**推荐策略**:
```bash
# 每批重构独立提交
git add game_engine/src/resources/
git commit -m "refactor(resources): 使用simple_error!重构错误类型

- 重构ObjLoader, TextureLoader, ShaderLoader
- 减少约200行重复代码
- 所有测试通过
"

# 使用详细的commit消息
git commit -m "refactor(module): 简短描述

详细说明：
- 做了什么改动
- 为什么改动
- 影响范围
- 测试结果

Ref: #issue-number"
```

---

## 常见问题

### Q1: 宏编译失败怎么办？

**问题**:
```
error: macro expansion ignores token ...
```

**解决方案**:
1. 检查宏语法是否正确
2. 确保所有必需的属性都已添加
3. 查看宏定义的文档和示例
4. 如果宏不支持，可以继续手写代码

### Q2: 如何保持API兼容性？

**原则**:
- ✅ 保持错误变体名称不变
- ✅ 保持构造函数签名不变
- ✅ 保持公共方法签名不变
- ✅ 可以添加新的辅助方法

**示例**:
```rust
// 旧API保持不变
pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
}

// 可以添加新的便捷方法
pub fn origin() -> Self {
    Self::new(0, 0)
}
```

### Q3: 性能会受影响吗？

**答案**: 不会

**原因**:
- ✅ 宏在编译时展开，无运行时开销
- ✅ `Platform` trait可能引入少量虚函数调用，但影响可忽略
- ✅ 编译器会优化掉抽象层

**验证**:
```bash
# 重构前后对比
cargo bench --all | tee before.txt
# ... 进行重构 ...
cargo bench --all | tee after.txt
diff before.txt after.txt
```

### Q4: 如何处理复杂的错误类型？

**原则**: 只重构简单的错误类型

**复杂错误类型示例**:
```rust
// 这种错误类型不适合用宏重构
#[derive(Debug, Error)]
pub enum ComplexError {
    #[error("Custom error with logic: {0}")]
    Custom(String),

    #[error("Error with conversion")]
    WithConversion {
        value: String,
        #[from]
        source: std::io::Error,
    },
}

impl ComplexError {
    // 自定义逻辑
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Custom(..))
    }

    pub fn into_details(self) -> ErrorDetails {
        // 复杂的转换逻辑
        // ...
    }
}
```

**建议**: 继续手写，不要强制使用宏

### Q5: 如何处理循环依赖？

**问题**: 错误类型互相引用

**解决方案**:
```rust
// 使用Box打破循环
combined_error! {
    pub enum Error {
        #[error("Network error: {0}")]
        Network: Box<NetworkError>,

        #[error("IO error: {0}")]
        Io: Box<IoError>,
    }
}
```

---

## 检查清单

### 6.1 重构前检查

- [ ] 阅读完整的迁移指南
- [ ] 创建feature分支
- [ ] 建立测试基线
- [ ] 备份当前代码
- [ ] 通知团队成员

### 6.2 重构中检查

- [ ] 每批重构独立编译
- [ ] 运行相关测试
- [ ] 检查Clippy警告
- [ ] 验证API兼容性
- [ ] 更新文档注释

### 6.3 重构后检查

- [ ] 所有测试通过
- [ ] 性能基准无回归
- [ ] 代码审查完成
- [ ] 文档更新完成
- [ ] 创建Pull Request

### 6.4 发布前检查

- [ ] CHANGELOG更新
- [ ] 版本号更新
- [ ] 发布说明准备
- [ ] 迁移指南发布
- [ ] 示例代码更新

---

## 示例项目

### 7.1 完整示例：错误类型重构

**文件**: `game_engine/src/resources/texture_loader.rs`

**之前**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    ImageDecode(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Texture too large: {size} bytes (max: {max})")]
    TextureTooLarge { size: usize, max: usize },
}
```

**之后**:
```rust
simple_error! {
    #[derive(Debug)]
    pub enum TextureLoadError {
        #[error("IO error: {0}")]
        Io: std::io::Error,

        #[error("Image decode error: {0}")]
        ImageDecode: String,

        #[error("Unsupported format: {0}")]
        UnsupportedFormat: String,
    }
}

field_error! {
    #[derive(Debug)]
    pub enum TextureLoadErrorFields {
        #[error("Invalid dimensions: {width}x{height}")]
        InvalidDimensions { width: u32, height: u32 },

        #[error("Texture too large: {size} bytes (max: {max})")]
        TextureTooLarge { size: usize, max: usize },
    }
}

combined_error! {
    #[derive(Debug)]
    pub enum TextureLoadError {
        #[error("IO error: {0}")]
        Io: std::io::Error,

        #[error("Image decode error: {0}")]
        ImageDecode: String,

        #[error("Unsupported format: {0}")]
        UnsupportedFormat: String,

        #[error("Dimensions error: {0}")]
        InvalidDimensions: TextureDimensionsError,

        #[error("Size error: {0}")]
        TextureTooLarge: TextureSizeError,
    }
}
```

### 7.2 完整示例：构造函数重构

**文件**: `game_engine/src/config/render.rs`

**之前**:
```rust
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub samples: u8,
    pub power_preference: PowerPreference,
}

impl RenderConfig {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            vsync: true,
            samples: 4,
            power_preference: PowerPreference::HighPerformance,
        }
    }

    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn with_samples(mut self, samples: u8) -> Self {
        self.samples = samples;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::new()
    }
}
```

**之后**:
```rust
builder! {
    pub struct RenderConfig {
        pub width: u32,
        pub height: u32,
        pub vsync: bool,
        pub samples: u8,
        pub power_preference: PowerPreference,
    }
}

// 使用
let config = RenderConfig::builder()
    .width(1920)
    .height(1080)
    .vsync(true)
    .samples(4)
    .power_preference(PowerPreference::HighPerformance)
    .build()?;
```

### 7.3 完整示例：平台代码重构

**文件**: `game_engine/src/platform/file_system.rs`

**之前**:
```rust
#[cfg(target_os = "windows")]
pub fn get_config_dir() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
}

#[cfg(target_os = "macos")]
pub fn get_config_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|p| PathBuf::from(p).join("Library/Application Support"))
        .unwrap_or_else(|_| PathBuf::from("/Library/Application Support"))
}

#[cfg(target_os = "linux")]
pub fn get_config_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|p| PathBuf::from(p).join(".config"))
        .unwrap_or_else(|_| PathBuf::from("/etc"))
}
```

**之后**:
```rust
use crate::platform::traits::{current_platform, Platform};

pub fn get_config_dir() -> PathBuf {
    current_platform()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("./config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = get_config_dir();
        assert!(dir.exists());
    }
}
```

---

## 资源

### 8.1 相关文档

- [重构计划](./CODE_DEDUPLICATION_REFACTORING_PLAN.md)
- [执行报告](./CODE_DEDUPLICATION_REFACTORING_EXECUTION_REPORT.md)
- [错误处理文档](./game_engine/src/error/README.md)
- [Platform trait文档](./game_engine/src/platform/traits.rs)

### 8.2 工具文档

- [simple_macros.rs](./game_engine/src/error/simple_macros.rs) - 错误处理宏
- [constructor.rs](./game_engine/src/core/constructor.rs) - 构造函数宏
- [traits.rs](./game_engine/src/platform/traits.rs) - Platform trait

### 8.3 示例代码

- [错误示例](./game_engine/src/error/simple_macros.rs#tests) - 错误宏测试
- [构造函数示例](./game_engine/src/core/constructor.rs#tests) - 构造函数宏测试
- [Platform示例](./game_engine/src/platform/traits.rs#tests) - Platform trait测试

---

## 贡献

如果你发现问题或有改进建议，请：

1. 创建issue描述问题
2. 提交Pull Request
3. 包含测试和文档

---

**文档版本**: 1.0
**最后更新**: 2026-01-02
**维护者**: Game Engine Team
