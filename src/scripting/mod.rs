pub mod engine;

pub use engine::*;

use bevy_ecs::prelude::*;

/// 脚本系统占位
pub fn scripting_system() {
    // TODO: 实现脚本执行逻辑
}

/// 初始化脚本系统
pub fn setup_scripting(_world: &mut World) {
    // TODO: 初始化脚本运行时
    // 需要重新设计线程安全的架构
}
