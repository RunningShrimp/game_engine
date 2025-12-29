# GitHub Issues模板

## Issue标题模板

### 代码质量改进类

**标题**: `[Code Quality] P0-1: 移除lib.rs中的Clippy豁免 - 移除unused_variables/unused_mut`

**描述**:
```markdown
## 任务描述
移除`game_engine/src/lib.rs`中的`unused_variables`和`unused_mut`豁免，改为在具体使用处添加`#[allow]`或修复。

## 背景
当前lib.rs级别豁免了大量Clippy警告，影响代码质量检测。需要渐进式移除这些豁免。

## 具体步骤
1. 运行`cargo clippy`统计当前unused数量
2. 逐个修复或使用`_`前缀
3. 在具体函数上添加`#[allow(unused_variables)]`
4. 验证编译通过
5. 从lib.rs移除对应豁免

## 验收标准
- [ ] `cargo clippy`无unused相关警告（除明确允许的）
- [ ] `cargo test`全部通过
- [ ] lib.rs移除对应豁免

## 工作量
0.5天

## 依赖
无

## 进度追踪
- 追踪表: `docs/quality-tracker/clippy-migration-tracker.md`
- 基线指标: `docs/coverage/baseline/metrics.json`
```

---

## Bug报告模板

**标题**: `[Bug] 模块名称 - 简短描述`

**描述**:
```markdown
## Bug描述
清晰描述bug是什么

## 复现步骤
1. 步骤1
2. 步骤2
3. ...

## 期望行为
描述期望的正确行为

## 实际行为
描述实际发生的错误行为

## 环境信息
- OS:
- Rust版本:
- 引擎版本:

## 相关日志
粘贴相关错误日志

## 额外信息
任何其他有助于解决问题的信息
```

---

## 功能请求模板

**标题**: `[Feature Request] 功能名称 - 简短描述`

**描述**:
```markdown
## 功能描述
清晰描述想要的功能

## 使用场景
描述该功能的使用场景和为什么需要它

## 提议的实现
如果有实现想法，请描述

## 替代方案
描述其他可能的解决方案

## 优先级
- [ ] P0 - 紧急/关键
- [ ] P1 - 高优先级
- [ ] P2 - 中优先级
- [ ] P3 - 低优先级
```

---

## 性能优化模板

**标题**: `[Performance] 模块名称 - 性能问题/优化`

**描述**:
```markdown
## 性能问题描述
描述性能瓶颈或优化机会

## 当前性能
- 帧率: XX fps
- 内存占用: XX MB
- 加载时间: XX ms

## 目标性能
- 帧率: XX fps
- 内存占用: XX MB
- 加载时间: XX ms

## 分析工具
- Tracy Profiler
- Criterion benchmarks
- 自定义分析工具

## 建议的优化方案
1. 优化点1
2. 优化点2
3. ...

## 基准测试结果
粘贴benchmark结果

## 验收标准
- [ ] 性能提升XX%
- [ ] 无功能回归
- [ ] 基准测试通过
```
