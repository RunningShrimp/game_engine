# 性能最佳实践指南

## 概述

本文档总结了游戏引擎项目的性能优化策略和最佳实践。

---

## 目录

1. [并发容器选择](#1-并发容器选择)
2. [异步开销优化](#2-异步开销优化)
3. [CPU密集型并行化](#3-cpu密集型并行化)
4. [SIMD优化](#4-simd优化)

## 1. 并发容器选择

### DashMap vs RwLock<HashMap>

**DashMap** (高并发):
- 并发读取: 11.83x vs std::sync::RwLock
- 并发写入: 10.86x
- 适合: 频繁读写、细粒度锁定

**RwLock<parking_lot::Mutex>>** (读多写少):
- 并发读取: 2.5x-8x vs std
- 适合: 90%+读取场景

## 2. 异步开销优化

### 使用 blocking_read()

**优化前** (80-350µs开销):
```rust
pub async fn subscriber_count(&self) -> usize {
    self.subscribers.read().await.len()
}
```

**优化后** (<1µs):
```rust
pub fn subscriber_count(&self) -> usize {
    self.subscribers.blocking_read().len()
}
```

**性能提升**: ~1000x

## 3. CPU密集型并行化

### Rayon并行化

**AI寻路批量处理**:
```rust
pub fn find_paths_batch_parallel(&self, paths: Vec<(Vec3, Vec3)>) -> Vec<Option<Vec<Vec3>>> {
    paths.par_iter()
        .map(|(start, end)| self.find_path(start, end))
        .collect()
}
```

**性能提升**: 4-8x (50+请求)

## 4. SIMD优化

### 运行时特性检测

```rust
#[cfg(target_arch = "x86_64")]
{
    if is_x86_feature_detected!("avx2") {
        return unsafe { process_avx2(data) };
    }
}
// 标量回退
process_scalar(data)
```

---
**版本**: v0.1.0
**更新**: 2025-12-31
