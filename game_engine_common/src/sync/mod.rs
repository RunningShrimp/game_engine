//! 同步原语和线程安全数据结构
//!
//! 提供高性能的线程安全类型，包括原子操作、锁包装器和同步队列。
//!
//! ## 类型
//!
//! - [`AtomicCounter`]: 原子计数器，线程安全的整数计数
//! - [`AtomicFlag`]: 原子布尔标志
//! - [`RwLockWrapper<T>`]: 带指标跟踪的读写锁包装器
//! - [`SynchronizedQueue<T>`]: 基于通道的同步队列
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine_common::sync::{AtomicCounter, SynchronizedQueue};
//!
//! let counter = AtomicCounter::new(0);
//! counter.increment();
//! assert_eq!(counter.get(), 1);
//!
//! let queue = SynchronizedQueue::new();
//! queue.push(42);
//! assert_eq!(queue.pop(), Some(42));
//! ```

mod synchronized;

pub use synchronized::*;
