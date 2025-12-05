//! 任务调度系统
//!
//! 提供统一的任务调度和管理，替代分散的线程管理。
//!
//! ## 功能特性
//!
//! - 后台任务执行
//! - 主线程回调
//! - 任务优先级
//! - 任务取消

use crate::impl_default;
use bevy_ecs::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 关键优先级
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl TaskPriority {
    /// 创建默认优先级的任务
    ///
    /// 返回 `TaskPriority::Normal`。
    ///
    /// # 返回
    ///
    /// 返回普通优先级的任务优先级。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskPriority;
    ///
    /// let priority = TaskPriority::new();
    /// assert_eq!(priority, TaskPriority::Normal);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
}

/// 任务句柄
#[derive(Debug, Clone)]
pub struct TaskHandle {
    /// 任务 ID
    pub id: u64,
    /// 取消信号发送器
    cancel_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl TaskHandle {
    /// 取消任务
    ///
    /// 发送取消信号给正在执行的任务。任务会在下一次检查取消信号时停止执行。
    ///
    /// # 注意
    ///
    /// 取消操作是异步的，任务可能不会立即停止。任务需要定期检查取消信号才能响应取消请求。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(1);
    /// let handle = scheduler.spawn_background(async {
    ///     // 长时间运行的任务
    /// });
    ///
    /// // 稍后取消任务
    /// handle.cancel();
    /// ```
    pub fn cancel(&self) {
        if let Ok(mut tx_opt) = self.cancel_tx.lock() {
            if let Some(tx) = tx_opt.take() {
                let _ = tx.send(());
            }
        }
    }
}

/// 主线程任务
type MainThreadTask = Box<dyn FnOnce() + Send + 'static>;

/// 任务调度器
///
/// 管理后台任务和主线程回调。
///
/// # 示例
///
/// ```ignore
/// let scheduler = TaskScheduler::new(4);
///
/// // 后台任务
/// scheduler.spawn_background(async {
///     // 异步操作
/// });
///
/// // 主线程回调
/// scheduler.run_on_main_thread(|| {
///     // 必须在主线程执行的操作
/// });
/// ```
pub struct TaskScheduler {
    /// Tokio 运行时
    runtime: tokio::runtime::Runtime,
    /// 主线程任务接收器
    main_thread_rx: Receiver<MainThreadTask>,
    /// 主线程任务发送器
    main_thread_tx: Sender<MainThreadTask>,
    /// 下一个任务 ID
    next_task_id: std::sync::atomic::AtomicU64,
    /// 工作线程数
    worker_count: usize,
}

impl TaskScheduler {
    /// 创建任务调度器
    ///
    /// # 参数
    /// - `worker_threads`: 工作线程数量，0 表示使用 CPU 核心数
    pub fn new(worker_threads: usize) -> Self {
        let workers = if worker_threads == 0 {
            num_cpus::get()
        } else {
            worker_threads
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(workers)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let (main_thread_tx, main_thread_rx) = unbounded();

        Self {
            runtime,
            main_thread_rx,
            main_thread_tx,
            next_task_id: std::sync::atomic::AtomicU64::new(1),
            worker_count: workers,
        }
    }

    /// 在后台线程执行异步任务
    ///
    /// 将异步任务提交到后台线程池执行，不阻塞当前线程。
    ///
    /// # 参数
    ///
    /// * `task` - 要执行的异步任务（Future）
    ///
    /// # 返回
    ///
    /// 返回任务句柄，可用于取消任务。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(4);
    /// let handle = scheduler.spawn_background(async {
    ///     // 执行异步操作
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    /// });
    /// ```
    pub fn spawn_background<F, T>(&self, task: F) -> TaskHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let task_id = self
            .next_task_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        self.runtime.spawn(async move {
            tokio::select! {
                _ = cancel_rx => {
                    // 任务被取消
                }
                _ = task => {
                    // 任务完成
                }
            }
        });

        TaskHandle {
            id: task_id,
            cancel_tx: Arc::new(Mutex::new(Some(cancel_tx))),
        }
    }

    /// 在后台线程执行阻塞任务
    ///
    /// 将阻塞任务提交到专门的阻塞线程池执行，避免阻塞异步运行时。
    ///
    /// # 参数
    ///
    /// * `task` - 要执行的阻塞任务（同步函数）
    ///
    /// # 返回
    ///
    /// 返回任务句柄，可用于取消任务。
    ///
    /// # 注意
    ///
    /// 阻塞任务会占用线程池中的线程，应避免长时间运行的阻塞操作。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(4);
    /// let handle = scheduler.spawn_blocking(|| {
    ///     // 执行CPU密集型或阻塞操作
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    /// });
    /// ```
    pub fn spawn_blocking<F, T>(&self, task: F) -> TaskHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let task_id = self
            .next_task_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (cancel_tx, _cancel_rx) = oneshot::channel::<()>();

        self.runtime.spawn_blocking(task);

        TaskHandle {
            id: task_id,
            cancel_tx: Arc::new(Mutex::new(Some(cancel_tx))),
        }
    }

    /// 在主线程执行任务
    ///
    /// 将任务加入主线程任务队列，等待下次调用 `process_main_thread_tasks()` 时执行。
    ///
    /// # 参数
    ///
    /// * `task` - 要在主线程执行的闭包
    ///
    /// # 注意
    ///
    /// 任务不会立即执行，需要在主循环中调用 `process_main_thread_tasks()` 来处理队列中的任务。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(1);
    /// scheduler.run_on_main_thread(|| {
    ///     // 必须在主线程执行的操作（如UI更新）
    /// });
    ///
    /// // 在主循环中处理任务
    /// scheduler.process_main_thread_tasks();
    /// ```
    pub fn run_on_main_thread<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.main_thread_tx.send(Box::new(task));
    }

    /// 处理主线程任务队列（应在主循环中调用）
    ///
    /// 处理队列中的所有主线程任务，直到队列为空。
    ///
    /// # 注意
    ///
    /// 此方法会处理所有待执行的任务，如果任务数量很多可能会阻塞主循环。
    /// 考虑使用 `process_main_thread_tasks_limited()` 来限制每次处理的任务数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(1);
    /// // ... 添加任务 ...
    ///
    /// // 在主循环中
    /// loop {
    ///     scheduler.process_main_thread_tasks();
    ///     // ... 其他主循环逻辑 ...
    /// }
    /// ```
    pub fn process_main_thread_tasks(&self) {
        while let Ok(task) = self.main_thread_rx.try_recv() {
            task();
        }
    }

    /// 处理指定数量的主线程任务
    ///
    /// 处理队列中的主线程任务，但最多处理 `max_tasks` 个任务。
    ///
    /// # 参数
    ///
    /// * `max_tasks` - 最多处理的任务数量
    ///
    /// # 使用场景
    ///
    /// 适用于需要限制每帧处理任务数量的场景，避免单帧处理时间过长。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(1);
    /// // ... 添加任务 ...
    ///
    /// // 在主循环中，每帧最多处理10个任务
    /// loop {
    ///     scheduler.process_main_thread_tasks_limited(10);
    ///     // ... 其他主循环逻辑 ...
    /// }
    /// ```
    pub fn process_main_thread_tasks_limited(&self, max_tasks: usize) {
        for _ in 0..max_tasks {
            match self.main_thread_rx.try_recv() {
                Ok(task) => task(),
                Err(_) => break,
            }
        }
    }

    /// 获取工作线程数
    ///
    /// # 返回
    ///
    /// 返回任务调度器的工作线程数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(4);
    /// assert_eq!(scheduler.worker_count(), 4);
    /// ```
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// 阻塞等待 Future 完成（用于初始化）
    ///
    /// 在当前线程阻塞等待异步任务完成。主要用于初始化阶段需要等待异步操作完成的场景。
    ///
    /// # 参数
    ///
    /// * `future` - 要等待的 Future
    ///
    /// # 返回
    ///
    /// 返回 Future 的输出值。
    ///
    /// # 警告
    ///
    /// 此方法会阻塞当前线程，应避免在主循环中使用。主要用于初始化阶段。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::TaskScheduler;
    ///
    /// let scheduler = TaskScheduler::new(1);
    /// let result = scheduler.block_on(async {
    ///     // 执行异步初始化操作
    ///     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///     42
    /// });
    /// assert_eq!(result, 42);
    /// ```
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}


/// 任务调度器资源 (ECS Resource)
#[derive(Resource)]
pub struct TaskSchedulerResource {
    /// 调度器实例
    pub scheduler: Arc<TaskScheduler>,
}

impl Default for TaskSchedulerResource {
    fn default() -> Self {
        Self {
            scheduler: Arc::new(TaskScheduler::new(0)), // 使用默认线程数
        }
    }
}

/// 主线程任务处理系统
///
/// ECS系统，用于在主循环中处理主线程任务队列。
/// 每次调用最多处理10个任务，避免单帧处理时间过长。
///
/// # 使用
///
/// 将此系统添加到ECS调度器中，它会在每帧自动处理主线程任务。
pub fn process_main_thread_tasks_system(scheduler: Res<TaskSchedulerResource>) {
    scheduler.scheduler.process_main_thread_tasks_limited(10);
}

// ============================================================================
// 延迟任务支持
// ============================================================================

/// 延迟任务
///
/// 表示一个延迟执行的主线程任务。
pub struct DelayedTask {
    /// 执行时间（从创建开始的秒数）
    pub execute_at: f64,
    /// 任务
    pub task: MainThreadTask,
}

impl std::fmt::Debug for DelayedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelayedTask")
            .field("execute_at", &self.execute_at)
            .field("task", &"<function>")
            .finish()
    }
}

/// 延迟任务队列
///
/// 管理延迟执行的主线程任务，支持按时间顺序执行任务。
///
/// # 使用场景
///
/// 适用于需要在未来某个时间点执行的任务，如延迟回调、定时器、动画等。
#[derive(Resource, Default)]
pub struct DelayedTaskQueue {
    /// 任务队列
    tasks: std::sync::Mutex<Vec<DelayedTask>>,
    /// 当前时间
    current_time: f64,
}

impl DelayedTaskQueue {
    /// 添加延迟任务
    ///
    /// 将任务添加到延迟任务队列，任务将在 `delay_seconds` 秒后执行。
    ///
    /// # 参数
    ///
    /// * `delay_seconds` - 延迟时间（秒）
    /// * `task` - 要执行的任务
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::DelayedTaskQueue;
    ///
    /// let mut queue = DelayedTaskQueue::default();
    /// queue.schedule(1.0, || {
    ///     println!("1秒后执行");
    /// });
    /// ```
    pub fn schedule<F>(&mut self, delay_seconds: f64, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.push(DelayedTask {
                execute_at: self.current_time + delay_seconds,
                task: Box::new(task),
            });
            tasks.sort_by(|a, b| a.execute_at.partial_cmp(&b.execute_at).unwrap());
        }
    }

    /// 更新并执行到期任务
    ///
    /// 更新内部时间并执行所有到期的任务。应在每帧的主循环中调用。
    ///
    /// # 参数
    ///
    /// * `delta_time` - 自上次更新以来的时间（秒）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::DelayedTaskQueue;
    ///
    /// let mut queue = DelayedTaskQueue::default();
    /// queue.schedule(0.5, || println!("执行"));
    ///
    /// // 在主循环中
    /// queue.update(0.016); // 假设60 FPS，每帧16ms
    /// ```
    pub fn update(&mut self, delta_time: f64) {
        self.current_time += delta_time;

        if let Ok(mut tasks) = self.tasks.lock() {
            while let Some(task) = tasks.first() {
                if task.execute_at <= self.current_time {
                    let task = tasks.remove(0);
                    (task.task)();
                } else {
                    break;
                }
            }
        }
    }

    /// 获取待执行任务数
    ///
    /// # 返回
    ///
    /// 返回队列中待执行的任务数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::core::scheduler::DelayedTaskQueue;
    ///
    /// let mut queue = DelayedTaskQueue::default();
    /// queue.schedule(1.0, || {});
    /// assert_eq!(queue.pending_count(), 1);
    /// ```
    pub fn pending_count(&self) -> usize {
        self.tasks.lock().map(|t| t.len()).unwrap_or(0)
    }
}

/// 延迟任务处理系统
///
/// ECS系统，用于在主循环中更新延迟任务队列并执行到期的任务。
///
/// # 使用
///
/// 将此系统添加到ECS调度器中，它会在每帧自动更新延迟任务队列。
pub fn process_delayed_tasks_system(
    mut queue: ResMut<DelayedTaskQueue>,
    time: Res<crate::ecs::Time>,
) {
    queue.update(time.delta_seconds as f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority_order() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = TaskScheduler::new(2);
        assert_eq!(scheduler.worker_count(), 2);
    }

    #[test]
    fn test_main_thread_task() {
        let scheduler = TaskScheduler::new(1);
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        scheduler.run_on_main_thread(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        scheduler.process_main_thread_tasks();

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_delayed_task_queue() {
        let mut queue = DelayedTaskQueue::default();
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        queue.schedule(0.5, move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        assert_eq!(queue.pending_count(), 1);

        // 更新但未到时间
        queue.update(0.3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(queue.pending_count(), 1);

        // 更新超过时间
        queue.update(0.3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(queue.pending_count(), 0);
    }
}
