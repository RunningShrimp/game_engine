//! 性能面板
//!
//! 显示FPS、Draw Calls、内存使用等性能指标，并提供可视化图表。

use super::Panel;
use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// 性能面板
///
/// 显示实时性能指标和历史图表。
pub struct PerformancePanel {
    /// 是否显示面板
    visible: bool,
    /// 帧时间历史
    frame_time_history: VecDeque<f32>,
    /// FPS历史
    fps_history: VecDeque<f32>,
    /// Draw Calls历史
    draw_calls_history: VecDeque<usize>,
    /// 内存使用历史
    memory_history: VecDeque<f64>,
    /// 最大历史记录数
    max_history_size: usize,
    /// 当前性能指标
    current_metrics: PerformanceMetrics,
    /// 是否显示图表
    show_charts: bool,
    /// 图表缩放
    chart_zoom: f32,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 当前FPS
    pub fps: f32,
    /// 当前帧时间（毫秒）
    pub frame_time_ms: f32,
    /// 当前Draw Calls
    pub draw_calls: usize,
    /// 当前三角形数量
    pub triangle_count: usize,
    /// 当前内存使用（MB）
    pub memory_usage_mb: f64,
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f32,
    /// GPU使用率（百分比）
    pub gpu_usage_percent: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            draw_calls: 0,
            triangle_count: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            gpu_usage_percent: 0.0,
        }
    }
}

impl PerformancePanel {
    /// 创建新的性能面板
    pub fn new() -> Self {
        Self::with_history_size(300)
    }

    /// 使用自定义历史记录大小创建性能面板
    pub fn with_history_size(size: usize) -> Self {
        Self {
            visible: true,
            frame_time_history: VecDeque::with_capacity(size),
            fps_history: VecDeque::with_capacity(size),
            draw_calls_history: VecDeque::with_capacity(size),
            memory_history: VecDeque::with_capacity(size),
            max_history_size: size,
            current_metrics: PerformanceMetrics::default(),
            show_charts: true,
            chart_zoom: 1.0,
        }
    }

    /// 更新性能指标
    pub fn update_metrics(&mut self, frame_time: f32, frame_count: u64) {
        // 计算FPS
        let fps = if frame_time > 0.0 {
            1.0 / frame_time
        } else {
            0.0
        };

        let frame_time_ms = frame_time * 1000.0;

        // 更新当前指标
        self.current_metrics.fps = fps;
        self.current_metrics.frame_time_ms = frame_time_ms;

        // 添加到历史记录
        self.frame_time_history.push_back(frame_time_ms);
        self.fps_history.push_back(fps);

        // 限制历史记录大小
        if self.frame_time_history.len() > self.max_history_size {
            self.frame_time_history.pop_front();
        }
        if self.fps_history.len() > self.max_history_size {
            self.fps_history.pop_front();
        }

        // 更新其他指标（这些通常从其他系统获取）
        self.update_system_metrics();
    }

    /// 更新系统指标（内存、CPU等）
    fn update_system_metrics(&mut self) {
        // 获取内存使用
        if let Ok(memory_usage) = Self::get_memory_usage() {
            self.current_metrics.memory_usage_mb = memory_usage;
            self.memory_history.push_back(memory_usage);
            if self.memory_history.len() > self.max_history_size {
                self.memory_history.pop_front();
            }
        }

        // CPU和GPU使用率通常需要平台特定的实现
        // 这里设置为0作为占位符
        self.current_metrics.cpu_usage_percent = 0.0;
        self.current_metrics.gpu_usage_percent = 0.0;
    }

    /// 获取当前内存使用（MB）
    fn get_memory_usage() -> Result<f64, Box<dyn std::error::Error>> {
        // 使用系统相关的API获取内存使用
        // 这里是一个简化的实现
        #[cfg(target_os = "macos")]
        {
            // macOS实现
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()?;
            let rss = String::from_utf8_lossy(&output.stdout).trim().parse::<u64>()?;
            Ok((rss as f64) / 1024.0) // 转换为MB
        }

        #[cfg(not(target_os = "macos"))]
        {
            // 其他平台的占位符实现
            Ok(0.0)
        }
    }

    /// 更新Draw Calls
    pub fn update_draw_calls(&mut self, draw_calls: usize, triangle_count: usize) {
        self.current_metrics.draw_calls = draw_calls;
        self.current_metrics.triangle_count = triangle_count;

        self.draw_calls_history.push_back(draw_calls);
        if self.draw_calls_history.len() > self.max_history_size {
            self.draw_calls_history.pop_front();
        }
    }

    /// 获取当前FPS
    pub fn current_fps(&self) -> Option<f32> {
        if self.current_metrics.fps > 0.0 {
            Some(self.current_metrics.fps)
        } else {
            None
        }
    }

    /// 显示面板
    pub fn show(&mut self, ctx: &egui::Context, _ui_state: &super::super::ui::DebugUIState) {
        if !self.visible {
            return;
        }

        egui::Window::new("Performance").default_size([500.0, 600.0]).show(ctx, |ui| {
            // 当前指标
            self.show_current_metrics(ui);

            ui.separator();

            // 图表控制
            ui.horizontal(|ui| {
                ui.label("Charts:");
                if ui.checkbox(&mut self.show_charts, "Show").clicked() {}
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.chart_zoom, 0.1..=5.0).logarithmic(true));
            });

            ui.separator();

            // 显示图表
            if self.show_charts {
                self.show_charts_ui(ctx);
            }

            // 历史统计
            ui.separator();
            self.show_statistics(ui);
        });
    }

    /// 显示当前指标
    fn show_current_metrics(&mut self, ui: &mut egui::Ui) {
        ui.heading("Current Metrics");

        egui::Grid::new("current_metrics")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                // FPS
                ui.label("FPS:");
                let fps_color = if self.current_metrics.fps >= 60.0 {
                    egui::Color32::GREEN
                } else if self.current_metrics.fps >= 30.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(fps_color, format!("{:.1}", self.current_metrics.fps));
                ui.end_row();

                // 帧时间
                ui.label("Frame Time:");
                ui.label(format!("{:.2} ms", self.current_metrics.frame_time_ms));
                ui.end_row();

                // Draw Calls
                ui.label("Draw Calls:");
                ui.label(format!("{}", self.current_metrics.draw_calls));
                ui.end_row();

                // 三角形数量
                ui.label("Triangles:");
                ui.label(format!("{}", self.current_metrics.triangle_count));
                ui.end_row();

                // 内存使用
                ui.label("Memory:");
                ui.label(format!("{:.2} MB", self.current_metrics.memory_usage_mb));
                ui.end_row();

                // CPU使用率
                ui.label("CPU:");
                ui.label(format!("{:.1}%", self.current_metrics.cpu_usage_percent));
                ui.end_row();

                // GPU使用率
                ui.label("GPU:");
                ui.label(format!("{:.1}%", self.current_metrics.gpu_usage_percent));
                ui.end_row();
            });
    }

    /// 显示图表
    fn show_charts_ui(&mut self, ctx: &egui::Context) {
        // FPS图表
        if !self.fps_history.is_empty() {
            egui::Window::new("FPS History").fixed_size([400.0, 150.0]).show(ctx, |ui| {
                self.plot_fps(ui);
            });
        }

        // 帧时间图表
        if !self.frame_time_history.is_empty() {
            egui::Window::new("Frame Time History")
                .fixed_size([400.0, 150.0])
                .show(ctx, |ui| {
                    self.plot_frame_time(ui);
                });
        }

        // 内存图表
        if !self.memory_history.is_empty() {
            egui::Window::new("Memory History").fixed_size([400.0, 150.0]).show(ctx, |ui| {
                self.plot_memory(ui);
            });
        }
    }

    /// 绘制FPS图表（简化版，不使用egui::plot）
    fn plot_fps(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("FPS History:");

            // 显示最近60帧的FPS
            let recent_fps: Vec<_> = self.fps_history.iter().rev().take(60).collect();
            for (i, fps) in recent_fps.iter().enumerate() {
                let color = if **fps >= 60.0 {
                    egui::Color32::GREEN
                } else if **fps >= 30.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };

                // 简单的条形图
                let height = (**fps / 120.0).min(1.0);
                ui.add_sized(
                    [4.0, 40.0],
                    egui::Label::new(egui::RichText::new("█").color(color)),
                );
            }
        });

        // 计算统计信息
        let current_fps = self.current_metrics.fps;
        let fps_values: Vec<f32> = self.fps_history.iter().copied().collect();
        let min_fps = fps_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_fps = fps_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let avg_fps = if fps_values.is_empty() {
            0.0
        } else {
            fps_values.iter().sum::<f32>() / fps_values.len() as f32
        };

        ui.label(format!(
            "Current: {:.1} FPS | Min: {:.1} | Max: {:.1} | Avg: {:.1}",
            current_fps, min_fps, max_fps, avg_fps
        ));
    }

    /// 绘制帧时间图表（简化版）
    fn plot_frame_time(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Frame Time (ms):");

            // 显示最近60帧的帧时间
            let recent_times: Vec<_> = self.frame_time_history.iter().rev().take(60).collect();
            for frame_time in recent_times {
                let color = if *frame_time <= 16.67 {
                    egui::Color32::GREEN
                } else if *frame_time <= 33.33 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };

                let height = (*frame_time / 50.0).min(1.0);
                ui.add_sized(
                    [4.0, 40.0],
                    egui::Label::new(egui::RichText::new("█").color(color)),
                );
            }
        });

        if let Some(latest) = self.frame_time_history.back() {
            ui.label(format!("Current: {:.2} ms", latest));
        }
    }

    /// 绘制内存图表（简化版）
    fn plot_memory(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Memory (MB):");

            // 显示最近60个内存数据点
            let recent_mem: Vec<_> = self.memory_history.iter().rev().take(60).collect();
            for mem in recent_mem {
                ui.add_sized(
                    [4.0, 40.0],
                    egui::Label::new(egui::RichText::new("█").color(egui::Color32::BLUE)),
                );
            }
        });

        if let Some(latest) = self.memory_history.back() {
            ui.label(format!("Total: {:.1} MB", latest));
        }
    }

    /// 显示统计信息
    fn show_statistics(&mut self, ui: &mut egui::Ui) {
        ui.heading("Statistics");

        if let Some(avg_fps) = self.calculate_average_fps() {
            ui.label(format!("Average FPS: {:.1}", avg_fps));
        }

        if let Some(avg_frame_time) = self.calculate_average_frame_time() {
            ui.label(format!("Average Frame Time: {:.2} ms", avg_frame_time));
        }

        if let Some(min_fps) = self.calculate_min_fps() {
            ui.label(format!("Min FPS: {:.1}", min_fps));
        }

        if let Some(max_fps) = self.calculate_max_fps() {
            ui.label(format!("Max FPS: {:.1}", max_fps));
        }
    }

    /// 计算平均FPS
    fn calculate_average_fps(&self) -> Option<f32> {
        if self.fps_history.is_empty() {
            return None;
        }
        let sum: f32 = self.fps_history.iter().sum();
        Some(sum / self.fps_history.len() as f32)
    }

    /// 计算平均帧时间
    fn calculate_average_frame_time(&self) -> Option<f32> {
        if self.frame_time_history.is_empty() {
            return None;
        }
        let sum: f32 = self.frame_time_history.iter().sum();
        Some(sum / self.frame_time_history.len() as f32)
    }

    /// 计算最小FPS
    fn calculate_min_fps(&self) -> Option<f32> {
        self.fps_history.iter().copied().reduce(f32::min)
    }

    /// 计算最大FPS
    fn calculate_max_fps(&self) -> Option<f32> {
        self.fps_history.iter().copied().reduce(f32::max)
    }
}

impl Default for PerformancePanel {
    fn default() -> Self {
        Self::new()
    }
}
