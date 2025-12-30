//  性能监控仪表板后端服务
//
//  提供 REST API 和 WebSocket 接口，供前端仪表板获取性能数据。
//  支持实时指标、历史数据和告警信息。
//
//  ## API 端点
//
//  - `GET /api/metrics` - 获取当前性能指标
//  - `GET /api/chart-data` - 获取图表数据
//  - `GET /api/alerts` - 获取告警信息
//  - `WS /ws` - WebSocket 实时数据推送
//
//  ## 使用示例
//
//  ```ignore
//  // 启动仪表板服务
//  let dashboard = DashboardService::new(profiling_service);
//  // dashboard.start_server("127.0.0.1:8080").await?;
//  ```

use axum::extract::ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade};
use axum::{
    Json, Router,
    extract::{Extension, Query},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;

use super::ProfilingService;

/// 实时指标数据结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct RealtimeMetrics {
    /// 时间戳（毫秒）
    pub timestamp: u64,
    /// 帧率 (FPS)
    pub fps: f64,
    /// 帧渲染时间（毫秒）
    pub frame_time: f64,
    /// CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 内存使用量（MB）
    pub memory_usage: f64,
    /// GPU使用率（百分比）
    pub gpu_usage: f64,
    /// 绘制调用数
    pub draw_calls: u64,
    /// 三角形数量
    pub triangle_count: u64,
    /// 物理计算时间（毫秒）
    pub physics_time: f64,
    /// 音频延迟（毫秒）
    pub audio_latency: f64,
    /// 协程任务统计
    pub coroutine: Option<CoroutineMetrics>,
    /// SIMD性能统计
    pub simd: Option<SimdMetrics>,
}

impl Default for RealtimeMetrics {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Test: operation should succeed")
                .as_millis() as u64,
            fps: 0.0,
            frame_time: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            gpu_usage: 0.0,
            draw_calls: 0,
            triangle_count: 0,
            physics_time: 0.0,
            audio_latency: 0.0,
            coroutine: None,
            simd: None,
        }
    }
}

/// 协程任务指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoroutineMetrics {
    /// 活跃任务数
    pub active_tasks: usize,
    /// 已完成任务数
    pub tasks_completed: u64,
    /// 失败任务数
    pub tasks_failed: u64,
    /// 平均任务执行时间（毫秒）
    pub avg_task_time: f64,
    /// 任务队列长度
    pub queue_length: usize,
}

/// SIMD性能指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimdMetrics {
    /// 当前使用的SIMD后端
    pub backend: String,
    /// SIMD向量宽度（位）
    pub simd_width: u32,
    /// 可并行处理的f32数量
    pub f32_lanes: usize,
    /// SIMD优化使用率（百分比）
    pub usage_percent: f64,
    /// SIMD加速比（相对于标量实现）
    pub speedup_ratio: f64,
}

/// 告警信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertInfo {
    /// 告警ID
    pub id: String,
    /// 告警级别
    pub severity: String,
    /// 告警消息
    pub message: String,
    /// 告警时间戳
    pub timestamp: u64,
    /// 指标名称
    pub metric_name: Option<String>,
    /// 当前值
    pub current_value: Option<f64>,
    /// 阈值
    pub threshold: Option<f64>,
}

/// 性能趋势数据点
#[derive(Debug, Clone, serde::Serialize)]
pub struct TrendDataPoint {
    /// 时间戳
    pub timestamp: u64,
    /// 平均值
    pub avg: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 第95百分位数
    pub p95: f64,
    /// 第99百分位数
    pub p99: f64,
}

/// 性能趋势数据
#[derive(Debug, Clone, serde::Serialize)]
pub struct TrendData {
    /// 指标名称
    pub metric_name: String,
    /// 指标单位
    pub unit: String,
    /// 数据点
    pub data_points: Vec<TrendDataPoint>,
    /// 时间范围（秒）
    pub time_range_seconds: u64,
}

/// 仪表板配置
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// 服务器地址
    pub bind_address: String,
    /// 是否启用CORS
    pub enable_cors: bool,
    /// 数据保留时间（秒）
    pub data_retention_seconds: u64,
    /// 告警阈值配置
    pub alert_thresholds: AlertThresholds,
    /// WebSocket端口
    pub ws_port: u16,
    /// 是否启用WebSocket
    pub enable_websocket: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            enable_cors: true,
            data_retention_seconds: 300,
            alert_thresholds: AlertThresholds::default(),
            ws_port: 8080,
            enable_websocket: true,
        }
    }
}

/// 告警阈值配置
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// 低帧率阈值（FPS）
    pub low_fps_threshold: f64,
    /// 高帧时间阈值（ms）
    pub high_frame_time_threshold: f64,
    /// 高内存使用率阈值（%）
    pub high_memory_threshold: f64,
    /// 高CPU使用率阈值（%）
    pub high_cpu_threshold: f64,
    /// 高GPU使用率阈值（%）
    pub high_gpu_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            low_fps_threshold: 30.0,
            high_frame_time_threshold: 33.3, // 30 FPS的倒数
            high_memory_threshold: 90.0,
            high_cpu_threshold: 85.0,
            high_gpu_threshold: 90.0,
        }
    }
}

/// 渲染指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderMetrics {
    /// 帧率
    pub fps: f64,
    /// 帧渲染时间
    pub frame_time: f64,
    /// Draw Call数量
    pub draw_calls: u64,
    /// 实例数量
    pub instance_count: u64,
    /// 纹理加载数量
    pub texture_loads: u64,
    /// 纹理加载失败数
    pub texture_load_failures: u64,
}

/// 内存指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryMetrics {
    /// 使用率（%）
    pub usage_percent: f64,
    /// 已分配内存（MB）
    pub allocated_mb: f64,
    /// 缓冲区数量
    pub buffer_count: u64,
    /// 纹理数量
    pub texture_count: u64,
}

/// 物理指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct PhysicsMetrics {
    /// 计算时间
    pub calc_time: f64,
    /// 碰撞检测次数
    pub collision_count: u64,
    /// 同步操作次数
    pub sync_count: u64,
    /// 跳过的休眠体数量
    pub sleeping_skipped: u64,
}

/// 系统指标
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemMetrics {
    /// CPU使用率（%）
    pub cpu_usage: f64,
    /// GPU使用率（%）
    pub gpu_usage: f64,
    /// 线程数量
    pub thread_count: u32,
}

/// 图表数据响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChartDataResponse {
    /// 时间标签
    pub labels: Vec<String>,
    /// 数值
    pub values: Vec<f64>,
}

/// 性能指标响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsResponse {
    /// 渲染指标
    pub render: Option<RenderMetrics>,
    /// 内存指标
    pub memory: Option<MemoryMetrics>,
    /// 物理指标
    pub physics: Option<PhysicsMetrics>,
    /// 系统指标
    pub system: Option<SystemMetrics>,
    /// 协程指标
    pub coroutine: Option<CoroutineMetrics>,
    /// SIMD指标
    pub simd: Option<SimdMetrics>,
    /// 告警信息
    pub alerts: Vec<AlertInfo>,
}

/// 仪表板服务
pub struct DashboardService {
    /// 性能监控服务
    pub profiling_service: Arc<ProfilingService>,
    /// 配置
    pub config: DashboardConfig,
    /// 历史数据
    pub historical_data: Arc<RwLock<HashMap<String, Vec<(Instant, f64)>>>>,
    /// WebSocket连接列表
    pub websocket_connections: Arc<Mutex<Vec<WebSocketSender>>>,
}

/// WebSocket发送器包装（使用 mpsc 发送 JSON 文本消息）
#[derive(Debug, Clone)]
pub struct WebSocketSender {
    /// 发送器（用于发送文本消息）
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
    /// 连接ID
    pub id: String,
}

impl DashboardService {
    /// 创建新的仪表板服务
    pub fn new(profiling_service: Arc<ProfilingService>) -> Self {
        Self::with_config(profiling_service, DashboardConfig::default())
    }

    /// 使用指定配置创建仪表板服务
    pub fn with_config(profiling_service: Arc<ProfilingService>, config: DashboardConfig) -> Self {
        Self {
            profiling_service,
            config,
            historical_data: Arc::new(RwLock::new(HashMap::new())),
            websocket_connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 启动HTTP和WebSocket服务器
    pub async fn start_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let profiling_service = self.profiling_service.clone();
        let historical_data = self.historical_data.clone();
        let websocket_connections = self.websocket_connections.clone();
        let config = self.config.clone();

        // 构造共享状态
        let state = Arc::new(AppState {
            profiling_service: profiling_service.clone(),
            historical_data: historical_data.clone(),
            connections: websocket_connections.clone(),
            config: config.clone(),
        });

        // 构建路由
        let app = Router::new()
            .route("/api/metrics", get(get_metrics_axum))
            .route("/api/chart-data", get(get_chart_data_axum))
            .route("/api/alerts", get(get_alerts_axum))
            .route("/ws", get(ws_handler))
            .layer(Extension(state.clone()));

        // CORS and Trace layers are temporarily disabled to avoid dependency version mismatches.
        // Re-enable `TraceLayer` and `CorsLayer` after updating `tower-http`/`http` to compatible versions.

        // 启动实时推送任务（如果启用）
        if config.enable_websocket {
            self.start_realtime_push().await;
        }

        let addr = self.config.bind_address.clone();
        tracing::info!(
            "Starting dashboard server on {} (WebSocket: {})",
            addr,
            config.enable_websocket
        );

        let socket_addr = addr.parse::<std::net::SocketAddr>()?;
        axum_server::bind(socket_addr).serve(app.into_make_service()).await?;

        Ok(())
    }

    /// 启动实时数据推送任务
    async fn start_realtime_push(&self) {
        let profiling_service = self.profiling_service.clone();
        let connections = self.websocket_connections.clone();
        let update_interval = Duration::from_millis(100); // 10Hz更新频率

        tokio::spawn(async move {
            let mut interval = interval(update_interval);

            loop {
                interval.tick().await;

                // 收集实时指标
                let realtime_metrics = collect_realtime_metrics(&profiling_service).await;

                // 序列化数据
                if let Ok(json_data) = serde_json::to_string(&realtime_metrics) {
                    // 广播给所有连接的客户端
                    let mut connections_guard = connections.lock().await;
                    let mut to_remove = Vec::new();

                    for (i, sender) in connections_guard.iter().enumerate() {
                        if sender.sender.send(json_data.clone()).is_err() {
                            // 连接已断开，标记为待移除
                            to_remove.push(i);
                            tracing::warn!("WebSocket connection {} disconnected", sender.id);
                        }
                    }

                    // 移除断开的连接
                    for &i in to_remove.iter().rev() {
                        connections_guard.remove(i);
                    }
                }
            }
        });
    }

    /// 收集当前性能指标
    async fn collect_metrics(&self) -> MetricsResponse {
        let service = &self.profiling_service;

        // 收集渲染指标
        let render = self.collect_render_metrics(service).await;

        // 收集内存指标
        let memory = self.collect_memory_metrics(service).await;

        // 收集物理指标
        let physics = self.collect_physics_metrics(service).await;

        // 收集系统指标
        let system = self.collect_system_metrics(service).await;

        // 收集协程指标
        let coroutine = self.collect_coroutine_metrics().await;

        // 收集SIMD指标
        let simd = self.collect_simd_metrics().await;

        // 检查告警
        let alerts = self.check_alerts(&render, &memory, &physics, &system).await;

        MetricsResponse {
            render,
            memory,
            physics,
            system,
            coroutine,
            simd,
            alerts,
        }
    }

    /// 收集渲染指标
    async fn collect_render_metrics(&self, service: &ProfilingService) -> Option<RenderMetrics> {
        let metrics = service.get_realtime_metrics().ok()?;

        Some(RenderMetrics {
            fps: metrics.fps,
            frame_time: metrics.frame_time,
            draw_calls: metrics.draw_calls,
            instance_count: 0,
            texture_loads: 0,
            texture_load_failures: 0,
        })
    }

    /// 收集内存指标
    async fn collect_memory_metrics(&self, service: &ProfilingService) -> Option<MemoryMetrics> {
        let metrics = service.get_realtime_metrics().ok()?;
        let total_memory_mb = 4096.0;
        let usage_percent = (metrics.memory_usage / total_memory_mb) * 100.0;

        Some(MemoryMetrics {
            usage_percent,
            allocated_mb: metrics.memory_usage,
            buffer_count: 0,
            texture_count: 0,
        })
    }

    /// 收集物理指标
    async fn collect_physics_metrics(&self, service: &ProfilingService) -> Option<PhysicsMetrics> {
        let metrics = service.get_realtime_metrics().ok()?;

        Some(PhysicsMetrics {
            calc_time: metrics.physics_time,
            collision_count: 0,
            sync_count: 0,
            sleeping_skipped: 0,
        })
    }

    /// 收集系统指标
    async fn collect_system_metrics(&self, service: &ProfilingService) -> Option<SystemMetrics> {
        let metrics = service.get_realtime_metrics().ok()?;

        Some(SystemMetrics {
            cpu_usage: metrics.cpu_usage,
            gpu_usage: metrics.gpu_usage,
            thread_count: 0,
        })
    }

    /// 收集协程指标
    async fn collect_coroutine_metrics(&self) -> Option<CoroutineMetrics> {
        // 注意：这里需要从ECS世界获取CoroutineTaskManager
        // 由于DashboardService没有直接访问ECS世界的权限，
        // 我们需要通过ProfilingService或其他方式获取这些数据
        // 暂时返回None，实际实现需要从ECS世界获取
        None
    }

    /// 收集SIMD指标
    async fn collect_simd_metrics(&self) -> Option<SimdMetrics> {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            use game_engine_simd::SimdBackend;
            let backend = SimdBackend::best_available();
            let backend_name = format!("{backend:?}");
            let simd_width = match backend.width() {
                game_engine_simd::SimdWidth::W128 => 128,
                game_engine_simd::SimdWidth::W256 => 256,
                game_engine_simd::SimdWidth::W512 => 512,
            };
            let f32_lanes = backend.f32_lanes();

            Some(SimdMetrics {
                backend: backend_name,
                simd_width,
                f32_lanes,
                usage_percent: 100.0, // 假设SIMD始终在使用（实际需要运行时统计）
                speedup_ratio: match backend {
                    SimdBackend::Avx512 => 8.0,
                    SimdBackend::Avx2 => 4.0,
                    SimdBackend::Avx => 3.0,
                    SimdBackend::Sse41 => 2.5,
                    SimdBackend::Sse2 => 2.0,
                    SimdBackend::Neon => 2.0,
                    SimdBackend::Sve => 4.0,
                    SimdBackend::Scalar => 1.0,
                },
            })
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            None
        }
    }

    /// 检查告警条件
    async fn check_alerts(
        &self,
        render: &Option<RenderMetrics>,
        memory: &Option<MemoryMetrics>,
        _physics: &Option<PhysicsMetrics>,
        system: &Option<SystemMetrics>,
    ) -> Vec<AlertInfo> {
        let mut alerts = Vec::new();
        let thresholds = &self.config.alert_thresholds;

        // 检查低帧率
        if let Some(render) = render
            && render.fps < thresholds.low_fps_threshold
        {
            alerts.push(AlertInfo {
                id: format!("low_fps_{}", current_timestamp()),
                severity: "warning".to_string(),
                message: format!("低帧率: {:.1} FPS", render.fps),
                timestamp: current_timestamp(),
                metric_name: Some("fps".to_string()),
                current_value: Some(render.fps),
                threshold: Some(thresholds.low_fps_threshold),
            });
        }

        // 检查高帧时间
        if let Some(render) = render
            && render.frame_time > thresholds.high_frame_time_threshold
        {
            alerts.push(AlertInfo {
                id: format!("high_frame_time_{}", current_timestamp()),
                severity: "warning".to_string(),
                message: format!("高帧时间: {:.2} ms", render.frame_time),
                timestamp: current_timestamp(),
                metric_name: Some("frame_time".to_string()),
                current_value: Some(render.frame_time),
                threshold: Some(thresholds.high_frame_time_threshold),
            });
        }

        // 检查高内存使用率
        if let Some(memory) = memory
            && memory.usage_percent > thresholds.high_memory_threshold
        {
            alerts.push(AlertInfo {
                id: format!("high_memory_{}", current_timestamp()),
                severity: "error".to_string(),
                message: format!("高内存使用率: {:.1}%", memory.usage_percent),
                timestamp: current_timestamp(),
                metric_name: Some("memory_usage".to_string()),
                current_value: Some(memory.usage_percent),
                threshold: Some(thresholds.high_memory_threshold),
            });
        }

        // 检查高CPU使用率
        if let Some(system) = system
            && system.cpu_usage > thresholds.high_cpu_threshold
        {
            alerts.push(AlertInfo {
                id: format!("high_cpu_{}", current_timestamp()),
                severity: "error".to_string(),
                message: format!("高CPU使用率: {:.1}%", system.cpu_usage),
                timestamp: current_timestamp(),
                metric_name: Some("cpu_usage".to_string()),
                current_value: Some(system.cpu_usage),
                threshold: Some(thresholds.high_cpu_threshold),
            });
        }

        // 检查高GPU使用率
        if let Some(system) = system
            && system.gpu_usage > thresholds.high_gpu_threshold
        {
            alerts.push(AlertInfo {
                id: format!("high_gpu_{}", current_timestamp()),
                severity: "error".to_string(),
                message: format!("高GPU使用率: {:.1}%", system.gpu_usage),
                timestamp: current_timestamp(),
                metric_name: Some("gpu_usage".to_string()),
                current_value: Some(system.gpu_usage),
                threshold: Some(thresholds.high_gpu_threshold),
            });
        }

        alerts
    }

    /// 更新历史数据
    async fn update_historical_data(&self) {
        let service = &self.profiling_service;
        let mut data = self.historical_data.write().await;

        let metrics = match service.get_realtime_metrics() {
            Ok(m) => m,
            Err(_) => return,
        };

        add_data_point(&mut data, "fps", metrics.fps);
        add_data_point(&mut data, "frame_time", metrics.frame_time);
        add_data_point(&mut data, "draw_calls", metrics.draw_calls as f64);
        add_data_point(&mut data, "memory_usage", metrics.memory_usage);
        add_data_point(&mut data, "physics_time", metrics.physics_time);

        let retention = Duration::from_secs(self.config.data_retention_seconds);
        let cutoff = Instant::now() - retention;

        for (_, values) in data.iter_mut() {
            values.retain(|(timestamp, _)| *timestamp >= cutoff);
        }
    }
}

/// 添加数据点到历史数据
fn add_data_point(data: &mut HashMap<String, Vec<(Instant, f64)>>, key: &str, value: f64) {
    let values = data.entry(key.to_string()).or_default();
    values.push((Instant::now(), value));
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// 处理WebSocket连接（axum 版本）
async fn handle_websocket_connection(
    websocket: WebSocket,
    _service: Arc<ProfilingService>,
    connections: Arc<Mutex<Vec<WebSocketSender>>>,
) {
    let (ws_tx, mut ws_rx) = websocket.split();

    // 生成唯一连接ID
    let connection_id = format!("ws_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    // mpsc 发送器用于推送 JSON 文本
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let sender = WebSocketSender {
        sender: tx.clone(),
        id: connection_id.clone(),
    };

    // 添加到连接列表
    {
        let mut connections_guard = connections.lock().await;
        connections_guard.push(sender.clone());
        tracing::info!("WebSocket connection {} established", connection_id);
    }

    // 启动写任务：从 rx 中读取字符串并发送到 WebSocket
    let write_task = tokio::spawn(async move {
        let mut write_ws_tx = ws_tx;
        while let Some(msg) = rx.recv().await {
            if write_ws_tx.send(AxumMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // 发送欢迎消息
    let welcome_msg = serde_json::json!({
        "type": "welcome",
        "message": "Connected to performance monitoring dashboard",
        "connection_id": connection_id
    });
    if let Ok(json) = serde_json::to_string(&welcome_msg) {
        let _ = tx.send(json);
    }

    // 处理客户端消息
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            AxumMessage::Text(t) => {
                handle_client_message(&t, &connection_id).await;
            }
            AxumMessage::Close(_) => break,
            _ => {}
        }
    }

    // 连接关闭，从列表中移除
    {
        let mut connections_guard = connections.lock().await;
        if let Some(pos) = connections_guard.iter().position(|s| s.id == connection_id) {
            connections_guard.remove(pos);
            tracing::info!("WebSocket connection {} closed", connection_id);
        }
    }

    // 关闭写任务
    drop(tx);
    let _ = write_task.await;
}

/// 处理客户端消息
async fn handle_client_message(message: &str, connection_id: &str) {
    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(message) {
        match msg.get("type").and_then(|v| v.as_str()) {
            Some("ping") => {
                tracing::debug!("Received ping from connection {}", connection_id);
            }
            Some("subscribe") => {
                if let Some(metrics) = msg.get("metrics").and_then(|v| v.as_array()) {
                    let metric_names: Vec<String> =
                        metrics.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    tracing::info!(
                        "Connection {} subscribed to metrics: {:?}",
                        connection_id,
                        metric_names
                    );
                }
            }
            Some("unsubscribe") => {
                if let Some(metrics) = msg.get("metrics").and_then(|v| v.as_array()) {
                    let metric_names: Vec<String> =
                        metrics.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    tracing::info!(
                        "Connection {} unsubscribed from metrics: {:?}",
                        connection_id,
                        metric_names
                    );
                }
            }
            _ => {}
        }
    } else {
        tracing::warn!(
            "Invalid JSON message from connection {}: {}",
            connection_id,
            message
        );
    }
}

/// 收集实时指标数据
async fn collect_realtime_metrics(service: &ProfilingService) -> RealtimeMetrics {
    service.get_realtime_metrics().unwrap_or_default()
}

/// 应用状态（用于 axum 提取）
#[derive(Clone)]
struct AppState {
    profiling_service: Arc<ProfilingService>,
    historical_data: Arc<RwLock<HashMap<String, Vec<(Instant, f64)>>>>,
    connections: Arc<Mutex<Vec<WebSocketSender>>>,
    config: DashboardConfig,
}

/// WebSocket 路由处理器（升级并交给真实处理函数）
async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        let svc = state.profiling_service.clone();
        let conns = state.connections.clone();
        handle_websocket_connection(socket, svc, conns)
    })
}

/// API处理器：获取当前性能指标（axum 版本）
async fn get_metrics_axum(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let dashboard =
        DashboardService::with_config(state.profiling_service.clone(), state.config.clone());
    let metrics = dashboard.collect_metrics().await;
    // 更新历史数据
    dashboard.update_historical_data().await;
    Json(metrics)
}

/// API处理器：获取图表数据（axum 版本）
async fn get_chart_data_axum(
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let metric = params.get("metric").unwrap_or(&"fps".to_string()).clone();
    let range_seconds = params.get("range").and_then(|s| s.parse().ok()).unwrap_or(300);

    let data = state.historical_data.read().await;
    let values = data.get(&metric).cloned().unwrap_or_default();

    // 过滤时间范围
    let cutoff = Instant::now() - Duration::from_secs(range_seconds);
    let filtered_values: Vec<_> =
        values.into_iter().filter(|(timestamp, _)| *timestamp >= cutoff).collect();

    // 转换为图表格式
    let (labels, chart_values): (Vec<_>, Vec<_>) = filtered_values
        .into_iter()
        .map(|(timestamp, value)| {
            let secs = timestamp.elapsed().as_secs();
            let time_str = format!("-{}s", range_seconds - secs);
            (time_str, value)
        })
        .unzip();

    let response = ChartDataResponse {
        labels,
        values: chart_values,
    };

    Json(response)
}

/// API处理器：获取告警信息（axum 版本）
async fn get_alerts_axum(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let dashboard = DashboardService::new(state.profiling_service.clone());
    let metrics = dashboard.collect_metrics().await;
    Json(metrics.alerts)
}
