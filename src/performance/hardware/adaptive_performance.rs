/// 自适应性能系统
/// 
/// 运行时动态调整画质以维持目标帧率

use super::auto_config::{AutoConfig, QualityPreset};
use super::soc_power::{PowerManager, ThermalState};
use std::time::{Instant, Duration};

/// 自适应性能管理器
pub struct AdaptivePerformance {
    config: AutoConfig,
    power_manager: PowerManager,
    
    // 性能目标
    target_fps: f32,
    target_frame_time_ms: f32,
    
    // 性能监控
    frame_times: Vec<f32>,
    last_adjustment: Instant,
    adjustment_cooldown: Duration,
    
    // 调整历史
    adjustment_count: u32,
    total_adjustments: u32,
}

impl AdaptivePerformance {
    /// 创建自适应性能管理器
    pub fn new(config: AutoConfig, power_manager: PowerManager) -> Self {
        let target_fps = config.target_fps as f32;
        let target_frame_time_ms = 1000.0 / target_fps;
        
        Self {
            config,
            power_manager,
            target_fps,
            target_frame_time_ms,
            frame_times: Vec::with_capacity(300),
            last_adjustment: Instant::now(),
            adjustment_cooldown: Duration::from_secs(3),
            adjustment_count: 0,
            total_adjustments: 0,
        }
    }
    
    /// 更新性能数据
    pub fn update(&mut self, frame_time_ms: f32) {
        // 更新功耗管理器
        self.power_manager.update(frame_time_ms);
        
        // 记录帧时间
        if self.frame_times.len() >= 300 {
            self.frame_times.remove(0);
        }
        self.frame_times.push(frame_time_ms);
        
        // 检查是否需要调整
        if self.should_adjust() {
            self.perform_adjustment();
        }
    }
    
    /// 是否应该进行调整
    fn should_adjust(&self) -> bool {
        // 冷却时间未到
        if self.last_adjustment.elapsed() < self.adjustment_cooldown {
            return false;
        }
        
        // 数据不足
        if self.frame_times.len() < 60 {
            return false;
        }
        
        true
    }
    
    /// 执行性能调整
    fn perform_adjustment(&mut self) {
        let avg_frame_time = self.average_frame_time();
        let target = self.target_frame_time_ms;
        
        // 计算性能偏差
        let deviation = (avg_frame_time - target) / target;
        
        // 检查热状态
        let thermal_state = self.power_manager.thermal_state();
        
        // 决定调整方向
        if thermal_state >= ThermalState::Hot {
            // 热节流：降低画质
            self.decrease_quality("热节流");
        } else if deviation > 0.2 {
            // 性能不足：降低画质
            self.decrease_quality("性能不足");
        } else if deviation < -0.3 && thermal_state == ThermalState::Normal {
            // 性能过剩：提升画质
            self.increase_quality("性能过剩");
        }
    }
    
    /// 降低画质
    fn decrease_quality(&mut self, reason: &str) {
        println!("[自适应] 降低画质 - 原因: {}", reason);
        
        // 优先级：分辨率 > 阴影 > 粒子 > 后处理
        
        if self.config.resolution_scale > 0.5 {
            self.config.resolution_scale = (self.config.resolution_scale - 0.1).max(0.5);
            println!("  分辨率缩放: {:.2}", self.config.resolution_scale);
        } else if self.config.shadow_quality.can_decrease() {
            self.config.shadow_quality = self.config.shadow_quality.decrease();
            println!("  阴影质量: {:?}", self.config.shadow_quality);
        } else if self.config.bloom || self.config.motion_blur || self.config.depth_of_field {
            if self.config.depth_of_field {
                self.config.depth_of_field = false;
                println!("  关闭景深");
            } else if self.config.motion_blur {
                self.config.motion_blur = false;
                println!("  关闭动态模糊");
            } else if self.config.bloom {
                self.config.bloom = false;
                println!("  关闭泛光");
            }
        } else {
            // 最后手段：降低目标帧率
            if self.target_fps > 30.0 {
                self.target_fps = 30.0;
                self.target_frame_time_ms = 1000.0 / self.target_fps;
                self.config.target_fps = 30;
                println!("  目标帧率: 30 FPS");
            }
        }
        
        self.adjustment_count += 1;
        self.total_adjustments += 1;
        self.last_adjustment = Instant::now();
    }
    
    /// 提升画质
    fn increase_quality(&mut self, reason: &str) {
        // 限制提升频率
        if self.adjustment_count > 0 {
            return;
        }
        
        println!("[自适应] 提升画质 - 原因: {}", reason);
        
        // 按降低的逆序提升
        
        if self.target_fps < 60.0 {
            self.target_fps = 60.0;
            self.target_frame_time_ms = 1000.0 / self.target_fps;
            self.config.target_fps = 60;
            println!("  目标帧率: 60 FPS");
        } else if !self.config.bloom {
            self.config.bloom = true;
            println!("  开启泛光");
        } else if !self.config.motion_blur {
            self.config.motion_blur = true;
            println!("  开启动态模糊");
        } else if !self.config.depth_of_field {
            self.config.depth_of_field = true;
            println!("  开启景深");
        } else if self.config.shadow_quality.can_increase() {
            self.config.shadow_quality = self.config.shadow_quality.increase();
            println!("  阴影质量: {:?}", self.config.shadow_quality);
        } else if self.config.resolution_scale < 1.0 {
            self.config.resolution_scale = (self.config.resolution_scale + 0.1).min(1.0);
            println!("  分辨率缩放: {:.2}", self.config.resolution_scale);
        }
        
        self.total_adjustments += 1;
        self.last_adjustment = Instant::now();
    }
    
    /// 获取平均帧时间
    fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return self.target_frame_time_ms;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }
    
    /// 获取当前配置
    pub fn config(&self) -> &AutoConfig {
        &self.config
    }
    
    /// 获取性能统计
    pub fn stats(&self) -> AdaptiveStats {
        AdaptiveStats {
            current_fps: 1000.0 / self.average_frame_time(),
            target_fps: self.target_fps,
            average_frame_time_ms: self.average_frame_time(),
            target_frame_time_ms: self.target_frame_time_ms,
            total_adjustments: self.total_adjustments,
            current_resolution_scale: self.config.resolution_scale,
            thermal_state: self.power_manager.thermal_state(),
        }
    }
    
    /// 重置调整计数
    pub fn reset_adjustment_count(&mut self) {
        self.adjustment_count = 0;
    }
}

/// 自适应性能统计
#[derive(Debug, Clone)]
pub struct AdaptiveStats {
    pub current_fps: f32,
    pub target_fps: f32,
    pub average_frame_time_ms: f32,
    pub target_frame_time_ms: f32,
    pub total_adjustments: u32,
    pub current_resolution_scale: f32,
    pub thermal_state: ThermalState,
}

// 扩展ShadowQuality以支持增减
use super::auto_config::ShadowQuality;

trait QualityAdjustable {
    fn can_increase(&self) -> bool;
    fn can_decrease(&self) -> bool;
    fn increase(&self) -> Self;
    fn decrease(&self) -> Self;
}

impl QualityAdjustable for ShadowQuality {
    fn can_increase(&self) -> bool {
        !matches!(self, ShadowQuality::Ultra)
    }
    
    fn can_decrease(&self) -> bool {
        !matches!(self, ShadowQuality::Off)
    }
    
    fn increase(&self) -> Self {
        match self {
            ShadowQuality::Off => ShadowQuality::Low,
            ShadowQuality::Low => ShadowQuality::Medium,
            ShadowQuality::Medium => ShadowQuality::High,
            ShadowQuality::High => ShadowQuality::Ultra,
            ShadowQuality::Ultra => ShadowQuality::Ultra,
        }
    }
    
    fn decrease(&self) -> Self {
        match self {
            ShadowQuality::Ultra => ShadowQuality::High,
            ShadowQuality::High => ShadowQuality::Medium,
            ShadowQuality::Medium => ShadowQuality::Low,
            ShadowQuality::Low => ShadowQuality::Off,
            ShadowQuality::Off => ShadowQuality::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::hardware::{detect_gpu, detect_npu, detect_soc};
    use crate::performance::hardware::{HardwareCapability, AutoConfig};

    #[test]
    fn test_adaptive_performance() {
        let gpu = detect_gpu();
        let npu = detect_npu();
        let soc = detect_soc();
        
        let capability = HardwareCapability::evaluate(&gpu, &npu, &soc);
        let config = AutoConfig::from_capability(&capability);
        let power_manager = PowerManager::new(soc);
        
        let mut adaptive = AdaptivePerformance::new(config, power_manager);
        
        // 模拟性能不足的情况
        println!("=== 模拟性能不足 ===");
        for _ in 0..100 {
            adaptive.update(25.0); // 40fps
        }
        
        std::thread::sleep(Duration::from_secs(4));
        adaptive.update(25.0);
        
        let stats = adaptive.stats();
        println!("Stats: {:#?}", stats);
        
        // 模拟性能恢复
        println!("\n=== 模拟性能恢复 ===");
        adaptive.reset_adjustment_count();
        for _ in 0..100 {
            adaptive.update(10.0); // 100fps
        }
        
        std::thread::sleep(Duration::from_secs(4));
        adaptive.update(10.0);
        
        let stats = adaptive.stats();
        println!("Stats: {:#?}", stats);
    }
}
