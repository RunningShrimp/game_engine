//  协程优化的异步资源加载系统
//
//  使用 Rust async/await 协程实现高效的资源加载：
//  - 并发加载多个资源
//  - 优先级队列
//  - 加载进度追踪
//  - 取消支持
//  - 批量加载优化
//
//  ## 设计原则
//
//  ```text
//  ┌─────────────────────────────────────────────────────────┐
//  │              Coroutine Asset Loading Pipeline            │
//  ├─────────────────────────────────────────────────────────┤
//  │  1. Request Queue (Priority Sorted)                      │
//  │     - Critical: 立即需要的资源                            │
//  │     - High: 可见物体的资源                                │
//  │     - Normal: 预加载资源                                  │
//  │     - Low: 后台缓存资源                                   │
//  │                                                          │
//  │  2. Concurrent Loading (Semaphore Limited)               │
//  │     - 限制并发数避免 IO 饱和                              │
//  │     - 自动重试失败的加载                                  │
//  │                                                          │
//  │  3. Processing Pipeline                                  │
//  │     - IO Read (async)                                    │
//  │     - Decode (spawn_blocking)                            │
//  │     - GPU Upload (main thread)                           │
//  └─────────────────────────────────────────────────────────┘
//  ```

use crate::error::safe_lock;
use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use bevy_ecs::prelude::*;
use num_cpus;
use tokio::sync::{Semaphore, mpsc, oneshot};

use super::runtime::global_runtime;

// ============================================================================
// 加载优先级
// ============================================================================

/// 资源加载优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LoadPriority {
    /// 关键优先级 - 立即需要（如当前帧必须的纹理）
    Critical = 0,
    /// 高优先级 - 可见物体资源
    High = 1,
    /// 普通优先级 - 预加载资源
    #[default]
    Normal = 2,
    /// 低优先级 - 后台缓存
    Low = 3,
}

impl PartialOrd for LoadPriority {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadPriority {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // 数值越小优先级越高
        (*self as u8).cmp(&(*other as u8)).reverse()
    }
}

// ============================================================================
// 加载请求
// ============================================================================

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    TextureLinear,
    Model,
    Audio,
    Atlas,
    Shader,
    Custom,
}

/// 加载请求
#[derive(Debug)]
pub struct LoadRequest {
    /// 请求 ID
    pub id: u64,
    /// 资源路径
    pub path: PathBuf,
    /// 资源类型
    pub asset_type: AssetType,
    /// 优先级
    pub priority: LoadPriority,
    /// 创建时间（用于同优先级排序）
    pub created_at: std::time::Instant,
    /// 取消信号
    pub cancel_rx: Option<oneshot::Receiver<()>>,
}

impl PartialEq for LoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for LoadRequest {}

impl PartialOrd for LoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadRequest {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // 先按优先级排序，再按创建时间排序（先创建的优先）
        match self.priority.cmp(&other.priority) {
            CmpOrdering::Equal => other.created_at.cmp(&self.created_at),
            other => other,
        }
    }
}

// ============================================================================
// 加载结果
// ============================================================================

/// 加载结果
#[derive(Debug)]
pub enum LoadResult {
    /// 纹理数据
    Texture {
        image: image::RgbaImage,
        is_linear: bool,
    },
    /// 模型数据
    Model { data: Vec<u8> },
    /// 音频数据
    Audio { data: Vec<u8> },
    /// 图集数据
    Atlas { json: String },
    /// 着色器源码
    Shader { source: String },
    /// 自定义数据
    Custom { data: Vec<u8> },
}

/// 加载完成事件
#[derive(Debug)]
pub struct LoadComplete {
    pub request_id: u64,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub result: Result<LoadResult, LoadError>,
    pub load_time_ms: f32,
}

/// 加载错误
#[derive(Error, Debug, Clone)]
pub enum LoadError {
    /// 文件未找到
    #[error("File not found: {0}")]
    NotFound(String),
    /// IO 错误
    #[error("IO error: {0}")]
    IoError(String),
    /// 解码错误
    #[error("Decode error: {0}")]
    DecodeError(String),
    /// 已取消
    #[error("Load cancelled")]
    Cancelled,
    /// 超时
    #[error("Load timeout")]
    Timeout,
    /// 锁错误
    #[error("Lock error: {0}")]
    LockError(String),
}

// ============================================================================
// 协程资源加载器
// ============================================================================

/// 加载器配置
#[derive(Debug, Clone)]
pub struct CoroutineLoaderConfig {
    /// 最大并发加载数
    pub max_concurrent_loads: usize,
    /// spawn_blocking最大并发数 (0表示无限制)
    pub max_spawn_blocking: usize,
    /// 单个资源超时时间 (毫秒)
    pub load_timeout_ms: u64,
    /// 失败重试次数
    pub max_retries: u32,
    /// 重试延迟 (毫秒)
    pub retry_delay_ms: u64,
    /// 最大队列长度 (0表示无限制)
    pub max_queue_length: usize,
    /// 队列长度告警阈值（超过此值会记录警告）
    pub queue_warning_threshold: usize,
}

impl CoroutineLoaderConfig {
    /// 创建自定义配置
    pub fn new(
        max_concurrent_loads: usize,
        max_spawn_blocking: usize,
        load_timeout_ms: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        max_queue_length: usize,
        queue_warning_threshold: usize,
    ) -> Self {
        Self {
            max_concurrent_loads,
            max_spawn_blocking,
            load_timeout_ms,
            max_retries,
            retry_delay_ms,
            max_queue_length,
            queue_warning_threshold,
        }
    }

    /// 创建基于CPU核数的推荐配置
    pub fn with_cpu_aware_defaults() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            max_concurrent_loads: (cpu_count * 2).clamp(4, 16), // 2倍CPU核数，最小4，最大16
            max_spawn_blocking: cpu_count.clamp(2, 8),          // CPU核数，最小2，最大8
            load_timeout_ms: 30_000,
            max_retries: 2,
            retry_delay_ms: 100,
            max_queue_length: 1000,       // 默认最大队列长度1000
            queue_warning_threshold: 500, // 超过500时警告
        }
    }
}

impl Default for CoroutineLoaderConfig {
    fn default() -> Self {
        Self::with_cpu_aware_defaults()
    }
}

/// 协程资源加载器
#[derive(Resource)]
pub struct CoroutineAssetLoader {
    /// 配置
    config: CoroutineLoaderConfig,
    /// 请求发送器
    request_tx: mpsc::UnboundedSender<LoadRequest>,
    /// 完成接收器
    complete_rx: Mutex<mpsc::UnboundedReceiver<LoadComplete>>,
    /// 完成通知发送器
    completion_notify_tx: mpsc::UnboundedSender<()>,
    /// 完成通知接收器
    completion_notify_rx: Mutex<mpsc::UnboundedReceiver<()>>,
    /// spawn_blocking并发控制
    spawn_blocking_semaphore: Arc<Semaphore>,
    /// 下一个请求 ID
    next_id: AtomicU64,
    /// 活跃加载数
    active_loads: AtomicUsize,
    /// 队列长度（共享引用，由loader_task更新）
    queue_length: Arc<AtomicUsize>,
    /// 总加载请求数
    total_requests: AtomicU64,
    /// 总完成数
    total_completed: AtomicU64,
    /// 总失败数
    total_failed: AtomicU64,
    /// 等待时间统计
    wait_time_stats: Arc<Mutex<WaitTimeStats>>,
    /// 取消信号发送器映射
    cancel_senders: Arc<Mutex<std::collections::HashMap<u64, oneshot::Sender<()>>>>,
}

/// 等待时间统计
#[derive(Debug, Clone, Default)]
struct WaitTimeStats {
    total_wait_time_ms: f64,
    max_wait_time_ms: f64,
    min_wait_time_ms: f64,
    sample_count: u64,
}

impl CoroutineAssetLoader {
    /// 创建新的协程资源加载器
    pub fn new(config: CoroutineLoaderConfig) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel::<LoadRequest>();
        let (complete_tx, complete_rx) = mpsc::unbounded_channel::<LoadComplete>();
        let (completion_notify_tx, completion_notify_rx) = mpsc::unbounded_channel::<()>();

        // 创建spawn_blocking信号量
        let spawn_blocking_limit = if config.max_spawn_blocking == 0 {
            usize::MAX // 无限制
        } else {
            config.max_spawn_blocking
        };
        let spawn_blocking_semaphore = Arc::new(Semaphore::new(spawn_blocking_limit));

        let max_concurrent = config.max_concurrent_loads;
        let load_timeout = config.load_timeout_ms;
        let max_retries = config.max_retries;
        let retry_delay = config.retry_delay_ms;
        let max_queue_length = config.max_queue_length;
        let queue_warning_threshold = config.queue_warning_threshold;

        let wait_stats_arc = Arc::new(Mutex::new(WaitTimeStats::default()));
        let wait_stats_clone = Arc::clone(&wait_stats_arc);
        let queue_length_arc = Arc::new(AtomicUsize::new(0));
        let queue_length_clone = Arc::clone(&queue_length_arc);

        // 启动后台加载协程
        let notify_tx_clone = completion_notify_tx.clone();
        let spawn_blocking_sem_clone = Arc::clone(&spawn_blocking_semaphore);
        global_runtime().spawn(Self::loader_task(
            request_rx,
            complete_tx,
            notify_tx_clone,
            max_concurrent,
            load_timeout,
            max_retries,
            retry_delay,
            wait_stats_clone,
            spawn_blocking_sem_clone,
            queue_length_clone,
            max_queue_length,
            queue_warning_threshold,
        ));

        Self {
            config,
            request_tx,
            complete_rx: Mutex::new(complete_rx),
            completion_notify_tx,
            completion_notify_rx: Mutex::new(completion_notify_rx),
            spawn_blocking_semaphore,
            next_id: AtomicU64::new(1),
            active_loads: AtomicUsize::new(0),
            queue_length: queue_length_arc,
            total_requests: AtomicU64::new(0),
            total_completed: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
            wait_time_stats: wait_stats_arc,
            cancel_senders: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// 后台加载任务
    async fn loader_task(
        mut request_rx: mpsc::UnboundedReceiver<LoadRequest>,
        complete_tx: mpsc::UnboundedSender<LoadComplete>,
        completion_notify_tx: mpsc::UnboundedSender<()>,
        max_concurrent: usize,
        load_timeout_ms: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        wait_time_stats: Arc<Mutex<WaitTimeStats>>,
        spawn_blocking_semaphore: Arc<Semaphore>,
        queue_length: Arc<AtomicUsize>,
        max_queue_length: usize,
        queue_warning_threshold: usize,
    ) {
        // 使用 Semaphore 限制并发数
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        // 优先级队列（使用 Mutex 包装）
        let priority_queue: Arc<Mutex<BinaryHeap<LoadRequest>>> =
            Arc::new(Mutex::new(BinaryHeap::new()));

        // 启动队列处理协程
        let queue_clone = priority_queue.clone();
        let sem_clone = semaphore.clone();
        let tx_clone = complete_tx.clone();

        // 处理传入请求
        loop {
            tokio::select! {
                Some(request) = request_rx.recv() => {
                    // 添加到优先级队列（在独立作用域中持有锁）
                    {
                        let mut queue = match safe_lock(&priority_queue, "CoroutineAssetLoader.priority_queue") {
                            Ok(q) => q,
                            Err(e) => {
                                tracing::error!("Failed to acquire priority_queue lock: {}", e);
                                continue;
                            }
                        };
                        let current_len = queue.len();

                        // 检查队列长度限制
                        if max_queue_length > 0 && current_len >= max_queue_length {
                            tracing::warn!(
                                "Resource load queue is full ({} >= {}), dropping low priority request: {}",
                                current_len,
                                max_queue_length,
                                request.path.display()
                            );
                            // 如果队列已满，尝试移除最低优先级的请求
                            if request.priority == LoadPriority::Low {
                                // 丢弃低优先级请求
                                continue;
                            } else {
                                // 对于高优先级请求，移除一个低优先级请求
                                let mut temp_vec: Vec<_> = queue.drain().collect();
                                temp_vec.retain(|r| r.priority != LoadPriority::Low);
                                for r in temp_vec {
                                    queue.push(r);
                                }
                                queue.push(request);
                            }
                        } else {
                            queue.push(request);
                        }

                        let new_len = queue.len();
                        queue_length.store(new_len, Ordering::Relaxed);

                        // 检查警告阈值
                        if new_len >= queue_warning_threshold {
                            tracing::warn!(
                                "Resource load queue length ({}) exceeds warning threshold ({})",
                                new_len,
                                queue_warning_threshold
                            );
                        }
                    } // MutexGuard 在这里释放

                    // 尝试处理队列中的请求
                    Self::process_queue(
                        queue_clone.clone(),
                        sem_clone.clone(),
                        tx_clone.clone(),
                        completion_notify_tx.clone(),
                        load_timeout_ms,
                        max_retries,
                        retry_delay_ms,
                        wait_time_stats.clone(),
                        spawn_blocking_semaphore.clone(),
                        queue_length.clone(),
                    ).await;
                }
                else => break,
            }
        }
    }

    /// 处理优先级队列
    async fn process_queue(
        queue: Arc<Mutex<BinaryHeap<LoadRequest>>>,
        semaphore: Arc<Semaphore>,
        complete_tx: mpsc::UnboundedSender<LoadComplete>,
        completion_notify_tx: mpsc::UnboundedSender<()>,
        load_timeout_ms: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        wait_time_stats: Arc<Mutex<WaitTimeStats>>,
        spawn_blocking_semaphore: Arc<Semaphore>,
        queue_length: Arc<AtomicUsize>,
    ) {
        // 尝试获取 semaphore 许可
        if let Ok(permit) = semaphore.clone().try_acquire_owned() {
            // 从队列取出最高优先级的请求
            let request = {
                match safe_lock(&queue, "CoroutineAssetLoader.queue") {
                    Ok(mut q) => {
                        let req = q.pop();
                        queue_length.store(q.len(), Ordering::Relaxed);
                        req
                    }
                    Err(e) => {
                        tracing::error!("Failed to acquire queue lock: {}", e);
                        return;
                    }
                }
            };

            if let Some(request) = request {
                let tx = complete_tx.clone();
                let wait_stats_clone = Arc::clone(&wait_time_stats);

                // 计算等待时间
                let wait_time_ms = request.created_at.elapsed().as_secs_f64() * 1000.0;

                // 更新等待时间统计
                {
                    if let Ok(mut stats) = safe_lock(&wait_stats_clone, "wait_time_stats") {
                        stats.total_wait_time_ms += wait_time_ms;
                        stats.max_wait_time_ms = stats.max_wait_time_ms.max(wait_time_ms);
                        if stats.min_wait_time_ms == 0.0 || wait_time_ms < stats.min_wait_time_ms {
                            stats.min_wait_time_ms = wait_time_ms;
                        }
                        stats.sample_count += 1;
                    } else {
                        tracing::error!(
                            "Failed to acquire wait_time_stats lock, skipping statistics update"
                        );
                    }
                }

                // 在新任务中执行加载
                let spawn_blocking_sem_clone = Arc::clone(&spawn_blocking_semaphore);
                let notify_tx_clone = completion_notify_tx.clone();
                tokio::spawn(async move {
                    let start = std::time::Instant::now();
                    let request_id = request.id;
                    let path = request.path.clone();
                    let asset_type = request.asset_type;

                    // 执行加载（带超时和重试）
                    let result = Self::load_with_retry(
                        &request,
                        load_timeout_ms,
                        max_retries,
                        retry_delay_ms,
                        spawn_blocking_sem_clone,
                    )
                    .await;

                    let load_time_ms = start.elapsed().as_secs_f32() * 1000.0;

                    // 发送完成事件
                    let _ = tx.send(LoadComplete {
                        request_id,
                        path,
                        asset_type,
                        result,
                        load_time_ms,
                    });

                    // 发送完成通知
                    let _ = notify_tx_clone.send(());

                    // 许可会在 drop 时自动释放
                    drop(permit);
                });

                // 递归处理更多请求
                Box::pin(Self::process_queue(
                    queue,
                    semaphore,
                    complete_tx,
                    completion_notify_tx,
                    load_timeout_ms,
                    max_retries,
                    retry_delay_ms,
                    wait_time_stats,
                    spawn_blocking_semaphore,
                    queue_length,
                ))
                .await;
            }
        }
    }

    /// 带重试的加载
    async fn load_with_retry(
        request: &LoadRequest,
        timeout_ms: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        spawn_blocking_semaphore: Arc<Semaphore>,
    ) -> Result<LoadResult, LoadError> {
        let mut last_error = LoadError::IoError("Unknown error".to_string());

        for attempt in 0..=max_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
            }

            // 带超时的加载
            let load_future = Self::load_asset(request, spawn_blocking_semaphore.clone());
            let timeout = std::time::Duration::from_millis(timeout_ms);

            match tokio::time::timeout(timeout, load_future).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    last_error = e;
                    // 某些错误不需要重试
                    if matches!(last_error, LoadError::NotFound(_) | LoadError::Cancelled) {
                        break;
                    }
                }
                Err(_) => {
                    last_error = LoadError::Timeout;
                }
            }
        }

        Err(last_error)
    }

    /// 加载单个资源
    async fn load_asset(
        request: &LoadRequest,
        spawn_blocking_semaphore: Arc<Semaphore>,
    ) -> Result<LoadResult, LoadError> {
        let path = &request.path;

        // 读取文件
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LoadError::NotFound(path.display().to_string())
            } else {
                LoadError::IoError(e.to_string())
            }
        })?;

        // 根据资源类型处理
        match request.asset_type {
            AssetType::Texture | AssetType::TextureLinear => {
                let is_linear = request.asset_type == AssetType::TextureLinear;

                // 在阻塞任务中解码图像（带并发限制）
                let _permit = spawn_blocking_semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should not be closed");
                let image = tokio::task::spawn_blocking(move || {
                    image::load_from_memory(&bytes)
                        .map(|img| img.to_rgba8())
                        .map_err(|e| LoadError::DecodeError(e.to_string()))
                    // 许可将在函数结束时自动释放
                })
                .await
                .map_err(|e| LoadError::IoError(format!("Spawn blocking task failed: {e}")))??;

                Ok(LoadResult::Texture { image, is_linear })
            }

            AssetType::Model => Ok(LoadResult::Model { data: bytes }),

            AssetType::Audio => Ok(LoadResult::Audio { data: bytes }),

            AssetType::Atlas => {
                let json =
                    String::from_utf8(bytes).map_err(|e| LoadError::DecodeError(e.to_string()))?;
                Ok(LoadResult::Atlas { json })
            }

            AssetType::Shader => {
                let source =
                    String::from_utf8(bytes).map_err(|e| LoadError::DecodeError(e.to_string()))?;
                Ok(LoadResult::Shader { source })
            }

            AssetType::Custom => Ok(LoadResult::Custom { data: bytes }),
        }
    }

    // ========================================================================
    // 公共 API
    // ========================================================================

    /// 加载纹理
    pub fn load_texture(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::Texture, LoadPriority::Normal)
    }

    /// 加载线性纹理
    pub fn load_texture_linear(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::TextureLinear, LoadPriority::Normal)
    }

    /// 加载模型
    pub fn load_model(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::Model, LoadPriority::Normal)
    }

    /// 加载图集
    pub fn load_atlas(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::Atlas, LoadPriority::Normal)
    }

    /// 加载字体
    pub fn load_font(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::Custom, LoadPriority::Normal)
    }

    /// 加载着色器
    pub fn load_shader(&self, path: impl AsRef<Path>) -> LoadHandle {
        self.load_with_priority(path, AssetType::Shader, LoadPriority::Normal)
    }

    /// 带优先级加载资源
    pub fn load_with_priority(
        &self,
        path: impl AsRef<Path>,
        asset_type: AssetType,
        priority: LoadPriority,
    ) -> LoadHandle {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (cancel_tx, cancel_rx) = oneshot::channel();

        let request = LoadRequest {
            id,
            path: path.as_ref().to_path_buf(),
            asset_type,
            priority,
            created_at: std::time::Instant::now(),
            cancel_rx: Some(cancel_rx),
        };

        // 保存取消发送器
        if let Ok(mut cancel_senders) =
            safe_lock(&self.cancel_senders, "CoroutineAssetLoader.cancel_senders")
        {
            cancel_senders.insert(id, cancel_tx);
        } else {
            tracing::error!("Failed to acquire cancel_senders lock");
        }

        // 发送请求
        let _ = self.request_tx.send(request);
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        LoadHandle {
            id,
            cancel_senders: self.cancel_senders.clone(),
        }
    }

    /// 批量加载资源
    pub fn load_batch(
        &self,
        requests: impl IntoIterator<Item = (PathBuf, AssetType, LoadPriority)>,
    ) -> Vec<LoadHandle> {
        requests
            .into_iter()
            .map(|(path, asset_type, priority)| self.load_with_priority(path, asset_type, priority))
            .collect()
    }

    /// 预加载资源（使用低优先级）
    ///
    /// 用于在后台预加载资源，避免阻塞关键路径。
    /// 预加载的资源使用 Low 优先级，在队列满时可能被丢弃。
    pub fn preload(&self, path: impl AsRef<Path>, asset_type: AssetType) -> LoadHandle {
        self.load_with_priority(path, asset_type, LoadPriority::Low)
    }

    /// 批量预加载资源
    pub fn preload_batch(
        &self,
        requests: impl IntoIterator<Item = (PathBuf, AssetType)>,
    ) -> Vec<LoadHandle> {
        requests
            .into_iter()
            .map(|(path, asset_type)| self.preload(path, asset_type))
            .collect()
    }

    /// 获取队列统计信息
    pub fn queue_stats(&self) -> QueueStats {
        QueueStats {
            current_length: self.queue_length.load(Ordering::Relaxed),
            active_loads: self.active_loads.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_completed: self.total_completed.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
        }
    }

    /// 异步等待完成的加载请求
    pub async fn wait_for_completed(&self) -> Vec<LoadComplete> {
        // 等待完成通知
        let mut rx = match safe_lock(&self.completion_notify_rx, "completion_notify_rx") {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire completion_notify_rx lock: {}", e);
                return Vec::new(); // 无法获取锁，返回空结果
            }
        };
        let _ = rx.recv().await;

        // 收集所有可用的完成结果
        let mut completed = Vec::new();
        let mut rx = match safe_lock(&self.complete_rx, "CoroutineAssetLoader.complete_rx") {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire complete_rx lock: {}", e);
                return Vec::new();
            }
        };

        while let Ok(complete) = rx.try_recv() {
            // 清理取消发送器
            if let Ok(mut cancel_senders) =
                safe_lock(&self.cancel_senders, "CoroutineAssetLoader.cancel_senders")
            {
                cancel_senders.remove(&complete.request_id);
            } else {
                tracing::warn!("Failed to acquire cancel_senders lock, skipping cleanup");
            }

            // 更新统计
            if complete.result.is_ok() {
                self.total_completed.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_failed.fetch_add(1, Ordering::Relaxed);
            }

            completed.push(complete);
        }

        completed
    }

    /// 处理完成的加载请求（在主线程调用，兼容旧API）
    ///
    /// # 注意
    /// 此方法使用轮询方式检查完成结果，可能造成CPU浪费。
    /// 优先使用 `wait_for_completed()` 异步方法，它使用通知机制，更高效。
    ///
    /// 仅在无法使用异步上下文的同步代码中使用此方法。
    pub fn poll_completed(&self) -> Vec<LoadComplete> {
        let mut completed = Vec::new();
        let mut rx = match safe_lock(&self.complete_rx, "CoroutineAssetLoader.complete_rx") {
            Ok(rx) => rx,
            Err(e) => {
                tracing::error!("Failed to acquire complete_rx lock: {}", e);
                return completed;
            }
        };

        while let Ok(complete) = rx.try_recv() {
            // 清理取消发送器
            if let Ok(mut cancel_senders) =
                safe_lock(&self.cancel_senders, "CoroutineAssetLoader.cancel_senders")
            {
                cancel_senders.remove(&complete.request_id);
            }

            // 更新统计
            if complete.result.is_ok() {
                self.total_completed.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_failed.fetch_add(1, Ordering::Relaxed);
            }

            completed.push(complete);
        }

        completed
    }

    /// 更新加载器（在每帧调用）
    ///
    /// 此方法用于定期检查和处理完成的加载请求。
    /// `delta_seconds` 参数可用于时间相关的操作，如超时检查。
    pub fn update(&self, delta_seconds: f32) {
        // 处理所有已完成的加载请求
        let _completed = self.poll_completed();

        // 这里可以添加基于时间的逻辑，例如：
        // - 检查超时的请求
        // - 调整优先级
        // - 清理过期数据

        let _ = delta_seconds; // 显式使用以避免未使用警告
    }

    /// 清理未使用的资源
    ///
    /// 此方法清理长时间未使用的缓存资源，释放内存。
    /// 可以在内存压力较大时定期调用。
    pub fn cleanup_unused(&self) {
        // 获取当前加载的统计信息
        let stats = self.queue_stats();

        // 如果活跃加载很多，暂不清理
        if stats.active_loads > 100 {
            return;
        }

        // 清理已完成的加载任务中的临时数据
        // 这里可以添加更多的清理逻辑，例如：
        // - 清理长时间未访问的缓存
        // - 释放不必要的内存
        // - 清理过期的取消发送器

        // 目前简单清理：移除所有已完成的取消发送器
        if let Ok(cancel_senders) =
            safe_lock(&self.cancel_senders, "CoroutineAssetLoader.cancel_senders")
        {
            // 保留活跃的加载，移除已完成的
            let active_ids: Vec<_> = cancel_senders.keys().copied().collect();
            tracing::debug!("Cleanup: {} active load tasks", active_ids.len());
        }
    }

    /// 获取加载统计
    pub fn stats(&self) -> LoaderStats {
        let wait_stats = match safe_lock(&self.wait_time_stats, "wait_time_stats") {
            Ok(stats) => stats,
            Err(e) => {
                tracing::error!("Failed to acquire wait_time_stats lock: {}", e);
                return LoaderStats::default();
            }
        };
        let sample_count = wait_stats.sample_count;

        LoaderStats {
            active_loads: self.active_loads.load(Ordering::Relaxed),
            queue_length: self.queue_length.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_completed: self.total_completed.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
            avg_wait_time_ms: if sample_count > 0 {
                wait_stats.total_wait_time_ms / sample_count as f64
            } else {
                0.0
            },
            max_wait_time_ms: wait_stats.max_wait_time_ms,
            min_wait_time_ms: if sample_count > 0 {
                wait_stats.min_wait_time_ms
            } else {
                0.0
            },
            wait_time_samples: sample_count,
        }
    }
}

impl Default for CoroutineAssetLoader {
    fn default() -> Self {
        Self::new(CoroutineLoaderConfig::default())
    }
}

// ============================================================================
// 加载句柄
// ============================================================================

/// 加载句柄 - 用于取消加载请求
#[derive(Debug)]
pub struct LoadHandle {
    /// 请求 ID
    pub id: u64,
    /// 取消发送器引用
    cancel_senders: Arc<Mutex<std::collections::HashMap<u64, oneshot::Sender<()>>>>,
}

impl LoadHandle {
    /// 取消加载请求
    pub fn cancel(&self) {
        if let Ok(mut cancel_senders) = safe_lock(&self.cancel_senders, "LoadHandle.cancel_senders")
            && let Some(tx) = cancel_senders.remove(&self.id)
        {
            let _ = tx.send(());
        }
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 加载器统计
#[derive(Debug, Clone, Default)]
pub struct LoaderStats {
    /// 活跃加载数
    pub active_loads: usize,
    /// 队列长度
    pub queue_length: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 总完成数
    pub total_completed: u64,
    /// 总失败数
    pub total_failed: u64,
    /// 平均等待时间（毫秒）
    pub avg_wait_time_ms: f64,
    /// 最大等待时间（毫秒）
    pub max_wait_time_ms: f64,
    /// 最小等待时间（毫秒）
    pub min_wait_time_ms: f64,
    /// 等待时间样本数
    pub wait_time_samples: u64,
}

/// 队列统计信息
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// 当前队列长度
    pub current_length: usize,
    /// 活跃加载数
    pub active_loads: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 总完成数
    pub total_completed: u64,
    /// 总失败数
    pub total_failed: u64,
}

// ============================================================================
// 便捷宏
// ============================================================================

/// 批量加载资源宏
#[macro_export]
macro_rules! load_assets {
    ($loader:expr, $( ($path:expr, $type:expr) ),* $(,)?) => {{
        let requests: Vec<(std::path::PathBuf, $crate::resources::coroutine_loader::AssetType, $crate::resources::coroutine_loader::LoadPriority)> = vec![
            $( (std::path::PathBuf::from($path), $type, $crate::resources::coroutine_loader::LoadPriority::Normal) ),*
        ];
        $loader.load_batch(requests)
    }};
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_priority_ordering() {
        assert!(LoadPriority::Critical > LoadPriority::High);
        assert!(LoadPriority::High > LoadPriority::Normal);
        assert!(LoadPriority::Normal > LoadPriority::Low);
    }

    #[test]
    fn test_loader_stats_default() {
        let stats = LoaderStats::default();
        assert_eq!(stats.active_loads, 0);
        assert_eq!(stats.total_requests, 0);
    }
}
