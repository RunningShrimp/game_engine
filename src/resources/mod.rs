use std::path::Path;

pub mod runtime;
pub use runtime::{global_runtime, spawn, block_on};

pub struct AssetLoader;

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader {
    pub fn new() -> Self { Self }
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load_bytes(&self, path: &Path) -> Result<Vec<u8>, ()> { std::fs::read(path).map_err(|_| ()) }
    #[cfg(target_arch = "wasm32")]
    pub async fn load_bytes(&self, path: &Path) -> Result<Vec<u8>, ()> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let win = web_sys::window().ok_or(())?;
        // IndexedDB 读取封装，命中则直接返回
        async fn idb_get_bytes(url: &str) -> Result<Option<Vec<u8>>, ()> {
            use wasm_bindgen::JsCast;
            let win = web_sys::window().ok_or(())?;
            let factory = win.indexed_db().map_err(|_| ())?;
            let Some(factory) = factory else { return Ok(None) };
            let req = factory.open_with_u32("assets-cache", 1).ok_or(())?;
            // onupgradeneeded: create store if missing
            {
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
                    let req = e.target().unwrap().dyn_into::<web_sys::IdbOpenDbRequest>().unwrap();
                    let db = req.result().unwrap().dyn_into::<web_sys::IdbDatabase>().unwrap();
                    let _ = db.create_object_store("files");
                }) as Box<dyn FnMut(_)>);
                req.set_onupgradeneeded(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            let (tx, rx) = futures::channel::oneshot::channel();
            {
                let url_string = url.to_string();
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
                    if let Ok(open_req) = e.target().unwrap().dyn_into::<web_sys::IdbOpenDbRequest>() {
                        if let Ok(db) = open_req.result().and_then(|x| x.dyn_into::<web_sys::IdbDatabase>()) {
                            let txn = db.transaction_with_str_and_mode("files", web_sys::IdbTransactionMode::Readonly).unwrap();
                            let store = txn.object_store("files").unwrap();
                            let get_req = store.get(&wasm_bindgen::JsValue::from_str(&url_string)).unwrap();
                            let inner_tx = tx;
                            let success = wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::Event| {
                                let reqv = ev.target().unwrap().dyn_into::<web_sys::IdbRequest>().unwrap();
                                let result = reqv.result();
                                if result.is_undefined() || result.is_null() {
                                    let _ = inner_tx.send(None);
                                } else {
                                    // object with .bytes Uint8Array
                                    let bytes_val = js_sys::Reflect::get(&result, &wasm_bindgen::JsValue::from_str("bytes")).unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                                    if bytes_val.is_undefined() || bytes_val.is_null() {
                                        let _ = inner_tx.send(None);
                                    } else {
                                        let u8arr = js_sys::Uint8Array::new(&bytes_val);
                                        let mut v = vec![0u8; u8arr.length() as usize];
                                        u8arr.copy_to(&mut v[..]);
                                        let _ = inner_tx.send(Some(v));
                                    }
                                }
                            }) as Box<dyn FnMut(_)>);
                            get_req.set_onsuccess(Some(success.as_ref().unchecked_ref()));
                            success.forget();
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                req.set_onsuccess(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
            let res = rx.await.map_err(|_| ())?;
            Ok(res)
        }
        if let Some(bytes) = idb_get_bytes(&path.to_string_lossy()).await? { return Ok(bytes); }
        // try CacheStorage (Service Worker 管理的缓存)
        if let Ok(caches) = win.caches() {
            if let Ok(promise) = caches.match_with_str(&path.to_string_lossy()) {
                if let Ok(resp_val) = JsFuture::from(promise).await {
                    if !resp_val.is_undefined() && !resp_val.is_null() {
                        let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| ())?;
                        if !resp.ok() { return Err(()); }
                        let buf_promise = resp.array_buffer().map_err(|_| ())?;
                        let buf = JsFuture::from(buf_promise).await.map_err(|_| ())?;
                        let u8arr = js_sys::Uint8Array::new(&buf);
                        let mut v = vec![0u8; u8arr.length() as usize];
                        u8arr.copy_to(&mut v[..]);
                        return Ok(v);
                    }
                }
            }
        }
        if let Some(storage) = win.local_storage().map_err(|_| ())? {
            if let Some(k) = storage.get_item(&path.to_string_lossy()).map_err(|_| ())? {
                let bytes = base64::decode(k).map_err(|_| ())?;
                return Ok(bytes);
            }
        }
        let url = path.to_string_lossy();
        let resp_val = JsFuture::from(win.fetch_with_str(&url)).await.map_err(|_| ())?;
        let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| ())?;
        if !resp.ok() { return Err(()); }
        let ctype = resp.headers().get("Content-Type").map_err(|_| ())?.unwrap_or_default();
        let buf_promise = resp.array_buffer().map_err(|_| ())?;
        let buf = JsFuture::from(buf_promise).await.map_err(|_| ())?;
        let u8arr = js_sys::Uint8Array::new(&buf);
        let mut v = vec![0u8; u8arr.length() as usize];
        u8arr.copy_to(&mut v[..]);
        // IndexedDB 持久缓存写入
        if let Some(factory) = win.indexed_db().map_err(|_| ())? {
            if let Some(dbreq) = factory.open_with_u32("assets-cache", 1) {
                let onup = {
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
                        let req = e.target().unwrap().dyn_into::<web_sys::IdbOpenDbRequest>().unwrap();
                        let db = req.result().unwrap().dyn_into::<web_sys::IdbDatabase>().unwrap();
                        let _ = db.create_object_store("files");
                    }) as Box<dyn FnMut(_)>);
                    dbreq.set_onupgradeneeded(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                    ()
                };
                let onok = {
                    let url = path.to_string_lossy().to_string();
                    let bytes = v.clone();
                    let ctype_cl = ctype.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
                        if let Some(req) = e.target().and_then(|t| t.dyn_into::<web_sys::IdbOpenDbRequest>().ok()) {
                            if let Ok(db) = req.result().and_then(|x| x.dyn_into::<web_sys::IdbDatabase>()) {
                                let tx = db.transaction_with_str_and_mode("files", web_sys::IdbTransactionMode::Readwrite).unwrap();
                                let store = tx.object_store("files").unwrap();
                                let obj = js_sys::Object::new();
                                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("url"), &JsValue::from_str(&url));
                                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("content_type"), &JsValue::from_str(&ctype_cl));
                                let u8a = js_sys::Uint8Array::from(bytes.as_slice());
                                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("bytes"), &u8a);
                                let _ = store.put_with_key(&obj, &JsValue::from_str(&url));
                            }
                        }
                    }) as Box<dyn FnMut(_)>);
                    dbreq.set_onsuccess(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                    ()
                };
                let _ = (onup, onok);
            }
        }
        if let Some(storage) = win.local_storage().map_err(|_| ())? {
            // 按类型选择是否缓存
            let cacheable = ctype.starts_with("image/") || ctype.starts_with("application/json");
            if cacheable {
                let b64 = base64::encode(&v);
                let _ = storage.set_item(&path.to_string_lossy(), &b64);
            }
        }
        Ok(v)
    }
}
pub mod atlas;
pub mod manager;
pub mod hot_reload;
pub mod font;
pub mod events;
pub mod coroutine_loader;
pub mod staging_buffer;
pub mod upload_queue;

// Re-export coroutine loader for convenience
pub use coroutine_loader::{
    CoroutineAssetLoader, CoroutineLoaderConfig,
    LoadPriority, AssetType, LoadResult, LoadError, LoadComplete, LoadHandle, LoaderStats,
};

// Re-export staging buffer and upload queue
pub use staging_buffer::{StagingBuffer, StagingBufferPool, PoolStats};
pub use upload_queue::{UploadQueue, UploadStats, TextureUploadInfo, TextureUploadBuilder};

#[cfg(test)]
mod tests;
