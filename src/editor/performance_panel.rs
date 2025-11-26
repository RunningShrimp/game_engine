use crate::performance::Profiler;
use std::collections::VecDeque;

/// 性能监控面板
pub struct PerformancePanel {
    /// FPS历史记录
    fps_history: VecDeque<f32>,
    /// 帧时间历史记录 (毫秒)
    frame_time_history: VecDeque<f32>,
    /// 最大历史记录数量
    max_history: usize,
}

impl PerformancePanel {
    pub fn new() -> Self {
        Self {
            fps_history: VecDeque::new(),
            frame_time_history: VecDeque::new(),
            max_history: 120, // 保留最近120帧的数据
        }
    }
    
    /// 更新性能数据
    pub fn update(&mut self, delta_time: f32) {
        let fps = if delta_time > 0.0 { 1.0 / delta_time } else { 0.0 };
        let frame_time_ms = delta_time * 1000.0;
        
        self.fps_history.push_back(fps);
        self.frame_time_history.push_back(frame_time_ms);
        
        // 限制历史记录数量
        if self.fps_history.len() > self.max_history {
            self.fps_history.pop_front();
        }
        if self.frame_time_history.len() > self.max_history {
            self.frame_time_history.pop_front();
        }
    }
    
    /// 渲染性能监控面板
    pub fn render(&self, ui: &mut egui::Ui, profiler: Option<&Profiler>) {
        ui.heading("Performance Monitor");
        ui.separator();
        
        // 当前FPS和帧时间
        if let (Some(&fps), Some(&frame_time)) = (self.fps_history.back(), self.frame_time_history.back()) {
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
            
            ui.colored_label(fps_color, format!("Status: {}", 
                if fps >= 60.0 { "Excellent" } 
                else if fps >= 30.0 { "Good" } 
                else { "Poor" }
            ));
        }
        
        ui.separator();
        
        // FPS统计
        if !self.fps_history.is_empty() {
            let avg_fps: f32 = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
            let min_fps = self.fps_history.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_fps = self.fps_history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            
            ui.label(format!("Average FPS: {:.1}", avg_fps));
            ui.label(format!("Min FPS: {:.1}", min_fps));
            ui.label(format!("Max FPS: {:.1}", max_fps));
        }
        
        ui.separator();
        
        // 帧时间统计
        if !self.frame_time_history.is_empty() {
            let avg_frame_time: f32 = self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32;
            let min_frame_time = self.frame_time_history.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_frame_time = self.frame_time_history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            
            ui.label(format!("Average Frame Time: {:.2} ms", avg_frame_time));
            ui.label(format!("Min Frame Time: {:.2} ms", min_frame_time));
            ui.label(format!("Max Frame Time: {:.2} ms", max_frame_time));
        }
        
        ui.separator();
        
        // Profiler统计信息
        if let Some(profiler) = profiler {
            ui.label("Profiler Statistics:");
            
            let mut stats: Vec<_> = profiler.all_stats().into_iter().collect();
            stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
            
            for stat in stats.iter().take(10) {
                ui.horizontal(|ui| {
                    ui.label(&stat.name);
                    ui.label(format!("{:.2} ms", stat.average_time().as_secs_f64() * 1000.0));
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
}

impl Default for PerformancePanel {
    fn default() -> Self {
        Self::new()
    }
}
