# GitHub Projects 看板配置

## 项目名称
游戏引擎代码质量改进 - 2025 Q1

## 看板列（Columns）

### 1. 📋 Backlog
- 所有待办任务的初始位置
- 按优先级排序

### 2. 🚀 Ready to Start
- 已准备开始的任务
- 依赖已满足
- 资源已分配

### 3. 🔨 In Progress
- 正在执行的任务
- 限制同时进行数（建议3-5个）

### 4. 🧪 In Review
- 代码审查中
- 待验证的任务

### 5. ✅ Done
- 已完成的任务
- 验收通过

### 6. ⏸️ Blocked
- 被阻塞的任务
- 等待依赖或外部资源

---

## 标签（Labels）

### 优先级标签
- `P0 - Critical` 🔴 - 紧急/关键
- `P1 - High` 🟠 - 高优先级
- `P2 - Medium` 🟡 - 中优先级
- `P3 - Low` 🟢 - 低优先级

### 类型标签
- `code-quality` - 代码质量改进
- `testing` - 测试相关
- `refactoring` - 重构任务
- `documentation` - 文档编写
- `infrastructure` - 基础设施

### 模块标签
- `core` - 引擎核心
- `render` - 渲染系统
- `physics` - 物理引擎
- `network` - 网络系统
- `ecs` - ECS系统
- `domain` - 领域层
- `platform` - 平台相关

### 状态标签
- `in-review` - 审查中
- `needs-testing` - 需要测试
- `ready-to-merge` - 可合并

---

## 初始Issues列表

### Phase 1: 短期改进（1-2周）

#### P0-1: 移除lib.rs中的Clippy豁免
**标题**: `[Code Quality][P0] 移除lib.rs中的Clippy豁免 - 批次1: unused_variables/unused_mut`
**标签**: `P0 - Critical`, `code-quality`, `core`
**预估**: 0.5天
**列**: 🚀 Ready to Start

**子任务**:
- [ ] 统计当前unused数量
- [ ] 修复或使用`_`前缀
- [ ] 验证编译通过
- [ ] 从lib.rs移除豁免

---

#### P0-2: 重构profiling/tracy.rs
**标题**: `[Refactoring][P0] 修复profiling/tracy.rs的条件编译复杂度`
**标签**: `P0 - Critical`, `refactoring`, `profiling`
**预估**: 3-4天
**列**: 🚀 Ready to Start

**子任务**:
- [ ] 设计ProfilerBackend trait
- [ ] 实现TracyBackend
- [ ] 实现StubBackend
- [ ] 重构TracyProfiler
- [ ] 更新tracy_macros.rs
- [ ] 添加测试

---

#### P0-3: 建立代码质量基线
**标题**: `[Infrastructure][P0] 建立代码质量基线`
**标签**: `P0 - Critical`, `infrastructure`
**预估**: 1-2天
**列**: 🚀 Ready to Start

**子任务**:
- [x] 创建目录结构
- [x] 生成基线指标JSON
- [ ] 运行完整覆盖率测试
- [ ] 生成HTML覆盖率报告
- [ ] 创建GitHub Issue追踪
- [ ] 设置CI质量门禁

---

### Phase 2: 中期改进（1-2月）

#### P1-4: 验证key_exchange.rs
**标题**: `[Code Quality][P1] 验证network/key_exchange.rs重构质量`
**标签**: `P1 - High`, `code-quality`, `network`
**预估**: 2-3天
**列**: 📋 Backlog

---

#### P1-5: 核心模块测试
**标题**: `[Testing][P1] 添加ECS模块单元测试`
**标签**: `P1 - High`, `testing`, `ecs`
**预估**: 3天
**列**: 📋 Backlog

**标题**: `[Testing][P1] 添加Physics模块单元测试`
**标签**: `P1 - High`, `testing`, `physics`
**预估**: 2天
**列**: 📋 Backlog

---

## Milestones

### Milestone 1: Week 2 (2025-01-10)
**目标**: 建立基线，完成P0任务
**Issues**:
- P0-1: lib.rs豁免移除
- P0-2: tracy.rs重构
- P0-3: 基线建立

**成功标准**:
- [ ] lib.rs豁免降至3个以下
- [ ] tracy.rs条件编译降至5个以下
- [ ] 覆盖率基线报告生成

---

### Milestone 2: Week 8 (2025-02-21)
**目标**: 测试覆盖率提升至50%
**Issues**:
- P1-4至P1-6全部完成

**成功标准**:
- [ ] 覆盖率 > 50%
- [ ] unwrap/expect < 500
- [ ] key_exchange.rs验证完成

---

### Milestone 3: Week 16 (2025-04-18)
**目标**: 架构优化完成
**成功标准**:
- [ ] 条件编译简化完成
- [ ] domain层测试完善
- [ ] 异步测试添加

---

### Milestone 4: Week 24 (2025-06-20)
**目标**: 全部完成
**成功标准**:
- [ ] 覆盖率 > 80%
- [ ] 文档完善
- [ ] 所有质量目标达成

---

## 自动化规则

### 自动移动规则
1. **当PR合并时** → 自动移动到Done
2. **当分配负责人时** → 从Backlog移到Ready
3. **当标记为blocked时** → 移动到Blocked

### 通知规则
1. **P0任务** → 每日站会提醒
2. **PR打开** → 通知相关人员
3. **里程碑临近** → 提前3天提醒

---

## 报告和仪表板

### 每周报告
- 每周一自动生成进度报告
- 发送到团队频道
- 包含:
  - 完成的任务
  - 进行中的任务
  - 阻塞和风险
  - 下周计划

### 仪表板指标
- 总任务数
- 完成百分比
- 按优先级分布
- 按模块分布
- 平均周期时间

---

## 配置文件

### `.github/projects/config.yml`
```yaml
# 自动化配置
automation:
  close_issues: true
  move_columns: true
  labels:
    - P0 - Critical
    - P1 - High
    - P2 - Medium
    - P3 - Low
```

---

**看板设置完成日期**: 2025-12-28
**下次审查**: 2025-01-04
