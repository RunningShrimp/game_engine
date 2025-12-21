//! 资源预加载管理器
//!
//! 提供资源预加载策略和实现，支持：
//! - 基于场景的预加载
//! - 基于距离的预加载
//! - 基于优先级的预加载
//! - 预加载进度追踪

use crate::resources::coroutine_loader::{CoroutineAssetLoader, LoadPriority, AssetType};
use crate::resources::dependency_manager::DependencyGraph;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// 预加载策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadStrategy {
    /// 立即预加载（最高优先级）
    Immediate,
    /// 基于场景的预加载
    SceneBased,
    /// 基于距离的预加载
    DistanceBased,
    /// 后台预加载（最低优先级）
    Background,
}

impl PartialOrd for PreloadStrategy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreloadStrategy {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级顺序：Immediate > SceneBased > DistanceBased > Background
        let order = |s: &PreloadStrategy| match s {
            PreloadStrategy::Immediate => 0,
            PreloadStrategy::SceneBased => 1,
            PreloadStrategy::DistanceBased => 2,
            PreloadStrategy::Background => 3,
        };
        order(self).cmp(&order(other))
    }
}

/// 预加载配置
#[derive(Debug, Clone)]
pub struct PreloadConfig {
    /// 预加载策略
    pub strategy: PreloadStrategy,
    /// 最大并发预加载数
    pub max_concurrent: usize,
    /// 预加载距离阈值（用于DistanceBased策略）
    pub distance_threshold: f32,
    /// 是否预加载依赖资源
    pub preload_dependencies: bool,
    /// 预加载超时时间（秒）
    pub timeout_seconds: f32,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            strategy: PreloadStrategy::Background,
            max_concurrent: 4,
            distance_threshold: 100.0,
            preload_dependencies: true,
            timeout_seconds: 30.0,
        }
    }
}

/// 预加载请求
#[derive(Debug, Clone)]
pub struct PreloadRequest {
    /// 资源路径
    pub path: PathBuf,
    /// 资源类型
    pub asset_type: AssetType,
    /// 优先级
    pub priority: LoadPriority,
    /// 预加载策略
    pub strategy: PreloadStrategy,
    /// 请求时间
    pub requested_at: Instant,
}

/// 预加载状态
#[derive(Debug, Clone)]
pub struct PreloadStatus {
    /// 资源路径
    pub path: PathBuf,
    /// 加载状态
    pub state: PreloadState,
    /// 进度（0.0-1.0）
    pub progress: f32,
    /// 开始时间
    pub started_at: Option<Instant>,
    /// 完成时间
    pub completed_at: Option<Instant>,
}

/// 预加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadState {
    /// 排队中
    Queued,
    /// 加载中
    Loading,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 预加载统计
#[derive(Debug, Clone, Default)]
pub struct PreloadStats {
    /// 总预加载请求数
    pub total_requests: u32,
    /// 已完成数
    pub completed: u32,
    /// 失败数
    pub failed: u32,
    /// 取消数
    pub cancelled: u32,
    /// 平均加载时间（秒）
    pub avg_load_time: f32,
    /// 总预加载时间（秒）
    pub total_load_time: f32,
}

/// 预加载管理器
pub struct PreloadManager {
    /// 配置
    config: PreloadConfig,
    /// 依赖图
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// 预加载请求队列
    request_queue: Arc<RwLock<Vec<PreloadRequest>>>,
    /// 预加载状态映射
    status_map: Arc<RwLock<HashMap<PathBuf, PreloadStatus>>>,
    /// 正在加载的资源集合
    loading_set: Arc<RwLock<HashSet<PathBuf>>>,
    /// 统计信息
    stats: Arc<RwLock<PreloadStats>>,
}

impl PreloadManager {
    /// 创建新的预加载管理器
    pub fn new(config: PreloadConfig) -> Self {
        Self {
            config,
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            request_queue: Arc::new(RwLock::new(Vec::new())),
            status_map: Arc::new(RwLock::new(HashMap::new())),
            loading_set: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(PreloadStats::default())),
        }
    }

    /// 添加预加载请求
    ///
    /// # 参数
    ///
    /// * `path` - 资源路径
    /// * `asset_type` - 资源类型
    /// * `priority` - 加载优先级
    /// * `strategy` - 预加载策略
    pub fn request_preload(
        &self,
        path: PathBuf,
        asset_type: AssetType,
        priority: LoadPriority,
        strategy: PreloadStrategy,
    ) {
        let request = PreloadRequest {
            path: path.clone(),
            asset_type,
            priority,
            strategy,
            requested_at: Instant::now(),
        };

        // 添加到请求队列
        {
            let mut queue = self.request_queue.write().unwrap();
            queue.push(request);
        }

        // 更新状态
        {
            let mut status_map = self.status_map.write().unwrap();
            status_map.insert(
                path.clone(),
                PreloadStatus {
                    path: path.clone(),
                    state: PreloadState::Queued,
                    progress: 0.0,
                    started_at: None,
                    completed_at: None,
                },
            );
        }

        // 更新统计
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_requests += 1;
        }
    }

    /// 批量添加预加载请求
    pub fn request_preload_batch(&self, requests: Vec<(PathBuf, AssetType, LoadPriority, PreloadStrategy)>) {
        for (path, asset_type, priority, strategy) in requests {
            self.request_preload(path, asset_type, priority, strategy);
        }
    }

    /// 基于场景预加载
    ///
    /// 预加载场景所需的所有资源。
    ///
    /// # 参数
    ///
    /// * `scene_resources` - 场景资源列表（路径，类型）
    pub fn preload_scene(&self, scene_resources: Vec<(PathBuf, AssetType)>) {
        for (path, asset_type) in scene_resources {
            self.request_preload(
                path,
                asset_type,
                LoadPriority::High,
                PreloadStrategy::SceneBased,
            );
        }
    }

    /// 基于距离预加载
    ///
    /// 根据距离阈值预加载资源。
    ///
    /// # 参数
    ///
    /// * `resources` - 资源列表（路径，类型，距离）
    pub fn preload_by_distance(&self, resources: Vec<(PathBuf, AssetType, f32)>) {
        for (path, asset_type, distance) in resources {
            if distance <= self.config.distance_threshold {
                let priority = if distance < self.config.distance_threshold * 0.5 {
                    LoadPriority::High
                } else {
                    LoadPriority::Normal
                };
                self.request_preload(path, asset_type, priority, PreloadStrategy::DistanceBased);
            }
        }
    }

    /// 更新预加载管理器
    ///
    /// 处理预加载队列，启动新的加载任务。
    ///
    /// # 参数
    ///
    /// * `loader` - 资源加载器
    pub fn update(&self, loader: &mut CoroutineAssetLoader) {
        let mut queue = self.request_queue.write().unwrap();
        let mut loading_set = self.loading_set.write().unwrap();
        let mut status_map = self.status_map.write().unwrap();

        // 按优先级和策略排序
        queue.sort_by(|a, b| {
            match a.strategy.cmp(&b.strategy) {
                Ordering::Equal => a.priority.cmp(&b.priority),
                other => other,
            }
        });

        // 启动新的加载任务（不超过最大并发数）
        while loading_set.len() < self.config.max_concurrent {
            if let Some(request) = queue.pop() {
                // 检查是否已经在加载
                if loading_set.contains(&request.path) {
                    continue;
                }

                // 检查依赖是否已加载（如果启用）
                if self.config.preload_dependencies {
                    let graph = self.dependency_graph.read().unwrap();
                    if !graph.can_load(&request.path) {
                        // 依赖未就绪，放回队列
                        queue.push(request);
                        continue;
                    }
                }

                // 标记为加载中
                loading_set.insert(request.path.clone());
                if let Some(status) = status_map.get_mut(&request.path) {
                    status.state = PreloadState::Loading;
                    status.started_at = Some(Instant::now());
                }

                // 启动加载（这里简化处理，实际应该使用异步加载）
                // 注意：实际实现中应该使用loader的异步加载API
            } else {
                break;
            }
        }
    }

    /// 标记预加载完成
    pub fn mark_completed(&self, path: &PathBuf, success: bool) {
        let mut loading_set = self.loading_set.write().unwrap();
        let mut status_map = self.status_map.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        loading_set.remove(path);

        if let Some(status) = status_map.get_mut(path) {
            status.completed_at = Some(Instant::now());
            status.progress = 1.0;
            
            if success {
                status.state = PreloadState::Completed;
                stats.completed += 1;
                
                // 计算加载时间
                if let Some(started_at) = status.started_at {
                    let load_time = status.completed_at.unwrap().duration_since(started_at).as_secs_f32();
                    stats.total_load_time += load_time;
                    stats.avg_load_time = stats.total_load_time / stats.completed as f32;
                }
            } else {
                status.state = PreloadState::Failed;
                stats.failed += 1;
            }
        }

        // 更新依赖图状态
        {
            let mut graph = self.dependency_graph.write().unwrap();
            graph.set_load_state(
                path,
                if success {
                    crate::resources::dependency_manager::LoadState::Loaded
                } else {
                    crate::resources::dependency_manager::LoadState::Failed
                },
            );
        }
    }

    /// 取消预加载
    pub fn cancel_preload(&self, path: &PathBuf) {
        let mut loading_set = self.loading_set.write().unwrap();
        let mut status_map = self.status_map.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        loading_set.remove(path);

        if let Some(status) = status_map.get_mut(path) {
            status.state = PreloadState::Cancelled;
            stats.cancelled += 1;
        }
    }

    /// 获取预加载状态
    pub fn get_status(&self, path: &PathBuf) -> Option<PreloadStatus> {
        self.status_map.read().unwrap().get(path).cloned()
    }

    /// 获取统计信息
    pub fn stats(&self) -> PreloadStats {
        self.stats.read().unwrap().clone()
    }

    /// 获取依赖图（用于外部添加依赖关系）
    pub fn dependency_graph(&self) -> Arc<RwLock<DependencyGraph>> {
        Arc::clone(&self.dependency_graph)
    }

    /// 清除所有预加载请求
    pub fn clear(&self) {
        self.request_queue.write().unwrap().clear();
        self.status_map.write().unwrap().clear();
        self.loading_set.write().unwrap().clear();
    }
}

impl Default for PreloadManager {
    fn default() -> Self {
        Self::new(PreloadConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_request() {
        let manager = PreloadManager::new(PreloadConfig::default());
        
        manager.request_preload(
            PathBuf::from("texture.png"),
            AssetType::Texture,
            LoadPriority::High,
            PreloadStrategy::Immediate,
        );

        let stats = manager.stats();
        assert_eq!(stats.total_requests, 1);
    }

    #[test]
    fn test_preload_scene() {
        let manager = PreloadManager::new(PreloadConfig::default());
        
        let scene_resources = vec![
            (PathBuf::from("texture1.png"), AssetType::Texture),
            (PathBuf::from("texture2.png"), AssetType::Texture),
        ];

        manager.preload_scene(scene_resources);

        let stats = manager.stats();
        assert_eq!(stats.total_requests, 2);
    }

    #[test]
    fn test_preload_by_distance() {
        let mut config = PreloadConfig::default();
        config.distance_threshold = 100.0;
        let manager = PreloadManager::new(config);
        
        let resources = vec![
            (PathBuf::from("near.png"), AssetType::Texture, 50.0),
            (PathBuf::from("far.png"), AssetType::Texture, 150.0),
        ];

        manager.preload_by_distance(resources);

        let stats = manager.stats();
        assert_eq!(stats.total_requests, 1); // 只有near.png在阈值内
    }
}

