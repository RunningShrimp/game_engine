// 渲染优化模块
// 
// 注意：此模块独立于game_engine核心，避免循环依赖。
// 主要提供独立的渲染优化功能。

pub mod batch_renderer;

// 注意：FrustumCulling, LodManager, OcclusionCulling 已迁移到 game_engine::performance::rendering
// 请直接使用 game_engine::performance::rendering 中的实现
pub use batch_renderer::BatchRenderer;

