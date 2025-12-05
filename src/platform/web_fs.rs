#[cfg(target_arch = "wasm32")]
use super::{Filesystem, FsError};
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Window};

/// Web平台文件系统实现
/// 使用fetch API加载资源,使用localStorage作为缓存
pub struct WebFilesystem {
    window: Window,
}

impl WebFilesystem {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
        Ok(Self { window })
    }

    fn get_storage(&self) -> Result<web_sys::Storage, FsError> {
        self.window
            .local_storage()
            .map_err(|_| FsError::IoError("Failed to access localStorage".to_string()))?
            .ok_or_else(|| FsError::IoError("localStorage not available".to_string()))
    }
}

impl Filesystem for WebFilesystem {
    fn read_async(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, FsError>> + Send>> {
        let url = url.to_string();
        let window = self.window.clone();

        Box::pin(async move {
            // 创建fetch请求
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| FsError::NetworkError(format!("Failed to create request: {:?}", e)))?;

            // 发起fetch
            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| FsError::NetworkError(format!("Fetch failed: {:?}", e)))?;

            // 转换为Response
            let resp: Response = resp_value
                .dyn_into()
                .map_err(|_| FsError::NetworkError("Invalid response".to_string()))?;

            // 检查状态码
            if !resp.ok() {
                return Err(FsError::NetworkError(format!("HTTP {}", resp.status())));
            }

            // 读取数据
            let array_buffer = JsFuture::from(resp.array_buffer().map_err(|e| {
                FsError::NetworkError(format!("Failed to get array buffer: {:?}", e))
            })?)
            .await
            .map_err(|e| FsError::NetworkError(format!("Failed to read array buffer: {:?}", e)))?;

            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let mut data = vec![0u8; uint8_array.length() as usize];
            uint8_array.copy_to(&mut data);

            Ok(data)
        })
    }

    fn cache_get(&self, key: &str) -> Option<Vec<u8>> {
        let storage = self.get_storage().ok()?;
        let value_str = storage.get_item(key).ok()??;

        // 从base64解码
        let bytes = base64::decode(&value_str).ok()?;
        Some(bytes)
    }

    fn cache_set(&self, key: &str, data: &[u8]) {
        if let Ok(storage) = self.get_storage() {
            // 编码为base64
            let encoded = base64::encode(data);
            let _ = storage.set_item(key, &encoded);
        }
    }
}

impl Default for WebFilesystem {
    fn default() -> Self {
        Self::new().expect("Failed to create WebFilesystem")
    }
}
