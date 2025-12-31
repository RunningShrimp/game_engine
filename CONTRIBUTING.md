# 贡献指南

感谢您对游戏引擎项目的关注！我们欢迎各种形式的贡献。

---

## 🤝 如何贡献

### 报告问题

如果您发现了bug或有功能建议：

1. 检查[Issues](../../issues)是否已有相关问题
2. 如果没有，创建新Issue并提供：
   - 清晰的标题
   - 详细的问题描述
   - 重现步骤（如果是bug）
   - 预期行为
   - 实际行为
   - 环境信息（Rust版本、操作系统等）

### 提交代码

#### 准备工作

1. **Fork仓库**
   ```bash
   # 点击GitHub上的Fork按钮
   git clone https://github.com/YOUR_USERNAME/game_engine.git
   cd game_engine
   ```

2. **设置开发环境**
   ```bash
   # 安装Rust（如果还没有）
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # 阅读快速开始指南
   cat QUICKSTART.md

   # 启动开发环境
   ./scripts/dev.sh
   ```

3. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

#### 开发规范

##### 代码风格

- 使用`rustfmt`格式化代码：
  ```bash
  cargo fmt
  ```

- 遵循`clippy`建议：
  ```bash
  cargo clippy -- -D warnings
  ```

##### 提交消息

使用[Conventional Commits](https://www.conventionalcommits.org/)格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型（type）**:
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**:
```
feat(physics): 添加碰撞检测功能

实现了AABB碰撞检测算法，支持2D和3D场景。

Closes #123
```

```
fix(audio): 修复音频播放时的内存泄漏

使用AudioContext正确管理音频资源生命周期。

Fixes #456
```

##### 编写测试

- 所有新功能必须有测试
- 目标测试覆盖率：≥70%
- 测试文件：`module_name/tests.rs`或`tests/module_name_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let result = function_to_test();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        // 测试边界情况
    }
}
```

##### 编写文档

- 所有公共API必须有文档注释
- 使用`///`或`//!`添加文档
- 包含使用示例

```rust
/// 计算两点之间的距离
///
/// # 参数
///
/// * `x1` - 第一个点的x坐标
/// * `y1` - 第一个点的y坐标
/// * `x2` - 第二个点的x坐标
/// * `y2` - 第二个点的y坐标
///
/// # 返回
///
/// 两点之间的欧几里得距离
///
/// # 示例
///
/// ```
/// use game_engine::math::distance;
///
/// let dist = distance(0.0, 0.0, 3.0, 4.0);
/// assert_eq!(dist, 5.0);
/// ```
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // 实现...
}
```

#### 提交流程

1. **确保测试通过**
   ```bash
   ./scripts/test.sh
   # 或
   cargo test --workspace
   ```

2. **提交代码**
   ```bash
   git add .
   git commit -m "feat: add your feature"  # 遵循提交消息规范
   ```

3. **推送到Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **创建Pull Request**
   - 访问原始仓库
   - 点击"New Pull Request"
   - 填写PR模板
   - 等待代码审查

---

## 📋 Pull Request检查清单

提交PR前，请确保：

- [ ] 代码通过`cargo fmt`格式化
- [ ] 代码通过`cargo clippy`检查（无警告）
- [ ] 所有测试通过（`cargo test --workspace`）
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 提交消息遵循规范
- [ ] PR描述清晰说明变更内容

---

## 🏗️ 开发指南

### 项目结构

```
game_engine/
├── game_engine/           # 主引擎库
│   ├── src/
│   │   ├── ai/           # AI系统
│   │   ├── audio/        # 音频引擎
│   │   ├── core/         # 核心功能
│   │   ├── ecs/          # 实体组件系统
│   │   ├── physics/      # 物理引擎
│   │   ├── render/       # 渲染系统
│   │   └── resources/    # 资源管理
│   └── Cargo.toml
├── game_engine_macros/   # 过程宏
├── game_engine_profiling/# 性能分析
├── examples/             # 示例代码
├── tests/                # 集成测试
├── docs/                 # 文档
├── scripts/              # 开发脚本
└── benches/              # 基准测试
```

### 技术栈

- **语言**: Rust 2024 Edition
- **ECS**: bevy_ecs
- **渲染**: wgpu
- **物理**: rapier2d/rapier3d
- **数学**: glam
- **异步**: tokio
- **并发**: parking_lot, dashmap

### 性能优化

本项目注重性能，贡献时请注意：

- 使用`parking_lot::RwLock`代替`std::sync::RwLock`（2.5x-8x faster）
- 高并发场景使用`DashMap`（10x-20x faster）
- 避免不必要的内存分配
- 使用对象池复用对象
- 参考`examples/performance_examples.rs`

### 测试指南

#### 单元测试

测试单个函数或方法：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_function() {
        assert_eq!(2 + 2, 4);
    }
}
```

#### 集成测试

测试多个模块的交互：

```rust
// tests/integration_test.rs
use game_engine::prelude::*;

#[test]
fn test_full_workflow() {
    // 测试完整工作流
}
```

#### 性能测试

使用criterion进行基准测试：

```rust
// benches/example.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_example(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            // 要测试的代码
            black_box(result)
        });
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
```

运行：
```bash
cargo bench --bench example
```

---

## 📖 代码审查标准

PR代码审查时会关注：

1. **代码质量**
   - 清晰易读
   - 遵循Rust最佳实践
   - 适当的错误处理

2. **性能**
   - 避免不必要的分配
   - 使用适当的并发原语
   - 性能测试无退化

3. **测试**
   - 充分的测试覆盖
   - 边界情况测试
   - 有意义的测试断言

4. **文档**
   - 公共API有文档
   - 复杂逻辑有注释
   - 示例代码正确

---

## 🐛 调试技巧

### 启用日志

```rust
use log::{info, debug, error};

fn main() {
    env_logger::init();

    info!("Starting game...");
    debug!("Debug info");
    error!("Something went wrong");
}
```

### 使用rust-analyzer

VSCode配置已包含在`.vscode/settings.json`中。

### 性能分析

```bash
# 使用Flamegraph
cargo install flamegraph
cargo flamegraph

# 使用Tracy（需要feature）
cargo run --features tracy
```

---

## 📚 资源

- **Rust文档**: https://doc.rust-lang.org/
- **API文档**: `cargo doc --open`
- **快速开始**: [QUICKSTART.md](QUICKSTART.md)
- **示例代码**: `examples/`

---

## 🎯 优先事项

我们特别欢迎以下方面的贡献：

- [ ] 性能优化
- [ ] 测试覆盖提升
- [ ] 文档完善
- [ ] 示例代码
- [ ] Bug修复
- [ ] 跨平台支持

---

## 📧 联系方式

- **Issues**: [GitHub Issues](../../issues)
- **Discussions**: [GitHub Discussions](../../discussions)
- **Discord**: [加入我们的Discord社区](https://discord.gg/gameengine)
- **Stack Overflow**: 使用标签 `rust-gameengine`

---

## 📄 许可证

贡献的代码将使用项目的MIT/Apache-2.0双重许可。

---

## 🌟 社区资源

### 获取帮助

1. **Discord社区** - 实时讨论和帮助
2. **GitHub Discussions** - 技术讨论和问答
3. **Stack Overflow** - 标签: `rust-gameengine`
4. **文档** - [docs.gameengine.dev](https://docs.gameengine.dev)

### 贡献者认可

我们会在以下地方认可贡献者:
- README.md 贡献者列表
- Release Notes 中特别感谢
- Discord @contributor 角色
- 年度贡献者报告

---

## 🎯 专项贡献计划

### 性能优化专项

- **目标**: 提升引擎性能
- **奖励**: @performance-badge 角色
- **资源**: `#performance` 频道

### 文档改进专项

- **目标**: 完善文档和教程
- **奖励**: @documenter 角色
- **资源**: `#docs` 频道

### 示例代码专项

- **目标**: 创建高质量示例
- **奖励**: @example-creator 角色
- **资源**: `#examples` 频道

---

**感谢您的贡献！** 🙏

我们重视每一个贡献,无论是代码、文档、Bug报告还是仅仅是提出建议。一起让这个游戏引擎变得更好！
