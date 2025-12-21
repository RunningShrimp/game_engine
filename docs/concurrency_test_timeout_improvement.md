# 并发测试超时机制改进

## 概述

为所有并发安全测试添加了超时保护机制，防止死锁或挂起导致测试无法完成。

## 改进内容

### 1. 超时辅助函数

实现了两个辅助函数：

#### `join_with_timeout<T>`
- 功能：为单个线程的`join()`操作添加超时
- 参数：
  - `handle`: 线程句柄
  - `timeout`: 超时时间
  - `thread_name`: 线程名称（用于错误报告）
- 返回：`Result<T, String>`
- 行为：如果线程在指定时间内未完成，返回错误信息

#### `join_all_with_timeout<T>`
- 功能：等待多个线程完成，每个线程都有超时保护
- 参数：
  - `handles`: 线程句柄向量
  - `timeout_per_thread`: 每个线程的超时时间
- 返回：`Result<(), String>`
- 行为：
  - 逐个等待线程完成
  - 检查总超时时间
  - 为每个线程分配剩余的超时时间

### 2. 超时配置

不同测试使用不同的超时时间：

| 测试类型 | 超时时间 | 原因 |
|---------|---------|------|
| 锁安全测试 | 30秒/线程 | 标准并发操作 |
| 对象池测试 | 30秒/线程 | 标准并发操作 |
| 事件总线测试 | 30秒/线程 | 标准并发操作 |
| 事件溯源测试 | 60秒/线程 | 涉及序列化，可能较慢 |
| 性能压力测试 | 60秒/线程 | 大量操作，需要更多时间 |
| 死锁预防测试 | 10秒/线程 | 如果发生死锁应快速检测 |

### 3. 错误处理

当超时发生时：
- 测试会立即失败
- 报告详细的错误信息：
  - 哪个线程超时
  - 超时时间
  - 可能的死锁或挂起警告
- 不会无限期等待

## 使用示例

### 之前（无超时保护）

```rust
// 等待所有线程完成
for handle in handles {
    handle.join().unwrap(); // 可能永远挂起
}
```

### 之后（带超时保护）

```rust
// 等待所有线程完成（带超时保护）
join_all_with_timeout(handles, Duration::from_secs(30))
    .expect("Threads did not complete within timeout");
```

## 优势

1. **防止测试挂起**：超时机制确保测试不会无限期等待
2. **快速失败**：死锁或问题能够快速检测
3. **清晰的错误信息**：超时时提供详细的错误报告
4. **CI/CD友好**：不会导致CI/CD管道挂起

## 注意事项

1. **线程无法强制终止**：Rust标准库不支持强制终止线程，超时后线程可能仍在运行
2. **资源清理**：超时的线程会继续运行直到自然结束，可能占用资源
3. **超时时间选择**：需要根据测试复杂度合理设置超时时间

## 测试验证

所有并发测试都已更新为使用超时机制：

- ✅ `test_concurrent_mutex_access` - 30秒超时
- ✅ `test_concurrent_rwlock_access` - 30秒超时
- ✅ `test_try_lock_non_blocking` - 5秒超时
- ✅ `test_lock_poison_recovery` - 5秒超时
- ✅ `test_multiple_locks_no_deadlock` - 10秒超时
- ✅ `test_object_pool_concurrent_access` - 30秒超时
- ✅ `test_pool_manager_concurrent_access` - 30秒超时
- ✅ `test_event_bus_concurrent_publish` - 30秒超时
- ✅ `test_event_bus_concurrent_subscribe_publish` - 5秒超时
- ✅ `test_event_sourcing_concurrent_save` - 60秒超时
- ✅ `test_lock_performance_under_load` - 60秒超时
- ✅ `test_object_pool_performance_under_load` - 60秒超时
- ✅ `test_rwlock_read_heavy_workload` - 60秒超时

## 未来改进

- [ ] 考虑使用`std::thread::scope`（Rust 1.63+）简化超时实现
- [ ] 添加可配置的超时时间（通过环境变量）
- [ ] 实现线程泄漏检测
- [ ] 添加超时统计信息

