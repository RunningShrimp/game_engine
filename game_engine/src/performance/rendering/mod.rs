pub mod render_optimization;

// 重新导出game_engine_performance中的BatchRenderer
pub use game_engine_performance::rendering::batch_renderer::BatchRenderer;
pub use render_optimization::{FrustumCulling, LodManager, OcclusionCulling};
