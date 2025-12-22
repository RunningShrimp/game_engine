//  OpenXR 手部追踪扩展集成
// 
//  实现OpenXR手部追踪扩展（XR_EXT_hand_tracking），提供高精度手部关节追踪。
// 
//  ## 功能特性
// 
//  - 26个手部关节追踪（每只手）
//  - 关节位置、旋转和半径
//  - 手部姿态有效性检测
//  - 双手独立追踪
// 
//  ## 使用示例
// 
//  ```rust
//  use crate::xr::hand_tracking::*;
// 
//  // 初始化手部追踪
//  let mut hand_tracker = HandTracker::new()?;
// 
//  // 更新追踪数据
//  hand_tracker.update()?;
// 
//  // 获取左手关节数据
//  if let Some(joints) = hand_tracker.get_hand_joints(Hand::Left) {
//      if let Some(palm) = joints.get_joint(HandJointType::Palm) {
//          println!("Palm position: {:?}", palm.pose.position);
//      }
//  }
//  ```

use super::*;
use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use std::collections::HashMap;
use glam::Vec3;

/// 手部追踪器 - 管理OpenXR手部追踪扩展，提供双手关节追踪功能
pub struct HandTracker {
    /// 是否已初始化
    initialized: bool,
    /// 是否支持手部追踪
    supported: bool,
    /// 左手所有关节的数据
    left_hand_joints: HandJoints,
    /// 右手所有关节的数据
    right_hand_joints: HandJoints,
    /// 最后更新时间戳（毫秒）
    last_update_time: u64,
    /// 追踪系统的当前状态
    tracking_state: HandTrackingState,
}

/// 手部关节集合 - 存储一只手的所有关节追踪数据
#[derive(Debug, Clone, Default)]
pub struct HandJoints {
    /// 关节映射表，从关节类型到关节数据
    joints: HashMap<HandJointType, HandJoint>,
    /// 此手的关节数据是否有效
    is_valid: bool,
    /// 追踪置信度，范围0.0（无置信）到1.0（完全置信）
    confidence: f32,
    /// 最后更新时间戳（毫秒）
    last_update_time: u64,
}

impl HandJoints {
    /// 创建新的手部关节集合
    /// 
    /// # Returns
    /// 返回一个空的、无效的HandJoints实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新指定关节的数据
    /// 
    /// # Arguments
    /// * `joint_type` - 关节的类型
    /// * `joint` - 新的关节数据
    pub fn update_joint(&mut self, joint_type: HandJointType, joint: HandJoint) {
        self.joints.insert(joint_type, joint);
        self.last_update_time = current_timestamp_ms();
    }

    /// 获取指定关节的数据
    /// 
    /// # Arguments
    /// * `joint_type` - 关节的类型
    /// 
    /// # Returns
    /// 如果关节存在返回引用，否则返回None
    pub fn get_joint(&self, joint_type: HandJointType) -> Option<&HandJoint> {
        self.joints.get(&joint_type)
    }

    /// 获取所有关节的映射表引用
    /// 
    /// # Returns
    /// 返回所有关节映射的引用
    pub fn get_all_joints(&self) -> &HashMap<HandJointType, HandJoint> {
        &self.joints
    }

    /// 设置此手部数据的有效性
    /// 
    /// # Arguments
    /// * `valid` - true表示有效，false表示无效
    pub fn set_valid(&mut self, valid: bool) {
        self.is_valid = valid;
    }

    /// 设置追踪置信度
    /// 
    /// # Arguments
    /// * `confidence` - 置信度值，自动限制到0.0-1.0范围
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// 检查此手部数据是否有效且有关节
    /// 
    /// # Returns
    /// 只有当标记为有效且至少有一个关节时才返回true
    pub fn is_valid(&self) -> bool {
        self.is_valid && !self.joints.is_empty()
    }

    /// 获取当前的追踪置信度
    /// 
    /// # Returns
    /// 返回置信度值（0.0-1.0）
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// 获取手掌中心位置（如果可用）
    /// 
    /// # Returns
    /// 如果手掌关节有效则返回其位置，否则返回None
    pub fn get_palm_position(&self) -> Option<Vec3> {
        self.get_joint(HandJointType::Palm).map(|j| j.pose.position)
    }

    /// 简单的手势识别（占位实现）
    pub fn detect_gesture(&self) -> Option<HandGesture> {
        let curl_threshold = 0.7;
        let extended_threshold = 0.3;

        let index_curl = self.get_finger_curl(Finger::Index).unwrap_or(0.0);
        let middle_curl = self.get_finger_curl(Finger::Middle).unwrap_or(0.0);
        let ring_curl = self.get_finger_curl(Finger::Ring).unwrap_or(0.0);
        let little_curl = self.get_finger_curl(Finger::Little).unwrap_or(0.0);

        let all_curled = index_curl > curl_threshold
            && middle_curl > curl_threshold
            && ring_curl > curl_threshold
            && little_curl > curl_threshold;

        let all_extended = index_curl < extended_threshold
            && middle_curl < extended_threshold
            && ring_curl < extended_threshold
            && little_curl < extended_threshold;

        if all_curled {
            Some(HandGesture::Fist)
        } else if all_extended {
            Some(HandGesture::OpenHand)
        } else {
            None
        }
    }

    /// 简单的手势识别（占位实现）
    pub fn detect_gesture(&self) -> Option<HandGesture> {
        // 使用各手指弯曲度进行粗略判断
        let curl_threshold = 0.7;
        let extended_threshold = 0.3;

        let index_curl = self.get_finger_curl(Finger::Index);
        let middle_curl = self.get_finger_curl(Finger::Middle);
        let ring_curl = self.get_finger_curl(Finger::Ring);
        let little_curl = self.get_finger_curl(Finger::Little);

        let all_curled = index_curl > curl_threshold
            && middle_curl > curl_threshold
            && ring_curl > curl_threshold
            && little_curl > curl_threshold;

        let all_extended = index_curl < extended_threshold
            && middle_curl < extended_threshold
            && ring_curl < extended_threshold
            && little_curl < extended_threshold;

        if all_curled {
            Some(HandGesture::Fist)
        } else if all_extended {
            Some(HandGesture::OpenHand)
        } else {
            None
        }
    }

    /// 获取手腕位置（如果可用）
    /// 
    /// # Returns
    /// 如果手腕关节有效则返回其位置，否则返回None
    pub fn get_wrist_position(&self) -> Option<Vec3> {
        self.get_joint(HandJointType::Wrist)
            .map(|j| j.pose.position)
    }

    /// 获取指定手指的尖端位置
    /// 
    /// # Arguments
    /// * `finger` - 手指类型
    /// 
    /// # Returns
    /// 如果手指尖端关节有效则返回其位置，否则返回None
    pub fn get_finger_tip(&self, finger: Finger) -> Option<Vec3> {
        let joint_type = match finger {
            Finger::Thumb => HandJointType::ThumbTip,
            Finger::Index => HandJointType::IndexTip,
            Finger::Middle => HandJointType::MiddleTip,
            Finger::Ring => HandJointType::RingTip,
            Finger::Little => HandJointType::LittleTip,
        };
        self.get_joint(joint_type).map(|j| j.pose.position)
    }

    /// 计算指定手指的弯曲度
    /// 
    /// # Arguments
    /// * `finger` - 手指类型
    /// 
    /// # Returns
    /// 返回弯曲度值（0.0=完全伸直，1.0=完全弯曲），如果无法计算则返回None
    pub fn get_finger_curl(&self, finger: Finger) -> Option<f32> {
        let joints: Vec<HandJointType> = match finger {
            Finger::Thumb => vec![
                HandJointType::ThumbMetacarpal,
                HandJointType::ThumbProximal,
                HandJointType::ThumbDistal,
                HandJointType::ThumbTip,
            ],
            Finger::Index => vec![
                HandJointType::IndexMetacarpal,
                HandJointType::IndexProximal,
                HandJointType::IndexIntermediate,
                HandJointType::IndexDistal,
                HandJointType::IndexTip,
            ],
            Finger::Middle => vec![
                HandJointType::MiddleMetacarpal,
                HandJointType::MiddleProximal,
                HandJointType::MiddleIntermediate,
                HandJointType::MiddleDistal,
                HandJointType::MiddleTip,
            ],
            Finger::Ring => vec![
                HandJointType::RingMetacarpal,
                HandJointType::RingProximal,
                HandJointType::RingIntermediate,
                HandJointType::RingDistal,
                HandJointType::RingTip,
            ],
            Finger::Little => vec![
                HandJointType::LittleMetacarpal,
                HandJointType::LittleProximal,
                HandJointType::LittleIntermediate,
                HandJointType::LittleDistal,
                HandJointType::LittleTip,
            ],
        };

        // 计算关节角度来估算弯曲度
        // 简化实现：基于关节位置计算
        let mut total_curl = 0.0;
        let mut count = 0;

        for i in 0..joints.len().saturating_sub(1) {
            if let (Some(joint1), Some(joint2)) =
                (self.get_joint(joints[i]), self.get_joint(joints[i + 1]))
            {
                let dir = (joint2.pose.position - joint1.pose.position).normalize();
                // 简化实现：计算相对于初始方向的弯曲度
                // 对于第一对关节，使用该方向作为参考
                if i == 0 {
                    // 对于第一个关节对，假设完全伸直状态
                    // 弯曲度为0.0（完全伸直）
                    total_curl += 0.0;
                } else {
                    // 获取前一个关节对的方向
                    if let Some(joint0) = self.get_joint(joints[i - 1]) {
                        let prev_dir = (joint1.pose.position - joint0.pose.position).normalize();
                        // 计算两个方向的点积，点积接近1.0表示方向相同（伸直）
                        // 点积接近0.0或负表示方向不同（弯曲）
                        let dot = prev_dir.dot(dir);
                        // 将点积转换为弯曲度（0.0=伸直，1.0=弯曲）
                        let curl = 1.0 - dot;
                        total_curl += curl;
                    }
                }
                count += 1;
            }
        }

        if count > 0 {
            Some((total_curl / count as f32).clamp(0.0, 1.0))
        } else {
            None
        }
    }
}

/// 手势类型 - 识别的手势
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gesture {
    /// 抓取手势（手指弯曲成拳）
    Grasp,
    /// 指向手势（食指伸直）
    Point,
    /// 挥手手势（手部快速移动）
    Wave,
    /// 捏取手势（拇指和食指靠近）
    Pinch,
    /// 张开手势（手指全部伸直）
    Open,
    /// 握拳手势（所有手指弯曲）
    Fist,
    /// 点赞手势（拇指向上）
    ThumbsUp,
    /// OK手势（拇指和食指形成圆圈）
    Ok,
}

/// 手势事件
#[derive(Debug, Clone)]
pub struct GestureEvent {
    /// 手势类型
    pub gesture: Gesture,
    /// 手部（左手或右手）
    pub hand: Hand,
    /// 手势置信度（0.0-1.0）
    pub confidence: f32,
    /// 手势位置（手部中心位置）
    pub position: Vec3,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}

/// 简单的高层手势枚举，供 XR 模块使用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandGesture {
    /// 握拳
    Fist,
    /// 张开手
    OpenHand,
}

/// 手指类型 - 表示手部的五根手指
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Finger {
    /// 大拇指
    Thumb,
    /// 食指
    Index,
    /// 中指
    Middle,
    /// 无名指
    Ring,
    /// 小指
    Little,
}

/// 手部追踪状态 - 表示手部追踪系统的当前状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandTrackingState {
    /// 未初始化 - 手部追踪系统尚未初始化
    Uninitialized,
    /// 初始化中 - 手部追踪系统正在初始化
    Initializing,
    /// 已初始化，等待追踪 - 系统准备就绪，等待用户开始手部输入
    Ready,
    /// 追踪中 - 手部关节正在被正确追踪
    Tracking,
    /// 追踪丢失 - 无法追踪用户手部（可能因为手离开视野）
    Lost,
    /// 错误 - 追踪系统发生错误，包含错误信息
    Error(String),
}

impl HandTracker {
    /// 创建新的手部追踪器
    /// 
    /// # Returns
    /// 返回一个新的、未初始化的HandTracker实例
    pub fn new() -> Result<Self, XrError> {
        Ok(Self {
            initialized: false,
            supported: false,
            left_hand_joints: HandJoints::new(),
            right_hand_joints: HandJoints::new(),
            last_update_time: 0,
            tracking_state: HandTrackingState::Uninitialized,
        })
    }

    /// 初始化手部追踪系统（需要OpenXR会话）
    /// 
    /// 检查系统是否支持手部追踪扩展并初始化追踪系统
    /// 
    /// # Returns
    /// 成功时返回Ok(())，如果不支持手部追踪返回Err(XrError::NotSupported)
    pub fn initialize(&mut self) -> Result<(), XrError> {
        // NOTE: 实际实现中需要：
        // 1. 检查OpenXR扩展是否支持手部追踪 (XR_EXT_hand_tracking)
        // 2. 创建手部追踪空间
        // 3. 设置追踪回调

        // 占位实现
        self.tracking_state = HandTrackingState::Initializing;

        // 模拟检查扩展支持
        // 实际应该调用: instance.enumerate_extensions() 并检查 "XR_EXT_hand_tracking"
        self.supported = true; // 假设支持

        if self.supported {
            self.tracking_state = HandTrackingState::Ready;
            self.initialized = true;
            Ok(())
        } else {
            self.tracking_state =
                HandTrackingState::Error("Hand tracking not supported".to_string());
            Err(XrError::NotSupported)
        }
    }

    /// 更新手部追踪数据
    /// 
    /// 从OpenXR运行时获取最新的手部关节位置和状态
    /// 
    /// # Returns
    /// 成功时返回Ok(())，如果追踪器未初始化返回SessionNotReady
    pub fn update(&mut self) -> Result<(), XrError> {
        if !self.initialized {
            return Err(XrError::SessionNotReady);
        }

        match &self.tracking_state {
            HandTrackingState::Ready | HandTrackingState::Tracking => {
                // 继续更新
            }
            _ => {
                return Ok(()); // 未准备好，跳过更新
            }
        }

        // NOTE: 实际实现中需要：
        // 1. 调用 xr::HandTrackerEXT::locate_hand_joints()
        // 2. 获取左右手关节数据
        // 3. 更新 HandJoints

        // 占位实现：模拟更新
        self.last_update_time = current_timestamp_ms();

        // 检查是否有有效的手部数据
        let left_valid = self.left_hand_joints.is_valid();
        let right_valid = self.right_hand_joints.is_valid();

        if left_valid || right_valid {
            self.tracking_state = HandTrackingState::Tracking;
        } else {
            self.tracking_state = HandTrackingState::Lost;
        }

        Ok(())
    }

    /// 获取指定手的关节数据
    /// 
    /// # Arguments
    /// * `hand` - 要查询的手部（左或右）
    /// 
    /// # Returns
    /// 返回该手的HandJoints引用
    pub fn get_hand_joints(&self, hand: Hand) -> Option<&HandJoints> {
        match hand {
            Hand::Left => Some(&self.left_hand_joints),
            Hand::Right => Some(&self.right_hand_joints),
        }
    }

    /// 获取指定手的关节数据（可变引用）
    /// 
    /// # Arguments
    /// * `hand` - 要查询的手部（左或右）
    /// 
    /// # Returns
    /// 返回该手的HandJoints可变引用
    pub fn get_hand_joints_mut(&mut self, hand: Hand) -> Option<&mut HandJoints> {
        match hand {
            Hand::Left => Some(&mut self.left_hand_joints),
            Hand::Right => Some(&mut self.right_hand_joints),
        }
    }

    /// 检查系统是否支持手部追踪
    /// 
    /// # Returns
    /// 如果硬件和OpenXR运行时都支持手部追踪返回true
    pub fn is_supported(&self) -> bool {
        self.supported
    }

    /// 检查指定的手是否正在被追踪
    /// 
    /// # Arguments
    /// * `hand` - 要检查的手部
    /// 
    /// # Returns
    /// 如果该手正在被追踪且数据有效返回true
    pub fn is_tracking(&self, hand: Hand) -> bool {
        if !matches!(self.tracking_state, HandTrackingState::Tracking) {
            return false;
        }

        match hand {
            Hand::Left => self.left_hand_joints.is_valid(),
            Hand::Right => self.right_hand_joints.is_valid(),
        }
    }

    /// 获取追踪系统的当前状态
    /// 
    /// # Returns
    /// 返回当前的HandTrackingState
    pub fn tracking_state(&self) -> &HandTrackingState {
        &self.tracking_state
    }

    /// 获取指定手的追踪置信度
    /// 
    /// # Arguments
    /// * `hand` - 要查询的手部
    /// 
    /// # Returns
    /// 返回置信度值（0.0-1.0），0.0表示完全不可信，1.0表示完全可信
    pub fn get_confidence(&self, hand: Hand) -> f32 {
        match hand {
            Hand::Left => self.left_hand_joints.confidence(),
            Hand::Right => self.right_hand_joints.confidence(),
        }
    }

    /// 手动设置手部关节数据（用于测试或模拟）
    /// 
    /// # Arguments
    /// * `hand` - 要更新的手部
    /// * `joints` - 新的关节数据
    pub fn set_hand_joints(&mut self, hand: Hand, joints: HandJoints) {
        match hand {
            Hand::Left => self.left_hand_joints = joints,
            Hand::Right => self.right_hand_joints = joints,
        }
        self.last_update_time = current_timestamp_ms();
    }

    /// 从OpenXR手部追踪数据更新（实际实现中调用）
    #[allow(dead_code)]
    fn update_from_openxr(
        &mut self,
        hand: Hand,
        openxr_joints: &[openxr::HandJointEXT],
    ) -> Result<(), XrError> {
        // NOTE: 实际实现中需要：
        // 1. 遍历 openxr_joints
        // 2. 将 OpenXR 关节类型映射到 HandJointType
        // 3. 转换姿态和半径
        // 4. 更新 HandJoints

        let hand_joints = match hand {
            Hand::Left => &mut self.left_hand_joints,
            Hand::Right => &mut self.right_hand_joints,
        };

        // 占位实现
        hand_joints.set_valid(!openxr_joints.is_empty());

        Ok(())
    }
}

impl Default for HandTracker {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            initialized: false,
            supported: false,
            left_hand_joints: HandJoints::new(),
            right_hand_joints: HandJoints::new(),
            last_update_time: 0,
            tracking_state: HandTrackingState::Uninitialized,
        })
    }
}

/// 手部追踪配置 - 控制手部追踪系统的行为参数
#[derive(Debug, Clone)]
pub struct HandTrackingConfig {
    /// 是否启用手部追踪功能
    pub enabled: bool,
    /// 最小置信度阈值（0.0-1.0），低于此值的追踪数据认为无效
    pub min_confidence: f32,
    /// 追踪更新频率（Hz），通常为60或120Hz
    pub update_rate: f32,
    /// 是否启用手指弯曲度计算（计算量较大）
    pub enable_finger_curl: bool,
}

impl_default!(HandTrackingConfig {
    enabled: true,
    min_confidence: 0.5,
    update_rate: 60.0,
    enable_finger_curl: true,
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hand_joints() {
        let mut joints = HandJoints::new();

        let palm_joint = HandJoint {
            joint_type: HandJointType::Palm,
            pose: Pose {
                position: Vec3::new(0.0, 1.0, 0.0),
                orientation: Quat::IDENTITY,
            },
            radius: 0.05,
            is_valid: true,
        };

        joints.update_joint(HandJointType::Palm, palm_joint);
        joints.set_valid(true);
        joints.set_confidence(0.9);

        assert!(joints.is_valid());
        assert_eq!(joints.confidence(), 0.9);
        assert!(joints.get_palm_position().is_some());
    }

    #[test]
    fn test_hand_tracker() {
        let mut tracker = HandTracker::new().unwrap();

        assert!(!tracker.is_supported());
        assert!(matches!(
            tracker.tracking_state(),
            HandTrackingState::Uninitialized
        ));

        // 初始化
        let _ = tracker.initialize();

        // 创建测试关节数据
        let mut joints = HandJoints::new();
        let palm_joint = HandJoint {
            joint_type: HandJointType::Palm,
            pose: Pose::default(),
            radius: 0.05,
            is_valid: true,
        };
        joints.update_joint(HandJointType::Palm, palm_joint);
        joints.set_valid(true);

        tracker.set_hand_joints(Hand::Left, joints);

        assert!(tracker.get_hand_joints(Hand::Left).is_some());
    }
}
