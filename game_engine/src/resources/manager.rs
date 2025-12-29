use bevy_ecs::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};
// use crossbeam_channel:: {unbounded, Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
// 移除未使用的AsyncReadExt导入，如果将来需要异步读取，可以重新导入
// use futures::future::FutureExt;
use super::atlas::Atlas;
use crate::render::wgpu_utils::WgpuRenderer;
// use super::runtime::global_runtime;
use std::collections::HashMap;

// --- GLTF Support ---
// GLTF-specific code has been extracted to gltf_assets.rs
// Consolidated GLTF imports with single cfg block
#[cfg(feature = "gltf")]
pub use super::gltf_assets::{GltfAssetLoader, import_gltf_to_world};
#[cfg(feature = "gltf")]
pub use super::gltf_loader::GltfScene;

// --- Handle System ---

#[derive(Clone, Debug)]
pub enum LoadState<T> {
    Loading,
    Loaded(T),
    Failed(String),
}

#[derive(Debug)]
pub struct AssetContainer<T> {
    pub state: RwLock<LoadState<T>>,
}

#[derive(Clone, Component, Debug)]
pub struct Handle<T: 'static + Send + Sync> {
    pub container: Arc<AssetContainer<T>>,
}

impl<T: 'static + Send + Sync> Handle<T> {
    pub fn new_loading() -> Self {
        Self {
            container: Arc::new(AssetContainer {
                state: RwLock::new(LoadState::Loading),
            }),
        }
    }

    /// Create a Handle from an Arc<AssetContainer> (for internal use)
    fn from_container(container: Arc<AssetContainer<T>>) -> Self {
        Self { container }
    }

    /// 获取资源（优化版本：如果T是Arc<_>，避免额外克隆）
    ///
    /// 优化：当T = Arc<U>时，直接克隆Arc指针而不是克隆内部数据
    /// 这对于Handle<Arc<GpuMesh>>等常见用法可以显著减少开销
    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.container
            .state
            .read()
            .ok() // ✅ 处理锁中毒情况
            .and_then(|state| match &*state {
                LoadState::Loaded(v) => {
                    // 如果T已经是Arc<_>，克隆Arc只是增加引用计数，非常快
                    // 这比克隆内部数据（如GpuMesh）要快得多
                    Some(v.clone())
                }
                _ => None,
            })
    }

    pub fn is_loaded(&self) -> bool {
        self.container
            .state
            .read()
            .ok() // ✅ 处理锁中毒情况
            .map(|state| matches!(*state, LoadState::Loaded(_)))
            .unwrap_or(false) // ✅ 锁中毒时返回false
    }

    // Removed get_ref method as it couldn't be implemented safely due to lifetime issues

    /// 非阻塞方式获取资源，立即返回结果
    pub fn get_non_blocking(&self) -> Option<T>
    where
        T: Clone,
    {
        self.container
            .state
            .try_read()
            .ok() // ✅ 处理锁中毒情况
            .and_then(|state| match &*state {
                LoadState::Loaded(v) => Some(v.clone()),
                _ => None,
            })
    }

    /// 非阻塞方式获取加载状态
    pub fn get_state_non_blocking(&self) -> Option<LoadState<T>>
    where
        T: Clone,
    {
        self.container.state.try_read().ok().map(|state| match &*state {
            LoadState::Loaded(v) => LoadState::Loaded(v.clone()),
            LoadState::Failed(e) => LoadState::Failed(e.clone()),
            LoadState::Loading => LoadState::Loading,
        })
    }

    /// 带超时的资源获取，在指定时间内尝试获取资源
    pub fn get_with_timeout(&self, timeout: Duration) -> Option<T>
    where
        T: Clone,
    {
        let start = std::time::Instant::now();

        // 先尝试直接获取，也许正好资源已准备好
        if let Some(result) = self.get_non_blocking() {
            return Some(result);
        }

        // 自适应等待策略：初始短等待，逐渐增加等待时间
        let mut wait_time = Duration::from_micros(100); // 初始100微秒
        let max_wait_time = Duration::from_millis(10); // 最大10毫秒

        while start.elapsed() < timeout {
            std::thread::sleep(wait_time);

            if let Some(result) = self.get_non_blocking() {
                return Some(result);
            }

            // 指数退避，但不超过最大等待时间
            wait_time = (wait_time * 2).min(max_wait_time);
        }

        // 超时后最后一次尝试
        self.get_non_blocking()
    }

    /// 阻塞等待资源加载完成（注意：这会阻塞当前线程）
    pub fn get_blocking(&self) -> Option<T>
    where
        T: Clone,
    {
        loop {
            match self.container.state.read() {
                Ok(state) => match &*state {
                    LoadState::Loaded(v) => return Some(v.clone()),
                    LoadState::Failed(_) => return None,
                    LoadState::Loading => {} // 继续等待
                },
                Err(_) => {
                    // 锁中毒，返回None但不panic
                    return None;
                }
            }
            // 短暂休眠避免CPU占用过高
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// 获取资源状态信息（安全的元数据获取）
    pub fn get_status(&self) -> Result<String, &'static str> {
        self.container
            .state
            .read()
            .map_err(|_| "Lock poisoned")
            .map(|state| match &*state {
                LoadState::Loading => "loading".to_string(),
                LoadState::Loaded(_) => "loaded".to_string(),
                LoadState::Failed(err) => format!("failed: {}", err),
            })
    }
}

// --- Asset Server ---

enum AssetTask {
    Texture {
        path: PathBuf,
        handle: Arc<AssetContainer<u32>>,  // Use Arc directly to avoid Handle cloning
        is_linear: bool,
        start: std::time::Instant,
    },
    Atlas {
        path: PathBuf,
        handle: Arc<AssetContainer<Atlas>>,  // Use Arc directly
        start: std::time::Instant,
    },
    #[cfg(feature = "gltf")]
    Gltf {
        path: PathBuf,
        handle: Arc<AssetContainer<GltfScene>>,  // Use Arc directly
        start: std::time::Instant,
    },
}

pub enum AssetResult {
    Bytes(Vec<u8>),
    Image(image::RgbaImage),
    #[cfg(feature = "gltf")]
    Gltf(GltfScene),
}

/// 资源统计信息
#[derive(Debug, Default, Clone)]
pub struct AssetStats {
    /// 已加载的纹理数量
    pub loaded_textures: usize,
    /// 已加载的图集数量
    pub loaded_atlases: usize,
    /// 已加载的GLTF场景数量
    #[cfg(feature = "gltf")]
    pub loaded_gltf_scenes: usize,
    /// 失败的纹理加载次数
    pub failed_textures: usize,
    /// 失败的图集加载次数
    pub failed_atlases: usize,
    /// 总内存使用（字节）
    pub total_memory_bytes: usize,
    /// 平均加载时间（毫秒）
    pub average_load_time_ms: f64,
}

#[derive(Resource)]
pub struct AssetServer {
    tx: mpsc::UnboundedSender<AssetTask>,
    rx: mpsc::UnboundedReceiver<(AssetTask, Result<AssetResult, String>)>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// 已加载纹理数量统计（原子操作，用于快速查询）
    texture_count: std::sync::atomic::AtomicUsize,
    /// 资源统计信息（详细统计）
    stats: std::sync::RwLock<AssetStats>,
}

#[derive(Clone, Debug)]
pub enum AssetEvent {
    TextureLoaded(Handle<u32>, f32),
    AtlasLoaded(Handle<Atlas>, f32),
    #[cfg(feature = "gltf")]
    GltfLoaded(Handle<GltfScene>, f32),
    TextureFailed(Handle<u32>, String),
    AtlasFailed(Handle<Atlas>, String),
    #[cfg(feature = "gltf")]
    GltfFailed(Handle<GltfScene>, String),
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    /// Helper method to wait for asset loading with timeout (reduces code duplication)
    async fn wait_for_load<T>(&self, handle: &Handle<T>) -> Result<Handle<T>, String>
    where
        T: Clone + Send + Sync,
    {
        let mut timeout_counter = 0;
        const MAX_TIMEOUT: u32 = 1000;

        while timeout_counter < MAX_TIMEOUT {
            if let Some(result) = handle.get_state_non_blocking() {
                return match result {
                    LoadState::Loaded(_) => Ok(handle.clone()),
                    LoadState::Failed(e) => Err(e),
                    LoadState::Loading => {
                        timeout_counter += 1;
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        continue;
                    }
                };
            }
            timeout_counter += 1;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        Err("Timeout waiting for asset to load".to_string())
    }

    pub fn new() -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel::<AssetTask>();
        let (done_tx, done_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let worker_handle = std::thread::Builder::new()
            .name("asset-loader".to_string())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("Failed to create asset loader runtime: {}", e);
                        // 如果无法创建runtime，线程将退出，AssetServer将无法工作
                        // 这是一个严重的初始化错误，应该被上层代码检测到
                        return;
                    }
                };

                rt.block_on(async move {
                    let mut shutdown_rx = shutdown_rx;
                    let mut task_rx = task_rx;

                    loop {
                        tokio::select! {
                            _ = &mut shutdown_rx => {
                                log::info!("Asset loader received shutdown signal");
                                break;
                            }
                            task = task_rx.recv() => {
                                match task {
                                    Some(task) => {
                                        let tx = done_tx.clone();
                                        tokio::spawn(async move {
                                            let result = match &task {
                                                AssetTask::Texture { path, .. } => {
                                                    match tokio::fs::read(path).await {
                                                        Ok(bytes) => {
                                                            // Decode in blocking task
                                                            let decode_res = tokio::task::spawn_blocking(move || {
                                                                image::load_from_memory(&bytes)
                                                                    .map(|img| AssetResult::Image(img.to_rgba8()))
                                                                    .map_err(|e| e.to_string())
                                                            }).await;

                                                            match decode_res {
                                                                Ok(res) => res,
                                                                Err(e) => Err(e.to_string()),
                                                            }
                                                        },
                                                        Err(e) => Err(e.to_string()),
                                                    }
                                                },
                                                AssetTask::Atlas { path, .. } => {
                                                    tokio::fs::read(path).await
                                                        .map(AssetResult::Bytes)
                                                        .map_err(|e| e.to_string())
                                                },
                                                #[cfg(feature = "gltf")]
                                                AssetTask::Gltf { path, .. } => {
                                                    match tokio::fs::read(path).await {
                                                        Ok(bytes) => {
                                                            super::gltf_assets::GltfAssetLoader::load_from_bytes(bytes).await
                                                                .map(AssetResult::Gltf)
                                                        },
                                                        Err(e) => Err(e.to_string()),
                                                    }
                                                },
                                            };

                                            let _ = tx.send((task, result));
                                        });
                                    }
                                    None => {
                                        log::info!("Asset task channel closed");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });
            })
            .unwrap_or_else(|e| {
                // 如果无法创建线程，这是一个严重的初始化错误
                // 记录错误并panic，因为AssetServer无法在没有工作线程的情况下工作
                log::error!("Failed to spawn asset loader thread: {}", e);
                panic!("Failed to spawn asset loader thread: {}", e);
            });

        Self {
            tx: task_tx,
            rx: done_rx,
            worker_handle: Some(worker_handle),
            shutdown_tx: Some(shutdown_tx),
            texture_count: std::sync::atomic::AtomicUsize::new(0),
            stats: std::sync::RwLock::new(AssetStats::default()),
        }
    }

    /// 异步加载纹理
    pub async fn load_texture_async(&self, path: &Path) -> Result<Handle<u32>, String> {
        let _load_span =
            crate::performance::tracing_metrics::TracingMetricsManager::asset_load_span(
                &path.display().to_string(),
                "texture",
            )
            .entered();

        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Texture {
            path: path.to_path_buf(),
            handle: container,
            is_linear: false,
            start: std::time::Instant::now(),
        };

        // 发送任务并等待结果
        let _ = self.tx.send(task);
        self.wait_for_load(&handle).await
    }

    /// 异步加载线性纹理
    pub async fn load_texture_linear_async(&self, path: &Path) -> Result<Handle<u32>, String> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Texture {
            path: path.to_path_buf(),
            handle: container,
            is_linear: true,
            start: std::time::Instant::now(),
        };

        // 发送任务并等待结果
        let _ = self.tx.send(task);
        self.wait_for_load(&handle).await
    }

    /// 异步加载图集
    pub async fn load_atlas_async(&self, path: &Path) -> Result<Handle<Atlas>, String> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Atlas {
            path: path.to_path_buf(),
            handle: container,
            start: std::time::Instant::now(),
        };

        // 发送任务并等待结果
        let _ = self.tx.send(task);
        self.wait_for_load(&handle).await
    }

    #[cfg(feature = "gltf")]
    /// 异步加载GLTF场景
    pub async fn load_gltf_async(&self, path: &Path) -> Result<Handle<GltfScene>, String> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Gltf {
            path: path.to_path_buf(),
            handle: container,
            start: std::time::Instant::now(),
        };

        // 发送任务并等待结果
        let _ = self.tx.send(task);
        self.wait_for_load(&handle).await
    }

    pub fn load_texture(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Texture {
            path: path.to_path_buf(),
            handle: container,
            is_linear: false,
            start: std::time::Instant::now(),
        };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_texture_linear(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Texture {
            path: path.to_path_buf(),
            handle: container,
            is_linear: true,
            start: std::time::Instant::now(),
        };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_atlas(&self, path: &Path) -> Handle<Atlas> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Atlas {
            path: path.to_path_buf(),
            handle: container,
            start: std::time::Instant::now(),
        };
        let _ = self.tx.send(task);
        handle
    }

    #[cfg(feature = "gltf")]
    pub fn load_gltf(&self, path: &Path) -> Handle<GltfScene> {
        let handle = Handle::new_loading();
        let container = handle.container.clone();  // Single Arc clone
        let task = AssetTask::Gltf {
            path: path.to_path_buf(),
            handle: container,
            start: std::time::Instant::now(),
        };
        let _ = self.tx.send(task);
        handle
    }

    // This must be called in the main thread loop
    pub fn update(&mut self, renderer: &mut WgpuRenderer) -> Vec<AssetEvent> {
        let mut events = Vec::new();
        while let Ok((task, result)) = self.rx.try_recv() {
            match (task, result) {
                (
                    AssetTask::Texture {
                        handle,
                        is_linear,
                        start,
                        ..
                    },
                    Ok(AssetResult::Image(img)),
                ) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f64() * 1000.0;
                    if let Some(tex_id) = renderer.load_texture_from_image(img.clone(), is_linear) {
                        if let Ok(mut state) = handle.state.write() {
                            *state = LoadState::Loaded(tex_id);
                        } // ✅ 处理锁中毒情况，忽略更新失败

                        // 更新统计信息
                        self.texture_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if let Ok(mut stats) = self.stats.write() {
                            stats.loaded_textures += 1;
                            stats.total_memory_bytes += img.len() * 4; // RGBA = 4 bytes per pixel
                            // 更新平均加载时间
                            let total_loads = stats.loaded_textures + stats.failed_textures;
                            if total_loads > 0 {
                                stats.average_load_time_ms =
                                    (stats.average_load_time_ms * (total_loads - 1) as f64 + ms)
                                        / total_loads as f64;
                            } else {
                                stats.average_load_time_ms = ms;
                            }
                        }

                        let handle = Handle::from_container(handle.clone());
                        events.push(AssetEvent::TextureLoaded(handle, ms as f32));
                    } else {
                        if let Ok(mut state) = handle.state.write() {
                            *state = LoadState::Failed("Failed to create texture".to_string());
                        } // ✅ 处理锁中毒情况，忽略更新失败

                        // 更新失败统计
                        if let Ok(mut stats) = self.stats.write() {
                            stats.failed_textures += 1;
                        }

                        let handle = Handle::from_container(handle.clone());
                        events.push(AssetEvent::TextureFailed(
                            handle,
                            "Failed to create texture".to_string(),
                        ));
                    }
                }
                (AssetTask::Atlas { handle, start, .. }, Ok(AssetResult::Bytes(bytes))) => {
                    let bytes_len = bytes.len();
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f64() * 1000.0;
                    if let Ok(json_str) = String::from_utf8(bytes) {
                        if let Some(atlas) = Atlas::from_json(&json_str) {
                            if let Ok(mut state) = handle.state.write() {
                                *state = LoadState::Loaded(atlas);
                            } // ✅ 处理锁中毒情况，忽略更新失败

                            // 更新统计信息
                            if let Ok(mut stats) = self.stats.write() {
                                stats.loaded_atlases += 1;
                                stats.total_memory_bytes += bytes_len;
                            }

                            let handle = Handle::from_container(handle.clone());
                            events.push(AssetEvent::AtlasLoaded(handle, ms as f32));
                        } else {
                            if let Ok(mut state) = handle.state.write() {
                                *state = LoadState::Failed("Invalid Atlas JSON".to_string());
                            } // ✅ 处理锁中毒情况，忽略更新失败

                            // 更新统计信息
                            if let Ok(mut stats) = self.stats.write() {
                                stats.failed_atlases += 1;
                            }

                            let handle = Handle::from_container(handle.clone());
                            events.push(AssetEvent::AtlasFailed(
                                handle,
                                "Invalid Atlas JSON".to_string(),
                            ));
                        }
                    } else {
                        if let Ok(mut state) = handle.state.write() {
                            *state = LoadState::Failed("Invalid UTF-8".to_string());
                        } // ✅ 处理锁中毒情况，忽略更新失败
                        let handle = Handle::from_container(handle.clone());
                        events.push(AssetEvent::AtlasFailed(
                            handle,
                            "Invalid UTF-8".to_string(),
                        ));
                    }
                }
                #[cfg(feature = "gltf")]
                (AssetTask::Gltf { handle, start, .. }, Ok(AssetResult::Gltf(scene))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f64() * 1000.0;
                    if let Ok(mut state) = handle.state.write() {
                        *state = LoadState::Loaded(scene);
                    } // ✅ 处理锁中毒情况，忽略更新失败

                    // 更新统计信息
                    if let Ok(mut stats) = self.stats.write() {
                        stats.loaded_gltf_scenes += 1;
                    }

                    let handle = Handle::from_container(handle.clone());
                    events.push(AssetEvent::GltfLoaded(handle, ms as f32));
                }
                (AssetTask::Texture { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.state.write() {
                        *state = LoadState::Failed(e.clone());
                    } // ✅ 处理锁中毒情况，忽略更新失败

                    // 更新失败统计
                    if let Ok(mut stats) = self.stats.write() {
                        stats.failed_textures += 1;
                    }

                    let handle = Handle::from_container(handle.clone());
                    events.push(AssetEvent::TextureFailed(handle, e));
                }
                (AssetTask::Atlas { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.state.write() {
                        *state = LoadState::Failed(e.clone());
                    } // ✅ 处理锁中毒情况，忽略更新失败

                    // 更新失败统计
                    if let Ok(mut stats) = self.stats.write() {
                        stats.failed_atlases += 1;
                    }

                    let handle = Handle::from_container(handle.clone());
                    events.push(AssetEvent::AtlasFailed(handle, e));
                }
                #[cfg(feature = "gltf")]
                (AssetTask::Gltf { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.state.write() {
                        *state = LoadState::Failed(e.clone());
                    } // ✅ 处理锁中毒情况，忽略更新失败
                    let handle = Handle::from_container(handle.clone());
                    events.push(AssetEvent::GltfFailed(handle, e));
                }
                _ => {}
            }
        }
        events
    }

    // Helper for legacy code compatibility
    pub fn atlas_region(
        &self,
        atlas_handle: &Handle<Atlas>,
        sprite_name: &str,
    ) -> Option<([f32; 2], [f32; 2])> {
        if let Some(atlas) = atlas_handle.get() {
            return atlas.get(sprite_name);
        }
        None
    }

    /// 检查资产服务器是否空闲，即没有正在进行的资产加载任务
    pub fn is_idle(&self) -> bool {
        // 检查接收通道中是否还有未处理的任务结果
        // 如果通道为空，则表示所有任务都已完成
        self.rx.is_empty()
    }

    /// 清除资产缓存
    pub fn clear_cache(&mut self) {
        // 目前这个方法是空的，将来可以实现缓存清理逻辑
        log::info!("Asset cache cleared");
    }

    /// 获取已加载纹理数量（用于完整性检查）
    ///
    /// 返回当前已成功加载的纹理数量。
    ///
    /// # 返回
    ///
    /// 已加载的纹理数量
    pub fn get_loaded_texture_count(&self) -> usize {
        self.texture_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取资源统计信息
    ///
    /// 返回详细的资源统计信息，包括加载数量、内存使用等。
    ///
    /// # 返回
    ///
    /// 资源统计信息的副本
    pub fn get_stats(&self) -> AssetStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// 重置统计信息
    ///
    /// 将所有统计计数器重置为0。
    pub fn reset_stats(&self) {
        self.texture_count.store(0, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut stats) = self.stats.write() {
            *stats = AssetStats::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_stats_default() {
        let stats = AssetStats::default();
        assert_eq!(stats.loaded_textures, 0);
        assert_eq!(stats.loaded_atlases, 0);
        assert_eq!(stats.failed_textures, 0);
        assert_eq!(stats.failed_atlases, 0);
        assert_eq!(stats.total_memory_bytes, 0);
        assert_eq!(stats.average_load_time_ms, 0.0);
    }

    #[test]
    fn test_asset_server_creation() {
        let server = AssetServer::new();
        assert_eq!(server.get_loaded_texture_count(), 0);

        let stats = server.get_stats();
        assert_eq!(stats.loaded_textures, 0);
    }

    #[test]
    fn test_asset_stats_clone() {
        let mut stats = AssetStats::default();
        stats.loaded_textures = 5;
        stats.total_memory_bytes = 1024;

        let cloned = stats.clone();
        assert_eq!(cloned.loaded_textures, 5);
        assert_eq!(cloned.total_memory_bytes, 1024);
    }

    #[test]
    fn test_asset_server_reset_stats() {
        let server = AssetServer::new();
        server.reset_stats();

        let stats = server.get_stats();
        assert_eq!(stats.loaded_textures, 0);
        assert_eq!(server.get_loaded_texture_count(), 0);
    }
}

impl Drop for AssetServer {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take()
            && let Err(e) = handle.join()
        {
            log::error!("Asset loader thread panicked: {:?}", e);
        }
    }
}

// Re-export from gltf_assets module
#[cfg(feature = "gltf")]
pub use super::gltf_assets::to_rgba;

pub fn generate_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut tangents = vec![[0.0f32; 4]; positions.len()];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let p0 = glam::Vec3::from_array(positions[i0]);
        let p1 = glam::Vec3::from_array(positions[i1]);
        let p2 = glam::Vec3::from_array(positions[i2]);
        let uv0 = glam::Vec2::from_array(uvs[i0]);
        let uv1 = glam::Vec2::from_array(uvs[i1]);
        let uv2 = glam::Vec2::from_array(uvs[i2]);
        let dp1 = p1 - p0;
        let dp2 = p2 - p0;
        let duv1 = uv1 - uv0;
        let duv2 = uv2 - uv0;
        let r = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x);
        let t = (dp1 * duv2.y - dp2 * duv1.y) * r;
        let n0 = glam::Vec3::from_array(normals[i0]);
        let t0 = (t - n0 * n0.dot(t)).normalize_or_zero();
        tangents[i0] = [t0.x, t0.y, t0.z, 1.0];
        tangents[i1] = tangents[i0];
        tangents[i2] = tangents[i0];
    }
    tangents
}
#[derive(Resource, Default)]
pub struct MaterialRegistry {
    pub materials: HashMap<
        u64,
        (
            std::sync::Arc<wgpu::BindGroup>, // material uniform BG
            std::sync::Arc<wgpu::Buffer>,    // material uniform buffer
            std::sync::Arc<wgpu::BindGroup>, // textures BG
        ),
    >,
}

#[derive(Resource, Default)]
pub struct MaterialPendingUpdates {
    pub params: Vec<(u64, crate::render::pbr::PbrMaterial)>,
}

impl MaterialPendingUpdates {
    pub fn push(&mut self, id: u64, mat: crate::render::pbr::PbrMaterial) {
        self.params.push((id, mat));
    }
    pub fn take_all(&mut self) -> Vec<(u64, crate::render::pbr::PbrMaterial)> {
        std::mem::take(&mut self.params)
    }
}

impl MaterialRegistry {
    pub fn update_material_params(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr: &crate::render::pbr_renderer::PbrRenderer,
        mat_id: u64,
        mat: &crate::render::pbr::PbrMaterial,
    ) -> bool {
        if let Some((bg, buf, tex)) = self.materials.get_mut(&mat_id) {
            let uniform = crate::render::pbr_renderer::PbrRenderer::encode_material_uniform(mat);
            queue.write_buffer(buf, 0, bytemuck::bytes_of(&uniform));
            // bg和tex用于材质绑定，在渲染时使用
            let _bg_ref = bg;
            let _tex_ref = tex;
            // bind group布局不变，无需重建
            true
        } else {
            // 创建并登记
            let (new_bg, new_buf) = pbr.create_material_bind_group(device, queue, mat);
            let new_tex = wgpu_dummy_bg(device, &pbr.textures_bgl);
            self.materials
                .insert(mat_id, (new_bg.clone(), new_buf.clone(), new_tex.clone()));
            true
        }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr: &crate::render::pbr_renderer::PbrRenderer,
        mat_id: u64,
        images: [image::RgbaImage; 5],
        srgb: [bool; 5],
    ) -> bool {
        let tex_set = pbr.create_texture_set_from_images(device, queue, images, srgb);
        let tex_bg = std::sync::Arc::new(tex_set.bind_group);
        if let Some(entry) = self.materials.get_mut(&mat_id) {
            let (_, _, old_tex_bg) = entry;
            *old_tex_bg = tex_bg;
            true
        } else {
            false
        }
    }
}

fn wgpu_dummy_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
) -> std::sync::Arc<wgpu::BindGroup> {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("DummyTex"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    std::sync::Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("DummyTexBG"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    }))
}
