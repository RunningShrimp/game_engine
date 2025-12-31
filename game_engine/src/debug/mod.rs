//! 调试UI模块
//!
//! 提供基于egui的交互式调试面板，用于实时监控和调试游戏引擎状态。
//!
//! ## 功能特性
//!
//! - **实体面板** - 显示所有实体和组件
//! - **组件面板** - 显示组件详细信息
//! - **性能面板** - FPS、Draw Calls、内存使用
//! - **控制台** - 脚本日志和错误
//! - **资源面板** - 资源加载状态
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::debug::DebugUI;
//!
//! // 创建调试UI
//! let mut debug_ui = DebugUI::new();
//!
//! // 在渲染循环中
//! debug_ui.render(&egui_ctx, &world);
//! ```

pub mod panels;
pub mod ui;
pub mod visualizer;

pub use ui::DebugUI;

// 导出各个面板
pub use panels::{ComponentPanel, ConsolePanel, EntityPanel, PerformancePanel, ResourcePanel};

/// 调试UI配置
#[derive(Clone, Debug)]
pub struct DebugConfig {
    /// 是否启用调试UI
    pub enabled: bool,
    /// 默认面板可见性
    pub show_entities: bool,
    pub show_components: bool,
    pub show_performance: bool,
    pub show_console: bool,
    pub show_resources: bool,
    /// 性能历史记录长度
    pub performance_history_size: usize,
    /// 控制台日志最大行数
    pub console_max_lines: usize,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_entities: true,
            show_components: false,
            show_performance: true,
            show_console: true,
            show_resources: false,
            performance_history_size: 300,
            console_max_lines: 1000,
        }
    }
}

/// 调试UI错误类型
#[derive(thiserror::Error, Debug)]
pub enum DebugUIError {
    #[error("ECS query error: {0}")]
    ECSQueryError(String),

    #[error("Component access error: {0}")]
    ComponentAccessError(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),
}

/// 调试UI结果类型
pub type Result<T> = std::result::Result<T, DebugUIError>;

// 测试模块
#[cfg(test)]
mod tests;
