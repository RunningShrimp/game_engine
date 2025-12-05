//! 移动平台配置模块

use crate::impl_default;
use serde::{Deserialize, Serialize};

/// 移动平台性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobilePerformanceConfig {
    /// 目标帧率
    pub target_fps: u32,
    /// 是否启用自适应帧率
    pub adaptive_fps: bool,
    /// 是否启用功耗优化
    pub power_saving: bool,
    /// 最大分辨率缩放
    pub max_resolution_scale: f32,
    /// 是否启用动态分辨率
    pub dynamic_resolution: bool,
}

impl_default!(MobilePerformanceConfig {
    target_fps: 60,
    adaptive_fps: true,
    power_saving: true,
    max_resolution_scale: 1.0,
    dynamic_resolution: true,
});

/// 移动平台输入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileInputConfig {
    /// 触摸灵敏度
    pub touch_sensitivity: f32,
    /// 是否启用陀螺仪
    pub gyroscope_enabled: bool,
    /// 是否启用多点触控
    pub multi_touch_enabled: bool,
    /// 是否启用手势识别
    pub gesture_recognition: bool,
}

impl_default!(MobileInputConfig {
    touch_sensitivity: 1.0,
    gyroscope_enabled: false,
    multi_touch_enabled: true,
    gesture_recognition: true,
});

/// 移动平台完整配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MobileConfig {
    /// 性能配置
    pub performance: MobilePerformanceConfig,
    /// 输入配置
    pub input: MobileInputConfig,
}

impl MobileConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
