//! 状态机系统
//!
//! 实现有限状态机。

use super::*;
use std::collections::HashMap;

/// 状态 trait，定义状态的基本行为
pub trait State: Send + Sync {
    /// 进入状态时调用
    fn enter(&mut self);

    /// 更新状态，返回转换到的目标状态名称（如果需要转换）
    fn update(&mut self, delta_time: f32) -> Option<String>;

    /// 退出状态时调用
    fn exit(&mut self);
}

/// 状态转换结构体
pub struct Transition {
    /// 转换条件
    pub condition: String,
    /// 目标状态名称
    pub target_state: String,
}

/// 事件结构体，用于触发状态转换
pub struct Event {
    /// 事件名称
    pub name: String,
    /// 事件数据（可选）
    pub data: Option<String>,
}

/// 空闲状态结构体
pub struct IdleState;

impl State for IdleState {
    fn enter(&mut self) {
        tracing::debug!(target: "ai", "Entering idle state");
    }

    fn update(&mut self, _delta_time: f32) -> Option<String> {
        None
    }

    fn exit(&mut self) {
        tracing::debug!(target: "ai", "Exiting idle state");
    }
}

/// 行走状态结构体
pub struct WalkingState {
    steps: u32,
}

impl WalkingState {
    pub fn new() -> Self {
        Self { steps: 0 }
    }
}

impl State for WalkingState {
    fn enter(&mut self) {
        tracing::debug!(target: "ai", "Entering walking state");
    }

    fn update(&mut self, _delta_time: f32) -> Option<String> {
        self.steps += 1;
        if self.steps > 10 {
            Some("idle".to_string())
        } else {
            None
        }
    }

    fn exit(&mut self) {
        tracing::debug!(target: "ai", "Exiting walking state");
    }
}

/// 状态机结构体
pub struct StateMachine {
    /// 当前状态
    pub current_state: Option<Box<dyn State>>,
    /// 状态映射
    pub states: HashMap<String, Box<dyn State>>,
    /// 转换列表
    pub transitions: Vec<Transition>,
}

impl StateMachine {
    /// 创建新的状态机
    pub fn new() -> Self {
        Self {
            current_state: None,
            states: HashMap::new(),
            transitions: Vec::new(),
        }
    }

    /// 添加状态到映射
    pub fn add_state(&mut self, name: String, state: Box<dyn State>) {
        self.states.insert(name, state);
    }

    /// 设置初始状态
    pub fn set_initial_state(&mut self, name: &str) {
        if let Some(mut state) = self.states.remove(name) {
            if let Some(ref mut current) = self.current_state {
                current.exit();
            }
            state.enter();
            self.current_state = Some(state);
        }
    }

    /// 更新状态机
    pub fn update(&mut self, delta_time: f32) {
        if let Some(mut current) = self.current_state.take() {
            if let Some(next_state_name) = current.update(delta_time) {
                current.exit();
                if let Some(mut next_state) = self.states.remove(&next_state_name) {
                    next_state.enter();
                    self.current_state = Some(next_state);
                } else {
                    // 如果目标状态不存在，保持当前状态
                    self.current_state = Some(current);
                }
            } else {
                self.current_state = Some(current);
            }
        }
    }
}
