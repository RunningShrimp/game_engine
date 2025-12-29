---
name: Performance issue
about: 报告性能问题或瓶颈
title: '[PERF] '
labels: performance
assignees: ''
---

## 性能问题描述
<!-- 描述你遇到的性能问题 -->

在[特定场景]下，[组件/功能]的性能不符合预期...

## 性能指标
<!-- 提供具体的性能数据 -->

### 期望性能
- FPS: ____
- 帧时间: ____ ms
- 内存使用: ____ MB
- 加载时间: ____ s
- 其他指标: ____

### 实际性能
- FPS: ____
- 帧时间: ____ ms
- 内存使用: ____ MB
- 加载时间: ____ s
- 其他指标: ____

### 性能差距
- 差距: ____%
- 影响: [严重/中等/轻微]

## 复现场景
<!-- 描述如何触发这个性能问题 -->

1. 操作步骤：
   ```
   步骤1: ...
   步骤2: ...
   步骤3: ...
   ```

2. 测试场景：
   - 场景大小：____
   - 对象数量：____
   - 其他参数：____

3. 运行时间：____ 分钟/小时

## 环境信息

**操作系统:**
- [ ] Linux (发行版: _____)
- [ ] macOS (版本: _____)
- [ ] Windows (版本: _____)

**硬件配置:**
- CPU: _____ (核心数: _____)
- GPU: _____ (VRAM: _____)
- 内存: _____ (类型: _____)
- 存储: _____ [SSD/HDD]

**Rust版本:**
```bash
rustc --version
# 输出: paste here
cargo --version
```

**引擎版本:**
- commit hash: _____
- 编译模式: [debug/release]
- 优化级别: _____

## Profiling数据
<!-- 如果有profiling数据，请提供 -->

### flamegraph
[上传flamegraph图片或链接]

### perf/cachegrind/其他profiler输出
```
[paste profiling output here]
```

### 热点函数
列出最耗时的函数：
1. `function_name`: ____% (____ ms)
2. `function_name`: ____% (____ ms)
3. `function_name`: ____% (____ ms)

## 最小复现代码
<!-- 提供复现性能问题的代码 -->

```rust
// 最小性能测试代码
use game_engine::prelude::*;

fn main() {
    // setup code

    // performance critical section
    for _ in 0..iterations {
        // code that shows the performance issue
    }
}
```

## 基准测试结果 (如有)
```bash
cargo bench --bench benchmark_name
# [paste output here]
```

## 已尝试的优化
<!-- 描述你已经尝试过的优化方案 -->

1. 尝试1：
   - 描述：...
   - 结果：[成功/失败/部分成功]

2. 尝试2：
   - 描述：...
   - 结果：[成功/失败/部分成功]

## 建议的优化方向
<!-- 如果你对优化有想法 -->

1. 优化方向1：
   - 描述：...
   - 预期收益：...

2. 优化方向2：
   - 描述：...
   - 预期收益：...

## 附加信息
<!-- 任何其他相关信息 -->

### 屏幕录制/演示
[如果有屏幕录制或演示，请提供链接]

### 参考资源
- [相关的性能优化文章]
- [类似问题的解决方案]

## 优先级
- [ ] 阻塞性 (导致无法使用)
- [ ] 高优先级 (严重影响用户体验)
- [ ] 中优先级 (可以接受但影响体验)
- [ ] 低优先级 (优化点)

## Checklist
- [ ] 我已经在release模式下测试
- [ ] 我提供了具体的性能指标
- [ ] 我提供了profiling数据 (如果可能)
- [ ] 我提供了硬件配置信息
- [ ] 我搜索了相关的性能优化方案
