# Stack Overflow 标签使用指南

本指南帮助你在Stack Overflow上有效地提问和回答关于游戏引擎的问题。

## 目录

- [标签信息](#标签信息)
- [如何提问](#如何提问)
- [标签使用](#标签使用)
- [常见问题](#常见问题)
- [响应承诺](#响应承诺)
- [回答指南](#回答指南)

---

## 标签信息

### 主标签

**`rust-gameengine`**

- **描述:** Rust游戏引擎相关问题
- **使用场景:** 所有关于本引擎的问题
- **关注者:** 0 (初期)
- **问题数:** 0 (初期)

### 相关标签

- **`rust`** - Rust语言相关问题
- **`game-development`** - 游戏开发通用问题
- **`webgpu`** - WebGPU图形API问题
- **`ecs`** - Entity Component System架构
- **`bevy`** - Bevy引擎相关问题(如果对比)
- **`gamedev`** - 游戏开发讨论

---

## 如何提问

### 提问前检查清单

在提问前,请确保:

1. ✅ **搜索现有问题**
   - 搜索Stack Overflow
   - 搜索GitHub Issues
   - 搜索官方文档
   - 搜索Discord历史消息

2. ✅ **准备最小可复现示例**
   - 代码可以独立运行
   - 只包含必要代码
   - 清晰展示问题
   - 包含输入和输出

3. ✅ **收集环境信息**
   - Rust版本
   - 操作系统
   - 引擎版本
   - 相关依赖版本

4. ✅ **描述尝试过的方案**
   - 你尝试了什么
   - 结果如何
   - 为什么不work

### 好的问题格式

```markdown
## 标题
简短描述问题,包含关键信息

示例:
"How to implement collision detection in Rust gameengine ECS?"
"Why does WebGPU rendering fail on macOS with this error?"

## 问题内容

### 描述
清晰描述你想要实现什么,遇到了什么问题。

### 最小可复现代码
```rust
use game_engine::prelude::*;

fn main() {
    // 最小可复现示例
    let mut world = World::new();
    // ...
}
```

### 错误信息
```
paste complete error message here
```

### 环境
- Rust: 1.75.0
- OS: macOS 14.0
- gameengine: v1.2.0
- wgpu: 0.19.0

### 尝试过的方案
1. 尝试X: 结果Y
2. 尝试Z: 结果W

### 预期行为 vs 实际行为
**预期:** 应该发生什么
**实际:** 实际发生了什么
```

### 问题标题示例

#### ✅ 好的标题

```
"How to handle entity lifecycle in ECS system?"
"WebGPU compute shader not writing to storage buffer"
"Rust gameengine panic: 'borrow of moved value'"
```

#### ❌ 不好的标题

```
"Help!"
"Error in my code"
"Doesn't work"
"How to game"
```

---

## 标签使用

### 标签组合建议

根据问题类型使用合适的标签组合:

#### ECS相关问题
```
rust-gameengine + rust + ecs
```

#### 渲染问题
```
rust-gameengine + rust + webgpu + graphics
```

#### 物理问题
```
rust-gameengine + rust + physics + game-development
```

#### 性能问题
```
rust-gameengine + rust + performance + optimization
```

#### 编译错误
```
rust-gameengine + rust + compilation-error
```

#### WebAssembly问题
```
rust-gameengine + rust + webassembly + wasm
```

### 标签使用示例

#### 示例1: ECS查询问题

```markdown
Title: "How to filter entities by multiple components in ECS?"

Tags: rust-gameengine, rust, ecs, bevy-ecs

Body:
I'm trying to query entities that have both Health and Position components...
```

#### 示例2: WebGPU渲染问题

```markdown
Title: "WebGPU render pass ends with Validation Error"

Tags: rust-gameengine, rust, webgpu, wgpu

Body:
I'm getting a validation error when creating a render pass...
```

#### 示例3: 性能优化问题

```markdown
Title: "Optimize particle system with 10k+ entities"

Tags: rust-gameengine, rust, performance, optimization, ecs

Body:
I have a particle system with 10k+ entities and it's running at 30fps...
```

---

## 常见问题

### FAQ 1: 如何快速开始?

**Q:** 如何创建第一个游戏?

**A:** 请参考我们的[快速开始指南](https://docs.gameengine.dev/quickstart)和[示例代码](https://github.com/gameengine/examples)。

**标签:** `rust-gameengine` + `rust` + `getting-started`

---

### FAQ 2: ECS查询最佳实践

**Q:** 如何高效查询ECS实体?

**A:** 使用查询(query)系统,避免嵌套循环。参考[ECS文档](https://docs.gameengine.dev/ecs/queries)。

**标签:** `rust-gameengine` + `rust` + `ecs`

---

### FAQ 3: WebGPU兼容性

**Q:** 支持哪些浏览器和平台?

**A:** WebGPU需要支持的浏览器:Chrome 113+, Edge 113+, Firefox Nightly。参考[WebGPU兼容性](https://docs.gameengine.dev/webgpu/support)。

**标签:** `rust-gameengine` + `rust` + `webgpu`

---

### FAQ 4: 性能优化

**Q:** 如何提升游戏性能?

**A:**
1. 使用Profiling工具识别瓶颈
2. 优化ECS查询
3. 使用对象池减少分配
4. 启用SIMD优化

参考[性能优化指南](https://docs.gameengine.dev/performance)。

**标签:** `rust-gameengine` + `rust` + `performance` + `optimization`

---

### FAQ 5: 跨平台编译

**Q:** 如何编译到WebAssembly?

**A:** 使用wasm-pack:
```bash
cargo install wasm-pack
wasm-pack build --target web
```

参考[Wasm教程](https://docs.gameengine.dev/platforms/web)。

**标签:** `rust-gameengine` + `rust` + `webassembly` + `wasm`

---

## 响应承诺

### 官方支持时间

我们承诺以下响应时间:

| 问题优先级 | 响应时间 | 说明 |
|-----------|---------|------|
| 阻塞性 | 24小时 | 无法继续开发 |
| 高优先级 | 48小时 | 功能严重受限 |
| 中优先级 | 72小时 | 有变通方案 |
| 低优先级 | 1周 | 小问题或改进 |

### 如何标记优先级

在问题标题或内容中标记:

```
[URGENT] Production game crashes on startup
[HIGH] Memory leak in physics system
[MEDIUM] UI rendering glitch on specific device
[LOW] Documentation typo
```

### 加速响应

获得更快响应的方法:

1. **提供完整信息**
   - MCVE(最小可复现示例)
   - 完整错误信息
   - 环境信息

2. **使用正确标签**
   - `rust-gameengine` (必需)
   - 相关技术标签

3. **格式清晰**
   - 使用Markdown代码块
   - 组织清晰的结构
   - 添加截图或图表

4. **分享到社区**
   - 在Discord #help 链接到问题
   - 在GitHub Discussions提及
   - Twitter @gameengine

---

## 回答指南

### 如何提供好的回答

#### 1. 理解问题

- 仔细阅读问题描述
- 理解预期和实际行为
- 识别核心问题

#### 2. 提供解决方案

**代码示例:**
```rust
// 解决方案的完整代码
use game_engine::prelude::*;

fn solution() -> Result<()> {
    // 清晰、可运行的代码
    Ok(())
}
```

**说明:**
- 解释为什么这样解决
- 说明关键步骤
- 指出潜在陷阱

#### 3. 提供资源

- 链接到相关文档
- 引用类似问题
- 推荐学习资源

#### 4. 验证答案

- 确保代码可运行
- 测试提供的解决方案
- 更新过时的信息

### 回答模板

```markdown
## 解决方案

直接回答问题的核心。

### 代码示例
```rust
// 完整的、可运行的解决方案
```

### 说明
1. 步骤1: 说明
2. 步骤2: 说明
3. 步骤3: 说明

### 为什么这样工作
解释原理和设计决策。

### 替代方案
如果有其他方法,也列出来。

### 参考资源
- [文档链接](URL)
- [相关问题](URL)
```

### 获得认可

好的回答者会获得:

- 🏆 Stack Overflow声望
- ⭐ 在社区中被认可
- 💎 可能成为核心贡献者
- 📝 被邀请撰写博客

---

## 社区规范

### 行为准则

- **尊重他人** - 礼貌和专业
- **建设性** - 专注于解决问题
- **包容性** - 欢迎所有经验水平
- **协作** - 帮助他人学习

### 禁止行为

- 粗鲁和讽刺
- 嘲笑初学者
- 重复回答相同内容
- 推广无关产品

---

## 监控和通知

### 我们如何监控

- 官方团队订阅 `rust-gameengine` 标签
- 自动通知到Discord #help
- 每日问题审查
- 每周 unanswered 问题回顾

### 问题转发

- 新问题自动通知到Discord
- 高优先级问题立即处理
- 未回答问题定期提醒

---

## 资源链接

### 官方资源

- **Stack Overflow:** https://stackoverflow.com/questions/tagged/rust-gameengine
- **文档:** https://docs.gameengine.dev
- **GitHub:** https://github.com/gameengine/engine
- **Discord:** https://discord.gg/gameengine

### 学习资源

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Game Programming Patterns](https://gameprogrammingpatterns.com/)

---

## 反馈和改进

如果你有改进这个指南的建议:

1. 在GitHub创建Issue
2. 或在Discord #docs 讨论
3. 或提交PR改进文档

---

## 总结

### 关键要点

1. ✅ 提问前先搜索
2. ✅ 提供MCVE(最小可复现示例)
3. ✅ 使用正确的标签
4. ✅ 描述清楚问题和环境
5. ✅ 标记适当的优先级

### 获取帮助的顺序

1. 📚 搜索文档
2. 🔍 搜索现有问题
3. 💬 在Discord询问
4. ❓ 在Stack Overflow提问
5. 🐛 在GitHub创建Issue(如果是Bug)

---

**感谢你使用游戏引擎!** 我们在Stack Overflow上等你!

---

**最后更新:** 2024-12-31
**标签:** `rust-gameengine`
**维护者:** 游戏引擎团队
