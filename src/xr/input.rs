//! XR 输入系统
//!
//! 实现控制器输入、手部追踪和触觉反馈

use crate::impl_default;
use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// XR 输入管理器
pub struct XrInputManager {
    /// 左控制器状态
    left_controller: Arc<Mutex<ControllerState>>,
    /// 右控制器状态
    right_controller: Arc<Mutex<ControllerState>>,
    /// 头部姿态
    head_pose: Arc<Mutex<Pose>>,
    /// 手部追踪数据（如果支持）
    hand_tracking: Option<HandTrackingData>,
    /// 触觉反馈队列
    haptic_queue: Vec<HapticFeedback>,
    /// 控制器连接状态
    controller_connected: HashMap<Hand, bool>,
}

/// 手部追踪数据
#[derive(Debug, Clone)]
pub struct HandTrackingData {
    /// 左手关节姿态
    left_hand_joints: Vec<HandJoint>,
    /// 右手关节姿态
    right_hand_joints: Vec<HandJoint>,
    /// 是否有效
    is_valid: bool,
}

/// 手部关节
#[derive(Debug, Clone)]
pub struct HandJoint {
    /// 关节类型
    pub joint_type: HandJointType,
    /// 姿态
    pub pose: Pose,
    /// 半径（用于碰撞检测）
    pub radius: f32,
    /// 是否有效
    pub is_valid: bool,
}

/// 手部关节类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandJointType {
    Palm,
    Wrist,
    ThumbMetacarpal,
    ThumbProximal,
    ThumbDistal,
    ThumbTip,
    IndexMetacarpal,
    IndexProximal,
    IndexIntermediate,
    IndexDistal,
    IndexTip,
    MiddleMetacarpal,
    MiddleProximal,
    MiddleIntermediate,
    MiddleDistal,
    MiddleTip,
    RingMetacarpal,
    RingProximal,
    RingIntermediate,
    RingDistal,
    RingTip,
    LittleMetacarpal,
    LittleProximal,
    LittleIntermediate,
    LittleDistal,
    LittleTip,
}

/// 触觉反馈请求
#[derive(Debug, Clone)]
pub struct HapticFeedback {
    /// 目标手
    pub hand: Hand,
    /// 振幅 (0.0 - 1.0)
    pub amplitude: f32,
    /// 持续时间 (纳秒)
    pub duration_ns: i64,
    /// 频率 (Hz, 可选)
    pub frequency: Option<f32>,
}

impl XrInputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新控制器状态
    pub fn update_controller(&mut self, hand: Hand, state: ControllerState) {
        let controller = match hand {
            Hand::Left => &self.left_controller,
            Hand::Right => &self.right_controller,
        };

        if let Ok(mut ctrl) = controller.lock() {
            *ctrl = state;
        }

        self.controller_connected.insert(hand, true);
    }

    /// 更新头部姿态
    pub fn update_head_pose(&mut self, pose: Pose) {
        if let Ok(mut head) = self.head_pose.lock() {
            *head = pose;
        }
    }

    /// 更新手部追踪数据
    pub fn update_hand_tracking(&mut self, data: HandTrackingData) {
        self.hand_tracking = Some(data);
    }

    /// 从HandTracker更新手部追踪数据
    pub fn update_from_hand_tracker(&mut self, tracker: &crate::xr::hand_tracking::HandTracker) {
        let mut left_joints = Vec::new();
        let mut right_joints = Vec::new();

        if let Some(left_hand) = tracker.get_hand_joints(Hand::Left) {
            for (joint_type, joint) in left_hand.get_all_joints() {
                left_joints.push(HandJoint {
                    joint_type: *joint_type,
                    pose: joint.pose,
                    radius: joint.radius,
                    is_valid: joint.is_valid,
                });
            }
        }

        if let Some(right_hand) = tracker.get_hand_joints(Hand::Right) {
            for (joint_type, joint) in right_hand.get_all_joints() {
                right_joints.push(HandJoint {
                    joint_type: *joint_type,
                    pose: joint.pose,
                    radius: joint.radius,
                    is_valid: joint.is_valid,
                });
            }
        }

        self.hand_tracking = Some(HandTrackingData {
            left_hand_joints: left_joints,
            right_hand_joints: right_joints,
            is_valid: tracker.is_tracking(Hand::Left) || tracker.is_tracking(Hand::Right),
        });
    }

    /// 获取控制器状态
    pub fn get_controller(&self, hand: Hand) -> Option<ControllerState> {
        let controller = match hand {
            Hand::Left => &self.left_controller,
            Hand::Right => &self.right_controller,
        };

        controller.lock().ok().map(|c| c.clone())
    }

    /// 获取头部姿态
    pub fn get_head_pose(&self) -> Pose {
        self.head_pose.lock().map(|p| *p).unwrap_or_default()
    }

    /// 获取手部追踪数据
    pub fn get_hand_tracking(&self) -> Option<&HandTrackingData> {
        self.hand_tracking.as_ref()
    }

    /// 检查控制器是否连接
    pub fn is_controller_connected(&self, hand: Hand) -> bool {
        self.controller_connected
            .get(&hand)
            .copied()
            .unwrap_or(false)
    }

    /// 添加触觉反馈
    pub fn add_haptic_feedback(&mut self, feedback: HapticFeedback) {
        self.haptic_queue.push(feedback);
    }

    /// 处理触觉反馈队列
    pub fn process_haptic_queue(&mut self) -> Vec<HapticFeedback> {
        std::mem::take(&mut self.haptic_queue)
    }

    /// 触发控制器震动
    pub fn vibrate(&mut self, hand: Hand, amplitude: f32, duration_ns: i64) {
        self.add_haptic_feedback(HapticFeedback {
            hand,
            amplitude: amplitude.clamp(0.0, 1.0),
            duration_ns,
            frequency: None,
        });
    }

    /// 检查按钮是否按下
    pub fn is_button_pressed(&self, hand: Hand, button: ControllerButton) -> bool {
        if let Some(state) = self.get_controller(hand) {
            match button {
                ControllerButton::A => state.buttons.a,
                ControllerButton::B => state.buttons.b,
                ControllerButton::X => state.buttons.x,
                ControllerButton::Y => state.buttons.y,
                ControllerButton::Menu => state.buttons.menu,
                ControllerButton::TriggerClick => state.buttons.trigger_click,
                ControllerButton::SqueezeClick => state.buttons.squeeze_click,
                ControllerButton::ThumbstickClick => state.buttons.thumbstick_click,
            }
        } else {
            false
        }
    }

    /// 获取触发器值
    pub fn get_trigger_value(&self, hand: Hand) -> f32 {
        self.get_controller(hand).map(|s| s.trigger).unwrap_or(0.0)
    }

    /// 获取握力值
    pub fn get_squeeze_value(&self, hand: Hand) -> f32 {
        self.get_controller(hand).map(|s| s.squeeze).unwrap_or(0.0)
    }

    /// 获取摇杆值
    pub fn get_thumbstick_value(&self, hand: Hand) -> [f32; 2] {
        self.get_controller(hand)
            .map(|s| s.thumbstick)
            .unwrap_or([0.0, 0.0])
    }

    /// 获取控制器姿态
    pub fn get_controller_pose(&self, hand: Hand) -> Option<Pose> {
        self.get_controller(hand).map(|s| s.pose)
    }

    /// 获取瞄准姿态（用于射线投射）
    pub fn get_aim_pose(&self, hand: Hand) -> Option<Pose> {
        self.get_controller(hand).map(|s| s.aim_pose)
    }

    /// 获取握持姿态
    pub fn get_grip_pose(&self, hand: Hand) -> Option<Pose> {
        self.get_controller(hand).map(|s| s.grip_pose)
    }
}

impl Default for XrInputManager {
    fn default() -> Self {
        Self {
            left_controller: Arc::new(Mutex::new(ControllerState::default())),
            right_controller: Arc::new(Mutex::new(ControllerState::default())),
            head_pose: Arc::new(Mutex::new(Pose::default())),
            hand_tracking: None,
            haptic_queue: Vec::new(),
            controller_connected: HashMap::new(),
        }
    }
}

/// 控制器按钮枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerButton {
    A,
    B,
    X,
    Y,
    Menu,
    TriggerClick,
    SqueezeClick,
    ThumbstickClick,
}

/// XR 输入事件
#[derive(Debug, Clone)]
pub enum XrInputEvent {
    /// 按钮按下
    ButtonPressed {
        hand: Hand,
        button: ControllerButton,
    },
    /// 按钮释放
    ButtonReleased {
        hand: Hand,
        button: ControllerButton,
    },
    /// 触发器值变化
    TriggerChanged { hand: Hand, value: f32 },
    /// 握力值变化
    SqueezeChanged { hand: Hand, value: f32 },
    /// 摇杆值变化
    ThumbstickChanged { hand: Hand, value: [f32; 2] },
    /// 控制器连接
    ControllerConnected { hand: Hand },
    /// 控制器断开
    ControllerDisconnected { hand: Hand },
    /// 手部追踪开始
    HandTrackingStarted,
    /// 手部追踪停止
    HandTrackingStopped,
}

/// XR 输入事件处理器
pub trait XrInputEventHandler: Send + Sync {
    fn handle_event(&mut self, event: &XrInputEvent);
}

/// 输入事件队列
#[derive(Default)]
pub struct XrInputEventQueue {
    events: Vec<XrInputEvent>,
    handlers: Vec<Box<dyn XrInputEventHandler>>,
}

impl XrInputEventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加事件
    pub fn push_event(&mut self, event: XrInputEvent) {
        self.events.push(event);
    }

    /// 注册事件处理器
    pub fn register_handler(&mut self, handler: Box<dyn XrInputEventHandler>) {
        self.handlers.push(handler);
    }

    /// 处理所有事件
    pub fn process_events(&mut self) {
        let events = std::mem::take(&mut self.events);

        for event in &events {
            for handler in &mut self.handlers {
                handler.handle_event(event);
            }
        }
    }

    /// 清空事件队列
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
