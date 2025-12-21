# Service/Manager命名规范

## 1. 命名原则

### Service命名
- **领域服务**：`{Domain}DomainService` (例如：`AudioDomainService`, `PhysicsDomainService`)
- **应用服务**：`{Domain}Service` (例如：`ScriptingService`, `RenderService`)
- **用途**：封装业务逻辑，协调多个聚合根或领域对象

### Manager命名
- **资源管理**：`{Resource}Manager` (例如：`SceneManager`, `BatchManager`)
- **生命周期管理**：`{Entity}Manager` (例如：`TaskManager`, `MilestoneManager`)
- **用途**：管理资源生命周期、状态、集合

## 2. 当前命名分析

### 符合规范的命名
- ✅ `AudioDomainService` - 领域服务
- ✅ `PhysicsDomainService` - 领域服务
- ✅ `SceneDomainService` - 领域服务
- ✅ `SceneManager` - 场景资源管理
- ✅ `BatchManager` - 批处理资源管理
- ✅ `TaskManager` - 任务生命周期管理
- ✅ `RecoveryManager` - 恢复资源管理

### 需要统一的命名
- ⚠️ `AudioService` vs `AudioDomainService` - 需要确认职责区分
- ⚠️ `EventSourcingManager` - 应该改为 `EventSourcingService`（业务逻辑）
- ⚠️ `SpatialPartitionManager` - 符合规范（资源管理）
- ⚠️ `PreloadManager` - 符合规范（资源管理）

## 3. 统一策略

### 步骤1：识别职责
- **Service**：包含业务逻辑，协调操作
- **Manager**：管理资源集合，生命周期

### 步骤2：逐步重构
1. 保持向后兼容（添加类型别名）
2. 更新文档
3. 逐步迁移使用方

## 4. 迁移计划

### 高优先级
- `EventSourcingManager` → `EventSourcingService` (业务逻辑)

### 中优先级
- 统一 `AudioService` 和 `AudioDomainService` 的职责

### 低优先级
- 审查其他Manager/Service的命名合理性

