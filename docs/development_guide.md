# 开发规范指南

## 概述

本文档定义了游戏引擎的代码风格、开发流程和最佳实践。

## 代码风格

### 命名规范

#### 类型和结构体

```rust
// 使用PascalCase
pub struct GameEntity {
    // ...
}

pub enum RenderMode {
    // ...
}
```

#### 函数和方法

```rust
// 使用snake_case
pub fn update_transform() {
    // ...
}

impl GameEntity {
    pub fn get_position(&self) -> Vec3 {
        // ...
    }
}
```

#### 常量和静态变量

```rust
// 使用SCREAMING_SNAKE_CASE
pub const MAX_ENTITIES: usize = 10000;
pub static GLOBAL_CONFIG: Config = Config::default();
```

#### 模块和文件

```rust
// 使用snake_case
// 文件名: render_system.rs
pub mod render_system;
```

### 代码格式

使用`rustfmt`自动格式化代码：

```bash
cargo fmt --all
```

### 注释规范

#### 文档注释

```rust
/// 计算两个向量的点积
///
/// # 参数
/// * `a` - 第一个向量
/// * `b` - 第二个向量
///
/// # 返回
/// 返回点积结果
///
/// # 示例
/// ```
/// let result = dot_product(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));
/// assert_eq!(result, 32.0);
/// ```
pub fn dot_product(a: Vec3, b: Vec3) -> f32 {
    // ...
}
```

#### 行内注释

```rust
// 使用行内注释解释复杂逻辑
let result = complex_calculation(); // 注意：这里需要特殊处理边界情况
```

### 错误处理

```rust
// 使用Result类型处理错误
pub fn load_texture(path: &Path) -> Result<Texture, TextureError> {
    // ...
}

// 使用?操作符传播错误
let texture = load_texture(&path)?;
```

## 代码质量工具

### Clippy

使用`clippy`检查代码质量：

```bash
cargo clippy --workspace -- -D warnings
```

### 配置

创建`.clippy.toml`文件：

```toml
# 允许的警告
allow = [
    "clippy::too_many_arguments",
    "clippy::complexity",
]

# 禁止的警告
deny = [
    "clippy::unwrap_used",
    "clippy::expect_used",
]
```

### Rustfmt配置

创建`rustfmt.toml`文件：

```toml
# 使用4个空格缩进
tab_spaces = 4

# 最大行宽
max_width = 100

# 使用Unix风格的换行符
newline_style = "Unix"

# 格式化文档注释
format_code_in_doc_comments = true
```

## 代码审查流程

### 审查清单

1. **功能正确性**
   - [ ] 代码实现符合需求
   - [ ] 边界情况已处理
   - [ ] 错误处理完善

2. **代码质量**
   - [ ] 代码格式正确
   - [ ] 没有clippy警告
   - [ ] 注释充分

3. **性能**
   - [ ] 没有明显的性能问题
   - [ ] 内存使用合理
   - [ ] 并发安全

4. **测试**
   - [ ] 单元测试覆盖
   - [ ] 集成测试通过
   - [ ] 性能测试通过

### 审查模板

```markdown
## 代码审查

### 功能
- [ ] 功能实现正确
- [ ] 边界情况处理
- [ ] 错误处理

### 代码质量
- [ ] 代码格式
- [ ] 命名规范
- [ ] 注释充分

### 性能
- [ ] 性能影响评估
- [ ] 内存使用
- [ ] 并发安全

### 测试
- [ ] 测试覆盖
- [ ] 测试通过

### 建议
- 改进点1
- 改进点2
```

## Git工作流

### 提交消息规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type类型

- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试
- `chore`: 构建/工具

#### 示例

```
feat(render): 添加GPU实例化渲染支持

实现了增量实例数据更新机制，优化GPU缓冲区更新策略。
添加了性能监控和统计功能。

Closes #123
```

### 分支策略

- `main`: 主分支，稳定版本
- `develop`: 开发分支
- `feature/*`: 功能分支
- `fix/*`: 修复分支
- `release/*`: 发布分支

## 测试规范

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### 集成测试

```rust
// tests/integration/my_feature_test.rs
#[test]
fn test_feature_integration() {
    // 测试多个模块的集成
}
```

### 性能测试

```rust
// benches/my_benchmark.rs
#[bench]
fn bench_function(b: &mut Bencher) {
    b.iter(|| {
        function_under_test();
    });
}
```

## 最佳实践

### 内存安全

- 避免使用`unsafe`代码，除非必要
- 使用`Arc`和`Mutex`进行共享所有权
- 使用`Rc`和`RefCell`进行单线程共享

### 并发安全

- 使用`Send`和`Sync`trait确保并发安全
- 避免数据竞争
- 使用通道进行线程间通信

### 性能优化

- 测量优先：先测量再优化
- 避免过早优化
- 使用profiler识别瓶颈

## 相关文档

- [性能优化指南](guides/performance_optimization_guide.md)
- [架构设计文档](architecture/README.md)

