# 架构决策记录（ADR）

本目录包含游戏引擎的关键架构决策记录。

## 什么是ADR？

架构决策记录（Architecture Decision Record）是一种记录架构决策的文档格式。每个ADR记录一个重要的架构决策，包括：
- 决策的上下文
- 决策内容
- 决策的后果（优点和缺点）
- 替代方案

## ADR列表

- [ADR-0001: 使用领域驱动设计（DDD）](./0001-use-domain-driven-design.md)
- [ADR-0002: 使用Actor模式进行并发控制](./0002-use-actor-model-for-concurrency.md)
- [ADR-0003: 错误处理和恢复策略](./0003-error-handling-strategy.md)
- [ADR-0004: 使用wgpu进行渲染](./0004-use-wgpu-for-rendering.md)
- [ADR-0005: 使用Bevy ECS进行实体管理](./0005-use-bevy-ecs.md)

## 如何添加新的ADR

1. 创建新文件 `docs/adr/XXXX-description.md`
2. 使用以下模板：

```markdown
# ADR-XXXX: 决策标题

**状态**: 提议/已采用/已废弃  
**日期**: YYYY-MM-DD  
**决策者**: 决策者名称

## 上下文

描述决策的上下文和问题。

## 决策

描述所做的决策。

## 后果

### 优点
- 优点1
- 优点2

### 缺点
- 缺点1
- 缺点2

### 实施细节
- 实施细节1
- 实施细节2

## 替代方案

1. **方案1**: 描述
   - 拒绝原因: 原因

2. **方案2**: 描述
   - 拒绝原因: 原因

## 相关决策

- ADR-XXXX: 相关决策
```

3. 更新本README文件，添加新ADR到列表



