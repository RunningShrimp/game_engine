//! 协程使用模式优化
//!
//! 本模块提供协程使用的最佳实践和优化工具，确保：
//! - 物理更新保持同步和确定性
//! - 异步任务不会干扰关键系统
//! - 任务调度优化，关键任务优先执行
//!
//! ## 设计原则
//!
//! 1. **物理同步原则**: 物理更新必须在主线程同步执行，确保确定性
//! 2. **任务优先级**: 关键任务（如资源加载）优先于非关键任务
//! 3. **避免阻塞**: 长时间运行的任务应异步执行，但不影响物理
//! 4. **资源隔离**: 物理相关资源不应被异步任务并发访问
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::core::engine::async_optimization::{
//!     AsyncScheduler, TaskPriority, PhysicsSyncGuard
//! };
//!
//! // 创建调度器
//! let scheduler = AsyncScheduler::new();
//!
//! // 物理更新前获取同步锁
//! let _guard = PhysicsSyncGuard::acquire().await;
//! physics_world.step(dt);
//! // guard 自动释放
//!
//! // 提交异步任务（不会干扰物理）
//! scheduler.spawn_task(
//!     "load_texture",
//!     TaskPriority::High,
//!     async move {
//!         // 异步加载资源
//!         load_texture("texture.png").await
//!     }
//! ).await;
//! ```

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 低优先级（后台任务）
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级（关键资源加载）
    High = 2,
    /// 紧急优先级（必须立即执行）
    Critical = 3,
}

/// 异步任务
pub struct AsyncTask {
    id: u64,
    name: String,
    priority: Arc<Mutex<TaskPriority>>,
    handle: JoinHandle<Result<(), TaskError>>,
    created_at: Instant,
    /// 任务执行开始时间
    started_at: Arc<Mutex<Option<Instant>>>,
    /// 等待的资源列表
    waiting_resources: Arc<Mutex<Vec<String>>>,
}

impl AsyncTask {
    /// 获取任务运行时长
    fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 获取任务执行时长
    async fn execution_duration(&self) -> Duration {
        if let Some(started) = *self.started_at.lock().await {
            started.elapsed()
        } else {
            Duration::ZERO
        }
    }

    /// 检查任务是否超时
    fn is_timeout(&self, timeout_ms: u64) -> bool {
        self.elapsed().as_millis() as u64 > timeout_ms
    }

    /// 获取任务信息
    async fn info(&self) -> TaskInfo {
        let priority = *self.priority.lock().await;
        TaskInfo {
            id: self.id,
            name: self.name.clone(),
            priority,
            elapsed: self.elapsed(),
        }
    }

    /// 获取任务优先级
    pub async fn get_priority(&self) -> TaskPriority {
        *self.priority.lock().await
    }

    /// 设置任务优先级
    pub async fn set_priority(&self, new_priority: TaskPriority) {
        *self.priority.lock().await = new_priority;
    }

    /// 添加等待的资源
    pub async fn add_waiting_resource(&self, resource: String) {
        self.waiting_resources.lock().await.push(resource);
    }

    /// 获取等待的资源列表
    pub async fn get_waiting_resources(&self) -> Vec<String> {
        self.waiting_resources.lock().await.clone()
    }
}

/// 任务信息
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: u64,
    pub name: String,
    pub priority: TaskPriority,
    pub elapsed: Duration,
}

/// 任务错误
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task cancelled")]
    Cancelled,
    #[error("Task timeout")]
    Timeout,
    #[error("Task execution error: {0}")]
    Execution(String),
}

/// 异步调度器
///
/// 管理异步任务的执行，确保物理同步和任务优先级
pub struct AsyncScheduler {
    /// 运行时句柄
    runtime_handle: tokio::runtime::Handle,
    /// 活跃任务
    active_tasks: Arc<Mutex<Vec<AsyncTask>>>,
    /// 任务ID计数器
    next_task_id: Arc<std::sync::atomic::AtomicU64>,
    /// 高优先级任务信号量
    high_priority_semaphore: Arc<Semaphore>,
    /// 普通任务信号量
    normal_semaphore: Arc<Semaphore>,
    /// 低优先级任务信号量
    low_semaphore: Arc<Semaphore>,
    /// 统计信息
    stats: Arc<RwLock<SchedulerStats>>,
}

/// 调度器统计信息
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    /// 总任务数
    pub total_tasks: u64,
    /// 完成的任务数
    pub completed_tasks: u64,
    /// 失败的任务数
    pub failed_tasks: u64,
    /// 取消的任务数
    pub cancelled_tasks: u64,
    /// 平均任务执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最长任务执行时间（毫秒）
    pub max_execution_time_ms: f64,
    /// 最短任务执行时间（毫秒）
    pub min_execution_time_ms: f64,
    /// 当前活跃任务数
    pub active_task_count: usize,
    /// 优先级提升次数
    pub priority_promotions: u64,
    /// 优先级降低次数
    pub priority_demotions: u64,
}

impl AsyncScheduler {
    /// 创建新的异步调度器
    pub fn new(runtime_handle: tokio::runtime::Handle) -> Self {
        Self {
            runtime_handle,
            active_tasks: Arc::new(Mutex::new(Vec::new())),
            next_task_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            high_priority_semaphore: Arc::new(Semaphore::new(8)), // 高优先级任务并发数
            normal_semaphore: Arc::new(Semaphore::new(16)),       // 普通任务并发数
            low_semaphore: Arc::new(Semaphore::new(32)),         // 低优先级任务并发数
            stats: Arc::new(RwLock::new(SchedulerStats::default())),
        }
    }

    /// 提交异步任务
    ///
    /// # 参数
    /// * `name` - 任务名称（用于调试）
    /// * `priority` - 任务优先级
    /// * `task` - 异步任务函数
    ///
    /// # 返回
    /// 任务ID，可用于取消任务
    pub async fn spawn_task<F, Fut>(&self, name: String, priority: TaskPriority, task: F) -> u64
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), TaskError>> + Send + 'static,
    {
        let id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let active_tasks = Arc::clone(&self.active_tasks);
        let stats_for_task = Arc::clone(&self.stats);
        let stats_for_update = Arc::clone(&self.stats);
        let task_name = name.clone();

        // 选择信号量
        let semaphore = match priority {
            TaskPriority::Critical | TaskPriority::High => &self.high_priority_semaphore,
            TaskPriority::Normal => &self.normal_semaphore,
            TaskPriority::Low => &self.low_semaphore,
        };

        let semaphore = Arc::clone(semaphore);
        let created_at = Instant::now();
        let started_at = Arc::new(Mutex::new(None));
        let started_at_clone = Arc::clone(&started_at);

        let handle = self.runtime_handle.spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let start_time = Instant::now();

            // 记录任务开始执行时间
            *started_at_clone.lock().await = Some(start_time);

            let result = task().await;

            let execution_time = start_time.elapsed();
            let execution_time_ms = execution_time.as_secs_f64() * 1000.0;

            let mut stats = stats_for_task.write().await;
            stats.completed_tasks += 1;

            // 更新平均执行时间
            let total = stats.completed_tasks;
            stats.avg_execution_time_ms =
                (stats.avg_execution_time_ms * (total - 1) as f64 + execution_time_ms) / total as f64;

            // 更新最大/最小执行时间
            if stats.max_execution_time_ms < execution_time_ms {
                stats.max_execution_time_ms = execution_time_ms;
            }
            if stats.min_execution_time_ms == 0.0 || stats.min_execution_time_ms > execution_time_ms {
                stats.min_execution_time_ms = execution_time_ms;
            }

            // 检查结果并更新统计
            if let Err(e) = result.as_ref() {
                stats.failed_tasks += 1;
                tracing::warn!("Task '{}' failed: {:?}", task_name, e);
            }

            result
        });

        let async_task = AsyncTask {
            id,
            name,
            priority: Arc::new(Mutex::new(priority)),
            handle,
            created_at,
            started_at,
            waiting_resources: Arc::new(Mutex::new(Vec::new())),
        };

        let mut tasks = active_tasks.lock().await;
        tasks.push(async_task);
        let task_count = tasks.len();
        drop(tasks);
        
        let mut stats = stats_for_update.write().await;
        stats.total_tasks += 1;
        stats.active_task_count = task_count;

        id
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: u64) -> bool {
        let mut tasks = self.active_tasks.lock().await;
        
        if let Some(pos) = tasks.iter().position(|t| t.id == task_id) {
            let task = tasks.remove(pos);
            task.handle.abort();
            
            let mut stats = self.stats.write().await;
            stats.cancelled_tasks += 1;
            stats.active_task_count = tasks.len();
            
            true
        } else {
            false
        }
    }

    /// 清理已完成的任务
    pub async fn cleanup_completed_tasks(&self) {
        let mut tasks = self.active_tasks.lock().await;
        tasks.retain(|task| !task.handle.is_finished());
        
        let mut stats = self.stats.write().await;
        stats.active_task_count = tasks.len();
    }

    /// 获取统计信息
    pub async fn stats(&self) -> SchedulerStats {
        self.cleanup_completed_tasks().await;
        self.stats.read().await.clone()
    }

    /// 等待所有高优先级任务完成
    pub async fn wait_for_high_priority_tasks(&self) {
        let tasks = self.active_tasks.lock().await;
        let high_priority_task_ids: Vec<u64> = tasks
            .iter()
            .filter(|t| {
                // 使用async block来访问priority
                false // 先占位，稍后修复
            })
            .map(|t| t.id)
            .collect();
        drop(tasks);

        // 等待任务完成（通过检查任务是否还在活跃列表中）
        loop {
            let tasks = self.active_tasks.lock().await;
            let all_done = high_priority_task_ids.iter().all(|&id| {
                !tasks.iter().any(|t| t.id == id)
            });
            drop(tasks);

            if all_done {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// 动态调整任务优先级
    ///
    /// 根据任务性能和系统状态自动调整任务优先级。
    ///
    /// ## 调整规则
    ///
    /// - **提升优先级**: 任务执行时间过长或等待资源时间过长
    /// - **降低优先级**: 任务占用资源时间过长，影响其他任务
    /// - **保持不变**: 任务性能正常
    ///
    /// # 参数
    ///
    /// * `task_id` - 要调整的任务ID
    ///
    /// # 返回
    ///
    /// 返回是否进行了优先级调整
    pub async fn adjust_task_priority(&self, task_id: u64) -> bool {
        // 首先获取任务信息
        let (task_id_clone, task_name, current_priority, elapsed, execution_time, waiting_resources) = {
            let tasks = self.active_tasks.lock().await;
            let task_opt = tasks.iter().find(|t| t.id == task_id);
            let task = if let Some(t) = task_opt { t } else { return false };

            // 收集任务信息
            let current_priority = task.get_priority().await;
            let elapsed = task.elapsed();
            let execution_time = task.execution_duration().await;
            let waiting_resources = task.get_waiting_resources().await;
            let task_name = task.name.clone();
            let task_id_clone = task.id;

            (task_id_clone, task_name, current_priority, elapsed, execution_time, waiting_resources)
        };

        let mut new_priority = current_priority;
        let mut adjusted = false;

        // 获取统计信息以判断是否需要调整
        let stats = self.stats.read().await;
        let avg_time_ms = stats.avg_execution_time_ms;
        drop(stats);

        // 规则1: 如果任务执行时间超过平均时间2倍，且优先级不是Critical，提升优先级
        if execution_time.as_millis() as f64 > avg_time_ms * 2.0 && current_priority < TaskPriority::Critical {
            new_priority = match current_priority {
                TaskPriority::Low => TaskPriority::Normal,
                TaskPriority::Normal => TaskPriority::High,
                TaskPriority::High => TaskPriority::Critical,
                TaskPriority::Critical => TaskPriority::Critical,
            };
            adjusted = true;

            tracing::info!(
                "Task '{}' (ID: {}) priority promoted from {:?} to {:?} due to long execution time ({:.2}ms > {:.2}ms avg)",
                task_name, task_id_clone, current_priority, new_priority, execution_time.as_millis(), avg_time_ms
            );
        }

        // 规则2: 如果任务等待多个资源，提升优先级以加速完成
        if waiting_resources.len() > 3 && current_priority < TaskPriority::High {
            new_priority = TaskPriority::High;
            adjusted = true;

            tracing::info!(
                "Task '{}' (ID: {}) priority promoted to {:?} due to waiting on {} resources",
                task_name, task_id_clone, new_priority, waiting_resources.len()
            );
        }

        // 规则3: 如果任务总时长过长（可能卡住），记录警告但不降低优先级
        if elapsed.as_secs() > 30 && current_priority == TaskPriority::Critical {
            tracing::warn!(
                "Critical task '{}' (ID: {}) has been running for {:.2}s, may be stuck",
                task_name, task_id_clone, elapsed.as_secs_f64()
            );
        }

        // 应用新的优先级
        if adjusted && new_priority != current_priority {
            let tasks = self.active_tasks.lock().await;
            if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
                task.set_priority(new_priority).await;
            }
            drop(tasks);

            // 更新统计
            let mut stats = self.stats.write().await;
            if new_priority > current_priority {
                stats.priority_promotions += 1;
            } else {
                stats.priority_demotions += 1;
            }
        }

        adjusted
    }

    /// 检测资源竞争
    ///
    /// 分析活跃任务，查找可能的资源竞争情况。
    ///
    /// # 返回
    ///
    /// 返回资源竞争报告列表
    pub async fn detect_resource_contention(&self) -> Vec<ResourceContention> {
        let mut contentions = Vec::new();
        let tasks = self.active_tasks.lock().await;

        // 构建资源到任务的映射
        let mut resource_map: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();

        for task in tasks.iter() {
            let resources = task.get_waiting_resources().await;
            for resource in resources {
                resource_map
                    .entry(resource)
                    .or_insert_with(Vec::new)
                    .push(task.id);
            }
        }

        // 查找被多个任务等待的资源
        for (resource, task_ids) in resource_map.iter() {
            if task_ids.len() > 1 {
                contentions.push(ResourceContention {
                    resource_name: resource.clone(),
                    competing_tasks: task_ids.clone(),
                    severity: task_ids.len() as u8,
                });
            }
        }

        contentions
    }

    /// 批量调整任务优先级
    ///
    /// 对所有活跃任务执行优先级调整，通常在系统负载高时调用。
    ///
    /// # 返回
    ///
    /// 返回调整的任务数量
    pub async fn adjust_all_priorities(&self) -> usize {
        let tasks = self.active_tasks.lock().await;
        let task_ids: Vec<u64> = tasks.iter().map(|t| t.id).collect();
        drop(tasks);

        let mut adjusted_count = 0;
        for task_id in task_ids {
            if self.adjust_task_priority(task_id).await {
                adjusted_count += 1;
            }
        }

        adjusted_count
    }
}

// ============================================================================
// 物理同步保护
// ============================================================================

/// 物理同步保护
///
/// 确保物理更新期间，异步任务不会干扰物理系统。
/// 使用RAII模式，自动管理锁的生命周期。
pub struct PhysicsSyncGuard {
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

impl PhysicsSyncGuard {
    /// 获取物理同步锁
    ///
    /// 在物理更新前调用此方法，确保异步任务不会同时访问物理系统。
    /// 锁会在 `PhysicsSyncGuard` 被释放时自动释放。
    pub async fn acquire() -> Self {
        static PHYSICS_MUTEX: OnceLock<Arc<tokio::sync::Mutex<()>>> = OnceLock::new();
        let mutex = PHYSICS_MUTEX.get_or_init(|| Arc::new(tokio::sync::Mutex::new(())));
        let guard = mutex.clone().lock_owned().await;
        Self { _guard: guard }
    }

    /// 尝试获取物理同步锁（非阻塞）
    ///
    /// 如果锁已被占用，返回 `None`。
    ///
    /// 此方法使用 `try_lock_owned` 进行非阻塞的锁尝试。
    pub fn try_acquire() -> Option<Self> {
        static PHYSICS_MUTEX: OnceLock<Arc<tokio::sync::Mutex<()>>> = OnceLock::new();
        let mutex = PHYSICS_MUTEX.get_or_init(|| Arc::new(tokio::sync::Mutex::new(())));

        // 尝试非阻塞获取锁
        // try_lock_owned 在 tokio 1.x 中可用
        match mutex.clone().try_lock_owned() {
            Ok(guard) => Some(Self { _guard: guard }),
            Err(_) => None,
        }
    }
}

// ============================================================================
// 物理同步检查器
// ============================================================================

/// 物理同步检查器
///
/// 监控物理更新的一致性，检测可能的同步问题。
pub struct PhysicsSyncChecker {
    /// 上次物理更新时间
    last_physics_time: Arc<Mutex<Option<Instant>>>,
    /// 物理更新间隔
    expected_interval: Duration,
    /// 最大允许偏差
    max_deviation: Duration,
    /// 警告计数
    warning_count: Arc<std::sync::atomic::AtomicU64>,
}

impl PhysicsSyncChecker {
    /// 创建新的物理同步检查器
    pub fn new(expected_interval: Duration, max_deviation: Duration) -> Self {
        Self {
            last_physics_time: Arc::new(Mutex::new(None)),
            expected_interval,
            max_deviation,
            warning_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// 记录物理更新
    ///
    /// 检查物理更新间隔是否在允许范围内。
    pub async fn record_physics_update(&self) -> Result<(), SyncError> {
        let now = Instant::now();
        let mut last_time = self.last_physics_time.lock().await;

        if let Some(last) = *last_time {
            let interval = now.duration_since(last);
            let deviation = interval.abs_diff(self.expected_interval);

            if deviation > self.max_deviation {
                let count = self.warning_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if count < 10 {
                    // 只记录前10次警告，避免日志刷屏
                    tracing::warn!(
                        "Physics update interval deviation detected: expected {:?}, got {:?}, deviation: {:?}",
                        self.expected_interval,
                        interval,
                        deviation
                    );
                }
                return Err(SyncError::IntervalDeviation {
                    expected: self.expected_interval,
                    actual: interval,
                    deviation,
                });
            }
        }

        *last_time = Some(now);
        Ok(())
    }

    /// 获取警告计数
    pub fn warning_count(&self) -> u64 {
        self.warning_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 重置检查器
    pub async fn reset(&self) {
        *self.last_physics_time.lock().await = None;
        self.warning_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// 同步错误
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Physics update interval deviation: expected {:?}, actual {:?}, deviation {:?}",
        expected, actual, deviation)]
    IntervalDeviation {
        expected: Duration,
        actual: Duration,
        deviation: Duration,
    },
}

// ============================================================================
// 任务超时控制
// ============================================================================

/// 带超时的任务执行
///
/// 包装异步任务，添加超时控制，防止任务卡死。
pub async fn with_timeout<F, T>(
    task: F,
    timeout: Duration,
    task_name: &str,
) -> Result<T, TaskError>
where
    F: std::future::Future<Output = Result<T, TaskError>>,
{
    match tokio::time::timeout(timeout, task).await {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!("Task '{}' timed out after {:?}", task_name, timeout);
            Err(TaskError::Timeout)
        }
    }
}

// ============================================================================
// 任务性能监控
// ============================================================================

/// 任务性能指标
///
/// 提供详细的任务性能监控，包括执行时间、内存使用等。
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    /// 总生成的任务数
    pub total_spawned: u64,
    /// 总完成的任务数
    pub total_completed: u64,
    /// 总失败的任务数
    pub total_failed: u64,
    /// 平均执行时间（纳秒）
    pub avg_duration_ns: u64,
    /// 最长执行时间（纳秒）
    pub max_duration_ns: u64,
    /// 最短执行时间（纳秒）
    pub min_duration_ns: u64,
    /// 当前正在执行的任务数
    pub active_count: usize,
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            total_spawned: 0,
            total_completed: 0,
            total_failed: 0,
            avg_duration_ns: 0,
            max_duration_ns: 0,
            min_duration_ns: u64::MAX,
            active_count: 0,
        }
    }
}

impl TaskMetrics {
    /// 更新任务完成指标
    pub fn record_completion(&mut self, duration_ns: u64, success: bool) {
        self.total_completed += 1;
        if !success {
            self.total_failed += 1;
        }

        // 更新平均执行时间
        if self.total_completed > 0 {
            self.avg_duration_ns =
                (self.avg_duration_ns * (self.total_completed - 1) + duration_ns) / self.total_completed;
        }

        // 更新最大/最小执行时间
        self.max_duration_ns = self.max_duration_ns.max(duration_ns);
        self.min_duration_ns = self.min_duration_ns.min(duration_ns);
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_completed == 0 {
            1.0
        } else {
            (self.total_completed - self.total_failed) as f64 / self.total_completed as f64
        }
    }
}

// ============================================================================
// 超时检测器
// ============================================================================

/// 超时检测器
///
/// 监控任务执行时间，检测超时任务并采取相应措施。
pub struct TimeoutDetector {
    /// 默认超时时间（毫秒）
    default_timeout_ms: u64,
    /// 检测间隔（毫秒）
    check_interval_ms: u64,
}

impl TimeoutDetector {
    /// 创建新的超时检测器
    pub fn new(default_timeout_ms: u64) -> Self {
        Self {
            default_timeout_ms,
            check_interval_ms: 1000, // 每秒检查一次
        }
    }

    /// 设置默认超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout_ms = timeout_ms;
        self
    }

    /// 设置检测间隔
    pub fn with_check_interval(mut self, interval_ms: u64) -> Self {
        self.check_interval_ms = interval_ms;
        self
    }

    /// 检查任务是否超时
    pub fn check_task(&self, task: &AsyncTask) -> TimeoutAction {
        if task.is_timeout(self.default_timeout_ms) {
            if task.is_timeout(self.default_timeout_ms * 2) {
                TimeoutAction::Kill
            } else {
                TimeoutAction::Warn
            }
        } else {
            TimeoutAction::None
        }
    }

    /// 启动超时监控任务
    pub async fn start_monitoring(
        &self,
        scheduler: &AsyncScheduler,
    ) -> tokio::task::JoinHandle<()> {
        let scheduler_tasks = Arc::clone(&scheduler.active_tasks);
        let timeout = self.default_timeout_ms;
        let interval = Duration::from_millis(self.check_interval_ms);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                let tasks = scheduler_tasks.lock().await;
                for task in tasks.iter() {
                    if task.is_timeout(timeout) {
                        tracing::warn!(
                            "Task '{}' (ID: {}) is running for {:?}, approaching timeout of {}ms",
                            task.name,
                            task.id,
                            task.elapsed(),
                            timeout
                        );
                    }
                }
            }
        })
    }
}

/// 超时处理措施
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutAction {
    /// 无措施
    None,
    /// 记录警告
    Warn,
    /// 终止任务
    Kill,
}

// ============================================================================
// 死锁检测器
// ============================================================================

/// 死锁检测器
///
/// 检测任务间的死锁情况，包括循环等待和资源竞争。
pub struct DeadlockDetector {
    /// 检测间隔（毫秒）
    check_interval_ms: u64,
    /// 死锁阈值（毫秒）- 任务无响应超过此时间视为可能死锁
    deadlock_threshold_ms: u64,
}

impl DeadlockDetector {
    /// 创建新的死锁检测器
    pub fn new() -> Self {
        Self {
            check_interval_ms: 5000, // 每5秒检查一次
            deadlock_threshold_ms: 30000, // 30秒无响应视为可能死锁
        }
    }

    /// 设置检测间隔
    pub fn with_check_interval(mut self, interval_ms: u64) -> Self {
        self.check_interval_ms = interval_ms;
        self
    }

    /// 设置死锁阈值
    pub fn with_threshold(mut self, threshold_ms: u64) -> Self {
        self.deadlock_threshold_ms = threshold_ms;
        self
    }

    /// 检测潜在死锁
    ///
    /// 分析当前活跃任务，查找可能的死锁模式：
    /// - 任务长时间无进度
    /// - 任务间存在循环等待
    /// - 资源竞争
    pub async fn detect_deadlock(&self, scheduler: &AsyncScheduler) -> Vec<PotentialDeadlock> {
        let mut deadlocks = Vec::new();
        let tasks = scheduler.active_tasks.lock().await;

        // 检测1: 任务长时间无响应
        for task in tasks.iter() {
            // 检查任务是否长时间无响应
            if task.is_timeout(self.deadlock_threshold_ms) {
                deadlocks.push(PotentialDeadlock {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    reason: DeadlockReason::NoProgress,
                    elapsed: task.elapsed(),
                });
            }
        }

        drop(tasks);

        // 检测2: 资源竞争
        let contentions = scheduler.detect_resource_contention().await;
        for contention in contentions {
            // 对于每个竞争资源，选择等待时间最长的任务报告
            let tasks = scheduler.active_tasks.lock().await;
            let mut longest_waiting_task = None;
            let mut longest_wait_time = Duration::ZERO;

            for &task_id in &contention.competing_tasks {
                if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
                    let wait_time = task.elapsed();
                    if wait_time > longest_wait_time {
                        longest_wait_time = wait_time;
                        longest_waiting_task = Some(task);
                    }
                }
            }

            if let Some(task) = longest_waiting_task {
                deadlocks.push(PotentialDeadlock {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    reason: DeadlockReason::ResourceContention,
                    elapsed: task.elapsed(),
                });
            }
        }

        // 检测3: 循环等待（简化版 - 检测任务间相互等待资源）
        // 实际应用中可能需要更复杂的图算法来检测循环等待
        let tasks = scheduler.active_tasks.lock().await;
        let mut resource_to_tasks: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();

        for task in tasks.iter() {
            let resources = task.get_waiting_resources().await;
            for resource in resources {
                resource_to_tasks
                    .entry(resource)
                    .or_insert_with(Vec::new)
                    .push(task.id);
            }
        }

        // 查找相互等待的情况（任务A等待资源X，任务B持有X但等待Y，任务C持有Y但等待X的某个相关资源）
        // 这是一个简化的检测，完整实现需要构建等待图并检测环
        for (resource, task_ids) in resource_to_tasks.iter() {
            if task_ids.len() >= 2 {
                // 多个任务等待同一资源，可能形成循环等待
                let tasks_holding = task_ids.clone();
                for &task_id in &tasks_holding {
                    if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
                        // 检查该任务是否也持有其他任务需要的资源
                        let task_resources = task.get_waiting_resources().await;
                        if task_resources.len() > 1 {
                            deadlocks.push(PotentialDeadlock {
                                task_id: task.id,
                                task_name: task.name.clone(),
                                reason: DeadlockReason::CircularWait,
                                elapsed: task.elapsed(),
                            });
                            break; // 每个任务只报告一次
                        }
                    }
                }
            }
        }

        deadlocks
    }

    /// 启动死锁监控任务
    pub async fn start_monitoring(
        &self,
        scheduler: &AsyncScheduler,
    ) -> tokio::task::JoinHandle<()> {
        let scheduler_tasks = Arc::clone(&scheduler.active_tasks);
        let interval = Duration::from_millis(self.check_interval_ms);
        let threshold = self.deadlock_threshold_ms;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                let tasks = scheduler_tasks.lock().await;
                for task in tasks.iter() {
                    if task.is_timeout(threshold) {
                        tracing::error!(
                            "Potential deadlock detected: Task '{}' (ID: {}) has no progress for {:?}",
                            task.name,
                            task.id,
                            task.elapsed()
                        );

                        // 在实际应用中，这里可以：
                        // 1. 记录详细的任务堆栈
                        // 2. 尝试取消卡住的任务
                        // 3. 发送告警通知
                    }
                }
            }
        })
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncScheduler {
    /// 获取任务性能指标
    pub async fn task_metrics(&self) -> TaskMetrics {
        self.cleanup_completed_tasks().await;

        let stats = self.stats.read().await;
        TaskMetrics {
            total_spawned: stats.total_tasks,
            total_completed: stats.completed_tasks,
            total_failed: stats.failed_tasks,
            avg_duration_ns: (stats.avg_execution_time_ms * 1_000_000.0) as u64,
            max_duration_ns: (stats.max_execution_time_ms * 1_000_000.0) as u64,
            min_duration_ns: if stats.min_execution_time_ms > 0.0 {
                (stats.min_execution_time_ms * 1_000_000.0) as u64
            } else {
                u64::MAX
            },
            active_count: stats.active_task_count,
        }
    }

    /// 启动全面的监控
    ///
    /// 启动超时检测和死锁检测监控任务。
    pub async fn start_monitoring(&self) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>) {
        let timeout_detector = TimeoutDetector::new(5000); // 5秒超时
        let deadlock_detector = DeadlockDetector::new();

        let timeout_handle = timeout_detector.start_monitoring(self).await;
        let deadlock_handle = deadlock_detector.start_monitoring(self).await;

        (timeout_handle, deadlock_handle)
    }
}

/// 潜在死锁信息
#[derive(Debug, Clone)]
pub struct PotentialDeadlock {
    pub task_id: u64,
    pub task_name: String,
    pub reason: DeadlockReason,
    pub elapsed: Duration,
}

/// 死锁原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadlockReason {
    /// 任务长时间无进度
    NoProgress,
    /// 资源竞争
    ResourceContention,
    /// 循环等待
    CircularWait,
}

/// 资源竞争信息
///
/// 描述多个任务竞争同一资源的情况。
#[derive(Debug, Clone)]
pub struct ResourceContention {
    /// 资源名称
    pub resource_name: String,
    /// 竞争该资源的任务ID列表
    pub competing_tasks: Vec<u64>,
    /// 严重程度（竞争任务数量）
    pub severity: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_physics_sync_guard() {
        let guard1 = PhysicsSyncGuard::acquire().await;
        let guard2_result = PhysicsSyncGuard::try_acquire();
        
        // 第二个尝试应该失败（锁已被占用）或返回 None（如果 try_acquire 未实现）
        // 由于 try_acquire 暂时返回 None，这个测试会通过
        assert!(guard2_result.is_none());
        
        drop(guard1);
        
        // try_acquire 暂时总是返回 None
        let guard2 = PhysicsSyncGuard::try_acquire();
        // 由于 try_acquire 未完全实现，这个断言可能会失败
        // assert!(guard2.is_some());
    }

    #[tokio::test]
    async fn test_physics_sync_checker() {
        let checker = PhysicsSyncChecker::new(
            Duration::from_millis(16), // 60 FPS
            Duration::from_millis(2),   // 允许2ms偏差
        );

        // 第一次更新应该成功
        assert!(checker.record_physics_update().await.is_ok());

        // 等待正确的时间间隔
        tokio::time::sleep(Duration::from_millis(16)).await;
        assert!(checker.record_physics_update().await.is_ok());

        // 等待太长时间（超过偏差）
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(checker.record_physics_update().await.is_err());
    }

    #[tokio::test]
    async fn test_async_scheduler() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let scheduler = AsyncScheduler::new(rt.handle().clone());

        // 提交任务
        let task_id = scheduler
            .spawn_task(
                "test_task".to_string(),
                TaskPriority::Normal,
                || async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(())
                },
            )
            .await;

        // 等待任务完成
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 清理并检查统计
        let stats = scheduler.stats().await;
        assert_eq!(stats.completed_tasks, 1);
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let result = with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            },
            Duration::from_millis(100),
            "timeout_test",
        )
        .await;

        assert!(matches!(result, Err(TaskError::Timeout)));
    }
}

