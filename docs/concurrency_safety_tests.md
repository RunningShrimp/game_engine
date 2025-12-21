# 并发安全测试

## 概述

并发安全测试模块提供全面的并发安全测试，确保系统在多线程环境下的正确性和性能。

## 测试覆盖

### 1. 锁安全测试

#### test_concurrent_mutex_access
测试多个线程并发访问Mutex的正确性。

- **场景**：10个线程，每个线程增加计数器1000次
- **验证**：最终计数器值应为10000
- **目的**：验证`safe_lock`的线程安全性

#### test_concurrent_rwlock_access
测试多个线程并发访问RwLock的正确性。

- **场景**：5个写线程 + 10个读线程
- **验证**：写操作正确完成，读操作不阻塞
- **目的**：验证RwLock的读写分离特性

#### test_try_lock_non_blocking
测试非阻塞锁获取。

- **场景**：一个线程持有锁，另一个线程尝试非阻塞获取
- **验证**：非阻塞获取应该失败（返回错误）
- **目的**：验证`try_lock`的非阻塞行为

#### test_lock_poison_recovery
测试锁污染恢复机制。

- **场景**：线程在持有锁时panic
- **验证**：`safe_lock`能够恢复并继续使用
- **目的**：验证锁污染恢复功能

#### test_multiple_locks_no_deadlock
测试多锁场景下的死锁预防。

- **场景**：多个线程以不同顺序获取多个锁
- **验证**：不会发生死锁
- **目的**：验证死锁预防机制

### 2. 对象池并发测试

#### test_object_pool_concurrent_access
测试对象池的并发访问。

- **场景**：20个线程并发获取和归还对象
- **验证**：统计信息正确，有缓存命中
- **目的**：验证`SyncObjectPool`的线程安全性

#### test_pool_manager_concurrent_access
测试对象池管理器的并发访问。

- **场景**：多个线程访问不同的对象池
- **验证**：所有池的统计信息正确
- **目的**：验证`PoolManager`的线程安全性

### 3. 事件总线并发测试

#### test_event_bus_concurrent_publish
测试事件总线的并发发布。

- **场景**：10个线程并发发布事件
- **验证**：所有事件都被正确处理
- **目的**：验证`SafeEventBus`的线程安全性

#### test_event_bus_concurrent_subscribe_publish
测试事件总线的并发订阅和发布。

- **场景**：多个线程同时订阅和发布事件
- **验证**：订阅者能收到所有事件
- **目的**：验证事件总线的并发安全性

### 4. 事件溯源系统并发测试

#### test_event_sourcing_concurrent_save
测试事件溯源系统的并发保存。

- **场景**：10个线程并发保存事件
- **验证**：所有事件都被正确保存
- **目的**：验证`EventSourcingManager`的线程安全性

### 5. 性能压力测试

#### test_lock_performance_under_load
测试锁在高负载下的性能。

- **场景**：100个线程，每个线程执行1000次锁操作
- **验证**：在合理时间内完成（<5秒）
- **目的**：验证锁操作的性能

#### test_object_pool_performance_under_load
测试对象池在高负载下的性能。

- **场景**：50个线程，每个线程执行1000次分配/释放
- **验证**：在合理时间内完成（<3秒）
- **目的**：验证对象池的性能

#### test_rwlock_read_heavy_workload
测试RwLock在读密集型工作负载下的性能。

- **场景**：100个读线程，每个线程执行1000次读操作
- **验证**：在合理时间内完成（<10秒）
- **目的**：验证RwLock的读并发性能

## 超时保护机制

所有并发测试都配备了超时保护机制，防止死锁或挂起导致测试无法完成。

### 超时配置

| 测试类型 | 超时时间 | 原因 |
|---------|---------|------|
| 锁安全测试 | 30秒/线程 | 标准并发操作 |
| 对象池测试 | 30秒/线程 | 标准并发操作 |
| 事件总线测试 | 30秒/线程 | 标准并发操作 |
| 事件溯源测试 | 60秒/线程 | 涉及序列化，可能较慢 |
| 性能压力测试 | 60秒/线程 | 大量操作，需要更多时间 |
| 死锁预防测试 | 30秒/线程 | 如果发生死锁应快速检测 |

### 超时实现

使用`join_with_timeout`和`join_all_with_timeout`辅助函数：

- **`join_with_timeout`**: 为单个线程的`join()`操作添加超时
- **`join_all_with_timeout`**: 等待多个线程完成，每个线程都有超时保护

### 错误报告

如果测试超时，会报告详细的错误信息，包括：
- 哪个线程超时
- 超时时间
- 可能的死锁或挂起警告

### 注意事项

- Rust标准库不支持强制终止线程，超时后线程可能仍在运行
- 超时的线程会继续运行直到自然结束，可能占用资源
- 超时时间需要根据测试复杂度合理设置

## 运行测试

### 运行所有并发测试

```bash
cargo test -p game_engine error::concurrency_tests --lib
```

### 运行特定测试组

```bash
# 锁安全测试
cargo test -p game_engine error::concurrency_tests::lock_safety_tests --lib

# 对象池并发测试
cargo test -p game_engine error::concurrency_tests::object_pool_concurrency_tests --lib

# 事件总线并发测试
cargo test -p game_engine error::concurrency_tests::event_bus_concurrency_tests --lib

# 事件溯源并发测试
cargo test -p game_engine error::concurrency_tests::event_sourcing_concurrency_tests --lib

# 性能压力测试
cargo test -p game_engine error::concurrency_tests::performance_stress_tests --lib
```

### 运行单个测试

```bash
cargo test -p game_engine test_concurrent_mutex_access --lib
```

## 测试结果解读

### 成功标准

- **正确性**：所有测试应该通过，没有数据竞争或死锁
- **性能**：性能测试应该在合理时间内完成
- **统计信息**：对象池应该有缓存命中

### 失败处理

如果测试失败：

1. **死锁**：检查锁的获取顺序，确保没有循环依赖
2. **数据竞争**：检查所有共享数据的访问是否都有锁保护
3. **性能问题**：检查锁的粒度，考虑使用更细粒度的锁

## 最佳实践

### 1. 锁的使用

- 使用`safe_lock`替代`lock().unwrap()`
- 保持锁的持有时间尽可能短
- 避免在持有锁时调用可能阻塞的操作

### 2. 对象池的使用

- 及时归还对象到池中
- 使用RAII包装器自动管理对象生命周期
- 监控池的命中率，优化池大小

### 3. 事件系统的使用

- 确保事件处理器是线程安全的
- 避免在事件处理器中执行长时间操作
- 使用批量处理减少锁竞争

## 持续集成

这些测试应该在CI/CD流程中运行，确保：

- 每次代码提交都通过并发安全测试
- 性能回归能够被及时检测
- 新功能不会引入并发安全问题

## 未来改进

- [ ] 添加更多并发场景测试
- [ ] 实现自动死锁检测
- [ ] 添加内存泄漏检测
- [ ] 实现性能基准测试
- [ ] 添加压力测试工具

