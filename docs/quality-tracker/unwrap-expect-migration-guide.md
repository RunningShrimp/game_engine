# P0-1.4 & P1-6: unwrap/expect 迁移指南

**目标**: 替换1,415个unwrap/expect为安全错误处理
**策略**: 分批次、分模块渐进式迁移

---

## 现状统计

### 全局统计
- **unwrap使用**: ~800次
- **expect使用**: ~615次
- **总计**: 1,415次
- **目标**: 降至500次以下

### 高频文件TOP 10

| 文件 | unwrap/expect数 | 优先级 |
|------|----------------|--------|
| scripting/lua_tests.rs | 50+ | P2 |
| core/engine/input_handler.rs | 30+ | P1 |
| physics/spatial_partition.rs | 20+ | P1 |
| render/webgl_adapter.rs | 15+ | P1 |
| domain/tests/scene_tests.rs | 15+ | P2 |
| domain/tests/services_tests.rs | 12+ | P2 |
| xr/openxr_impl.rs | 10+ | P1 |
| resources/tests.rs | 10+ | P2 |
| ecs/entity_manager.rs | 8+ | P1 |
| error/concurrency_tests.rs | 8+ | P2 |

---

## 迁移原则

### 1. 分类处理

**Category A: 必须替换**（核心业务逻辑）
- core/, ecs/, physics/, render/, network/
- 替换为 Result<_, EngineError>

**Category B: 延迟处理**（测试代码）
- *_tests.rs
- 保留部分unwrap，添加注释说明

**Category C: 暂时保留**（FFI/底层）
- bindings/, platform/
- 添加安全注释和issue追踪

### 2. 替换模式

#### Pattern 1: Option unwrap
```rust
// ❌ Before
let value = option.unwrap();

// ✅ After
let value = option.ok_or_else(|| EngineError::NotFound(...))?;
```

#### Pattern 2: Result expect
```rust
// ❌ Before
let value = result.expect("Failed to parse");

// ✅ After
let value = result.map_err(|e| EngineError::InvalidInput(...))?;
```

#### Pattern 3: Vec/Map indexing
```rust
// ❌ Before
let item = vec.get(10).unwrap();
let value = map.get(&key).unwrap();

// ✅ After (使用convenience.rs)
use game_engine::error::convenience::{vec_get_or_err, map_get_or_err};
let item = vec_get_or_err(&vec, 10, "index out of bounds")?;
let value = map_get_or_err(&map, &key, "key not found")?;
```

#### Pattern 4: String parsing
```rust
// ❌ Before
let num: i32 = str.parse().expect("Not a number");

// ✅ After
use game_engine::error::convenience::parse_to_number_or_err;
let num: i32 = parse_to_number_or_err(str, "Failed to parse number")?;
```

---

## 迁移批次计划

### 批次1: 核心模块（P1-6.1, 4天）

**core/engine/** (input_handler.rs: 30个)
```rust
// 文件: game_engine/src/core/engine/input_handler.rs

// 示例替换
// Before:
let device = self.devices.get(&device_id).unwrap();

// After:
use crate::error::convenience::map_get_or_err;
let device = map_get_or_err(&self.devices, &device_id, "Input device not found")?;
```

**ecs/** (entity_manager.rs: 8个, component_validator.rs: 6个)
```rust
// Before:
let entity = self.entities.get(entity_id).unwrap();

// After:
let entity = self.entities.get(entity_id)
    .ok_or_else(|| EngineError::EntityNotFound(entity_id))?;
```

**physics/** (spatial_partition.rs: 20个)
```rust
// Before:
let partition = self.partitions.get(index).unwrap();

// After:
let partition = vec_get_or_err(&self.partitions, index, "Spatial partition index")?;
```

**验收标准**:
- [ ] unwrap/expect < 50个
- [ ] 所有测试通过
- [ ] 无性能回归

---

### 批次2: 渲染和网络（P1-6.2, 3天）

**render/** (webgl_adapter.rs: 15个, domain_objects.rs: 6个)
**network/** (debugging/: 10个)

**验收标准**:
- [ ] unwrap/expect < 80个
- [ ] 渲染性能无影响

---

### 批次3: 其他模块（P1-6.3, 3天）

**xr/** (openxr_impl.rs: 10个)
**resources/** (manager.rs, streaming_loader.rs, etc.)
**audio/**, **ai/**, **scripting/**

**验收标准**:
- [ ] unwrap/expect < 100个
- [ ] 全部模块 < 10个/文件

---

### 批次4: 测试代码（P1-6.4, 1天）

**策略**: 保留必要的unwrap，添加注释

```rust
#[test]
fn test_example() {
    let result = function_under_test();

    // 测试代码中的unwrap可以接受，因为失败应该导致测试失败
    assert!(result.is_ok());
    let value = result.unwrap(); // OK: test code
}
```

**验收标准**:
- [ ] 测试代码unwrap减少50%
- [ ] 保留的unwrap都有注释

---

## 安全错误处理工具箱

### 已提供的工具（error/convenience.rs）

```rust
// Option/Result转换
safe_unwrap_option(option, context, error_msg)
safe_unwrap_result(result, context)

// 集合访问
vec_get_or_err(vec, index, context)
vec_get_mut_or_err(vec, index, context)
map_get_or_err(map, key, context)
map_get_mut_or_err(map, key, context)

// 验证
check_range_or_err(value, range, context)
check_non_empty_or_err(str, context)
ok_or_else_err(condition, error_msg)

// 解析
parse_to_number_or_err(str, context)

// 验证器链
Validator::new()
    .validate(|| condition, "error msg")
    .check()?
```

---

## 执行步骤

### Step 1: 准备（0.5天）
- [ ] 为每个批次创建feature分支
- [ ] 设置baseline指标
- [ ] 准备测试脚本

### Step 2: 批次1执行（4天）
- [ ] Day 1: core/engine/
- [ ] Day 2: ecs/
- [ ] Day 3: physics/
- [ ] Day 4: 验证和文档

### Step 3: 批次2-4执行（7天）
- [ ] 按批次计划执行
- [ ] 每日运行测试套件
- [ ] 更新追踪文档

### Step 4: 验收（0.5天）
- [ ] 统计最终unwrap/expect数量
- [ ] 性能基准测试
- [ ] 代码审查

---

## 风险与缓解

### 风险1: 性能回归
**缓解**: 使用benchmarking模块监控关键路径

### 风险2: API破坏性变更
**缓解**: 使用TryFrom/TryInto trait，保持向后兼容

### 风险3: 错误处理过度复杂
**缓解**: 使用convenience.rs简化常见模式

---

## 追踪与报告

### 每日更新模板

```markdown
## 日期: YYYY-MM-DD

### 批次进度
- 处理文件: X个
- 替换unwrap: Y个
- 替换expect: Z个

### 剩余工作
- 总unwrap: A个
- 总expect: B个
- 完成百分比: C%

### 遇到的问题
1. ...
```

---

## 成功标准

- [ ] 总unwrap/expect从1,415降至500以下
- [ ] 核心模块unwrap < 50个
- [ ] 所有unwrap都有issue追踪或注释
- [ ] 测试覆盖率不降低
- [ ] 性能无回归

---

**开始时间**: 待P0-1.5和P0-1.6完成后
**预估完成**: 12天
**负责人**: 待分配
