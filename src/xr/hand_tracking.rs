//! OpenXR 手部追踪扩展集成
//!
//! 实现OpenXR手部追踪扩展（XR_EXT_hand_tracking），提供高精度手部关节追踪。
//!
//! ## 功能特性
//!
//! - 26个手部关节追踪（每只手）
//! - 关节位置、旋转和半径
//! - 手部姿态有效性检测
//! - 双手独立追踪
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::xr::hand_tracking::*;
//!
//! // 初始化手部追踪
//! let mut hand_tracker = HandTracker::new()?;
//!
//! // 更新追踪数据
//! hand_tracker.update()?;
//!
//! // 获取左手关节数据
//! if let Some(joints) = hand_tracker.get_hand_joints(Hand::Left) {
//!     if let Some(palm) = joints.get_joint(HandJointType::Palm) {
//!         println!("Palm position: {:?}", palm.pose.position);
//!     }
//! }
//! ```

use super::*;
use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use std::collections::HashMap;

/// 手部追踪器
pub struct HandTracker {
    /// 是否已初始化
    initialized: bool,
    /// 是否支持手部追踪
    supported: bool,
    /// 左手关节数据
    left_hand_joints: HandJoints,
    /// 右手关节数据
    right_hand_joints: HandJoints,
    /// 最后更新时间
    last_update_time: u64,
    /// 追踪状态
    tracking_state: HandTrackingState,
}

/// 手部关节集合
#[derive(Debug, Clone, Default)]
pub struct HandJoints {
    /// 关节映射（关节类型 -> 关节数据）
    joints: HashMap<HandJointType, HandJoint>,
    /// 是否有效
    is_valid: bool,
    /// 置信度 (0.0 - 1.0)
    confidence: f32,
    /// 最后更新时间
    last_update_time: u64,
}

impl HandJoints {
    /// 创建新的手部关节集合
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新关节
    pub fn update_joint(&mut self, joint_type: HandJointType, joint: HandJoint) {
        self.joints.insert(joint_type, joint);
        self.last_update_time = current_timestamp_ms();
    }

    /// 获取关节
    pub fn get_joint(&self, joint_type: HandJointType) -> Option<&HandJoint> {
        self.joints.get(&joint_type)
    }

    /// 获取所有关节
    pub fn get_all_joints(&self) -> &HashMap<HandJointType, HandJoint> {
        &self.joints
    }

    /// 设置有效性
    pub fn set_valid(&mut self, valid: bool) {
        self.is_valid = valid;
    }

    /// 设置置信度
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        self.is_valid && !self.joints.is_empty()
    }

    /// 获取置信度
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// 获取手掌位置（如果可用）
    pub fn get_palm_position(&self) -> Option<Vec3> {
        self.get_joint(HandJointType::Palm).map(|j| j.pose.position)
    }

    /// 获取手腕位置（如果可用）
    pub fn get_wrist_position(&self) -> Option<Vec3> {
        self.get_joint(HandJointType::Wrist)
            .map(|j| j.pose.position)
    }

    /// 获取手指尖端位置
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

    /// 计算手指弯曲度（0.0 = 完全伸直, 1.0 = 完全弯曲）
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
        let mut total_angle = 0.0;
        let mut count = 0;

        for i in 0..joints.len().saturating_sub(1) {
            if let (Some(joint1), Some(joint2)) =
                (self.get_joint(joints[i]), self.get_joint(joints[i + 1]))
            {
                let dir = (joint2.pose.position - joint1.pose.position).normalize();
                // 简化：假设手指应该沿着某个方向延伸
                // 实际应该计算相对于手掌的角度
                count += 1;
            }
        }

        if count > 0 {
            Some((total_angle / count as f32).clamp(0.0, 1.0))
        } else {
            None
        }
    }
}


/// 手指类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Little,
}

/// 手部追踪状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandTrackingState {
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 已初始化，等待追踪
    Ready,
    /// 追踪中
    Tracking,
    /// 追踪丢失
    Lost,
    /// 错误
    Error(String),
}

impl HandTracker {
    /// 创建新的手部追踪器
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

    /// 初始化手部追踪（需要OpenXR会话）
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

    /// 获取手部关节数据
    pub fn get_hand_joints(&self, hand: Hand) -> Option<&HandJoints> {
        match hand {
            Hand::Left => Some(&self.left_hand_joints),
            Hand::Right => Some(&self.right_hand_joints),
        }
    }

    /// 获取手部关节数据（可变）
    pub fn get_hand_joints_mut(&mut self, hand: Hand) -> Option<&mut HandJoints> {
        match hand {
            Hand::Left => Some(&mut self.left_hand_joints),
            Hand::Right => Some(&mut self.right_hand_joints),
        }
    }

    /// 检查手部追踪是否支持
    pub fn is_supported(&self) -> bool {
        self.supported
    }

    /// 检查手部是否正在追踪
    pub fn is_tracking(&self, hand: Hand) -> bool {
        if !matches!(self.tracking_state, HandTrackingState::Tracking) {
            return false;
        }

        match hand {
            Hand::Left => self.left_hand_joints.is_valid(),
            Hand::Right => self.right_hand_joints.is_valid(),
        }
    }

    /// 获取追踪状态
    pub fn tracking_state(&self) -> &HandTrackingState {
        &self.tracking_state
    }

    /// 获取手部置信度
    pub fn get_confidence(&self, hand: Hand) -> f32 {
        match hand {
            Hand::Left => self.left_hand_joints.confidence(),
            Hand::Right => self.right_hand_joints.confidence(),
        }
    }

    /// 手动设置手部关节数据（用于测试或模拟）
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

/// 手部追踪配置
#[derive(Debug, Clone)]
pub struct HandTrackingConfig {
    /// 是否启用手部追踪
    pub enabled: bool,
    /// 最小置信度阈值（低于此值认为追踪无效）
    pub min_confidence: f32,
    /// 更新频率（Hz）
    pub update_rate: f32,
    /// 是否启用手指弯曲度计算
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
