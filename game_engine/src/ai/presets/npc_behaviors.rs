//! # AI行为预设库
//!
//! 提供预定义的NPC行为树和AI组件，用于快速配置游戏AI。
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::presets::*;
//!
//! // 创建战士NPC
//! let warrior_ai = WarriorBehavior::new();
//! warrior_ai.set_aggressiveness(0.8);
//! warrior_ai.set_combat_range(5.0);
//!
//! // 创建商NPC
//! let merchant_ai = MerchantBehavior::new();
//! merchant_ai.set_shop_hours(8.0, 20.0);
//!
//! // 创建守卫NPC
//! let guard_ai = GuardBehavior::new();
//! guard_ai.set_patrol_route(vec![
//!     Vec3::new(0.0, 0.0, 0.0),
//!     Vec3::new(10.0, 0.0, 0.0),
//! ]);
//! ```

use crate::ai::components::*;
use glam::Vec3;
use std::collections::VecDeque;

// ============================================================================
// NPC行为树预设
// ============================================================================

/// 战士行为树
///
/// 适合近战战斗的NPC，特点是高生命值、近战攻击、肉盾定位。
#[derive(Debug, Clone)]
pub struct WarriorBehavior {
    /// 攻击性（0.0-1.0）
    pub aggressiveness: f32,
    /// 战斗范围
    pub combat_range: f32,
    /// 是否使用盾牌
    pub use_shield: bool,
    /// 战斗姿态
    pub stance: WarriorStance,
}

/// 战士姿态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarriorStance {
    /// 攻击姿态
    Aggressive,
    /// 防御姿态
    Defensive,
    /// 平衡姿态
    Balanced,
}

impl WarriorBehavior {
    /// 创建新的战士行为
    pub fn new() -> Self {
        Self {
            aggressiveness: 0.7,
            combat_range: 5.0,
            use_shield: true,
            stance: WarriorStance::Balanced,
        }
    }

    /// 设置攻击性
    pub fn set_aggressiveness(&mut self, value: f32) {
        self.aggressiveness = value.clamp(0.0, 1.0);
    }

    /// 设置战斗范围
    pub fn set_combat_range(&mut self, range: f32) {
        self.combat_range = range.max(0.0);
    }

    /// 生成行为树
    pub fn build_behavior_tree(&self) -> BehaviorTree {
        let mut tree = BehaviorTree::new("Warrior");

        // 选择器：尝试不同行动
        let mut selector = SelectorNode::new("Combat Selector");

        // 1. 检测敌人
        let detect_enemy = ConditionNode::new(
            "Detect Enemy",
            Box::new(|ctx| {
                // 实现敌人检测逻辑
                // 从黑板获取敌人信息，或进行视野范围内的敌人检测
                let has_enemy = ctx.blackboard.get_bool("has_enemy").unwrap_or(false);

                // 如果没有敌人信息，尝试从视野中检测
                if !has_enemy {
                    // 检查视野范围内是否有敌人
                    // 这里使用黑板上的"enemies_in_range"信息
                    ctx.blackboard
                        .get_vec("enemies_in_range")
                        .map(|enemies| !enemies.is_empty())
                        .unwrap_or(false)
                } else {
                    true
                }
            }),
        );

        // 2. 攻击敌人
        let mut attack_sequence = SequenceNode::new("Attack Sequence");

        let move_to_enemy = ActionNode::new(
            "Move to Enemy",
            Box::new(|ctx| {
                // 实现移动到敌人的逻辑
                // 从黑板获取最近敌人的位置
                if let Some(enemy_pos) = ctx.blackboard.get_vec3("nearest_enemy_position") {
                    if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                        // 计算移动方向
                        let direction = (enemy_pos - self_pos).normalize();

                        // 设置移动速度（这里假设移动速度为3.0）
                        let move_speed = 3.0;
                        let velocity = direction * move_speed;

                        // 将速度存储到黑板，供移动系统使用
                        ctx.blackboard.set("movement_velocity", velocity);

                        // 检查是否到达攻击范围
                        let distance = (enemy_pos - self_pos).length();
                        let combat_range = ctx.blackboard.get_f32("combat_range").unwrap_or(5.0);

                        if distance <= combat_range {
                            BehaviorStatus::Success
                        } else {
                            BehaviorStatus::Running
                        }
                    } else {
                        BehaviorStatus::Failure
                    }
                } else {
                    BehaviorStatus::Failure
                }
            }),
        );

        let perform_attack = ActionNode::new(
            "Perform Attack",
            Box::new(|ctx| {
                // 实现攻击逻辑
                // 从黑板获取攻击性参数，决定攻击类型
                let aggressiveness = ctx.blackboard.get_f32("aggressiveness").unwrap_or(0.7);
                let use_shield = ctx.blackboard.get_bool("use_shield").unwrap_or(false);

                // 根据攻击性选择攻击策略
                if aggressiveness > 0.8 {
                    // 高攻击性：使用重击
                    ctx.blackboard.set("attack_type", "heavy");
                } else if aggressiveness > 0.5 {
                    // 中等攻击性：普通攻击
                    ctx.blackboard.set("attack_type", "normal");
                } else {
                    // 低攻击性：防御姿态
                    ctx.blackboard.set("attack_type", "defensive");
                }

                // 如果使用盾牌，设置防御姿态
                if use_shield {
                    ctx.blackboard.set("shield_active", true);
                }

                // 触发攻击动画
                ctx.blackboard.set("trigger_attack", true);

                BehaviorStatus::Success
            }),
        );

        attack_sequence.add_child(Box::new(move_to_enemy));
        attack_sequence.add_child(Box::new(perform_attack));

        // 3. 巡逻/待机
        let idle = ActionNode::new(
            "Idle",
            Box::new(|ctx| {
                // 实现待机逻辑
                // 检查是否有巡逻路径
                if let Some(_patrol_route) = ctx.blackboard.get_vec("patrol_route") {
                    // 有巡逻路径，执行巡逻逻辑
                    let current_patrol_index =
                        ctx.blackboard.get_i32("current_patrol_index").unwrap_or(0);

                    // 获取当前巡逻点
                    if let Some(patrol_points) = ctx.blackboard.get_vec("patrol_points") {
                        if let Some(target_point) = patrol_points.get(current_patrol_index as usize)
                        {
                            // 移动到巡逻点
                            if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                                let distance = (target_point - self_pos).length();

                                if distance < 1.0 {
                                    // 到达巡逻点，移动到下一个
                                    let next_index =
                                        (current_patrol_index + 1) % patrol_points.len() as i32;
                                    ctx.blackboard.set("current_patrol_index", next_index);
                                } else {
                                    // 继续移动到当前巡逻点
                                    let direction = (target_point - self_pos).normalize();
                                    let patrol_speed = 1.5; // 巡逻速度较慢
                                    let velocity = direction * patrol_speed;
                                    ctx.blackboard.set("movement_velocity", velocity);
                                }
                            }
                        }
                    }

                    BehaviorStatus::Running
                } else {
                    // 没有巡逻路径，执行待机逻辑
                    // 停止移动
                    ctx.blackboard.set("movement_velocity", Vec3::ZERO);

                    // 设置待机动画
                    ctx.blackboard.set("animation_state", "idle");

                    // 偶尔环顾四周（每2秒一次）
                    let elapsed = ctx.blackboard.get_f32("idle_time").unwrap_or(0.0);
                    if elapsed > 2.0 {
                        ctx.blackboard.set("look_around", true);
                        ctx.blackboard.set("idle_time", 0.0);
                    } else {
                        ctx.blackboard.set("idle_time", elapsed + 0.016); // 假设60fps
                        ctx.blackboard.set("look_around", false);
                    }

                    BehaviorStatus::Success
                }
            }),
        );

        selector.add_child(Box::new(detect_enemy));
        selector.add_child(Box::new(attack_sequence));
        selector.add_child(Box::new(idle));

        tree.set_root(Box::new(selector));
        tree
    }
}

impl Default for WarriorBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 法师行为树
///
/// 适合远程魔法攻击的NPC，特点是低生命值、高伤害、范围攻击。
#[derive(Debug, Clone)]
pub struct MageBehavior {
    /// 魔法类型
    pub magic_type: MagicType,
    /// 施法距离
    pub cast_range: f32,
    /// 是否使用护盾
    pub use_magic_shield: bool,
    /// 优先攻击目标
    pub priority_target: TargetPriority,
}

/// 魔法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicType {
    /// 火焰魔法
    Fire,
    /// 冰霜魔法
    Ice,
    /// 闪电魔法
    Lightning,
    /// 治疗魔法
    Healing,
}

/// 目标优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPriority {
    /// 优先攻击低血量目标
    LowHealth,
    /// 优先攻击高血量目标
    HighHealth,
    /// 优先攻击最近目标
    Nearest,
    /// 优先攻击远程目标
    Ranged,
}

impl MageBehavior {
    pub fn new() -> Self {
        Self {
            magic_type: MagicType::Fire,
            cast_range: 20.0,
            use_magic_shield: true,
            priority_target: TargetPriority::Nearest,
        }
    }

    pub fn set_cast_range(&mut self, range: f32) {
        self.cast_range = range.max(0.0);
    }

    pub fn build_behavior_tree(&self) -> BehaviorTree {
        let mut tree = BehaviorTree::new("Mage");

        let mut selector = SelectorNode::new("Magic Selector");

        // 1. 检查法力值
        let check_mana = ConditionNode::new(
            "Check Mana",
            Box::new(|ctx| ctx.blackboard.get_f32("mana").unwrap_or(0.0) > 20.0),
        );

        // 2. 施法序列
        let mut cast_sequence = SequenceNode::new("Cast Spell");

        let select_target = ActionNode::new(
            "Select Target",
            Box::new(|ctx| {
                // 根据priority_target选择目标
                let priority_target =
                    ctx.blackboard.get_string("priority_target").unwrap_or("nearest".to_string());

                match priority_target.as_str() {
                    "low_health" => {
                        // 优先选择低血量目标
                        if let Some(enemies) = ctx.blackboard.get_vec("enemies_in_range") {
                            let target = enemies.iter().min_by(|a, b| {
                                let health_a = ctx
                                    .blackboard
                                    .get_f32(&format!("{}_health", a))
                                    .unwrap_or(100.0);
                                let health_b = ctx
                                    .blackboard
                                    .get_f32(&format!("{}_health", b))
                                    .unwrap_or(100.0);
                                health_a.partial_cmp(&health_b).unwrap_or(std::cmp::Ordering::Equal)
                            });

                            if let Some(best_target) = target {
                                ctx.blackboard.set("selected_target", best_target.clone());
                                BehaviorStatus::Success
                            } else {
                                BehaviorStatus::Failure
                            }
                        } else {
                            BehaviorStatus::Failure
                        }
                    }
                    "high_health" => {
                        // 优先选择高血量目标（坦克）
                        if let Some(enemies) = ctx.blackboard.get_vec("enemies_in_range") {
                            let target = enemies.iter().max_by(|a, b| {
                                let health_a =
                                    ctx.blackboard.get_f32(&format!("{}_health", a)).unwrap_or(0.0);
                                let health_b =
                                    ctx.blackboard.get_f32(&format!("{}_health", b)).unwrap_or(0.0);
                                health_a.partial_cmp(&health_b).unwrap_or(std::cmp::Ordering::Equal)
                            });

                            if let Some(best_target) = target {
                                ctx.blackboard.set("selected_target", best_target.clone());
                                BehaviorStatus::Success
                            } else {
                                BehaviorStatus::Failure
                            }
                        } else {
                            BehaviorStatus::Failure
                        }
                    }
                    "ranged" => {
                        // 优先选择远程目标（弓箭手、法师）
                        if let Some(enemies) = ctx.blackboard.get_vec("enemies_in_range") {
                            let target = enemies.iter().find(|enemy| {
                                let enemy_type = ctx
                                    .blackboard
                                    .get_string(&format!("{}_type", enemy))
                                    .unwrap_or("unknown".to_string());
                                enemy_type == "archer" || enemy_type == "mage"
                            });

                            if let Some(best_target) = target {
                                ctx.blackboard.set("selected_target", best_target.clone());
                                BehaviorStatus::Success
                            } else {
                                BehaviorStatus::Failure
                            }
                        } else {
                            BehaviorStatus::Failure
                        }
                    }
                    _ => {
                        // 默认：选择最近的目标
                        if let Some(nearest_enemy) =
                            ctx.blackboard.get_vec3("nearest_enemy_position")
                        {
                            ctx.blackboard.set("selected_target", nearest_enemy);
                            BehaviorStatus::Success
                        } else {
                            BehaviorStatus::Failure
                        }
                    }
                }
            }),
        );

        let cast_spell = ActionNode::new(
            "Cast Spell",
            Box::new(|ctx| {
                // 施放魔法
                let magic_type =
                    ctx.blackboard.get_string("magic_type").unwrap_or("fire".to_string());
                let mana_cost = match magic_type.as_str() {
                    "fire" => 25.0,
                    "ice" => 30.0,
                    "lightning" => 35.0,
                    "healing" => 40.0,
                    _ => 20.0,
                };

                // 检查法力值是否足够
                let current_mana = ctx.blackboard.get_f32("mana").unwrap_or(100.0);
                if current_mana < mana_cost {
                    // 法力不足，返回失败
                    BehaviorStatus::Failure
                } else {
                    // 消耗法力值
                    ctx.blackboard.set("mana", current_mana - mana_cost);

                    // 设置魔法攻击参数
                    ctx.blackboard.set("casting_spell", true);
                    ctx.blackboard.set("spell_type", magic_type);
                    ctx.blackboard.set(
                        "spell_damage",
                        match magic_type.as_str() {
                            "fire" => 50.0,
                            "ice" => 40.0,
                            "lightning" => 60.0,
                            "healing" => -30.0, // 负值表示治疗
                            _ => 35.0,
                        },
                    );

                    // 触发施法动画
                    ctx.blackboard.set("animation_state", "casting");
                    ctx.blackboard.set("trigger_spell_effect", true);

                    BehaviorStatus::Success
                }
            }),
        );

        cast_sequence.add_child(Box::new(select_target));
        cast_sequence.add_child(Box::new(cast_spell));

        // 3. 后退保持距离
        let retreat = ActionNode::new(
            "Retreat",
            Box::new(|ctx| {
                // 后退到安全距离
                let cast_range = ctx.blackboard.get_f32("cast_range").unwrap_or(20.0);
                let safe_distance = cast_range * 0.8; // 保持80%的施法距离作为安全距离

                if let Some(enemy_pos) = ctx.blackboard.get_vec3("nearest_enemy_position") {
                    if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                        let distance = (enemy_pos - self_pos).length();

                        if distance < safe_distance {
                            // 距离太近，需要后退
                            let away_direction = (self_pos - enemy_pos).normalize();
                            let retreat_speed = 2.5; // 后退速度
                            let velocity = away_direction * retreat_speed;

                            ctx.blackboard.set("movement_velocity", velocity);
                            ctx.blackboard.set("is_retreating", true);

                            BehaviorStatus::Running
                        } else if distance > cast_range {
                            // 距离太远，需要靠近
                            let toward_direction = (enemy_pos - self_pos).normalize();
                            let approach_speed = 2.0;
                            let velocity = toward_direction * approach_speed;

                            ctx.blackboard.set("movement_velocity", velocity);
                            ctx.blackboard.set("is_retreating", false);

                            BehaviorStatus::Running
                        } else {
                            // 距离合适，停止移动
                            ctx.blackboard.set("movement_velocity", Vec3::ZERO);
                            ctx.blackboard.set("is_retreating", false);

                            BehaviorStatus::Success
                        }
                    } else {
                        BehaviorStatus::Failure
                    }
                } else {
                    BehaviorStatus::Failure
                }
            }),
        );

        selector.add_child(Box::new(check_mana));
        selector.add_child(Box::new(cast_sequence));
        selector.add_child(Box::new(retreat));

        tree.set_root(Box::new(selector));
        tree
    }
}

impl Default for MageBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 弓箭手行为树
///
/// 适合远程物理攻击的NPC，特点是中等生命值、远程攻击、高机动性。
#[derive(Debug, Clone)]
pub struct ArcherBehavior {
    /// 射击距离
    pub shoot_range: f32,
    /// 是否移动射击
    pub mobile_shooting: bool,
    /// 箭矢类型
    pub arrow_type: ArrowType,
}

/// 箭矢类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowType {
    /// 普通箭
    Normal,
    /// 火焰箭
    Fire,
    /// 冰冻箭
    Ice,
    /// 穿甲箭
    Piercing,
}

impl ArcherBehavior {
    pub fn new() -> Self {
        Self {
            shoot_range: 30.0,
            mobile_shooting: true,
            arrow_type: ArrowType::Normal,
        }
    }

    pub fn set_shoot_range(&mut self, range: f32) {
        self.shoot_range = range.max(0.0);
    }

    pub fn build_behavior_tree(&self) -> BehaviorTree {
        let mut tree = BehaviorTree::new("Archer");

        let mut selector = SelectorNode::new("Ranged Selector");

        // 1. 瞄准敌人
        let aim = ConditionNode::new(
            "Aim Target",
            Box::new(|ctx| ctx.blackboard.get_bool("has_clear_shot").unwrap_or(false)),
        );

        // 2. 射击
        let mut shoot_sequence = SequenceNode::new("Shoot Sequence");

        let draw_bow = ActionNode::new(
            "Draw Bow",
            Box::new(|ctx| {
                // 拉弓
                // 检查是否有箭矢
                let arrow_count = ctx.blackboard.get_i32("arrow_count").unwrap_or(999);

                if arrow_count > 0 {
                    // 开始拉弓动作
                    ctx.blackboard.set("is_drawing_bow", true);

                    // 检查拉弓进度
                    let draw_progress = ctx.blackboard.get_f32("bow_draw_progress").unwrap_or(0.0);
                    let full_draw_time = ctx.blackboard.get_f32("full_draw_time").unwrap_or(1.0);

                    if draw_progress < full_draw_time {
                        // 正在拉弓
                        let new_progress = (draw_progress + 0.016).min(full_draw_time); // 假设60fps
                        ctx.blackboard.set("bow_draw_progress", new_progress);

                        if new_progress >= full_draw_time {
                            // 拉弓完成
                            ctx.blackboard.set("is_bow_drawn", true);
                            ctx.blackboard.set("bow_draw_progress", full_draw_time);
                            BehaviorStatus::Success
                        } else {
                            // 继续拉弓
                            ctx.blackboard.set("animation_state", "drawing_bow");
                            BehaviorStatus::Running
                        }
                    } else {
                        BehaviorStatus::Success
                    }
                } else {
                    // 没有箭矢
                    BehaviorStatus::Failure
                }
            }),
        );

        let release = ActionNode::new(
            "Release Arrow",
            Box::new(|ctx| {
                // 释放箭矢
                if ctx.blackboard.get_bool("is_bow_drawn").unwrap_or(false) {
                    // 释放箭矢
                    ctx.blackboard.set("trigger_shoot", true);

                    // 确定箭矢类型和伤害
                    let arrow_type =
                        ctx.blackboard.get_string("arrow_type").unwrap_or("normal".to_string());
                    let arrow_damage = match arrow_type.as_str() {
                        "fire" => 45.0,
                        "ice" => 35.0,
                        "piercing" => 55.0,
                        _ => 30.0,
                    };

                    ctx.blackboard.set("arrow_damage", arrow_damage);

                    // 减少箭矢数量
                    let arrow_count = ctx.blackboard.get_i32("arrow_count").unwrap_or(999);
                    ctx.blackboard.set("arrow_count", arrow_count - 1);

                    // 重置拉弓状态
                    ctx.blackboard.set("is_bow_drawn", false);
                    ctx.blackboard.set("bow_draw_progress", 0.0);
                    ctx.blackboard.set("is_drawing_bow", false);

                    // 触发射击动画
                    ctx.blackboard.set("animation_state", "shooting");

                    BehaviorStatus::Success
                } else {
                    // 弓未拉满
                    BehaviorStatus::Failure
                }
            }),
        );

        shoot_sequence.add_child(Box::new(draw_bow));
        shoot_sequence.add_child(Box::new(release));

        // 3. 移动到有利位置
        let reposition = ActionNode::new(
            "Reposition",
            Box::new(|ctx| {
                // 移动到射击位置
                let shoot_range = ctx.blackboard.get_f32("shoot_range").unwrap_or(30.0);
                let optimal_range = shoot_range * 0.7; // 最佳射击距离为射程的70%

                if let Some(enemy_pos) = ctx.blackboard.get_vec3("nearest_enemy_position") {
                    if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                        let distance = (enemy_pos - self_pos).length();
                        let mobile_shooting =
                            ctx.blackboard.get_bool("mobile_shooting").unwrap_or(true);

                        if distance > shoot_range {
                            // 超出射程，需要靠近
                            let toward_direction = (enemy_pos - self_pos).normalize();
                            let run_speed = 4.0; // 弓箭手移动速度较快
                            let velocity = toward_direction * run_speed;

                            ctx.blackboard.set("movement_velocity", velocity);
                            ctx.blackboard.set("animation_state", "running");

                            BehaviorStatus::Running
                        } else if distance < optimal_range && mobile_shooting {
                            // 距离太近，需要后退（如果支持移动射击）
                            let away_direction = (self_pos - enemy_pos).normalize();
                            let retreat_speed = 2.0;
                            let velocity = away_direction * retreat_speed;

                            ctx.blackboard.set("movement_velocity", velocity);
                            ctx.blackboard.set("animation_state", "backpedaling");

                            BehaviorStatus::Running
                        } else {
                            // 距离合适，停止移动，准备射击
                            ctx.blackboard.set("movement_velocity", Vec3::ZERO);

                            // 检查是否有清晰射击线
                            let has_clear_shot =
                                ctx.blackboard.get_bool("has_clear_shot").unwrap_or(true);

                            if has_clear_shot {
                                ctx.blackboard.set("animation_state", "aiming");
                                BehaviorStatus::Success
                            } else {
                                // 寻找新的射击位置
                                ctx.blackboard.set("animation_state", "repositioning");
                                BehaviorStatus::Running
                            }
                        }
                    } else {
                        BehaviorStatus::Failure
                    }
                } else {
                    BehaviorStatus::Failure
                }
            }),
        );

        selector.add_child(Box::new(aim));
        selector.add_child(Box::new(shoot_sequence));
        selector.add_child(Box::new(reposition));

        tree.set_root(Box::new(selector));
        tree
    }
}

impl Default for ArcherBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 商人行为树
///
/// 适合交易NPC，特点是开店/闭店时间、交易交互、静态行为。
#[derive(Debug, Clone)]
pub struct MerchantBehavior {
    /// 开店时间（小时）
    pub open_time: f32,
    /// 闭店时间（小时）
    pub close_time: f32,
    /// 交易物品列表
    pub trade_items: Vec<String>,
}

impl MerchantBehavior {
    pub fn new() -> Self {
        Self {
            open_time: 8.0,
            close_time: 20.0,
            trade_items: vec![],
        }
    }

    pub fn set_shop_hours(&mut self, open: f32, close: f32) {
        self.open_time = open;
        self.close_time = close;
    }

    pub fn build_behavior_tree(&self) -> BehaviorTree {
        let mut tree = BehaviorTree::new("Merchant");

        let mut selector = SelectorNode::new("Shop Selector");

        // 1. 检查营业时间
        let check_hours = ConditionNode::new(
            "Check Shop Hours",
            Box::new(|ctx| {
                let current_hour = ctx.blackboard.get_f32("game_hour").unwrap_or(12.0);
                current_hour >= 8.0 && current_hour <= 20.0
            }),
        );

        // 2. 交易序列
        let mut trade_sequence = SequenceNode::new("Trade Sequence");

        let greet = ActionNode::new(
            "Greet Customer",
            Box::new(|ctx| {
                // 问候顾客
                // 检查是否有顾客在附近
                if let Some(customer_nearby) =
                    ctx.blackboard.get_bool("customer_nearby").unwrap_or(false)
                {
                    // 问候顾客
                    let greeting_type = ctx
                        .blackboard
                        .get_string("greeting_type")
                        .unwrap_or("friendly".to_string());

                    let greeting_message = match greeting_type.as_str() {
                        "formal" => "Welcome, honored customer.",
                        "casual" => "Hey there! What can I get ya?",
                        "rude" => "What do you want? Make it quick.",
                        _ => "Welcome to my shop!",
                    };

                    ctx.blackboard.set("current_dialogue", greeting_message);
                    ctx.blackboard.set("trigger_greeting_animation", true);
                    ctx.blackboard.set("is_trading", true);

                    // 检查是否是首次顾客
                    let is_first_time =
                        ctx.blackboard.get_bool("is_first_time_customer").unwrap_or(true);
                    if is_first_time {
                        ctx.blackboard.set("discount_offer", 0.1); // 10%首次购买折扣
                        ctx.blackboard.set("is_first_time_customer", false);
                    }

                    BehaviorStatus::Success
                } else {
                    // 没有顾客
                    BehaviorStatus::Failure
                }
            }),
        );

        let show_wares = ActionNode::new(
            "Show Wares",
            Box::new(|ctx| {
                // 展示商品
                let trade_items = ctx.blackboard.get_vec("trade_items").unwrap_or(vec![]);

                if !trade_items.is_empty() {
                    // 获取当前顾客的需求类型
                    let customer_interest = ctx
                        .blackboard
                        .get_string("customer_interest")
                        .unwrap_or("general".to_string());

                    // 根据顾客兴趣过滤商品
                    let recommended_items: Vec<_> = if customer_interest != "general" {
                        trade_items
                            .iter()
                            .filter(|item| {
                                let item_type = ctx
                                    .blackboard
                                    .get_string(&format!("{}_type", item))
                                    .unwrap_or("general".to_string());
                                item_type == customer_interest || item_type == "general"
                            })
                            .collect()
                    } else {
                        trade_items.iter().collect()
                    };

                    // 设置展示的商品列表
                    ctx.blackboard.set("displayed_items", recommended_items);
                    ctx.blackboard.set("shop_menu_open", true);

                    // 设置商品价格（可能有折扣）
                    let discount = ctx.blackboard.get_f32("discount_offer").unwrap_or(0.0);
                    ctx.blackboard.set("current_discount", discount);

                    // 触发展示动画
                    ctx.blackboard.set("animation_state", "showing_wares");
                    ctx.blackboard.set("trigger_show_items", true);

                    BehaviorStatus::Success
                } else {
                    // 没有商品可展示
                    BehaviorStatus::Failure
                }
            }),
        );

        trade_sequence.add_child(Box::new(greet));
        trade_sequence.add_child(Box::new(show_wares));

        // 3. 闭店
        let close_shop = ActionNode::new(
            "Close Shop",
            Box::new(|ctx| {
                // 闭店动作
                let current_hour = ctx.blackboard.get_f32("game_hour").unwrap_or(12.0);
                let close_time = ctx.blackboard.get_f32("close_time").unwrap_or(20.0);

                if current_hour >= close_time {
                    // 开始闭店流程
                    ctx.blackboard.set("is_open", false);

                    // 清点货物
                    ctx.blackboard.set("animation_state", "counting_inventory");

                    // 计算今日收益
                    let daily_earnings = ctx.blackboard.get_f32("daily_earnings").unwrap_or(0.0);
                    ctx.blackboard.set("final_earnings", daily_earnings);

                    // 存储货物到保险箱
                    ctx.blackboard.set("trigger_store_items", true);

                    // 锁门
                    ctx.blackboard.set("animation_state", "locking_door");
                    ctx.blackboard.set("trigger_lock_door", true);

                    // 关闭店铺菜单
                    ctx.blackboard.set("shop_menu_open", false);

                    // 设置闭店对话
                    ctx.blackboard.set(
                        "current_dialogue",
                        "Shop's closed for today. Come back tomorrow!",
                    );

                    // 重置当日收益
                    ctx.blackboard.set("daily_earnings", 0.0);

                    BehaviorStatus::Success
                } else {
                    // 还没到闭店时间
                    BehaviorStatus::Failure
                }
            }),
        );

        selector.add_child(Box::new(check_hours));
        selector.add_child(Box::new(trade_sequence));
        selector.add_child(Box::new(close_shop));

        tree.set_root(Box::new(selector));
        tree
    }
}

impl Default for MerchantBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 守卫行为树
///
/// 适合守卫NPC，特点是巡逻路线、警报系统、区域防御。
#[derive(Debug, Clone)]
pub struct GuardBehavior {
    /// 巡逻路线
    pub patrol_route: Vec<Vec3>,
    /// 当前巡逻点索引
    pub current_patrol_index: usize,
    /// 警报距离
    pub alert_distance: f32,
    /// 是否在警报状态
    pub is_alerted: bool,
}

impl GuardBehavior {
    pub fn new() -> Self {
        Self {
            patrol_route: vec![],
            current_patrol_index: 0,
            alert_distance: 15.0,
            is_alerted: false,
        }
    }

    pub fn set_patrol_route(&mut self, route: Vec<Vec3>) {
        self.patrol_route = route;
        self.current_patrol_index = 0;
    }

    pub fn set_alert_distance(&mut self, distance: f32) {
        self.alert_distance = distance.max(0.0);
    }

    pub fn build_behavior_tree(&self) -> BehaviorTree {
        let mut tree = BehaviorTree::new("Guard");

        let mut selector = SelectorNode::new("Guard Selector");

        // 1. 检查入侵者
        let check_intruder = ConditionNode::new(
            "Check Intruder",
            Box::new(|ctx| {
                ctx.blackboard.get_f32("nearest_enemy_distance").unwrap_or(f32::MAX) < 15.0
            }),
        );

        // 2. 追击/战斗序列
        let mut combat_sequence = SequenceNode::new("Combat Sequence");

        let raise_alarm = ActionNode::new(
            "Raise Alarm",
            Box::new(|ctx| {
                // 触发警报
                // 检查是否已经触发警报
                let is_alerted = ctx.blackboard.get_bool("is_alerted").unwrap_or(false);

                if !is_alerted {
                    // 触发警报
                    ctx.blackboard.set("is_alerted", true);

                    // 设置警报级别
                    let threat_level = ctx.blackboard.get_f32("threat_level").unwrap_or(0.5);
                    let alert_level = if threat_level > 0.8 {
                        "critical"
                    } else if threat_level > 0.5 {
                        "high"
                    } else {
                        "medium"
                    };

                    ctx.blackboard.set("alert_level", alert_level);

                    // 通知其他守卫
                    ctx.blackboard.set("trigger_alarm_signal", true);
                    ctx.blackboard.set("alarm_position", ctx.blackboard.get_vec3("self_position"));

                    // 发出警报音效
                    ctx.blackboard.set("trigger_alarm_sound", true);

                    // 设置警报对话
                    let alert_message = match alert_level {
                        "critical" => "INTRUDER! Critical threat! Everyone to arms!",
                        "high" => "INTRUDER! Alert! Intruder detected!",
                        _ => "Halt! Identify yourself!",
                    };

                    ctx.blackboard.set("current_dialogue", alert_message);

                    // 触发警报动画
                    ctx.blackboard.set("animation_state", "raising_alarm");

                    // 加快移动速度
                    ctx.blackboard.set("movement_speed_multiplier", 1.5);

                    BehaviorStatus::Success
                } else {
                    // 警报已经触发
                    BehaviorStatus::Running
                }
            }),
        );

        let engage = ActionNode::new(
            "Engage Enemy",
            Box::new(|ctx| {
                // 与敌人战斗
                if let Some(enemy_pos) = ctx.blackboard.get_vec3("nearest_enemy_position") {
                    if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                        let distance = (enemy_pos - self_pos).length();
                        let combat_range =
                            ctx.blackboard.get_f32("guard_combat_range").unwrap_or(5.0);

                        if distance <= combat_range {
                            // 在攻击范围内，进行攻击
                            ctx.blackboard.set("movement_velocity", Vec3::ZERO);

                            // 守卫使用近战攻击
                            ctx.blackboard.set("trigger_attack", true);
                            ctx.blackboard.set("attack_type", "melee");

                            // 根据警报等级调整攻击强度
                            let alert_level = ctx
                                .blackboard
                                .get_string("alert_level")
                                .unwrap_or("medium".to_string());
                            let attack_damage = match alert_level.as_str() {
                                "critical" => 35.0,
                                "high" => 25.0,
                                _ => 15.0,
                            };

                            ctx.blackboard.set("attack_damage", attack_damage);

                            // 设置攻击动画
                            ctx.blackboard.set("animation_state", "attacking");

                            BehaviorStatus::Success
                        } else {
                            // 不在攻击范围内，移动到敌人
                            let direction = (enemy_pos - self_pos).normalize();
                            let base_speed = 3.0;
                            let speed_multiplier =
                                ctx.blackboard.get_f32("movement_speed_multiplier").unwrap_or(1.0);
                            let velocity = direction * base_speed * speed_multiplier;

                            ctx.blackboard.set("movement_velocity", velocity);

                            // 如果速度倍增器大于1.0，使用奔跑动画
                            if speed_multiplier > 1.0 {
                                ctx.blackboard.set("animation_state", "running");
                            } else {
                                ctx.blackboard.set("animation_state", "walking");
                            }

                            BehaviorStatus::Running
                        }
                    } else {
                        BehaviorStatus::Failure
                    }
                } else {
                    BehaviorStatus::Failure
                }
            }),
        );

        combat_sequence.add_child(Box::new(raise_alarm));
        combat_sequence.add_child(Box::new(engage));

        // 3. 巡逻序列
        let mut patrol_sequence = SequenceNode::new("Patrol Sequence");

        let move_to_next = ActionNode::new(
            "Move to Next Point",
            Box::new(|ctx| {
                // 移动到下一个巡逻点
                let patrol_route = ctx.blackboard.get_vec("patrol_route").unwrap_or(vec![]);

                if !patrol_route.is_empty() {
                    let current_index =
                        ctx.blackboard.get_usize("current_patrol_index").unwrap_or(0);
                    let target_index = (current_index + 1) % patrol_route.len();

                    if let Some(target_point) = patrol_route.get(target_index) {
                        if let Some(self_pos) = ctx.blackboard.get_vec3("self_position") {
                            let distance = (*target_point - self_pos).length();

                            if distance < 2.0 {
                                // 到达巡逻点
                                ctx.blackboard.set("movement_velocity", Vec3::ZERO);
                                ctx.blackboard.set("current_patrol_index", target_index);
                                ctx.blackboard.set("animation_state", "idle");

                                BehaviorStatus::Success
                            } else {
                                // 移动到巡逻点
                                let direction = (*target_point - self_pos).normalize();
                                let patrol_speed = 1.8; // 巡逻速度
                                let velocity = direction * patrol_speed;

                                ctx.blackboard.set("movement_velocity", velocity);
                                ctx.blackboard.set("animation_state", "patrolling");

                                BehaviorStatus::Running
                            }
                        } else {
                            BehaviorStatus::Failure
                        }
                    } else {
                        BehaviorStatus::Failure
                    }
                } else {
                    // 没有巡逻路线，原地待机
                    ctx.blackboard.set("movement_velocity", Vec3::ZERO);
                    ctx.blackboard.set("animation_state", "idle");
                    BehaviorStatus::Success
                }
            }),
        );

        let look_around = ActionNode::new(
            "Look Around",
            Box::new(|ctx| {
                // 环顾四周
                // 获取当前环顾方向
                let look_direction = ctx.blackboard.get_i32("look_direction_index").unwrap_or(0);

                // 设置环顾方向：0=前, 1=右, 2=后, 3=左
                let look_angles = [0.0, 90.0, 180.0, 270.0];
                let current_angle = look_angles[look_direction % 4];

                ctx.blackboard.set("look_angle", current_angle);
                ctx.blackboard.set("animation_state", "looking_around");

                // 设置下次环顾的方向
                let next_direction = (look_direction + 1) % 4;
                ctx.blackboard.set("look_direction_index", next_direction);

                // 如果完成了一圈，返回成功
                if next_direction == 0 {
                    // 重置环顾状态
                    ctx.blackboard.set("look_direction_index", 0);
                    BehaviorStatus::Success
                } else {
                    // 继续环顾
                    BehaviorStatus::Running
                }
            }),
        );

        patrol_sequence.add_child(Box::new(move_to_next));
        patrol_sequence.add_child(Box::new(look_around));

        selector.add_child(Box::new(check_intruder));
        selector.add_child(Box::new(combat_sequence));
        selector.add_child(Box::new(patrol_sequence));

        tree.set_root(Box::new(selector));
        tree
    }
}

impl Default for GuardBehavior {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AI组件预设
// ============================================================================

/// 寻路组件预设
#[derive(Debug, Clone)]
pub struct PathfindingPreset {
    /// 寻路算法类型
    pub algorithm: PathfindingAlgorithm,
    /// 是否允许动态障碍物
    pub dynamic_obstacles: bool,
    /// 路径平滑
    pub path_smoothing: bool,
}

/// 寻路算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathfindingAlgorithm {
    /// A*算法
    AStar,
    /// Dijkstra算法
    Dijkstra,
    /// 航点网格（NavMesh）
    NavMesh,
    /// 流场寻路（Flow Field）
    FlowField,
}

impl PathfindingPreset {
    pub fn new() -> Self {
        Self {
            algorithm: PathfindingAlgorithm::AStar,
            dynamic_obstacles: true,
            path_smoothing: true,
        }
    }

    /// 创建A*寻路预设
    pub fn a_star() -> Self {
        Self {
            algorithm: PathfindingAlgorithm::AStar,
            dynamic_obstacles: true,
            path_smoothing: true,
        }
    }

    /// 创建NavMesh寻路预设
    pub fn navmesh() -> Self {
        Self {
            algorithm: PathfindingAlgorithm::NavMesh,
            dynamic_obstacles: true,
            path_smoothing: false,
        }
    }

    /// 创建流场寻路预设（适合RTS大量单位）
    pub fn flow_field() -> Self {
        Self {
            algorithm: PathfindingAlgorithm::FlowField,
            dynamic_obstacles: false,
            path_smoothing: false,
        }
    }
}

impl Default for PathfindingPreset {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 感知组件预设
#[derive(Debug, Clone)]
pub struct PerceptionPreset {
    /// 视野距离
    pub sight_range: f32,
    /// 视野角度（度）
    pub sight_angle: f32,
    /// 听觉距离
    pub hearing_range: f32,
    /// 是否可以感知隐藏敌人
    pub detect_hidden: bool,
}

impl PerceptionPreset {
    pub fn new() -> Self {
        Self {
            sight_range: 20.0,
            sight_angle: 90.0,
            hearing_range: 10.0,
            detect_hidden: false,
        }
    }

    /// 创建人类感知预设
    pub fn human() -> Self {
        Self {
            sight_range: 15.0,
            sight_angle: 120.0,
            hearing_range: 5.0,
            detect_hidden: false,
        }
    }

    /// 创建动物感知预设
    pub fn animal() -> Self {
        Self {
            sight_range: 30.0,
            sight_angle: 270.0,
            hearing_range: 20.0,
            detect_hidden: false,
        }
    }

    /// 创建精英感知预设
    pub fn elite() -> Self {
        Self {
            sight_range: 40.0,
            sight_angle: 360.0,
            hearing_range: 15.0,
            detect_hidden: true,
        }
    }
}

impl Default for PerceptionPreset {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// 决策组件预设
#[derive(Debug, Clone)]
pub struct DecisionPreset {
    /// 决策风格
    pub style: DecisionStyle,
    /// 反应时间（秒）
    pub reaction_time: f32,
    /// 记忆持续时间（秒）
    pub memory_duration: f32,
}

/// 决策风格
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionStyle {
    /// 冲动型（快速反应，不考虑后果）
    Impulsive,
    /// 谨慎型（深思熟虑，规避风险）
    Cautious,
    /// 平衡型（综合考虑）
    Balanced,
    /// 侵略型（优先攻击）
    Aggressive,
    /// 防御型（优先防御）
    Defensive,
}

impl DecisionPreset {
    pub fn new() -> Self {
        Self {
            style: DecisionStyle::Balanced,
            reaction_time: 0.5,
            memory_duration: 30.0,
        }
    }

    /// 创建快速反应决策预设
    pub fn fast_reacter() -> Self {
        Self {
            style: DecisionStyle::Impulsive,
            reaction_time: 0.1,
            memory_duration: 10.0,
        }
    }

    /// 创建战术决策预设
    pub fn tactical() -> Self {
        Self {
            style: DecisionStyle::Cautious,
            reaction_time: 1.0,
            memory_duration: 60.0,
        }
    }
}

impl Default for DecisionPreset {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warrior_behavior_creation() {
        let warrior = WarriorBehavior::new();
        assert_eq!(warrior.aggressiveness, 0.7);
        assert_eq!(warrior.combat_range, 5.0);
        assert!(warrior.use_shield);
    }

    #[test]
    fn test_warrior_behavior_tree() {
        let warrior = WarriorBehavior::new();
        let tree = warrior.build_behavior_tree();
        assert_eq!(tree.name, "Warrior");
    }

    #[test]
    fn test_mage_behavior_creation() {
        let mage = MageBehavior::new();
        assert_eq!(mage.magic_type, MagicType::Fire);
        assert_eq!(mage.cast_range, 20.0);
    }

    #[test]
    fn test_archer_behavior_creation() {
        let archer = ArcherBehavior::new();
        assert_eq!(archer.shoot_range, 30.0);
        assert!(archer.mobile_shooting);
    }

    #[test]
    fn test_guard_patrol_route() {
        let mut guard = GuardBehavior::new();
        let route = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
        ];
        guard.set_patrol_route(route);
        assert_eq!(guard.patrol_route.len(), 3);
        assert_eq!(guard.current_patrol_index, 0);
    }

    #[test]
    fn test_pathfinding_presets() {
        let a_star = PathfindingPreset::a_star();
        assert_eq!(a_star.algorithm, PathfindingAlgorithm::AStar);

        let navmesh = PathfindingPreset::navmesh();
        assert_eq!(navmesh.algorithm, PathfindingAlgorithm::NavMesh);

        let flow_field = PathfindingPreset::flow_field();
        assert_eq!(flow_field.algorithm, PathfindingAlgorithm::FlowField);
    }

    #[test]
    fn test_perception_presets() {
        let human = PerceptionPreset::human();
        assert_eq!(human.sight_range, 15.0);

        let animal = PerceptionPreset::animal();
        assert_eq!(animal.sight_angle, 270.0);

        let elite = PerceptionPreset::elite();
        assert!(elite.detect_hidden);
    }

    #[test]
    fn test_decision_presets() {
        let fast = DecisionPreset::fast_reacter();
        assert_eq!(fast.style, DecisionStyle::Impulsive);
        assert_eq!(fast.reaction_time, 0.1);

        let tactical = DecisionPreset::tactical();
        assert_eq!(tactical.style, DecisionStyle::Cautious);
        assert_eq!(tactical.reaction_time, 1.0);
    }
}
