//! 资源面板
//!
//! 显示资源加载状态、内存使用和资源统计信息。

use super::Panel;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// 资源面板
///
/// 显示资源加载状态和管理信息。
pub struct ResourcePanel {
    /// 是否显示面板
    visible: bool,
    /// 资源统计
    resource_stats: HashMap<String, ResourceStats>,
    /// 搜索过滤文本
    filter_text: String,
    /// 选中的资源类型
    selected_type: Option<String>,
}

/// 资源统计信息
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// 资源类型
    pub resource_type: String,
    /// 总数量
    pub total_count: usize,
    /// 已加载数量
    pub loaded_count: usize,
    /// 失败数量
    pub failed_count: usize,
    /// 总大小（字节）
    pub total_size: u64,
    /// 加载中的数量
    pub loading_count: usize,
}

impl ResourcePanel {
    /// 创建新的资源面板
    pub fn new() -> Self {
        Self {
            visible: true,
            resource_stats: HashMap::new(),
            filter_text: String::new(),
            selected_type: None,
        }
    }

    /// 更新资源统计
    pub fn update_stats(&mut self, resource_type: String, stats: ResourceStats) {
        self.resource_stats.insert(resource_type.clone(), stats);
    }

    /// 显示面板
    pub fn show(&mut self, ctx: &egui::Context, _world: &World) {
        if !self.visible {
            return;
        }

        egui::Window::new("Resources")
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                // 搜索框
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter_text);
                });

                ui.separator();

                // 资源类型选择
                if !self.resource_stats.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Resource Type:");
                        for resource_type in self.resource_stats.keys() {
                            if self
                                .selected_type
                                .as_ref()
                                .map_or(false, |t| t == resource_type)
                            {
                                ui.selectable_label(true, resource_type);
                            } else if ui.button(resource_type).clicked() {
                                self.selected_type = Some(resource_type.clone());
                            }
                        }
                    });
                }

                ui.separator();

                // 显示资源统计
                if let Some(selected_type) = &self.selected_type {
                    if let Some(stats) = self.resource_stats.get(selected_type).cloned() {
                        self.show_resource_stats(ui, &stats);
                    }
                } else {
                    // 显示所有资源的汇总
                    self.show_all_resources(ui);
                }

                ui.separator();

                // 总体统计
                self.show_overall_stats(ui);
            });
    }

    /// 显示单个资源类型的统计
    fn show_resource_stats(&mut self, ui: &mut egui::Ui, stats: &ResourceStats) {
        ui.heading(format!("{} Resources", stats.resource_type));

        ui.separator();

        egui::Grid::new("resource_stats")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Total:");
                ui.label(format!("{}", stats.total_count));
                ui.end_row();

                ui.label("Loaded:");
                ui.colored_label(egui::Color32::GREEN, format!("{}", stats.loaded_count));
                ui.end_row();

                ui.label("Loading:");
                ui.colored_label(egui::Color32::YELLOW, format!("{}", stats.loading_count));
                ui.end_row();

                ui.label("Failed:");
                ui.colored_label(egui::Color32::RED, format!("{}", stats.failed_count));
                ui.end_row();

                ui.label("Total Size:");
                ui.label(format!("{} MB", stats.total_size / 1024 / 1024));
                ui.end_row();
            });

        // 加载进度条
        let progress = if stats.total_count > 0 {
            stats.loaded_count as f32 / stats.total_count as f32
        } else {
            0.0
        };

        ui.separator();
        ui.label("Loading Progress:");
        ui.add(egui::ProgressBar::new(progress).show_percentage());
    }

    /// 显示所有资源的汇总
    fn show_all_resources(&mut self, ui: &mut egui::Ui) {
        ui.heading("All Resources");

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 先收集需要显示的资源（避免借用冲突）
                let resources_to_show: Vec<_> = self.resource_stats
                    .iter()
                    .filter(|(rt, _)| {
                        self.filter_text.is_empty() || rt.contains(&self.filter_text)
                    })
                    .map(|(rt, stats)| (rt.clone(), stats.clone()))
                    .collect();

                for (resource_type, stats) in resources_to_show {
                    egui::CollapsingHeader::new(resource_type)
                        .default_open(false)
                        .show(ui, |ui| {
                            self.show_resource_stats(ui, &stats);
                        });
                }
            });
    }

    /// 显示总体统计
    fn show_overall_stats(&mut self, ui: &mut egui::Ui) {
        ui.heading("Overall Statistics");

        let total_count: usize = self.resource_stats.values().map(|s| s.total_count).sum();
        let total_loaded: usize = self.resource_stats.values().map(|s| s.loaded_count).sum();
        let total_failed: usize = self.resource_stats.values().map(|s| s.failed_count).sum();
        let total_loading: usize = self.resource_stats.values().map(|s| s.loading_count).sum();
        let total_size: u64 = self.resource_stats.values().map(|s| s.total_size).sum();

        egui::Grid::new("overall_stats")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Total Resources:");
                ui.label(format!("{}", total_count));
                ui.end_row();

                ui.label("Total Loaded:");
                ui.colored_label(egui::Color32::GREEN, format!("{}", total_loaded));
                ui.end_row();

                ui.label("Total Loading:");
                ui.colored_label(egui::Color32::YELLOW, format!("{}", total_loading));
                ui.end_row();

                ui.label("Total Failed:");
                ui.colored_label(egui::Color32::RED, format!("{}", total_failed));
                ui.end_row();

                ui.label("Total Memory:");
                ui.label(format!("{} MB", total_size / 1024 / 1024));
                ui.end_row();
            });

        // 总体进度
        let progress = if total_count > 0 {
            total_loaded as f32 / total_count as f32
        } else {
            0.0
        };

        ui.separator();
        ui.label("Overall Loading Progress:");
        ui.add(egui::ProgressBar::new(progress).show_percentage());
    }

    /// 清除统计
    pub fn clear(&mut self) {
        self.resource_stats.clear();
        self.selected_type = None;
    }

    /// 获取资源类型数量
    pub fn resource_type_count(&self) -> usize {
        self.resource_stats.len()
    }
}

impl Default for ResourcePanel {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源类型枚举（常见资源类型）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Texture,
    Mesh,
    Shader,
    Audio,
    Font,
    Model,
    Animation,
    Script,
    Config,
    Other(String),
}

impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::Texture => "Texture",
            ResourceType::Mesh => "Mesh",
            ResourceType::Shader => "Shader",
            ResourceType::Audio => "Audio",
            ResourceType::Font => "Font",
            ResourceType::Model => "Model",
            ResourceType::Animation => "Animation",
            ResourceType::Script => "Script",
            ResourceType::Config => "Config",
            ResourceType::Other(s) => s,
        }
    }
}

/// 资源详细信息
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// 资源路径
    pub path: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源大小（字节）
    pub size: u64,
    /// 是否已加载
    pub loaded: bool,
    /// 加载进度（0.0 - 1.0）
    pub load_progress: f32,
    /// 是否加载失败
    pub failed: bool,
    /// 加载时间戳
    pub load_timestamp: Option<chrono::DateTime<chrono::Local>>,
    /// 引用计数
    pub ref_count: usize,
}

impl ResourceInfo {
    /// 创建新的资源信息
    pub fn new(path: String, resource_type: ResourceType) -> Self {
        Self {
            path,
            resource_type,
            size: 0,
            loaded: false,
            load_progress: 0.0,
            failed: false,
            load_timestamp: None,
            ref_count: 0,
        }
    }
}
