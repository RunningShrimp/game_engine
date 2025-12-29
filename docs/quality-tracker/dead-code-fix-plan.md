# P0-1.3: 移除dead_code和unreachable_pub豁免

**任务状态**: 🟡 进行中
**依赖**: P0-1.2完成
**预估**: 1天

---

## 策略

### dead_code处理
**原则**: 不应在全局级别豁免dead_code

**方法**:
1. 在具体函数/结构体上添加局部`#[allow(dead_code)]`
2. 添加注释说明保留原因
3. 对于真正不需要的代码，考虑删除

**示例**:
```rust
// ❌ Before (全局豁免)
#![allow(dead_code)]
fn deprecated_function() { }

// ✅ After (局部豁免 + 注释)
#[allow(dead_code)]  // 保留用于API兼容性
fn deprecated_function() { }
```

### unreachable_pub处理
**原则**: 公开API应该被使用

**方法**:
1. 移除不需要的pub
2. 或添加文档说明使用场景
3. 使用#[deprecated]标记过时API

**示例**:
```rust
// ❌ Before
#![allow(unreachable_pub)]
pub mod internal {
    pub fn helper() { }
}

// ✅ Option 1: 移除pub
pub mod internal {
    fn helper() { }  // 内部函数
}

// ✅ Option 2: 添加文档
pub mod internal {
    /// 内部辅助函数，仅供模块内部使用
    #[doc(hidden)]
    pub fn helper() { }
}
```

---

## 执行计划

### 步骤1: 搜索dead_code使用
```bash
grep -r "dead_code" game_engine/src --include="*.rs" | wc -l
```

### 步骤2: 逐模块处理

**优先级**:
1. core/engine/
2. ecs/
3. domain/
4. 其他模块

### 步骤3: 处理unreachable_pub
```bash
# 查找可能不需要pub的项
grep -r "pub fn" game_engine/src/core | wc -l
grep -r "pub struct" game_engine/src/core | wc -l
```

---

## 验收标准

- [ ] 从lib.rs移除dead_code
- [ ] 从lib.rs移除unreachable_pub
- [ ] 所有dead_code都有局部#[allow]和注释
- [ ] 所有不必要的pub都已移除或标注
- [ ] cargo build成功

---

**开始时间**: 2025-12-28 (P0-1.2完成后立即开始)
