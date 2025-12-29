# Profiling/tracy.rs 重构设计文档

**文件**: game_engine/src/profiling/tracy.rs
**当前状态**: 22个条件编译指令
**目标**: <5个条件编译指令
**策略**: ProfilerBackend trait抽象

---

## 当前问题

### 现有模式（重复22次）
```rust
#[cfg(feature = "tracy")]
{ enabled: true }
#[cfg(not(feature = "tracy"))]
{ enabled: false }
```

**问题**:
1. 代码重复
2. 维护困难
3. 可读性差

---

## 设计方案

### 核心Trait定义

```rust
/// 性能分析后端trait
pub trait ProfilerBackend {
    /// 开始性能分析区域
    fn begin_span(&self, name: &str);

    /// 结束性能分析区域
    fn end_span(&self);

    /// 标记一个即时事件
    fn mark_event(&self, name: &str);

    /// 检查是否启用
    fn is_enabled(&self) -> bool;
}

/// 作用域guard - 自动管理分析区域
pub struct ProfilerScope<'a> {
    backend: &'a dyn ProfilerBackend,
    name: String,
}

impl<'a> ProfilerScope<'a> {
    pub fn new(backend: &'a dyn ProfilerBackend, name: &str) -> Self {
        backend.begin_span(name);
        Self {
            backend,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ProfilerScope<'a> {
    fn drop(&mut self) {
        self.backend.end_span();
    }
}
```

### Tracy实现

```rust
#[cfg(feature = "tracy")]
use tracy_client::*;

pub struct TracyBackend {
    client: Option<Client>,
}

impl ProfilerBackend for TracyBackend {
    fn begin_span(&self, name: &str) {
        if let Some(client) = &self.client {
            // Tracy实现
        }
    }

    fn end_span(&self) {
        // 结束tracy span
    }

    fn mark_event(&self, name: &str) {
        // 标记事件
    }

    fn is_enabled(&self) -> bool {
        self.client.is_some()
    }
}
```

### Stub实现

```rust
#[cfg(not(feature = "tracy"))]
pub struct StubBackend;

impl ProfilerBackend for StubBackend {
    fn begin_span(&self, _name: &str) {
        // 空实现 - 编译器会优化掉
    }

    fn end_span(&self) {
        // 空实现
    }

    fn mark_event(&self, _name: &str) {
        // 空实现
    }

    fn is_enabled(&self) -> bool {
        false
    }
}
```

### 类型别名

```rust
#[cfg(feature = "tracy")]
type BackendImpl = TracyBackend;

#[cfg(not(feature = "tracy"))]
type BackendImpl = StubBackend;
```

### 重构后的TracyProfiler

```rust
pub struct TracyProfiler {
    backend: BackendImpl,
}

impl TracyProfiler {
    pub fn new() -> Self {
        Self {
            backend: BackendImpl::new(),
        }
    }

    pub fn scope(&self, name: &str) -> ProfilerScope {
        ProfilerScope::new(&self.backend, name)
    }

    pub fn mark(&self, name: &str) {
        self.backend.mark_event(name);
    }

    pub fn is_enabled(&self) -> bool {
        self.backend.is_enabled()
    }
}

impl Default for TracyProfiler {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 迁移计划

### 阶段1: 创建backend.rs (0.5天)
- [ ] 创建`profiling/backend.rs`
- [ ] 定义ProfilerBackend trait
- [ ] 实现TracyBackend
- [ ] 实现StubBackend

### 阶段2: 重构tracy.rs (1天)
- [ ] 更新TracyProfiler使用Backend
- [ ] 更新所有方法
- [ ] 移除重复的条件编译

### 阶段3: 更新tracy_macros.rs (0.5天)
- [ ] 同步宏实现
- [ ] 使用新的trait

### 阶段4: 测试 (0.5天)
- [ ] 测试两种feature配置
- [ ] 验证性能无回归
- [ ] 运行所有测试

### 阶段5: 文档 (0.5天)
- [ ] 更新模块文档
- [ ] 添加使用示例

---

## 预期结果

### 条件编译减少

| 项目 | 当前 | 目标 |
|------|------|------|
| #[cfg(feature = "tracy")] | 22 | 2 |
| 总行数 | ~150 | ~120 |
| 代码重复 | 高 | 无 |

### 优势
1. ✅ 代码清晰度提升
2. ✅ 维护性改善
3. ✅ 易于测试
4. ✅ 性能无损失（编译器优化）

---

**设计完成时间**: 2025-12-28
**预计实施时间**: 3天
**状态**: 准备开始实施
