//! 异步资源加载器（旧版实现）
//!
//! 注意：此模块已被 `CoroutineAssetLoader` 取代。
//! 保留此模块仅用于向后兼容。

use std::{path::PathBuf, sync::{Arc, Mutex}};
use tokio::task::JoinHandle;

pub struct Handle<T> { inner: Arc<Mutex<Option<T>>> }
impl<T> Handle<T> { pub fn new() -> Self { Self { inner: Arc::new(Mutex::new(None)) } } pub fn get(&self) -> Option<T> where T: Clone { self.inner.lock().unwrap().clone() } }

pub struct AssetManagerAsync;
impl AssetManagerAsync {
    pub fn load_bytes(path: PathBuf) -> (Handle<Vec<u8>>, JoinHandle<()>) {
        let handle = Handle::<Vec<u8>>::new();
        let inner = handle.inner.clone();
        let task = tokio::spawn(async move {
            let p = path;
            let data = tokio::fs::read(p).await.ok();
            if let Some(d) = data { *inner.lock().unwrap() = Some(d); }
        });
        (handle, task)
    }

    pub fn load_texture(name: String, path: PathBuf) -> (Handle<()>, JoinHandle<()>) {
        let h = Handle::<()>::new();
        let task = tokio::spawn(async move {
            let _ = tokio::fs::read(&path).await.ok();
            crate::resources::events::push_texture_ready(name);
        });
        (h, task)
    }

    pub fn load_atlas(name: String, path: PathBuf) -> (Handle<()>, JoinHandle<()>) {
        let h = Handle::<()>::new();
        let task = tokio::spawn(async move {
            let _ = tokio::fs::read(&path).await.ok();
            crate::resources::events::push_atlas_ready(name);
        });
        (h, task)
    }
}
