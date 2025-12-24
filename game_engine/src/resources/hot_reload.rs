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

/// 热重载管理器
///
/// 监视资源文件系统变化，并自动触发资源重新加载。
/// 支持依赖关系处理：当依赖的资源被修改时，自动重新加载依赖它的资源。
pub struct HotReloadManager {
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

impl HotReloadManager {
    /// 创建新的热重载管理器
    ///
    /// # 参数
    /// - `watch_path`: 要监视的目录路径
    /// - `dependency_graph`: 资源依赖图
    ///
    /// # 返回
    /// 新的热重载管理器实例
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
        let mut watched = self.watched_resources.write().unwrap();
        watched.insert(path);
    }

    /// 移除要监视的资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    pub fn unwatch_resource(&self, path: &PathBuf) {
        let mut watched = self.watched_resources.write().unwrap();
        watched.remove(path);
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
        let watched = self.watched_resources.read().unwrap();
        if !watched.contains(modified_path) {
            return targets;
        }
        drop(watched);

        // 添加被修改的资源本身
        targets.push(modified_path.clone());

        // 获取所有依赖此资源的资源（反向依赖）
        let graph = self.dependency_graph.read().unwrap();
        let dependents = graph.get_dependents(modified_path);
        drop(graph);

        // 递归获取所有依赖的资源
        for dependent in dependents {
            if !targets.contains(&dependent) {
                targets.push(dependent);
            }

            // 递归获取依赖的依赖
            let graph = self.dependency_graph.read().unwrap();
            let sub_dependents = graph.get_dependents(&dependent);
            drop(graph);

            for sub_dependent in sub_dependents {
                if !targets.contains(&sub_dependent) {
                    targets.push(sub_dependent);
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
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let last_modified = self.last_modified.read().unwrap();
                if let Some(&last_known) = last_modified.get(path) {
                    if modified > last_known {
                        // 文件已被修改
                        return true;
                    }
                } else {
                    // 首次检查，记录修改时间
                    drop(last_modified);
                    let mut last_modified = self.last_modified.write().unwrap();
                    last_modified.insert(path.clone(), modified);
                    return false;
                }
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
        let mut last_modified = self.last_modified.write().unwrap();
        last_modified.insert(path, modified);
    }

    /// 获取依赖图引用（用于外部操作）
    pub fn dependency_graph(&self) -> Arc<RwLock<DependencyGraph>> {
        self.dependency_graph.clone()
    }

    /// 获取被监视的资源数量
    pub fn watched_count(&self) -> usize {
        self.watched_resources.read().unwrap().len()
    }
}

/// 热重载服务（简化版本，向后兼容）
///
/// 这是对旧API的兼容包装，建议使用`HotReloadManager`。
pub struct HotReloadService {
    manager: HotReloadManager,
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
        let manager = HotReloadManager::new(path, dependency_graph)?;
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
    pub fn manager(&self) -> &HotReloadManager {
        &self.manager
    }

    /// 获取内部管理器（可变，用于高级功能）
    pub fn manager_mut(&mut self) -> &mut HotReloadManager {
        &mut self.manager
    }
}
