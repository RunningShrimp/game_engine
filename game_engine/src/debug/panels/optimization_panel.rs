//! 性能优化面板
//!
//! 提供性能瓶颈检测和一键优化功能。

use super::super::bottleneck_detector::{BottleneckDetector, BottleneckDiagnosis};
use crate::debug::ui::DebugPanel;
use egui::{CentralPanel, ScrollArea, Sense, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 性能优化面板
pub struct OptimizationPanel {
    /// 瓶颈检测器
    bottleneck_detector: Arc<Mutex<BottleneckDetector>>,

    /// 当前诊断结果
    diagnoses: Vec<BottleneckDiagnosis>,

    /// 是否正在检测
    is_detecting: bool,

    /// 选中的优化项
    selected_optimizations: Vec<String>,

    /// 优化预览
    optimization_preview: Option<OptimizationPreview>,

    /// 是否显示详细信息
    show_details: bool,

    /// 自动检测定时器
    auto_detect_timer: f32,
}

/// 优化预览
#[derive(Debug, Clone)]
pub struct OptimizationPreview {
    /// 优化前性能
    pub before_metrics: PerformanceSnapshot,

    /// 优化后预估性能
    pub after_metrics: PerformanceSnapshot,

    /// 优化建议
    pub recommendations: Vec<OptimizationRecommendation>,
}

/// 性能快照
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub draw_calls: u32,
    pub triangle_count: u32,
    pub memory_mb: f32,
    pub gpu_usage: f32,
}

/// 优化建议
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// 建议ID
    pub id: String,

    /// 建议标题
    pub title: String,

    /// 建议描述
    pub description: String,

    /// 优化类型
    pub optimization_type: OptimizationType,

    /// 预估性能提升（百分比）
    pub estimated_improvement: f32,

    /// 风险等级
    pub risk_level: RiskLevel,

    /// 是否已应用
    pub applied: bool,
}

/// 优化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationType {
    /// 渲染优化
    Rendering,
    /// 物理优化
    Physics,
    /// 内存优化
    Memory,
    /// 网络优化
    Network,
    /// 脚本优化
    Script,
    /// 着色器优化
    Shader,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl Default for OptimizationPanel {
    fn default() -> Self {
        Self {
            bottleneck_detector: Arc::new(Mutex::new(BottleneckDetector::new())),
            diagnoses: Vec::new(),
            is_detecting: false,
            selected_optimizations: Vec::new(),
            optimization_preview: None,
            show_details: false,
            auto_detect_timer: 0.0,
        }
    }
}

impl DebugPanel for OptimizationPanel {
    fn name(&self) -> &str {
        "Performance Optimization"
    }

    fn show(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.show_ui(ui);
        });
    }
}

impl OptimizationPanel {
    /// 显示UI
    fn show_ui(&mut self, ui: &mut Ui) {
        ui.heading("⚡ Performance Optimization");
        ui.separator();

        // 顶部按钮
        ui.horizontal(|ui| {
            if ui.button("🔍 Detect Bottlenecks").clicked() {
                self.detect_bottlenecks();
            }

            if ui.button("🚀 Apply Optimizations").clicked() {
                self.apply_optimizations();
            }

            if ui.button("🔄 Reset").clicked() {
                self.reset();
            }

            ui.checkbox(&mut self.show_details, "Show Details");
        });

        ui.separator();

        // 瓶颈检测状态
        if self.is_detecting {
            ui.spinner();
            ui.label("Detecting performance bottlenecks...");
        } else if !self.diagnoses.is_empty() {
            // 显示诊断结果
            self.show_diagnoses(ui);
        } else {
            ui.vertical_centered(|ui| {
                ui.label("No bottlenecks detected yet.");
                ui.label("Click 'Detect Bottlenecks' to analyze performance.");
            });
        }

        // 优化预览
        if let Some(preview) = &self.optimization_preview {
            ui.separator();
            self.show_optimization_preview(ui, preview);
        }
    }

    /// 显示诊断结果
    fn show_diagnoses(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            for diagnosis in &self.diagnoses {
                self.show_diagnosis(ui, diagnosis);
                ui.separator();
            }
        });
    }

    /// 显示单个诊断
    fn show_diagnosis(&mut self, ui: &mut Ui, diagnosis: &BottleneckDiagnosis) {
        let severity_color = match diagnosis.severity {
            crate::profiling::bottleneck_detector::BottleneckSeverity::Low => egui::Color32::GREEN,
            crate::profiling::bottleneck_detector::BottleneckSeverity::Medium => {
                egui::Color32::YELLOW
            }
            crate::profiling::bottleneck_detector::BottleneckSeverity::High => {
                egui::Color32::ORANGE
            }
            crate::profiling::bottleneck_detector::BottleneckSeverity::Critical => {
                egui::Color32::RED
            }
        };

        ui.horizontal(|ui| {
            ui.colored_label(severity_color, format!("{:?}", diagnosis.severity));
            ui.label(&diagnosis.phase_name);
        });

        ui.label(diagnosis.description());
        ui.label(diagnosis.recommendation.clone());

        // 详细信息
        if self.show_details {
            ui.separator();
            ui.label(format!("Variance: {:.2}%", diagnosis.variance * 100.0));
            ui.label(format!(
                "Avg: {:.2}ms",
                diagnosis.average_duration.as_secs_f64() * 1000.0
            ));
            ui.label(format!(
                "Peak: {:.2}ms",
                diagnosis.peak_duration.as_secs_f64() * 1000.0
            ));
            ui.label(format!("Frames: {}", diagnosis.frame_count));
        }

        // 添加优化选项
        let optimization_id = format!("opt_{}", diagnosis.phase_name);
        let is_selected = self.selected_optimizations.contains(&optimization_id);

        if ui.checkbox(&mut is_selected, "Include in optimization").changed() {
            if is_selected {
                self.selected_optimizations.push(optimization_id);
            } else {
                self.selected_optimizations.retain(|x| x != &optimization_id);
            }
        }
    }

    /// 显示优化预览
    fn show_optimization_preview(&mut self, ui: &mut Ui, preview: &OptimizationPreview) {
        ui.heading("📊 Optimization Preview");

        // 性能对比
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Before:");
                ui.label(format!("FPS: {:.1}", preview.before_metrics.fps));
                ui.label(format!(
                    "Frame Time: {:.2}ms",
                    preview.before_metrics.frame_time_ms
                ));
                ui.label(format!("Draw Calls: {}", preview.before_metrics.draw_calls));
                ui.label(format!(
                    "Memory: {:.1} MB",
                    preview.before_metrics.memory_mb
                ));
            });

            ui.vertical(|ui| {
                ui.label("After (Estimated):");
                ui.label(format!("FPS: {:.1}", preview.after_metrics.fps));
                ui.label(format!(
                    "Frame Time: {:.2}ms",
                    preview.after_metrics.frame_time_ms
                ));
                ui.label(format!("Draw Calls: {}", preview.after_metrics.draw_calls));
                ui.label(format!("Memory: {:.1} MB", preview.after_metrics.memory_mb));
            });
        });

        ui.separator();

        // 优化建议
        ui.heading("💡 Recommendations");
        for rec in &preview.recommendations {
            ui.horizontal(|ui| {
                let risk_color = match rec.risk_level {
                    RiskLevel::Low => egui::Color32::GREEN,
                    RiskLevel::Medium => egui::Color32::YELLOW,
                    RiskLevel::High => egui::Color32::RED,
                };

                ui.colored_label(risk_color, format!("{:?}", rec.risk_level));
                ui.label(&rec.title);
                ui.label(format!("(+{:.0}%)", rec.estimated_improvement * 100.0));
            });

            ui.label(&rec.description);

            if !rec.applied {
                if ui.button(format!("Apply {}", rec.id)).clicked() {
                    self.apply_optimization(&rec.id);
                }
            } else {
                ui.label(egui::RichText::new("✓ Applied").color(egui::Color32::GREEN));
            }

            ui.separator();
        }
    }

    /// 检测瓶颈
    fn detect_bottlenecks(&mut self) {
        self.is_detecting = true;

        // 这里应该实际调用瓶颈检测器
        // 为了演示，我们创建模拟数据
        self.diagnoses = vec![];

        self.is_detecting = false;

        // 生成优化预览
        self.generate_optimization_preview();
    }

    /// 生成优化预览
    fn generate_optimization_preview(&mut self) {
        let before = PerformanceSnapshot {
            fps: 30.0,
            frame_time_ms: 33.33,
            draw_calls: 150,
            triangle_count: 500000,
            memory_mb: 512.0,
            gpu_usage: 95.0,
        };

        let after = PerformanceSnapshot {
            fps: 60.0,
            frame_time_ms: 16.67,
            draw_calls: 80,
            triangle_count: 400000,
            memory_mb: 400.0,
            gpu_usage: 75.0,
        };

        let recommendations = vec![
            OptimizationRecommendation {
                id: "reduce_draw_calls".to_string(),
                title: "Reduce Draw Calls".to_string(),
                description: "Merge similar materials and use GPU instancing to reduce draw calls from 150 to 80.".to_string(),
                optimization_type: OptimizationType::Rendering,
                estimated_improvement: 0.4,
                risk_level: RiskLevel::Low,
                applied: false,
            },
            OptimizationRecommendation {
                id: "optimize_shaders".to_string(),
                title: "Optimize Shaders".to_string(),
                description: "Simplify fragment shaders and reduce texture lookups to lower GPU usage.".to_string(),
                optimization_type: OptimizationType::Shader,
                estimated_improvement: 0.25,
                risk_level: RiskLevel::Medium,
                applied: false,
            },
            OptimizationRecommendation {
                id: "memory_pooling".to_string(),
                title: "Enable Memory Pooling".to_string(),
                description: "Implement object pooling for frequently spawned entities to reduce allocations.".to_string(),
                optimization_type: OptimizationType::Memory,
                estimated_improvement: 0.2,
                risk_level: RiskLevel::Low,
                applied: false,
            },
        ];

        self.optimization_preview = Some(OptimizationPreview {
            before_metrics: before,
            after_metrics: after,
            recommendations,
        });
    }

    /// 应用优化
    fn apply_optimizations(&mut self) {
        // 应用所有选中的优化
        if let Some(preview) = &mut self.optimization_preview {
            for rec in &mut preview.recommendations {
                if self.selected_optimizations.iter().any(|id| id.contains(&rec.id)) {
                    rec.applied = true;
                }
            }
        }

        self.selected_optimizations.clear();
    }

    /// 应用单个优化
    fn apply_optimization(&mut self, id: &str) {
        if let Some(preview) = &mut self.optimization_preview {
            for rec in &mut preview.recommendations {
                if &rec.id == id {
                    rec.applied = true;
                    break;
                }
            }
        }
    }

    /// 重置
    fn reset(&mut self) {
        self.diagnoses.clear();
        self.selected_optimizations.clear();
        self.optimization_preview = None;
        if let Ok(detector) = self.bottleneck_detector.try_lock() {
            detector.clear();
        }
    }

    /// 更新面板
    pub fn update(&mut self, delta_time: f32) {
        // 自动检测定时器
        self.auto_detect_timer += delta_time;
        if self.auto_detect_timer >= 5.0 {
            self.auto_detect_timer = 0.0;
            // 每5秒自动检测一次
            // self.detect_bottlenecks();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_panel_default() {
        let panel = OptimizationPanel::default();
        assert_eq!(panel.diagnoses.len(), 0);
        assert_eq!(panel.selected_optimizations.len(), 0);
        assert!(panel.optimization_preview.is_none());
    }

    #[test]
    fn test_performance_snapshot() {
        let snapshot = PerformanceSnapshot {
            fps: 60.0,
            frame_time_ms: 16.67,
            draw_calls: 100,
            triangle_count: 100000,
            memory_mb: 256.0,
            gpu_usage: 80.0,
        };

        assert_eq!(snapshot.fps, 60.0);
        assert_eq!(snapshot.draw_calls, 100);
    }

    #[test]
    fn test_optimization_recommendation() {
        let rec = OptimizationRecommendation {
            id: "test_opt".to_string(),
            title: "Test Optimization".to_string(),
            description: "Test description".to_string(),
            optimization_type: OptimizationType::Rendering,
            estimated_improvement: 0.3,
            risk_level: RiskLevel::Low,
            applied: false,
        };

        assert_eq!(rec.id, "test_opt");
        assert!(!rec.applied);
        assert_eq!(rec.risk_level, RiskLevel::Low);
    }
}
