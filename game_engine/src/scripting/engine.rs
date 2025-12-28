// ============================================================================
// 脚本系统
// ============================================================================

use super::system::ScriptValue;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

// Script 已删除 - 请使用 ScriptComponent 替代

/// 脚本资源句柄
#[derive(Component)]
pub struct ScriptAsset {
    /// 脚本路径
    pub path: String,
    /// 是否启用热重载
    pub hot_reload: bool,
}

/// 脚本运行时状态
#[derive(Component, Default)]
pub struct ScriptState {
    /// 是否已初始化
    pub initialized: bool,
    /// 上次修改时间 (用于热重载)
    pub last_modified: u64,
    /// 本地变量存储
    pub locals: HashMap<String, ScriptValue>,
}
