//  XR 输入系统
//
//  实现控制器输入、手部追踪和触觉反馈

use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// XR 输入管理器 - 管理所有XR输入源（控制器、手部追踪、头部追踪）
pub struct XrInputManager {
    /// 左控制器的当前状态（线程安全）
    left_controller: Arc<Mutex<ControllerState>>,
    /// 右控制器的当前状态（线程安全）
    right_controller: Arc<Mutex<ControllerState>>,
    /// 头部的当前姿态（线程安全）
    head_pose: Arc<Mutex<Pose>>,
    /// 手部追踪数据（如果系统支持手部追踪）
    hand_tracking: Option<HandTrackingData>,
    /// 待处理的触觉反馈队列
    haptic_queue: Vec<HapticFeedback>,
    /// 跟踪每个手部控制器的连接状态
    controller_connected: HashMap<Hand, bool>,
}

/// 手部追踪数据 - 包含来自手部追踪系统的关节信息
#[derive(Debug, Clone)]
pub struct HandTrackingData {
    /// 左手各关节的姿态和有效性信息
    left_hand_joints: Vec<HandJoint>,
    /// 右手各关节的姿态和有效性信息
    right_hand_joints: Vec<HandJoint>,
    /// 此追踪数据帧是否有效
    is_valid: bool,
}

impl HandTrackingData {
    /// 获取左手所有关节的切片
    ///
    /// # Returns
    /// 返回左手关节的切片引用
    pub fn left_hand_joints(&self) -> &[HandJoint] {
        &self.left_hand_joints
    }

    /// 获取右手所有关节的切片
    ///
    /// # Returns
    /// 返回右手关节的切片引用
    pub fn right_hand_joints(&self) -> &[HandJoint] {
        &self.right_hand_joints
    }

    /// 检查此追踪数据帧是否有效
    ///
    /// # Returns
    /// 如果追踪有效返回true，否则返回false
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// 获取指定手的关节数量
    ///
    /// # Arguments
    /// * `hand` - 要查询的手部（左或右）
    ///
    /// # Returns
    /// 返回该手部的关节总数
    pub fn joint_count(&self, hand: Hand) -> usize {
        match hand {
            Hand::Left => self.left_hand_joints.len(),
            Hand::Right => self.right_hand_joints.len(),
        }
    }
}

/// 手部关节 - 表示手部骨骼系统中的一个单个关节
#[derive(Debug, Clone)]
pub struct HandJoint {
    /// 关节的类型（例如拇指、食指等）
    pub joint_type: HandJointType,
    /// 关节在空间中的位置和旋转（世界坐标系）
    pub pose: Pose,
    /// 关节的半径，用于碰撞检测和交互范围计算（单位：米）
    pub radius: f32,
    /// 此关节的追踪是否有效（追踪丢失时为false）
    pub is_valid: bool,
}

/// 手部关节类型 - 遵循OpenXR Hand Tracking标准
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandJointType {
    /// 手掌中心
    Palm,
    /// 手腕
    Wrist,
    /// 大拇指掌骨
    ThumbMetacarpal,
    /// 大拇指近端指骨
    ThumbProximal,
    /// 大拇指远端指骨
    ThumbDistal,
    /// 大拇指顶端
    ThumbTip,
    /// 食指掌骨
    IndexMetacarpal,
    /// 食指近端指骨
    IndexProximal,
    /// 食指中间指骨
    IndexIntermediate,
    /// 食指远端指骨
    IndexDistal,
    /// 食指顶端
    IndexTip,
    /// 中指掌骨
    MiddleMetacarpal,
    /// 中指近端指骨
    MiddleProximal,
    /// 中指中间指骨
    MiddleIntermediate,
    /// 中指远端指骨
    MiddleDistal,
    /// 中指顶端
    MiddleTip,
    /// 无名指掌骨
    RingMetacarpal,
    /// 无名指近端指骨
    RingProximal,
    /// 无名指中间指骨
    RingIntermediate,
    /// 无名指远端指骨
    RingDistal,
    /// 无名指顶端
    RingTip,
    /// 小指掌骨
    LittleMetacarpal,
    /// 小指近端指骨
    LittleProximal,
    /// 小指中间指骨
    LittleIntermediate,
    /// 小指远端指骨
    LittleDistal,
    /// 小指顶端
    LittleTip,
}

/// 触觉反馈请求 - 描述一个控制器或手部设备的振动反馈
#[derive(Debug, Clone)]
pub struct HapticFeedback {
    /// 目标手部设备（左或右）
    pub hand: Hand,
    /// 振幅强度，范围为0.0到1.0（0.0=无振动，1.0=最大振幅）
    pub amplitude: f32,
    /// 反馈持续时间，单位为纳秒
    pub duration_ns: i64,
    /// 可选的振动频率（Hz），如果为None则使用设备默认频率
    pub frequency: Option<f32>,
}

impl XrInputManager {
    /// 创建新的输入管理器
    ///
    /// # Returns
    /// 返回一个新初始化的输入管理器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新控制器状态
    ///
    /// # Arguments
    /// * `hand` - 要更新的控制器手部（左或右）
    /// * `state` - 新的控制器状态
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
    ///
    /// # Arguments
    /// * `pose` - 头部的新姿态
    pub fn update_head_pose(&mut self, pose: Pose) {
        if let Ok(mut head) = self.head_pose.lock() {
            *head = pose;
        }
    }

    /// 更新手部追踪数据
    ///
    /// # Arguments
    /// * `data` - 新的手部追踪数据
    pub fn update_hand_tracking(&mut self, data: HandTrackingData) {
        self.hand_tracking = Some(data);
    }

    /// 从HandTracker更新手部追踪数据
    ///
    /// # Arguments
    /// * `tracker` - 手部追踪器引用
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
    ///
    /// # Arguments
    /// * `hand` - 要获取状态的控制器手部
    ///
    /// # Returns
    /// 如果控制器已连接返回Some(状态)，否则返回None
    pub fn get_controller(&self, hand: Hand) -> Option<ControllerState> {
        let controller = match hand {
            Hand::Left => &self.left_controller,
            Hand::Right => &self.right_controller,
        };

        controller.lock().ok().map(|c| c.clone())
    }

    /// 获取头部姿态
    ///
    /// # Returns
    /// 返回头部的当前姿态
    pub fn get_head_pose(&self) -> Pose {
        self.head_pose.lock().map(|p| *p).unwrap_or_default()
    }

    /// 获取手部追踪数据
    ///
    /// # Returns
    /// 如果有可用的手部追踪数据返回引用，否则返回None
    pub fn get_hand_tracking(&self) -> Option<&HandTrackingData> {
        self.hand_tracking.as_ref()
    }

    /// 检查控制器是否连接
    ///
    /// # Arguments
    /// * `hand` - 要检查的控制器手部
    ///
    /// # Returns
    /// 如果控制器已连接返回true，否则返回false
    pub fn is_controller_connected(&self, hand: Hand) -> bool {
        self.controller_connected.get(&hand).copied().unwrap_or(false)
    }

    /// 添加触觉反馈请求到队列
    ///
    /// # Arguments
    /// * `feedback` - 要添加的触觉反馈
    pub fn add_haptic_feedback(&mut self, feedback: HapticFeedback) {
        self.haptic_queue.push(feedback);
    }

    /// 处理触觉反馈队列并返回所有待处理的反馈
    ///
    /// # Returns
    /// 返回触觉反馈队列中的所有项并清空队列
    pub fn process_haptic_queue(&mut self) -> Vec<HapticFeedback> {
        std::mem::take(&mut self.haptic_queue)
    }

    /// 触发控制器简单的震动反馈
    ///
    /// # Arguments
    /// * `hand` - 要振动的控制器手部
    /// * `amplitude` - 振幅强度（0.0-1.0）
    /// * `duration_ns` - 持续时间（纳秒）
    pub fn vibrate(&mut self, hand: Hand, amplitude: f32, duration_ns: i64) {
        self.add_haptic_feedback(HapticFeedback {
            hand,
            amplitude: amplitude.clamp(0.0, 1.0),
            duration_ns,
            frequency: None,
        });
    }

    /// 检查按钮是否按下
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    /// * `button` - 要检查的按钮
    ///
    /// # Returns
    /// 如果按钮被按下返回true，否则返回false
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

    /// 获取触发器值（0.0-1.0）
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 返回触发器的当前值，如果控制器未连接返回0.0
    pub fn get_trigger_value(&self, hand: Hand) -> f32 {
        self.get_controller(hand).map(|s| s.trigger).unwrap_or(0.0)
    }

    /// 获取握力值（0.0-1.0）
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 返回握力的当前值，如果控制器未连接返回0.0
    pub fn get_squeeze_value(&self, hand: Hand) -> f32 {
        self.get_controller(hand).map(|s| s.squeeze).unwrap_or(0.0)
    }

    /// 获取摇杆值
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 返回摇杆的[x, y]值，范围-1.0到1.0；如果控制器未连接返回[0.0, 0.0]
    pub fn get_thumbstick_value(&self, hand: Hand) -> [f32; 2] {
        self.get_controller(hand).map(|s| s.thumbstick).unwrap_or([0.0, 0.0])
    }

    /// 获取控制器姿态
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 如果控制器已连接返回Some(姿态)，否则返回None
    pub fn get_controller_pose(&self, hand: Hand) -> Option<Pose> {
        self.get_controller(hand).map(|s| s.pose)
    }

    /// 获取瞄准姿态（用于射线投射和指向）
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 如果控制器已连接返回Some(瞄准姿态)，否则返回None
    pub fn get_aim_pose(&self, hand: Hand) -> Option<Pose> {
        self.get_controller(hand).map(|s| s.aim_pose)
    }

    /// 获取握持姿态（用于拿取物体）
    ///
    /// # Arguments
    /// * `hand` - 控制器手部
    ///
    /// # Returns
    /// 如果控制器已连接返回Some(握持姿态)，否则返回None
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

/// 控制器按钮枚举 - 标准XR控制器按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerButton {
    /// A 按钮（右手控制器）
    A,
    /// B 按钮（右手控制器）
    B,
    /// X 按钮（左手控制器）
    X,
    /// Y 按钮（左手控制器）
    Y,
    /// 菜单按钮
    Menu,
    /// 触发器点击（完全按下）
    TriggerClick,
    /// 握力点击（完全握紧）
    SqueezeClick,
    /// 摇杆按下
    ThumbstickClick,
}

/// XR 输入事件 - 表示来自控制器和手部追踪的各类输入事件
#[derive(Debug, Clone)]
pub enum XrInputEvent {
    /// 按钮按下事件
    ButtonPressed {
        /// 按下按钮的手部
        hand: Hand,
        /// 被按下的按钮
        button: ControllerButton,
    },
    /// 按钮释放事件
    ButtonReleased {
        /// 释放按钮的手部
        hand: Hand,
        /// 被释放的按钮
        button: ControllerButton,
    },
    /// 触发器值变化事件（0.0-1.0）
    TriggerChanged {
        /// 触发器所在的手部
        hand: Hand,
        /// 新的触发器值
        value: f32,
    },
    /// 握力值变化事件（0.0-1.0）
    SqueezeChanged {
        /// 握力传感器所在的手部
        hand: Hand,
        /// 新的握力值
        value: f32,
    },
    /// 摇杆值变化事件
    ThumbstickChanged {
        /// 摇杆所在的手部
        hand: Hand,
        /// 摇杆的[x, y]值范围-1.0到1.0
        value: [f32; 2],
    },
    /// 控制器连接事件
    ControllerConnected {
        /// 连接的控制器手部
        hand: Hand,
    },
    /// 控制器断开连接事件
    ControllerDisconnected {
        /// 断开的控制器手部
        hand: Hand,
    },
    /// 手部追踪开始事件
    HandTrackingStarted,
    /// 手部追踪停止事件
    HandTrackingStopped,
}

/// XR 输入事件处理器 - 实现此trait以接收和处理输入事件
pub trait XrInputEventHandler: Send + Sync {
    /// 处理一个输入事件
    ///
    /// # Arguments
    /// * `event` - 要处理的输入事件引用
    fn handle_event(&mut self, event: &XrInputEvent);
}

/// 输入事件队列 - 管理输入事件的收集和分发
#[derive(Default)]
pub struct XrInputEventQueue {
    /// 待处理的事件队列
    events: Vec<XrInputEvent>,
    /// 注册的事件处理器列表
    handlers: Vec<Box<dyn XrInputEventHandler>>,
}

impl XrInputEventQueue {
    /// 创建新的输入事件队列
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加事件到队列
    ///
    /// # Arguments
    /// * `event` - 要添加的输入事件
    pub fn push_event(&mut self, event: XrInputEvent) {
        self.events.push(event);
    }

    /// 注册事件处理器
    ///
    /// # Arguments
    /// * `handler` - 实现XrInputEventHandler的事件处理器
    pub fn register_handler(&mut self, handler: Box<dyn XrInputEventHandler>) {
        self.handlers.push(handler);
    }

    /// 处理队列中的所有事件并分发给已注册的处理器
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
