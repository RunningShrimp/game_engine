// 重新导出game_engine_performance中的BatchRenderer
pub use game_engine_performance::rendering::batch_renderer::BatchRenderer;

// 注意: FrustumCulling, LodManager, OcclusionCulling 已移至 render 模块
// 使用 render::frustum, render::lod, render::occlusion_culling 替代
