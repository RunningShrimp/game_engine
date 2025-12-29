# P0-1.5: 移除命名规范豁免

**任务状态**: 🟡 进行中
**预估**: 1天
**影响文件**: 2个

---

## 分析结果

### 扫描结果

**豁免类型**: `non_snake_case`, `non_camel_case_types`, `non_upper_case_globals`

**发现文件**:
1. `game_engine/src/network/key_exchange.rs`
2. `game_engine/src/core/engine/engine.rs`

### 非snake_case函数搜索
**结果**: 未发现违规（搜索`fn [a-z]+[A-Z]`返回空）

**结论**: 大部分代码已遵循命名规范，只需处理少量局部豁免。

---

## 处理策略

### 策略1: 局部allow + 注释
**适用**: 必须保留的非规范命名（外部API、FFI、协议规范）

```rust
// ✅ After
#[allow(non_snake_case)]  // 保留：匹配外部协议规范
pub struct TLSConfig {
    pub min_version: String,
}
```

### 策略2: 重命名
**适用**: 可以自由修改的内部代码

```rust
// ❌ Before
struct HTTPResponse { }

// ✅ After
struct HttpResponse { }
```

---

## 执行计划

### Step 1: 检查局部allow（0.5小时）

```bash
# 查找所有局部命名allow
grep -r "#\[allow(non_snake_case)\]" game_engine/src
grep -r "#\[allow(non_camel_case_types)\]" game_engine/src
grep -r "#\[allow(non_upper_case_globals)\]" game_engine/src
```

### Step 2: 逐文件处理（0.5天）

#### 文件1: network/key_exchange.rs
- 检查局部allow的原因
- 添加注释说明
- 或重命名以符合规范

#### 文件2: core/engine/engine.rs
- 同上处理

### Step 3: 移除全局豁免（0.5小时）

从lib.rs移除：
```rust
#![allow(
    non_snake_case,           // ← 移除
    non_camel_case_types,     // ← 移除
    non_upper_case_globals,   // ← 移除
    // ...
)]
```

---

## 验收标准

- [ ] 从lib.rs移除3个命名豁免
- [ ] 所有命名违规都有局部`#[allow]`和注释
- [ ] `cargo clippy`无命名警告
- [ ] `cargo build`成功

---

## 风险评估

### 低风险
- 影响文件少（2个）
- 未发现大规模违规
- 命名变更不影响ABI（Rust无稳定ABI）

### 注意事项
- 重命名public API需要考虑下游用户
- FFI函数必须匹配外部库签名
- 协议相关的字段名可能需要保留

---

**开始时间**: 待P0-1.6完成后
**预计完成**: 同一天
**状态**: 准备就绪
