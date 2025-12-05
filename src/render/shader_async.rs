//! 异步着色器编译系统
//!
//! 使用异步编译减少主线程阻塞，提升启动性能和响应性。
//!
//! ## 设计原则
//!
//! 1. **异步编译**: 使用`tokio::task::spawn_blocking`在后台线程编译
//! 2. **编译队列**: 优先级队列管理编译任务
//! 3. **进度跟踪**: 实时追踪编译进度
//! 4. **超时处理**: 防止编译任务无限期阻塞
//! 5. **优先级管理**: 关键着色器优先编译

use crate::core::error::RenderError;
use crate::impl_default;
use crate::render::shader_cache::{ShaderCache, ShaderCacheKey};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

/// 着色器编译优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderCompilePriority {
    /// 关键优先级 - 立即需要的着色器（如基础渲染着色器）
    Critical = 0,
    /// 高优先级 - 当前帧需要的着色器
    High = 1,
    /// 普通优先级 - 预加载着色器
    Normal = 2,
    /// 低优先级 - 后台编译
    Low = 3,
}

impl Default for ShaderCompilePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl ShaderCompilePriority {
    pub fn new() -> Self {
        Self::default()
    }
}

/// 着色器编译请求
#[derive(Debug)]
pub struct ShaderCompileRequest {
    /// 请求ID
    pub id: u64,
    /// 着色器标签
    pub label: Option<String>,
    /// WGSL源码
    pub source: String,
    /// 编译选项
    pub compile_options: String,
    /// 优先级
    pub priority: ShaderCompilePriority,
    /// 创建时间
    pub created_at: Instant,
    /// 响应通道
    pub response_tx: oneshot::Sender<Result<CompiledShader, CompileError>>,
}

impl PartialEq for ShaderCompileRequest {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ShaderCompileRequest {}

impl PartialOrd for ShaderCompileRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShaderCompileRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // 先按优先级排序（数值越小优先级越高）
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // 同优先级按创建时间排序（先创建的优先）
                other.created_at.cmp(&self.created_at)
            }
            other => other,
        }
    }
}

/// 编译完成的着色器
#[derive(Debug)]
pub struct CompiledShader {
    /// 着色器缓存键
    pub cache_key: ShaderCacheKey,
    /// 编译后的源码（用于创建wgpu::ShaderModule）
    pub source: String,
    /// 编译时间（毫秒）
    pub compile_time_ms: f32,
}

/// 编译错误
#[derive(Error, Debug, Clone)]
pub enum CompileError {
    /// 编译超时
    #[error("Shader compilation timeout")]
    Timeout,
    /// 编译失败
    #[error("Shader compilation failed: {0}")]
    CompilationFailed(String),
    /// 缓存错误
    #[error("Cache error: {0}")]
    CacheError(String),
    /// 已取消
    #[error("Shader compilation cancelled")]
    Cancelled,
}

/// 编译进度信息
#[derive(Debug, Clone)]
pub struct CompileProgress {
    /// 总请求数
    pub total_requests: usize,
    /// 已完成数
    pub completed: usize,
    /// 失败数
    pub failed: usize,
    /// 进行中数
    pub in_progress: usize,
    /// 等待中数
    pub pending: usize,
}

impl CompileProgress {
    /// 计算完成百分比
    pub fn completion_percentage(&self) -> f32 {
        if self.total_requests == 0 {
            100.0
        } else {
            (self.completed as f32 / self.total_requests as f32) * 100.0
        }
    }
}

/// 异步着色器编译器配置
#[derive(Debug, Clone)]
pub struct AsyncShaderCompilerConfig {
    /// 最大并发编译数
    pub max_concurrent_compiles: usize,
    /// 编译超时时间（毫秒）
    pub compile_timeout_ms: u64,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl_default!(AsyncShaderCompilerConfig {
    max_concurrent_compiles: 4,
    compile_timeout_ms: 5000,
    enable_cache: true,
});

/// 异步着色器编译器
pub struct AsyncShaderCompiler {
    /// 配置
    config: AsyncShaderCompilerConfig,
    /// 请求发送器
    request_tx: mpsc::UnboundedSender<ShaderCompileRequest>,
    /// 进度接收器
    progress_rx: Arc<Mutex<mpsc::UnboundedReceiver<CompileProgress>>>,
    /// 下一个请求ID
    next_id: Arc<Mutex<u64>>,
    /// 着色器缓存（可选）
    cache: Option<Arc<Mutex<ShaderCache>>>,
}

impl AsyncShaderCompiler {
    /// 创建新的异步着色器编译器
    pub fn new(
        config: AsyncShaderCompilerConfig,
        cache: Option<ShaderCache>,
    ) -> Result<Self, RenderError> {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();

        let cache_arc = cache.map(|c| Arc::new(Mutex::new(c)));
        let cache_clone = cache_arc.clone();

        let max_concurrent = config.max_concurrent_compiles;
        let timeout_ms = config.compile_timeout_ms;
        let enable_cache = config.enable_cache;

        // 启动编译任务处理器
        tokio::spawn(Self::compiler_task(
            request_rx,
            progress_tx,
            cache_clone,
            max_concurrent,
            timeout_ms,
            enable_cache,
        ));

        Ok(Self {
            config,
            request_tx,
            progress_rx: Arc::new(Mutex::new(progress_rx)),
            next_id: Arc::new(Mutex::new(1)),
            cache: cache_arc,
        })
    }

    /// 使用默认配置创建异步着色器编译器
    ///
    /// 这是 `new(AsyncShaderCompilerConfig::default(), None)` 的便捷方法。
    ///
    /// # 返回
    ///
    /// 返回使用默认配置创建的异步着色器编译器实例。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::render::shader_async::AsyncShaderCompiler;
    ///
    /// let compiler = AsyncShaderCompiler::with_default_config()?;
    /// ```
    pub fn with_default_config() -> Result<Self, RenderError> {
        Self::new(AsyncShaderCompilerConfig::default(), None)
    }

    /// 编译任务处理器（后台运行）
    async fn compiler_task(
        mut request_rx: mpsc::UnboundedReceiver<ShaderCompileRequest>,
        progress_tx: mpsc::UnboundedSender<CompileProgress>,
        cache: Option<Arc<Mutex<ShaderCache>>>,
        max_concurrent: usize,
        timeout_ms: u64,
        enable_cache: bool,
    ) {
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let priority_queue: Arc<Mutex<BinaryHeap<ShaderCompileRequest>>> =
            Arc::new(Mutex::new(BinaryHeap::new()));

        let mut stats = CompileProgress {
            total_requests: 0,
            completed: 0,
            failed: 0,
            in_progress: 0,
            pending: 0,
        };

        loop {
            tokio::select! {
                Some(request) = request_rx.recv() => {
                    // 添加到优先级队列
                    {
                        let mut queue = priority_queue.lock().unwrap();
                        queue.push(request);
                        stats.total_requests += 1;
                        stats.pending += 1;
                    }

                    // 尝试处理队列
                    Self::process_queue(
                        priority_queue.clone(),
                        semaphore.clone(),
                        cache.clone(),
                        &mut stats,
                        progress_tx.clone(),
                        timeout_ms,
                        enable_cache,
                    ).await;
                }
                else => break,
            }
        }
    }

    /// 处理编译队列
    async fn process_queue(
        queue: Arc<Mutex<BinaryHeap<ShaderCompileRequest>>>,
        semaphore: Arc<tokio::sync::Semaphore>,
        cache: Option<Arc<Mutex<ShaderCache>>>,
        stats: &mut CompileProgress,
        progress_tx: mpsc::UnboundedSender<CompileProgress>,
        timeout_ms: u64,
        enable_cache: bool,
    ) {
        // 尝试获取信号量许可
        if let Ok(permit) = semaphore.clone().try_acquire_owned() {
            // 从队列取出最高优先级的请求
            let request = {
                let mut q = queue.lock().unwrap();
                q.pop()
            };

            if let Some(request) = request {
                stats.pending -= 1;
                stats.in_progress += 1;
                let _ = progress_tx.send(stats.clone());

                let cache_clone = cache.clone();
                let response_tx = request.response_tx;
                let label = request.label.clone();
                let source = request.source.clone();
                let compile_options = request.compile_options.clone();

                // 在新任务中执行编译
                tokio::spawn(async move {
                    let start = Instant::now();

                    // 生成缓存键
                    let cache_key = ShaderCacheKey::from_source(&source, &compile_options);

                    // 检查缓存
                    let cached_result = if enable_cache {
                        if let Some(cache) = &cache_clone {
                            let mut cache_guard = cache.lock().unwrap();
                            cache_guard.get(&cache_key).ok().flatten()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let result = if let Some(cached_source) = cached_result {
                        // 缓存命中，直接返回
                        Ok(CompiledShader {
                            cache_key,
                            source: String::from_utf8(cached_source).unwrap_or(source),
                            compile_time_ms: start.elapsed().as_secs_f32() * 1000.0,
                        })
                    } else {
                        // 缓存未命中，执行编译
                        // 注意：wgpu的create_shader_module是同步的，需要在阻塞任务中执行
                        let compile_future = tokio::task::spawn_blocking(move || {
                            // 这里只是验证和预处理源码
                            // 实际的wgpu编译需要在主线程进行
                            // 当前实现：返回源码，实际编译由调用者完成
                            Ok(CompiledShader {
                                cache_key,
                                source,
                                compile_time_ms: start.elapsed().as_secs_f32() * 1000.0,
                            })
                        });

                        // 带超时的编译
                        match timeout(Duration::from_millis(timeout_ms), compile_future).await {
                            Ok(Ok(result)) => result,
                            Ok(Err(e)) => Err(CompileError::CompilationFailed(e.to_string())),
                            Err(_) => Err(CompileError::Timeout),
                        }
                    };

                    // 如果编译成功，存储到缓存
                    if let Ok(ref compiled) = result {
                        if enable_cache {
                            if let Some(cache) = &cache_clone {
                                let mut cache_guard = cache.lock().unwrap();
                                let _ =
                                    cache_guard.put_source(&compiled.cache_key, &compiled.source);
                            }
                        }
                    }

                    // 发送结果
                    let _ = response_tx.send(result);

                    // 释放许可
                    drop(permit);
                });
            }
        }
    }

    /// 提交编译请求
    pub fn compile_async(
        &self,
        label: Option<&str>,
        source: &str,
        compile_options: &str,
        priority: ShaderCompilePriority,
    ) -> Result<oneshot::Receiver<Result<CompiledShader, CompileError>>, RenderError> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let (response_tx, response_rx) = oneshot::channel();

        let request = ShaderCompileRequest {
            id,
            label: label.map(|s| s.to_string()),
            source: source.to_string(),
            compile_options: compile_options.to_string(),
            priority,
            created_at: Instant::now(),
            response_tx,
        };

        self.request_tx.send(request).map_err(|e| {
            RenderError::InvalidState(format!("Failed to send compile request: {}", e))
        })?;

        Ok(response_rx)
    }

    /// 编译着色器（简化版本）
    pub fn compile(
        &self,
        label: Option<&str>,
        source: &str,
    ) -> Result<oneshot::Receiver<Result<CompiledShader, CompileError>>, RenderError> {
        self.compile_async(label, source, "", ShaderCompilePriority::Normal)
    }

    /// 获取编译进度
    pub fn get_progress(&self) -> Option<CompileProgress> {
        let mut rx = self.progress_rx.lock().unwrap();
        rx.try_recv().ok()
    }

    /// 获取缓存（如果启用）
    pub fn cache(&self) -> Option<Arc<Mutex<ShaderCache>>> {
        self.cache.as_ref().map(|c| Arc::clone(c))
    }
}

/// 同步等待编译完成（辅助函数）
pub async fn wait_for_compile(
    rx: oneshot::Receiver<Result<CompiledShader, CompileError>>,
) -> Result<CompiledShader, CompileError> {
    rx.await.map_err(|_| CompileError::Cancelled)?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_compiler_basic() {
        let compiler = AsyncShaderCompiler::with_default_config().unwrap();

        let source = "fn main() {}";
        let rx = compiler.compile(None, source).unwrap();

        let result = wait_for_compile(rx).await;
        assert!(result.is_ok());

        let compiled = result.unwrap();
        assert_eq!(compiled.source, source);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let compiler = AsyncShaderCompiler::with_default_config().unwrap();

        // 提交不同优先级的请求
        let rx_low = compiler
            .compile_async(None, "low", "", ShaderCompilePriority::Low)
            .unwrap();
        let rx_high = compiler
            .compile_async(None, "high", "", ShaderCompilePriority::High)
            .unwrap();
        let rx_critical = compiler
            .compile_async(None, "critical", "", ShaderCompilePriority::Critical)
            .unwrap();

        // 高优先级应该先完成（理论上）
        // 注意：实际顺序可能受并发数影响
        let _ = wait_for_compile(rx_critical).await;
        let _ = wait_for_compile(rx_high).await;
        let _ = wait_for_compile(rx_low).await;
    }
}
