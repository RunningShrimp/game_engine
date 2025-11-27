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

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::future::Future;
use crossbeam_channel::{Sender, Receiver, unbounded};
use tokio::sync::oneshot;
use bevy_ecs::prelude::*;

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
    pub fn spawn_background<F, T>(&self, task: F) -> TaskHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let task_id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        
        TaskHandle { id: task_id, cancel_tx: Arc::new(Mutex::new(Some(cancel_tx))) }
    }
    
    /// 在后台线程执行阻塞任务
    pub fn spawn_blocking<F, T>(&self, task: F) -> TaskHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let task_id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (cancel_tx, _cancel_rx) = oneshot::channel::<()>();
        
        self.runtime.spawn_blocking(task);
        
        TaskHandle { id: task_id, cancel_tx: Arc::new(Mutex::new(Some(cancel_tx))) }
    }
    
    /// 在主线程执行任务
    pub fn run_on_main_thread<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.main_thread_tx.send(Box::new(task));
    }
    
    /// 处理主线程任务队列（应在主循环中调用）
    pub fn process_main_thread_tasks(&self) {
        while let Ok(task) = self.main_thread_rx.try_recv() {
            task();
        }
    }
    
    /// 处理指定数量的主线程任务
    pub fn process_main_thread_tasks_limited(&self, max_tasks: usize) {
        for _ in 0..max_tasks {
            match self.main_thread_rx.try_recv() {
                Ok(task) => task(),
                Err(_) => break,
            }
        }
    }
    
    /// 获取工作线程数
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
    
    /// 阻塞等待 Future 完成（用于初始化）
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new(0)
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
            scheduler: Arc::new(TaskScheduler::default()),
        }
    }
}

/// 主线程任务处理系统
pub fn process_main_thread_tasks_system(scheduler: Res<TaskSchedulerResource>) {
    scheduler.scheduler.process_main_thread_tasks_limited(10);
}

// ============================================================================
// 延迟任务支持
// ============================================================================

/// 延迟任务
pub struct DelayedTask {
    /// 执行时间（从创建开始的秒数）
    pub execute_at: f64,
    /// 任务
    pub task: MainThreadTask,
}

/// 延迟任务队列
#[derive(Resource, Default)]
pub struct DelayedTaskQueue {
    /// 任务队列
    tasks: std::sync::Mutex<Vec<DelayedTask>>,
    /// 当前时间
    current_time: f64,
}

impl DelayedTaskQueue {
    /// 添加延迟任务
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
    pub fn pending_count(&self) -> usize {
        self.tasks.lock().map(|t| t.len()).unwrap_or(0)
}
}

/// 延迟任务处理系统
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
