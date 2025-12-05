# Clippy警告修复指南

**创建日期**: 2025-12-03  
**目的**: 记录clippy警告和修复建议

---

## 常见警告类型

### 1. 不可达代码 (unreachable_code)

**位置**: `game_engine_simd/src/math/dispatch.rs`

**问题**: 在return语句后有不可达代码

**修复**: 移除return后的代码或重构逻辑

### 2. 未使用的变量 (unused_variable)

**修复**: 
- 如果确实不需要，使用 `_` 前缀
- 如果将来需要，添加 `#[allow(unused)]` 属性

### 3. 未使用的导入 (unused_import)

**修复**: 移除未使用的导入

### 4. 不需要的可变变量 (variable_does_not_need_to_be_mutable)

**修复**: 移除 `mut` 关键字

---

## 自动修复

运行以下命令自动修复部分警告：

```bash
# 主项目
cargo clippy --fix --lib

# SIMD子项目
cargo clippy --fix --lib -p game_engine_simd

# Hardware子项目
cargo clippy --fix --lib -p game_engine_hardware
```

---

## 手动修复优先级

### 高优先级（影响代码质量）

1. 不可达代码
2. 未使用的导入
3. 不需要的可变变量

### 中优先级（代码风格）

1. 未使用的变量
2. 未使用的字段
3. 未使用的方法

### 低优先级（可选）

1. 未使用的枚举变体
2. 意外的cfg条件

---

## 更新记录

- 2025-12-03: 创建clippy修复指南

