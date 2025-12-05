/// SoC功耗管理和热节流优化
/// 
/// 针对移动平台的功耗和热管理

use super::detect::{SocInfo, SocVendor};
use crate::utils::ring_buffer::RingBuffer;
use std::time::Instant;

/// 功耗管理器
pub struct PowerManager {
    soc_info: Option<SocInfo>,
    thermal_state: ThermalState,
    power_mode: PowerMode,
    performance_history: PerformanceHistory,
    last_update: Instant,
}

/// 热状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    /// 正常
    Normal,
    /// 轻微发热
    Warm,
    /// 发热
    Hot,
    /// 严重发热（需要降频）
    Critical,
}

/// 功耗模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    /// 省电模式
    PowerSaver,
    /// 平衡模式
    Balanced,
    /// 性能模式
    Performance,
    /// 极致性能（不考虑功耗）
    Extreme,
}

/// 性能历史记录
struct PerformanceHistory {
    frame_times: RingBuffer<f32>,
    thermal_readings: RingBuffer<f32>,
}

impl PerformanceHistory {
    fn new(max_size: usize) -> Self {
        Self {
            frame_times: RingBuffer::new(max_size),
            thermal_readings: RingBuffer::new(max_size),
        }
    }
    
    fn add_frame_time(&mut self, time_ms: f32) {
        self.frame_times.push(time_ms);
    }
    
    fn add_thermal_reading(&mut self, temp: f32) {
        self.thermal_readings.push(temp);
    }
    
    fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 16.67; // 默认60fps
        }
        self.frame_times.average()
    }
    
    fn average_temperature(&self) -> f32 {
        if self.thermal_readings.is_empty() {
            return 35.0; // 默认温度
        }
        self.thermal_readings.average()
    }
}

/// 性能调整建议
#[derive(Debug, Clone)]
pub struct PerformanceAdjustment {
    pub resolution_scale: f32,
    pub target_fps: u32,
    pub shadow_quality: u32,
    pub particle_budget_scale: f32,
    pub lod_bias: f32,
    pub reason: String,
}

impl PowerManager {
    /// 创建功耗管理器
    pub fn new(soc_info: Option<SocInfo>) -> Self {
        let power_mode = if soc_info.is_some() {
            PowerMode::Balanced
        } else {
            PowerMode::Performance
        };
        
        Self {
            soc_info,
            thermal_state: ThermalState::Normal,
            power_mode,
            performance_history: PerformanceHistory::new(300), // 5秒历史（60fps）
            last_update: Instant::now(),
        }
    }
    
    /// 是否为移动平台
    pub fn is_mobile(&self) -> bool {
        self.soc_info.is_some()
    }
    
    /// 设置功耗模式
    pub fn set_power_mode(&mut self, mode: PowerMode) {
        self.power_mode = mode;
    }
    
    /// 获取当前功耗模式
    pub fn power_mode(&self) -> PowerMode {
        self.power_mode
    }
    
    /// 获取热状态
    pub fn thermal_state(&self) -> ThermalState {
        self.thermal_state
    }
    
    /// 更新性能数据
    pub fn update(&mut self, frame_time_ms: f32) {
        self.performance_history.add_frame_time(frame_time_ms);
        
        // 检测温度（简化实现）
        let temp = self.estimate_temperature();
        self.performance_history.add_thermal_reading(temp);
        
        // 更新热状态
        let new_thermal_state = self.classify_thermal_state(temp);
        
        // 如果热状态变化，记录日志
        if new_thermal_state != self.thermal_state {
            tracing::info!(target: "power_management", 
                "Thermal state changed: {:?} -> {:?} (temp: {:.1}°C)", 
                self.thermal_state, new_thermal_state, temp);
            self.thermal_state = new_thermal_state;
        }
        
        self.last_update = Instant::now();
    }
    
    /// 动态调整性能（根据热状态）
    pub fn adjust_performance_dynamically(&mut self) -> Option<PerformanceAdjustment> {
        // 如果设备过热，自动切换到省电模式
        if self.thermal_state >= ThermalState::Critical && self.power_mode != PowerMode::PowerSaver {
            tracing::warn!(target: "power_management", 
                "Device overheating, switching to power saver mode");
            self.power_mode = PowerMode::PowerSaver;
        }
        
        // 如果设备温度正常且性能模式过低，可以提升性能
        if self.thermal_state == ThermalState::Normal && 
           self.power_mode == PowerMode::PowerSaver {
            let avg_frame_time = self.performance_history.average_frame_time();
            // 如果帧时间稳定且低于目标，可以提升性能
            if avg_frame_time < 20.0 {
                tracing::info!(target: "power_management", 
                    "Device temperature normal, switching to balanced mode");
                self.power_mode = PowerMode::Balanced;
            }
        }
        
        self.get_adjustment_recommendation()
    }
    
    /// 估算温度
    fn estimate_temperature(&self) -> f32 {
        // 简化的温度估算（实际应该读取系统传感器）
        let avg_frame_time = self.performance_history.average_frame_time();
        
        // 帧时间越长，说明负载越高，温度越高
        let base_temp = 35.0;
        let load_factor = (avg_frame_time / 16.67).min(2.0);
        
        base_temp + load_factor * 20.0
    }
    
    /// 分类热状态
    fn classify_thermal_state(&self, temp: f32) -> ThermalState {
        if temp < 45.0 {
            ThermalState::Normal
        } else if temp < 55.0 {
            ThermalState::Warm
        } else if temp < 65.0 {
            ThermalState::Hot
        } else {
            ThermalState::Critical
        }
    }
    
    /// 获取性能调整建议
    pub fn get_adjustment_recommendation(&self) -> Option<PerformanceAdjustment> {
        if !self.is_mobile() {
            return None;
        }
        
        let avg_frame_time = self.performance_history.average_frame_time();
        let target_frame_time = match self.power_mode {
            PowerMode::PowerSaver => 33.33, // 30fps
            PowerMode::Balanced => 16.67,   // 60fps
            PowerMode::Performance => 16.67, // 60fps
            PowerMode::Extreme => 11.11,     // 90fps
        };
        
        // 根据热状态和性能调整
        match self.thermal_state {
            ThermalState::Critical => {
                Some(PerformanceAdjustment {
                    resolution_scale: 0.5,
                    target_fps: 30,
                    shadow_quality: 0,
                    particle_budget_scale: 0.3,
                    lod_bias: 2.0,
                    reason: "严重发热，大幅降低画质".to_string(),
                })
            }
            ThermalState::Hot => {
                Some(PerformanceAdjustment {
                    resolution_scale: 0.75,
                    target_fps: 30,
                    shadow_quality: 1,
                    particle_budget_scale: 0.5,
                    lod_bias: 1.5,
                    reason: "设备发热，降低画质".to_string(),
                })
            }
            ThermalState::Warm if avg_frame_time > target_frame_time * 1.2 => {
                Some(PerformanceAdjustment {
                    resolution_scale: 0.85,
                    target_fps: match self.power_mode {
                        PowerMode::PowerSaver => 30,
                        _ => 60,
                    },
                    shadow_quality: 1,
                    particle_budget_scale: 0.7,
                    lod_bias: 1.0,
                    reason: "轻微发热且性能不足".to_string(),
                })
            }
            ThermalState::Normal if avg_frame_time > target_frame_time * 1.5 => {
                Some(PerformanceAdjustment {
                    resolution_scale: 0.9,
                    target_fps: match self.power_mode {
                        PowerMode::PowerSaver => 30,
                        _ => 60,
                    },
                    shadow_quality: 2,
                    particle_budget_scale: 0.8,
                    lod_bias: 0.5,
                    reason: "性能不足，轻微降低画质".to_string(),
                })
            }
            _ => None,
        }
    }
    
    /// 获取电池优化建议
    pub fn get_battery_optimization(&self) -> Vec<&'static str> {
        if !self.is_mobile() {
            return vec!["非移动平台，无需电池优化"];
        }
        
        let mut tips = Vec::new();
        
        match self.power_mode {
            PowerMode::PowerSaver => {
                tips.push("✓ 已启用省电模式");
                tips.push("✓ 限制帧率到30fps");
                tips.push("✓ 降低分辨率");
                tips.push("✓ 关闭后处理特效");
            }
            PowerMode::Balanced => {
                tips.push("• 平衡模式：性能与功耗兼顾");
                tips.push("• 建议：充电时可切换到性能模式");
            }
            PowerMode::Performance | PowerMode::Extreme => {
                tips.push("⚠ 性能模式会快速消耗电量");
                tips.push("⚠ 建议连接充电器使用");
            }
        }
        
        if self.thermal_state >= ThermalState::Hot {
            tips.push("⚠ 设备发热，建议降低画质或休息片刻");
        }
        
        tips
    }
    
    /// 获取SoC特定优化建议
    pub fn get_soc_specific_tips(&self) -> Vec<String> {
        let Some(soc) = &self.soc_info else {
            return vec!["非移动平台".to_string()];
        };
        
        let mut tips = Vec::new();
        
        match soc.vendor {
            SocVendor::Apple => {
                tips.push("Apple芯片优化:".to_string());
                tips.push("  • 使用Metal API获得最佳性能".to_string());
                tips.push("  • 利用统一内存架构减少数据拷贝".to_string());
                tips.push("  • 使用Neural Engine加速AI功能".to_string());
            }
            SocVendor::Qualcomm => {
                tips.push("高通骁龙优化:".to_string());
                tips.push("  • 使用Adreno GPU的tile-based渲染".to_string());
                tips.push("  • 启用Hexagon DSP加速".to_string());
                tips.push("  • 注意热节流，避免长时间高负载".to_string());
            }
            SocVendor::MediaTek => {
                tips.push("联发科优化:".to_string());
                tips.push("  • 使用Mali GPU优化".to_string());
                tips.push("  • 启用APU加速AI功能".to_string());
                tips.push("  • 注意功耗管理".to_string());
            }
            _ => {
                tips.push("通用移动平台优化".to_string());
            }
        }
        
        tips
    }
    
    /// 获取性能统计
    pub fn get_stats(&self) -> PowerStats {
        PowerStats {
            average_frame_time_ms: self.performance_history.average_frame_time(),
            average_fps: 1000.0 / self.performance_history.average_frame_time(),
            estimated_temperature: self.performance_history.average_temperature(),
            thermal_state: self.thermal_state,
            power_mode: self.power_mode,
            is_mobile: self.is_mobile(),
        }
    }
}

/// 功耗统计
#[derive(Debug, Clone)]
pub struct PowerStats {
    pub average_frame_time_ms: f32,
    pub average_fps: f32,
    pub estimated_temperature: f32,
    pub thermal_state: ThermalState,
    pub power_mode: PowerMode,
    pub is_mobile: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect_soc;

    #[test]
    fn test_power_manager() {
        let soc = detect_soc();
        let mut manager = PowerManager::new(soc);
        
        println!("Is Mobile: {}", manager.is_mobile());
        println!("Power Mode: {:?}", manager.power_mode());
        
        // 模拟一些帧
        for i in 0..100 {
            let frame_time = 16.67 + (i as f32 * 0.1);
            manager.update(frame_time);
        }
        
        let stats = manager.get_stats();
        println!("Stats: {:#?}", stats);
        
        if let Some(adjustment) = manager.get_adjustment_recommendation() {
            println!("Adjustment: {:#?}", adjustment);
        }
    }

    #[test]
    fn test_battery_optimization() {
        let soc = detect_soc();
        let manager = PowerManager::new(soc);
        
        println!("Battery Optimization Tips:");
        for tip in manager.get_battery_optimization() {
            println!("  {}", tip);
        }
    }
}
