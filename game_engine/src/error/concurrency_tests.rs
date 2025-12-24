//! 并发安全测试模块
//!
//! 提供全面的并发安全测试，包括：
//! - 锁安全测试（死锁检测、锁竞争、锁污染恢复）
//! - 对象池并发测试
//! - 事件总线并发测试
//! - 事件溯源系统并发测试
//! - 性能压力测试

use crate::error::lock_safety::{safe_lock, safe_read, safe_write, try_lock, try_read, try_write};
use crate::performance::memory::{SyncObjectPool, PoolManager};
use crate::domain::events::{SafeEventBus, DomainEvent, EventError};
use crate::domain::event_sourcing::{EventSourcingManager, MemoryEventStore, MemorySnapshotStore};
use crate::domain::event_registry::EventRegistry;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// 带超时的线程join辅助函数
///
/// 如果线程在指定时间内未完成，会返回错误
/// 
/// 注意：当超时发生时，原始线程可能仍在运行，但测试会失败并报告超时
fn join_with_timeout<T: Send + 'static>(
    handle: thread::JoinHandle<T>,
    timeout: Duration,
    thread_name: &str,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    let wrapper_handle = thread::spawn(move || {
        let result = handle.join();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(join_result) => {
            // 等待包装线程完成（应该立即完成）
            let _ = wrapper_handle.join();
            join_result.map_err(|e| format!("Thread {} panicked: {:?}", thread_name, e))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // 超时：原始线程可能仍在运行，但测试应该失败
            // 注意：我们无法强制终止线程，但测试会失败
            Err(format!(
                "Thread {} did not complete within {:?} (possible deadlock or hang)",
                thread_name,
                timeout
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // 通道断开：包装线程可能panic了
            let _ = wrapper_handle.join();
            Err(format!("Thread {} channel disconnected", thread_name))
        }
    }
}

/// 等待所有线程完成，带超时保护
fn join_all_with_timeout<T: Send + 'static>(
    handles: Vec<thread::JoinHandle<T>>,
    timeout_per_thread: Duration,
) -> Result<(), String> {
    let start = Instant::now();
    let total_timeout = timeout_per_thread * (handles.len() as u32 + 1); // 给每个线程分配超时时间

    for (i, handle) in handles.into_iter().enumerate() {
        // 检查总超时
        if start.elapsed() > total_timeout {
            return Err(format!(
                "Total timeout exceeded: {} threads did not complete within {:?}",
                i,
                total_timeout
            ));
        }

        // 等待单个线程完成
        let remaining_timeout = total_timeout.saturating_sub(start.elapsed());
        let thread_timeout = remaining_timeout.min(timeout_per_thread);
        
        join_with_timeout(handle, thread_timeout, &format!("thread_{}", i))?;
    }

    Ok(())
}

#[cfg(test)]
mod lock_safety_tests {
    use super::*;

    #[test]
    fn test_concurrent_mutex_access() {
        let mutex = Arc::new(Mutex::new(0u32));
        let mut handles = Vec::new();

        // 创建10个线程，每个线程增加计数器1000次
        for _ in 0..10 {
            let mutex_clone = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let mut guard = safe_lock(&mutex_clone, "test_mutex").unwrap();
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护，每个线程最多30秒）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");

        // 验证最终值
        let guard = safe_lock(&mutex, "test_mutex_final").unwrap();
        assert_eq!(*guard, 10000);
    }

    #[test]
    fn test_concurrent_rwlock_access() {
        let rw_lock = Arc::new(RwLock::new(0u32));
        let mut handles = Vec::new();

        // 创建5个写线程
        for _ in 0..5 {
            let rw_lock_clone = Arc::clone(&rw_lock);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let mut guard = safe_write(&rw_lock_clone, "test_rwlock").unwrap();
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        // 创建10个读线程
        for _ in 0..10 {
            let rw_lock_clone = Arc::clone(&rw_lock);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let guard = safe_read(&rw_lock_clone, "test_rwlock").unwrap();
                    let _value = *guard; // 读取值
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");

        // 验证最终值（写操作应该完成）
        let guard = safe_read(&rw_lock, "test_rwlock_final").unwrap();
        assert_eq!(*guard, 500);
    }

    #[test]
    fn test_try_lock_non_blocking() {
        let mutex = Arc::new(Mutex::new(0u32));
        let mut handles = Vec::new();

        // 第一个线程持有锁
        let mutex_clone = Arc::clone(&mutex);
        let handle1 = thread::spawn(move || {
            let _guard = safe_lock(&mutex_clone, "test_try_lock").unwrap();
            thread::sleep(Duration::from_millis(100));
        });
        handles.push(handle1);

        // 等待一小段时间确保第一个线程获得锁
        thread::sleep(Duration::from_millis(10));

        // 第二个线程尝试非阻塞获取锁
        let mutex_clone = Arc::clone(&mutex);
        let handle2 = thread::spawn(move || {
            let result = try_lock(&mutex_clone, "test_try_lock");
            assert!(result.is_err()); // 应该失败（锁已被持有）
        });
        handles.push(handle2);

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(5))
            .expect("Threads did not complete within timeout");
    }

    #[test]
    fn test_lock_poison_recovery() {
        let mutex = Arc::new(Mutex::new(0u32));

        // 创建一个线程，在持有锁时panic（模拟锁污染）
        let mutex_clone = Arc::clone(&mutex);
        let handle = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("Simulated panic while holding lock");
        });

        // 等待线程panic（带超时保护）
        let _ = join_with_timeout(handle, Duration::from_secs(5), "poison_test")
            .ok(); // 忽略panic错误，因为我们期望它panic

        // 验证safe_lock能够恢复
        let result = safe_lock(&mutex, "test_poison_recovery");
        assert!(result.is_ok());
        // 锁应该已经被恢复，可以正常使用
    }

    #[test]
    fn test_multiple_locks_no_deadlock() {
        let lock1 = Arc::new(Mutex::new(0u32));
        let lock2 = Arc::new(Mutex::new(0u32));
        let mut handles = Vec::new();

        // 创建多个线程，以相同顺序获取锁（避免死锁）
        // 注意：死锁预防需要在应用层面实现（总是以相同顺序获取锁）
        for _ in 0..10 {
            let lock1_clone = Arc::clone(&lock1);
            let lock2_clone = Arc::clone(&lock2);
            let handle = thread::spawn(move || {
                // 所有线程都以相同顺序获取锁（lock1 -> lock2），避免死锁
                let _guard1 = safe_lock(&lock1_clone, "lock1").unwrap();
                thread::sleep(Duration::from_micros(10));
                let _guard2 = safe_lock(&lock2_clone, "lock2").unwrap();
                // 锁会在作用域结束时自动释放
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");
    }
}

#[cfg(test)]
mod object_pool_concurrency_tests {
    use super::*;

    #[test]
    fn test_object_pool_concurrent_access() {
        let pool = Arc::new(SyncObjectPool::new(
            || Vec::<u32>::new(),
            10,
            100,
        ));

        let mut handles = Vec::new();

        // 创建多个线程并发获取和归还对象
        for i in 0..20 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    let mut vec = pool_clone.acquire();
                    vec.push(i);
                    pool_clone.release(vec);
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");

        // 验证统计信息
        let stats = pool.stats();
        assert_eq!(stats.allocations, 1000); // 20线程 * 50次
        assert!(stats.cache_hits > 0); // 应该有缓存命中
    }

    #[test]
    fn test_pool_manager_concurrent_access() {
        let manager = Arc::new(PoolManager::new());
        let mut handles = Vec::new();

        // 测试多个池的并发访问
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                match i % 3 {
                    0 => {
                        let mut vec = manager_clone.vec_u8_pool().acquire();
                        vec.push(i as u8);
                        manager_clone.vec_u8_pool().release(vec);
                    }
                    1 => {
                        let mut vec = manager_clone.vec_f32_pool().acquire();
                        vec.push(i as f32);
                        manager_clone.vec_f32_pool().release(vec);
                    }
                    _ => {
                        let mut s = manager_clone.string_pool().acquire();
                        s.push_str(&i.to_string());
                        manager_clone.string_pool().release(s);
                    }
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");

        // 验证统计信息
        let stats = manager.stats();
        assert!(stats.vec_u8.allocations > 0);
        assert!(stats.vec_f32.allocations > 0);
        assert!(stats.string.allocations > 0);
    }
}

#[cfg(test)]
mod event_bus_concurrency_tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: u32,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn apply(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }

        fn revert(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }
    }

    #[test]
    fn test_event_bus_concurrent_publish() {
        let bus = Arc::new(SafeEventBus::new());
        let counter = Arc::new(Mutex::new(0u32));

        // 订阅事件
        let counter_clone = Arc::clone(&counter);
        bus.subscribe::<TestEvent>(move |event: &TestEvent| {
            let mut guard = safe_lock(&counter_clone, "event_counter").unwrap();
            *guard += event.value;
        });

        let mut handles = Vec::new();

        // 创建多个线程并发发布事件
        for i in 0..10 {
            let bus_clone = Arc::clone(&bus);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let event = TestEvent { value: (i * 100 + j) as u32 };
                    bus_clone.publish(&event);
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");

        // 等待事件处理完成（简单延迟）
        thread::sleep(Duration::from_millis(100));

        // 验证计数器值（0+1+2+...+999 = 499500）
        let guard = safe_lock(&counter, "event_counter_final").unwrap();
        let expected: u32 = (0..1000).sum();
        assert_eq!(*guard, expected);
    }

    #[test]
    fn test_event_bus_concurrent_subscribe_publish() {
        let bus = Arc::new(SafeEventBus::new());
        let mut handles = Vec::new();

        // 创建多个线程同时订阅和发布
        for i in 0..5 {
            let bus_clone = Arc::clone(&bus);
            let handle = thread::spawn(move || {
                // 订阅（每个线程有自己的计数器）
                let local_counter = Arc::new(Mutex::new(0u32));
                let counter_clone = Arc::clone(&local_counter);
                bus_clone.subscribe::<TestEvent>(move |event: &TestEvent| {
                    let mut guard = safe_lock(&counter_clone, "local_counter").unwrap();
                    *guard += event.value;
                });

                // 发布自己线程的事件
                let mut expected_sum = 0u32;
                for j in 0..50 {
                    let value = (i * 50 + j) as u32;
                    let event = TestEvent { value };
                    bus_clone.publish(&event);
                    expected_sum += value;
                }

                // 等待处理（给事件总线时间处理所有事件）
                thread::sleep(Duration::from_millis(200));

                // 验证本地计数器至少收到了自己发布的事件
                // 注意：由于并发，可能收到其他线程的事件，所以只验证至少收到自己发布的
                let guard = safe_lock(&local_counter, "local_counter_final").unwrap();
                assert!(
                    *guard >= expected_sum,
                    "Counter {} should be at least {} (received: {})",
                    i,
                    expected_sum,
                    *guard
                );
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(30))
            .expect("Threads did not complete within timeout");
    }
}

#[cfg(test)]
mod event_sourcing_concurrency_tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: u32,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn apply(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }

        fn revert(&self, _world: &mut World) -> Result<(), EventError> {
            Ok(())
        }
    }

    #[test]
    fn test_event_sourcing_concurrent_save() {
        let event_store: Arc<RwLock<Box<dyn crate::domain::event_sourcing::EventStore>>> =
            Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store: Arc<RwLock<Box<dyn crate::domain::event_sourcing::SnapshotStore>>> =
            Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));
        let event_registry = Arc::new(RwLock::new(EventRegistry::new()));

        // 注册事件类型
        {
            let  registry = event_registry.write().unwrap();
            registry.register::<TestEvent>("TestEvent", 1).unwrap();
        }

        let manager = Arc::new(EventSourcingManager::with_registry(
            event_store,
            snapshot_store,
            event_registry,
        ));

        let mut handles = Vec::new();

        // 创建多个线程并发保存事件
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                for j in 0..50 {
                    let event = TestEvent { value: (i * 50 + j) as u32 };
                    let world = World::default();
                    let _ = manager_clone.save_event(
                        &event,
                        Some("test_aggregate"),
                        j as u64,
                        &world,
                    );
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护）
        join_all_with_timeout(handles, Duration::from_secs(60))
            .expect("Threads did not complete within timeout");

        // 验证事件已保存（通过重放事件）
        let events = manager.replay_aggregate_events("test_aggregate", None).unwrap();
        assert_eq!(events.len(), 500); // 10线程 * 50事件
    }
}

#[cfg(test)]
mod performance_stress_tests {
    use super::*;

    #[test]
    fn test_lock_performance_under_load() {
        let mutex = Arc::new(Mutex::new(0u32));
        let start = Instant::now();

        let mut handles = Vec::new();

        // 创建大量线程进行锁操作
        for _ in 0..100 {
            let mutex_clone = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let mut guard = safe_lock(&mutex_clone, "perf_test").unwrap();
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护，最多60秒）
        join_all_with_timeout(handles, Duration::from_secs(60))
            .expect("Threads did not complete within timeout");

        let duration = start.elapsed();
        println!("Lock performance test: {}ms for 100k operations", duration.as_millis());

        // 验证正确性
        let guard = safe_lock(&mutex, "perf_test_final").unwrap();
        assert_eq!(*guard, 100000);

        // 性能断言：应该在合理时间内完成（例如5秒内）
        assert!(duration.as_secs() < 5);
    }

    #[test]
    fn test_object_pool_performance_under_load() {
        let pool = Arc::new(SyncObjectPool::new(
            || Vec::<u8>::with_capacity(1024),
            100,
            1000,
        ));

        let start = Instant::now();
        let mut handles = Vec::new();

        // 创建多个线程进行大量分配/释放操作
        for _ in 0..50 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let vec = pool_clone.acquire();
                    pool_clone.release(vec);
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护，最多60秒）
        join_all_with_timeout(handles, Duration::from_secs(60))
            .expect("Threads did not complete within timeout");

        let duration = start.elapsed();
        println!("Object pool performance test: {}ms for 50k operations", duration.as_millis());

        // 验证统计信息
        let stats = pool.stats();
        assert_eq!(stats.allocations, 50000);
        assert!(stats.cache_hits > 0);

        // 性能断言：应该在合理时间内完成
        assert!(duration.as_secs() < 3);
    }

    #[test]
    fn test_rwlock_read_heavy_workload() {
        let rw_lock = Arc::new(RwLock::new(0u32));
        let start = Instant::now();

        let mut handles = Vec::new();

        // 创建大量读线程
        for _ in 0..100 {
            let rw_lock_clone = Arc::clone(&rw_lock);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    let _guard = safe_read(&rw_lock_clone, "read_heavy").unwrap();
                    // 模拟读取操作
                    thread::sleep(Duration::from_micros(1));
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成（带超时保护，最多60秒）
        join_all_with_timeout(handles, Duration::from_secs(60))
            .expect("Threads did not complete within timeout");

        let duration = start.elapsed();
        println!("RwLock read-heavy workload: {}ms", duration.as_millis());

        // 性能断言：读操作应该能够并发执行
        assert!(duration.as_secs() < 10);
    }
}

