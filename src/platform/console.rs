//! 控制台平台支持模块
//!
//! 提供游戏主机平台的抽象和优化

use crate::config::graphics::{GraphicsConfig, QualityLevel};
use game_engine_hardware::{AutoConfig, HardwareInfo};

/// 控制台平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsolePlatform {
    /// PlayStation 5
    PlayStation5,
    /// PlayStation 4
    PlayStation4,
    /// Xbox Series X/S
    XboxSeries,
    /// Xbox One
    XboxOne,
    /// Nintendo Switch
    NintendoSwitch,
    /// 未知控制台
    Unknown,
}

/// 控制台平台配置
#[derive(Debug, Clone)]
pub struct ConsoleConfig {
    /// 平台类型
    pub platform: ConsolePlatform,
    /// 目标帧率（控制台通常锁定60或30 FPS）
    pub target_fps: u32,
    /// 是否启用性能模式（牺牲分辨率换取帧率）
    pub performance_mode: bool,
    /// 是否启用质量模式（牺牲帧率换取分辨率）
    pub quality_mode: bool,
    /// 是否启用光线追踪
    pub ray_tracing_enabled: bool,
    /// 最大分辨率
    pub max_resolution: (u32, u32),
    /// 是否启用HDR
    pub hdr_enabled: bool,
}

impl ConsoleConfig {
    /// 从硬件信息创建控制台配置
    pub fn from_hardware(hardware: &HardwareInfo) -> Self {
        let platform = Self::detect_platform();

        // 根据平台设置默认配置
        let (target_fps, max_resolution, ray_tracing_enabled, hdr_enabled) = match platform {
            ConsolePlatform::PlayStation5 => (60, (3840, 2160), true, true),
            ConsolePlatform::PlayStation4 => (30, (1920, 1080), false, true),
            ConsolePlatform::XboxSeries => (60, (3840, 2160), true, true),
            ConsolePlatform::XboxOne => (30, (1920, 1080), false, true),
            ConsolePlatform::NintendoSwitch => (30, (1920, 1080), false, false),
            ConsolePlatform::Unknown => (60, (1920, 1080), false, false),
        };

        Self {
            platform,
            target_fps,
            performance_mode: false,
            quality_mode: true,
            ray_tracing_enabled,
            max_resolution,
            hdr_enabled,
        }
    }

    /// 检测当前控制台平台
    pub fn detect_platform() -> ConsolePlatform {
        // 注意：实际检测需要平台特定的SDK
        // 这里提供占位实现

        #[cfg(target_os = "ps5")]
        return ConsolePlatform::PlayStation5;

        #[cfg(target_os = "ps4")]
        return ConsolePlatform::PlayStation4;

        #[cfg(target_os = "xbox")]
        {
            // 需要进一步检测是Series还是One
            // 这里简化为Series
            return ConsolePlatform::XboxSeries;
        }

        #[cfg(target_os = "horizon")]
        return ConsolePlatform::NintendoSwitch;

        // 默认未知
        ConsolePlatform::Unknown
    }

    /// 应用控制台优化到图形配置
    pub fn apply_to_graphics_config(&self, config: &mut GraphicsConfig) {
        // 设置分辨率
        config.resolution.width = self.max_resolution.0.min(config.resolution.width);
        config.resolution.height = self.max_resolution.1.min(config.resolution.height);

        // 启用VSync（控制台通常强制VSync）
        config.vsync = true;

        // 根据模式调整设置
        if self.performance_mode {
            // 性能模式：降低分辨率，提高帧率
            config.resolution.width = (config.resolution.width as f32 * 0.75) as u32;
            config.resolution.height = (config.resolution.height as f32 * 0.75) as u32;
            config.shadow_quality = std::cmp::min(config.shadow_quality, QualityLevel::Medium);
        } else if self.quality_mode {
            // 质量模式：提高分辨率，可能降低帧率
            config.shadow_quality = std::cmp::max(config.shadow_quality, QualityLevel::High);
            config.texture_quality = std::cmp::max(config.texture_quality, QualityLevel::High);
        }

        // 启用光线追踪（如果支持）
        if self.ray_tracing_enabled {
            config.ray_tracing.enabled = true;
        }
    }
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        let hardware = HardwareInfo::detect();
        Self::from_hardware(&hardware)
    }
}

/// 控制台输入处理
#[derive(Default)]
pub struct ConsoleInputHandler {
    /// 连接的控制器数量
    controller_count: u32,
    /// 控制器状态
    controllers: Vec<ControllerState>,
}

#[derive(Debug, Clone)]
pub struct ControllerState {
    /// 控制器ID
    pub id: u32,
    /// 是否连接
    pub connected: bool,
    /// 左摇杆
    pub left_stick: (f32, f32),
    /// 右摇杆
    pub right_stick: (f32, f32),
    /// 左扳机
    pub left_trigger: f32,
    /// 右扳机
    pub right_trigger: f32,
    /// 按钮状态
    pub buttons: ButtonState,
}

#[derive(Debug, Clone, Default)]
pub struct ButtonState {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub left_bumper: bool,
    pub right_bumper: bool,
    pub left_stick_click: bool,
    pub right_stick_click: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub menu: bool,
    pub view: bool,
}

impl ConsoleInputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新控制器状态
    pub fn update_controller(&mut self, id: u32, state: ControllerState) {
        if let Some(controller) = self.controllers.iter_mut().find(|c| c.id == id) {
            *controller = state;
        } else {
            self.controllers.push(state);
            self.controller_count = self.controllers.len() as u32;
        }
    }

    /// 获取控制器状态
    pub fn get_controller(&self, id: u32) -> Option<&ControllerState> {
        self.controllers.iter().find(|c| c.id == id)
    }

    /// 获取所有控制器
    pub fn get_controllers(&self) -> &[ControllerState] {
        &self.controllers
    }

    /// 获取控制器数量
    pub fn controller_count(&self) -> u32 {
        self.controller_count
    }
}


/// 控制台性能监控
pub struct ConsolePerformanceMonitor {
    /// 当前帧率
    current_fps: f32,
    /// 帧时间历史
    frame_times: Vec<f32>,
    /// GPU使用率（0.0-1.0）
    gpu_usage: f32,
    /// CPU使用率（0.0-1.0）
    cpu_usage: f32,
}

use crate::impl_default;

impl_default!(ConsolePerformanceMonitor {
    current_fps: 60.0,
    frame_times: Vec::with_capacity(60),
    gpu_usage: 0.0,
    cpu_usage: 0.0,
});

impl ConsolePerformanceMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新帧时间
    pub fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push(frame_time_ms);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.current_fps = 1000.0 / avg_frame_time;
    }

    /// 更新GPU使用率
    pub fn update_gpu_usage(&mut self, usage: f32) {
        self.gpu_usage = usage.clamp(0.0, 1.0);
    }

    /// 更新CPU使用率
    pub fn update_cpu_usage(&mut self, usage: f32) {
        self.cpu_usage = usage.clamp(0.0, 1.0);
    }

    /// 获取当前帧率
    pub fn current_fps(&self) -> f32 {
        self.current_fps
    }

    /// 获取GPU使用率
    pub fn gpu_usage(&self) -> f32 {
        self.gpu_usage
    }

    /// 获取CPU使用率
    pub fn cpu_usage(&self) -> f32 {
        self.cpu_usage
    }

    /// 检查性能问题
    pub fn check_performance_issues(&self, target_fps: u32) -> bool {
        self.current_fps < target_fps as f32 * 0.9 || self.gpu_usage > 0.95 || self.cpu_usage > 0.95
    }
}


/// 检测是否为控制台平台
pub fn is_console_platform() -> bool {
    ConsoleConfig::detect_platform() != ConsolePlatform::Unknown
}

/// 获取控制台配置
pub fn get_console_config() -> Option<ConsoleConfig> {
    if is_console_platform() {
        let hardware = HardwareInfo::detect();
        Some(ConsoleConfig::from_hardware(&hardware))
    } else {
        None
    }
}
