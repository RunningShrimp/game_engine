//! 调试面板模块
//!
//! 提供各种调试面板的实现。

mod component_panel;
mod console_panel;
mod entity_panel;
mod performance_panel;
mod resource_panel;

pub use component_panel::ComponentPanel;
pub use console_panel::ConsolePanel;
pub use entity_panel::EntityPanel;
pub use performance_panel::PerformancePanel;
pub use resource_panel::ResourcePanel;

use bevy_ecs::prelude::*;

/// 面板通用trait
pub trait Panel {
    /// 显示面板
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui上下文
    /// * `world` - ECS世界引用
    fn show(&mut self, ctx: &egui::Context, world: &World);
}

/// 组件信息结构
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// 组件名称
    pub name: String,
    /// 组件类型名
    pub type_name: String,
    /// 组件大小（字节）
    pub size: usize,
    /// 组件数据（用于显示）
    pub data: String,
}

/// 实体信息结构
#[derive(Debug, Clone)]
pub struct EntityInfo {
    /// 实体ID
    pub id: Entity,
    /// 实体名称
    pub name: Option<String>,
    /// 组件列表
    pub components: Vec<String>,
    /// 是否存活
    pub is_alive: bool,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::GRAY,
            LogLevel::Warning => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
            LogLevel::Debug => egui::Color32::LIGHT_BLUE,
        }
    }
}
