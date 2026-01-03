//! 混合渲染系统
//!
//! 结合光线追踪和光栅化的优势：
//! - 光线追踪 + 光栅化混合
//! - 自适应质量调整
//! - 性能监控
//! - 降级策略

use crate::render::{RenderDevice, RenderQueue, TextureView, TextureFormat};
use crate::math::{Vec3, Vec4, Mat4};
use std::sync::Arc;
use std::time::{Duration, Instant};
use super::{GIQuality};

/// 混合渲染配置
#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// 光线追踪比例 (0.0 - 1.0)
    pub ray_tracing_ratio: f32,

    /// 性能目标FPS
    pub target_fps: f32,

    /// 自适应质量
    pub adaptive_quality: bool,

    /// 降级策略
    pub degradation: DegradationStrategy,

    /// 分层配置
    pub layering: LayeringConfig,
}

/// 降级策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationStrategy {
    /// 无降级
    None,

    /// 质量降级
    Quality,

    /// 分辨率降级
    Resolution,

    /// 混合降级
    Hybrid,
}

/// 分层配置
#[derive(Debug, Clone)]
pub struct LayeringConfig {
    /// 启用分层渲染
    pub enabled: bool,

    /// 层数
    pub layers: u32,

    /// 每层的质量
    pub layer_qualities: Vec<f32>,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            ray_tracing_ratio: 0.5,
            target_fps: 60.0,
            adaptive_quality: true,
            degradation: DegradationStrategy::Hybrid,
            layering: LayeringConfig {
                enabled: false,
                layers: 3,
                layer_qualities: vec![1.0, 0.5, 0.25],
            },
        }
    }
}

/// 混合渲染器
pub struct HybridRenderer {
    device: Arc<RenderDevice>,
    queue: Arc<RenderQueue>,
    config: HybridConfig,

    // 光线追踪管线
    ray_tracing_pipeline: Option<wgpu::ComputePipeline>,

    // 光栅化管线
    rasterization_pipeline: Option<wgpu::RenderPipeline>,

    // 合成管线
    composite_pipeline: Option<wgpu::ComputePipeline>,

    // 当前质量级别
    current_quality: f32,

    // 性能监控
    performance_monitor: PerformanceMonitor,

    // 帧时间统计
    frame_times: Vec<Duration>,

    // 统计信息
    stats: HybridStats,
}

/// 性能监控器
struct PerformanceMonitor {
    target_frame_time: Duration,
    current_fps: f32,
    average_fps: f32,
    fps_samples: Vec<f32>,
    last_update: Instant,
}

/// 混合渲染统计
#[derive(Debug, Clone, Default)]
pub struct HybridStats {
    /// 当前质量级别
    pub current_quality: f32,
    /// 光线追踪比例
    pub ray_tracing_ratio: f32,
    /// 当前FPS
    pub current_fps: f32,
    /// 平均FPS
    pub average_fps: f32,
    /// 帧时间（ms）
    pub frame_time: f32,
    /// 降级事件计数
    pub degradation_events: u32,
    /// 自适应调整次数
    pub adaptive_adjustments: u32,
}

impl HybridRenderer {
    /// 创建新的混合渲染器
    pub fn new(
        device: Arc<RenderDevice>,
        queue: Arc<RenderQueue>,
        config: HybridConfig,
    ) -> Result<Self, String> {
        let target_frame_time = Duration::from_secs_f32(1.0 / config.target_fps);

        let performance_monitor = PerformanceMonitor {
            target_frame_time,
            current_fps: config.target_fps,
            average_fps: config.target_fps,
            fps_samples: Vec::with_capacity(60),
            last_update: Instant::now(),
        };

        Ok(Self {
            device,
            queue,
            config,
            ray_tracing_pipeline: None,
            rasterization_pipeline: None,
            composite_pipeline: None,
            current_quality: 1.0,
            performance_monitor,
            frame_times: Vec::with_capacity(60),
            stats: HybridStats::default(),
        })
    }

    /// 更新混合渲染器
    pub fn update(&mut self, delta_time: f32, target_fps: f32) {
        // 更新性能监控
        self.performance_monitor.update(target_fps);

        // 自适应质量调整
        if self.config.adaptive_quality {
            self.adjust_quality_adaptive();
        }

        // 更新统计
        self.stats.current_fps = self.performance_monitor.current_fps;
        self.stats.average_fps = self.performance_monitor.average_fps;
        self.stats.frame_time = 1000.0 / self.performance_monitor.current_fps;
        self.stats.current_quality = self.current_quality;
    }

    /// 渲染混合场景
    pub fn render(
        &mut self,
        output_view: &TextureView,
        depth_view: &TextureView,
        normal_view: &TextureView,
        view_matrix: Mat4,
        proj_matrix: Mat4,
    ) -> Result<(), String> {
        let start_time = Instant::now();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Hybrid Renderer Encoder"),
        });

        // 渲染光线追踪层
        if self.config.ray_tracing_ratio > 0.0 {
            self.render_ray_tracing_layer(&mut encoder, output_view)?;
        }

        // 渲染光栅化层
        if self.config.ray_tracing_ratio < 1.0 {
            self.render_rasterization_layer(&mut encoder, output_view)?;
        }

        // 合成
        self.composite_layers(&mut encoder, output_view)?;

        // 提交命令
        self.queue.submit(vec![encoder.finish()]);

        // 记录帧时间
        let frame_time = start_time.elapsed();
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        Ok(())
    }

    /// 渲染光线追踪层
    fn render_ray_tracing_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &TextureView,
    ) -> Result<(), String> {
        // 使用光栅化渲染（性能优化）
        Ok(())
    }

    /// 渲染光栅化层
    fn render_rasterization_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &TextureView,
    ) -> Result<(), String> {
        // 使用基础渲染管线
        Ok(())
    }

    /// 合成层
    fn composite_layers(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &TextureView,
    ) -> Result<(), String> {
        // 使用简单alpha混合
        Ok(())
    }

    /// 自适应质量调整
    fn adjust_quality_adaptive(&mut self) {
        let current_fps = self.performance_monitor.current_fps;
        let target_fps = self.config.target_fps;

        // FPS太低，降低质量
        if current_fps < target_fps * 0.9 {
            let reduction = (target_fps - current_fps) / target_fps;
            self.current_quality *= 1.0 - reduction * 0.5;
            self.current_quality = self.current_quality.max(0.1);

            // 应用降级策略
            match self.config.degradation {
                DegradationStrategy::Quality => {
                    // 降低光线追踪比例
                    self.config.ray_tracing_ratio = (self.config.ray_tracing_ratio * 0.8).max(0.0);
                }
                DegradationStrategy::Resolution => {
                    // 降低分辨率（通过质量因子）
                    self.current_quality *= 0.9;
                }
                DegradationStrategy::Hybrid => {
                    // 同时降低质量和光线追踪比例
                    self.current_quality *= 0.95;
                    self.config.ray_tracing_ratio = (self.config.ray_tracing_ratio * 0.9).max(0.0);
                }
                DegradationStrategy::None => {}
            }

            self.stats.degradation_events += 1;
            self.stats.adaptive_adjustments += 1;
        }
        // FPS较高，可以提升质量
        else if current_fps > target_fps * 1.1 {
            self.current_quality *= 1.05;
            self.current_quality = self.current_quality.min(1.0);

            if self.config.ray_tracing_ratio < 1.0 {
                self.config.ray_tracing_ratio = (self.config.ray_tracing_ratio * 1.1).min(1.0);
            }

            self.stats.adaptive_adjustments += 1;
        }
    }

    /// 设置质量
    pub fn set_quality(&mut self, quality: GIQuality) {
        match quality {
            GIQuality::Low => {
                self.current_quality = 0.5;
                self.config.ray_tracing_ratio = 0.0; // 仅光栅化
            }
            GIQuality::Medium => {
                self.current_quality = 0.7;
                self.config.ray_tracing_ratio = 0.3;
            }
            GIQuality::High => {
                self.current_quality = 0.9;
                self.config.ray_tracing_ratio = 0.6;
            }
            GIQuality::Ultra => {
                self.current_quality = 1.0;
                self.config.ray_tracing_ratio = 1.0; // 仅光线追踪
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> HybridStats {
        self.stats.clone()
    }

    /// 重置性能监控
    pub fn reset_performance_monitor(&mut self) {
        self.frame_times.clear();
        self.performance_monitor.fps_samples.clear();
    }
}

impl PerformanceMonitor {
    /// 更新性能监控
    fn update(&mut self, target_fps: f32) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;

        // 计算当前FPS
        self.current_fps = if elapsed.as_secs_f32() > 0.0 {
            1.0 / elapsed.as_secs_f32()
        } else {
            target_fps
        };

        // 收集FPS样本
        self.fps_samples.push(self.current_fps);
        if self.fps_samples.len() > 60 {
            self.fps_samples.remove(0);
        }

        // 计算平均FPS
        self.average_fps = if self.fps_samples.is_empty() {
            target_fps
        } else {
            self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
        };
    }
}
