//! 可视化工具模块
//!
//! 提供性能数据的可视化功能，包括图表、曲线等。

use std::collections::VecDeque;

/// 性能数据可视化器
pub struct PerformanceVisualizer {
    /// 数据历史
    data_history: VecDeque<f32>,
    /// 最大数据点数
    max_points: usize,
    /// 最小值
    min_value: f32,
    /// 最大值
    max_value: f32,
    /// 自动缩放
    auto_scale: bool,
}

impl PerformanceVisualizer {
    /// 创建新的可视化器
    pub fn new(max_points: usize) -> Self {
        Self {
            data_history: VecDeque::with_capacity(max_points),
            max_points,
            min_value: 0.0,
            max_value: 100.0,
            auto_scale: true,
        }
    }

    /// 添加数据点
    pub fn add_point(&mut self, value: f32) {
        self.data_history.push_back(value);

        if self.data_history.len() > self.max_points {
            self.data_history.pop_front();
        }

        if self.auto_scale {
            self.update_scale();
        }
    }

    /// 更新缩放范围
    fn update_scale(&mut self) {
        if let Some(min) = self.data_history.iter().copied().reduce(f32::min) {
            self.min_value = min;
        }
        if let Some(max) = self.data_history.iter().copied().reduce(f32::max) {
            self.max_value = max;
        }
    }

    /// 渲染可视化
    pub fn render(&self, ui: &mut egui::Ui, title: &str, color: egui::Color32) {
        ui.label(title);

        let available_size = ui.available_size();
        let rect = egui::Rect::from_min_size(ui.cursor().min, available_size);

        // 绘制背景
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_gray(20),
        );

        // 绘制数据曲线
        if self.data_history.len() > 1 {
            let points: Vec<egui::Pos2> = self
                .data_history
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let x = rect.min.x
                        + (i as f32 / (self.data_history.len() - 1) as f32) * rect.width();
                    let normalized = (value - self.min_value) / (self.max_value - self.min_value);
                    let y = rect.max.y - normalized * rect.height();
                    egui::Pos2::new(x, y)
                })
                .collect();

            ui.painter().line_segment(
                points.as_slice(),
                (2.0, color),
            );
        }

        // 绘制边框
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::GRAY),
        );

        // 占据空间
        ui.allocate_space(available_size);
    }

    /// 获取当前值
    pub fn current_value(&self) -> Option<f32> {
        self.data_history.back().copied()
    }

    /// 获取平均值
    pub fn average(&self) -> Option<f32> {
        if self.data_history.is_empty() {
            return None;
        }
        let sum: f32 = self.data_history.iter().sum();
        Some(sum / self.data_history.len() as f32)
    }

    /// 获取最小值
    pub fn min(&self) -> Option<f32> {
        self.data_history.iter().copied().reduce(f32::min)
    }

    /// 获取最大值
    pub fn max(&self) -> Option<f32> {
        self.data_history.iter().copied().reduce(f32::max)
    }

    /// 清空数据
    pub fn clear(&mut self) {
        self.data_history.clear();
    }
}

/// 内存使用可视化器
pub struct MemoryVisualizer {
    /// 总内存使用
    total_memory: VecDeque<f64>,
    /// 堆内存使用
    heap_memory: VecDeque<f64>,
    /// GPU内存使用
    gpu_memory: VecDeque<f64>,
    /// 最大数据点数
    max_points: usize,
}

impl MemoryVisualizer {
    /// 创建新的内存可视化器
    pub fn new(max_points: usize) -> Self {
        Self {
            total_memory: VecDeque::with_capacity(max_points),
            heap_memory: VecDeque::with_capacity(max_points),
            gpu_memory: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    /// 添加内存数据点
    pub fn add_memory_sample(&mut self, total: f64, heap: f64, gpu: f64) {
        self.total_memory.push_back(total);
        self.heap_memory.push_back(heap);
        self.gpu_memory.push_back(gpu);

        if self.total_memory.len() > self.max_points {
            self.total_memory.pop_front();
            self.heap_memory.pop_front();
            self.gpu_memory.pop_front();
        }
    }

    /// 渲染内存可视化
    pub fn render(&self, ui: &mut egui::Ui) {
        ui.heading("Memory Usage");

        let available_size = ui.available_size();
        let size = egui::vec2(available_size.x, 150.0);
        let rect = egui::Rect::from_min_size(ui.cursor().min, size);

        // 绘制背景
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_gray(20),
        );

        // 绘制总内存（蓝色）
        if self.total_memory.len() > 1 {
            self.draw_line(
                ui,
                rect,
                &self.total_memory,
                egui::Color32::BLUE,
                0.0,
                1000.0,
            );
        }

        // 绘制堆内存（绿色）
        if self.heap_memory.len() > 1 {
            self.draw_line(
                ui,
                rect,
                &self.heap_memory,
                egui::Color32::GREEN,
                0.0,
                1000.0,
            );
        }

        // 绘制GPU内存（红色）
        if self.gpu_memory.len() > 1 {
            self.draw_line(
                ui,
                rect,
                &self.gpu_memory,
                egui::Color32::RED,
                0.0,
                1000.0,
            );
        }

        // 绘制边框
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::GRAY),
        );

        // 占据空间
        ui.allocate_space(size);

        // 图例
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::BLUE, "● Total");
            ui.colored_label(egui::Color32::GREEN, "● Heap");
            ui.colored_label(egui::Color32::RED, "● GPU");
        });
    }

    /// 绘制线条
    fn draw_line(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        data: &VecDeque<f64>,
        color: egui::Color32,
        min_val: f64,
        max_val: f64,
    ) {
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
                let normalized = ((value - min_val) / (max_val - min_val)) as f32;
                let y = rect.max.y - normalized * rect.height();
                egui::Pos2::new(x, y)
            })
            .collect();

        for window in points.windows(2) {
            ui.painter()
                .line_segment([window[0], window[1]], (2.0, color));
        }
    }
}

/// FPS可视化器
pub struct FPSVisualizer {
    fps_history: VecDeque<f32>,
    max_points: usize,
}

impl FPSVisualizer {
    /// 创建新的FPS可视化器
    pub fn new(max_points: usize) -> Self {
        Self {
            fps_history: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    /// 添加FPS样本
    pub fn add_fps_sample(&mut self, fps: f32) {
        self.fps_history.push_back(fps);
        if self.fps_history.len() > self.max_points {
            self.fps_history.pop_front();
        }
    }

    /// 渲染FPS可视化
    pub fn render(&self, ui: &mut egui::Ui) {
        ui.heading("FPS");

        if let Some(current_fps) = self.fps_history.back() {
            let color = if *current_fps >= 60.0 {
                egui::Color32::GREEN
            } else if *current_fps >= 30.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };

            ui.colored_label(color, format!("{:.1} FPS", current_fps));
        }

        let available_size = ui.available_size();
        let size = egui::vec2(available_size.x, 100.0);
        let rect = egui::Rect::from_min_size(ui.cursor().min, size);

        // 绘制背景
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_gray(20),
        );

        // 绘制目标FPS线（60 FPS）
        let target_y = rect.min.y + (60.0 / 120.0) * rect.height();
        ui.painter().line_segment(
            [
                egui::Pos2::new(rect.min.x, target_y),
                egui::Pos2::new(rect.max.x, target_y),
            ],
            (1.0, egui::Color32::YELLOW),
        );

        // 绘制FPS曲线
        if self.fps_history.len() > 1 {
            let points: Vec<egui::Pos2> = self
                .fps_history
                .iter()
                .enumerate()
                .map(|(i, fps)| {
                    let x = rect.min.x
                        + (i as f32 / (self.fps_history.len() - 1) as f32) * rect.width();
                    let y = rect.max.y - (fps / 120.0).min(1.0) * rect.height();
                    egui::Pos2::new(x, y)
                })
                .collect();

            for window in points.windows(2) {
                ui.painter()
                    .line_segment([window[0], window[1]], (2.0, egui::Color32::GREEN));
            }
        }

        // 绘制边框
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(1.0, egui::Color32::GRAY),
        );

        // 占据空间
        ui.allocate_space(size);
    }

    /// 获取当前FPS
    pub fn current_fps(&self) -> Option<f32> {
        self.fps_history.back().copied()
    }

    /// 获取平均FPS
    pub fn average_fps(&self) -> Option<f32> {
        if self.fps_history.is_empty() {
            return None;
        }
        let sum: f32 = self.fps_history.iter().sum();
        Some(sum / self.fps_history.len() as f32)
    }
}
