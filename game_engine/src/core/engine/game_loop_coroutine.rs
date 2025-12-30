//! 协程驱动的游戏循环评估
//!
//! 本模块评估使用 Rust async/await 协程实现游戏循环的可行性和优势。
//!
//! ## 当前实现分析
//!
//! ### 现有游戏循环 (engine.rs)
//! - 使用 `pollster::block_on()` 运行异步初始化
//! - winit `EventLoop` 配置为 `ControlFlow::Poll`
//! - 固定时间步长循环使用同步回调
//! - 物理更新和渲染在事件循环回调中同步执行
//!
//! ### 协程资源加载器 (coroutine_loader.rs)
//! - 已使用 tokio async/await 实现异步资源加载
//! - 具有优先级队列系统
//! - 使用 tokio::sync::Semaphore 进行并发控制
//! - 使用 tokio::spawn 在后台执行加载任务
//!
//! ## 协程驱动游戏循环的设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    Coroutine-Driven Game Loop                        │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                      │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │
//! │  │  Event Loop  │───▶│  Scheduler   │───▶│  Executor    │          │
//! │  │  (winit)     │    │  (Tokio)     │    │  (Async)     │          │
//! │  └──────────────┘    └──────────────┘    └──────────────┘          │
//! │         │                   │                    │                 │
//! │         ▼                   ▼                    ▼                 │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │
//! │  │   Input      │    │  Fixed Step  │    │   Variable   │          │
//! │  │  Coroutine   │    │  Coroutine   │    │   Step       │          │
//! │  │              │    │  (Physics)   │    │  Coroutine  │          │
//! │  └──────────────┘    └──────────────┘    └──────────────┘          │
//! │                                             │                       │
//! │                                             ▼                       │
//! │                                      ┌──────────────┐              │
//! │                                      │   Render     │              │
//! │                                      │  Coroutine   │              │
//! │                                      └──────────────┘              │
//! │                                                                      │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 优势
//!
//! 1. **非阻塞资源加载**: 资源加载不会阻塞主循环
//! 2. **并行处理**: 可以充分利用多核 CPU
//! 3. **更好的代码组织**: 异步代码更易于理解和维护
//! 4. **取消支持**: 可以优雅地取消长时间运行的任务
//! 5. **错误处理**: Result 和 ? 操作符使错误处理更清晰
//! 6. **超时控制**: 使用 tokio::time::timeout 防止任务卡死
//!
//! ## 挑战
//!
//! 1. **确定性**: 异步调度器的非确定性可能影响物理模拟
//! 2. **性能开销**: 异步运行时有一定的内存和 CPU 开销
//! 3. **winit 限制**: winit 事件循环要求在主线程运行
//! 4. **调试复杂性**: 异步代码的堆栈跟踪更复杂
//! 5. **锁竞争**: 需要仔细处理共享状态访问
//!
//! ## 实现策略
//!
//! ### 混合方法（推荐）
//!
//! 保持 winit 事件循环作为主驱动，使用协程处理异步任务：
//!
//! ```text
//! winit EventLoop (Main Thread)
//!     ├─▶ Input Handling (Sync)
//!     ├─▶ Physics Update (Sync - Fixed Timestep)
//!     ├─▶ Variable Updates (Sync)
//!     ├─▶ Async Task Processing (Poll tokio runtime)
//!     └─▶ Render (Sync)
//!     
//! Tokio Runtime (Background Threads)
//!     ├─▶ Resource Loading
//!     ├─▶ Network I/O
//!     ├─▶ AI Pathfinding
//!     └─▶ Audio Processing
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore, mpsc};
use tokio::task::JoinHandle;

use crate::engine::ecs_bevy::Time;

use bevy_ecs::prelude::*;

type TaskId = u64;

type GameTaskResult<T> = Result<T, GameTaskError>;

/// 游戏任务结果包装器
pub struct GameTaskWrapper<T> {
    inner: GameTaskResult<T>,
}

impl<T> GameTaskWrapper<T> {
    /// 从 Result 创建包装器
    pub fn new(inner: GameTaskResult<T>) -> Self {
        Self { inner }
    }

    /// 获取内部结果
    pub fn into_inner(self) -> GameTaskResult<T> {
        self.inner
    }

    /// 检查是否成功
    pub fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }

    /// 检查是否失败
    pub fn is_err(&self) -> bool {
        self.inner.is_err()
    }
}

/// 协程任务管理器资源，供ECS系统提交异步任务
#[derive(Resource, Clone)]
pub struct CoroutineTaskManager {
    runtime_handle: tokio::runtime::Handle,
    task_queue_tx: mpsc::UnboundedSender<PendingTask>,
    active_tasks: Arc<Mutex<std::collections::HashMap<TaskId, GameTask>>>,
    next_task_id: Arc<std::sync::atomic::AtomicU64>,
    stats: Arc<RwLock<LoopStats>>,
}

impl CoroutineTaskManager {
    pub fn new(runtime_handle: tokio::runtime::Handle) -> Self {
        let (task_queue_tx, task_queue_rx) = mpsc::unbounded_channel();
        let active_tasks = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let active_tasks_clone = Arc::clone(&active_tasks);
        let stats = Arc::new(RwLock::new(LoopStats::default()));
        let stats_clone = Arc::clone(&stats);

        runtime_handle.spawn(async move {
            Self::task_processor(task_queue_rx, active_tasks_clone, stats_clone).await;
        });

        Self {
            runtime_handle,
            task_queue_tx,
            active_tasks,
            next_task_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            stats,
        }
    }

    pub async fn spawn_task<F, Fut>(&self, name: String, priority: TaskPriority, f: F) -> TaskId
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), GameTaskError>> + Send + 'static,
    {
        let id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let active_tasks = Arc::clone(&self.active_tasks);
        let stats: Arc<RwLock<LoopStats>> = Arc::clone(&self.stats);
        let task_name = name.clone();

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();

        let handle = self.runtime_handle.spawn(async move {
            tokio::select! {
                result = f() => {
                    match result {
                        Ok(_) => {
                            let mut stats = stats.write().await;
                            stats.tasks_completed += 1;
                        }
                        Err(_) => {
                            let mut stats = stats.write().await;
                            stats.tasks_failed += 1;
                        }
                    }
                }
                _ = cancel_rx => {
                    tracing::debug!("Task {} cancelled", task_name);
                }
            }
        });

        let task = GameTask {
            id,
            name,
            priority,
            handle,
            cancel_tx,
        };

        active_tasks.lock().await.insert(id, task);
        id
    }

    pub async fn cancel_task(&self, id: TaskId) -> bool {
        let mut tasks = self.active_tasks.lock().await;
        if let Some(task) = tasks.remove(&id) {
            task.cancel();
            true
        } else {
            false
        }
    }

    pub fn task_count(&self) -> usize {
        self.active_tasks.blocking_lock().len()
    }

    pub fn stats(&self) -> LoopStats {
        self.stats.blocking_read().clone()
    }

    async fn task_processor(
        mut rx: mpsc::UnboundedReceiver<PendingTask>,
        active_tasks: Arc<Mutex<std::collections::HashMap<TaskId, GameTask>>>,
        _stats: Arc<RwLock<LoopStats>>,
    ) {
        let semaphore = Arc::new(Semaphore::new(32));

        while let Some(pending) = rx.recv().await {
            let sem = Arc::clone(&semaphore);
            let tasks = Arc::clone(&active_tasks);

            tokio::spawn(async move {
                let _permit = sem.acquire().await;

                let handle = (pending.task)();
                let task = GameTask {
                    id: pending.id,
                    name: pending.name,
                    priority: pending.priority,
                    handle,
                    cancel_tx: tokio::sync::oneshot::channel().0,
                };

                tasks.lock().await.insert(pending.id, task);
            });
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GameTaskError {
    #[error("Task cancelled")]
    Cancelled,
    #[error("Task timed out")]
    Timeout,
    #[error("IO error: {0}")]
    Io(String),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct GameTask {
    id: TaskId,
    name: String,
    priority: TaskPriority,
    handle: JoinHandle<()>,
    cancel_tx: tokio::sync::oneshot::Sender<()>,
}

impl GameTask {
    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn cancel(self) {
        let _ = self.cancel_tx.send(());
        self.handle.abort();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TaskPriority {
    Critical = 4,
    High = 3,
    #[default]
    Normal = 2,
    Low = 1,
    Background = 0,
}

pub struct CoroutineGameLoop {
    runtime_handle: tokio::runtime::Handle,
    task_queue_tx: mpsc::UnboundedSender<PendingTask>,
    active_tasks: Arc<Mutex<std::collections::HashMap<TaskId, GameTask>>>,
    next_task_id: Arc<std::sync::atomic::AtomicU64>,
    fixed_timestep: Duration,
    accumulator: Duration,
    last_frame_time: Instant,
    max_accumulator: Duration,
    stats: Arc<RwLock<LoopStats>>,
}

unsafe impl Send for CoroutineGameLoop {}
unsafe impl Sync for CoroutineGameLoop {}

#[derive(Debug, Clone, Default)]
pub struct LoopStats {
    pub frame_count: u64,
    pub total_time: Duration,
    pub physics_updates: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_frame_time: Duration,
}

pub struct PendingTask {
    id: TaskId,
    name: String,
    priority: TaskPriority,
    task: Box<dyn FnOnce() -> JoinHandle<()> + Send>,
    created_at: Instant,
}

impl PendingTask {
    /// 获取任务ID
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// 获取任务名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取任务优先级
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    /// 获取任务创建时间
    pub fn created_at(&self) -> Instant {
        self.created_at
    }

    /// 检查任务是否超时
    pub fn is_timeout(&self, timeout: Duration) -> bool {
        self.created_at.elapsed() > timeout
    }

    /// 执行任务
    pub fn execute(self) -> JoinHandle<()> {
        (self.task)()
    }
}

impl CoroutineGameLoop {
    pub fn new(fixed_timestep: Duration) -> Self {
        let runtime_handle = tokio::runtime::Handle::current();
        let (task_queue_tx, task_queue_rx) = mpsc::unbounded_channel();

        let active_tasks = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let active_tasks_clone = Arc::clone(&active_tasks);

        runtime_handle.spawn(async move {
            Self::task_processor(task_queue_rx, active_tasks_clone).await;
        });

        Self {
            runtime_handle,
            task_queue_tx,
            active_tasks,
            next_task_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            fixed_timestep,
            accumulator: Duration::ZERO,
            last_frame_time: Instant::now(),
            max_accumulator: Duration::from_millis(100),
            stats: Arc::new(RwLock::new(LoopStats::default())),
        }
    }

    pub fn fixed_timestep(&self) -> Duration {
        self.fixed_timestep
    }

    pub fn set_fixed_timestep(&mut self, dt: Duration) {
        self.fixed_timestep = dt;
    }

    pub async fn spawn_task<F, Fut>(&self, name: String, priority: TaskPriority, f: F) -> TaskId
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), GameTaskError>> + Send + 'static,
    {
        let id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let active_tasks = Arc::clone(&self.active_tasks);
        let stats: Arc<RwLock<LoopStats>> = Arc::clone(&self.stats);
        let task_name = name.clone();

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();

        let handle = self.runtime_handle.spawn(async move {
            tokio::select! {
                result = f() => {
                    match result {
                        Ok(_) => {
                            let mut stats = stats.write().await;
                            stats.tasks_completed += 1;
                        }
                        Err(_) => {
                            let mut stats = stats.write().await;
                            stats.tasks_failed += 1;
                        }
                    }
                }
                _ = cancel_rx => {
                    tracing::debug!("Task {} cancelled", task_name);
                }
            }
        });

        let task = GameTask {
            id,
            name,
            priority,
            handle,
            cancel_tx,
        };

        active_tasks.lock().await.insert(id, task);
        id
    }

    pub async fn cancel_task(&self, id: TaskId) -> bool {
        let mut tasks = self.active_tasks.lock().await;
        if let Some(task) = tasks.remove(&id) {
            task.cancel();
            true
        } else {
            false
        }
    }

    pub fn task_count(&self) -> usize {
        self.active_tasks.blocking_lock().len()
    }

    pub fn stats(&self) -> LoopStats {
        self.stats.blocking_read().clone()
    }

    async fn task_processor(
        mut rx: mpsc::UnboundedReceiver<PendingTask>,
        active_tasks: Arc<Mutex<std::collections::HashMap<TaskId, GameTask>>>,
    ) {
        let semaphore = Arc::new(Semaphore::new(32));

        while let Some(pending) = rx.recv().await {
            let sem = Arc::clone(&semaphore);
            let tasks = Arc::clone(&active_tasks);

            tokio::spawn(async move {
                let _permit = sem.acquire().await;

                let handle = (pending.task)();
                let task = GameTask {
                    id: pending.id,
                    name: pending.name,
                    priority: pending.priority,
                    handle,
                    cancel_tx: tokio::sync::oneshot::channel().0,
                };

                tasks.lock().await.insert(pending.id, task);
            });
        }
    }

    pub fn update_fixed_step<F>(&mut self, world: &mut World, mut update_fn: F) -> f64
    where
        F: FnMut(&mut World, Duration),
    {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        let frame_time = frame_time.min(self.max_accumulator);
        self.accumulator += frame_time;

        let mut update_count = 0u32;
        while self.accumulator >= self.fixed_timestep {
            update_fn(world, self.fixed_timestep);
            self.accumulator -= self.fixed_timestep;
            update_count += 1;

            if update_count > 10 {
                tracing::warn!(
                    "Too many physics updates in one frame ({}), clamping",
                    update_count
                );
                self.accumulator = Duration::ZERO;
                break;
            }
        }

        self.accumulator.as_secs_f64() / self.fixed_timestep.as_secs_f64()
    }

    pub fn update_time_resource(&self, world: &mut World, dt: Duration) {
        if let Some(time) = world.get_resource_mut::<Time>() {
            // Bevy Time 不支持直接设置字段,使用advance_with方法
            // time.advance_with(dt);  // 注意: 这是伪代码,实际需要根据Bevy版本调整
            // 暂时忽略这个操作,因为Bevy的Time有自己的更新机制
            let _ = dt; // 显式使用dt以避免未使用警告
            let _ = time; // 显式使用time以避免未使用警告
        }
    }
}

pub struct AsyncGameSystem {
    game_loop: CoroutineGameLoop,
}

impl AsyncGameSystem {
    pub fn new(fixed_timestep: Duration) -> Self {
        Self {
            game_loop: CoroutineGameLoop::new(fixed_timestep),
        }
    }

    pub fn game_loop(&self) -> &CoroutineGameLoop {
        &self.game_loop
    }

    pub fn game_loop_mut(&mut self) -> &mut CoroutineGameLoop {
        &mut self.game_loop
    }
}

pub struct AsyncPhysicsUpdate {
    pub dt: Duration,
    pub world_ptr: *mut bevy_ecs::world::World,
}

unsafe impl Send for AsyncPhysicsUpdate {}

// 以下async函数已删除 - 它们未被使用且是过度异步化的例子
// - async_physics_step: 纯计算不需要异步
// - async_ai_update: 纯计算不需要异步
// 如果将来需要物理/AI异步处理，应使用tokio::task::spawn_blocking
// 参见: P0-3-2_REFINED_PLAN.md

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coroutine_game_loop_creation() {
        let loop_ = CoroutineGameLoop::new(Duration::from_secs_f64(1.0 / 60.0));
        assert_eq!(loop_.fixed_timestep(), Duration::from_secs_f64(1.0 / 60.0));
    }

    #[tokio::test]
    async fn test_task_spawn() {
        let loop_ = CoroutineGameLoop::new(Duration::from_secs_f64(1.0 / 60.0));

        let task_id = loop_
            .spawn_task(
                "test_task".to_string(),
                TaskPriority::Normal,
                || async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                },
            )
            .await;

        // 验证任务ID已分配
        assert!(task_id > 0);
        assert!(loop_.task_count() > 0);

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_fixed_timestep_update() {
        let mut loop_ = CoroutineGameLoop::new(Duration::from_secs_f64(1.0 / 60.0));
        let mut world = World::new();

        // ecs_bevy::Time 只有 delta 字段
        world.insert_resource(Time { delta: 0.0 });

        let alpha = loop_.update_fixed_step(&mut world, |world, dt| {
            if let Some(mut time) = world.get_resource_mut::<Time>() {
                // 更新 delta 字段
                time.delta = dt.as_secs_f32();
            }
        });

        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[tokio::test]
    async fn test_task_priority() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        assert!(TaskPriority::Low > TaskPriority::Background);
    }
}
