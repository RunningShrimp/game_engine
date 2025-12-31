//! 动画状态机系统
//!
//! 实现复杂的动画状态机，支持状态转换、混合参数、状态层和遮罩。

use super::blending::{BlendSpace1D, BlendSpace2D};
use crate::animation::{AnimationClip, InterpolationMode};
use bevy_ecs::prelude::*;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 动画状态机
#[derive(Debug, Clone, Component)]
pub struct AnimationStateMachine {
    /// 状态机ID
    pub id: String,

    /// 当前状态
    pub current_state: String,

    /// 所有状态
    pub states: HashMap<String, AnimationState>,

    /// 所有转换
    pub transitions: Vec<StateTransition>,

    /// 混合参数
    pub parameters: HashMap<String, Parameter>,

    /// 状态层
    pub layers: Vec<AnimationLayer>,

    /// 状态遮罩
    pub avatar_mask: Option<AvatarMask>,

    /// 是否启用
    pub enabled: bool,

    /// 当前时间（秒）
    pub current_time: f32,

    /// 播放速度
    pub playback_speed: f32,
}

impl AnimationStateMachine {
    /// 创建新的状态机
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_state: "idle".to_string(),
            states: HashMap::new(),
            transitions: Vec::new(),
            parameters: HashMap::new(),
            layers: vec![AnimationLayer::default()],
            avatar_mask: None,
            enabled: true,
            current_time: 0.0,
            playback_speed: 1.0,
        }
    }

    /// 添加状态
    pub fn add_state(&mut self, state: AnimationState) {
        self.states.insert(state.name.clone(), state);
    }

    /// 添加转换
    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }

    /// 添加参数
    pub fn add_parameter(&mut self, name: String, parameter: Parameter) {
        self.parameters.insert(name, parameter);
    }

    /// 设置参数值
    pub fn set_parameter(&mut self, name: &str, value: ParameterValue) {
        if let Some(param) = self.parameters.get_mut(name) {
            param.value = value;
        }
    }

    /// 获取参数值
    pub fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
        self.parameters.get(name).map(|p| p.value.clone())
    }

    /// 触发转换
    pub fn trigger(&mut self, trigger_name: &str) {
        for transition in &self.transitions {
            if transition.from_state == self.current_state {
                if let TransitionCondition::Trigger(name) = &transition.condition {
                    if name == trigger_name {
                        self.transition_to(transition.to_state.clone());
                        break;
                    }
                }
            }
        }
    }

    /// 更新状态机
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }

        self.current_time += delta_time * self.playback_speed;

        // 检查自动转换
        for transition in &self.transitions {
            if transition.from_state == self.current_state {
                if self.check_transition_condition(transition) {
                    self.transition_to(transition.to_state.clone());
                    break;
                }
            }
        }

        // 更新当前状态
        if let Some(state) = self.states.get_mut(&self.current_state) {
            state.update(delta_time * self.playback_speed);
        }
    }

    /// 检查转换条件
    fn check_transition_condition(&self, transition: &StateTransition) -> bool {
        match &transition.condition {
            TransitionCondition::Always => true,
            TransitionCondition::Trigger(_) => false,
            TransitionCondition::Parameter {
                name,
                operator,
                value,
            } => {
                if let Some(param) = self.parameters.get(name) {
                    match operator {
                        ParameterOperator::Equals => &param.value == value,
                        ParameterOperator::NotEquals => &param.value != value,
                        ParameterOperator::Greater => {
                            if let (ParameterValue::Float(a), ParameterValue::Float(b)) =
                                (&param.value, value)
                            {
                                a > b
                            } else {
                                false
                            }
                        }
                        ParameterOperator::Less => {
                            if let (ParameterValue::Float(a), ParameterValue::Float(b)) =
                                (&param.value, value)
                            {
                                a < b
                            } else {
                                false
                            }
                        }
                        ParameterOperator::GreaterEquals => {
                            if let (ParameterValue::Float(a), ParameterValue::Float(b)) =
                                (&param.value, value)
                            {
                                a >= b
                            } else {
                                false
                            }
                        }
                        ParameterOperator::LessEquals => {
                            if let (ParameterValue::Float(a), ParameterValue::Float(b)) =
                                (&param.value, value)
                            {
                                a <= b
                            } else {
                                false
                            }
                        }
                    }
                } else {
                    false
                }
            }
            TransitionCondition::AnimationEnd => {
                if let Some(state) = self.states.get(&self.current_state) {
                    state.is_finished()
                } else {
                    false
                }
            }
        }
    }

    /// 转换到新状态
    fn transition_to(&mut self, new_state: String) {
        if let Some(state) = self.states.get(&new_state) {
            if let Some(exit_action) = &state.on_exit {
                // 执行退出动作
            }

            if let Some(old_state) = self.states.get(&self.current_state) {
                if let Some(enter_action) = &old_state.on_enter {
                    // 执行进入动作
                }
            }

            self.current_state = new_state;
            self.current_time = 0.0;
        }
    }

    /// 获取当前动画姿势
    pub fn get_current_pose(&self) -> HashMap<String, Vec3> {
        if let Some(state) = self.states.get(&self.current_state) {
            state.get_pose(self.current_time)
        } else {
            HashMap::new()
        }
    }
}

/// 动画状态
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// 状态名称
    pub name: String,

    /// 状态类型
    pub state_type: AnimationStateType,

    /// 速度
    pub speed: f32,

    /// 进入动作
    pub on_enter: Option<String>,

    /// 退出动作
    pub on_exit: Option<String>,

    /// 动画剪辑
    pub clip: Option<Arc<AnimationClip>>,

    /// 混合树
    pub blend_tree: Option<super::blending::AnimationBlendTree>,

    /// 1D混合空间
    pub blend_space_1d: Option<BlendSpace1D>,

    /// 2D混合空间
    pub blend_space_2d: Option<BlendSpace2D>,

    /// 当前时间
    pub current_time: f32,

    /// 是否循环
    pub looping: bool,
}

impl AnimationState {
    /// 创建新的动画状态
    pub fn new(name: String) -> Self {
        Self {
            name,
            state_type: AnimationStateType::Motion,
            speed: 1.0,
            on_enter: None,
            on_exit: None,
            clip: None,
            blend_tree: None,
            blend_space_1d: None,
            blend_space_2d: None,
            current_time: 0.0,
            looping: false,
        }
    }

    /// 使用动画剪辑创建状态
    pub fn from_clip(name: String, clip: Arc<AnimationClip>) -> Self {
        Self {
            name,
            state_type: AnimationStateType::Motion,
            speed: 1.0,
            on_enter: None,
            on_exit: None,
            clip: Some(clip),
            blend_tree: None,
            blend_space_1d: None,
            blend_space_2d: None,
            current_time: 0.0,
            looping: false,
        }
    }

    /// 更新状态
    pub fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time * self.speed;

        if let Some(clip) = &self.clip {
            if self.current_time >= clip.duration {
                if clip.looping || self.looping {
                    self.current_time %= clip.duration;
                } else {
                    self.current_time = clip.duration;
                }
            }
        }
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        if let Some(clip) = &self.clip {
            if !clip.looping && !self.looping {
                return self.current_time >= clip.duration;
            }
        }
        false
    }

    /// 获取当前姿势
    pub fn get_pose(&self, time: f32) -> HashMap<String, Vec3> {
        if let Some(clip) = &self.clip {
            if let Some(pos) = clip.evaluate_position(time) {
                let mut pose = HashMap::new();
                pose.insert(clip.name.clone(), pos);
                return pose;
            }
        }
        HashMap::new()
    }
}

/// 动画状态类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationStateType {
    /// 运动状态
    Motion,
    /// 混合状态
    BlendTree,
    /// 叠加状态
    Additive,
}

/// 状态转换
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// 源状态
    pub from_state: String,

    /// 目标状态
    pub to_state: String,

    /// 转换条件
    pub condition: TransitionCondition,

    /// 转换持续时间（秒）
    pub duration: f32,

    /// 转换偏移（秒）
    pub offset: f32,

    /// 是否可中断
    pub can_interrupt: bool,

    /// 退出时间（秒）
    pub exit_time: Option<f32>,
}

impl StateTransition {
    /// 创建新的状态转换
    pub fn new(from_state: String, to_state: String, condition: TransitionCondition) -> Self {
        Self {
            from_state,
            to_state,
            condition,
            duration: 0.2,
            offset: 0.0,
            can_interrupt: true,
            exit_time: None,
        }
    }

    /// 设置转换持续时间
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// 设置可中断
    pub fn with_interrupt(mut self, can_interrupt: bool) -> Self {
        self.can_interrupt = can_interrupt;
        self
    }
}

/// 转换条件
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    /// 总是转换
    Always,
    /// 触发器
    Trigger(String),
    /// 参数条件
    Parameter {
        name: String,
        operator: ParameterOperator,
        value: ParameterValue,
    },
    /// 动画结束
    AnimationEnd,
}

/// 参数操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterOperator {
    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
}

/// 参数
#[derive(Debug, Clone)]
pub struct Parameter {
    /// 参数名称
    pub name: String,

    /// 参数值
    pub value: ParameterValue,

    /// 参数类型
    pub param_type: ParameterType,
}

/// 参数值
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Trigger,
}

/// 参数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    Float,
    Int,
    Bool,
    Trigger,
}

/// 动画层
#[derive(Debug, Clone)]
pub struct AnimationLayer {
    /// 层名称
    pub name: String,

    /// 层权重
    pub weight: f32,

    /// 混合模式
    pub blending_mode: LayerBlendingMode,

    /// 层状态机
    pub state_machine: Option<AnimationStateMachine>,

    /// 层遮罩
    pub avatar_mask: Option<AvatarMask>,
}

impl Default for AnimationLayer {
    fn default() -> Self {
        Self {
            name: "Base Layer".to_string(),
            weight: 1.0,
            blending_mode: LayerBlendingMode::Override,
            state_machine: None,
            avatar_mask: None,
        }
    }
}

/// 层混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerBlendingMode {
    /// 覆盖
    Override,
    /// 叠加
    Additive,
}

/// Avatar遮罩
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarMask {
    /// 遮罩名称
    pub name: String,

    /// 骨骼权重
    pub bone_weights: HashMap<String, f32>,

    /// 人体部位（Humanoid）
    pub human_bones: Option<HumanoidBones>,
}

impl AvatarMask {
    /// 创建新的Avatar遮罩
    pub fn new(name: String) -> Self {
        Self {
            name,
            bone_weights: HashMap::new(),
            human_bones: None,
        }
    }

    /// 设置骨骼权重
    pub fn set_bone_weight(&mut self, bone_name: String, weight: f32) {
        self.bone_weights.insert(bone_name, weight.clamp(0.0, 1.0));
    }

    /// 获取骨骼权重
    pub fn get_bone_weight(&self, bone_name: &str) -> f32 {
        self.bone_weights.get(bone_name).copied().unwrap_or(1.0)
    }
}

/// Humanoid骨骼
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanoidBones {
    /// 头部
    pub head: f32,
    /// 左臂
    pub left_arm: f32,
    /// 右臂
    pub right_arm: f32,
    /// 左腿
    pub left_leg: f32,
    /// 右腿
    pub right_leg: f32,
    /// 躯干
    pub body: f32,
}

impl Default for HumanoidBones {
    fn default() -> Self {
        Self {
            head: 1.0,
            left_arm: 1.0,
            right_arm: 1.0,
            left_leg: 1.0,
            right_leg: 1.0,
            body: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let sm = AnimationStateMachine::new("test_sm".to_string());
        assert_eq!(sm.current_state, "idle");
    }

    #[test]
    fn test_state_creation() {
        let state = AnimationState::new("walk".to_string());
        assert_eq!(state.name, "walk");
    }

    #[test]
    fn test_parameter_value() {
        let float_val = ParameterValue::Float(1.5);
        let int_val = ParameterValue::Int(42);
        let bool_val = ParameterValue::Bool(true);

        assert_eq!(float_val, ParameterValue::Float(1.5));
        assert_ne!(float_val, int_val);
    }

    #[test]
    fn test_avatar_mask() {
        let mut mask = AvatarMask::new("upper_body".to_string());
        mask.set_bone_weight("spine".to_string(), 1.0);
        mask.set_bone_weight("left_leg".to_string(), 0.0);

        assert_eq!(mask.get_bone_weight("spine"), 1.0);
        assert_eq!(mask.get_bone_weight("left_leg"), 0.0);
        assert_eq!(mask.get_bone_weight("unknown"), 1.0);
    }

    #[test]
    fn test_humanoid_bones_default() {
        let bones = HumanoidBones::default();
        assert_eq!(bones.head, 1.0);
        assert_eq!(bones.left_arm, 1.0);
    }
}
