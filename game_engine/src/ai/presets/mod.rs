//! # AI行为预设库
//!
//! 提供预定义的NPC行为树和AI组件，用于快速配置游戏AI。

pub mod npc_behaviors;

// 重新导出常用类型
pub use npc_behaviors::{
    ArcherBehavior, ArrowType, DecisionPreset, DecisionStyle, GuardBehavior, MageBehavior,
    MagicType, MerchantBehavior, PathfindingAlgorithm, PathfindingPreset, PerceptionPreset,
    TargetPriority, WarriorBehavior, WarriorStance,
};
