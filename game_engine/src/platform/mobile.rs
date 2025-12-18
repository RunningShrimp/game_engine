//! 移动平台优化模块
//!
//! 提供移动平台特定的优化和配置

use crate::config::graphics::GraphicsConfig;
use crate::impl_default;
use crate::render::texture_compression::Platform;
use game_engine_hardware::{AutoConfig, HardwareInfo};

/// 移动平台配置
#[derive(Debug, Clone)]
pub struct MobileConfig {
    /// 目标帧率（移动平台通常30或60 FPS）
    pub target_fps: u32,
    /// 是否启用自适应帧率
    pub adaptive_fps: bool,
    /// 是否启用功耗优化
    pub power_saving: bool,
    /// 是否启用热节流检测
    pub thermal_throttling_detection: bool,
    /// 最大分辨率缩放（0.5-1.0）
    pub max_resolution_scale: f32,
    /// 是否启用动态分辨率
    pub dynamic_resolution: bool,
    /// 触摸输入灵敏度
    pub touch_sensitivity: f32,
    /// 是否启用陀螺仪
    pub gyroscope_enabled: bool,
}

impl_default!(MobileConfig {
    target_fps: 60,
    adaptive_fps: true,
    power_saving: true,
    thermal_throttling_detection: true,
    max_resolution_scale: 1.0,
    dynamic_resolution: true,
    touch_sensitivity: 1.0,
    gyroscope_enabled: false,
});

impl MobileConfig {
    /// 从硬件信息创建移动平台配置
    pub fn from_hardware(hardware: &HardwareInfo) -> Self {
        let auto_config = &hardware.recommended_config;

        Self {
            target_fps: if hardware.capability.is_mobile {
                // 移动平台根据性能等级调整帧率
                match hardware.capability.tier {
                    game_engine_hardware::PerformanceTier::Flagship => 60,
                    game_engine_hardware::PerformanceTier::High => 60,
                    game_engine_hardware::PerformanceTier::MediumHigh => 60,
                    game_engine_hardware::PerformanceTier::Medium => 30,
                    game_engine_hardware::PerformanceTier::MediumLow => 30,
                    game_engine_hardware::PerformanceTier::Low => 30,
                }
            } else {
                60
            },
            adaptive_fps: true,
            power_saving: hardware.capability.is_mobile,
            thermal_throttling_detection: hardware.capability.thermal_limited,
            max_resolution_scale: auto_config.resolution_scale,
            dynamic_resolution: hardware.capability.is_mobile,
            touch_sensitivity: 1.0,
            gyroscope_enabled: false,
        }
    }

    /// 应用移动平台优化到图形配置
    pub fn apply_to_graphics_config(&self, config: &mut GraphicsConfig) {
        // 移动平台优化
        if Platform::current() == Platform::Android || Platform::current() == Platform::IOS {
            // 降低MSAA（移动平台性能敏感）
            config.msaa_samples = config.msaa_samples.min(4);

            // 启用纹理压缩
            config.texture_compression.enabled = true;

            // 降低阴影质量
            config.shadow_quality = config.shadow_quality.min(2);

            // 启用动态分辨率
            if self.dynamic_resolution {
                config.resolution_scale = self.max_resolution_scale;
            }
        }
    }
}

/// 移动平台性能监控器
pub struct MobilePerformanceMonitor {
    /// 当前帧率
    current_fps: f32,
    /// 帧时间历史
    frame_times: Vec<f32>,
    /// 热节流状态
    thermal_throttled: bool,
    /// 电池电量（0.0-1.0）
    battery_level: f32,
    /// 是否充电中
    charging: bool,
}

impl_default!(MobilePerformanceMonitor {
    current_fps: 60.0,
    frame_times: Vec::with_capacity(60),
    thermal_throttled: false,
    battery_level: 1.0,
    charging: true,
});

impl MobilePerformanceMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新帧时间
    pub fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push(frame_time_ms);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // 计算平均帧率
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.current_fps = 1000.0 / avg_frame_time;
    }

    /// 检测性能问题
    pub fn check_performance_issues(&self, target_fps: u32) -> PerformanceIssue {
        let target_frame_time = 1000.0 / target_fps as f32;

        // 检查帧率是否低于目标
        if self.current_fps < target_fps as f32 * 0.9 {
            return PerformanceIssue::LowFps {
                current: self.current_fps,
                target: target_fps as f32,
            };
        }

        // 检查是否热节流
        if self.thermal_throttled {
            return PerformanceIssue::ThermalThrottling;
        }

        // 检查电池电量
        if !self.charging && self.battery_level < 0.2 {
            return PerformanceIssue::LowBattery {
                level: self.battery_level,
            };
        }

        PerformanceIssue::None
    }

    /// 获取当前帧率
    pub fn current_fps(&self) -> f32 {
        self.current_fps
    }

    /// 设置热节流状态
    pub fn set_thermal_throttled(&mut self, throttled: bool) {
        self.thermal_throttled = throttled;
    }

    /// 更新电池状态
    pub fn update_battery_status(&mut self, level: f32, charging: bool) {
        self.battery_level = level;
        self.charging = charging;
    }
}


/// 性能问题
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceIssue {
    None,
    LowFps { current: f32, target: f32 },
    ThermalThrottling,
    LowBattery { level: f32 },
}

/// 移动平台自适应性能管理器
pub struct MobileAdaptivePerformance {
    config: MobileConfig,
    monitor: MobilePerformanceMonitor,
    /// 当前分辨率缩放
    current_resolution_scale: f32,
    /// 当前目标帧率
    current_target_fps: u32,
    /// 性能调整历史
    adjustment_history: Vec<PerformanceAdjustment>,
}

#[derive(Debug, Clone)]
struct PerformanceAdjustment {
    timestamp: f64,
    resolution_scale: f32,
    target_fps: u32,
    reason: String,
}

impl MobileAdaptivePerformance {
    pub fn new(config: MobileConfig) -> Self {
        Self {
            current_resolution_scale: config.max_resolution_scale,
            current_target_fps: config.target_fps,
            config,
            monitor: MobilePerformanceMonitor::new(),
            adjustment_history: Vec::new(),
        }
    }

    /// 更新性能监控并调整设置
    pub fn update(&mut self, frame_time_ms: f32, timestamp: f64) {
        self.monitor.update_frame_time(frame_time_ms);

        if !self.config.adaptive_fps {
            return;
        }

        // 检查性能问题
        let issue = self
            .monitor
            .check_performance_issues(self.current_target_fps);

        match issue {
            PerformanceIssue::LowFps { current, target } => {
                // 帧率低于目标，降低质量
                if self.current_resolution_scale > 0.5 {
                    self.current_resolution_scale = (self.current_resolution_scale - 0.1).max(0.5);
                    self.record_adjustment(timestamp, "Low FPS");
                } else if self.current_target_fps > 30 {
                    // 如果分辨率已经最低，降低帧率目标
                    self.current_target_fps = 30;
                    self.record_adjustment(timestamp, "Low FPS - Reduce target");
                }
            }
            PerformanceIssue::ThermalThrottling => {
                // 热节流，降低质量
                if self.current_resolution_scale > 0.5 {
                    self.current_resolution_scale = (self.current_resolution_scale - 0.1).max(0.5);
                    self.record_adjustment(timestamp, "Thermal throttling");
                }
                if self.current_target_fps > 30 {
                    self.current_target_fps = 30;
                    self.record_adjustment(timestamp, "Thermal throttling - Reduce FPS");
                }
            }
            PerformanceIssue::LowBattery { .. } => {
                // 低电量，启用省电模式
                if self.current_target_fps > 30 {
                    self.current_target_fps = 30;
                    self.record_adjustment(timestamp, "Low battery");
                }
            }
            PerformanceIssue::None => {
                // 性能良好，可以尝试提高质量
                if self.current_resolution_scale < self.config.max_resolution_scale {
                    let new_scale = (self.current_resolution_scale + 0.05)
                        .min(self.config.max_resolution_scale);
                    if new_scale != self.current_resolution_scale {
                        self.current_resolution_scale = new_scale;
                        self.record_adjustment(timestamp, "Performance good");
                    }
                }
            }
        }
    }

    fn record_adjustment(&mut self, timestamp: f64, reason: &str) {
        self.adjustment_history.push(PerformanceAdjustment {
            timestamp,
            resolution_scale: self.current_resolution_scale,
            target_fps: self.current_target_fps,
            reason: reason.to_string(),
        });

        // 保持历史记录在合理范围内
        if self.adjustment_history.len() > 100 {
            self.adjustment_history.remove(0);
        }
    }

    /// 获取当前分辨率缩放
    pub fn resolution_scale(&self) -> f32 {
        self.current_resolution_scale
    }

    /// 获取当前目标帧率
    pub fn target_fps(&self) -> u32 {
        self.current_target_fps
    }

    /// 获取性能监控器
    pub fn monitor(&self) -> &MobilePerformanceMonitor {
        &self.monitor
    }
}

/// 移动平台输入处理
#[derive(Default)]
pub struct MobileInputHandler {
    /// 触摸点
    touches: Vec<TouchPoint>,
    /// 陀螺仪数据
    gyroscope: Option<GyroscopeData>,
}

#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,
    pub position: (f32, f32),
    pub previous_position: (f32, f32),
    pub pressure: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct GyroscopeData {
    pub rotation_rate: (f32, f32, f32), // x, y, z (rad/s)
    pub acceleration: (f32, f32, f32),  // x, y, z (m/s²)
    pub timestamp: f64,
}

impl MobileInputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理触摸开始
    pub fn handle_touch_start(&mut self, id: u64, x: f32, y: f32, pressure: f32, timestamp: f64) {
        if let Some(touch) = self.touches.iter_mut().find(|t| t.id == id) {
            touch.position = (x, y);
            touch.pressure = pressure;
            touch.timestamp = timestamp;
        } else {
            self.touches.push(TouchPoint {
                id,
                position: (x, y),
                previous_position: (x, y),
                pressure,
                timestamp,
            });
        }
    }

    /// 处理触摸移动
    pub fn handle_touch_move(&mut self, id: u64, x: f32, y: f32, pressure: f32, timestamp: f64) {
        if let Some(touch) = self.touches.iter_mut().find(|t| t.id == id) {
            touch.previous_position = touch.position;
            touch.position = (x, y);
            touch.pressure = pressure;
            touch.timestamp = timestamp;
        }
    }

    /// 处理触摸结束
    pub fn handle_touch_end(&mut self, id: u64, timestamp: f64) {
        self.touches.retain(|t| t.id != id);
    }

    /// 获取触摸点
    pub fn get_touch(&self, id: u64) -> Option<&TouchPoint> {
        self.touches.iter().find(|t| t.id == id)
    }

    /// 获取所有触摸点
    pub fn get_touches(&self) -> &[TouchPoint] {
        &self.touches
    }

    /// 更新陀螺仪数据
    pub fn update_gyroscope(
        &mut self,
        rotation_rate: (f32, f32, f32),
        acceleration: (f32, f32, f32),
        timestamp: f64,
    ) {
        self.gyroscope = Some(GyroscopeData {
            rotation_rate,
            acceleration,
            timestamp,
        });
    }

    /// 获取陀螺仪数据
    pub fn get_gyroscope(&self) -> Option<&GyroscopeData> {
        self.gyroscope.as_ref()
    }
}

/// 检测是否为移动平台
pub fn is_mobile_platform() -> bool {
    Platform::current() == Platform::Android || Platform::current() == Platform::IOS
}

/// 获取移动平台特定配置
pub fn get_mobile_config() -> Option<MobileConfig> {
    if is_mobile_platform() {
        let hardware = HardwareInfo::detect();
        Some(MobileConfig::from_hardware(&hardware))
    } else {
        None
    }
}
