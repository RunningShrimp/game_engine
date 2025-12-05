//! 功耗感知优化模块
//!
//! 根据设备功耗状态调整性能策略，提升移动设备电池寿命。
//!
//! ## 功能特性
//!
//! - **功耗状态检测**: 检测设备功耗状态（充电/电池/低电量）
//! - **性能策略调整**: 根据功耗状态动态调整性能策略
//! - **帧率限制**: 低电量时降低帧率上限
//! - **特效降级**: 低电量时降低特效质量
//! - **后台优化**: 应用进入后台时降低性能消耗
//!
//! ## 预期效果
//!
//! - **电池寿命提升**: 10-20%
//! - **性能平衡**: 在性能和电池寿命之间取得平衡

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// 功耗状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// 充电中（可以全性能运行）
    Charging,
    /// 电池供电（正常性能）
    Battery,
    /// 低电量（需要降低性能）
    LowBattery,
    /// 极低电量（需要大幅降低性能）
    CriticalBattery,
}

impl PowerState {
    /// 获取性能缩放因子
    ///
    /// 返回0.0-1.0之间的值，表示性能缩放比例。
    pub fn performance_scale(&self) -> f32 {
        match self {
            PowerState::Charging => 1.0,
            PowerState::Battery => 0.9,
            PowerState::LowBattery => 0.7,
            PowerState::CriticalBattery => 0.5,
        }
    }

    /// 获取目标帧率
    ///
    /// 根据功耗状态返回目标帧率（FPS）。
    pub fn target_fps(&self) -> u32 {
        match self {
            PowerState::Charging => 60,
            PowerState::Battery => 60,
            PowerState::LowBattery => 30,
            PowerState::CriticalBattery => 20,
        }
    }
}

/// 功耗感知管理器
///
/// 管理设备功耗状态，并根据状态调整性能策略。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::platform::power_aware::{PowerAwareManager, PowerState};
///
/// // 创建功耗感知管理器
/// let mut manager = PowerAwareManager::new();
///
/// // 更新功耗状态
/// manager.update_power_state(PowerState::Battery);
///
/// // 获取性能缩放因子
/// let scale = manager.performance_scale();
///
/// // 获取目标帧率
/// let target_fps = manager.target_fps();
/// ```
pub struct PowerAwareManager {
    /// 当前功耗状态
    power_state: AtomicU8,
    /// 是否启用功耗感知优化
    enabled: AtomicBool,
    /// 是否在后台运行
    is_background: AtomicBool,
}

impl PowerAwareManager {
    /// 创建新的功耗感知管理器
    ///
    /// # 返回
    ///
    /// 返回一个初始化的功耗感知管理器。
    pub fn new() -> Self {
        Self {
            power_state: AtomicU8::new(PowerState::Battery as u8),
            enabled: AtomicBool::new(true),
            is_background: AtomicBool::new(false),
        }
    }

    /// 更新功耗状态
    ///
    /// # 参数
    ///
    /// * `state` - 新的功耗状态
    pub fn update_power_state(&self, state: PowerState) {
        self.power_state.store(state as u8, Ordering::Relaxed);
    }

    /// 获取当前功耗状态
    ///
    /// # 返回
    ///
    /// 返回当前功耗状态。
    pub fn power_state(&self) -> PowerState {
        match self.power_state.load(Ordering::Relaxed) {
            0 => PowerState::Charging,
            1 => PowerState::Battery,
            2 => PowerState::LowBattery,
            3 => PowerState::CriticalBattery,
            _ => PowerState::Battery,
        }
    }

    /// 获取性能缩放因子
    ///
    /// # 返回
    ///
    /// 返回0.0-1.0之间的值，表示性能缩放比例。
    pub fn performance_scale(&self) -> f32 {
        if !self.enabled.load(Ordering::Relaxed) {
            return 1.0;
        }

        let mut scale = self.power_state().performance_scale();

        // 后台运行时进一步降低性能
        if self.is_background.load(Ordering::Relaxed) {
            scale *= 0.5;
        }

        scale
    }

    /// 获取目标帧率
    ///
    /// # 返回
    ///
    /// 返回目标帧率（FPS）。
    pub fn target_fps(&self) -> u32 {
        if !self.enabled.load(Ordering::Relaxed) {
            return 60;
        }

        let mut fps = self.power_state().target_fps();

        // 后台运行时进一步降低帧率
        if self.is_background.load(Ordering::Relaxed) {
            fps = (fps / 2).max(10);
        }

        fps
    }

    /// 设置是否启用功耗感知优化
    ///
    /// # 参数
    ///
    /// * `enabled` - 是否启用
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// 检查是否启用
    ///
    /// # 返回
    ///
    /// 如果启用返回`true`，否则返回`false`。
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// 设置是否在后台运行
    ///
    /// # 参数
    ///
    /// * `is_background` - 是否在后台
    pub fn set_background(&self, is_background: bool) {
        self.is_background.store(is_background, Ordering::Relaxed);
    }

    /// 检查是否在后台运行
    ///
    /// # 返回
    ///
    /// 如果在后台返回`true`，否则返回`false`。
    pub fn is_background(&self) -> bool {
        self.is_background.load(Ordering::Relaxed)
    }

    /// 获取建议的LOD偏移
    ///
    /// 根据功耗状态返回建议的LOD距离偏移（正值表示增加距离，降低LOD）。
    ///
    /// # 返回
    ///
    /// 返回LOD距离偏移。
    pub fn lod_bias(&self) -> f32 {
        if !self.enabled.load(Ordering::Relaxed) {
            return 0.0;
        }

        match self.power_state() {
            PowerState::Charging => 0.0,
            PowerState::Battery => 5.0,
            PowerState::LowBattery => 15.0,
            PowerState::CriticalBattery => 30.0,
        }
    }

    /// 获取建议的特效质量等级
    ///
    /// 根据功耗状态返回建议的特效质量等级（0-3，0为最低，3为最高）。
    ///
    /// # 返回
    ///
    /// 返回特效质量等级。
    pub fn effect_quality(&self) -> u8 {
        if !self.enabled.load(Ordering::Relaxed) {
            return 3;
        }

        match self.power_state() {
            PowerState::Charging => 3,
            PowerState::Battery => 2,
            PowerState::LowBattery => 1,
            PowerState::CriticalBattery => 0,
        }
    }
}

impl Default for PowerAwareManager {
    fn default() -> Self {
        Self::new()
    }
}

// 注意：保留手动实现，因为new()有初始化逻辑

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_state_performance_scale() {
        assert_eq!(PowerState::Charging.performance_scale(), 1.0);
        assert_eq!(PowerState::Battery.performance_scale(), 0.9);
        assert_eq!(PowerState::LowBattery.performance_scale(), 0.7);
        assert_eq!(PowerState::CriticalBattery.performance_scale(), 0.5);
    }

    #[test]
    fn test_power_state_target_fps() {
        assert_eq!(PowerState::Charging.target_fps(), 60);
        assert_eq!(PowerState::Battery.target_fps(), 60);
        assert_eq!(PowerState::LowBattery.target_fps(), 30);
        assert_eq!(PowerState::CriticalBattery.target_fps(), 20);
    }

    #[test]
    fn test_power_aware_manager() {
        let manager = PowerAwareManager::new();
        assert!(manager.is_enabled());
        assert_eq!(manager.power_state(), PowerState::Battery);
        assert_eq!(manager.performance_scale(), 0.9);
        assert_eq!(manager.target_fps(), 60);
    }

    #[test]
    fn test_power_aware_manager_low_battery() {
        let manager = PowerAwareManager::new();
        manager.update_power_state(PowerState::LowBattery);
        assert_eq!(manager.power_state(), PowerState::LowBattery);
        assert_eq!(manager.performance_scale(), 0.7);
        assert_eq!(manager.target_fps(), 30);
        assert_eq!(manager.lod_bias(), 15.0);
        assert_eq!(manager.effect_quality(), 1);
    }

    #[test]
    fn test_power_aware_manager_background() {
        let manager = PowerAwareManager::new();
        manager.set_background(true);
        assert!(manager.is_background());
        assert_eq!(manager.performance_scale(), 0.45); // 0.9 * 0.5
        assert_eq!(manager.target_fps(), 30); // 60 / 2
    }
}

