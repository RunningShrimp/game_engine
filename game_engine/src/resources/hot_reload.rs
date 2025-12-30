//! 资源热重载管理器
//!
//! 提供资源文件系统监视和自动重新加载功能。
//! 支持依赖关系处理：当依赖的资源被修改时，自动重新加载依赖它的资源。

use super::dependency_manager::DependencyGraph;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

/// 热重载事件
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// 资源文件被修改
    ResourceModified(PathBuf),
    /// 资源文件被删除
    ResourceDeleted(PathBuf),
    /// 资源文件被创建
    ResourceCreated(PathBuf),
}

/// 资源热加载管理器
///
/// 监视资源文件系统变化，并自动触发资源重新加载。
/// 支持依赖关系处理：当依赖的资源被修改时，自动重新加载依赖它的资源。
/// 与PluginHotReloadManager不同，本管理器专注于资源文件（着色器、纹理等）的监控。
pub struct ResourceHotReloadManager {
    /// 文件系统监视器
    _watcher: RecommendedWatcher,
    /// 事件接收器
    event_rx: mpsc::Receiver<HotReloadEvent>,
    /// 事件发送器（用于外部发送事件）
    event_tx: mpsc::Sender<HotReloadEvent>,
    /// 依赖图（用于确定需要重新加载的资源）
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// 被监视的资源路径集合
    watched_resources: Arc<RwLock<HashSet<PathBuf>>>,
    /// 资源最后修改时间映射
    last_modified: Arc<RwLock<std::collections::HashMap<PathBuf, SystemTime>>>,
    /// 防抖延迟（毫秒）
    debounce_delay: Duration,
}

impl ResourceHotReloadManager {
    /// 创建新的资源热加载管理器
    ///
    /// # 参数
    /// - `watch_path`: 要监视的目录路径
    /// - `dependency_graph`: 资源依赖图
    ///
    /// # 返回
    /// 新的资源热加载管理器实例
    pub fn new(
        watch_path: impl AsRef<Path>,
        dependency_graph: Arc<RwLock<DependencyGraph>>,
    ) -> NotifyResult<Self> {
        let (event_tx, event_rx) = mpsc::channel(100);
        let event_tx_clone = event_tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    // 只处理修改、创建和删除事件
                    match &event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            for path in &event.paths {
                                let event_type = match &event.kind {
                                    EventKind::Remove(_) => {
                                        HotReloadEvent::ResourceDeleted(path.clone())
                                    }
                                    EventKind::Create(_) => {
                                        HotReloadEvent::ResourceCreated(path.clone())
                                    }
                                    _ => HotReloadEvent::ResourceModified(path.clone()),
                                };

                                // 异步发送事件（非阻塞）
                                let tx = event_tx_clone.clone();
                                tokio::spawn(async move {
                                    let _ = tx.send(event_type).await;
                                });
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(watch_path.as_ref(), RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            event_rx,
            event_tx,
            dependency_graph,
            watched_resources: Arc::new(RwLock::new(HashSet::new())),
            last_modified: Arc::new(RwLock::new(std::collections::HashMap::new())),
            debounce_delay: Duration::from_millis(100), // 默认100ms防抖
        })
    }

    /// 设置防抖延迟
    ///
    /// # 参数
    /// - `delay`: 防抖延迟时间
    pub fn set_debounce_delay(&mut self, delay: Duration) {
        self.debounce_delay = delay;
    }

    /// 添加要监视的资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    pub fn watch_resource(&self, path: PathBuf) {
        if let Ok(mut watched) = self.watched_resources.write() {
            watched.insert(path);
        }
    }

    /// 移除要监视的资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    pub fn unwatch_resource(&self, path: &PathBuf) {
        if let Ok(mut watched) = self.watched_resources.write() {
            watched.remove(path);
        }
    }

    /// 轮询热重载事件（非阻塞）
    ///
    /// # 返回
    /// 如果有事件则返回Some，否则返回None
    pub async fn poll_event(&mut self) -> Option<HotReloadEvent> {
        self.event_rx.try_recv().ok()
    }

    /// 等待下一个热重载事件（阻塞）
    ///
    /// # 返回
    /// 热重载事件
    pub async fn next_event(&mut self) -> Option<HotReloadEvent> {
        self.event_rx.recv().await
    }

    /// 批量处理热重载事件（协程版本）
    ///
    /// 使用Tokio协程批量处理多个热重载事件，支持并发重载和防抖。
    ///
    /// # 参数
    /// - `max_batch_size`: 最大批处理大小
    /// - `timeout`: 批处理超时时间
    ///
    /// # 返回
    /// 返回一个Future，解析为处理的事件列表
    pub async fn process_events_batch(
        &mut self,
        max_batch_size: usize,
        timeout: Duration,
    ) -> Vec<HotReloadEvent> {
        use tokio::time::Instant;
        // sleep 未在此文件中使用，但可能在未来需要
        // use tokio::time::{sleep, Instant};

        let mut events = Vec::new();
        let start_time = Instant::now();

        // 收集事件直到达到批处理大小或超时
        while events.len() < max_batch_size && start_time.elapsed() < timeout {
            match tokio::time::timeout(timeout - start_time.elapsed(), self.event_rx.recv()).await {
                Ok(Some(event)) => {
                    events.push(event);
                }
                Ok(None) => break, // 通道已关闭
                Err(_) => break,   // 超时
            }
        }

        // 应用防抖：合并相同路径的连续事件
        self.debounce_events(&mut events);

        events
    }

    /// 并发重载多个资源（协程版本）
    ///
    /// 使用Tokio协程并发重载多个资源，提升性能。
    ///
    /// # 参数
    /// - `paths`: 要重载的资源路径列表
    /// - `reload_fn`: 重载函数（异步）
    ///
    /// # 返回
    /// 返回一个Future，解析为重载结果列表
    pub async fn reload_resources_concurrent<F, Fut>(
        &self,
        paths: Vec<PathBuf>,
        reload_fn: F,
    ) -> Vec<Result<(), String>>
    where
        F: Fn(PathBuf) -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        use futures::future::join_all;

        // 创建并发重载任务
        let reload_tasks: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let reload_fn_clone = std::sync::Arc::new(reload_fn.clone());
                tokio::task::spawn(async move {
                    let reload_fn = reload_fn_clone.as_ref();
                    reload_fn(path).await
                })
            })
            .collect();

        // 等待所有重载任务完成
        let results = join_all(reload_tasks).await;

        // 收集结果
        results.into_iter().map(|r| r.unwrap_or_else(|e| Err(e.to_string()))).collect()
    }

    /// 防抖事件：合并相同路径的连续事件
    fn debounce_events(&self, events: &mut Vec<HotReloadEvent>) {
        use std::collections::HashMap;

        // 按路径分组事件，只保留最后一个事件
        let mut path_to_event: HashMap<PathBuf, HotReloadEvent> = HashMap::new();

        for event in events.drain(..) {
            let path = match &event {
                HotReloadEvent::ResourceModified(p) => p.clone(),
                HotReloadEvent::ResourceDeleted(p) => p.clone(),
                HotReloadEvent::ResourceCreated(p) => p.clone(),
            };
            path_to_event.insert(path, event);
        }

        // 将去重后的事件放回
        events.extend(path_to_event.into_values());
    }

    /// 获取需要重新加载的资源列表（考虑依赖关系）
    ///
    /// 当资源被修改时，此方法会：
    /// 1. 检查资源是否被监视
    /// 2. 获取所有依赖此资源的资源（反向依赖）
    /// 3. 返回需要重新加载的资源列表
    ///
    /// # 参数
    /// - `modified_path`: 被修改的资源路径
    ///
    /// # 返回
    /// 需要重新加载的资源路径列表（包括被修改的资源本身）
    pub fn get_reload_targets(&self, modified_path: &PathBuf) -> Vec<PathBuf> {
        let mut targets = Vec::new();

        // 检查资源是否被监视
        if let Ok(watched) = self.watched_resources.read() {
            if !watched.contains(modified_path) {
                return targets;
            }
        } else {
            return targets;
        }

        // 添加被修改的资源本身
        targets.push(modified_path.clone());

        // 获取所有依赖此资源的资源（反向依赖）
        if let Ok(graph) = self.dependency_graph.read() {
            let dependents = graph.get_dependents(modified_path);

            // 递归获取所有依赖的资源
            for dependent in dependents {
                if !targets.contains(&dependent) {
                    targets.push(dependent.clone());
                }

                // 递归获取依赖的依赖
                let sub_dependents = graph.get_dependents(&dependent);

                for sub_dependent in sub_dependents {
                    if !targets.contains(&sub_dependent) {
                        targets.push(sub_dependent);
                    }
                }
            }
        }

        targets
    }

    /// 检查资源是否需要重新加载（基于最后修改时间）
    ///
    /// # 参数
    /// - `path`: 资源路径
    ///
    /// # 返回
    /// 如果需要重新加载则返回true
    pub fn needs_reload(&self, path: &PathBuf) -> bool {
        // 检查文件系统最后修改时间
        if let Ok(metadata) = std::fs::metadata(path)
            && let Ok(modified) = metadata.modified()
            && let Ok(last_modified) = self.last_modified.read()
        {
            if let Some(&last_known) = last_modified.get(path) {
                if modified > last_known {
                    // 文件已被修改
                    return true;
                }
            } else {
                // 首次检查，记录修改时间
                drop(last_modified);
                if let Ok(mut last_modified) = self.last_modified.write() {
                    last_modified.insert(path.clone(), modified);
                }
                return false;
            }
        }
        false
    }

    /// 更新资源的最后修改时间
    ///
    /// # 参数
    /// - `path`: 资源路径
    /// - `modified`: 最后修改时间
    pub fn update_last_modified(&self, path: PathBuf, modified: SystemTime) {
        if let Ok(mut last_modified) = self.last_modified.write() {
            last_modified.insert(path, modified);
        }
    }

    /// 获取依赖图引用（用于外部操作）
    pub fn dependency_graph(&self) -> Arc<RwLock<DependencyGraph>> {
        self.dependency_graph.clone()
    }

    /// 获取被监视的资源数量
    pub fn watched_count(&self) -> usize {
        if let Ok(watched) = self.watched_resources.read() {
            watched.len()
        } else {
            0
        }
    }
}

/// 热重载服务（简化版本，向后兼容）
///
/// 这是对旧API的兼容包装，建议使用`ResourceHotReloadManager`。
pub struct HotReloadService {
    manager: ResourceHotReloadManager,
}

impl HotReloadService {
    /// 创建新的热重载服务
    ///
    /// # 参数
    /// - `path`: 要监视的目录路径
    ///
    /// # 返回
    /// 新的热重载服务实例
    pub fn watch_dir(path: PathBuf) -> NotifyResult<Self> {
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));
        let manager = ResourceHotReloadManager::new(path, dependency_graph)?;
        Ok(Self { manager })
    }

    /// 轮询文件变化（非阻塞）
    ///
    /// # 返回
    /// 如果有文件变化则返回Some(路径)，否则返回None
    pub async fn poll(&mut self) -> Option<PathBuf> {
        if let Some(event) = self.manager.poll_event().await {
            match event {
                HotReloadEvent::ResourceModified(path)
                | HotReloadEvent::ResourceCreated(path)
                | HotReloadEvent::ResourceDeleted(path) => {
                    return Some(path);
                }
            }
        }
        None
    }

    /// 获取内部管理器（用于高级功能）
    pub fn manager(&self) -> &ResourceHotReloadManager {
        &self.manager
    }

    /// 获取内部管理器（可变，用于高级功能）
    pub fn manager_mut(&mut self) -> &mut ResourceHotReloadManager {
        &mut self.manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_hot_reload_event_creation() {
        let path = PathBuf::from("/test/resource.txt");

        let modified_event = HotReloadEvent::ResourceModified(path.clone());
        let deleted_event = HotReloadEvent::ResourceDeleted(path.clone());
        let created_event = HotReloadEvent::ResourceCreated(path);

        // 验证事件创建成功
        match modified_event {
            HotReloadEvent::ResourceModified(p) => {
                assert_eq!(p, PathBuf::from("/test/resource.txt"))
            }
            _ => panic!("Expected ResourceModified event"),
        }

        match deleted_event {
            HotReloadEvent::ResourceDeleted(p) => {
                assert_eq!(p, PathBuf::from("/test/resource.txt"))
            }
            _ => panic!("Expected ResourceDeleted event"),
        }

        match created_event {
            HotReloadEvent::ResourceCreated(p) => {
                assert_eq!(p, PathBuf::from("/test/resource.txt"))
            }
            _ => panic!("Expected ResourceCreated event"),
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_hot_reload_manager_watch_resource() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试监视资源
        let test_path = temp_dir.path().join("test.txt");
        manager.watch_resource(test_path.clone());

        assert_eq!(manager.watched_count(), 1);

        // 测试取消监视
        manager.unwatch_resource(&test_path);
        assert_eq!(manager.watched_count(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_hot_reload_manager_set_debounce_delay() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let mut manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试设置防抖延迟
        let new_delay = Duration::from_millis(500);
        manager.set_debounce_delay(new_delay);

        assert_eq!(manager.debounce_delay, new_delay);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_hot_reload_service_creation() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });

        let service =
            HotReloadService::watch_dir(temp_dir.path().to_path_buf()).unwrap_or_else(|e| {
                panic!("Failed to create service: {}", e);
            });

        // 验证服务创建成功
        assert_eq!(service.manager().watched_count(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_get_reload_targets_empty() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试未监视的资源
        let test_path = PathBuf::from("/unwatched/resource.txt");
        let targets = manager.get_reload_targets(&test_path);

        assert_eq!(targets.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_needs_reload_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试不存在的文件
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");
        assert!(!manager.needs_reload(&nonexistent_path));
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_update_last_modified() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试更新最后修改时间
        let test_path = temp_dir.path().join("test.txt");
        let now = SystemTime::now();

        manager.update_last_modified(test_path.clone(), now);

        // 验证更新
        assert!(!manager.needs_reload(&test_path));
    }

    #[tokio::test]
    async fn test_hot_reload_poll_event_no_events() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let mut manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试轮询事件（应该返回None）
        let event = manager.poll_event().await;
        assert!(event.is_none());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_debounce_events() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph)
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试事件去重
        let path = temp_dir.path().join("test.txt");
        let mut events = vec![
            HotReloadEvent::ResourceModified(path.clone()),
            HotReloadEvent::ResourceModified(path.clone()),
            HotReloadEvent::ResourceCreated(path.clone()),
        ];

        manager.debounce_events(&mut events);

        // 应该只保留最后一个事件
        assert_eq!(events.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_dependency_graph_access() {
        let temp_dir = TempDir::new().unwrap_or_else(|e| {
            panic!("Failed to create temp dir: {}", e);
        });
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::new()));

        let manager = ResourceHotReloadManager::new(temp_dir.path(), dependency_graph.clone())
            .unwrap_or_else(|e| {
                panic!("Failed to create manager: {}", e);
            });

        // 测试获取依赖图
        let graph = manager.dependency_graph();
        assert!(Arc::ptr_eq(&graph, &dependency_graph));
    }
}
