use crate::impl_default;
use crate::performance::Profiler;
use std::collections::VecDeque;

/// 性能指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    FPS,
    FrameTime,
    DrawCalls,
    MemoryUsage,
    CPUTime,
    GPUTime,
}

/// 性能指标数据
#[derive(Debug, Clone)]
pub struct MetricData {
    pub metric_type: MetricType,
    pub values: VecDeque<f32>,
    pub max_history: usize,
    pub color: egui::Color32,
}

impl MetricData {
    pub fn new(metric_type: MetricType, max_history: usize, color: egui::Color32) -> Self {
        Self {
            metric_type,
            values: VecDeque::new(),
            max_history,
            color,
        }
    }

    pub fn add_value(&mut self, value: f32) {
        self.values.push_back(value);
        if self.values.len() > self.max_history {
            self.values.pop_front();
        }
    }

    pub fn get_min(&self) -> f32 {
        self.values.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    pub fn get_max(&self) -> f32 {
        self.values
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn get_avg(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f32>() / self.values.len() as f32
    }
}

/// 性能监控面板
pub struct PerformancePanel {
    /// FPS历史记录
    fps_history: VecDeque<f32>,
    /// 帧时间历史记录 (毫秒)
    frame_time_history: VecDeque<f32>,
    /// 最大历史记录数量
    max_history: usize,
    /// 是否显示图表
    show_charts: bool,
    /// 图表高度
    chart_height: f32,
    /// 额外的性能指标
    metrics: Vec<MetricData>,
}

impl PerformancePanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新性能数据
    pub fn update(&mut self, delta_time: f32) {
        let fps = if delta_time > 0.0 {
            1.0 / delta_time
        } else {
            0.0
        };
        let frame_time_ms = delta_time * 1000.0;

        self.fps_history.push_back(fps);
        self.frame_time_history.push_back(frame_time_ms);

        // 更新指标
        if let Some(fps_metric) = self
            .metrics
            .iter_mut()
            .find(|m| m.metric_type == MetricType::FPS)
        {
            fps_metric.add_value(fps);
        }
        if let Some(frame_time_metric) = self
            .metrics
            .iter_mut()
            .find(|m| m.metric_type == MetricType::FrameTime)
        {
            frame_time_metric.add_value(frame_time_ms);
        }

        // 限制历史记录数量
        if self.fps_history.len() > self.max_history {
            self.fps_history.pop_front();
        }
        if self.frame_time_history.len() > self.max_history {
            self.frame_time_history.pop_front();
        }
    }

    /// 添加自定义指标
    pub fn add_metric(&mut self, metric_type: MetricType, color: egui::Color32) {
        self.metrics
            .push(MetricData::new(metric_type, self.max_history, color));
    }

    /// 更新指标值
    pub fn update_metric(&mut self, metric_type: MetricType, value: f32) {
        if let Some(metric) = self
            .metrics
            .iter_mut()
            .find(|m| m.metric_type == metric_type)
        {
            metric.add_value(value);
        }
    }

    /// 渲染性能监控面板
    pub fn render(&mut self, ui: &mut egui::Ui, profiler: Option<&Profiler>) {
        ui.heading("Performance Monitor");
        ui.separator();

        // 当前FPS和帧时间
        if let (Some(&fps), Some(&frame_time)) =
            (self.fps_history.back(), self.frame_time_history.back())
        {
            ui.label(format!("FPS: {:.1}", fps));
            ui.label(format!("Frame Time: {:.2} ms", frame_time));

            // FPS颜色指示
            let fps_color = if fps >= 60.0 {
                egui::Color32::GREEN
            } else if fps >= 30.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };

            ui.colored_label(
                fps_color,
                format!(
                    "Status: {}",
                    if fps >= 60.0 {
                        "Excellent"
                    } else if fps >= 30.0 {
                        "Good"
                    } else {
                        "Poor"
                    }
                ),
            );
        }

        ui.separator();

        // FPS统计
        if !self.fps_history.is_empty() {
            let avg_fps: f32 = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
            let min_fps = self
                .fps_history
                .iter()
                .cloned()
                .fold(f32::INFINITY, f32::min);
            let max_fps = self
                .fps_history
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max);

            ui.label(format!("Average FPS: {:.1}", avg_fps));
            ui.label(format!("Min FPS: {:.1}", min_fps));
            ui.label(format!("Max FPS: {:.1}", max_fps));
        }

        ui.separator();

        // 帧时间统计
        if !self.frame_time_history.is_empty() {
            let avg_frame_time: f32 =
                self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32;
            let min_frame_time = self
                .frame_time_history
                .iter()
                .cloned()
                .fold(f32::INFINITY, f32::min);
            let max_frame_time = self
                .frame_time_history
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max);

            ui.label(format!("Average Frame Time: {:.2} ms", avg_frame_time));
            ui.label(format!("Min Frame Time: {:.2} ms", min_frame_time));
            ui.label(format!("Max Frame Time: {:.2} ms", max_frame_time));
        }

        ui.separator();

        // 图表显示选项
        ui.checkbox(&mut self.show_charts, "Show Charts");
        if self.show_charts {
            ui.add(egui::Slider::new(&mut self.chart_height, 50.0..=300.0).text("Chart Height"));
        }

        ui.separator();

        // 实时图表
        if self.show_charts {
            self.render_fps_chart(ui);
            ui.separator();
            self.render_frame_time_chart(ui);
            ui.separator();
        }

        // Profiler统计信息
        if let Some(profiler) = profiler {
            ui.label("Profiler Statistics:");

            let mut stats: Vec<_> = profiler.all_stats().into_iter().collect();
            stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));

            for stat in stats.iter().take(10) {
                ui.horizontal(|ui| {
                    ui.label(&stat.name);
                    ui.label(format!(
                        "{:.2} ms",
                        stat.average_time().as_secs_f64() * 1000.0
                    ));
                    ui.label(format!("({} calls)", stat.call_count));
                });
            }
        } else {
            ui.label("No profiler data available");
        }

        ui.separator();

        // 性能建议
        if let Some(&fps) = self.fps_history.back() {
            ui.label("Performance Tips:");
            if fps < 30.0 {
                ui.colored_label(egui::Color32::RED, "• Consider reducing shadow quality");
                ui.colored_label(egui::Color32::RED, "• Reduce number of lights");
                ui.colored_label(egui::Color32::RED, "• Enable frustum culling");
            } else if fps < 60.0 {
                ui.colored_label(egui::Color32::YELLOW, "• Consider optimizing draw calls");
                ui.colored_label(egui::Color32::YELLOW, "• Enable batch rendering");
            } else {
                ui.colored_label(egui::Color32::GREEN, "• Performance is optimal!");
            }
        }
    }

    /// 渲染FPS图表
    fn render_fps_chart(&self, ui: &mut egui::Ui) {
        ui.label("FPS Chart");

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), self.chart_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

        if !self.fps_history.is_empty() {
            let min_fps = 0.0;
            let max_fps = self
                .fps_history
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max)
                .max(60.0);
            let range = max_fps - min_fps;

            if range > 0.0 {
                let points: Vec<egui::Pos2> = self
                    .fps_history
                    .iter()
                    .enumerate()
                    .map(|(i, &fps)| {
                        let x =
                            rect.left() + (i as f32 / self.fps_history.len() as f32) * rect.width();
                        let y = rect.bottom() - ((fps - min_fps) / range) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();

                // 绘制FPS线
                painter.line_segment(
                    [
                        egui::Pos2::new(
                            rect.left(),
                            rect.bottom() - (60.0 / range) * rect.height(),
                        ),
                        egui::Pos2::new(
                            rect.right(),
                            rect.bottom() - (60.0 / range) * rect.height(),
                        ),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                );

                // 绘制FPS曲线
                if points.len() > 1 {
                    for i in 0..points.len() - 1 {
                        painter.line_segment(
                            [points[i], points[i + 1]],
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
                        );
                    }
                }
            }
        }

        // 显示当前值
        if let Some(&current_fps) = self.fps_history.back() {
            ui.label(format!("Current: {:.1} FPS", current_fps));
        }
    }

    /// 渲染帧时间图表
    fn render_frame_time_chart(&self, ui: &mut egui::Ui) {
        ui.label("Frame Time Chart");

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), self.chart_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

        if !self.frame_time_history.is_empty() {
            let min_time = 0.0;
            let max_time = self
                .frame_time_history
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max)
                .max(33.33); // 30fps = 33.33ms
            let range = max_time - min_time;

            if range > 0.0 {
                let points: Vec<egui::Pos2> = self
                    .frame_time_history
                    .iter()
                    .enumerate()
                    .map(|(i, &time)| {
                        let x = rect.left()
                            + (i as f32 / self.frame_time_history.len() as f32) * rect.width();
                        let y = rect.bottom() - ((time - min_time) / range) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();

                // 绘制16.67ms线（60fps）
                painter.line_segment(
                    [
                        egui::Pos2::new(
                            rect.left(),
                            rect.bottom() - (16.67 / range) * rect.height(),
                        ),
                        egui::Pos2::new(
                            rect.right(),
                            rect.bottom() - (16.67 / range) * rect.height(),
                        ),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)),
                );

                // 绘制33.33ms线（30fps）
                painter.line_segment(
                    [
                        egui::Pos2::new(
                            rect.left(),
                            rect.bottom() - (33.33 / range) * rect.height(),
                        ),
                        egui::Pos2::new(
                            rect.right(),
                            rect.bottom() - (33.33 / range) * rect.height(),
                        ),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 0)),
                );

                // 绘制帧时间曲线
                if points.len() > 1 {
                    for i in 0..points.len() - 1 {
                        let color = if points[i].y > rect.bottom() - (16.67 / range) * rect.height()
                        {
                            egui::Color32::from_rgb(255, 0, 0) // 红色（低于60fps）
                        } else {
                            egui::Color32::from_rgb(0, 255, 0) // 绿色（60fps以上）
                        };

                        painter.line_segment(
                            [points[i], points[i + 1]],
                            egui::Stroke::new(2.0, color),
                        );
                    }
                }
            }
        }

        // 显示当前值
        if let Some(&current_time) = self.frame_time_history.back() {
            ui.label(format!("Current: {:.2} ms", current_time));
        }
    }
}

impl Default for PerformancePanel {
    fn default() -> Self {
        let mut panel = Self {
            fps_history: VecDeque::new(),
            frame_time_history: VecDeque::new(),
            max_history: 300,
            show_charts: true,
            chart_height: 150.0,
            metrics: Vec::new(),
        };

        // 初始化默认指标
        panel.metrics.push(MetricData::new(
            MetricType::FPS,
            panel.max_history,
            egui::Color32::from_rgb(0, 255, 0),
        ));
        panel.metrics.push(MetricData::new(
            MetricType::FrameTime,
            panel.max_history,
            egui::Color32::from_rgb(255, 0, 0),
        ));

        panel
    }
}
