//! GLTF 模型加载器
//!
//! 提供异步 GLTF/GLB 模型加载功能，支持纹理、网格和场景数据解析。
//!
//! 这个模块使用条件编译来提供完整实现或存根实现，具体取决于 `gltf` 特性是否启用。

#[cfg(feature = "gltf")]
#[path = "gltf_loader_impl.rs"]
mod gltf_loader_impl;

#[cfg(not(feature = "gltf"))]
#[path = "gltf_loader_stub.rs"]
mod gltf_loader_stub;

// 统一导出接口
#[cfg(feature = "gltf")]
pub use gltf_loader_impl::{GltfLoadError, GltfLoader, GltfScene};

#[cfg(not(feature = "gltf"))]
pub use gltf_loader_stub::{GltfLoadError, GltfLoader, GltfScene};
