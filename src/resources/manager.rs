use std::{path::{Path, PathBuf}, sync::{Arc, RwLock}};
use bevy_ecs::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::render::wgpu::WgpuRenderer;
use super::atlas::Atlas;
use super::runtime::global_runtime;

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

    pub fn get(&self) -> Option<T> where T: Clone {
        match &*self.container.state.read().unwrap() {
            LoadState::Loaded(v) => Some(v.clone()),
            _ => None,
        }
    }
    
    pub fn is_loaded(&self) -> bool {
        matches!(*self.container.state.read().unwrap(), LoadState::Loaded(_))
    }
}

// --- Asset Server ---

enum AssetTask {
    Texture { path: PathBuf, handle: Handle<u32>, is_linear: bool, start: std::time::Instant },
    Atlas { path: PathBuf, handle: Handle<Atlas>, start: std::time::Instant },
}

pub enum AssetResult {
    Bytes(Vec<u8>),
    Image(image::RgbaImage),
}

#[derive(Resource)]
pub struct AssetServer {
    tx: Sender<AssetTask>,
    rx: Receiver<(AssetTask, Result<AssetResult, String>)>,
}

#[derive(Clone, Debug)]
pub enum AssetEvent {
    TextureLoaded(Handle<u32>, f32),
    AtlasLoaded(Handle<Atlas>, f32),
    TextureFailed(Handle<u32>, String),
    AtlasFailed(Handle<Atlas>, String),
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    pub fn new() -> Self {
        let (task_tx, task_rx) = unbounded::<AssetTask>();
        let (done_tx, done_rx) = unbounded();

        // 使用全局运行时处理异步IO任务
        let tx_clone = done_tx.clone();
        std::thread::spawn(move || {
            let rt = global_runtime();
            
            rt.block_on(async move {
                while let Ok(task) = task_rx.recv() {
                    let tx = tx_clone.clone();
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
                        };
                        
                        let _ = tx.send((task, result));
                    });
                }
            });
        });

        Self {
            tx: task_tx,
            rx: done_rx,
        }
    }

    pub fn load_texture(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let task = AssetTask::Texture { path: path.to_path_buf(), handle: handle.clone(), is_linear: false, start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_texture_linear(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let task = AssetTask::Texture { path: path.to_path_buf(), handle: handle.clone(), is_linear: true, start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_atlas(&self, path: &Path) -> Handle<Atlas> {
        let handle = Handle::new_loading();
        let task = AssetTask::Atlas { path: path.to_path_buf(), handle: handle.clone(), start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    // This must be called in the main thread loop
    pub fn update(&self, renderer: &mut WgpuRenderer) -> Vec<AssetEvent> {
        let mut events = Vec::new();
        while let Ok((task, result)) = self.rx.try_recv() {
            match (task, result) {
                (AssetTask::Texture { handle, is_linear, start, .. }, Ok(AssetResult::Image(img))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f32() * 1000.0;
                    if let Some(tex_id) = renderer.load_texture_from_image(img, is_linear) {
                         *handle.container.state.write().unwrap() = LoadState::Loaded(tex_id);
                         events.push(AssetEvent::TextureLoaded(handle.clone(), ms));
                    } else {
                         *handle.container.state.write().unwrap() = LoadState::Failed("Failed to create texture".to_string());
                         events.push(AssetEvent::TextureFailed(handle.clone(), "Failed to create texture".to_string()));
                    }
                },
                (AssetTask::Atlas { handle, start, .. }, Ok(AssetResult::Bytes(bytes))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f32() * 1000.0;
                    if let Ok(json_str) = String::from_utf8(bytes) {
                        if let Some(atlas) = Atlas::from_json(&json_str) {
                            *handle.container.state.write().unwrap() = LoadState::Loaded(atlas);
                            events.push(AssetEvent::AtlasLoaded(handle.clone(), ms));
                        } else {
                            *handle.container.state.write().unwrap() = LoadState::Failed("Invalid Atlas JSON".to_string());
                            events.push(AssetEvent::AtlasFailed(handle.clone(), "Invalid Atlas JSON".to_string()));
                        }
                    } else {
                        *handle.container.state.write().unwrap() = LoadState::Failed("Invalid UTF-8".to_string());
                        events.push(AssetEvent::AtlasFailed(handle.clone(), "Invalid UTF-8".to_string()));
                    }
                },
                (AssetTask::Texture { handle, .. }, Err(e)) => {
                    *handle.container.state.write().unwrap() = LoadState::Failed(e.clone());
                    events.push(AssetEvent::TextureFailed(handle.clone(), e));
                },
                (AssetTask::Atlas { handle, .. }, Err(e)) => {
                    *handle.container.state.write().unwrap() = LoadState::Failed(e.clone());
                    events.push(AssetEvent::AtlasFailed(handle.clone(), e));
                },
                _ => {}
            }
        }
        events
    }
    
    // Helper for legacy code compatibility
    pub fn atlas_region(&self, atlas_handle: &Handle<Atlas>, sprite_name: &str) -> Option<([f32; 2], [f32; 2])> {
        if let Some(atlas) = atlas_handle.get() {
            return atlas.get(sprite_name);
        }
        None
    }
}
