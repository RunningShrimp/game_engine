//! WGPU 渲染模块
//!
//! 将原本 1600+ 行的 wgpu.rs 拆分为多个子模块，提高可维护性。
//!
//! ## 模块结构
//!
//! - `types`: 公共类型定义（Instance, Vertex, Uniforms 等）
//! - `buffer`: 缓冲区管理（含 DoubleBufferedInstances）
//! - `pipeline`: 渲染管线创建
//! - `texture`: 纹理管理
//! - `renderer`: WgpuRenderer 核心实现
//!
//! ## 使用方式
//!
//! ```ignore
//! use crate::render::wgpu_modules::{WgpuRenderer, Instance, DoubleBufferedInstances};
//! ```

pub mod buffer;
pub mod pipeline;
pub mod texture;
pub mod types;

// 重导出主要类型
pub use buffer::{DoubleBufferedInstances, InstanceDirtyTracker};
pub use types::{
    DrawGroup, GpuPointLight, Instance, ModelUniform, ScreenUniform, UiInstance, Uniforms3D, Vertex,
};
