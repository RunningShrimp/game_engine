use crate::performance::{MemoryProfiler, GpuProfiler, AdvancedProfiler, PerformanceMetrics};
use std::collections::VecDeque;
use std::time::Duration;

/// 性能监控器
pub struct PerformanceMonitor {
    /// 内存分析器
    memory_profiler: MemoryProfiler,
    /// GPU分析器
    gpu_profiler: GpuProfiler,
    /// 高级分析器
    advanced_profiler: AdvancedProfiler,
    
    /// FPS历史记录
    fps_history: VecDeque<f32>,
    /// 帧时间历史记录
    frame_time_history: VecDeque<f32>,
    /// 内存使用历史记录
    memory_usage_history: VecDeque<f32>,
    
    /// 历史记录最大长度
    max_history_length: usize,
    
    /// 是否启用
    enabled: bool,
    /// 是否显示详细信息
    show_details: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            memory_profiler: MemoryProfiler::new(),
            gpu_profiler: GpuProfiler::new(),
            advanced_profiler: AdvancedProfiler::new(100),
            fps_history: VecDeque::with_capacity(100),
            frame_time_history: VecDeque::with_capacity(100),
            memory_usage_history: VecDeque::with_capacity(100),
            max_history_length: 100,
            enabled: true,
            show_details: false,
        }
    }
    
    /// 更新性能数据
    pub fn update(&mut self, delta_time: Duration) {
        if !self.enabled {
            return;
        }
        
        // 创建性能指标
        let metrics = PerformanceMetrics {
            frame_time: delta_time.as_secs_f32() * 1000.0,
            fps: 1.0 / delta_time.as_secs_f32(),
            render_time: 0.0,
            update_time: 0.0,
            physics_time: 0.0,
            memory_usage: self.memory_profiler.get_current_memory_usage() as f32 / 1024.0 / 1024.0,
            draw_calls: 0,
            triangle_count: 0,
        };
        
        // 更新高级分析器
        self.advanced_profiler.end_frame(metrics.clone());
        
        // 更新FPS历史
        self.fps_history.push_back(metrics.fps);
        if self.fps_history.len() > self.max_history_length {
            self.fps_history.pop_front();
        }
        
        // 更新帧时间历史
        self.frame_time_history.push_back(metrics.frame_time);
        if self.frame_time_history.len() > self.max_history_length {
            self.frame_time_history.pop_front();
        }
        
        // 更新内存使用历史
        let memory_usage = self.memory_profiler.get_current_memory_usage() as f32 / 1024.0 / 1024.0;
        self.memory_usage_history.push_back(memory_usage);
        if self.memory_usage_history.len() > self.max_history_length {
            self.memory_usage_history.pop_front();
        }
    }
    
    /// 渲染性能监控UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance Monitor");
        ui.separator();
        
        // 启用/禁用开关
        ui.checkbox(&mut self.enabled, "Enable Performance Monitoring");
        ui.checkbox(&mut self.show_details, "Show Details");
        ui.separator();
        
        if !self.enabled {
            return;
        }
        
        // 获取当前性能指标
        let metrics = self.advanced_profiler.get_latest_metrics().cloned().unwrap_or_default();
        
        // 显示实时性能指标
        ui.collapsing("Real-time Metrics", |ui| {
            ui.horizontal(|ui| {
                ui.label("FPS:");
                ui.label(format!("{:.1}", metrics.fps));
                
                ui.separator();
                
                ui.label("Frame Time:");
                ui.label(format!("{:.2}ms", metrics.frame_time));
            });
            
            ui.horizontal(|ui| {
                ui.label("Update Time:");
                ui.label(format!("{:.2}ms", metrics.update_time));
                
                ui.separator();
                
                ui.label("Render Time:");
                ui.label(format!("{:.2}ms", metrics.render_time));
            });
            
            ui.horizontal(|ui| {
                ui.label("Draw Calls:");
                ui.label(format!("{}", metrics.draw_calls));
                
                ui.separator();
                
                ui.label("Triangles:");
                ui.label(format!("{}", metrics.triangle_count));
            });
        });
        
        ui.separator();
        
        // FPS图表
        ui.collapsing("FPS History", |ui| {
            self.render_line_chart(
                ui,
                "FPS",
                &self.fps_history,
                0.0,
                120.0,
                egui::Color32::GREEN,
            );
        });
        
        ui.separator();
        
        // 帧时间图表
        ui.collapsing("Frame Time History", |ui| {
            self.render_line_chart(
                ui,
                "Frame Time (ms)",
                &self.frame_time_history,
                0.0,
                33.0,
                egui::Color32::YELLOW,
            );
        });
        
        ui.separator();
        
        // 内存使用图表
        ui.collapsing("Memory Usage History", |ui| {
            self.render_line_chart(
                ui,
                "Memory (MB)",
                &self.memory_usage_history,
                0.0,
                1024.0,
                egui::Color32::RED,
            );
        });
        
        ui.separator();
        
        // 内存分析器详细信息
        if self.show_details {
            ui.collapsing("Memory Profiler Details", |ui| {
                let current_memory = self.memory_profiler.get_current_memory_usage();
                let peak_memory = self.memory_profiler.get_peak_memory_usage();
                
                ui.label(format!("Current Memory: {:.2} MB", current_memory as f32 / 1024.0 / 1024.0));
                ui.label(format!("Peak Memory: {:.2} MB", peak_memory as f32 / 1024.0 / 1024.0));
                
                ui.separator();
                
                ui.label("Allocation Statistics:");
                let stats = self.memory_profiler.get_allocation_stats();
                let mut sorted_stats: Vec<_> = stats.iter().collect();
                sorted_stats.sort_by(|a, b| b.1.1.cmp(&a.1.1));
                
                for (tag, (count, size)) in sorted_stats.iter().take(10) {
                    ui.label(format!("  {}: {} allocations, {:.2} MB", 
                        tag, count, *size as f32 / 1024.0 / 1024.0));
                }
            });
            
            ui.separator();
            
            // GPU分析器详细信息
            ui.collapsing("GPU Profiler Details", |ui| {
                let queries = self.gpu_profiler.get_all_queries();
                
                if queries.is_empty() {
                    ui.label("No GPU queries recorded");
                } else {
                    let mut sorted_queries: Vec<_> = queries.iter().collect();
                    sorted_queries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
                    
                    let total_time: f32 = queries.values().sum();
                    ui.label(format!("Total GPU Time: {:.2}ms", total_time));
                    ui.separator();
                    
                    for (name, time) in sorted_queries {
                        let percentage = (time / total_time) * 100.0;
                        ui.label(format!("{}: {:.2}ms ({:.1}%)", name, time, percentage));
                    }
                }
            });
        }
        
        ui.separator();
        
        // 控制按钮
        ui.horizontal(|ui| {
            if ui.button("Clear History").clicked() {
                self.fps_history.clear();
                self.frame_time_history.clear();
                self.memory_usage_history.clear();
            }
            
            if ui.button("Reset Profilers").clicked() {
                self.memory_profiler.clear();
                self.gpu_profiler.clear();
                self.advanced_profiler = AdvancedProfiler::new(100);
            }
            
            if ui.button("Generate Report").clicked() {
                self.generate_report();
            }
        });
    }
    
    /// 渲染折线图
    fn render_line_chart(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        data: &VecDeque<f32>,
        min_value: f32,
        max_value: f32,
        color: egui::Color32,
    ) {
        if data.is_empty() {
            ui.label("No data");
            return;
        }
        
        let chart_height = 100.0;
        let chart_width = ui.available_width();
        
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(chart_width, chart_height),
            egui::Sense::hover(),
        );
        
        let rect = response.rect;
        
        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));
        
        // 绘制网格线
        for i in 0..5 {
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            painter.line_segment(
                [egui::Pos2::new(rect.left(), y), egui::Pos2::new(rect.right(), y)],
                egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
            );
        }
        
        // 绘制数据线
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let x = rect.left() + (i as f32 / (data.len() - 1).max(1) as f32) * rect.width();
                let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
                let y = rect.bottom() - normalized * rect.height();
                egui::Pos2::new(x, y)
            })
            .collect();
        
        if points.len() > 1 {
            painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
        }
        
        // 显示当前值
        if let Some(&current_value) = data.back() {
            ui.label(format!("{}: {:.2}", label, current_value));
        }
        
        // 显示平均值
        let avg: f32 = data.iter().sum::<f32>() / data.len() as f32;
        ui.label(format!("Average: {:.2}", avg));
        
        // 显示最小/最大值
        let min = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        ui.label(format!("Min: {:.2}, Max: {:.2}", min, max));
    }
    
    /// 生成性能报告
    fn generate_report(&self) {
        let mut report = String::new();
        
        report.push_str("=== Performance Monitor Report ===\n\n");
        
        // 添加内存分析器报告
        report.push_str(&self.memory_profiler.generate_report());
        report.push_str("\n");
        
        // 添加GPU分析器报告
        report.push_str(&self.gpu_profiler.generate_report());
        report.push_str("\n");
        
        // 添加高级分析器报告
        let metrics = self.advanced_profiler.get_latest_metrics().cloned().unwrap_or_default();
        report.push_str("=== Advanced Profiler Report ===\n\n");
        report.push_str(&format!("FPS: {:.1}\n", metrics.fps));
        report.push_str(&format!("Frame Time: {:.2}ms\n", metrics.frame_time));
        report.push_str(&format!("Update Time: {:.2}ms\n", metrics.update_time));
        report.push_str(&format!("Render Time: {:.2}ms\n", metrics.render_time));
        report.push_str(&format!("Draw Calls: {}\n", metrics.draw_calls));
        report.push_str(&format!("Triangles: {}\n", metrics.triangle_count));
        
        // 保存报告到文件
        let report_path = "performance_report.txt";
        if let Err(e) = std::fs::write(report_path, &report) {
            eprintln!("Failed to write performance report: {}", e);
        } else {
            println!("Performance report saved to {}", report_path);
        }
    }
    
    /// 获取内存分析器
    pub fn memory_profiler(&mut self) -> &mut MemoryProfiler {
        &mut self.memory_profiler
    }
    
    /// 获取GPU分析器
    pub fn gpu_profiler(&mut self) -> &mut GpuProfiler {
        &mut self.gpu_profiler
    }
    
    /// 获取高级分析器
    pub fn advanced_profiler(&mut self) -> &mut AdvancedProfiler {
        &mut self.advanced_profiler
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        // 更新性能数据
        monitor.update(Duration::from_millis(16));
        
        // 验证历史记录已更新
        assert!(!monitor.fps_history.is_empty());
        assert!(!monitor.frame_time_history.is_empty());
    }
}
