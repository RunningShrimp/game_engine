// ============================================================================
// 脚本系统 - 简化版本
// TODO: 重新设计线程安全的脚本引擎架构
// ============================================================================

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// 脚本组件
#[derive(Component, Default)]
pub struct Script {
    pub source: String,
    pub enabled: bool,
}

impl Script {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            enabled: true,
        }
    }
}

/// 脚本资源句柄
#[derive(Component)]
pub struct ScriptAsset {
    pub path: String,
    pub hot_reload: bool,
}

/// 脚本运行时状态
#[derive(Component)]
#[derive(Default)]
pub struct ScriptState {
    /// 是否已初始化
    pub initialized: bool,
    /// 上次修改时间 (用于热重载)
    pub last_modified: u64,
    /// 本地变量存储
    pub locals: HashMap<String, ScriptValue>,
}


/// 脚本值类型
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
}
