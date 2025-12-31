//! # DDGI (Dynamic Diffuse Global Illumination) 全局光照系统
//!
//! **API 稳定性**: 实验性 (Experimental) (v0.2.0)
//!
//! 提供基于探针的实时全局光照功能：
//! - 动态漫反射全局光照
//! - 探针网格管理
//! - 辐照度纹理
//! - 深度纹理
//! - 调试可视化
//!
//! ## API 稳定性声明
//!
//! **警告**: 此 API 处于实验性阶段，可能会在未来版本中发生破坏性变更。
//! - **状态**: 实验性 (Experimental) - WIP
//! - **引入版本**: v0.2.0
//! - **预期稳定版本**: v0.4.0
//!
//! ## 功能完整性追踪
//!
//! | 功能 | 状态 | 说明 |
//! |------|------|------|
//! | 探针网格生成 | ✅ 已实现 | 支持3D网格布局 |
//! | 探针渲染 | ✅ 已实现 | 6方向深度和法线渲染 |
//! | 辐照度更新 | ✅ 已实现 | 基于球谐函数的辐照度 |
//! | 光照传播 | ✅ 已实现 | 探针间光照传播 |
//! | 纹理管理 | ✅ 已实现 | 3D纹理和2D数组纹理 |
//! | 调试可视化 | ✅ 已实现 | 多种可视化模式 |
//! | 自适应更新 | 🚧 开发中 | 基于优先级的探针更新 |
//! | 时序滤波 | 🚧 开发中 | 减少闪烁和噪声 |
//! | 多质量级别 | ✅ 已实现 | Low/Medium/High |
//!
//! ## 使用说明
//!
//! DDGI 通过在场景中放置探针网格来捕获和传播全局光照。
//!
//! ### 示例
//!
//! ```rust,no_run
//! use game_engine::render::gi::{DDGIVolume, DDGIConfig};
//!
//! let config = DDGIConfig {
//!     probe_spacing: 2.0,
//!     probe_counts: glam::UVec3::new(10, 10, 10),
//!     irradiance_resolution: 16,
//!     depth_resolution: 16,
//!     ..Default::default()
//! };
//!
//! let volume = DDGIVolume::new(&device, &config)?;
//! ```
//!
//! ## 性能特性
//!
//! - **可配置更新率**: 支持每N帧更新一次
//! - **探针数量优化**: 通过调整间距和数量平衡质量和性能
//! - **重要性采样**: 优先更新重要探针
//! - **多质量级别**: Low/Medium/High预设
//!
//! ## 已知限制
//!
//! 1. 探针数量受限（过多会影响性能）
//! 2. 动态场景更新开销较大
//! 3. 光泄漏问题需要正常偏移处理
//! 4. 硬表面（镜面）需要单独处理
//!
//! ## 未来改进计划
//!
//! - [ ] 实现自适应探针更新
//! - [ ] 添加时序滤波和降噪
//! - [ ] 支持镜面反射探针
//! - [ ] 优化光照传播算法
//! - [ ] 添加级联DDGI支持
//! - [ ] 实现探针剔除优化

pub mod ddgi;
pub mod debug;
pub mod irradiance;
pub mod probe;
pub mod tests;
pub mod volume;

#[cfg(test)]
mod integration_test;

// Re-export main types
pub use ddgi::{DDGIError, DDGIQuality, DDGIVolume};
pub use debug::{GIDebugVisualizer, ProbeVisualization};
pub use irradiance::IrradianceTexture;
pub use probe::{DDGIProbe, ProbeManager};
pub use volume::DDGIConfig;
