//! 移动平台输入系统
//!
//! 提供多点触控、手势识别和虚拟控制器的移动输入支持。

use bevy_ecs::prelude::*;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 触摸输入事件
#[derive(Debug, Clone, Event)]
pub enum TouchEvent {
    /// 触摸开始
    Started {
        touch_id: u64,
        position: Vec2,
    },
    /// 触摸移动
    Moved {
        touch_id: u64,
        position: Vec2,
        delta: Vec2,
    },
    /// 触摸结束
    Ended {
        touch_id: u64,
        position: Vec2,
    },
    /// 触摸取消
    Cancelled {
        touch_id: u64,
    },
}

/// 手势类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureType {
    /// 点击
    Tap,
    /// 双击
    DoubleTap,
    /// 长按
    LongPress,
    /// 滑动
    Swipe {
        direction: SwipeDirection,
    },
    /// 缩放（双指捏合）
    Pinch {
        scale: f32,
    },
    /// 旋转（双指旋转）
    Rotation {
        angle: f32,
    },
}

/// 滑动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 手势事件
#[derive(Debug, Clone, Event)]
pub struct GestureEvent {
    /// 手势类型
    pub gesture_type: GestureType,
    /// 位置
    pub position: Vec2,
    /// 手势参数
    pub parameters: HashMap<String, f32>,
}

/// 手势识别器
///
/// 识别各种触摸手势。
#[derive(Component)]
pub struct GestureRecognizer {
    /// 当前活动的触摸点
    active_touches: HashMap<u64, TouchState>,

    /// 点击配置
    tap_config: TapConfig,
    /// 滑动配置
    swipe_config: SwipeConfig,
    /// 缩放配置
    pinch_config: PinchConfig,
}

/// 触摸状态
#[derive(Debug, Clone)]
struct TouchState {
    /// 初始位置
    start_position: Vec2,
    /// 当前位置
    current_position: Vec2,
    /// 开始时间
    start_time: f64,
    /// 上次移动时间
    last_move_time: f64,
}

/// 点击配置
#[derive(Debug, Clone)]
pub struct TapConfig {
    /// 最大移动距离（像素）
    pub max_movement: f32,
    /// 最大持续时间（秒）
    pub max_duration: f64,
    /// 双击间隔（秒）
    pub double_tap_interval: f64,
}

impl Default for TapConfig {
    fn default() -> Self {
        Self {
            max_movement: 10.0,
            max_duration: 0.3,
            double_tap_interval: 0.3,
        }
    }
}

/// 滑动配置
#[derive(Debug, Clone)]
pub struct SwipeConfig {
    /// 最小滑动距离
    pub min_distance: f32,
    /// 最大滑动时间
    pub max_duration: f64,
    /// 方向阈值（角度）
    pub direction_threshold: f32,
}

impl Default for SwipeConfig {
    fn default() -> Self {
        Self {
            min_distance: 50.0,
            max_duration: 1.0,
            direction_threshold: 30.0,
        }
    }
}

/// 缩放配置
#[derive(Debug, Clone)]
pub struct PinchConfig {
    /// 最小缩放距离
    pub min_distance: f32,
    /// 最大缩放距离
    pub max_distance: f32,
}

impl Default for PinchConfig {
    fn default() -> Self {
        Self {
            min_distance: 10.0,
            max_distance: 500.0,
        }
    }
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            active_touches: HashMap::new(),
            tap_config: TapConfig::default(),
            swipe_config: SwipeConfig::default(),
            pinch_config: PinchConfig::default(),
        }
    }

    /// 处理触摸事件
    pub fn handle_touch(&mut self, event: &TouchEvent) -> Option<GestureEvent> {
        match event {
            TouchEvent::Started { touch_id, position } => {
                let state = TouchState {
                    start_position: *position,
                    current_position: *position,
                    start_time: crate::core::utils::current_timestamp_f64(),
                    last_move_time: crate::core::utils::current_timestamp_f64(),
                };
                self.active_touches.insert(*touch_id, state);

                // 检查是否触发双击
                if self.check_double_tap(position) {
                    return Some(GestureEvent {
                        gesture_type: GestureType::DoubleTap,
                        position: *position,
                        parameters: HashMap::new(),
                    });
                }
            }
            TouchEvent::Moved { touch_id, position, delta } => {
                if let Some(state) = self.active_touches.get_mut(touch_id) {
                    state.current_position = *position;
                    state.last_move_time = crate::core::utils::current_timestamp_f64();
                }

                // 检查多指手势
                if self.active_touches.len() >= 2 {
                    if let Some(gesture) = self.check_multi_touch_gesture() {
                        return Some(gesture);
                    }
                }
            }
            TouchEvent::Ended { touch_id, position } => {
                if let Some(state) = self.active_touches.remove(touch_id) {
                    // 检查点击
                    if let Some(gesture) = self.check_tap(state, position) {
                        return Some(gesture);
                    }

                    // 检查滑动
                    if let Some(gesture) = self.check_swipe(state, position) {
                        return Some(gesture);
                    }
                }

                // 检查长按
                if let Some(gesture) = self.check_long_press(touch_id) {
                    return Some(gesture);
                }
            }
            TouchEvent::Cancelled { touch_id } => {
                self.active_touches.remove(touch_id);
            }
        }

        None
    }

    fn check_tap(&self, state: &TouchState, position: &Vec2) -> Option<GestureEvent> {
        let duration = crate::core::utils::current_timestamp_f64() - state.start_time;
        let distance = state.current_position.distance(*position);

        if duration < self.tap_config.max_duration && distance < self.tap_config.max_movement {
            Some(GestureEvent {
                gesture_type: GestureType::Tap,
                position: *position,
                parameters: HashMap::new(),
            })
        } else {
            None
        }
    }

    fn check_double_tap(&self, position: &Vec2) -> bool {
        // TODO: 实现双击检测
        false
    }

    fn check_long_press(&self, touch_id: &u64) -> Option<GestureEvent> {
        if let Some(state) = self.active_touches.get(touch_id) {
            let duration = crate::core::utils::current_timestamp_f64() - state.start_time;
            if duration > 0.5 {
                return Some(GestureEvent {
                    gesture_type: GestureType::LongPress,
                    position: state.current_position,
                    parameters: HashMap::new(),
                });
            }
        }
        None
    }

    fn check_swipe(&self, state: &TouchState, position: &Vec2) -> Option<GestureEvent> {
        let duration = crate::core::utils::current_timestamp_f64() - state.start_time;
        let delta = *position - state.start_position;

        if duration < self.swipe_config.max_duration {
            if delta.length() > self.swipe_config.min_distance {
                let angle = delta.y.atan2(delta.x).to_degrees();

                let direction = if angle.abs() < 45.0 {
                    if angle > 0.0 {
                        SwipeDirection::Right
                    } else {
                        SwipeDirection::Left
                    }
                } else {
                    if angle > 0.0 {
                        SwipeDirection::Down
                    } else {
                        SwipeDirection::Up
                    }
                };

                return Some(GestureEvent {
                    gesture_type: GestureType::Swipe { direction },
                    position: *position,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("distance".to_string(), delta.length());
                        params.insert("angle".to_string(), angle);
                        params
                    },
                });
            }
        }

        None
    }

    fn check_multi_touch_gesture(&self) -> Option<GestureEvent> {
        let touches: Vec<_> = self.active_touches.values().collect();

        if touches.len() == 2 {
            let touch0 = &touches[0];
            let touch1 = &touches[1];

            // 计算两点间距离
            let distance = touch0.current_position.distance(touch1.current_position);

            // 计算缩放
            let start_distance = touch0.start_position.distance(touch1.start_position);
            if start_distance > 1.0 {
                let scale = distance / start_distance;

                return Some(GestureEvent {
                    gesture_type: GestureType::Pinch { scale },
                    position: (touch0.current_position + touch1.current_position) / 2.0,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("scale".to_string(), scale);
                        params
                    },
                });
            }

            // 计算旋转
            let start_vector = touch1.start_position - touch0.start_position;
            let current_vector = touch1.current_position - touch0.current_position;
            let angle = start_vector.angle_to(current_vector);

            return Some(GestureEvent {
                gesture_type: GestureType::Rotation { angle },
                position: (touch0.current_position + touch1.current_position) / 2.0,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("angle".to_string(), angle.to_degrees());
                    params
                },
            });
        }

        None
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 虚拟摇杆
#[derive(Component)]
pub struct VirtualJoystick {
    /// 摇杆ID
    pub id: String,
    /// 位置
    pub position: Vec2,
    /// 大小
    pub size: f32,
    /// 当前摇杆值
    pub value: Vec2,
    /// 触摸点ID
    pub touch_id: Option<u64>,
    /// 是否激活
    pub active: bool,
}

impl VirtualJoystick {
    /// 创建新的虚拟摇杆
    pub fn new(id: String, position: Vec2, size: f32) -> Self {
        Self {
            id,
            position,
            size,
            value: Vec2::ZERO,
            touch_id: None,
            active: false,
        }
    }

    /// 处理触摸输入
    pub fn handle_touch(&mut self, event: &TouchEvent) -> bool {
        match event {
            TouchEvent::Started { touch_id, position } => {
                if !self.active && self.contains_point(*position) {
                    self.touch_id = Some(*touch_id);
                    self.active = true;
                    self.update_value(*position);
                    return true;
                }
            }
            TouchEvent::Moved { touch_id, position, .. } => {
                if self.touch_id == Some(*touch_id) && self.active {
                    self.update_value(*position);
                    return true;
                }
            }
            TouchEvent::Ended { touch_id, .. } | TouchEvent::Cancelled { touch_id, .. } => {
                if self.touch_id == Some(*touch_id) {
                    self.touch_id = None;
                    self.active = false;
                    self.value = Vec2::ZERO;
                    return true;
                }
            }
        }

        false
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let half_size = self.size / 2.0;
        let min = self.position - half_size;
        let max = self.position + half_size;

        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    fn update_value(&mut self, position: Vec2) {
        let delta = position - self.position;
        let max_distance = self.size / 2.0;

        if delta.length() > max_distance {
            self.value = delta.normalize() * max_distance / max_distance;
        } else {
            self.value = delta / max_distance;
        }

        // 限制在单位圆内
        if self.value.length() > 1.0 {
            self.value = self.value.normalize();
        }
    }
}

/// 虚拟按钮
#[derive(Component)]
pub struct VirtualButton {
    /// 按钮ID
    pub id: String,
    /// 位置
    pub position: Vec2,
    /// 大小
    pub size: Vec2,
    /// 触摸点ID
    pub touch_id: Option<u64>,
    /// 是否按下
    pub pressed: bool,
    /// 按钮标签
    pub label: String,
}

impl VirtualButton {
    /// 创建新的虚拟按钮
    pub fn new(id: String, position: Vec2, size: Vec2, label: String) -> Self {
        Self {
            id,
            position,
            size,
            touch_id: None,
            pressed: false,
            label,
        }
    }

    /// 处理触摸输入
    pub fn handle_touch(&mut self, event: &TouchEvent) -> bool {
        match event {
            TouchEvent::Started { touch_id, position } => {
                if self.contains_point(*position) {
                    self.touch_id = Some(*touch_id);
                    self.pressed = true;
                    return true;
                }
            }
            TouchEvent::Ended { touch_id, .. } | TouchEvent::Cancelled { touch_id, .. } => {
                if self.touch_id == Some(*touch_id) {
                    self.touch_id = None;
                    self.pressed = false;
                    return true;
                }
            }
            TouchEvent::Moved { .. } => {}
        }

        false
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let half_size = self.size / 2.0;
        let min = self.position - half_size;
        let max = self.position + half_size;

        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}

/// 移动输入管理器
///
/// 管理所有移动平台的输入组件。
#[derive(Resource)]
pub struct MobileInputManager {
    /// 手势识别器
    pub gesture_recognizer: GestureRecognizer,
    /// 虚拟摇杆列表
    pub joysticks: HashMap<String, VirtualJoystick>,
    /// 虚拟按钮列表
    pub buttons: HashMap<String, VirtualButton>,
}

impl MobileInputManager {
    /// 创建新的移动输入管理器
    pub fn new() -> Self {
        Self {
            gesture_recognizer: GestureRecognizer::new(),
            joysticks: HashMap::new(),
            buttons: HashMap::new(),
        }
    }

    /// 添加虚拟摇杆
    pub fn add_joystick(&mut self, joystick: VirtualJoystick) {
        self.joysticks.insert(joystick.id.clone(), joystick);
    }

    /// 添加虚拟按钮
    pub fn add_button(&mut self, button: VirtualButton) {
        self.buttons.insert(button.id.clone(), button);
    }

    /// 处理触摸事件
    pub fn handle_touch(&mut self, event: &TouchEvent) {
        // 处理手势
        if let Some(gesture_event) = self.gesture_recognizer.handle_touch(event) {
            // 发出手势事件
        }

        // 处理虚拟摇杆
        for joystick in self.joysticks.values_mut() {
            joystick.handle_touch(event);
        }

        // 处理虚拟按钮
        for button in self.buttons.values_mut() {
            button.handle_touch(event);
        }
    }
}

impl Default for MobileInputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_joystick_creation() {
        let joystick = VirtualJoystick::new(
            "joystick_1".to_string(),
            Vec2::new(100.0, 100.0),
            100.0,
        );

        assert_eq!(joystick.id, "joystick_1");
        assert!(!joystick.active);
    }

    #[test]
    fn test_virtual_button_creation() {
        let button = VirtualButton::new(
            "button_1".to_string(),
            Vec2::new(200.0, 100.0),
            Vec2::new(50.0, 50.0),
            "Jump".to_string(),
        );

        assert_eq!(button.label, "Jump");
        assert!(!button.pressed);
    }

    #[test]
    fn test_gesture_recognizer() {
        let recognizer = GestureRecognizer::new();
        assert_eq!(recognizer.active_touches.len(), 0);
    }
}
