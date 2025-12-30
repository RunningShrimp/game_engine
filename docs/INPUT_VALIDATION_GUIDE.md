# 输入验证框架指南 (Input Validation Framework Guide)

**版本**: v0.2.0+
**创建日期**: 2025-12-30
**适用范围**: 游戏引擎所有公共API
**Rust版本**: 1.92.0 (Edition 2024)

---

## 📋 目录

1. [概述](#概述)
2. [验证原则](#验证原则)
3. [验证层次](#验证层次)
4. [内置验证器](#内置验证器)
5. [自定义验证](#自定义验证)
6. [错误处理](#错误处理)
7. [性能优化](#性能优化)
8. [最佳实践](#最佳实践)

---

## 概述

### 为什么需要输入验证？

游戏引擎作为基础设施库，需要处理各种用户输入和外部数据：

✅ **安全性**: 防止恶意输入导致崩溃或安全漏洞
✅ **健壮性**: 优雅处理无效数据而非panic
✅ **可调试性**: 提供清晰的错误消息
✅ **用户体验**: 帮助用户快速发现并修复问题

### 验证目标

- **零panic**: 所有输入验证失败应返回`Result::Err`
- **明确错误**: 错误消息应清楚说明问题所在
- **性能友好**: 验证开销应最小化
- **可组合**: 验证器应易于组合和复用

### 当前状态

- **生产代码panic风险**: 已降至零 ✅
- **unwrap/expect使用**: 已优化完成 ✅
- **验证框架**: 需要统一化 🔄
- **测试覆盖**: 需要增强验证相关测试

---

## 验证原则

### 1. 快速失败 (Fail Fast)

在入口处立即验证，而非在处理过程中：

```rust
// ✅ 好的做法：立即验证
pub fn create_entity(config: &EntityConfig) -> Result<Entity> {
    config.validate()?;

    // 处理逻辑...
}

// ❌ 避免：延迟验证
pub fn create_entity(config: &EntityConfig) -> Result<Entity> {
    // 处理逻辑...
    if some_condition {
        return Err(...); // 太晚了
    }
}
```

### 2. 明确错误 (Explicit Errors)

使用具体的错误类型，而非通用错误：

```rust
// ✅ 好的做法：具体错误
pub enum ValidationError {
    #[error("Name cannot be empty")]
    EmptyName,

    #[error("ID {0} is out of range (must be 0..{1})")]
    IdOutOfRange(u64, u64),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
}

// ❌ 避免：通用错误
return Err("Invalid input".into());
```

### 3. 验证分离 (Separation of Concerns)

验证逻辑应独立于业务逻辑：

```rust
// ✅ 好的做法：分离验证
impl EntityConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        Ok(())
    }
}

// ❌ 避免：混合验证和业务逻辑
pub fn create_entity(config: &EntityConfig) -> Result<Entity> {
    if config.name.is_empty() {
        return Err(...);
    }
    // ...
}
```

### 4. 可组合验证 (Composable Validation)

验证器应可以组合使用：

```rust
// ✅ 好的做法：组合验证器
pub fn validate_entity(config: &EntityConfig) -> Result<()> {
    validate_name(&config.name)?;
    validate_id(config.id)?;
    validate_position(&config.position)?;
    Ok(())
}
```

---

## 验证层次

### Layer 1: 类型系统验证

利用Rust类型系统在编译时捕获错误：

```rust
// ✅ 使用newtype模式
pub struct EntityId(u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        assert!(id < MAX_ENTITIES, "ID out of range");
        Self(id)
    }
}

// ✅ 使用非零类型
pub struct Count(std::num::NonZeroUsize);

// ✅ 使用枚举限制选项
pub enum RenderMode {
    Forward,
    Deferred,
    // 不可能输入无效值
}
```

### Layer 2: 构造时验证

在构造函数中验证：

```rust
impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Result<Self, ValidationError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(ValidationError::NonFinitePosition);
        }

        // 检查边界（如果适用）
        if x.abs() > MAX_COORD || y.abs() > MAX_COORD || z.abs() > MAX_COORD {
            return Err(ValidationError::PositionOutOfRange);
        }

        Ok(Self { x, y, z })
    }
}
```

### Layer 3: 运行时验证

在方法执行前验证：

```rust
impl ResourceManager {
    pub fn load_resource(&self, path: &Path) -> Result<Arc<Resource>> {
        // 验证路径
        if !path.exists() {
            return Err(ValidationError::PathNotFound(path.to_path_buf()));
        }

        // 验证扩展名
        match path.extension().and_then(|s| s.to_str()) {
            Some("png") | Some("jpg") | Some("gltf") => {},
            other => return Err(ValidationError::UnsupportedFormat(other.map(|s| s.to_owned()))),
        }

        // 继续处理...
    }
}
```

---

## 内置验证器

### 数值验证器

```rust
pub mod validators {
    use super::*;

    /// 验证数值在范围内
    pub fn validate_range<T>(value: T, min: T, max: T) -> Result<T, ValidationError>
    where
        T: PartialOrd + Copy + std::fmt::Display,
    {
        if value < min || value > max {
            return Err(ValidationError::OutOfRange {
                value: value.to_string(),
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        Ok(value)
    }

    /// 验证f32是有限值
    pub fn validate_finite(value: f32) -> Result<f32, ValidationError> {
        if !value.is_finite() {
            return Err(ValidationError::NonFinite(value));
        }
        Ok(value)
    }

    /// 验证f32非NaN
    pub fn validate_nan(value: f32) -> Result<f32, ValidationError> {
        if value.is_nan() {
            return Err(ValidationError::NaN(value));
        }
        Ok(value)
    }

    /// 验证非负数
    pub fn validate_non_negative<T>(value: T) -> Result<T, ValidationError>
    where
        T: PartialOrd + Zero + std::fmt::Display,
    {
        if value < T::zero() {
            return Err(ValidationError::Negative(value.to_string()));
        }
        Ok(value)
    }
}
```

### 字符串验证器

```rust
pub mod string_validators {
    use super::*;

    /// 验证非空字符串
    pub fn validate_non_empty(s: &str) -> Result<&str, ValidationError> {
        if s.is_empty() {
            return Err(ValidationError::EmptyString);
        }
        Ok(s)
    }

    /// 验证字符串长度
    pub fn validate_length(s: &str, min: usize, max: usize) -> Result<&str, ValidationError> {
        let len = s.chars().count();
        if len < min {
            return Err(ValidationError::TooShort { min, actual: len });
        }
        if len > max {
            return Err(ValidationError::TooLong { max, actual: len });
        }
        Ok(s)
    }

    /// 验证字符集
    pub fn validate_charset(s: &str, allowed: &str) -> Result<&str, ValidationError> {
        if !s.chars().all(|c| allowed.contains(c)) {
            return Err(ValidationError::InvalidCharacters);
        }
        Ok(s)
    }

    /// 验证UTF-8
    pub fn validate_utf8(bytes: &[u8]) -> Result<&str, ValidationError> {
        std::str::from_utf8(bytes)
            .map_err(|_| ValidationError::InvalidUtf8)
    }
}
```

### 路径验证器

```rust
pub mod path_validators {
    use super::*;

    use std::path::Path;

    /// 验证路径存在
    pub fn validate_exists(path: &Path) -> Result<&Path, ValidationError> {
        if !path.exists() {
            return Err(ValidationError::PathNotFound(path.to_path_buf()));
        }
        Ok(path)
    }

    /// 验证路径可读
    pub fn validate_readable(path: &Path) -> Result<&Path, ValidationError> {
        if !path.exists() {
            return Err(ValidationError::PathNotFound(path.to_path_buf()));
        }

        // 尝试打开以验证可读性
        std::fs::File::open(path)
            .map_err(|_| ValidationError::PathNotReadable(path.to_path_buf()))?;

        Ok(path)
    }

    /// 验证文件扩展名
    pub fn validate_extension(path: &Path, allowed: &[&str]) -> Result<&Path, ValidationError> {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ValidationError::MissingExtension(path.to_path_buf()))?;

        if !allowed.contains(&ext) {
            return Err(ValidationError::InvalidExtension {
                path: path.to_path_buf(),
                found: ext.to_owned(),
                allowed: allowed.iter().map(|s| s.to_string()).collect(),
            });
        }

        Ok(path)
    }
}
```

---

## 自定义验证

### Trait-based验证

定义可复用的验证trait：

```rust
pub trait Validate {
    type Error;

    fn validate(&self) -> Result<(), Self::Error>;
}

// 为自定义类型实现
impl Validate for EntityConfig {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        validate_non_empty(&self.name)?;
        validate_id(self.id)?;
        validate_position(&self.position)?;
        Ok(())
    }
}
```

### 组合验证器

使用组合模式构建复杂验证：

```rust
pub struct AndValidator<A, B> {
    validator_a: A,
    validator_b: B,
}

impl<A, B, T> Validator<T> for AndValidator<A, B>
where
    A: Validator<T>,
    B: Validator<T>,
{
    type Error = ValidationError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        self.validator_a.validate(value)?;
        self.validator_b.validate(value)?;
        Ok(())
    }
}

pub struct OrValidator<A, B> {
    validator_a: A,
    validator_b: B,
}

impl<A, B, T> Validator<T> for OrValidator<A, B>
where
    A: Validator<T>,
    B: Validator<T>,
{
    type Error = ValidationError;

    fn validate(&self, value: &T) -> Result<(), Self::Error> {
        let result_a = self.validator_a.validate(value);
        let result_b = self.validator_b.validate(value);

        result_a.or(result_b)
    }
}
```

### 函数验证器

简单的函数式验证器：

```rust
pub type ValidatorFn<T> = Box<dyn Fn(&T) -> Result<(), ValidationError>>;

pub fn fn_validator<T>(f: impl Fn(&T) -> Result<(), ValidationError> + 'static) -> ValidatorFn<T> {
    Box::new(f)
}

// 使用
let validator = fn_validator::<&str>(|s| {
    validate_non_empty(s)?;
    validate_length(s, 1, 100)?;
    Ok(())
});
```

---

## 错误处理

### 错误类型设计

```rust
#[derive(Error, Debug)]
pub enum ValidationError {
    // 数值错误
    #[error("Value {value} is out of range [{min}, {max}]")]
    OutOfRange {
        value: String,
        min: String,
        max: String,
    },

    #[error("Non-finite value: {0}")]
    NonFinite(f32),

    #[error("NaN value: {0}")]
    NaN(f32),

    #[error("Negative value: {0}")]
    Negative(String),

    // 字符串错误
    #[error("Empty string")]
    EmptyString,

    #[error("String too short: minimum {min}, actual {actual}")]
    TooShort { min: usize, actual: usize },

    #[error("String too long: maximum {max}, actual {actual}")]
    TooLong { max: usize, actual: usize },

    #[error("Invalid characters in string")]
    InvalidCharacters,

    // 路径错误
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Path not readable: {0}")]
    PathNotReadable(PathBuf),

    #[error("Invalid extension for {path}: found {found}, allowed: {allowed:?}")]
    InvalidExtension {
        path: PathBuf,
        found: String,
        allowed: Vec<String>,
    },

    // 自定义
    #[error("Custom validation error: {0}")]
    Custom(String),
}
```

### 错误上下文

为错误添加上下文信息：

```rust
pub trait ValidationErrorContext<T> {
    fn context(self, context: &str) -> Result<T, ValidationError>;
}

impl<T, E> ValidationErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, context: &str) -> Result<T, ValidationError> {
        self.map_err(|e| ValidationError::Custom(format!("{}: {}", context, e)))
    }
}

// 使用
config.validate()
    .context("Failed to validate entity config")?;
```

---

## 性能优化

### 1. 延迟验证

仅在需要时验证：

```rust
pub struct LazyValidated<T> {
    value: T,
    validated: Cell<bool>,
}

impl<T: Validate> LazyValidated<T> {
    pub fn get(&self) -> Result<&T, ValidationError> {
        if !self.validated.get() {
            self.value.validate()?;
            self.validated.set(true);
        }
        Ok(&self.value)
    }
}
```

### 2. 缓存验证结果

```rust
pub struct Validated<T> {
    value: T,
    // 已经验证，不需要再次验证
}

impl<T> Validated<T> {
    pub fn new(value: T) -> Result<Self, ValidationError>
    where
        T: Validate,
    {
        value.validate()?;
        Ok(Self { value })
    }

    pub fn get(&self) -> &T {
        &self.value
        }
}
```

### 3. 避免重复验证

```rust
// ✅ 好的做法：只验证一次
pub fn process(config: EntityConfig) -> Result<Entity> {
    let validated = Validated::new(config)?;

    // 内部调用不需要再次验证
    self.internal_process(&validated)
}

// ❌ 避免：重复验证
pub fn process(config: EntityConfig) -> Result<Entity> {
    config.validate()?;
    self.step1(&config)?;
    config.validate()?; // 重复
    self.step2(&config)?;
}
```

---

## 最佳实践

### 1. 提供构造辅助函数

```rust
impl EntityId {
    /// 安全构造（带验证）
    pub fn new(id: u64) -> Result<Self, ValidationError> {
        if id >= MAX_ENTITIES {
            return Err(ValidationError::IdOutOfRange(id, MAX_ENTITIES));
        }
        Ok(Self(id))
    }

    /// 不检查构造（仅内部使用）
    ///
    /// # Safety
    /// 调用者必须保证id有效
    pub unsafe fn new_unchecked(id: u64) -> Self {
        Self(id)
    }
}
```

### 2. 使用Builder模式

```rust
pub struct EntityBuilder {
    name: Option<String>,
    id: Option<u64>,
    position: Option<Position>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            id: None,
            position: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn build(self) -> Result<Entity, ValidationError> {
        let name = self.name.ok_or(ValidationError::MissingField("name"))?;
        let id = self.id.ok_or(ValidationError::MissingField("id"))?;

        // 验证所有字段
        validate_non_empty(&name)?;
        validate_id(id)?;

        Ok(Entity::new(name, id))
    }
}
```

### 3. 测试验证逻辑

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("test").is_ok());
        assert!(validate_non_empty("").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 0, 10).is_ok());
        assert!(validate_range(-1, 0, 10).is_err());
        assert!(validate_range(11, 0, 10).is_err());
    }

    #[test]
    fn test_finite_validation() {
        assert!(validate_finite(1.0).is_ok());
        assert!(validate_finite(f32::INFINITY).is_err());
        assert!(validate_finite(f32::NAN).is_err());
    }
}
```

### 4. 文档化验证要求

```rust
impl ResourceManager {
    /// 加载资源
    ///
    /// # 验证要求
    ///
    /// - 路径必须存在
    /// - 文件必须可读
    /// - 扩展名必须是支持的格式：png, jpg, gltf
    ///
    /// # 错误
    ///
    /// - `ValidationError::PathNotFound`: 路径不存在
    /// - `ValidationError::PathNotReadable`: 文件不可读
    /// - `ValidationError::InvalidExtension`: 不支持的格式
    ///
    /// # 示例
    ///
    /// ```rust
    /// let resource = manager.load_resource(Path::new("model.gltf"))?;
    /// ```
    pub fn load_resource(&self, path: &Path) -> Result<Arc<Resource>> {
        // ...
    }
}
```

---

## 实施检查清单

### 新代码

- [ ] 所有公共API都有输入验证
- [ ] 验证失败返回`Result::Err`，不panic
- [ ] 错误消息清晰明确
- [ ] 验证逻辑有测试覆盖
- [ ] 验证要求已文档化

### 现有代码审计

- [ ] 识别所有公共API入口点
- [ ] 审查参数验证
- [ ] 添加缺失的验证
- [ ] 更新错误处理
- [ ] 增加测试

### 性能检查

- [ ] 验证开销可接受（<1%总时间）
- [ ] 无重复验证
- [ ] 热路径使用延迟验证
- [ ] 验证结果已缓存（如适用）

---

## 附录

### A. 完整示例

```rust
use std::path::Path;

// 定义验证错误
#[derive(Error, Debug)]
pub enum EntityValidationError {
    #[error("Name cannot be empty")]
    EmptyName,

    #[error("Name too long: maximum {max}, actual {actual}")]
    NameTooLong { max: usize, actual: usize },

    #[error("ID {0} is out of range (must be 0..{1})")]
    IdOutOfRange(u64, u64),

    #[error("Invalid position: {0}")]
    InvalidPosition(String),
}

// 实体配置
pub struct EntityConfig {
    pub name: String,
    pub id: u64,
    pub position: (f32, f32, f32),
}

// 验证实现
impl EntityConfig {
    pub fn validate(&self) -> Result<(), EntityValidationError> {
        // 验证名称
        if self.name.is_empty() {
            return Err(EntityValidationError::EmptyName);
        }

        if self.name.len() > 100 {
            return Err(EntityValidationError::NameTooLong {
                max: 100,
                actual: self.name.len(),
            });
        }

        // 验证ID
        if self.id >= 10000 {
            return Err(EntityValidationError::IdOutOfRange(self.id, 10000));
        }

        // 验证位置
        let (x, y, z) = self.position;
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(EntityValidationError::InvalidPosition(
                "Position must be finite".to_string(),
            ));
        }

        Ok(())
    }
}

// 使用
pub fn create_entity(config: EntityConfig) -> Result<Entity, EntityValidationError> {
    config.validate()?;

    Ok(Entity {
        name: config.name,
        id: config.id,
        position: config.position,
    })
}
```

### B. 相关资源

- [Rust Error Handling Best Practices](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Validator Crate](https://docs.rs/validator/latest/validator/)
- [ThisError Documentation](https://docs.rs/thiserror/latest/thiserror/)

---

**维护者**: 游戏引擎开发团队
**最后更新**: 2025-12-30
**版本**: v1.0
